# practicode

![Rust](https://img.shields.io/badge/Rust-terminal%20app-000000?logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/Ratatui-TUI-00B4D8)
![Local first](https://img.shields.io/badge/local--first-practice-14B8A6)
![AI ready](https://img.shields.io/badge/AI-Codex%20%2B%20Claude-111827)
![CI](https://github.com/baba9811/practicode/actions/workflows/ci.yml/badge.svg)

![practicode terminal UI](assets/practicode-terminal.svg)

Personal coding practice, right in your terminal.

`practicode` is a small Rust TUI for stdin/stdout practice: problem on the left, code on the right, judge loop in the same terminal.
No browser tab shuffle, no paste dance, just solve and run.

## Why It Exists

- Fast local judging for Python, TypeScript, Java, and Rust
- Gradual problem flow with local history
- AI-powered `/next <request>` when you want a custom problem
- Personal problem-generation notes
- Small stack: Rust, Ratatui, Crossterm, and plain process execution

## Quick Start

```bash
git clone https://github.com/baba9811/practicode.git
cd practicode
cargo run --
```

Want a local binary?

```bash
cargo install --path .
practicode
```

Prefer npm?

```bash
npm install -g .
practicode
```

The npm wrapper builds the Rust binary with Cargo, so Rust/Cargo is still required.

## Daily Loop

The code editor starts focused.

```text
write code
Esc, then /run
Esc, then /next easy string problem
```

Submissions are saved as you type under `submissions/<problem-id>/solution.<ext>`.

## Commands

Press `Esc`, then `/`, to focus the command bar.

| Command | Action |
| --- | --- |
| `/run` | Judge the current submission |
| `/next` | Open the next local problem, or ask AI to create one |
| `/next easy string problem` | Ask AI for a custom next problem |
| `/prev` | Go back through problem history |
| `/list` | Browse problems with `up/down` or `j/k`, open with `Enter` |
| `/open 2` | Open by number, id, or slug |
| `/giveup` | Show the reference answer |
| `/ai hint` | Ask the selected AI about the current problem and submission |
| `/provider codex` | Set AI provider: `codex` or `claude` |
| `/model sonnet` | Set the model for `/ai` and AI-backed `/next`; use `auto` for the CLI default |
| `/note prefer hashmap practice` | Append a standing note for future problem generation |
| `/notes` | Show your local next-problem notes |
| `/lang python` | Set language: `python`, `ts`, `java`, `rust` |
| `/ui ko` | Set UI language: `ko`, `en` |
| `/theme` | Toggle dark/light theme |
| `/source ai` | Prefer AI for next-problem generation |
| `/exit` | Quit |

The editor owns normal typing keys.
Press `Esc`, then `/`, when you want the command bar.

## Custom Problem Generation

`/next <request>` passes your request into the selected AI problem generator. Examples:

```text
/next a slightly harder string problem
/next hashmap practice, easy
/next sorting problem, no graph yet
```

AI generation reads [docs/problem-authoring-notes.md](docs/problem-authoring-notes.md) every time it creates a problem. Add personal preferences from inside the TUI:

```text
/note Prefer concise statements.
/note I want more string and hashmap practice.
/note Avoid DP until I ask for it.
```

Those notes are stored in `.practicode/problem_notes.md`, so they stay local.

## AI Providers

Codex is the default:

```text
/provider codex
/model auto
```

Claude Code is also supported:

```text
/provider claude
/model sonnet
/source ai
```

`/ai <prompt>` uses the current provider for coaching. AI-backed `/next` uses the same provider and model.
If you want a custom daemon or wrapper script, set `/ai-next-command <shell command>`; practicode passes `PRACTICODE_NEXT_REQUEST`, `PRACTICODE_AI_PROVIDER`, and `PRACTICODE_AI_MODEL`.

Generated problem banks stay local:

| Path | Purpose |
| --- | --- |
| `.practicode/problem_bank.json` | Local/custom/generated problem bank |
| `.practicode/problem_notes.md` | Optional personal problem-generation notes |
| `.practicode/problem-state.json` | Current problem, history, settings |
| `problems/` | Generated problem markdown/index files |
| `submissions/` | Your answer files |

Those paths are ignored by git, so your practice history stays yours.

## Safety

`/run` executes your local submission as a normal process. practicode runs it from `.practicode/build/<problem-id>/run`, but this is not an OS sandbox. Only run code you trust.

## Debug Prints

`/run` shows raw stdout when a case fails. If you want debug output without changing the judged answer, print to stderr:

```python
import sys

print("debug", value, file=sys.stderr)
```

## Development

```bash
cargo test
cargo run -- --smoke
cargo run --
```

The source is split by boring responsibility:

| Path | Role |
| --- | --- |
| `src/core.rs` | Problem bank, state, rendering, judging |
| `src/tui.rs` | Ratatui app, editor, command parser |
| `src/ai.rs` | Codex/Claude command integration and notes |
| `src/text.rs` | UTF-8 cursor math and Hangul composition |
| `src/process.rs` | Process execution helpers |
| `tests/` | Integration tests split by module |

## Discovery Notes

Recommended GitHub topics for this repo:
`coding-practice`, `competitive-programming`, `algorithms`, `ratatui`, `tui`, `rust`, `codex`, `claude-code`, `local-first`.

## References

- Ratatui terminal UI library: https://ratatui.rs/
- Crossterm terminal backend/events: https://github.com/crossterm-rs/crossterm
- Codex CLI open-source repo: https://github.com/openai/codex
- Claude Code CLI reference: https://docs.anthropic.com/en/docs/claude-code/cli-reference
- Kattis problem package format: https://www.kattis.com/problem-package-format/
- ICPC judging guidelines: https://icpc.global/regionals/regional-contest-cookbook-judging-guidelines
- GitHub README image guidance: https://docs.github.com/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes
- GitHub repository topics: https://docs.github.com/articles/classifying-your-repository-with-topics
