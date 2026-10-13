//! Track format v1 (see .orchestration/BRIEF.md).

use anyhow::{Context as _, Result, anyhow};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TrackManifest {
    pub id: String,
    pub title: String,
    pub version: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChapterManifest {
    pub title: String,
    #[serde(default)]
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExerciseManifest {
    pub id: String,
    pub title: String,
    #[serde(default = "default_xp")]
    pub xp: u32,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_file")]
    pub file: String,
    /// XP deducted from the award for each hint revealed.
    #[serde(default = "default_hint_cost")]
    pub hint_xp_cost: u32,
}

fn default_language() -> String {
    "python".into()
}
fn default_xp() -> u32 {
    100
}
fn default_kind() -> String {
    "coding".into()
}
fn default_file() -> String {
    "script.py".into()
}
fn default_hint_cost() -> u32 {
    30
}

#[derive(Debug, Clone)]
pub struct Track {
    pub manifest: TrackManifest,
    /// Directory containing track.toml.
    pub root: PathBuf,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub dir_name: String,
    pub manifest: ChapterManifest,
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone)]
pub struct Exercise {
    pub manifest: ExerciseManifest,
    /// Path relative to the track root, e.g. `02-numpy/02-subsetting-2d`.
    pub rel_dir: PathBuf,
    pub context: String,
    pub instructions: String,
    pub hints: Vec<String>,
}

impl Exercise {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
    /// Path of the learner's file relative to the workspace root.
    pub fn user_file(&self) -> PathBuf {
        self.rel_dir.join(&self.manifest.file)
    }
}

impl Track {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> {
        self.chapters.iter().flat_map(|c| c.exercises.iter())
    }

    pub fn exercise(&self, id: &str) -> Option<&Exercise> {
        self.exercises().find(|e| e.id() == id)
    }

    pub fn next_exercise(&self, id: &str) -> Option<&Exercise> {
        let mut iter = self.exercises().skip_while(|e| e.id() != id);
        iter.next();
        iter.next()
    }

    pub fn previous_exercise(&self, id: &str) -> Option<&Exercise> {
        let mut prev = None;
        for e in self.exercises() {
            if e.id() == id {
                return prev;
            }
            prev = Some(e);
        }
        None
    }

    pub fn total_xp(&self) -> u32 {
        self.exercises().map(|e| e.manifest.xp).sum()
    }

    /// Absolute path of a source file of an exercise inside the track definition.
    pub fn exercise_source(&self, exercise: &Exercise, file: &str) -> PathBuf {
        self.root.join(&exercise.rel_dir).join(file)
    }
}

pub fn parse_track_manifest(s: &str) -> Result<TrackManifest> {
    Ok(toml::from_str(s)?)
}
pub fn parse_chapter_manifest(s: &str) -> Result<ChapterManifest> {
    Ok(toml::from_str(s)?)
}
pub fn parse_exercise_manifest(s: &str) -> Result<ExerciseManifest> {
    Ok(toml::from_str(s)?)
}

/// Split hints.md into hints. Hints are separated by a line containing only `---`.
pub fn parse_hints(s: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let mut current = String::new();
    for line in s.lines() {
        if line.trim() == "---" {
            hints.push(std::mem::take(&mut current));
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    hints.push(current);
    hints
        .into_iter()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .collect()
}

fn read_optional(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))
}

fn sorted_subdirs(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    Ok(dirs)
}

/// Load a track from a directory containing `track.toml`.
pub fn load_track(root: &Path) -> Result<Track> {
    let manifest: TrackManifest = read_toml(&root.join("track.toml"))?;
    let mut chapters = Vec::new();
    for chapter_dir in sorted_subdirs(root)? {
        let chapter_toml = chapter_dir.join("chapter.toml");
        if !chapter_toml.exists() {
            continue;
        }
        let chapter_manifest: ChapterManifest = read_toml(&chapter_toml)?;
        let dir_name = file_name(&chapter_dir)?;
        let mut exercises = Vec::new();
        for exercise_dir in sorted_subdirs(&chapter_dir)? {
            let exercise_toml = exercise_dir.join("exercise.toml");
            if !exercise_toml.exists() {
                continue;
            }
            let exercise_manifest: ExerciseManifest = read_toml(&exercise_toml)?;
            exercises.push(Exercise {
                manifest: exercise_manifest,
                rel_dir: PathBuf::from(&dir_name).join(file_name(&exercise_dir)?),
                context: read_optional(&exercise_dir.join("context.md"))?,
                instructions: read_optional(&exercise_dir.join("instructions.md"))?,
                hints: parse_hints(&read_optional(&exercise_dir.join("hints.md"))?),
            });
        }
        chapters.push(Chapter {
            dir_name,
            manifest: chapter_manifest,
            exercises,
        });
    }
    // Directory names (NN- prefix) give the default order; an explicit `order` wins.
    chapters.sort_by_key(|c| c.manifest.order.unwrap_or(u32::MAX));
    Ok(Track {
        manifest,
        root: root.to_path_buf(),
        chapters,
    })
}

fn file_name(p: &Path) -> Result<String> {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("invalid directory name {}", p.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifests_with_defaults() {
        let t = parse_track_manifest("id = \"t\"\ntitle = \"T\"\nversion = \"1\"").unwrap();
        assert_eq!(t.language, "python");
        let c = parse_chapter_manifest("title = \"C\"\norder = 2").unwrap();
        assert_eq!(c.order, Some(2));
        let e = parse_exercise_manifest("id = \"e\"\ntitle = \"E\"").unwrap();
        assert_eq!(e.file, "script.py");
        assert_eq!(e.xp, 100);
        assert_eq!(e.kind, "coding");
        assert!(parse_exercise_manifest("title = \"no id\"").is_err());
    }

    #[test]
    fn splits_hints() {
        assert_eq!(parse_hints("a\n\n---\nb\n---\n"), vec!["a", "b"]);
        assert!(parse_hints("").is_empty());
    }

    #[test]
    fn loads_bundled_sample_track() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python");
        let track = load_track(&root).unwrap();
        assert_eq!(track.id(), "intro-to-python");
        assert_eq!(track.chapters.len(), 2);
        assert_eq!(track.chapters[1].manifest.title, "NumPy");
        let ex = track.exercise("subsetting-2d").unwrap();
        assert_eq!(
            ex.user_file(),
            Path::new("02-numpy/02-subsetting-2d/script.py")
        );
        assert_eq!(ex.hints.len(), 3);
        assert!(ex.instructions.contains("np_weight_lb"));
        assert_eq!(
            track.next_exercise("numpy-arrays").unwrap().id(),
            "subsetting-2d"
        );
        assert!(track.next_exercise("subsetting-2d").is_none());
        assert_eq!(
            track.previous_exercise("variables").unwrap().id(),
            "hello-python"
        );
        let setup = std::fs::read_to_string(track.exercise_source(ex, "setup.py")).unwrap();
        assert!(setup.contains("baseball"));
    }

    #[test]
    fn bad_toml_reports_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("track.toml"), "id = ").unwrap();
        let err = format!("{:#}", load_track(dir.path()).unwrap_err());
        assert!(err.contains("track.toml"), "{err}");
    }
}
