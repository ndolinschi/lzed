# TASKS
Status: todo | claimed:<agent> | review | done | blocked:<reason>

## M1 — core (crate `learn`)
| ID | Title | Est | Status |
|---|---|---|---|
| M1-01 | Scaffold crates/learn (Cargo.toml, lib.rs, add to workspace members) | 30m | todo |
| M1-02 | Track format types + TOML parsing (track/chapter/exercise) + loader walking dirs, sorted by NN prefix | 60m | todo |
| M1-03 | Sample track tracks/intro-to-python (NumPy chapter incl. subsetting-2d) | 60m | todo |
| M1-04 | learn_runner.py: run + check modes, helpers, JSON output | 90m | todo |
| M1-05 | Workspace materialization ~/LearnZed/<id>/ + .learn/track.lock, never overwrite, reset_exercise | 60m | todo |
| M1-06 | progress.json model (status, attempts, xp, timestamps), load/save atomic, progress % | 45m | todo |
| M1-07 | Runner invocation from Rust: spawn, timeout+kill, parse JSON, write .learn/runs/ | 90m | todo |
| M1-08 | Track registry: discover installed tracks (bundled + ~/LearnZed), last opened | 45m | todo |
| M1-09 | Isolation decision: app data dir/name (DECISIONS D3) — RESEARCH only until approved | 30m | todo |

Acceptance checks
- M1-01: `cargo check -p learn` passes; crate listed in root Cargo.toml.
- M1-02: unit tests parse fixtures for all 3 toml kinds; missing optional fields default; bad toml -> error with path.
- M1-03: loader test loads the sample track; exercise "subsetting-2d" has np_baseball/np_weight_lb setup; `python3 learn_runner.py check` passes on solution.py and fails on starter.py.
- M1-04: running solution -> {"passed":true}; starter -> passed:false with per-check messages; infinite loop is killed by timeout (test from Rust in M1-07). Schema exactly per BRIEF.
- M1-05: tests in tempdir: first open copies starter -> script.py; second open with edited script preserves edit; reset restores starter; track.lock contains id+version.
- M1-06: round-trip test; percent computed; corrupted file -> backed up + fresh state (no panic).
- M1-07: test with tiny fake runner script: pass, fail, timeout (killed <= timeout+1s), non-JSON output -> error result.
- M1-08: test with tempdir HOME override (no real home touched).
- M1-09: written proposal in DECISIONS.md; no code.

## M2 — UI (crate `learn_ui`)
| ID | Title | Est | Status |
|---|---|---|---|
| M2-01 | Scaffold crates/learn_ui + init() wired in crates/zed (`// LEARNZED:`) | 30m | todo |
| M2-02 | Track Picker view (Item like WelcomePage or Picker like RecentProjects) listing tracks w/ progress | 90m | todo |
| M2-03 | Startup hook in restore_or_create_workspace -> show picker when nothing restored | 45m | todo |
| M2-04 | Open track action: materialize + workspace::open_paths on ~/LearnZed/<id> | 45m | todo |
| M2-05 | LearnPanel (impl Panel) with chapter/exercise tree + status icons; register in initialize_panels | 90m | todo |
| M2-06 | Exercise view: Markdown context+instructions+XP; select opens script.py | 60m | todo |
| M2-07 | Run Code via TaskTemplate/schedule_resolved_task in terminal | 60m | todo |
| M2-08 | Submit: headless runner, show per-check results, award XP, unlock next, persist | 90m | todo |
| M2-09 | Reset / Next / Hint buttons | 60m | todo |
| M2-10 | Milestone full build + manual E2E (PLAN.md) — single agent only | 60m | todo |

Acceptance checks
- M2-01: `cargo check -p learn_ui -p zed` passes (check, not build).
- M2-02: `cargo check -p learn_ui`; gpui test renders picker with 1 fixture track.
- M2-03: only main.rs touched in existing crates; restore path unchanged when a workspace is restorable.
- M2-04: gpui test (TestAppContext + FakeFs where possible) opens workspace at track dir.
- M2-05: panel appears in left dock; ProjectPanel still toggleable; persistent_name "LearnPanel".
- M2-06: selecting exercise opens file in active pane (gpui test).
- M2-07: task label "Learn: Run <exercise>"; uses .learn/env python if present else python3.
- M2-08: pass updates progress.json and panel icon; fail shows messages; timeout shows message.
- M2-09: Reset asks confirm and restores starter; Hint deducts XP once per hint.
- M2-10: all PLAN.md E2E steps pass; report.
