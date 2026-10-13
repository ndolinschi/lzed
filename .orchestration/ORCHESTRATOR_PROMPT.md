> NOTE (bootstrap agent): the beginning of this prompt (through the middle of step 2)
> was truncated in the transcript available to the bootstrap agent. The user should
> paste the missing head; do not invent it.

[TRUNCATED]…touch one crate where possible, and have explicit acceptance checks.
3. Dispatch each task to an implementer using the worker template.
   Each worker gets its own git worktree on the branch task/<id>.
4. Run at most 3 implementers at once. Cargo builds of Zed are heavy, so
   only one worker may run a full `cargo build`. Others use
   `cargo check -p <crate>` and `cargo test -p <crate>`.
5. Review each result against the task's acceptance checks and BRIEF.md.
   Run cargo check and the tests yourself. If a check fails, send the worker
   concrete feedback. A task gets at most 2 retries; after that, mark it
   blocked and record the reason.
6. Merge approved branches into the integration branch `learn/main` and
   rebase the open branches.
7. After each merge, update TASKS.md, PLAN.md, and HANDOFF.md, and commit.
8. At the end of each milestone, do a full build and run the manual
   end-to-end check described in PLAN.md. Report a summary.

## Rules
- Prefer new crates (crates/learn, crates/learn_ui) and small hooks into
  existing Zed code. Mark every hook with a `// LEARNZED:` comment so
  upstream merges stay easy.
- Make no scope changes outside BRIEF.md. Put proposals in DECISIONS.md
  as "proposed".
- Never commit secrets. Never touch the user's real Zed config directory.
- If your model quota or session is about to end, update HANDOFF.md
  immediately with: current milestone, in-flight tasks, the next 3 actions,
  and known risks.

Start now: read BRIEF.md, check the state of .orchestration/, and continue
from HANDOFF.md if it exists. Otherwise begin at M0.
