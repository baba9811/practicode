# Global Data Home Design

## Goal

Make globally installed `practicode` behave like a standalone user application: every invocation uses one stable user-data directory instead of treating the shell's current directory as a project workspace, and AI commands work without requiring that directory to be a Git repository.

## Chosen Approach

Use `PRACTICODE_HOME` when it is set and non-empty; otherwise use `$HOME/.practicode` on Unix-like systems or `%USERPROFILE%\.practicode` on Windows. Return an actionable startup error when none of those locations is available.

The resolved directory is the application root. It contains:

```text
~/.practicode/
├── problem-state.json
├── problem_bank.json
├── problem_notes.md
├── problems/
├── submissions/
└── build/
```

This keeps Codex's writable workspace limited to application data instead of granting it the user's entire home directory.

## Alternatives Considered

1. Keep current-directory storage and only add `--skip-git-repo-check`. This is the smallest patch, but a globally installed application would continue scattering state across whichever directories users happen to launch it from.
2. Store settings globally while keeping problems and submissions per directory. This supports multiple workspaces, but introduces profile/workspace synchronization and selection behavior that the product does not currently expose.
3. Store all application data under one global directory. This matches the npm and Cargo global-install experience, removes current-directory dependence, and is the chosen approach.

## Root Resolution

Add a small standard-library-only resolver used by every CLI entry path, including `--smoke`. The resolver must not inspect or use the npm package directory. It resolves paths in this order:

1. Non-empty `PRACTICODE_HOME` exactly as supplied.
2. Non-empty `HOME` joined with `.practicode`.
3. Non-empty `USERPROFILE` joined with `.practicode`.
4. An error explaining that `PRACTICODE_HOME` can be set explicitly.

Core functions continue accepting an explicit root so unit and integration tests remain isolated.

## Legacy Data Migration

Before loading state, inspect only the launch directory captured at process start. A directory is a legacy practicode workspace only when `.practicode/problem-state.json` or `.practicode/problem_bank.json` exists.

If the new application root has no state or problem bank, copy legacy data non-destructively:

- `.practicode/problem-state.json`, `.practicode/problem_bank.json`, and `.practicode/problem_notes.md` to the new root;
- `problems/` to `<root>/problems/`;
- `submissions/` to `<root>/submissions/`;
- never copy `.practicode/build/`, because it is disposable compiled output.

Never overwrite a destination file and never delete legacy data. If the destination already has state or a bank, skip migration entirely to avoid merging two histories. Migration failure stops startup with source and destination context rather than silently starting with empty data.

## AI Commands

All Codex command paths use the resolved application root as `--cd` and process working directory. Add `--skip-git-repo-check` because an application data directory is intentionally not a Git repository. Preserve the existing `workspace-write` sandbox for generation and `read-only` sandbox for coaching prompts.

Claude behavior remains unchanged apart from receiving the new application root.

Update generated-problem prompts to reference `problem-state.json`, `problem_bank.json`, and `problem_notes.md` at the application root. Generated problem files remain under `problems/` and submissions under `submissions/`.

## Documentation and Compatibility

Document `~/.practicode` as the default location and `PRACTICODE_HOME` as the override. Remove claims that data is saved in the current directory or ignored by the current Git repository. Contributor fixtures may continue passing temporary roots directly.

This is a storage-layout change. The non-destructive legacy copy covers users who launch the new release from their previous practice directory. Users with multiple old workspaces can migrate another one by setting `PRACTICODE_HOME` to an empty destination and launching once from that workspace.

## Testing

Add focused tests for:

- root precedence across `PRACTICODE_HOME`, `HOME`, and `USERPROFILE`;
- the missing-home error;
- non-destructive legacy migration, including build-cache exclusion;
- migration being skipped when the destination already contains data;
- the generated Codex command containing `--skip-git-repo-check` and the resolved root;
- all existing state, problem, submission, syntax, judge, TUI, smoke, npm packaging, formatting, and Clippy checks.

Release verification must run `make test`, `cargo run -- --smoke` with an isolated `PRACTICODE_HOME`, and an npm-packed launcher smoke check with an isolated `PRACTICODE_HOME` before tagging the next patch release.

## Release

Release as `0.1.20` through the repository's tag-based workflow after the implementation commit is on `main`. Verify the GitHub Actions release job and confirm both npm and crates.io report `0.1.20`.
