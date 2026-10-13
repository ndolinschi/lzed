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
    let careers = crate::career::careers_dir(home);
    for (path, contents) in crate::bundled::BUNDLED_FILES {
        let dst = if let Some(rel) = path.strip_prefix("tracks/") {
            dir.join(rel)
        } else if let Some(rel) = path.strip_prefix("careers/") {
            careers.join(rel)
        } else {
            continue;
        };
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
    pub skill_earned_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EarnedSkill {
    pub skill: String,
    pub track_title: String,
    pub earned_at: u64,
}

/// Skills earned across all installed tracks, oldest first.
pub fn earned_skills(tracks: &[TrackSummary]) -> Vec<EarnedSkill> {
    let mut skills = tracks
        .iter()
        .filter_map(|t| {
            let skill = t.track.manifest.skill.clone()?;
            let earned_at = t.skill_earned_at.or_else(|| {
                if t.percent == 100 {
                    t.last_opened.or(Some(0))
                } else {
                    None
                }
            })?;
            Some(EarnedSkill {
                skill,
                track_title: t.track.manifest.title.clone(),
                earned_at,
            })
        })
        .collect::<Vec<_>>();
    skills.sort_by_key(|s| s.earned_at);
    skills
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
            let progress_path = TrackWorkspace::learn_dir(&workspace_dir).join("progress.json");
            let mut progress = Progress::load(&progress_path);
            if progress.award_skill_if_complete(&track) {
                progress.save(&progress_path).ok();
            }
            let skill_earned_at = progress.skill_earned_at.or_else(|| {
                if progress.percent(&track) == 100 && track.manifest.skill.is_some() {
                    progress.last_opened.or(Some(0))
                } else {
                    None
                }
            });
            TrackSummary {
                percent: progress.percent(&track),
                xp: progress.total_xp(),
                last_opened: progress.last_opened,
                skill_earned_at,
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
        assert_eq!(tracks.len(), 9);
        let s = tracks
            .iter()
            .find(|t| t.track.id() == "intro-to-python")
            .unwrap();
        assert_eq!(s.track.id(), "intro-to-python");
        assert_eq!(s.percent, 0);
        assert_eq!(s.workspace_dir, home.path().join("intro-to-python"));

        let ws = TrackWorkspace::materialize(&s.track, &s.workspace_dir).unwrap();
        assert!(ws.runner_script().is_file());
        let mut p = ws.load_progress();
        p.mark_opened("hello-python");
        p.record_submission("hello-python", true, 50, 30);
        ws.save_progress(&p).unwrap();
        let all = installed_tracks(home.path());
        let s = &all[0];
        assert_eq!(
            s.track.id(),
            "intro-to-python",
            "most recently opened first"
        );
        assert_eq!((s.percent, s.xp), (25, 50));
        assert!(s.last_opened.is_some());
        assert!(earned_skills(&all).is_empty());
    }

    #[test]
    fn imports_track_folder() {
        let home = tempfile::tempdir().unwrap();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python");
        let t = add_track_from_folder(home.path(), &src).unwrap();
        assert_eq!(t.root, tracks_dir(home.path()).join("intro-to-python"));
    }

    #[test]
    fn retroactively_awards_skills_for_completed_tracks() {
        let home = tempfile::tempdir().unwrap();
        install_bundled_tracks(home.path()).unwrap();
        let tracks = installed_tracks(home.path());
        let s = tracks
            .iter()
            .find(|t| t.track.id() == "intro-to-python")
            .unwrap();
        let ws = TrackWorkspace::materialize(&s.track, &s.workspace_dir).unwrap();
        let mut p = Progress::default();
        for e in s.track.exercises() {
            p.record_submission(e.id(), true, 10, 0);
        }
        // Simulate an older progress file where skill_earned_at was null despite 100% completion
        p.skill_earned_at = None;
        ws.save_progress(&p).unwrap();

        let all = installed_tracks(home.path());
        let s = all
            .iter()
            .find(|t| t.track.id() == "intro-to-python")
            .unwrap();
        assert_eq!(s.percent, 100);
        assert!(s.skill_earned_at.is_some());
        let skills = earned_skills(&all);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill, "Python");
    }
}
