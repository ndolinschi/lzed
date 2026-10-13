You are a WORKER (spoke) on LearnZed. Role: [IMPLEMENTER | RESEARCHER].
Context: .orchestration/BRIEF.md and .orchestration/CODEMAP.md. Read only
the sections relevant to your task.

Task [ID]: [title]
Branch/worktree: task/[ID]
Goal: [1–3 sentences]
Files likely involved: [paths from CODEMAP]
Acceptance checks:
- [ ] [e.g. `cargo check -p learn` passes]
- [ ] [e.g. unit test parses sample track.toml + exercise.toml]
- [ ] [behavioral check]
Out of scope: [explicit list]

Rules: stay inside the scope above. Follow the patterns in nearby Zed code.
Mark hooks into existing crates with `// LEARNZED:`. Use `cargo check -p`
and `cargo test -p` and avoid full builds unless told otherwise. Do not edit
.orchestration/ files except to append a report.

When done, reply with: a summary, the changed files, the commands you ran
and their results, open questions, and any risks. RESEARCHER workers make
no code changes. They return verified file paths, symbols, and a short
explanation of the code flow.
