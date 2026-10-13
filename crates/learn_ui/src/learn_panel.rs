use std::path::PathBuf;

use gpui::{
    Action, App, AsyncWindowContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    PromptLevel, Render, Subscription, Task, WeakEntity, Window, px,
};
use learn::{
    CheckOutcome, Exercise, ExerciseStatus, Progress, TrackWorkspace,
    runner::{self, DEFAULT_TIMEOUT},
};
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownStyle};
use project::Project;
use task::{RevealStrategy, SpawnInTerminal, TaskId};
use ui::{Divider, ListItem, ProgressBar, prelude::*};
use util::ResultExt as _;
use workspace::{
    OpenOptions, OpenVisible, Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};

use crate::ToggleFocus;

const LEARN_PANEL_KEY: &str = "LearnPanel";

struct SubmitState {
    outcome: CheckOutcome,
    xp_awarded: u32,
}

/// Left-dock panel with the chapter/exercise tree and the exercise view (AC3, AC4).
pub struct LearnPanel {
    workspace: WeakEntity<Workspace>,
    project: Entity<Project>,
    focus_handle: FocusHandle,
    position: DockPosition,
    track: Option<TrackWorkspace>,
    progress: Progress,
    selected: Option<String>,
    markdown: Option<Entity<Markdown>>,
    submit: Option<SubmitState>,
    checking: bool,
    auto_opened: bool,
    _subscriptions: Vec<Subscription>,
}

impl LearnPanel {
    pub fn load(
        workspace: WeakEntity<Workspace>,
        cx: AsyncWindowContext,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        cx.spawn(async move |cx| {
            workspace.update_in(cx, |workspace, window, cx| {
                let weak = cx.entity().downgrade();
                let project = workspace.project().clone();
                cx.new(|cx| LearnPanel::new(weak, project, window, cx))
            })
        })
    }

    fn new(
        workspace: WeakEntity<Workspace>,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.subscribe_in(&project, window, |this, _, event, window, cx| {
            if matches!(
                event,
                project::Event::WorktreeAdded(_) | project::Event::WorktreeRemoved(_)
            ) {
                this.detect_track(window, cx);
            }
        });
        let mut this = Self {
            workspace,
            project,
            focus_handle: cx.focus_handle(),
            position: DockPosition::Left,
            track: None,
            progress: Progress::default(),
            selected: None,
            markdown: None,
            submit: None,
            checking: false,
            auto_opened: false,
            _subscriptions: vec![subscription],
        };
        this.detect_track(window, cx);
        this
    }

    /// Whether this workspace is a LearnZed track workspace.
    pub fn is_track(&self) -> bool {
        self.track.is_some()
    }

    fn track_root(&self, cx: &App) -> Option<PathBuf> {
        self.project
            .read(cx)
            .visible_worktrees(cx)
            .map(|wt| wt.read(cx).abs_path().to_path_buf())
            .find(|root| TrackWorkspace::is_track_workspace(root))
    }

    fn detect_track(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let root = self.track_root(cx);
        if root.as_deref() == self.track.as_ref().map(|t| t.root.as_path()) {
            return;
        }
        self.track = root.and_then(|root| TrackWorkspace::open(&root).log_err());
        self.submit = None;
        self.selected = None;
        self.markdown = None;
        if let Some(track) = &self.track {
            self.progress = track.load_progress();
            if !self.auto_opened {
                self.auto_opened = true;
                let resume = self
                    .progress
                    .resume_exercise(&track.track)
                    .map(|e| e.id().to_string());
                cx.defer_in(window, move |this, window, cx| {
                    this.workspace
                        .update(cx, |workspace, cx| {
                            workspace.focus_panel::<LearnPanel>(window, cx)
                        })
                        .ok();
                    if let Some(id) = resume {
                        this.select_exercise(&id, window, cx);
                    }
                });
            }
        }
        cx.notify();
    }

    fn exercise(&self, id: &str) -> Option<Exercise> {
        self.track.as_ref()?.track.exercise(id).cloned()
    }

    fn selected_exercise(&self) -> Option<Exercise> {
        self.exercise(self.selected.as_deref()?)
    }

    fn save_progress(&self) {
        if let Some(track) = &self.track {
            track.save_progress(&self.progress).log_err();
        }
    }

    fn rebuild_markdown(&mut self, cx: &mut Context<Self>) {
        let Some(exercise) = self.selected_exercise() else {
            self.markdown = None;
            return;
        };
        let hints_used = self.progress.exercise(exercise.id()).hints_used as usize;
        let mut source = format!(
            "{}\n\n#### Instructions · {} XP\n\n{}\n",
            exercise.context.trim(),
            exercise.manifest.xp,
            exercise.instructions.trim()
        );
        for (ix, hint) in exercise.hints.iter().take(hints_used).enumerate() {
            source.push_str(&format!("\n#### Hint {}\n\n{}\n", ix + 1, hint));
        }
        let languages = self.project.read(cx).languages().clone();
        self.markdown = Some(cx.new(|cx| Markdown::new(source.into(), Some(languages), None, cx)));
    }

    fn select_exercise(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(track), Some(exercise)) = (self.track.clone(), self.exercise(id)) else {
            return;
        };
        if !self.progress.is_unlocked(&track.track, id) {
            return;
        }
        self.selected = Some(id.to_string());
        self.submit = None;
        self.progress.mark_opened(id);
        self.save_progress();
        self.rebuild_markdown(cx);
        let path = track.user_file(&exercise);
        self.workspace
            .update(cx, |workspace, cx| {
                workspace
                    .open_abs_path(
                        path,
                        OpenOptions {
                            visible: Some(OpenVisible::None),
                            ..Default::default()
                        },
                        window,
                        cx,
                    )
                    .detach_and_log_err(cx);
            })
            .ok();
        cx.notify();
    }

    /// Save the learner's file if it has unsaved edits.
    fn save_user_file(&self, cx: &mut Context<Self>) -> Task<anyhow::Result<()>> {
        let (Some(track), Some(exercise)) = (&self.track, self.selected_exercise()) else {
            return Task::ready(Ok(()));
        };
        let path = track.user_file(&exercise);
        let project = self.project.clone();
        let open = project.update(cx, |project, cx| project.open_local_buffer(&path, cx));
        cx.spawn(async move |_, cx| {
            let buffer = open.await?;
            let dirty = buffer.read_with(cx, |b, _| b.is_dirty());
            if dirty {
                project
                    .update(cx, |p, cx| p.save_buffer(buffer, cx))
                    .await?;
            }
            Ok(())
        })
    }

    fn run_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(track), Some(exercise)) = (self.track.clone(), self.selected_exercise()) else {
            return;
        };
        let save = self.save_user_file(cx);
        cx.spawn_in(window, async move |this, cx| {
            save.await.log_err();
            this.update_in(cx, |this, window, cx| {
                let (program, args) = runner::run_command(&track, &exercise);
                let program = program.to_string_lossy().into_owned();
                let label = format!("Learn: Run {}", exercise.manifest.title);
                let spawn = SpawnInTerminal {
                    id: TaskId("learnzed-run".into()),
                    full_label: label.clone(),
                    label,
                    command_label: format!("python {}", exercise.user_file().display()),
                    command: Some(program),
                    args,
                    cwd: Some(track.root.join(&exercise.rel_dir)),
                    use_new_terminal: false,
                    allow_concurrent_runs: false,
                    reveal: RevealStrategy::NoFocus,
                    show_summary: true,
                    show_command: true,
                    ..Default::default()
                };
                this.workspace
                    .update(cx, |workspace, cx| {
                        workspace.spawn_in_terminal(spawn, window, cx).detach();
                    })
                    .ok();
            })
            .ok();
        })
        .detach();
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(track), Some(exercise)) = (self.track.clone(), self.selected_exercise()) else {
            return;
        };
        if self.checking {
            return;
        }
        self.checking = true;
        self.submit = None;
        cx.notify();
        let save = self.save_user_file(cx);
        cx.spawn_in(window, async move |this, cx| {
            save.await.log_err();
            let ex = exercise.clone();
            let outcome = cx
                .background_spawn(
                    async move { runner::check_exercise(&track, &ex, DEFAULT_TIMEOUT) },
                )
                .await;
            this.update(cx, |this, cx| {
                this.checking = false;
                let passed = outcome.as_ref().is_ok_and(|r| r.passed);
                let xp_awarded = this.progress.record_submission(
                    exercise.id(),
                    passed,
                    exercise.manifest.xp,
                    exercise.manifest.hint_xp_cost,
                );
                this.save_progress();
                this.submit = Some(SubmitState {
                    outcome,
                    xp_awarded,
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn hint(&mut self, cx: &mut Context<Self>) {
        let Some(exercise) = self.selected_exercise() else {
            return;
        };
        self.progress.use_hint(exercise.id(), exercise.hints.len());
        self.save_progress();
        self.rebuild_markdown(cx);
        cx.notify();
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(track), Some(exercise)) = (self.track.clone(), self.selected_exercise()) else {
            return;
        };
        let answer = window.prompt(
            PromptLevel::Warning,
            "Reset this exercise?",
            Some("Your code will be replaced with the starter code."),
            &["Reset", "Cancel"],
            cx,
        );
        let project = self.project.clone();
        cx.spawn_in(window, async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            track.reset_exercise(&exercise).log_err();
            let path = track.user_file(&exercise);
            if let Ok(buffer) = project
                .update(cx, |p, cx| p.open_local_buffer(&path, cx))
                .await
            {
                let reload = buffer.update(cx, |b, cx| b.reload(cx));
                reload.await.ok();
            }
            this.update(cx, |this, cx| {
                this.submit = None;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = self.track.as_ref().and_then(|t| {
            t.track
                .next_exercise(self.selected.as_deref()?)
                .map(|e| e.id().to_string())
        });
        if let Some(id) = next {
            self.select_exercise(&id, window, cx);
        }
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .p_4()
            .gap_2()
            .child(Label::new("This folder is not a learning track."))
            .child(
                Button::new("open-picker", "Choose a track…").on_click(cx.listener(
                    |_, _, window, cx| {
                        window.dispatch_action(crate::OpenTrackPicker.boxed_clone(), cx)
                    },
                )),
            )
            .into_any_element()
    }

    fn render_tree(&self, track: &TrackWorkspace, cx: &mut Context<Self>) -> AnyElement {
        let mut items = Vec::new();
        for (cix, chapter) in track.track.chapters.iter().enumerate() {
            items.push(
                h_flex()
                    .px_2()
                    .pt_2()
                    .child(
                        Label::new(format!("{}. {}", cix + 1, chapter.manifest.title))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .into_any_element(),
            );
            for exercise in &chapter.exercises {
                let id = exercise.id().to_string();
                let status = self.progress.status(&id);
                let unlocked = self.progress.is_unlocked(&track.track, &id);
                let (icon, color) = match (status, unlocked) {
                    (ExerciseStatus::Passed, _) => (IconName::Check, Color::Success),
                    (ExerciseStatus::InProgress, _) => (IconName::Circle, Color::Accent),
                    (_, false) => (IconName::Lock, Color::Disabled),
                    _ => (IconName::Circle, Color::Muted),
                };
                let selected = self.selected.as_deref() == Some(id.as_str());
                items.push(
                    ListItem::new(SharedString::from(format!("ex-{id}")))
                        .toggle_state(selected)
                        .disabled(!unlocked)
                        .start_slot(Icon::new(icon).color(color).size(IconSize::Small))
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .child(Label::new(exercise.manifest.title.clone()).color(
                                    if unlocked {
                                        Color::Default
                                    } else {
                                        Color::Disabled
                                    },
                                ))
                                .child(
                                    Label::new(format!("{} XP", exercise.manifest.xp))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.select_exercise(&id, window, cx)
                        }))
                        .into_any_element(),
                );
            }
        }
        v_flex().gap_0p5().children(items).into_any_element()
    }

    fn render_result(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.checking {
            return Some(
                Label::new("Checking your code…")
                    .color(Color::Muted)
                    .into_any_element(),
            );
        }
        let state = self.submit.as_ref()?;
        let colors = cx.theme().colors();
        let el = match &state.outcome {
            Ok(result) => {
                let header = if result.passed {
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(Label::new("Great work! All checks passed.").color(Color::Success))
                        .when(state.xp_awarded > 0, |el| {
                            el.child(
                                Label::new(format!("+{} XP", state.xp_awarded))
                                    .color(Color::Accent),
                            )
                        })
                } else {
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::XCircle).color(Color::Error))
                        .child(Label::new("Incorrect submission").color(Color::Error))
                };
                let checks = result.checks.iter().filter(|c| !c.passed).map(|c| {
                    h_flex()
                        .gap_2()
                        .items_start()
                        .child(
                            Icon::new(IconName::Close)
                                .size(IconSize::Small)
                                .color(Color::Error),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Label::new(c.message.clone()).size(LabelSize::Small)),
                        )
                });
                let output = (!result.stdout.trim().is_empty()).then(|| {
                    v_flex()
                        .gap_1()
                        .child(
                            Label::new("Output")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                        .child(
                            div()
                                .p_2()
                                .rounded_sm()
                                .bg(colors.editor_background)
                                .font_buffer(cx)
                                .text_xs()
                                .child(result.stdout.trim().to_string()),
                        )
                });
                v_flex()
                    .gap_2()
                    .child(header)
                    .children(checks)
                    .children(output)
            }
            Err(err) => v_flex()
                .gap_2()
                .child(Label::new("Could not check your code").color(Color::Error))
                .child(Label::new(err.to_string()).size(LabelSize::Small)),
        };
        Some(
            el.p_2()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .into_any_element(),
        )
    }

    fn render_exercise(
        &self,
        exercise: &Exercise,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let p = self.progress.exercise(exercise.id());
        let hints_left = exercise.hints.len().saturating_sub(p.hints_used as usize);
        let passed = p.status == ExerciseStatus::Passed;
        let has_next = self
            .track
            .as_ref()
            .and_then(|t| t.track.next_exercise(exercise.id()))
            .is_some();
        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(Headline::new(exercise.manifest.title.clone()).size(HeadlineSize::Small))
                    .when(passed, |el| {
                        el.child(Icon::new(IconName::Check).color(Color::Success))
                    }),
            )
            .children(self.markdown.clone().map(|md| {
                MarkdownElement::new(md, MarkdownStyle::themed(MarkdownFont::Agent, window, cx))
            }))
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .child(
                        Button::new("learn-run", "Run Code")
                            .start_icon(Icon::new(IconName::PlayOutlined))
                            .on_click(cx.listener(|this, _, window, cx| this.run_code(window, cx))),
                    )
                    .child(
                        Button::new("learn-submit", "Submit Answer")
                            .style(ButtonStyle::Filled)
                            .disabled(self.checking)
                            .start_icon(Icon::new(IconName::Check))
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    )
                    .child(
                        Button::new(
                            "learn-hint",
                            format!("Hint (-{} XP)", exercise.manifest.hint_xp_cost),
                        )
                        .disabled(hints_left == 0 || passed)
                        .start_icon(Icon::new(IconName::Sparkle))
                        .on_click(cx.listener(|this, _, _, cx| this.hint(cx))),
                    )
                    .child(
                        Button::new("learn-reset", "Reset")
                            .start_icon(Icon::new(IconName::RotateCcw))
                            .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
                    )
                    .when(has_next, |el| {
                        el.child(
                            Button::new("learn-next", "Next")
                                .disabled(!passed)
                                .end_icon(Icon::new(IconName::ArrowRight))
                                .on_click(cx.listener(|this, _, window, cx| this.next(window, cx))),
                        )
                    }),
            )
            .children(self.render_result(cx))
            .into_any_element()
    }
}

impl Render for LearnPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.track.clone() {
            None => self.render_empty(cx),
            Some(track) => {
                let percent = self.progress.percent(&track.track);
                let exercise = self.selected_exercise();
                v_flex()
                    .id("learn-panel-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .p_2()
                    .gap_2()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                Headline::new(track.track.manifest.title.clone())
                                    .size(HeadlineSize::XSmall),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(div().flex_1().child(ProgressBar::new(
                                        "learn-progress",
                                        percent as f32,
                                        100.,
                                        cx,
                                    )))
                                    .child(
                                        Label::new(format!(
                                            "{percent}% · {} XP",
                                            self.progress.total_xp()
                                        ))
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                    ),
                            ),
                    )
                    .child(self.render_tree(&track, cx))
                    .child(Divider::horizontal())
                    .children(exercise.map(|e| self.render_exercise(&e, window, cx)))
                    .into_any_element()
            }
        };
        v_flex()
            .key_context("LearnPanel")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(content)
    }
}

impl Focusable for LearnPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for LearnPanel {}

impl Panel for LearnPanel {
    fn persistent_name() -> &'static str {
        "Learn Panel"
    }

    fn panel_key() -> &'static str {
        LEARN_PANEL_KEY
    }

    fn position(&self, _: &Window, _: &App) -> DockPosition {
        self.position
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }

    fn set_position(&mut self, position: DockPosition, _: &mut Window, cx: &mut Context<Self>) {
        self.position = position;
        cx.notify();
    }

    fn default_size(&self, _: &Window, _: &App) -> Pixels {
        px(420.)
    }

    fn icon(&self, _: &Window, _: &App) -> Option<IconName> {
        self.is_track().then_some(IconName::Book)
    }

    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some("Learn")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleFocus)
    }

    fn starts_open(&self, _: &Window, _: &App) -> bool {
        self.is_track()
    }

    fn activation_priority(&self) -> u32 {
        10
    }
}
