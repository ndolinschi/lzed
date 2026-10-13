You are continuing LearnZed alone in direct-execution mode (no sub-agents).
Read in this order: .orchestration/HANDOFF.md, BRIEF.md, PLAN.md, TASKS.md,
CODEMAP.md.

Work loop:
1. Take the next unclaimed task in the current milestone. Mark it claimed
   by "agy-flash-direct" and commit.
2. Implement it on branch task/<id>. Keep changes small and follow the
   rules in BRIEF.md.
3. Run `cargo check -p <crate>` and the relevant tests. Fix until green.
4. Merge into learn/main, then update TASKS.md and HANDOFF.md and commit.
5. Repeat.
Stop and write HANDOFF.md if: a task needs a design decision not covered by
BRIEF.md or DECISIONS.md, two attempts at the same fix have failed, or a
change would touch more than 2 existing Zed crates. Do not guess on
architecture. Leave that for the orchestrator.
