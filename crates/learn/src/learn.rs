//! LearnZed core: track format, workspace materialization, progress, and the
//! exercise runner contract. Contains no UI code.

mod bundled;
pub mod career;
pub mod progress;
pub mod registry;
pub mod runner;
pub mod track;
pub mod workspace;

pub use career::{Career, installed_careers};
pub use progress::{ExerciseProgress, ExerciseStatus, Progress};
pub use registry::{
    EarnedSkill, TrackSummary, add_track_from_folder, earned_skills, install_bundled_tracks,
    installed_tracks, learn_home, tracks_dir,
};
pub use runner::{CheckOutcome, CheckResult, RunnerError};
pub use track::{Chapter, Exercise, Layout, Track, load_track};
pub use workspace::{LEARN_DIR, TrackWorkspace};

/// Seconds since the unix epoch.
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
