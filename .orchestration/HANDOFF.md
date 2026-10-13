# HANDOFF (updated 2026-10-13)
Current milestone: MVP M0–M7 implemented on learn/main; stretch items open.

State
- `cargo test -p learn`: 17/17 pass (includes materialize_creates_agent_rules).
- `cargo test -p learn_ui`: 4/4 pass.
- AI-1 implemented: `TrackWorkspace::materialize` creates `.rules` with Socratic tutor persona, strict no-leak policy against `.learn/track/**/solution.*`, workspace layout, `.learn/progress.json` schema, and check result guidance.

In-flight: none.

Next 3 actions
1. AI-2: Agent tools for learning (context server / MCP `learnzed` or built-in tools).
2. AI-3: Slash commands / prompt library entries in agent chat (/exercise, /check, /hint, /explain-error).
3. Dev build & screenshots via scrot.


Known risks / notes
- Bin renamed `learnzed` (Zed asserts bin name == APP_NAME); script/bundle-*, zed-local still reference `zed`.
- Box has no GPU: run with lavapipe + `ZED_ALLOW_EMULATED_GPU=1`; frames lag, use `scrot -u` after waiting.
- Panel text: long check messages wrap (fixed after screenshot 03 was taken).
- Python required on host (`python3`, numpy for NumPy chapter); `.learn/env` venv used when present.
- Learner can see `.learn/track/**/solution.py` in the project panel (hidden dir) — acceptable for MVP.
- Startup hook removes onboarding on first run (intended by AC1).
