# codecode

Coding-test reps without leaving your terminal.

`codecode` is a local Textual app for stdin/stdout practice. The problem stays on the left, your code lives on the right, and `/run` judges the current submission against local cases.

No browser tab shuffle. No paste into a judge page. Just pick a problem, type, run, repeat.

## Quick Start

```bash
git clone https://github.com/baba9811/codecode.git
cd codecode
uv run codecode
```

The repo pins Python 3.13 with `.python-version`, so `uv` will use a compatible interpreter even if your global Python is older.

## Daily Loop

The code editor starts focused.

```text
type code in the right pane
Esc, then /run
Esc, then /next
Esc, then /codex hint
```

Submissions are saved as you type under `submissions/<problem-id>/solution.<ext>`.

## Debug Prints

`/run` shows raw stdout when a case fails, so quick `print(...)` checks are visible.

If you want debug output without changing the judged answer, print to stderr:

```python
import sys

print("debug", value, file=sys.stderr)
```

## Commands

| Command | Action |
| --- | --- |
| `/run` | Judge the current submission |
| `/edit` | Focus the inline code editor |
| `/next` | Open the next problem, or ask Codex to create one |
| `/prev` | Go back through problem history |
| `/list` | Browse problems with `up/down` or `j/k`, open with `Enter` |
| `/open 2` | Open by number, id, or slug |
| `/giveup` | Show the reference answer |
| `/codex hint` | Ask Codex about the current problem and submission |
| `/lang python` | Set language: `python`, `ts`, `java`, `rust` |
| `/ui ko` | Set UI language: `ko`, `en` |
| `/theme` | Toggle dark/light theme |
| `/source codex` | Prefer Codex for next-problem generation |
| `/exit` | Quit |

The editor owns normal typing keys. Press `Esc`, then `/`, when you want the command bar.

## Problem Sources

A fresh checkout starts with the built-in `001-hello-world` problem.

When `/next` runs out of local problems, `codecode` can ask Codex to generate one. Generated problem banks stay local by default:

| Path | Purpose |
| --- | --- |
| `.codecode/problem_bank.json` | Local/custom/generated problem bank |
| `.codex/problem-state.json` | Current problem, history, settings |
| `problems/` | Generated problem markdown/index files |
| `submissions/` | Your answer files |

Those paths are ignored by git, so the public repo stays clean while your practice history stays yours.

## Codex Hook

`/next` uses the local bank first. If there is no unseen problem left, or `/source codex` is enabled, the app runs the configured Codex next-problem command.

Default behavior:

```text
codex app-server daemon start
codex exec --sandbox workspace-write "create exactly one new non-duplicate problem"
```

`/codex <question>` is separate. It sends the current problem and current submission to Codex in read-only mode and prints the final response in the output pane.

Security note: `/next-command <cmd>` is a trusted local hook. Only set it to commands you would run directly in your shell.

## Development

```bash
uv sync
uv run pytest tests -q
uv run codecode --smoke
```

Small on purpose: Textual for the TUI, stdlib for judging/process work, and `uv` for setup.
