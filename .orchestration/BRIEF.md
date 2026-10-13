# LearnZed — Project Brief

> NOTE (bootstrap agent): the tail of "Engineering constraints" was truncated in the
> transcript available to the bootstrap agent (marked [TRUNCATED] below). The user
> should paste the remainder; do not invent it.

## Vision
Fork of Zed (repo: lzed) turned into a DataCamp-style interactive learning app.
The user picks a learning track instead of a project, works through lessons and
coding exercises in a real editor, runs code, and gets automated checks/feedback.

## MVP Acceptance Criteria
AC1 Track picker on launch
  - On startup (no restored workspace), show a Learning Track Picker instead of
    the recent-projects picker / welcome page.
  - Lists installed tracks (title, description, progress %, last opened).
  - "Open" on a track opens its workspace folder; "Add track" imports from a
    local folder or git URL (git import = stretch).
AC2 Track workspace with .learn folder
  - Opening a track creates (first time) or reopens ~/LearnZed/<track-id>/.
  - Exercise files are materialized there as editable files
    (e.g. 03-numpy/04-subsetting-2d/script.py), copied from starter code.
  - All app state lives in <workspace>/.learn/:
      progress.json   – per-exercise status, attempts, xp, timestamps
      track.lock      – track id + version the workspace was created from
      runs/           – last run/check logs per exercise
      env/            – (optional) isolated Python venv for the track
  - Reopening never overwrites user code; "Reset exercise" restores starter.
AC3 Learn sidebar
  - New dock panel "Learn" (left dock, replaces Project Panel as default when a
    track is open; Project Panel still reachable).
  - Tree: Chapters → Lessons → Exercises with status icons
    (not started / in progress / passed).
  - Exercise view: rendered Markdown (context + instructions + XP), buttons:
    Run Code, Submit (Check), Hint (cost XP), Reset, Next.
  - Selecting an exercise opens its script file in the editor.
AC4 Run & Check
  - Run Code: executes setup + user script, output shown in a terminal/output
    pane (IPython-like shell via Zed REPL = stretch).
  - Submit: runs the checker; result shown in the panel as pass/fail with
    per-check feedback messages; on pass, mark complete, award XP, unlock next.
  - Syntax highlighting/LSP: rely on Zed's existing language support.
AC5 Sample track
  - Ship one sample track "Intro to Python" with ≥1 chapter on NumPy, including
    "Subsetting 2D NumPy Arrays" (np_baseball, np_weight_lb, 124th player).

## Track Format (v1)
tracks/<track-id>/
  track.toml                    # id, title, version, language, description
  <NN-chapter>/chapter.toml     # title, order
  <NN-chapter>/<NN-exercise>/
     exercise.toml              # id, title, xp, kind="coding", language, file="script.py"
     context.md                 # explanation shown above instructions
     instructions.md            # bullet instructions
     hints.md                   # one hint per section
     setup.py                   # pre-exercise code (defines baseball etc.), not shown
     starter.py                 # initial user file
     solution.py                # reference solution
     check.py                   # checks using the runner's helper API

## Checker design
- Language-agnostic contract: Rust side spawns a runner process and reads JSON:
  {"passed": bool, "checks": [{"name","passed","message"}], "stdout", "stderr"}
- Python runner (bundled, `learn_runner.py`): exec setup.py, then user script in
  a captured namespace; exec solution in a separate namespace; then run check.py
  with helpers: check_object(name), check_equal(name), check_output_contains(text),
  check_function_called(name), custom asserts with messages.
- Timeouts (default 10s) and process kill on timeout. No network required.

## Non-goals (MVP)
Accounts, cloud sync, payments, video, leaderboard, track authoring UI,
collab features, AI tutor (keep Zed's assistant untouched but optional).

## Engineering constraints
- Rust + GPUI, follow existing Zed crate/idiom patterns; new code in new crates
  where p[TRUNCATED]
