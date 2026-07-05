# practicode

![Rust](https://img.shields.io/badge/Rust-terminal%20app-000000?logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/Ratatui-TUI-00B4D8)
![Local first](https://img.shields.io/badge/local--first-practice-14B8A6)
![AI ready](https://img.shields.io/badge/AI-Codex%20%2B%20Claude-111827)
![crates.io](https://img.shields.io/crates/v/practicode?logo=rust)
![npm](https://img.shields.io/npm/v/practicode?logo=npm)
![CI](https://github.com/baba9811/practicode/actions/workflows/ci.yml/badge.svg)
[Socket.dev package health](https://socket.dev/npm/package/practicode)

![practicode terminal UI](assets/practicode-terminal.svg)

Personal coding practice, right in your terminal.

`practicode` is a small Rust TUI for stdin/stdout practice: problem on the left, code on the right, judge loop in the same terminal.

## What You Get

- Local stdin/stdout judging for Python, TypeScript, Java, and Rust.
- A two-pane terminal UI with problem text, editor, output, and command palette.
- Local-first problem history under ignored `.practicode/`, `problems/`, and `submissions/` paths.
- Optional Codex or Claude Code help for hints and generated next problems.

## Install

### Prerequisites

- Node.js 18+ for npm installation.
- Rust and Cargo. The npm package builds the Rust binary during install, and also on first run if needed.
- A runtime for the language you practice in: Python, Node.js for TypeScript, JDK for Java, or Rust.

### npm

```bash
npm install -g practicode
practicode
```

The npm package has a `postinstall` step that runs `cargo build --release --locked` from the package root so the Rust TUI binary is ready. Set `PRACTICODE_SKIP_BUILD=1` to skip that install-time build; the `practicode` launcher will try the same locked Cargo build on first run if the binary is missing.

### Cargo

```bash
cargo install practicode
practicode
```

### Local checkout

```bash
git clone https://github.com/baba9811/practicode.git
cd practicode
npm install
npm start
```

### Check Install

```bash
practicode --version
practicode --smoke
practicode --help
```

## Daily Loop

On first run, practicode opens a setup panel. After that, the code editor starts focused.

```text
first run: review /profile setup
write code
Esc, then /
choose /run
choose /next when it passes
```

Typing `/` outside the editor opens the command palette. Use `up/down` to move, `Enter` to run or complete the selected command, and `Esc` to cancel. Press `?` for in-app help or `Ctrl+C` to quit.

Hints, failed cases, answers, and user profile panels are copy-friendly: drag in the output pane to use your terminal's normal text selection/copy behavior. When the code editor is visible in the right pane, mouse focus is enabled for that editor. Use `Esc`, `e`, or `/code` to return from output to code.

Submissions are saved as you type under `submissions/<problem-id>/solution.<ext>`.

## CLI Flags

| Flag | Action |
| --- | --- |
| `--help`, `-h` | Show non-interactive help |
| `--version`, `-V` | Print the installed version |
| `--smoke` | Print the current problem title and exit |

## Commands

| Command | Action |
| --- | --- |
| `/run` | Judge the current submission |
| `/code` | Return to the code editor |
| `/next` | Open the next unsolved problem, or ask AI only when none remain |
| `/generate easy string problem` | Ask AI to create a new problem in the background |
| `/back` | Go back through problem history |
| `/problems` | Browse problems with `up/down` or `j/k`, open with `Enter` |
| `/open 2` | Open by number, id, or slug |
| `/answer` | Show the reference answer |
| `/hint` | Ask the selected AI for a concise hint |
| `/hint explain my bug` | Ask the selected AI about the current problem and submission |
| `/profile` | Show your current user profile |
| `/difficulty auto` | Set difficulty preference: `auto`, `easy`, `medium`, `hard` |
| `/topics arrays, strings` | Set preferred topics for future problems |
| `/avoid dp, graph` | Set topics to avoid in future problems |
| `/generate-languages python, rust` | Limit generated answer languages, or use `all` |
| `/generate-ui ko, en` | Limit generated problem text languages, or use `all` |
| `/provider codex` | Set AI provider and show local CLI/daemon status |
| `/model auto` | Use the provider default model for `/hint` and AI-backed `/next` |
| `/effort auto` | Use the provider default effort, or set `low`, `medium`, `high`, `xhigh`; Claude also supports `max` |
| `/note` | Edit problem-generation notes used by AI-backed `/next` and `/generate` |
| `/notes` | Show saved problem-generation notes |
| `/language python` | Set code language: `python`, `ts`, `java`, `rust` |
| `/ui en` | Set UI language: `en`, `ko`, `ja`, `zh`, `es` |
| `/theme dark` | Set theme: `dark` or `light` |
| `/update` | Show update instructions when a newer version is available |
| `/exit` | Quit |

Older command names such as `/prev`, `/list`, `/giveup`, and `/lang` still work as aliases.

The default UI language is English. Switch it any time with `/ui ko`, `/ui ja`, `/ui zh`, or `/ui es`.

Your user profile is saved in `.practicode/problem-state.json`. It keeps UI language, code language, theme, preferred difficulty, preferred topics, topics to avoid, and generation language scope. `auto` difficulty follows gradual progression; a fixed difficulty asks local selection and AI generation to prefer that level.

Inside `/profile`, use `up/down` to move and `Space` or `Enter` to cycle common settings, AI provider/model/effort, or generated answer/UI languages. The notes row opens an editor for `.practicode/problem_notes.md`. Use slash commands for free-form lists such as `/topics arrays, strings`.

## Problem Flow

`/next` is local-first: it opens the next unsolved local problem before generating anything. When no unsolved problem remains, it asks the selected AI provider to create one.

Use `/generate <request>` when you explicitly want to create a new problem in the background while you keep solving the current one. The generated problem stays local and `/next` will pick it up later.

If `/next` has to generate because no local problem remains, it runs in the foreground. Editing and commands are paused so state cannot change halfway through that AI task. Press `Space` for the warmup timing drill, or `q`/`Ctrl+C` to quit.

```text
/generate a slightly harder string problem
/generate hashmap practice, easy
/generate sorting problem, no graph yet
```

Codex is the default provider:

```text
/provider codex
/model auto
/effort auto
```

Claude Code is also supported:

```text
/provider claude
/model sonnet
/effort high
```

Generated problems and submissions stay local:

| Path | Purpose |
| --- | --- |
| `.practicode/problem_bank.json` | Local/custom/generated problems |
| `.practicode/problem_notes.md` | Optional personal problem-generation notes |
| `.practicode/problem-state.json` | Current problem, history, settings |
| `problems/` | Generated problem markdown/index files |
| `submissions/` | Your answer files |

Those paths are ignored by git, so your practice history stays yours.

## Update

The app checks for newer npm releases in the background and shows `/update` in the status line when one is available. Disable that check with `PRACTICODE_NO_UPDATE_CHECK=1`.

```bash
npm update -g practicode
cargo install --force practicode
```

## Safety And Security

- `/run` executes your local submission as a normal process. practicode runs it from `.practicode/build/<problem-id>/run`, but this is not an OS sandbox. Only run code you trust.
- `/hint` sends the current problem and submission to the selected AI provider CLI.
- AI-backed `/next` and `/generate` can run a custom shell command from `settings.ai_next_command`; save only commands you trust.
- npm installs run the package `postinstall` script described above. It only invokes Cargo with the checked-in lockfile from this package root; it does not read local `.env`/`.npmrc` files or contact the configured AI provider.
- npm releases are published from GitHub Actions with registry signatures and provenance enabled in `package.json`. The release workflow is also prepared for npm Trusted Publishing/OIDC; maintainers should prefer that over long-lived publish tokens when the package setting is enabled on npm.
- Local `.env`, `.npmrc`, `.practicode/`, `problems/`, and `submissions/` are ignored by git. Do not commit tokens, private prompts, or answer keys.

## Development Checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --smoke
cargo audit --deny warnings
npm pack --dry-run
npm run smoke
```

CI runs the Rust audit gate with `cargo audit --deny warnings`; release publishing stops before crates.io/npm publish if that audit fails. This repo has no npm dependencies or lockfile today, so `npm audit` and `pnpm audit` are not applicable until a matching lockfile is added.

## Contributing

External contributions use the fork and pull request flow in [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md).

Maintainer-only review and release notes live in [docs/MAINTAINING.md](docs/MAINTAINING.md).

Code layout and extension boundaries live in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

practicode is MIT licensed. Third-party dependency license notes are in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
