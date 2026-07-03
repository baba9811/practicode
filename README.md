# codecode

Local coding-test practice TUI.

## Run

```bash
uv run codecode
```

Keys:

- `e`: open the problem and current submission in a `vim` split
- `r`: run hidden-style stdin/stdout cases
- `n`: load the next problem
- `p`: load the previous problem from history
- `g`: give up and show the answer
- `l`: cycle language: Python, TS, Java, Rust
- `u`: toggle Korean/English UI text
- `/`: command input, `Esc` exits it

Commands:

```text
/help
/vim
/run
/edit
/next
/prev
/list
/open 2
/giveup
/lang python
/lang ts
/lang java
/lang rust
/ui ko
/ui en
/source bank
/source codex
/next-command <custom next-problem command>
/codex <question about the current problem/code>
```

Submissions are written under `submissions/<problem-id>/`.

## Codex Next

`/source codex` makes `/next` or `n` call a Codex command first. The default command starts the local app-server daemon and then runs `codex exec` to create one new problem from repo instructions.

Override it with:

```text
/next-command codex app-server daemon start; codex exec --cd . --sandbox workspace-write "create the next problem"
```

If the Codex command fails or leaves the current problem unchanged, the app falls back to the local problem bank.

`/codex <question>` is separate from next-problem generation. It sends the current problem and current submission to Codex in read-only mode and prints the response in the output pane.

## Repo Tests

```bash
uv run pytest tests -q
```

Problem-specific legacy tests still run when explicitly targeted:

```bash
uv run pytest problems/001-running-sum -q
```
