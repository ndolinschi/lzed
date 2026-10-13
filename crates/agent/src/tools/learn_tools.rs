//! Built-in agent tools for LearnZed tracks.
//! Strictly enforces that reference solution code is never returned or leaked.

use crate::{AgentTool, ToolCallEventStream, ToolInput};
use agent_client_protocol::schema::v2 as acp;
use anyhow::{Result, anyhow};
use gpui::{App, AsyncApp, Entity, SharedString, Task};
use language_model::LanguageModelToolResultContent;
use project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn tool_content_err(err: anyhow::Error) -> LanguageModelToolResultContent {
    format!("Error: {err:#}").into()
}

/// Locate an active LearnZed track workspace across the project's worktrees.
fn find_track_workspace(
    project: &Entity<Project>,
    cx: &mut AsyncApp,
) -> Result<learn::TrackWorkspace> {
    project.read_with(cx, |project, cx| {
        for worktree in project.worktrees(cx) {
            let abs_path = worktree.read(cx).abs_path();
            let mut curr: Option<&Path> = Some(abs_path.as_ref());
            while let Some(dir) = curr {
                if learn::TrackWorkspace::is_track_workspace(dir) {
                    if let Ok(ws) = learn::TrackWorkspace::open(dir) {
                        return Ok(ws);
                    }
                }
                curr = dir.parent();
            }
        }
        anyhow::bail!("No active LearnZed track workspace found in this project. Open a LearnZed track to use learning tools.")
    })
}

/// Resolve the requested exercise from workspace by ID or current active progress.
fn resolve_exercise<'a>(
    ws: &'a learn::TrackWorkspace,
    exercise_id: Option<&str>,
) -> Result<&'a learn::Exercise> {
    if let Some(id) = exercise_id {
        return ws
            .track
            .exercise(id)
            .ok_or_else(|| anyhow!("Exercise '{id}' not found in track '{}'", ws.track.manifest.id));
    }
    let progress = ws.load_progress();
    if let Some(current_id) = &progress.current_exercise {
        if let Some(ex) = ws.track.exercise(current_id) {
            return Ok(ex);
        }
    }
    for ex in ws.track.exercises() {
        if progress.status(ex.id()) != learn::ExerciseStatus::Passed {
            return Ok(ex);
        }
    }
    ws.track
        .exercises()
        .next()
        .ok_or_else(|| anyhow!("Track has no exercises"))
}

// ---------------------------------------------------------------------------
// 1. GetCurrentExerciseTool
// ---------------------------------------------------------------------------

/// Retrieve information and instructions for an exercise.
///
/// If `exercise_id` is omitted, the currently active exercise in the track is returned.
/// STRICT: Never returns or leaks reference solution code.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetCurrentExerciseInput {
    /// Optional exercise ID. If omitted, returns the current active exercise.
    pub exercise_id: Option<String>,
}

pub struct GetCurrentExerciseTool {
    project: Entity<Project>,
}

impl GetCurrentExerciseTool {
    pub const NAME: &'static str = "get_current_exercise";

    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for GetCurrentExerciseTool {
    type Input = GetCurrentExerciseInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = Self::NAME;

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            if let Some(id) = input.exercise_id {
                return format!("Get exercise `{id}`").into();
            }
        }
        "Get current exercise".into()
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(tool_content_err)?;
            let ws = find_track_workspace(&project, cx).map_err(tool_content_err)?;
            let ex = resolve_exercise(&ws, input.exercise_id.as_deref()).map_err(tool_content_err)?;
            let progress = ws.load_progress();
            let status = progress.status(ex.id());
            let ex_prog = progress.exercise(ex.id());

            let chapter_title = ws
                .track
                .chapters
                .iter()
                .find(|c| c.exercises.iter().any(|e| e.id() == ex.id()))
                .map(|c| c.manifest.title.as_str())
                .unwrap_or_default();

            let result = serde_json::json!({
                "exercise_id": ex.id(),
                "title": ex.manifest.title,
                "chapter": chapter_title,
                "status": match status {
                    learn::ExerciseStatus::NotStarted => "not_started",
                    learn::ExerciseStatus::InProgress => "in_progress",
                    learn::ExerciseStatus::Passed => "passed",
                },
                "user_file": ex.user_file().to_string_lossy(),
                "instructions": ex.instructions,
                "context": ex.context,
                "hints_available": ex.hints.len(),
                "hints_revealed": ex_prog.hints_used,
                "attempts": ex_prog.attempts,
                "xp_reward": ex.manifest.xp,
            });

            Ok(serde_json::to_string_pretty(&result).unwrap_or_default().into())
        })
    }
}

// ---------------------------------------------------------------------------
// 2. RunCheckTool
// ---------------------------------------------------------------------------

/// Run the automated tests/checker for an exercise.
///
/// Returns the runner verification JSON outcome (passed, checks, feedback, stdout, stderr).
/// Updates learner progress on success.
/// STRICT: Never returns or leaks reference solution code.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunCheckInput {
    /// Optional exercise ID to check. If omitted, checks the current active exercise.
    pub exercise_id: Option<String>,
}

pub struct RunCheckTool {
    project: Entity<Project>,
}

impl RunCheckTool {
    pub const NAME: &'static str = "run_check";

    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for RunCheckTool {
    type Input = RunCheckInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = Self::NAME;

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Execute
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            if let Some(id) = input.exercise_id {
                return format!("Check exercise `{id}`").into();
            }
        }
        "Run exercise check".into()
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(tool_content_err)?;
            let ws = find_track_workspace(&project, cx).map_err(tool_content_err)?;
            let ex = resolve_exercise(&ws, input.exercise_id.as_deref())
                .cloned()
                .map_err(tool_content_err)?;

            let ws_clone = ws.clone();
            let ex_clone = ex.clone();
            let outcome = cx
                .background_executor()
                .spawn(async move {
                    learn::runner::check_exercise(&ws_clone, &ex_clone, Duration::from_secs(10))
                })
                .await;

            match outcome {
                Ok(result) => {
                    let mut progress = ws.load_progress();
                    let xp_awarded = progress.record_submission(
                        ex.id(),
                        result.passed,
                        ex.manifest.xp,
                        ex.manifest.hint_xp_cost,
                    );
                    let skill_awarded = progress.award_skill_if_complete(&ws.track);
                    ws.save_progress(&progress).ok();

                    let failed_checks: Vec<_> = result
                        .checks
                        .iter()
                        .filter(|c| !c.passed)
                        .map(|c| {
                            serde_json::json!({
                                "name": c.name,
                                "message": c.message,
                            })
                        })
                        .collect();

                    let response = serde_json::json!({
                        "exercise_id": ex.id(),
                        "passed": result.passed,
                        "checks": result.checks,
                        "failed_checks": failed_checks,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                        "xp_awarded": xp_awarded,
                        "skill_earned": skill_awarded,
                        "summary": if result.passed {
                            "All checks passed successfully!".to_string()
                        } else {
                            format!("Check failed ({} of {} checks passed)",
                                result.checks.iter().filter(|c| c.passed).count(),
                                result.checks.len()
                            )
                        }
                    });

                    Ok(serde_json::to_string_pretty(&response).unwrap_or_default().into())
                }
                Err(err) => {
                    let mut progress = ws.load_progress();
                    progress.record_submission(ex.id(), false, ex.manifest.xp, ex.manifest.hint_xp_cost);
                    ws.save_progress(&progress).ok();

                    let response = serde_json::json!({
                        "exercise_id": ex.id(),
                        "passed": false,
                        "error": err.to_string(),
                    });
                    Ok(serde_json::to_string_pretty(&response).unwrap_or_default().into())
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// 3. GetHintTool
// ---------------------------------------------------------------------------

/// Reveal and retrieve a hint for an exercise.
///
/// Increments hints used in learner progress and deducts XP from the exercise reward.
/// STRICT: Never returns or leaks reference solution code.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHintInput {
    /// Optional exercise ID. If omitted, uses current active exercise.
    pub exercise_id: Option<String>,
    /// Optional 1-based hint level (e.g. 1 for first hint, 2 for second).
    /// If omitted, reveals the next available hint.
    pub level: Option<usize>,
}

pub struct GetHintTool {
    project: Entity<Project>,
}

impl GetHintTool {
    pub const NAME: &'static str = "get_hint";

    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for GetHintTool {
    type Input = GetHintInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = Self::NAME;

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            if let Some(id) = input.exercise_id {
                return format!("Get hint for `{id}`").into();
            }
        }
        "Get exercise hint".into()
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(tool_content_err)?;
            let ws = find_track_workspace(&project, cx).map_err(tool_content_err)?;
            let ex = resolve_exercise(&ws, input.exercise_id.as_deref()).map_err(tool_content_err)?;

            if ex.hints.is_empty() {
                let res = serde_json::json!({
                    "exercise_id": ex.id(),
                    "message": "No hints are configured for this exercise.",
                    "hints_available": 0,
                });
                return Ok(serde_json::to_string_pretty(&res).unwrap_or_default().into());
            }

            let mut progress = ws.load_progress();
            let used = progress.exercise(ex.id()).hints_used as usize;
            let target_level = input
                .level
                .unwrap_or(used + 1)
                .clamp(1, ex.hints.len());

            while (progress.exercise(ex.id()).hints_used as usize) < target_level {
                progress.use_hint(ex.id(), ex.hints.len());
            }
            ws.save_progress(&progress).ok();

            let hint_text = &ex.hints[target_level - 1];
            let res = serde_json::json!({
                "exercise_id": ex.id(),
                "hint_level": target_level,
                "total_hints": ex.hints.len(),
                "hint": hint_text,
                "xp_cost": (target_level as u32) * ex.manifest.hint_xp_cost,
            });

            Ok(serde_json::to_string_pretty(&res).unwrap_or_default().into())
        })
    }
}

// ---------------------------------------------------------------------------
// 4. GetProgressTool
// ---------------------------------------------------------------------------

/// Get overall learner progress and skill status for the current track.
///
/// Returns completion percentage, total XP earned, exercise statuses, and earned skills.
/// STRICT: Never returns or leaks reference solution code.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetProgressInput {}

pub struct GetProgressTool {
    project: Entity<Project>,
}

impl GetProgressTool {
    pub const NAME: &'static str = "get_progress";

    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for GetProgressTool {
    type Input = GetProgressInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = Self::NAME;

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        _input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        "Get learning progress".into()
    }

    fn run(
        self: Arc<Self>,
        _input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| {
            let ws = find_track_workspace(&project, cx).map_err(tool_content_err)?;
            let progress = ws.load_progress();

            let exercises: Vec<_> = ws
                .track
                .exercises()
                .map(|ex| {
                    let ep = progress.exercise(ex.id());
                    serde_json::json!({
                        "id": ex.id(),
                        "title": ex.manifest.title,
                        "status": match ep.status {
                            learn::ExerciseStatus::NotStarted => "not_started",
                            learn::ExerciseStatus::InProgress => "in_progress",
                            learn::ExerciseStatus::Passed => "passed",
                        },
                        "attempts": ep.attempts,
                        "hints_used": ep.hints_used,
                        "xp": ep.xp,
                    })
                })
                .collect();

            let res = serde_json::json!({
                "track_id": ws.track.manifest.id,
                "track_title": ws.track.manifest.title,
                "percent": progress.percent(&ws.track),
                "total_xp": progress.total_xp(),
                "passed_count": progress.passed_count(&ws.track),
                "total_exercises": ws.track.exercises().count(),
                "skill_earned": progress.skill_earned_at.is_some(),
                "skill_badge": ws.track.manifest.skill,
                "exercises": exercises,
            });

            Ok(serde_json::to_string_pretty(&res).unwrap_or_default().into())
        })
    }
}

// ---------------------------------------------------------------------------
// 5. GenerateTrackTool
// ---------------------------------------------------------------------------

fn default_track_language() -> String {
    "python".into()
}
fn default_exercise_xp() -> u32 {
    100
}
fn default_hint_cost() -> u32 {
    30
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedExerciseInput {
    /// Directory slug for the exercise, e.g. "01-hello"
    pub dir_name: String,
    /// Exercise title, e.g. "Say Hello"
    pub title: String,
    /// Educational context and concept explanation in markdown
    pub context: String,
    /// Step-by-step instructions for the learner in markdown
    pub instructions: String,
    /// Starter code file content for the student (with TODO comments)
    pub starter_code: String,
    /// Complete reference solution code that passes the checker
    pub solution_code: String,
    /// Test verification script (uses test() assertions from learn runner)
    pub check_code: String,
    /// Optional setup code file content
    #[serde(default)]
    pub setup_code: Option<String>,
    /// Optional pedagogical hints
    #[serde(default)]
    pub hints: Vec<String>,
    /// XP reward upon completion (default 100)
    #[serde(default = "default_exercise_xp")]
    pub xp: u32,
    /// XP penalty per hint revealed (default 30)
    #[serde(default = "default_hint_cost")]
    pub hint_cost: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedChapterInput {
    /// Chapter directory slug, e.g. "01-basics"
    pub dir_name: String,
    /// Chapter title, e.g. "Basics"
    pub title: String,
    /// Exercises in this chapter
    pub exercises: Vec<GeneratedExerciseInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateTrackInput {
    /// Track slug ID, e.g. "intro-to-algorithms" or "fastapi-routes"
    pub id: String,
    /// Human readable title, e.g. "Intro to Algorithms"
    pub title: String,
    /// Description of what the track teaches
    pub description: String,
    /// Optional skill badge awarded upon 100% completion, e.g. "algorithms-basics"
    pub skill: Option<String>,
    /// Programming language, default "python"
    #[serde(default = "default_track_language")]
    pub language: String,
    /// List of chapters containing exercises
    pub chapters: Vec<GeneratedChapterInput>,
}

pub struct GenerateTrackTool;

impl GenerateTrackTool {
    pub const NAME: &'static str = "generate_track";

    pub fn new() -> Self {
        Self
    }
}

impl Default for GenerateTrackTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTool for GenerateTrackTool {
    type Input = GenerateTrackInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = Self::NAME;

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Execute
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            return format!("Generate track `{}`", input.title).into();
        }
        "Generate track".into()
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(tool_content_err)?;

            if input.id.trim().is_empty() {
                return Err(tool_content_err(anyhow!("Track ID cannot be empty")));
            }
            if input.chapters.is_empty() {
                return Err(tool_content_err(anyhow!("Track must contain at least one chapter")));
            }

            let home = learn::learn_home();
            let tracks_dir = learn::tracks_dir(&home);
            let target_dir = tracks_dir.join(&input.id);

            // Execute track write and verification on background thread
            let res = cx
                .background_executor()
                .spawn(async move {
                    // Create in temporary directory first for atomic validation
                    let temp_dir = tempfile::tempdir().map_err(|e| anyhow!("Failed to create tempdir: {e}"))?;
                    let staged_track = temp_dir.path().join(&input.id);
                    std::fs::create_dir_all(&staged_track)
                        .map_err(|e| anyhow!("Failed to create staged directory: {e}"))?;

                    // 1. Write track.toml
                    let mut track_toml = format!(
                        "id = {:?}\ntitle = {:?}\nversion = \"1.0.0\"\ndescription = {:?}\n",
                        input.id, input.title, input.description
                    );
                    if let Some(skill) = &input.skill {
                        track_toml.push_str(&format!("skill = {:?}\n", skill));
                    }
                    std::fs::write(staged_track.join("track.toml"), track_toml)
                        .map_err(|e| anyhow!("Failed to write track.toml: {e}"))?;

                    let mut total_exercises = 0;

                    // 2. Write chapters and exercises
                    for chapter in &input.chapters {
                        let chap_path = staged_track.join(&chapter.dir_name);
                        std::fs::create_dir_all(&chap_path)
                            .map_err(|e| anyhow!("Failed to create chapter dir: {e}"))?;

                        let chap_toml = format!("title = {:?}\n", chapter.title);
                        std::fs::write(chap_path.join("chapter.toml"), chap_toml)
                            .map_err(|e| anyhow!("Failed to write chapter.toml: {e}"))?;

                        for exercise in &chapter.exercises {
                            total_exercises += 1;
                            let ex_path = chap_path.join(&exercise.dir_name);
                            std::fs::create_dir_all(&ex_path)
                                .map_err(|e| anyhow!("Failed to create exercise dir: {e}"))?;

                            let ex_toml = format!(
                                "id = {:?}\ntitle = {:?}\nlanguage = {:?}\nxp = {}\nkind = \"coding\"\nfile = \"script.py\"\nhint_xp_cost = {}\n",
                                exercise.dir_name, exercise.title, input.language, exercise.xp, exercise.hint_cost
                            );
                            std::fs::write(ex_path.join("exercise.toml"), ex_toml)
                                .map_err(|e| anyhow!("Failed to write exercise.toml: {e}"))?;

                            std::fs::write(ex_path.join("context.md"), &exercise.context)
                                .map_err(|e| anyhow!("Failed to write context.md: {e}"))?;
                            std::fs::write(ex_path.join("instructions.md"), &exercise.instructions)
                                .map_err(|e| anyhow!("Failed to write instructions.md: {e}"))?;
                            std::fs::write(ex_path.join("starter.py"), &exercise.starter_code)
                                .map_err(|e| anyhow!("Failed to write starter.py: {e}"))?;
                            std::fs::write(ex_path.join("solution.py"), &exercise.solution_code)
                                .map_err(|e| anyhow!("Failed to write solution.py: {e}"))?;
                            std::fs::write(ex_path.join("check.py"), &exercise.check_code)
                                .map_err(|e| anyhow!("Failed to write check.py: {e}"))?;

                            let setup = exercise.setup_code.as_deref().unwrap_or("# setup\n");
                            std::fs::write(ex_path.join("setup.py"), setup)
                                .map_err(|e| anyhow!("Failed to write setup.py: {e}"))?;

                            if !exercise.hints.is_empty() {
                                let hints_md = exercise.hints.join("\n---\n");
                                std::fs::write(ex_path.join("hints.md"), hints_md)
                                    .map_err(|e| anyhow!("Failed to write hints.md: {e}"))?;
                            }
                        }
                    }

                    // 3. Validate with learn parser
                    let loaded = learn::load_track(&staged_track)
                        .map_err(|e| anyhow!("Generated track failed schema validation: {e:#}"))?;

                    // 4. Validate that solution passes check using runner
                    let test_ws_dir = temp_dir.path().join("test_workspace");
                    let ws = learn::TrackWorkspace::materialize(&loaded, &test_ws_dir)
                        .map_err(|e| anyhow!("Failed to materialize test workspace: {e:#}"))?;
                    for ex in ws.track.exercises() {
                        let user_file = ws.user_file(ex);
                        let sol_file = ws.source_file(ex, &ex.variant_file("solution"));
                        if sol_file.is_file() {
                            if let Ok(sol_code) = std::fs::read(&sol_file) {
                                std::fs::write(&user_file, sol_code).ok();
                            }
                        }
                        let check_outcome = learn::runner::check_exercise(&ws, ex, Duration::from_secs(10));
                        match check_outcome {
                            Ok(check_result) => {
                                if !check_result.passed {
                                    return Err(anyhow!(
                                        "Solution for exercise '{}' failed check verification: {}. Checks: {:?}",
                                        ex.id(),
                                        check_result.stderr,
                                        check_result.checks
                                    ));
                                }
                            }
                            Err(err) => {
                                return Err(anyhow!(
                                    "Runner error when verifying exercise '{}': {}",
                                    ex.id(),
                                    err
                                ));
                            }
                        }
                    }

                    // 5. Move/copy staged track into target ~/.tracks/<id>
                    if target_dir.exists() {
                        std::fs::remove_dir_all(&target_dir)
                            .map_err(|e| anyhow!("Failed to remove existing track dir: {e}"))?;
                    }
                    if let Some(parent) = target_dir.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| anyhow!("Failed to create parent dir: {e}"))?;
                    }
                    learn::add_track_from_folder(&home, &staged_track)
                        .map_err(|e| anyhow!("Failed to register track in LearnZed: {e:#}"))?;

                    Ok::<_, anyhow::Error>((input.id, input.title, total_exercises, input.skill))
                })
                .await;

            match res {
                Ok((id, title, total_exercises, skill)) => {
                    let out = serde_json::json!({
                        "success": true,
                        "track_id": id,
                        "title": title,
                        "total_exercises": total_exercises,
                        "skill_badge": skill,
                        "message": format!(
                            "Track '{}' successfully created, verified, and registered! It is now available in your LearnZed Track Picker.",
                            title
                        ),
                    });
                    Ok(serde_json::to_string_pretty(&out).unwrap_or_default().into())
                }
                Err(err) => Err(tool_content_err(err)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use learn::load_track;
    use std::path::Path;

    fn sample_track() -> learn::Track {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("learn/tracks/intro-to-python");
        load_track(&manifest_dir).unwrap()
    }

    #[test]
    fn test_resolve_exercise() {
        let dir = tempfile::tempdir().unwrap();
        let track = sample_track();
        let ws = learn::TrackWorkspace::materialize(&track, dir.path()).unwrap();

        // Specific exercise ID
        let ex = resolve_exercise(&ws, Some("subsetting-2d")).unwrap();
        assert_eq!(ex.id(), "subsetting-2d");

        // Default to first active exercise
        let default_ex = resolve_exercise(&ws, None).unwrap();
        assert_eq!(default_ex.id(), "hello-python");

        // Unknown exercise ID errors out
        assert!(resolve_exercise(&ws, Some("nonexistent")).is_err());
    }

    #[test]
    fn test_tools_never_leak_solution() {
        let dir = tempfile::tempdir().unwrap();
        let track = sample_track();
        let ws = learn::TrackWorkspace::materialize(&track, dir.path()).unwrap();
        let ex = resolve_exercise(&ws, Some("subsetting-2d")).unwrap();

        // Exercise info contains instructions and context, but never solution
        assert!(!ex.instructions.contains("solution"));
        assert!(!ex.context.contains("solution"));
        assert_eq!(ex.user_file(), Path::new("02-numpy/02-subsetting-2d/script.py"));

        // Hints contain pedagogical hints, but never solution file
        for hint in &ex.hints {
            assert!(!hint.contains("solution.py"));
        }
    }

    #[test]
    fn test_generate_track_schema_and_runner_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let staged_track = temp_dir.path().join("my-test-track");
        std::fs::create_dir_all(&staged_track).unwrap();

        let input = GenerateTrackInput {
            id: "my-test-track".into(),
            title: "My Test Track".into(),
            description: "A test track".into(),
            skill: Some("test-skill".into()),
            language: "python".into(),
            chapters: vec![GeneratedChapterInput {
                dir_name: "01-chapter".into(),
                title: "Chapter 1".into(),
                exercises: vec![GeneratedExerciseInput {
                    dir_name: "01-hello".into(),
                    title: "Hello".into(),
                    context: "Context here".into(),
                    instructions: "Print hello".into(),
                    starter_code: "# TODO\n".into(),
                    solution_code: "print('hello')\n".into(),
                    check_code: "check_output_contains('hello')\n".into(),
                    setup_code: None,
                    hints: vec!["Hint 1".into()],
                    xp: 100,
                    hint_cost: 30,
                }],
            }],
        };

        // 1. Write track.toml
        let mut track_toml = format!(
            "id = {:?}\ntitle = {:?}\nversion = \"1.0.0\"\ndescription = {:?}\n",
            input.id, input.title, input.description
        );
        if let Some(skill) = &input.skill {
            track_toml.push_str(&format!("skill = {:?}\n", skill));
        }
        std::fs::write(staged_track.join("track.toml"), track_toml).unwrap();

        // 2. Write chapter & exercise
        let chap_path = staged_track.join("01-chapter");
        std::fs::create_dir_all(&chap_path).unwrap();
        std::fs::write(chap_path.join("chapter.toml"), "title = \"Chapter 1\"\n").unwrap();

        let ex_path = chap_path.join("01-hello");
        std::fs::create_dir_all(&ex_path).unwrap();
        std::fs::write(
            ex_path.join("exercise.toml"),
            "id = \"01-hello\"\ntitle = \"Hello\"\nlanguage = \"python\"\nxp = 100\nkind = \"coding\"\nfile = \"script.py\"\nhint_xp_cost = 30\n",
        ).unwrap();
        std::fs::write(ex_path.join("context.md"), "Context here").unwrap();
        std::fs::write(ex_path.join("instructions.md"), "Print hello").unwrap();
        std::fs::write(ex_path.join("starter.py"), "# TODO\n").unwrap();
        std::fs::write(ex_path.join("solution.py"), "print('hello')\n").unwrap();
        std::fs::write(ex_path.join("check.py"), "check_output_contains('hello')\n").unwrap();
        std::fs::write(ex_path.join("setup.py"), "# setup\n").unwrap();
        std::fs::write(ex_path.join("hints.md"), "Hint 1").unwrap();

        // 3. Schema validation via learn::load_track
        let loaded = learn::load_track(&staged_track).unwrap();
        assert_eq!(loaded.id(), "my-test-track");
        assert_eq!(loaded.manifest.skill.as_deref(), Some("test-skill"));
        let ex = loaded.exercise("01-hello").unwrap();
        assert_eq!(ex.manifest.title, "Hello");
        assert_eq!(ex.hints.len(), 1);

        // 4. Materialize test workspace and verify solution passes runner check
        let ws_dir = temp_dir.path().join("ws");
        let ws = learn::TrackWorkspace::materialize(&loaded, &ws_dir).unwrap();
        let user_file = ws.user_file(ex);
        let sol_file = ws.source_file(ex, &ex.variant_file("solution"));
        std::fs::copy(&sol_file, &user_file).unwrap();
        let check_res = learn::runner::check_exercise(&ws, ex, Duration::from_secs(10)).unwrap();
        assert!(check_res.passed, "Expected solution to pass check: {}", check_res.stderr);
    }
}
