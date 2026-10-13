use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, PathPromptOptions, Render, WeakEntity,
    Window,
};
use learn::{TrackSummary, TrackWorkspace};
use ui::{Divider, ProgressBar, prelude::*};
use util::ResultExt as _;
use workspace::{OpenMode, Workspace, item::{Item, ItemEvent}};

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
        self.workspace
            .update(cx, |workspace, cx| {
                workspace
                    .open_workspace_for_paths(OpenMode::Activate, vec![dir], window, cx)
                    .detach_and_log_err(cx);
            })
            .log_err();
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

    fn render_track(&self, index: usize, summary: &TrackSummary, cx: &mut Context<Self>) -> impl IntoElement {
        let manifest = &summary.track.manifest;
        let exercises = summary.track.exercises().count();
        let chapters = summary.track.chapters.len();
        let last_opened = match summary.last_opened {
            Some(ts) => format_ago(learn::now_unix().saturating_sub(ts)),
            None => "Not started".to_string(),
        };
        v_flex()
            .id(("track", index))
            .p_4()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().elevated_surface_background)
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Icon::new(IconName::Book).color(Color::Accent))
                            .child(Headline::new(manifest.title.clone()).size(HeadlineSize::Small)),
                    )
                    .child(
                        Button::new(("open-track", index), if summary.last_opened.is_some() { "Continue" } else { "Start" })
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_track(index, window, cx)
                            })),
                    ),
            )
            .child(Label::new(manifest.description.clone()).color(Color::Muted))
            .child(
                h_flex()
                    .gap_3()
                    .child(Label::new(format!("{chapters} chapters · {exercises} exercises")).size(LabelSize::Small).color(Color::Muted))
                    .child(Label::new(format!("{} XP", summary.xp)).size(LabelSize::Small).color(Color::Muted))
                    .child(Label::new(format!("Last opened: {last_opened}")).size(LabelSize::Small).color(Color::Muted)),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div().flex_1().child(ProgressBar::new(
                            ("progress", index),
                            summary.percent as f32,
                            100.,
                            cx,
                        )),
                    )
                    .child(Label::new(format!("{}%", summary.percent)).size(LabelSize::Small)),
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
        h_flex()
            .key_context("TrackPicker")
            .track_focus(&self.focus_handle)
            .size_full()
            .justify_center()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .id("track-picker")
                    .w(px(640.))
                    .h_full()
                    .py_12()
                    .gap_6()
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Headline::new("Choose a learning track").size(HeadlineSize::Large))
                            .child(Label::new("Pick a track to continue where you left off, or import a new one.").color(Color::Muted)),
                    )
                    .when_some(self.error.clone(), |el, err| el.child(Label::new(err).color(Color::Error)))
                    .when(tracks.is_empty(), |el| el.child(Label::new("No tracks installed.").color(Color::Muted)))
                    .children(tracks)
                    .child(Divider::horizontal())
                    .child(
                        h_flex().child(
                            Button::new("add-track", "Add track from folder…")
                                .start_icon(Icon::new(IconName::FolderOpen))
                                .on_click(cx.listener(|this, _, window, cx| this.add_track(window, cx))),
                        ),
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
