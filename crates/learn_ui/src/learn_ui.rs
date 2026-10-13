//! LearnZed UI: track picker (AC1) and the Learn panel (AC3/AC4).

mod learn_panel;
mod track_picker;

pub use learn_panel::LearnPanel;
pub use track_picker::TrackPicker;

use gpui::{App, Window, actions};
use workspace::Workspace;

actions!(
    learn,
    [
        /// Toggle focus on the Learn panel.
        ToggleFocus,
        /// Open the learning track picker.
        OpenTrackPicker,
    ]
);

pub fn init(cx: &mut App) {
    if let Err(err) = learn::install_bundled_tracks(&learn::learn_home()) {
        log::error!("failed to install bundled LearnZed tracks: {err:#}");
    }
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleFocus, window, cx| {
            workspace.toggle_panel_focus::<LearnPanel>(window, cx);
        });
        workspace.register_action(|workspace, _: &OpenTrackPicker, window, cx| {
            show_track_picker(workspace, window, cx);
        });
    })
    .detach();
}

/// Show the track picker as the active item of the workspace.
pub fn show_track_picker(workspace: &mut Workspace, window: &mut Window, cx: &mut gpui::Context<Workspace>) {
    let weak = cx.entity().downgrade();
    let picker = cx.new(|cx| TrackPicker::new(weak, cx));
    workspace.add_item_to_active_pane(Box::new(picker), None, true, window, cx);
}
