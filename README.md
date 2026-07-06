# practicode

![Rust](https://img.shields.io/badge/Rust-terminal%20app-000000?logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/Ratatui-TUI-00B4D8)
![Local first](https://img.shields.io/badge/local--first-practice-14B8A6)
![AI ready](https://img.shields.io/badge/AI-Codex%20%2B%20Claude-111827)
![crates.io](https://img.shields.io/crates/v/practicode?logo=rust)
![npm](https://img.shields.io/npm/v/practicode?logo=npm)
![CI](https://github.com/baba9811/practicode/actions/workflows/ci.yml/badge.svg)
[Socket.dev package health](https://socket.dev/npm/package/practicode)

Personal coding practice, right in your terminal.

`practicode` is a small Rust TUI for syntax drills and stdin/stdout coding-test practice. It keeps your problems, settings, and submissions local by default.

<figure>
  <img src="assets/practicode-home.svg" alt="Practicode home screen with Learn syntax and Practice coding tests choices">
  <figcaption>First run opens a small home screen for choosing learning or practice.</figcaption>
</figure>

<figure>
  <img src="assets/practicode-terminal.svg" alt="Practicode terminal UI with problem text, code editor, status line, and command palette">
  <figcaption>Practice mode keeps the problem, editor, judge output, and command palette in one terminal.</figcaption>
</figure>

## What You Get

- Syntax learning and coding-test practice from one entry screen.
- Local judging for Python, TypeScript, Java, and Rust.
- A two-pane Ratatui UI with editor, output, status line, and command palette.
- Optional Codex or Claude Code help for hints and generated problems.

## Install

### npm

```bash
npm install -g practicode
practicode
```

The npm package builds the Rust binary during install. Set `PRACTICODE_SKIP_BUILD=1` to skip that step; the launcher will try the same locked Cargo build on first run if the binary is missing.

<details>
<summary>Cargo</summary>

```bash
cargo install practicode
practicode
```

</details>

<details>
<summary>Local checkout</summary>

```bash
git clone https://github.com/baba9811/practicode.git
cd practicode
npm install
npm start
```

</details>

### Language runtimes

Install only the languages you plan to practice. Python uses `python3` or `python`; TypeScript uses `node --experimental-strip-types`; Java uses `javac` and `java`; Rust uses `rustc`.

<details>
<summary>macOS</summary>

```bash
brew install python node
brew install --cask temurin@21
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary>Windows</summary>

```powershell
winget install -e --id Python.Python.3.12
winget install -e --id OpenJS.NodeJS.LTS
winget install -e --id EclipseAdoptium.Temurin.21.JDK
winget install -e --id Rustlang.Rustup
```

Restart the terminal after installing so `python`, `node`, `javac`, and `rustc` are on `PATH`.

</details>

<details>
<summary>Ubuntu / Debian</summary>

```bash
sudo apt update
sudo apt install -y python3 nodejs npm openjdk-21-jdk curl build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If `node --version` is below `v22.6.0`, install a newer Node.js from the official Node.js downloads or your preferred version manager before using TypeScript practice.

</details>

Verify runtimes:

```bash
python3 --version
node --version
javac -version
rustc --version
```

References: [Python](https://docs.python.org/3/using/), [Node.js](https://nodejs.org/en/download), [Rust](https://www.rust-lang.org/tools/install), [Eclipse Temurin](https://adoptium.net/installation/).

After starting practicode, run `/doctor` to check these runtimes from inside the TUI.

Check the install:

```bash
practicode --version
practicode --smoke
practicode --help
```

## Update

npm is the primary install path:

```bash
npm update -g practicode
```

The app checks npm for newer releases in the background and shows `/update` in the status line when one is available. Disable that check with `PRACTICODE_NO_UPDATE_CHECK=1`.

<details>
<summary>Cargo update</summary>

```bash
cargo install --force practicode
```

</details>

<details>
<summary>Local checkout update</summary>

```bash
git pull --ff-only
npm install
npm start
```

</details>

## First Run

On first run, choose a mode:

```text
Learn syntax: read a short lesson, edit the drill, /run, then /next
Practice coding tests: write code, Esc, /run, then /next when it passes
```

Use arrow keys to move on the home screen, and `Enter` or `Space` to open the selected mode.

## Commands

Type `/` outside the editor to open the command palette. Use `up/down` to move, `Enter` to run or complete the selected command, and `Esc` to cancel.

Most-used commands:

| Command | Action |
| --- | --- |
| `/home` | Return to the mode chooser |
| `/run` | Judge the current submission or drill |
| `/next` | Open the next problem or lesson |
| `/back` | Go to the previous problem or lesson |
| `/doctor` | Check local runtimes and show install hints |
| `/profile` | Edit language, theme, difficulty, topics, and AI settings |

See [docs/COMMANDS.md](docs/COMMANDS.md) for the full command list, aliases, AI generation commands, and profile settings.

## Local Data

Generated problems and submissions stay local:

| Path | Purpose |
| --- | --- |
| `.practicode/problem_bank.json` | Local/custom/generated problems |
| `.practicode/problem_notes.md` | Optional personal problem-generation notes |
| `.practicode/problem-state.json` | Current problem, history, and settings |
| `problems/` | Generated problem markdown/index files |
| `submissions/` | Your answer files |

Those paths are ignored by git.

## Safety

- `/run` executes your local submission as a normal process. It is not an OS sandbox.
- `/hint`, AI-backed `/next`, and `/generate` send the current problem/submission context to the selected provider CLI.
- `settings.ai_next_command` can run a custom shell command. Save only commands you trust.
- Do not commit tokens, private prompts, `.env`, `.npmrc`, `.practicode/`, `problems/`, or `submissions/`.

Security reporting details live in [SECURITY.md](SECURITY.md).

## Contributing

- External contribution flow: [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
- Maintainer and release notes: [docs/MAINTAINING.md](docs/MAINTAINING.md)
- Code layout and extension rules: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

Local checks:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --smoke
```

## License

practicode is MIT licensed. Third-party dependency license notes are in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
