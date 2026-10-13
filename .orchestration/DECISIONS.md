# DECISIONS
- D1 (accepted, from BRIEF): new code in crates/learn + crates/learn_ui; hooks marked `// LEARNZED:`.
- D2 (proposed): Submit/Check runs headless (util::command::new_command), not in terminal; Run Code uses Zed tasks in terminal.
- D3 (proposed): App isolation. Options: (a) dev via `--user-data-dir` only (no code change); (b) change paths::APP_NAME to "LearnZed" (touches paths + release_channel display names, affects all dirs). Recommend (a) for M1/M2, (b) later. Needs user/orchestrator approval.
- D4 (proposed): Track Picker as full-pane Item modeled on workspace/src/welcome.rs WelcomePage (not a modal), using a Picker list inside.
- D5 (proposed): Sample tracks bundled under assets/learn/tracks, copied/registered at first run.
