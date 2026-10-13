//! Runner process contract: spawn, enforce timeout, parse JSON result.

use crate::{Exercise, TrackWorkspace};
use serde::{Deserialize, Serialize};
use std::io::Read as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    #[serde(default)]
    pub message: String,
}

/// JSON emitted by the runner in `check` mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunnerError {
    Spawn(String),
    Timeout(Duration),
    InvalidOutput { stdout: String, stderr: String, error: String },
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerError::Spawn(e) => write!(f, "Could not start the runner: {e}"),
            RunnerError::Timeout(d) => write!(
                f,
                "Your code took longer than {}s and was stopped. Check for infinite loops.",
                d.as_secs()
            ),
            RunnerError::InvalidOutput { error, stderr, .. } => {
                write!(f, "The runner returned invalid output ({error}). {}", stderr.trim())
            }
        }
    }
}

pub type CheckOutcome = Result<CheckResult, RunnerError>;

pub fn parse_check_output(stdout: &str, stderr: &str) -> CheckOutcome {
    serde_json::from_str::<CheckResult>(stdout.trim()).map_err(|e| RunnerError::InvalidOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        error: e.to_string(),
    })
}

/// Run a process to completion, killing it after `timeout`. Returns (stdout, stderr).
pub fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<(String, String), RunnerError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|e| RunnerError::Spawn(e.to_string()))?;
    let mut out = child.stdout.take().expect("piped");
    let mut err = child.stderr.take().expect("piped");
    let out_thread = std::thread::spawn(move || {
        let mut s = String::new();
        out.read_to_string(&mut s).ok();
        s
    });
    let err_thread = std::thread::spawn(move || {
        let mut s = String::new();
        err.read_to_string(&mut s).ok();
        s
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() >= timeout => {
                child.kill().ok();
                child.wait().ok();
                return Err(RunnerError::Timeout(timeout));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(RunnerError::Spawn(e.to_string())),
        }
    }
    Ok((out_thread.join().unwrap_or_default(), err_thread.join().unwrap_or_default()))
}

/// Program and arguments for `run` mode (used for the terminal "Run Code").
pub fn run_command(ws: &TrackWorkspace, exercise: &Exercise) -> (PathBuf, Vec<String>) {
    (
        ws.python(),
        vec![
            ws.runner_script().to_string_lossy().into_owned(),
            "run".into(),
            "--setup".into(),
            ws.source_file(exercise, "setup.py").to_string_lossy().into_owned(),
            "--script".into(),
            ws.user_file(exercise).to_string_lossy().into_owned(),
        ],
    )
}

/// Run the checker for an exercise (blocking) and store the log in `.learn/runs/`.
pub fn check_exercise(ws: &TrackWorkspace, exercise: &Exercise, timeout: Duration) -> CheckOutcome {
    let mut command = Command::new(ws.python());
    command
        .current_dir(ws.root.join(&exercise.rel_dir))
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(ws.runner_script())
        .arg("check")
        .arg("--setup")
        .arg(ws.source_file(exercise, "setup.py"))
        .arg("--script")
        .arg(ws.user_file(exercise))
        .arg("--solution")
        .arg(ws.source_file(exercise, "solution.py"))
        .arg("--check")
        .arg(ws.source_file(exercise, "check.py"));
    let outcome = run_with_timeout(command, timeout).and_then(|(out, err)| parse_check_output(&out, &err));
    let log = match &outcome {
        Ok(result) => serde_json::to_string_pretty(result).unwrap_or_default(),
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    };
    let runs = ws.runs_dir();
    std::fs::create_dir_all(&runs).ok();
    std::fs::write(runs.join(format!("{}.json", exercise.id())), log).ok();
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sh(script: &str) -> Command {
        let mut c = Command::new("sh");
        c.arg("-c").arg(script);
        c
    }

    #[test]
    fn parses_contract() {
        let r = parse_check_output(
            r#"{"passed": false, "checks": [{"name": "a", "passed": false, "message": "m"}], "stdout": "x", "stderr": ""}"#,
            "",
        )
        .unwrap();
        assert!(!r.passed);
        assert_eq!(r.checks[0].message, "m");
        assert!(matches!(parse_check_output("Traceback", "boom"), Err(RunnerError::InvalidOutput { .. })));
    }

    #[cfg(unix)]
    #[test]
    fn fake_runner_pass_and_timeout() {
        let (out, _) = run_with_timeout(sh(r#"echo '{"passed": true}'"#), DEFAULT_TIMEOUT).unwrap();
        assert!(parse_check_output(&out, "").unwrap().passed);

        let start = Instant::now();
        let r = run_with_timeout(sh("sleep 5"), Duration::from_millis(300));
        assert_eq!(r, Err(RunnerError::Timeout(Duration::from_millis(300))));
        assert!(start.elapsed() < Duration::from_secs(2));

        assert!(matches!(
            run_with_timeout(Command::new("/nonexistent/runner"), DEFAULT_TIMEOUT),
            Err(RunnerError::Spawn(_))
        ));
    }

    /// End-to-end with the real Python runner (skipped if python3/numpy missing).
    #[test]
    fn real_runner_checks_sample_exercise() {
        let has_numpy = Command::new("python3")
            .args(["-c", "import numpy"])
            .status()
            .is_ok_and(|s| s.success());
        if !has_numpy {
            eprintln!("skipping: python3 with numpy not available");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let track = crate::load_track(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tracks/intro-to-python")).unwrap();
        let ws = TrackWorkspace::materialize(&track, dir.path()).unwrap();
        let ex = ws.track.exercise("subsetting-2d").unwrap().clone();

        let r = check_exercise(&ws, &ex, DEFAULT_TIMEOUT).unwrap();
        assert!(!r.passed);
        assert!(r.checks.iter().any(|c| !c.passed && c.message.contains("np_weight_lb")));
        assert!(ws.runs_dir().join("subsetting-2d.json").is_file());

        std::fs::copy(ws.source_file(&ex, "solution.py"), ws.user_file(&ex)).unwrap();
        let r = check_exercise(&ws, &ex, DEFAULT_TIMEOUT).unwrap();
        assert!(r.passed, "{r:?}");

        std::fs::write(ws.user_file(&ex), "while True: pass").unwrap();
        assert!(matches!(
            check_exercise(&ws, &ex, Duration::from_secs(1)),
            Err(RunnerError::Timeout(_))
        ));

        std::fs::write(ws.user_file(&ex), "1/0").unwrap();
        let r = check_exercise(&ws, &ex, DEFAULT_TIMEOUT).unwrap();
        assert!(!r.passed);
        assert!(r.checks[0].message.contains("ZeroDivisionError"));
    }
}
