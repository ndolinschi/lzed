# TASKS (status as of 2026-10-13)
| ID | Title | Status |
|---|---|---|
| M1-01 | APP_NAME=LearnZed, release channel names/app ids, bin rename | done |
| M2-01 | crates/learn scaffold + workspace member | done |
| M2-02 | Track format parsing + loader (tests) | done |
| M2-03 | Materialization ~/LearnZed/<id>, .learn/{track.lock,track,runs}, reset | done |
| M2-04 | progress.json (atomic save, corrupt backup, %, unlock, XP) | done |
| M2-05 | Registry: bundled tracks extraction, installed list, import folder | done |
| M3-01 | TrackPicker item + startup hook (main.rs) | done |
| M3-02 | Add track from local folder | done (untested in UI) |
| M3-03 | Add track from git URL | todo (stretch) |
| M4-01 | LearnPanel (Panel impl, tree, status icons) + initialize_panels hook | done |
| M4-02 | Exercise markdown view (context + instructions + revealed hints) | done |
| M5-01 | learn_runner.py run/check modes + helpers | done |
| M5-02 | Rust runner spawn/timeout/JSON + runs/ log (tests incl. real python) | done |
| M5-03 | Run Code in terminal, Submit in panel | done |
| M5-04 | IPython-like REPL via Zed REPL | todo (stretch) |
| M6-01 | Sample track (4 exercises incl. Subsetting 2D) | done |
| M6-02 | E2E manual test (PLAN.md) + screenshots | done |
| M7-01 | Hint (XP cost), Reset (confirm), Next (after pass) | done |
| M7-02 | Auto-trust bundled tracks (D9) | done |
| M7-03 | gpui tests for learn_ui (picker/panel) | done |
| M7-04 | Update script/bundle-* for `learnzed` bin, desktop file, icons | done |
| M7-05 | Award/derive skills for already-completed tracks | done |

## Approved by user (2026-10-13): AI skills for Zed's agent chat
| id | title | milestone | owner | status | branch | notes |
|----|-------|-----------|-------|--------|--------|-------|
| AI-1 | Generate `.rules` (agent rules file) into each track workspace on materialize: tutor persona, no solution leaks, how to read .learn/progress.json, exercise layout, how to interpret check results | M8 | | done | task/AI-1 | Zed auto-loads .rules into agent context |
| AI-2 | Agent tools for learning (context server / MCP `learnzed` or built-in tools): get_current_exercise, run_check (returns runner JSON), get_hint(level), get_progress/skills | M8 | | todo | task/AI-2 | no tool returns solution.py |
| AI-3 | Slash commands / prompt library entries in agent chat: /exercise, /check, /hint, /explain-error | M8 | | todo | task/AI-3 | |
| AI-4 | "Generate track" (approved by user): button in track picker -> AI interview in Zed agent chat (asks goals, current level, language/stack, time budget), then generates a valid track (track.toml, chapters, exercises with starter/solution/check.py, layout) into ~/LearnZed/.tracks/<id>, validates with learn parser + runs solution through checker, registers it in picker. Same flow for "Generate skill" (short track awarding a skill). Uses Zed language_model; if no provider -> connect prompt. | M8 | | todo | task/AI-4 | after AI-1..AI-3 |
