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
use ui::{CommonAnimationExt as _, ListItem, ProgressBar, TintColor, prelude::*};
use util::ResultExt as _;
use workspace::{
    OpenOptions, OpenVisible, Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};

use crate::ToggleFocus;

const LEARN_PANEL_KEY: &str = "LearnPanel";

pub(crate) struct SubmitState {
    outcome: CheckOutcome,
    xp_awarded: u32,
    skill_earned: Option<String>,
}

/// Left-dock panel with the chapter/exercise tree and the exercise view (AC3, AC4).
pub struct LearnPanel {
    pub(crate) workspace: WeakEntity<Workspace>,
    pub(crate) project: Option<Entity<Project>>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) position: DockPosition,
    pub(crate) track: Option<TrackWorkspace>,
    pub(crate) progress: Progress,
    pub(crate) selected: Option<String>,
    pub(crate) markdown: Option<Entity<Markdown>>,
    pub(crate) submit: Option<SubmitState>,
    pub(crate) checking: bool,
    pub(crate) auto_opened: bool,
    pub(crate) show_ai_connect: bool,
    pub(crate) _subscriptions: Vec<Subscription>,
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
                cx.new(|cx| LearnPanel::new(weak, Some(project), window, cx))
            })
        })
    }

    pub(crate) fn new(
        workspace: WeakEntity<Workspace>,
        project: Option<Entity<Project>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = project.as_ref().map(|project| {
            cx.subscribe_in(project, window, |this, _, event, window, cx| {
                if matches!(
                    event,
                    project::Event::WorktreeAdded(_) | project::Event::WorktreeRemoved(_)
                ) {
                    this.detect_track(window, cx);
                }
            })
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
            show_ai_connect: false,
            _subscriptions: subscription.into_iter().collect(),
        };
        this.detect_track(window, cx);
        this
    }

    /// Whether this workspace is a LearnZed track workspace.
    pub fn is_track(&self) -> bool {
        self.track.is_some()
    }

    fn track_root(&self, cx: &App) -> Option<PathBuf> {
        let project = self.project.as_ref()?;
        project
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
            "{}\n\n#### Instructions\n\n{}\n",
            exercise.context.trim(),
            exercise.instructions.trim()
        );
        for (ix, hint) in exercise.hints.iter().take(hints_used).enumerate() {
            source.push_str(&format!("\n#### Hint {}\n\n{}\n", ix + 1, hint));
        }
        let languages = self.project.as_ref().map(|p| p.read(cx).languages().clone());
        self.markdown = Some(cx.new(|cx| Markdown::new(source.into(), languages, None, cx)));
    }

    pub(crate) fn select_exercise(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
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
        self.show_ai_connect = false;
        let path = track.user_file(&exercise);
        let layout = exercise.manifest.layout;
        self.workspace
            .update(cx, |workspace, cx| {
                apply_layout(workspace, layout, window, cx);
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

    #[cfg(test)]
    pub(crate) fn set_track(
        &mut self,
        track: TrackWorkspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.track = Some(track);
        self.progress = self.track.as_ref().unwrap().load_progress();
        if let Some(first) = self.track.as_ref().and_then(|t| t.track.exercises().next()) {
            let id = first.id().to_string();
            self.select_exercise(&id, window, cx);
        }
    }

    /// Save the learner's file if it has unsaved edits.
    fn save_user_file(&self, cx: &mut Context<Self>) -> Task<anyhow::Result<()>> {
        let (Some(track), Some(exercise), Some(project)) =
            (&self.track, self.selected_exercise(), &self.project)
        else {
            return Task::ready(Ok(()));
        };
        let path = track.user_file(&exercise);
        let project = project.clone();
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
                let track = this.track.clone();
                let skill_earned = track.and_then(|t| {
                    this.progress
                        .award_skill_if_complete(&t.track)
                        .then(|| t.track.manifest.skill.clone())
                        .flatten()
                });
                this.save_progress();
                this.submit = Some(SubmitState {
                    outcome,
                    xp_awarded,
                    skill_earned,
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn ai_prompt(&self, exercise: &Exercise) -> String {
        let code = self
            .track
            .as_ref()
            .and_then(|t| std::fs::read_to_string(t.user_file(exercise)).ok())
            .unwrap_or_default();
        let mut prompt = format!(
            "You are a patient programming tutor inside LearnZed. Help me with the exercise below. \
             Do NOT write the full solution or the exact final code. Point out what is wrong or \
             missing, explain the relevant concept, and give me the next small step or a hint.\n\n\
             Exercise: {}\n\nContext:\n{}\n\nInstructions:\n{}\n\nMy current code ({}):\n```{}\n{}\n```\n",
            exercise.manifest.title,
            exercise.context.trim(),
            exercise.instructions.trim(),
            exercise.manifest.file,
            exercise.manifest.language,
            code.trim_end(),
        );
        if let Some(SubmitState {
            outcome: Ok(result),
            ..
        }) = &self.submit
        {
            let failed: Vec<_> = result
                .checks
                .iter()
                .filter(|c| !c.passed)
                .map(|c| format!("- {}", c.message))
                .collect();
            if !failed.is_empty() {
                prompt.push_str(&format!(
                    "\nMy last submission failed these checks:\n{}\n",
                    failed.join("\n")
                ));
            }
        }
        prompt
    }

    /// Whether a language model provider is configured and authenticated.
    fn ai_available(cx: &App) -> bool {
        language_model::LanguageModelRegistry::read_global(cx)
            .default_model()
            .is_some()
    }

    fn ask_ai(&mut self, force: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(exercise) = self.selected_exercise() else {
            return;
        };
        if !force && !Self::ai_available(cx) {
            self.show_ai_connect = true;
            cx.notify();
            return;
        }
        self.show_ai_connect = false;
        let save = self.save_user_file(cx);
        cx.spawn_in(window, async move |this, cx| {
            save.await.log_err();
            // LEARNZED: build the prompt first, then touch the workspace outside our own
            // update so the dock can deactivate this panel without a double lease.
            let Ok((prompt, workspace)) = this.update(cx, |this, cx| {
                cx.notify();
                (this.ai_prompt(&exercise), this.workspace.clone())
            }) else {
                return;
            };
            let Some(workspace) = workspace.upgrade() else {
                return;
            };
            workspace
                .update_in(cx, |workspace, window, cx| {
                    let Some(panel) = workspace.panel::<agent_ui::AgentPanel>(cx) else {
                        return;
                    };
                    workspace.focus_panel::<agent_ui::AgentPanel>(window, cx);
                    let prompt = agent_ui::ExternalSourcePrompt::new(&prompt);
                    panel.update(cx, |panel, cx| {
                        panel.new_agent_thread_with_external_source_prompt(prompt, window, cx)
                    });
                })
                .ok();
        })
        .detach();
    }

    fn render_ai_connect(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = cx.theme().colors();
        v_flex()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface_background)
            .child(
                h_flex()
                    .gap_2()
                    .child(Icon::new(IconName::Sparkle).color(Color::Accent))
                    .child(Label::new("Connect an AI tutor"))
                    .child(div().flex_1())
                    .child(
                        IconButton::new("ai-connect-close", IconName::Close)
                            .icon_size(IconSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_ai_connect = false;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Label::new(
                    "Ask AI needs a language model. Add an API key for a provider \
                     (Anthropic, OpenAI, Ollama, …) in the agent settings.",
                )
                .size(LabelSize::Small),
            )
            .child(
                Label::new(
                    "Prefer your own harness? Connect an external agent such as Claude Code, \
                     Codex or Gemini CLI via the Agent Client Protocol (ACP) — it gets the same \
                     exercise context.",
                )
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
            .child(
                h_flex()
                    .gap_1()
                    .flex_wrap()
                    .child(
                        Button::new("ai-connect-provider", "Connect AI provider")
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    zed_actions::agent::OpenSettings.boxed_clone(),
                                    cx,
                                )
                            }),
                    )
                    .child(
                        Button::new("ai-connect-external", "Use an external agent")
                            .style(ButtonStyle::Subtle)
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    zed_actions::agent::OpenSettings.boxed_clone(),
                                    cx,
                                )
                            }),
                    )
                    .child(
                        Button::new("ai-connect-open", "Open agent panel")
                            .style(ButtonStyle::Subtle)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.ask_ai(true, window, cx)),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn hint(&mut self, cx: &mut Context<Self>) {
        let Some(exercise) = self.selected_exercise() else {
            return;
        };
        self.progress.use_hint(exercise.id(), exercise.hints.len());
        self.save_progress();
        self.rebuild_markdown(cx);
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn reset_exercise_starter(&mut self) -> anyhow::Result<()> {
        let (Some(track), Some(exercise)) = (&self.track, self.selected_exercise()) else {
            anyhow::bail!("no exercise selected");
        };
        track.reset_exercise(&exercise)
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
            if let Some(project) = project {
                if let Ok(buffer) = project
                    .update(cx, |p, cx| p.open_local_buffer(&path, cx))
                    .await
                {
                    let reload = buffer.update(cx, |b, cx| b.reload(cx));
                    reload.await.ok();
                }
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
                div()
                    .px_3()
                    .when(cix > 0, |el| el.pt_3())
                    .pb_1()
                    .child(
                        Label::new(chapter.manifest.title.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .into_any_element(),
            );
            for exercise in &chapter.exercises {
                let id = exercise.id().to_string();
                let status = self.progress.status(&id);
                let unlocked = self.progress.is_unlocked(&track.track, &id);
                let selected = self.selected.as_deref() == Some(id.as_str());
                let (icon, color) = match (status, unlocked) {
                    (ExerciseStatus::Passed, _) => (IconName::Check, Color::Success),
                    (_, false) => (IconName::Lock, Color::Muted),
                    (ExerciseStatus::InProgress, _) => (IconName::Circle, Color::Accent),
                    _ => (IconName::Circle, Color::Muted),
                };
                items.push(
                    div()
                        .px_1()
                        .child(
                            ListItem::new(SharedString::from(format!("ex-{id}")))
                                .toggle_state(selected)
                                .disabled(!unlocked)
                                .start_slot(Icon::new(icon).color(color).size(IconSize::Small))
                                .end_slot(
                                    Label::new(format!("{} XP", exercise.manifest.xp))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                                .child(Label::new(exercise.manifest.title.clone()).color(
                                    if unlocked {
                                        Color::Default
                                    } else {
                                        Color::Muted
                                    },
                                ))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_exercise(&id, window, cx)
                                })),
                        )
                        .into_any_element(),
                );
            }
        }
        v_flex().py_2().children(items).into_any_element()
    }

    fn render_result(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let status = cx.theme().status().clone();
        let colors = cx.theme().colors().clone();
        if self.checking {
            return Some(
                h_flex()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border_variant)
                    .child(
                        Icon::new(IconName::LoadCircle)
                            .size(IconSize::Small)
                            .color(Color::Muted)
                            .with_rotate_animation(2),
                    )
                    .child(Label::new("Checking your code…").color(Color::Muted))
                    .into_any_element(),
            );
        }
        let state = self.submit.as_ref()?;
        let code_block = |text: String| {
            div()
                .w_full()
                .p_2()
                .rounded_sm()
                .bg(colors.editor_background)
                .font_buffer(cx)
                .text_xs()
                .child(text)
        };
        let el = match &state.outcome {
            Ok(result) if result.passed => v_flex()
                .gap_2()
                .bg(status.success_background)
                .border_color(status.success_border)
                .child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(Label::new("Correct — all checks passed").color(Color::Success))
                        .when(state.xp_awarded > 0, |el| {
                            el.child(div().flex_1()).child(
                                Label::new(format!("+{} XP", state.xp_awarded))
                                    .color(Color::Success),
                            )
                        }),
                )
                .when_some(state.skill_earned.clone(), |el, skill| {
                    el.child(
                        h_flex()
                            .gap_2()
                            .child(
                                Icon::new(IconName::Star)
                                    .size(IconSize::Small)
                                    .color(Color::Success),
                            )
                            .child(
                                Label::new(format!(
                                    "Track complete — you earned the {skill} skill"
                                ))
                                .size(LabelSize::Small)
                                .color(Color::Success),
                            ),
                    )
                })
                .when(!result.stdout.trim().is_empty(), |el| {
                    el.child(code_block(result.stdout.trim().to_string()))
                }),
            Ok(result) => {
                let failed: Vec<_> = result.checks.iter().filter(|c| !c.passed).collect();
                v_flex()
                    .gap_2()
                    .bg(status.error_background)
                    .border_color(status.error_border)
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Icon::new(IconName::XCircle).color(Color::Error))
                            .child(Label::new("Not quite yet").color(Color::Error))
                            .child(div().flex_1())
                            .child(
                                Label::new(format!(
                                    "{} of {} checks failed",
                                    failed.len(),
                                    result.checks.len()
                                ))
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                            ),
                    )
                    .children(failed.into_iter().map(|c| {
                        h_flex()
                            .gap_2()
                            .items_start()
                            .child(
                                div().pt_0p5().child(
                                    Icon::new(IconName::Close)
                                        .size(IconSize::XSmall)
                                        .color(Color::Error),
                                ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Label::new(c.message.clone()).size(LabelSize::Small)),
                            )
                    }))
                    .child(
                        Label::new("Fix the items above and submit again, or take a hint.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .when(!result.stdout.trim().is_empty(), |el| {
                        el.child(code_block(result.stdout.trim().to_string()))
                    })
            }
            Err(err) => v_flex()
                .gap_2()
                .bg(status.error_background)
                .border_color(status.error_border)
                .child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::XCircle).color(Color::Error))
                        .child(Label::new("Couldn't check your code").color(Color::Error)),
                )
                .child(
                    div()
                        .min_w_0()
                        .child(Label::new(err.to_string()).size(LabelSize::Small)),
                ),
        };
        Some(el.p_3().rounded_md().border_1().into_any_element())
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
        let primary = if passed && has_next {
            Button::new("learn-next", "Next exercise")
                .style(ButtonStyle::Tinted(TintColor::Accent))
                .full_width()
                .end_icon(Icon::new(IconName::ArrowRight).size(IconSize::Small))
                .on_click(cx.listener(|this, _, window, cx| this.next(window, cx)))
        } else {
            Button::new(
                "learn-submit",
                if self.checking {
                    "Checking…"
                } else {
                    "Submit answer"
                },
            )
            .style(ButtonStyle::Tinted(TintColor::Accent))
            .full_width()
            .disabled(self.checking)
            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)))
        };
        v_flex()
            .px_3()
            .py_4()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new(exercise.manifest.title.clone()).size(HeadlineSize::Small))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Label::new(format!("{} XP", exercise.manifest.xp))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                            .when(passed, |el| {
                                el.child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            Icon::new(IconName::Check)
                                                .size(IconSize::XSmall)
                                                .color(Color::Success),
                                        )
                                        .child(
                                            Label::new(format!("Completed · earned {} XP", p.xp))
                                                .size(LabelSize::Small)
                                                .color(Color::Success),
                                        ),
                                )
                            }),
                    ),
            )
            .children(self.markdown.clone().map(|md| {
                MarkdownElement::new(md, MarkdownStyle::themed(MarkdownFont::Agent, window, cx))
            }))
            .child(
                v_flex().gap_2().child(primary).child(
                    h_flex()
                        .gap_1()
                        .child(
                            Button::new("learn-run", "Run code")
                                .style(ButtonStyle::Subtle)
                                .start_icon(Icon::new(IconName::PlayOutlined).size(IconSize::Small))
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.run_code(window, cx)),
                                ),
                        )
                        .child(
                            Button::new(
                                "learn-hint",
                                format!("Hint · −{} XP", exercise.manifest.hint_xp_cost),
                            )
                            .style(ButtonStyle::Subtle)
                            .disabled(hints_left == 0 || passed)
                            .start_icon(Icon::new(IconName::Info).size(IconSize::Small))
                            .on_click(cx.listener(|this, _, _, cx| this.hint(cx))),
                        )
                        .child(
                            Button::new("learn-ask-ai", "Ask AI")
                                .style(ButtonStyle::Subtle)
                                .start_icon(Icon::new(IconName::Sparkle).size(IconSize::Small))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.ask_ai(false, window, cx)
                                })),
                        )
                        .child(div().flex_1())
                        .child(
                            Button::new("learn-reset", "Reset")
                                .style(ButtonStyle::Subtle)
                                .start_icon(Icon::new(IconName::RotateCcw).size(IconSize::Small))
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.reset(window, cx)),
                                ),
                        ),
                ),
            )
            .when(self.show_ai_connect, |el| {
                el.child(self.render_ai_connect(cx))
            })
            .children(self.render_result(cx))
            .into_any_element()
    }
}

impl Render for LearnPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors().border_variant;
        let content = match self.track.clone() {
            None => self.render_empty(cx),
            Some(track) => {
                let percent = self.progress.percent(&track.track);
                let done = self.progress.passed_count(&track.track);
                let total = track.track.exercises().count();
                let exercise = self.selected_exercise();
                v_flex()
                    .id("learn-panel-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .px_3()
                            .py_3()
                            .gap_2()
                            .border_b_1()
                            .border_color(border)
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        Headline::new(track.track.manifest.title.clone())
                                            .size(HeadlineSize::XSmall),
                                    )
                                    .child(
                                        Label::new(format!("{} XP", self.progress.total_xp()))
                                            .size(LabelSize::Small)
                                            .color(Color::Accent),
                                    ),
                            )
                            .child(ProgressBar::new("learn-progress", percent as f32, 100., cx))
                            .when_some(track.track.manifest.skill.clone(), |el, skill| {
                                let earned = self.progress.skill_earned_at.is_some()
                                    || self.track.as_ref().map_or(false, |t| self.progress.percent(&t.track) == 100);
                                el.child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            Icon::new(if earned {
                                                IconName::Check
                                            } else {
                                                IconName::Star
                                            })
                                            .size(IconSize::XSmall)
                                            .color(
                                                if earned { Color::Success } else { Color::Muted },
                                            ),
                                        )
                                        .child(
                                            Label::new(if earned {
                                                format!("Skill earned: {skill}")
                                            } else {
                                                format!("Complete the track to earn: {skill}")
                                            })
                                            .size(LabelSize::XSmall)
                                            .color(
                                                if earned { Color::Success } else { Color::Muted },
                                            ),
                                        ),
                                )
                            })
                            .child(
                                Label::new(format!("{done} of {total} exercises · {percent}%"))
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        div()
                            .border_b_1()
                            .border_color(border)
                            .child(self.render_tree(&track, cx)),
                    )
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

/// LEARNZED: `layout = "focused"` hides the Project Panel's dock (unless it
/// also hosts the Learn panel); `layout = "project"` reveals it.
fn apply_layout(
    workspace: &mut Workspace,
    layout: learn::Layout,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    for position in [
        DockPosition::Left,
        DockPosition::Right,
        DockPosition::Bottom,
    ] {
        let dock = workspace.dock_at_position(position).clone();
        let (has_project, has_learn) = {
            let dock = dock.read(cx);
            (
                dock.panel::<project_panel::ProjectPanel>().is_some(),
                dock.panel::<LearnPanel>().is_some(),
            )
        };
        if !has_project || has_learn {
            continue;
        }
        match layout {
            learn::Layout::Focused => dock.update(cx, |dock, cx| dock.set_open(false, window, cx)),
            learn::Layout::Project => {
                workspace.open_panel::<project_panel::ProjectPanel>(window, cx)
            }
        }
    }
}
