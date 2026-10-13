# CODEMAP (M0 recon, verified against main @ 26363aabdb, 2026-10-13)
All paths relative to repo root; line numbers are approximate anchors (may drift).

## Startup flow (AC1)
- crates/zed/src/main.rs: `fn main()` (~L201); `--user-data-dir` arg (`user_data_dir`, ~L1713) -> `paths::set_custom_data_dir` (~L263).
- crates/zed/src/main.rs: `pub(crate) async fn restore_or_create_workspace` (~L1422).
  Branches: (a) `restorable_workspaces` -> restore; (b) `kvp.read_kvp(FIRST_OPEN)` None -> `onboarding::show_onboarding_view` (~L1556);
  (c) else `workspace::open_new(...)` + `RestoreOnStartupBehavior::Launchpad` / `Editor::new_file` (~L1559).
  **AC1 hook**: branch (c) (and optionally (b)) -> open Learning Track Picker instead. Mark `// LEARNZED:`.
- crates/onboarding/src/onboarding.rs: `FIRST_OPEN`, `show_onboarding_view` (~L183), `Onboarding` impl `Item`, `SerializableItem`.
- crates/workspace/src/welcome.rs: `WelcomePage` (~L240, `WelcomePage::new` ~L248, `impl Render` ~L414, has `fallback_to_recent_projects`); action namespace `welcome`. Good template for a full-pane picker Item.
- crates/recent_projects/src/recent_projects.rs: `RecentProjects` (ModalView, ~L610), `RecentProjectsDelegate` (PickerDelegate, ~L853), `get_recent_projects` (~L121), `init` (~L286). Template for a picker-based track list.
- crates/onboarding/src/base_keymap_picker.rs: small `PickerDelegate` example (`BaseKeymapSelectorDelegate`).
- crates/settings_content/src/workspace.rs: `RestoreOnStartupBehavior` { EmptyTab, LastWorkspace, LastSession(default), Launchpad } (~L521-531).
- crates/workspace/src/workspace.rs: `pub fn open_paths` (free fn ~L11153), `pub fn open_new` (~L11359) — use to open ~/LearnZed/<track-id>/.

## Dock / panels (AC3)
- crates/workspace/src/dock.rs: `pub trait Panel` (~L36), `PanelHandle` (~L106), `pub enum DockPosition` (~L324).
- crates/workspace/src/workspace.rs: `Workspace::add_panel` (~L2828), `focus_panel` (~L4683), `toggle_panel_focus` (~L4696).
- crates/project_panel/src/project_panel.rs: `impl Panel for ProjectPanel` (~L8175: position, default_size, persistent_name, starts_open), `ProjectPanel::load`, `init` (~L501). Reference impl for LearnPanel.
- crates/zed/src/zed.rs: `fn initialize_panels` (~L777) with `add_panel_when_ready(...)` for project/outline/terminal/git/channels/debug panels (~L803-808). **AC3 hook**: add `LearnPanel::load` here (`// LEARNZED:`). Left-dock default: LearnPanel `starts_open` true when workspace has `.learn/`, and ProjectPanel stays registered.
- crates/outline_panel/: second smaller Panel example.

## Markdown rendering (AC3 exercise view)
- crates/markdown/src/markdown.rs: `pub struct Markdown` (~L487, `Markdown::new` ~L654), `pub struct MarkdownStyle` (~L109), `pub struct MarkdownElement` (~L1731, `MarkdownElement::new(markdown, style)` ~L1753).
- crates/markdown_preview/: full-file preview (alternative).

## Execution (AC4)
- crates/task/src/task.rs: `SpawnInTerminal` (~L42), `ResolvedTask` (~L114); crates/task/src/task_template.rs: `TaskTemplate` (~L24).
- crates/workspace/src/tasks.rs: `Workspace::schedule_resolved_task` (~L64), `schedule_resolved_task_with_completion` (~L82), `spawn_in_terminal` (~L222). **Run Code hook**: build a `TaskTemplate`/`SpawnInTerminal` running `python learn_runner.py run ...` and schedule it -> output in terminal panel.
- crates/project/src/terminals.rs: `create_terminal_task` (~L65). crates/terminal/src/terminal.rs: `Terminal` (~L1522), `TerminalBuilder` (~L978). crates/tasks_ui/src/tasks_ui.rs: `spawn_task_or_modal` (~L185).
- **Submit/Check**: do NOT use terminal; spawn runner headless via `util::command::new_command` (crates/util/src/command.rs ~L16) in a background task, parse JSON, enforce timeout, render in panel.
- crates/repl/src/: `init` (repl.rs ~L31), `ReplStore` (repl_store.rs ~L27), `Session` (session.rs ~L47) — stretch only.

## Paths / app identity (M1 isolation)
- crates/paths/src/paths.rs: `APP_NAME = "Zed"` (~L18), `APP_NAME_LOWERCASE`, `set_custom_data_dir` (~L103), `config_dir` (~L122), `data_dir` (~L144), `logs_dir` (~L228), `database_dir` (~L258), `home_dir`.
- crates/release_channel/src/lib.rs: `RELEASE_CHANNEL_NAME` (~L13), `RELEASE_CHANNEL` (~L37), `app_identifier()` (~L45), `ReleaseChannel` (~L139), `display_name` ("Zed" for Stable, ~L206), `dev_name`, `app_id` (~L228).
- crates/db/src/db.rs: re-exports `paths::database_dir`; `DB_FILE_NAME = "db.sqlite"` (~L138); `open_db` (~L174).
- Safe dev isolation today without code changes: run with `--user-data-dir /tmp/learnzed-dev` (config under <dir>/config).

## Workspace layout
- Cargo.toml: `[workspace] members = [...]`, `default-members = ["crates/zed"]`. New crates must be added to members + workspace deps.
- 249 crates under crates/. Agent guidance files: AGENTS.md, CLAUDE.md, GEMINI.md (read before coding).
