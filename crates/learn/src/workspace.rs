//! Track workspaces: `~/LearnZed/<track-id>/` with app state in `.learn/`.

use crate::{Exercise, Progress, Track, load_track, now_unix};
use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const LEARN_DIR: &str = ".learn";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackLock {
    pub id: String,
    pub version: String,
    pub created_at: u64,
}

/// An opened (materialized) track workspace.
#[derive(Debug, Clone)]
pub struct TrackWorkspace {
    pub root: PathBuf,
    /// Track definition copied into `.learn/track/` (setup, solution, checks).
    pub track: Track,
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

impl TrackWorkspace {
    pub fn learn_dir(root: &Path) -> PathBuf {
        root.join(LEARN_DIR)
    }

    /// Whether `root` is a materialized track workspace.
    pub fn is_track_workspace(root: &Path) -> bool {
        Self::learn_dir(root).join("track.lock").is_file()
    }

    /// Create (first time) or reopen the workspace for `source` in `workspace_root`.
    /// Never overwrites existing learner files.
    pub fn materialize(source: &Track, workspace_root: &Path) -> Result<Self> {
        let learn = Self::learn_dir(workspace_root);
        let lock_path = learn.join("track.lock");
        let track_copy = learn.join("track");
        std::fs::create_dir_all(learn.join("runs"))?;

        let existing_lock: Option<TrackLock> = std::fs::read_to_string(&lock_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok());
        if let Some(lock) = &existing_lock {
            if lock.id != source.manifest.id {
                bail!(
                    "{} belongs to track {:?}, not {:?}",
                    workspace_root.display(),
                    lock.id,
                    source.manifest.id
                );
            }
        }
        let needs_copy = !track_copy.join("track.toml").exists()
            || existing_lock
                .as_ref()
                .is_none_or(|l| l.version != source.manifest.version);
        if needs_copy && source.root != track_copy {
            if track_copy.exists() {
                std::fs::remove_dir_all(&track_copy)?;
            }
            copy_dir(&source.root, &track_copy)?;
            let lock = TrackLock {
                id: source.manifest.id.clone(),
                version: source.manifest.version.clone(),
                created_at: existing_lock.map(|l| l.created_at).unwrap_or_else(now_unix),
            };
            std::fs::write(&lock_path, toml::to_string(&lock)?)?;
        }

        let this = Self::open(workspace_root)?;
        crate::registry::write_runner(&this.runner_script())?;
        for exercise in this.track.exercises() {
            let dst = this.user_file(exercise);
            if !dst.exists() {
                this.write_starter(exercise)?;
            }
        }
        Ok(this)
    }

    /// Open an already materialized workspace.
    pub fn open(workspace_root: &Path) -> Result<Self> {
        let track = load_track(&Self::learn_dir(workspace_root).join("track"))?;
        Ok(Self {
            root: workspace_root.to_path_buf(),
            track,
        })
    }

    pub fn lock(&self) -> Result<TrackLock> {
        Ok(toml::from_str(&std::fs::read_to_string(
            Self::learn_dir(&self.root).join("track.lock"),
        )?)?)
    }

    pub fn progress_path(&self) -> PathBuf {
        Self::learn_dir(&self.root).join("progress.json")
    }

    pub fn load_progress(&self) -> Progress {
        Progress::load(&self.progress_path())
    }

    pub fn save_progress(&self, progress: &Progress) -> Result<()> {
        progress.save(&self.progress_path())
    }

    pub fn runs_dir(&self) -> PathBuf {
        Self::learn_dir(&self.root).join("runs")
    }

    pub fn user_file(&self, exercise: &Exercise) -> PathBuf {
        self.root.join(exercise.user_file())
    }

    pub fn source_file(&self, exercise: &Exercise, name: &str) -> PathBuf {
        self.track.exercise_source(exercise, name)
    }

    fn write_starter(&self, exercise: &Exercise) -> Result<()> {
        let dst = self.user_file(exercise);
        if let Some(dir) = dst.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let starter = self.source_file(exercise, &exercise.variant_file("starter"));
        let contents = std::fs::read(&starter).unwrap_or_default();
        std::fs::write(&dst, contents).with_context(|| format!("writing {}", dst.display()))
    }

    /// Restore the starter code for an exercise.
    pub fn reset_exercise(&self, exercise: &Exercise) -> Result<()> {
        self.write_starter(exercise)
    }

    /// Python interpreter: `.learn/env` venv when present, else `python3`.
    pub fn python(&self) -> PathBuf {
        let venv = Self::learn_dir(&self.root).join("env");
        let candidate = if cfg!(windows) {
            venv.join("Scripts").join("python.exe")
        } else {
            venv.join("bin").join("python")
        };
        if candidate.exists() {
            candidate
        } else {
            PathBuf::from(if cfg!(windows) { "python" } else { "python3" })
        }
    }

    /// Location of the bundled runner script (written by `install_bundled_tracks`).
    pub fn runner_script(&self) -> PathBuf {
        crate::registry::runner_script_path(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Track {
        load_track(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python")).unwrap()
    }

    #[test]
    fn materialize_preserves_user_code_and_resets() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("intro-to-python");
        let track = sample();
        let ws = TrackWorkspace::materialize(&track, &root).unwrap();
        assert!(TrackWorkspace::is_track_workspace(&root));
        let ex = ws.track.exercise("subsetting-2d").unwrap().clone();
        let file = root.join("02-numpy/02-subsetting-2d/script.py");
        let starter = std::fs::read_to_string(&file).unwrap();
        assert!(starter.contains("np_baseball = np.array(baseball)"));
        assert!(root.join(".learn/runs").is_dir());
        let lock = ws.lock().unwrap();
        assert_eq!(
            (lock.id.as_str(), lock.version.as_str()),
            ("intro-to-python", "1.0.0")
        );

        std::fs::write(&file, "print('mine')").unwrap();
        let ws = TrackWorkspace::materialize(&track, &root).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "print('mine')");

        ws.reset_exercise(&ex).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), starter);
    }

    #[test]
    fn refuses_other_track() {
        let dir = tempfile::tempdir().unwrap();
        let track = sample();
        TrackWorkspace::materialize(&track, dir.path()).unwrap();
        let mut other = track;
        other.manifest.id = "other".into();
        assert!(TrackWorkspace::materialize(&other, dir.path()).is_err());
    }
}
