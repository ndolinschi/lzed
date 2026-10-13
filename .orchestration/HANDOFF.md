# HANDOFF (updated 2026-10-13 by bootstrap agent)
Current milestone: M0 DONE -> start M1.
Branch: learn/main (integration). No code changes yet; only .orchestration/.

State
- CODEMAP.md verified by rg against main.
- DECISIONS D2-D5 are "proposed".

In-flight tasks: none.

Next 3 actions
1. Get missing BRIEF/orchestrator text from user; commit.
2. Dispatch M1-01, then M1-02 / M1-04 / M1-03 in parallel (max 3).
3. Resolve D3 (isolation) before any M2 task that runs the app.

Risks
- Full Zed build is heavy (one agent only); first `cargo check -p` will also be slow.
- Runner needs python3 + numpy on host; no network allowed in checks.
- Startup hook touches main.rs which changes frequently upstream; keep it tiny.
- Never run the app without --user-data-dir until D3 decided (protect real Zed config).
