use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, PathPromptOptions, Render, WeakEntity,
    Window,
};
use learn::{TrackSummary, TrackWorkspace};
use ui::{ProgressBar, TintColor, prelude::*};
use util::ResultExt as _;
use workspace::{
    OpenMode, Workspace,
    item::{Item, ItemEvent},
};

/// Full-pane learning track picker shown on startup (AC1).
pub struct TrackPicker {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    tracks: Vec<TrackSummary>,
    error: Option<SharedString>,
}

impl TrackPicker {
    pub fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            tracks: Vec::new(),
            error: None,
        };
        this.refresh();
        this
    }

    fn refresh(&mut self) {
        self.tracks = learn::installed_tracks(&learn::learn_home());
    }

    fn open_track(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(summary) = self.tracks.get(index) else {
            return;
        };
        if let Err(err) = TrackWorkspace::materialize(&summary.track, &summary.workspace_dir) {
            self.error = Some(format!("Failed to open track: {err:#}").into());
            cx.notify();
            return;
        }
        let dir = summary.workspace_dir.clone();
        // Deferred: the workspace reads its items (including this picker) while opening.
        let workspace = self.workspace.clone();
        window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    workspace
                        .open_workspace_for_paths(OpenMode::Activate, vec![dir], window, cx)
                        .detach_and_log_err(cx);
                })
                .log_err();
        });
    }

    fn add_track(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Import Track".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let result = learn::registry::add_track_from_folder(&learn::learn_home(), &path);
            this.update(cx, |this, cx| {
                this.error = result
                    .err()
                    .map(|err| format!("Not a valid track folder: {err:#}").into());
                this.refresh();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_track(
        &self,
        index: usize,
        summary: &TrackSummary,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let manifest = &summary.track.manifest;
        let total = summary.track.exercises().count();
        let done = (summary.percent as usize * total).div_ceil(100).min(total);
        let chapters = summary.track.chapters.len();
        let (cta, started) = match (summary.last_opened, summary.percent) {
            (_, 100) => ("Review", true),
            (Some(_), _) => ("Continue", true),
            (None, _) => ("Start track", false),
        };
        let mut meta = format!(
            "{chapters} chapters · {total} exercises · {}",
            manifest.language
        );
        if let Some(ts) = summary.last_opened {
            meta.push_str(&format!(
                " · opened {}",
                format_ago(learn::now_unix().saturating_sub(ts))
            ));
        }
        let colors = cx.theme().colors();
        v_flex()
            .id(("track", index))
            .rounded_md()
            .border_1()
            .border_color(colors.border_variant)
            .bg(colors.editor_background)
            .hover(|style| style.border_color(colors.border))
            .child(
                v_flex()
                    .p_4()
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_3()
                            .justify_between()
                            .child(
                                Headline::new(manifest.title.clone()).size(HeadlineSize::Small),
                            )
                            .child(
                                Button::new(("open-track", index), cta)
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .end_icon(Icon::new(IconName::ArrowRight).size(IconSize::Small))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_track(index, window, cx)
                                    })),
                            ),
                    )
                    .child(Label::new(meta).size(LabelSize::Small).color(Color::Muted))
                    .when(!manifest.description.is_empty(), |el| {
                        el.child(div().pt_2().child(Label::new(manifest.description.clone())))
                    }),
            )
            .child(
                v_flex()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .border_t_1()
                    .border_color(colors.border_variant)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                Label::new(if started {
                                    format!("{done} of {total} exercises complete · {} XP", summary.xp)
                                } else {
                                    "Not started yet".to_string()
                                })
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                            )
                            .child(
                                Label::new(format!("{}%", summary.percent))
                                    .size(LabelSize::Small)
                                    .color(if summary.percent == 100 {
                                        Color::Success
                                    } else {
                                        Color::Default
                                    }),
                            ),
                    )
                    .child(ProgressBar::new(
                        ("progress", index),
                        summary.percent as f32,
                        100.,
                        cx,
                    )),
            )
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .items_center()
            .gap_3()
            .py_12()
            .rounded_md()
            .border_1()
            .border_dashed()
            .border_color(cx.theme().colors().border_variant)
            .child(Icon::new(IconName::Book).size(IconSize::Medium).color(Color::Muted))
            .child(Label::new("No learning tracks installed"))
            .child(
                Label::new("Import a track folder that contains a track.toml.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Button::new("empty-add-track", "Add track")
                    .style(ButtonStyle::Tinted(TintColor::Accent))
                    .on_click(cx.listener(|this, _, window, cx| this.add_track(window, cx))),
            )
    }
}

fn format_ago(secs: u64) -> String {
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86400 => format!("{} h ago", secs / 3600),
        _ => format!("{} days ago", secs / 86400),
    }
}

impl Render for TrackPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tracks = self
            .tracks
            .clone()
            .iter()
            .enumerate()
            .map(|(ix, t)| self.render_track(ix, t, cx).into_any_element())
            .collect::<Vec<_>>();
        let empty = tracks.is_empty();
        div()
            .id("track-picker")
            .key_context("TrackPicker")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().colors().editor_background)
            .child(
                h_flex().w_full().justify_center().child(
                    v_flex()
                        .w_full()
                        .max_w(px(560.))
                        .px_6()
                        .pt_24()
                        .pb_12()
                        .gap_6()
                        .child(
                            h_flex()
                                .items_end()
                                .justify_between()
                                .gap_4()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            Headline::new("Choose a learning track")
                                                .size(HeadlineSize::Large),
                                        )
                                        .child(
                                            Label::new("Pick up where you left off, or start something new.")
                                                .color(Color::Muted),
                                        ),
                                )
                                .when(!empty, |el| {
                                    el.child(
                                        Button::new("add-track", "Add track")
                                            .style(ButtonStyle::Subtle)
                                            .start_icon(
                                                Icon::new(IconName::Plus).size(IconSize::Small),
                                            )
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.add_track(window, cx)
                                            })),
                                    )
                                }),
                        )
                        .when_some(self.error.clone(), |el, err| {
                            el.child(
                                h_flex()
                                    .gap_2()
                                    .p_2()
                                    .rounded_md()
                                    .bg(cx.theme().status().error_background)
                                    .child(Icon::new(IconName::XCircle).size(IconSize::Small).color(Color::Error))
                                    .child(Label::new(err).size(LabelSize::Small)),
                            )
                        })
                        .when(empty, |el| el.child(self.render_empty(cx)))
                        .child(v_flex().gap_3().children(tracks)),
                ),
            )
    }
}

impl EventEmitter<ItemEvent> for TrackPicker {}

impl Focusable for TrackPicker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for TrackPicker {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Learning Tracks".into()
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event)
    }
}
