//! Installed tracks: bundled tracks are extracted into `<learn_home>/.tracks/`.

use crate::{Progress, Track, TrackWorkspace, load_track};
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

const RUNNER_FILE: &str = "runner/learn_runner.py";

/// Root of all LearnZed workspaces: `$LEARNZED_HOME` or `~/LearnZed`.
pub fn learn_home() -> PathBuf {
    std::env::var_os("LEARNZED_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::home_dir().join("LearnZed"))
}

pub fn tracks_dir(home: &Path) -> PathBuf {
    home.join(".tracks")
}

pub(crate) fn runner_script_path(workspace_root: &Path) -> PathBuf {
    TrackWorkspace::learn_dir(workspace_root).join("learn_runner.py")
}

pub(crate) fn write_runner(path: &Path) -> Result<()> {
    let contents = crate::bundled::BUNDLED_FILES
        .iter()
        .find(|(p, _)| *p == RUNNER_FILE)
        .map(|(_, c)| *c)
        .context("runner not bundled")?;
    if std::fs::read(path).ok().as_deref() != Some(contents) {
        std::fs::write(path, contents)?;
    }
    Ok(())
}

/// Write bundled sample tracks into `<home>/.tracks/` (refreshed every launch).
pub fn install_bundled_tracks(home: &Path) -> Result<()> {
    let dir = tracks_dir(home);
    for (path, contents) in crate::bundled::BUNDLED_FILES {
        let Some(rel) = path.strip_prefix("tracks/") else {
            continue;
        };
        let dst = dir.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if std::fs::read(&dst).ok().as_deref() != Some(*contents) {
            std::fs::write(&dst, contents).with_context(|| format!("writing {}", dst.display()))?;
        }
    }
    Ok(())
}

/// Import a track from a local folder by copying it into `<home>/.tracks/<id>`.
pub fn add_track_from_folder(home: &Path, src: &Path) -> Result<Track> {
    let track = load_track(src)?;
    let dst = tracks_dir(home).join(track.id());
    if dst.exists() {
        std::fs::remove_dir_all(&dst)?;
    }
    copy_tree(src, &dst)?;
    load_track(&dst)
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &dst.join(name))?;
        } else {
            std::fs::copy(entry.path(), dst.join(name))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TrackSummary {
    pub track: Track,
    pub workspace_dir: PathBuf,
    pub percent: u32,
    pub xp: u32,
    pub last_opened: Option<u64>,
}

/// List installed tracks with their progress, most recently opened first.
pub fn installed_tracks(home: &Path) -> Vec<TrackSummary> {
    let Ok(entries) = std::fs::read_dir(tracks_dir(home)) else {
        return Vec::new();
    };
    let mut summaries = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("track.toml").is_file())
        .filter_map(|e| {
            load_track(&e.path())
                .map_err(|err| log::error!("failed to load track {:?}: {err:#}", e.path()))
                .ok()
        })
        .map(|track| {
            let workspace_dir = home.join(track.id());
            let progress = Progress::load(&TrackWorkspace::learn_dir(&workspace_dir).join("progress.json"));
            TrackSummary {
                percent: progress.percent(&track),
                xp: progress.total_xp(),
                last_opened: progress.last_opened,
                workspace_dir,
                track,
            }
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|a, b| {
        b.last_opened
            .cmp(&a.last_opened)
            .then_with(|| a.track.manifest.title.cmp(&b.track.manifest.title))
    });
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_and_lists_bundled_tracks() {
        let home = tempfile::tempdir().unwrap();
        install_bundled_tracks(home.path()).unwrap();
        let tracks = installed_tracks(home.path());
        assert_eq!(tracks.len(), 1);
        let s = &tracks[0];
        assert_eq!(s.track.id(), "intro-to-python");
        assert_eq!(s.percent, 0);
        assert_eq!(s.workspace_dir, home.path().join("intro-to-python"));

        let ws = TrackWorkspace::materialize(&s.track, &s.workspace_dir).unwrap();
        assert!(ws.runner_script().is_file());
        let mut p = ws.load_progress();
        p.mark_opened("hello-python");
        p.record_submission("hello-python", true, 50, 30);
        ws.save_progress(&p).unwrap();
        let s = &installed_tracks(home.path())[0];
        assert_eq!((s.percent, s.xp), (25, 50));
        assert!(s.last_opened.is_some());
    }

    #[test]
    fn imports_track_folder() {
        let home = tempfile::tempdir().unwrap();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python");
        let t = add_track_from_folder(home.path(), &src).unwrap();
        assert_eq!(t.root, tracks_dir(home.path()).join("intro-to-python"));
    }
}
