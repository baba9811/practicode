# Architecture

Practicode is local-first: user data stays under `PRACTICODE_HOME` or `~/.practicode` by default.

## Source Layout

- `src/core.rs` is the public core facade. Keep new domain logic in nested `src/core/` modules.
- `src/core/model.rs` owns persisted/user-facing data shapes and core constants.
- `src/core/bank.rs` owns local problem-bank loading, saving, starter data, and bank validation.
- `src/core/state.rs` owns state loading, saving, and settings normalization.
- `src/core/learning.rs` owns deterministic mastery transitions, due-review selection, completion, and legacy progress migration.
- `src/core/language.rs` owns language/provider normalization, templates, and extension mapping.
- `src/core/render.rs` owns plain/markdown problem rendering.
- `src/core/judge.rs` owns submission file creation, runtime commands, compilation, and judging.
- `src/core/syntax.rs` validates and loads embedded executable course assets plus localized lesson copy.
- `src/core/progress.rs` owns give-up/next/previous/pass history transitions.
- `src/core/problem_files.rs` owns generated problem README/index file writes.
- `src/core/profile.rs` owns user-profile defaults and normalization helpers.
- `src/lib.rs` resolves the application data root and migrates legacy current-directory data before loading the TUI.
- `src/tui.rs` owns the `PracticodeApp` state shell, construction, run loop, and test accessors. Keep new TUI behavior in nested `src/tui/` modules.
- `src/tui/actions.rs` owns user actions such as run, next, generate, language/theme/profile changes.
- `src/tui/command_handlers.rs` owns slash-command routing.
- `src/tui/command_input.rs` owns command palette input, completion, and Hangul composition.
- `src/tui/events.rs` owns keyboard/mouse event routing.
- `src/tui/learning.rs` owns the Review/Delta/Predict/Exercise/Reflect session queue and learning views.
- `src/tui/tasks.rs` owns background AI/update/model tasks and output writing helpers.
- `src/tui/view.rs` owns Ratatui drawing, pane styling, mouse-capture toggles, and cursor placement.
- `src/tui/problem_list.rs` owns problem-list rendering and navigation.
- `src/tui/status.rs` owns status-line text, busy-game text, mode hints, and help text.
- `src/tui/commands.rs` owns the command palette catalog.
- `src/tui/editor.rs` owns the in-terminal code editor state.
- `src/tui/problem_view.rs` owns problem-statement rendering.
- `src/tui/settings_panel.rs` owns `/profile` setup-panel rendering and keyboard toggles.
- `src/ai.rs` owns provider commands, daemon/model checks, and AI prompts for foreground `/next` generation and background `/generate` prefetch.
- `src/update.rs` owns update checks.
- `src/text.rs` owns terminal text editing and markdown/plain rendering helpers.
- `assets/i18n/*.json` stores UI labels and command text.
- `assets/lessons/<programming-language>/course.json` stores ordered executable lessons, cases, and primary references.
- `assets/lessons/<programming-language>/<ui-language>.json` stores required lesson study copy, while `review-manifest.json` records independently reviewed content hashes. See [../assets/lessons/README.md](../assets/lessons/README.md).
- `bin/launcher.js` maps npm installs to versioned GitHub Release binaries, verifies checksums, and owns the per-user binary cache. `bin/practicode.js` is only its executable entry point.

## Extension Rules

- Add domain logic under the owning nested module first; keep `core.rs` and `tui.rs` as facades/shells, not catch-alls.
- Add user-visible commands in `src/tui/commands.rs`, then route behavior in `PracticodeApp::handle_command`.
- Add persisted user profile settings to `Settings`, normalize them in `normalize_settings`, and cover old-state compatibility with tests.
- Add syntax lesson copy under `assets/lessons/<programming-language>/<ui-language>.json`; every supported UI language must define the required study fields.
- Keep deterministic cases and executable behavior in `course.json`, never in localized prose. Refresh review evidence only after an independent agent verifies the final hash.
- Keep provider-specific behavior in `src/ai.rs`; TUI should ask for status or start tasks, not know provider internals.
- Keep foreground and background generation flows separate: `/next` may block when no local problem exists, while `/generate` must preserve the current problem and user profile state.
- Keep output panes copy-friendly. Mouse capture should be enabled for the visible code editor, but disabled while output, hints, answers, lists, or settings panels are shown so terminal drag selection keeps working.
- Keep local user data backwards-compatible. Missing fields should default cleanly.

## Release

Tags build five native binaries before crates.io/npm publication. The npm native path never invokes Cargo; `--docker` remains an explicit source-image build. See [MAINTAINING.md](MAINTAINING.md).
