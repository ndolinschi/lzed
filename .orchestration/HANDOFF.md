# HANDOFF (updated 2026-10-13)
Current milestone: MVP M0–M7 implemented on learn/main; stretch items open.

State
- `cargo test -p learn`: 13/13 pass (parsing, progress, materialization, registry, runner incl. real python+numpy, timeout).
- `cargo check`/`clippy -p learn -p learn_ui --tests`: clean. Dev build `cargo build -p zed` OK → target/debug/learnzed.
- Manual E2E done on box (PLAN.md), screenshots in .orchestration/screenshots/.

In-flight: none.

Next 3 actions
1. D9: auto-trust bundled track workspaces (avoid Restricted Mode prompt) — needs orchestrator OK.
2. gpui tests for TrackPicker/LearnPanel (TestAppContext) — M7-03.
3. Stretch: git-URL import (M3-03), REPL runner (M5-04), packaging (M7-04).

Known risks / notes
- Bin renamed `learnzed` (Zed asserts bin name == APP_NAME); script/bundle-*, zed-local still reference `zed`.
- Box has no GPU: run with lavapipe + `ZED_ALLOW_EMULATED_GPU=1`; frames lag, use `scrot -u` after waiting.
- Panel text: long check messages wrap (fixed after screenshot 03 was taken).
- Python required on host (`python3`, numpy for NumPy chapter); `.learn/env` venv used when present.
- Learner can see `.learn/track/**/solution.py` in the project panel (hidden dir) — acceptable for MVP.
- Startup hook removes onboarding on first run (intended by AC1).
