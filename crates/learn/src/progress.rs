//! `.learn/progress.json`: per-exercise status, attempts, XP, timestamps.

use crate::{Track, now_unix};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseStatus {
    #[default]
    NotStarted,
    InProgress,
    Passed,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExerciseProgress {
    pub status: ExerciseStatus,
    pub attempts: u32,
    pub xp: u32,
    pub hints_used: u32,
    pub first_opened: Option<u64>,
    pub last_attempt: Option<u64>,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub version: u32,
    pub last_opened: Option<u64>,
    pub current_exercise: Option<String>,
    pub exercises: BTreeMap<String, ExerciseProgress>,
    /// When the track's skill was earned (all exercises passed).
    pub skill_earned_at: Option<u64>,
}

impl Progress {
    /// Load progress; a missing file yields fresh state, a corrupt file is
    /// moved aside to `progress.json.bak` and fresh state is returned.
    pub fn load(path: &Path) -> Progress {
        match std::fs::read_to_string(path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(p) => p,
                Err(err) => {
                    log::error!("corrupt progress file {}: {err}", path.display());
                    std::fs::rename(path, path.with_extension("json.bak")).ok();
                    Progress::default()
                }
            },
            Err(_) => Progress::default(),
        }
    }

    /// Atomically write progress (temp file + rename).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("renaming to {}", path.display()))?;
        Ok(())
    }

    pub fn exercise(&self, id: &str) -> ExerciseProgress {
        self.exercises.get(id).cloned().unwrap_or_default()
    }

    pub fn status(&self, id: &str) -> ExerciseStatus {
        self.exercises.get(id).map(|e| e.status).unwrap_or_default()
    }

    fn entry(&mut self, id: &str) -> &mut ExerciseProgress {
        self.exercises.entry(id.to_string()).or_default()
    }

    pub fn mark_opened(&mut self, id: &str) {
        let now = now_unix();
        self.last_opened = Some(now);
        self.current_exercise = Some(id.to_string());
        let e = self.entry(id);
        e.first_opened.get_or_insert(now);
        if e.status == ExerciseStatus::NotStarted {
            e.status = ExerciseStatus::InProgress;
        }
    }

    /// Record a hint reveal. Returns the number of hints now revealed.
    pub fn use_hint(&mut self, id: &str, available: usize) -> u32 {
        let e = self.entry(id);
        if (e.hints_used as usize) < available {
            e.hints_used += 1;
        }
        e.hints_used
    }

    /// Record a submission. On the first pass, XP is awarded (minus hint cost).
    /// Returns the XP awarded by this submission.
    pub fn record_submission(&mut self, id: &str, passed: bool, xp: u32, hint_cost: u32) -> u32 {
        let now = now_unix();
        let e = self.entry(id);
        e.attempts += 1;
        e.last_attempt = Some(now);
        if passed && e.status != ExerciseStatus::Passed {
            e.status = ExerciseStatus::Passed;
            e.completed_at = Some(now);
            e.xp = xp.saturating_sub(e.hints_used * hint_cost);
            return e.xp;
        }
        if !passed && e.status == ExerciseStatus::NotStarted {
            e.status = ExerciseStatus::InProgress;
        }
        0
    }

    /// Record the track's skill once every exercise is passed.
    /// Returns true when the skill was newly earned.
    pub fn award_skill_if_complete(&mut self, track: &Track) -> bool {
        if self.skill_earned_at.is_none()
            && track.manifest.skill.is_some()
            && track.exercises().count() > 0
            && self.passed_count(track) == track.exercises().count()
        {
            self.skill_earned_at = Some(now_unix());
            return true;
        }
        false
    }

    pub fn total_xp(&self) -> u32 {
        self.exercises.values().map(|e| e.xp).sum()
    }

    pub fn passed_count(&self, track: &Track) -> usize {
        track
            .exercises()
            .filter(|e| self.status(e.id()) == ExerciseStatus::Passed)
            .count()
    }

    /// Percent of exercises passed (0-100).
    pub fn percent(&self, track: &Track) -> u32 {
        let total = track.exercises().count();
        if total == 0 {
            return 0;
        }
        (self.passed_count(track) * 100 / total) as u32
    }

    /// An exercise is unlocked if it is the first one, was already started,
    /// or the previous exercise has been passed.
    pub fn is_unlocked(&self, track: &Track, id: &str) -> bool {
        if self.status(id) != ExerciseStatus::NotStarted {
            return true;
        }
        match track.previous_exercise(id) {
            None => true,
            Some(prev) => self.status(prev.id()) == ExerciseStatus::Passed,
        }
    }

    /// First exercise that is not passed (or the last exercise when all passed).
    pub fn resume_exercise<'a>(&self, track: &'a Track) -> Option<&'a crate::Exercise> {
        if let Some(cur) = self
            .current_exercise
            .as_deref()
            .and_then(|id| track.exercise(id))
        {
            if self.status(cur.id()) != ExerciseStatus::Passed {
                return Some(cur);
            }
        }
        track
            .exercises()
            .find(|e| self.status(e.id()) != ExerciseStatus::Passed)
            .or_else(|| track.exercises().last())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Track {
        crate::load_track(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python"))
            .unwrap()
    }

    #[test]
    fn round_trip_and_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".learn/progress.json");
        let mut p = Progress::default();
        p.mark_opened("hello-python");
        assert_eq!(p.record_submission("hello-python", true, 50, 30), 50);
        p.save(&path).unwrap();
        assert_eq!(Progress::load(&path), p);

        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(Progress::load(&path), Progress::default());
        assert!(path.with_extension("json.bak").exists());
        assert_eq!(
            Progress::load(&dir.path().join("missing.json")),
            Progress::default()
        );
    }

    #[test]
    fn xp_hints_unlock_and_percent() {
        let track = sample();
        let mut p = Progress::default();
        assert!(p.is_unlocked(&track, "hello-python"));
        assert!(!p.is_unlocked(&track, "variables"));
        assert_eq!(p.record_submission("hello-python", false, 50, 30), 0);
        assert_eq!(p.status("hello-python"), ExerciseStatus::InProgress);
        assert_eq!(p.use_hint("hello-python", 2), 1);
        assert_eq!(p.record_submission("hello-python", true, 50, 30), 20);
        // Passing again awards nothing.
        assert_eq!(p.record_submission("hello-python", true, 50, 30), 0);
        assert!(p.is_unlocked(&track, "variables"));
        assert_eq!(p.percent(&track), 25);
        assert_eq!(p.total_xp(), 20);
        assert_eq!(p.exercise("hello-python").attempts, 3);
        assert_eq!(p.resume_exercise(&track).unwrap().id(), "variables");
        assert_eq!(p.use_hint("variables", 1), 1);
        assert_eq!(p.use_hint("variables", 1), 1);
        assert!(!p.award_skill_if_complete(&track));
        for e in track.exercises() {
            p.record_submission(e.id(), true, 10, 0);
        }
        assert!(p.award_skill_if_complete(&track));
        assert!(!p.award_skill_if_complete(&track));
        assert!(p.skill_earned_at.is_some());
    }
}
