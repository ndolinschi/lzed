# HANDOFF (updated 2026-10-13)
Current milestone: MVP M0–M7 implemented on learn/main; stretch items open.

State
- `cargo test -p learn`: 19/19 pass (including review exercises and planted bugs).
- `cargo test -p learn_ui`: 4/4 pass.
- `cargo test -p agent_skills`: 80/80 pass.
- AI-1 implemented: `TrackWorkspace::materialize` creates `.rules` with Socratic tutor persona, strict no-leak policy against `.learn/track/**/solution.*`, workspace layout, `.learn/progress.json` schema, and check result guidance.
- AI-2 implemented: Built-in agent tools (`get_current_exercise`, `run_check`, `get_hint`, `get_progress`), registered in `Thread::add_default_tools` and allowed in default profiles. Defensive read restriction added to `read_file_tool` to block solution leaks. Unit tests pass.
- AI-3 implemented: Builtin learning skills (`exercise`, `check`, `hint`, `explain-error`) registered in `agent_skills::builtin_skills()`, recognized by message editor slash commands and dispatching to agent with pedagogical instructions. Unit tests pass (80/80 in `agent_skills`).
- AI-4 implemented: "Generate track" and "Generate skill" buttons added to track picker, triggering AI interview in agent chat (`/generate-track`, `/generate-skill`). `GenerateTrackTool` writes `track.toml`, chapters, and exercises, validates schema with `learn::load_track`, tests solution against `check.py` with runner, and registers track in picker. Unit tests pass.

Screenshots captured (DISPLAY=:5, /workspace/learnzed-shots/):
- `agy-01-picker.png`: Track Picker showing "Generate Track", "Generate Skill", "Import Track", user profile with earned skills, and career paths.
- `agy-02-learn-panel.png`: Learn panel with full exercise view (instructions, code, Run code, Hint, Ask AI).
- `agy-03-ask-ai.png`: Ask AI in Agent panel alongside editor.
- `agy-04-slash-commands.png`: Slash commands popup showing `/exercise`, `/check`, `/hint`, `/explain-error`, `/generate-track`, `/generate-skill`.
- `agy-05-generate-track.png`: `/generate-track` interactive prompt in Agent panel.

In-flight: none.

Next 3 actions
1. User review of AI-1 through AI-4.
2. AI-5: Review exercises (kind="review" with planted bugs) - in review on task/claude-tracks.
3. TR-1: Claude AI Engineering tracks - in review on task/claude-tracks.


Known risks / notes
- Bin renamed `learnzed` (Zed asserts bin name == APP_NAME); script/bundle-*, zed-local still reference `zed`.
- Box has no GPU: run with lavapipe + `ZED_ALLOW_EMULATED_GPU=1`; frames lag, use `scrot -u` after waiting.
- Panel text: long check messages wrap (fixed after screenshot 03 was taken).
- Python required on host (`python3`, numpy for NumPy chapter); `.learn/env` venv used when present.
- Learner can see `.learn/track/**/solution.py` in the project panel (hidden dir) — acceptable for MVP.
- Startup hook removes onboarding on first run (intended by AC1).
