# HANDOFF (updated 2026-10-13)
Current milestone: MVP M0–M7 implemented on learn/main; stretch items open.

State
- `cargo test -p learn`: 16/16 pass (parsing, progress, materialization, registry, runner incl. real python+numpy, timeout, retroactive skills).
- `cargo check`/`clippy -p learn -p learn_ui --tests`: clean. Dev build `cargo build -p zed` OK → target/debug/learnzed.
- Retroactive skills bug resolved: already-completed tracks derive/award skills upon loading progress or installed tracks.
- Manual E2E done on box (PLAN.md), screenshots in .orchestration/screenshots/.

In-flight: none.

Next 3 actions
1. M7-03: gpui UI tests for learn_ui (TrackPicker/LearnPanel using TestAppContext).
2. M7-04: packaging scripts binary rename (script/bundle-*, desktop file, icons) for `learnzed` bin.
3. M7-02: D9 auto-trust for bundled tracks.

Known risks / notes
- Bin renamed `learnzed` (Zed asserts bin name == APP_NAME); script/bundle-*, zed-local still reference `zed`.
- Box has no GPU: run with lavapipe + `ZED_ALLOW_EMULATED_GPU=1`; frames lag, use `scrot -u` after waiting.
- Panel text: long check messages wrap (fixed after screenshot 03 was taken).
- Python required on host (`python3`, numpy for NumPy chapter); `.learn/env` venv used when present.
- Learner can see `.learn/track/**/solution.py` in the project panel (hidden dir) — acceptable for MVP.
- Startup hook removes onboarding on first run (intended by AC1).
