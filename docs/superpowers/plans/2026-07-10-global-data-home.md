# Global Data Home Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Store every globally installed practicode user's state, generated problems, submissions, and build output under one stable `~/.practicode` directory and let Codex operate there without a Git repository.

**Architecture:** Resolve one application root at CLI startup from `PRACTICODE_HOME`, `HOME`, or `USERPROFILE`, then keep the existing explicit-root core APIs. Flatten the old `.practicode/*` metadata paths into that root, migrate a detected legacy launch-directory workspace without overwriting or deleting it, and scope AI and Docker writes to the resolved root.

**Tech Stack:** Rust 2024 standard library, existing `anyhow`, Node.js standard library, Cargo tests, npm packaging, GitHub Actions tag release.

## Global Constraints

- Add no dependencies.
- Default data root is `$HOME/.practicode` or `%USERPROFILE%\.practicode`.
- A non-empty `PRACTICODE_HOME` is the exact data root.
- Legacy migration never overwrites or deletes user data and never copies build cache.
- Codex keeps its existing sandbox mode and skips only the Git-repository check.
- Docker data survives container removal in the same host data root.
- Do not start Docker, build a local Docker image, or run a local container during implementation verification; use static launcher checks and npm packaging only.
- Release version is `0.1.20`.

---

### Task 1: Flatten Storage Paths Into the Application Root

**Files:**
- Modify: `src/core/model.rs:8-10`
- Modify: `src/core/judge.rs:77,205,236`
- Modify: `tests/core.rs`
- Modify: `tests/tui.rs`
- Modify: `tests/ai.rs`

**Interfaces:**
- Produces: `BANK_PATH = "problem_bank.json"`, `STATE_PATH = "problem-state.json"`, and `PROBLEM_NOTES_PATH = "problem_notes.md"` relative to an explicit application root.
- Preserves: existing `load_*`, `save_*`, judge, TUI, and AI note function signatures.

- [ ] **Step 1: Change test expectations to the new root-relative layout**

Replace test-only paths as follows:

```text
.practicode/problem_bank.json  -> problem_bank.json
.practicode/problem-state.json -> problem-state.json
.practicode/problem_notes.md   -> problem_notes.md
.practicode/build              -> build
```

Remove now-unneeded `create_dir_all(root.join(".practicode"))` setup lines. Keep migration fixtures for Task 2 explicitly under `.practicode`.

- [ ] **Step 2: Run focused tests to verify RED**

Run:

```bash
cargo test --test core save_bank_creates_local_custom_problem_bank
```

Expected: FAIL because production still writes `.practicode/problem_bank.json`.

- [ ] **Step 3: Implement the minimal path changes**

In `src/core/model.rs`:

```rust
pub const BANK_PATH: &str = "problem_bank.json";
pub const STATE_PATH: &str = "problem-state.json";
pub const PROBLEM_NOTES_PATH: &str = "problem_notes.md";
```

In `src/core/judge.rs`, replace each `root.join(".practicode/build")` with `root.join("build")`.

- [ ] **Step 4: Run storage and TUI tests to verify GREEN**

Run:

```bash
cargo test --test core
cargo test --test tui
cargo test --test ai
```

Expected: all tests pass with direct root-relative data paths.

- [ ] **Step 5: Commit**

```bash
git add src/core/model.rs src/core/judge.rs tests/core.rs tests/tui.rs tests/ai.rs
git commit -m "refactor: flatten practicode data layout"
```

---

### Task 2: Resolve the Global Root and Migrate Legacy Data

**Files:**
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: private `resolve_data_root(practicode_home: Option<OsString>, home: Option<OsString>, user_profile: Option<OsString>) -> Result<PathBuf>`.
- Produces: private `migrate_legacy_data(launch_dir: &Path, root: &Path) -> Result<()>`.
- Consumes: `core::{BANK_PATH, PROBLEM_NOTES_PATH, STATE_PATH}` from Task 1.
- Preserves: public `run_cli() -> Result<()>`.

- [ ] **Step 1: Add failing resolver and migration tests**

Add unit tests in `src/lib.rs` that assert:

```rust
assert_eq!(
    resolve_data_root(Some("/custom".into()), Some("/home/user".into()), None).unwrap(),
    PathBuf::from("/custom")
);
assert_eq!(
    resolve_data_root(None, Some("/home/user".into()), None).unwrap(),
    PathBuf::from("/home/user/.practicode")
);
assert_eq!(
    resolve_data_root(None, None, Some("C:\\Users\\user".into())).unwrap(),
    PathBuf::from("C:\\Users\\user").join(".practicode")
);
assert!(resolve_data_root(None, None, None).unwrap_err().to_string().contains("PRACTICODE_HOME"));
```

Use unique temporary roots to verify migration copies state, bank, notes, `problems/`, and `submissions/`; excludes `.practicode/build/`; keeps an existing destination note unchanged; skips a distinct destination that already contains state; resumes an interrupted copy; rejects nested roots and destination symlinks; propagates marker lookup errors; and does not copy ambiguous sibling folders when the legacy metadata directory already equals the new root. Remove fixtures at each test's end.

- [ ] **Step 2: Run unit tests to verify RED**

Run:

```bash
cargo test --lib resolve_data_root
```

Expected: compile failure because the resolver does not exist.

- [ ] **Step 3: Implement root resolution**

Add a private helper that ignores empty environment values, then resolve in this exact order:

```rust
fn resolve_data_root(
    practicode_home: Option<OsString>,
    home: Option<OsString>,
    user_profile: Option<OsString>,
) -> Result<PathBuf> {
    if let Some(path) = non_empty_path(practicode_home) {
        return Ok(path);
    }
    if let Some(path) = non_empty_path(home) {
        return Ok(path.join(".practicode"));
    }
    if let Some(path) = non_empty_path(user_profile) {
        return Ok(path.join(".practicode"));
    }
    anyhow::bail!("cannot find a user home directory; set PRACTICODE_HOME")
}
```

Capture `env::current_dir()` before resolving the root. Call migration before `--smoke` or TUI loading so every CLI path sees the same data.

- [ ] **Step 4: Implement non-destructive legacy migration**

Detect a legacy workspace only from `<launch>/.practicode/problem-state.json` or `<launch>/.practicode/problem_bank.json`. If the legacy metadata directory and destination are distinct and the destination already has either file, return without copying. Otherwise:

```rust
for name in [STATE_PATH, BANK_PATH, PROBLEM_NOTES_PATH] {
    copy_file_if_missing(&legacy_meta.join(name), &root.join(name))?;
}
for name in ["problems", "submissions"] {
    copy_tree_missing(&launch_dir.join(name), &root.join(name))?;
}
```

`copy_tree_missing` creates missing directories, copies regular files only with no-clobber creation, recurses into directories, and ignores source symlinks. Use an in-progress marker for retry, reject roots nested below either legacy tree, reject destination symlinks, and attach source/destination context to lookup and copy errors. Do not traverse `.practicode/build`.

- [ ] **Step 5: Run unit and full tests to verify GREEN**

Run:

```bash
cargo test --lib
cargo test
```

Expected: resolver/migration tests and all existing tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs
git commit -m "feat: use a global practicode data home"
```

---

### Task 3: Make AI Commands Data-Root Aware Without Git

**Files:**
- Modify: `src/ai.rs:220-255,403-420,475-507`
- Modify: `tests/ai.rs`

**Interfaces:**
- Consumes: application root passed to existing AI functions.
- Produces: Codex invocations containing `--skip-git-repo-check`.
- Produces: prompts using `problem_notes.md`, `problem_bank.json`, and `problem-state.json` at root.

- [ ] **Step 1: Write failing prompt and command assertions**

Update `tests/ai.rs` to require root-relative prompt paths and add:

```rust
assert!(command.contains("--skip-git-repo-check"));
assert!(!default_ai_next_prompt("arrays").contains(".practicode/problem"));
```

Change the background assertion to:

```rust
assert!(background.contains("Preserve problem-state.json current_problem"));
```

- [ ] **Step 2: Run focused tests to verify RED**

Run:

```bash
cargo test --test ai default_codex_command_uses_model_when_set
cargo test --test ai default_ai_next_prompt_reads_notes_and_includes_request
```

Expected: failures for the missing flag and old nested prompt paths.

- [ ] **Step 3: Implement the minimal Codex and prompt changes**

Add `--skip-git-repo-check` to the direct `Command::new("codex")` argument list used by coaching prompts and to both shell command strings:

```text
codex exec --ephemeral --skip-git-repo-check --cd ...
```

Replace nested metadata references in both generation prompt builders with the three root-level filenames. Keep `problems/INDEX.md` and problem directories unchanged.

- [ ] **Step 4: Run AI tests to verify GREEN**

Run:

```bash
cargo test --test ai
```

Expected: all AI tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ai.rs tests/ai.rs
git commit -m "fix: run Codex outside Git workspaces"
```

---

### Task 4: Persist the Global Root Through Docker

**Files:**
- Modify: `bin/practicode.js`

**Interfaces:**
- Consumes: host `PRACTICODE_HOME` or `os.homedir()`.
- Produces: host data directory bind-mounted at `/data` with container `PRACTICODE_HOME=/data`.

- [ ] **Step 1: Record the pre-change launcher check**

Run:

```bash
node --check bin/practicode.js
rg -n "PRACTICODE_HOME=/data|target=/data" bin/practicode.js
```

Expected: syntax check passes and `rg` returns no matches, demonstrating missing Docker persistence wiring.

- [ ] **Step 2: Add the minimum host-data mount**

Import `homedir` from `node:os` and `mkdirSync` from `node:fs`. In `runDockerSandbox`, resolve and create:

```js
const dataHome = path.resolve(
  process.env.PRACTICODE_HOME || path.join(homedir(), ".practicode"),
);
mkdirSync(dataHome, { recursive: true });
```

Add a bind mount `type=bind,source=${dataHome},target=/data` and environment entry `PRACTICODE_HOME=/data`. Keep `/workspace` mounted for legacy discovery.

- [ ] **Step 3: Verify launcher syntax and generated wiring**

Run:

```bash
node --check bin/practicode.js
rg -n "PRACTICODE_HOME=/data|target=/data" bin/practicode.js
npm pack --dry-run
```

Expected: syntax check and package dry run pass; both Docker strings are present.

- [ ] **Step 4: Commit**

```bash
git add bin/practicode.js
git commit -m "fix: persist Docker practice data globally"
```

---

### Task 5: Update User and Maintainer Documentation

**Files:**
- Modify: `README.md`
- Modify: `SECURITY.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/CONTRIBUTING.md`
- Modify: `docs/problem-authoring-notes.md`

**Interfaces:**
- Documents: `~/.practicode`, `PRACTICODE_HOME`, migration behavior, Docker persistence, and direct metadata filenames inside the root.

- [ ] **Step 1: Update every current-layout reference**

Document this table in the README:

```text
~/.practicode/problem_bank.json  local/custom/generated problems
~/.practicode/problem_notes.md   optional generation notes
~/.practicode/problem-state.json current problem, history, settings
~/.practicode/problems/          generated statements and index
~/.practicode/submissions/       learner code
```

Explain that `PRACTICODE_HOME` overrides the directory, legacy data is copied only when the global destination is empty, and originals remain. Update Docker copy to describe the `/data` mount. Change packaged authoring documentation to direct metadata filenames because AI runs with the application root as its working directory. Keep repository-only `AGENTS.md` rules and `.gitignore` legacy entries unchanged so old source-checkout workflows remain protected.

- [ ] **Step 2: Check for stale tracked documentation**

Run:

```bash
rg -n "current directory.*saved|\.practicode/problem_(bank|notes)|\.practicode/problem-state|root.join\(\"\.practicode/build\"" README.md SECURITY.md docs src --glob '!docs/superpowers/**'
```

Expected: no stale runtime-layout references; legacy migration text may still name old paths explicitly.

- [ ] **Step 3: Commit**

```bash
git add README.md SECURITY.md docs/ARCHITECTURE.md docs/CONTRIBUTING.md docs/problem-authoring-notes.md
git commit -m "docs: explain global practicode storage"
```

---

### Task 6: Verify the Packed npm Experience

**Files:**
- No production files.

**Interfaces:**
- Verifies: Rust CLI and npm-packed launcher honor an isolated `PRACTICODE_HOME` from a non-Git launch directory.

- [ ] **Step 1: Run all repository checks**

Run:

```bash
make test
```

Expected: rustfmt, Clippy with warnings denied, all Cargo tests, and npm package dry run pass.

- [ ] **Step 2: Smoke-test Cargo from a non-Git directory**

Build once, then run the binary from an empty temporary directory with a separate temporary data root:

```bash
cargo build
launch_dir=$(mktemp -d)
data_dir=$(mktemp -d)
(cd "$launch_dir" && PRACTICODE_HOME="$data_dir" /Users/bany9/workspace/personal/codecode/target/debug/practicode --smoke)
```

Expected stdout: the built-in Hello World problem title, with exit code 0 and no Git requirement.

- [ ] **Step 3: Smoke-test the packed npm launcher**

Run:

```bash
repo=$PWD
tarball=$(npm pack --silent)
prefix=$(mktemp -d)
npm_launch=$(mktemp -d)
npm_data=$(mktemp -d)
npm install --prefix "$prefix" --ignore-scripts "$repo/$tarball"
(cd "$npm_launch" && PRACTICODE_HOME="$npm_data" "$prefix/node_modules/.bin/practicode" --smoke)
rm -rf "$prefix" "$npm_launch" "$npm_data"
rm -f "$repo/$tarball"
```

Expected: `practicode --smoke` exits 0 and prints the built-in title. Remove only the temporary launch, data, prefix, and tarball paths afterward.

- [ ] **Step 4: Review the final diff and commits**

Run:

```bash
git diff origin/main...HEAD --check
git status --short
git log --oneline origin/main..HEAD
```

Expected: no whitespace errors, clean working tree, and only the design plus focused implementation/documentation commits.

---

### Task 7: Release and Verify 0.1.20

**Files:**
- Modify through release script: `Cargo.toml`
- Modify through release script: `Cargo.lock`
- Modify through release script: `package.json`

**Interfaces:**
- Produces: Git tag `v0.1.20`, crates.io package `practicode 0.1.20`, and npm package `practicode@0.1.20`.

- [ ] **Step 1: Push verified implementation commits to main**

Run:

```bash
git push origin main
```

Expected: `origin/main` advances to the verified documentation/implementation head.

- [ ] **Step 2: Run the repository release command**

Run:

```bash
make release VERSION=0.1.20
```

Expected: version files update, release tests pass, commit `Release v0.1.20` is created, tag `v0.1.20` is created, and main plus the tag are pushed.

- [ ] **Step 3: Verify GitHub release automation**

Run `gh run list --workflow release.yml --limit 5`, identify the run for `v0.1.20`, then use `gh run watch <run-id> --exit-status`.

Expected: the tagged release workflow completes successfully.

- [ ] **Step 4: Verify both registries**

Poll the authoritative registries until propagation completes:

```bash
npm view practicode version --silent
cargo search practicode --limit 1
```

Expected: npm reports `0.1.20`; crates.io reports `practicode = "0.1.20"`.

- [ ] **Step 5: Verify final repository state**

Run:

```bash
git status --short
git branch --show-current
git rev-parse HEAD
git rev-parse origin/main
git rev-parse v0.1.20
```

Expected: clean `main`; local HEAD and `origin/main` match the release commit; tag `v0.1.20` resolves to that commit.
