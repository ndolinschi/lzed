use super::*;
use gpui::TestAppContext;
use learn::{TrackWorkspace, install_bundled_tracks};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn setup_test_home() -> (tempfile::TempDir, PathBuf, MutexGuard<'static, ()>) {
    let lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().to_path_buf();
    unsafe {
        std::env::set_var("LEARNZED_HOME", &home);
    }
    install_bundled_tracks(&home).expect("install bundled tracks");
    (dir, home, lock)
}

#[gpui::test]
fn test_track_picker_discovers_installed_tracks_and_careers(cx: &mut TestAppContext) {
    let (_dir, _home, _lock) = setup_test_home();
    let cx = cx.add_empty_window();

    cx.update(|_window, cx| {
        let picker = cx.new(|cx| TrackPicker::new(gpui::WeakEntity::new_invalid(), cx));
        let picker = picker.read(cx);

        assert!(picker.tracks.len() >= 4);
        let track_ids: Vec<_> = picker.tracks.iter().map(|t| t.track.id()).collect();
        assert!(track_ids.contains(&"intro-to-python"));
        assert!(track_ids.contains(&"flask-api"));
        assert!(track_ids.contains(&"rust-axum"));
        assert!(track_ids.contains(&"claude-code"));

        assert!(picker.careers.len() >= 2);
        let career_ids: Vec<_> = picker.careers.iter().map(|c| c.id.as_str()).collect();
        assert!(career_ids.contains(&"python-backend-developer"));
        assert!(career_ids.contains(&"ai-assisted-engineer"));
    });
}

#[gpui::test]
fn test_track_picker_displays_retroactive_skills(cx: &mut TestAppContext) {
    let (_dir, home, _lock) = setup_test_home();
    let tracks = learn::installed_tracks(&home);
    let intro = tracks
        .iter()
        .find(|t| t.track.id() == "intro-to-python")
        .expect("intro-to-python track");

    let ws = TrackWorkspace::materialize(&intro.track, &intro.workspace_dir).unwrap();
    let mut p = learn::Progress::default();
    for ex in intro.track.exercises() {
        p.record_submission(ex.id(), true, 50, 0);
    }
    p.skill_earned_at = None;
    ws.save_progress(&p).unwrap();

    let cx = cx.add_empty_window();
    cx.update(|_window, cx| {
        let picker = cx.new(|cx| TrackPicker::new(gpui::WeakEntity::new_invalid(), cx));
        let picker = picker.read(cx);

        let intro_summary = picker
            .tracks
            .iter()
            .find(|t| t.track.id() == "intro-to-python")
            .unwrap();
        assert_eq!(intro_summary.percent, 100);
        assert!(intro_summary.skill_earned_at.is_some());

        let skills = learn::earned_skills(&picker.tracks);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill, "Python");
    });
}

#[gpui::test]
fn test_track_picker_open_track_materializes_workspace(cx: &mut TestAppContext) {
    let (_dir, home, _lock) = setup_test_home();
    let cx = cx.add_empty_window();

    cx.update(|window, cx| {
        let picker = cx.new(|cx| TrackPicker::new(gpui::WeakEntity::new_invalid(), cx));
        picker.update(cx, |this, cx| {
            let index = this
                .tracks
                .iter()
                .position(|t| t.track.id() == "intro-to-python")
                .unwrap();
            this.open_track(index, window, cx);
        });
    });

    let workspace_dir = home.join("intro-to-python");
    assert!(workspace_dir.join(".learn/track.lock").is_file());
    assert!(workspace_dir.join(".learn/learn_runner.py").is_file());
    assert!(workspace_dir.join("01-python-basics/01-hello-python/script.py").is_file());
}

#[gpui::test]
fn test_learn_panel_exercise_selection_hints_and_reset(cx: &mut TestAppContext) {
    let (_dir, home, _lock) = setup_test_home();
    let tracks = learn::installed_tracks(&home);
    let intro = tracks
        .iter()
        .find(|t| t.track.id() == "intro-to-python")
        .unwrap();

    let ws = TrackWorkspace::materialize(&intro.track, &intro.workspace_dir).unwrap();
    let user_file = ws.root.join("01-python-basics/01-hello-python/script.py");
    let original_starter = std::fs::read_to_string(&user_file).unwrap();

    let cx = cx.add_empty_window();
    cx.update(|window, cx| {
        let panel = cx.new(|cx| LearnPanel::new(gpui::WeakEntity::new_invalid(), None, window, cx));
        panel.update(cx, |panel, cx| {
            assert!(!panel.is_track());
            panel.set_track(ws, window, cx);
            assert!(panel.is_track());
            assert_eq!(panel.selected.as_deref(), Some("hello-python"));

            // Check hints functionality
            assert_eq!(panel.progress.exercise("hello-python").hints_used, 0);
            panel.hint(cx);
            assert_eq!(panel.progress.exercise("hello-python").hints_used, 1);

            // Test reset
            std::fs::write(&user_file, "corrupted learner code").unwrap();
            panel.reset_exercise_starter().unwrap();
            let restored = std::fs::read_to_string(&user_file).unwrap();
            assert_eq!(restored, original_starter);
        });
    });
}
