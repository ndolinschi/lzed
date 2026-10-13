# PLAN (milestones match BRIEF.md)
- M0 Recon → CODEMAP.md — DONE
- M1 App identity + data dir separation — DONE (D3)
- M2 crates/learn: parsing, materialization, progress.json — DONE (13 unit tests)
- M3 Track picker on launch (AC1) — DONE (git import = stretch, not done)
- M4 Learn panel (AC3) — DONE
- M5 Runner + check integration (AC4) — DONE (REPL = stretch, not done)
- M6 Sample track (AC5) + E2E script — DONE (see E2E below)
- M7 Polish: XP, hints, reset, next, status icons — DONE (basic)

## Build / run (dev profile only)
    source ~/.cargo/env
    CARGO_PROFILE_DEV_DEBUG=0 cargo test -p learn
    CARGO_PROFILE_DEV_DEBUG=0 cargo check -p learn_ui
    CARGO_PROFILE_DEV_DEBUG=0 cargo build -p zed      # binary: target/debug/learnzed
    ZED_ALLOW_EMULATED_GPU=1 target/debug/learnzed --user-data-dir /tmp/lz-data
(Box has no GPU: mesa-vulkan-drivers/lavapipe installed; screenshots need `scrot -u` — root-window captures can be stale.)

## Manual E2E (verified 2026-10-13, screenshots in .orchestration/screenshots/)
1. Fresh `--user-data-dir`, no ~/LearnZed → Track Picker shows "Intro to Python 0%" (01).
2. Start → ~/LearnZed/intro-to-python/ with .learn/{track.lock,track/,runs/,learn_runner.py}; Learn panel active in left dock, first exercise opened (02). Restricted-mode prompt → Trust.
3. Subsetting 2D: Submit starter → "Incorrect submission" + per-check messages (03).
4. Type solution → Submit → "All checks passed +100 XP", 100% (04). Run Code → output in terminal (05).
5. progress.json updated; reopening keeps code; Reset restores starter (unit-tested; UI prompt).
