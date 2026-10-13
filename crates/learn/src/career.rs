//! Career paths: `career.toml` groups several skill tracks into an ordered path.

use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Career {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Track ids, in the order they should be taken.
    pub tracks: Vec<String>,
}

pub fn careers_dir(home: &Path) -> std::path::PathBuf {
    home.join(".careers")
}

pub fn load_career(path: &Path) -> Result<Career> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))
}

/// Installed careers, sorted by title.
pub fn installed_careers(home: &Path) -> Vec<Career> {
    let Ok(entries) = std::fs::read_dir(careers_dir(home)) else {
        return Vec::new();
    };
    let mut careers = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("career.toml"))
        .filter(|p| p.is_file())
        .filter_map(|p| load_career(&p).map_err(|err| log::error!("{err:#}")).ok())
        .collect::<Vec<_>>();
    careers.sort_by(|a, b| a.title.cmp(&b.title));
    careers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_and_lists_bundled_careers() {
        let home = tempfile::tempdir().unwrap();
        crate::install_bundled_tracks(home.path()).unwrap();
        let careers = installed_careers(home.path());
        let backend = careers
            .iter()
            .find(|c| c.id == "python-backend-developer")
            .unwrap();
        assert_eq!(backend.tracks, vec!["intro-to-python", "flask-api"]);
        let tracks = crate::installed_tracks(home.path());
        for career in &careers {
            for id in &career.tracks {
                assert!(
                    tracks.iter().any(|t| t.track.id() == id),
                    "missing track {id}"
                );
            }
        }
    }
}
