# PLAN
Milestones
- M0 Recon + scaffolding docs — DONE (CODEMAP.md).
- M1 Core model, no UI: crate `learn` (track format parsing, workspace materialization, progress.json, runner contract + bundled learn_runner.py, sample track). Plus app isolation (data dir / app name) decision.
- M2 UI: crate `learn_ui` (Track Picker AC1, Learn panel AC3, Run/Submit wiring AC4).
- M3 Polish/stretch (hints XP cost, git import, REPL).

Crate layout (proposed, see DECISIONS.md)
- crates/learn: pure logic, no GPUI view code (gpui only for App/Task if needed). Tests with `cargo test -p learn`.
- crates/learn_ui: GPUI views, depends on learn, workspace, picker, markdown, ui.
- assets/learn/learn_runner.py and tracks/intro-to-python/ (bundled sample track).

Manual E2E check (end of M2; full build by one agent only)
1. `cargo run -- --user-data-dir /tmp/learnzed-e2e` with no restorable workspace -> Track Picker shows "Intro to Python" 0%.
2. Open -> ~/LearnZed/intro-to-python/ created with .learn/{progress.json,track.lock}; Learn panel on left dock.
3. Select "Subsetting 2D NumPy Arrays" -> script.py opens, Markdown instructions render.
4. Run Code -> output in terminal. Submit with starter -> fail w/ messages. Paste solution -> pass, XP awarded, next unlocked, progress.json updated.
5. Restart -> code preserved; Reset restores starter.
(Requires python3 + numpy on the test machine.)
