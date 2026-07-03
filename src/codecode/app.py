from __future__ import annotations

import argparse
from pathlib import Path
import subprocess

from textual import events
from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal
from textual.widgets import Input, Markdown, Static

from codecode.core import (
    LANGUAGES,
    UI_LANGUAGES,
    edit_command,
    ensure_edit_files,
    ensure_problem_files,
    ensure_submission,
    give_up,
    judge,
    load_bank,
    load_state,
    next_problem,
    problem_by_id,
    previous_problem,
    record_pass,
    render_problem,
    run_codex_next,
    run_codex_prompt,
    save_state,
)


HELP = """Commands
/help                 show this help
/vim                  Vim quick help
/run                  judge current submission
/edit                 open problem + solution in Vim
/next                 next problem
/prev                 previous problem
/list                 show problem list
/open 2               open a problem by number, id, or slug
/giveup               show answer
/lang python|ts|java|rust
/ui ko|en
/source bank|codex   choose next-problem source
/next-command <cmd>   set custom Codex next command
/codex <question>     ask Codex about current problem + code
"""


VIM_HELP = """Vim quick help
i insert mode
esc normal mode
:w save
:q quit
:wq save and quit
h/j/k/l move left/down/up/right
/text search
dd delete line
u undo
"""


class CodeCodeApp(App[None]):
    CSS = """
    Screen {
        layout: vertical;
        background: #0f1117;
        color: #edf2f7;
    }
    #body {
        height: 1fr;
        padding: 1 2;
    }
    #problem {
        width: 58%;
        padding: 1 2;
        border: round #3e4658;
        background: #151a22;
        color: #f8fafc;
        overflow-y: auto;
    }
    #output {
        width: 42%;
        margin-left: 1;
        padding: 1 2;
        border: round #3e4658;
        background: #121821;
        color: #dbe4f0;
        overflow-y: auto;
    }
    #status {
        height: 1;
        padding: 0 2;
        background: #202637;
        color: #7dd3fc;
        text-style: bold;
    }
    #command {
        height: 3;
        margin: 0 2 1 2;
        border: round #4b5568;
        background: #0d1016;
    }
    """
    BINDINGS = [
        Binding("e", "edit", "Edit"),
        Binding("r", "run", "Run"),
        Binding("n", "next", "Next"),
        Binding("p", "previous", "Prev"),
        Binding("g", "give_up", "Give up"),
        Binding("l", "cycle_language", "Language"),
        Binding("u", "toggle_ui_language", "UI"),
        Binding("slash", "focus_command", "Command"),
        Binding("q", "quit", "Quit"),
    ]

    def __init__(self, root: Path | None = None) -> None:
        super().__init__()
        self.root = root or Path.cwd()
        self.bank = load_bank()
        self.state = load_state(self.root, self.bank)
        self.problem = problem_by_id(self.bank, self.state.current_problem)

    def compose(self) -> ComposeResult:
        with Horizontal(id="body"):
            yield Markdown(id="problem")
            output = Markdown(id="output")
            output.can_focus = True
            yield output
        yield Static(id="status")
        yield Input(placeholder="/help, /run, /edit, /next, /prev, /list, /open 2, /codex hint", id="command")

    def on_mount(self) -> None:
        self.refresh_view("Ready\n\nPress /help for commands.")
        self.call_after_refresh(self.set_focus, None)

    def refresh_view(self, output: str | None = None) -> None:
        self.query_one("#status", Static).update(
            f" CODECODE | {self.problem.id} | {self.problem.difficulty} | "
            f"lang:{self.state.settings.language} | ui:{self.state.settings.ui_language} | "
            f"next:{self.state.settings.next_source} | /help "
        )
        self.query_one("#problem", Markdown).update(render_problem(self.problem, self.state.settings.ui_language))
        if output is not None:
            self.write_output(output)

    def write_output(self, output: str, loading: bool = False) -> None:
        markdown = self.query_one("#output", Markdown)
        markdown.loading = False
        markdown.update(output)
        markdown.loading = loading

    def on_key(self, event: events.Key) -> None:
        command = self.query_one("#command", Input)
        if event.key == "escape" and self.focused is command:
            command.value = ""
            command.blur()
            event.stop()

    def action_focus_command(self) -> None:
        self.query_one("#command", Input).focus()

    def action_edit(self) -> None:
        statement, solution = ensure_edit_files(self.root, self.problem, self.state.settings)
        with self.suspend():
            subprocess.run(edit_command(self.state.settings.editor, statement, solution))
        self.refresh_view(f"Edited {solution}")

    def action_run(self) -> None:
        result = judge(self.root, self.problem, self.state.settings)
        if result.passed:
            record_pass(self.root, self.problem, self.state)
        self.refresh_view(result.output)

    def action_next(self) -> None:
        old_problem = self.state.current_problem
        if self.state.settings.next_source == "codex":
            self.start_next_problem(old_problem)
            return
        self.finish_next_problem("", old_problem)

    def start_next_problem(self, old_problem: str) -> None:
        self.write_output("Loading next problem...", loading=True)
        self.run_worker(lambda: self.ask_next_problem(old_problem), thread=True, exclusive=True, exit_on_error=False)

    def ask_next_problem(self, old_problem: str) -> None:
        try:
            output = run_codex_next(self.root, self.state)
        except Exception as error:
            output = f"Codex next failed\n{error}"
        self.call_from_thread(self.finish_next_problem, output, old_problem)

    def finish_next_problem(self, output: str, old_problem: str) -> None:
        if self.state.settings.next_source == "codex":
            self.bank = load_bank()
            self.state = load_state(self.root, self.bank)
        self.problem = problem_by_id(self.bank, self.state.current_problem)
        if self.state.settings.next_source != "codex" or self.state.current_problem == old_problem:
            self.problem = next_problem(self.root, self.bank, self.state)
        self.refresh_view(output or f"Loaded {self.problem.id}")

    def action_previous(self) -> None:
        old_problem = self.state.current_problem
        self.problem = previous_problem(self.root, self.bank, self.state)
        if self.state.current_problem == old_problem:
            self.refresh_view("Already at the first known problem.")
        else:
            self.refresh_view(f"Loaded {self.problem.id}")

    def action_give_up(self) -> None:
        answer = give_up(self.root, self.problem, self.state)
        self.refresh_view(f"Answer for {self.state.settings.language}:\n\n{answer}")

    def action_cycle_language(self) -> None:
        current = LANGUAGES.index(self.state.settings.language)
        self.state.settings.language = LANGUAGES[(current + 1) % len(LANGUAGES)]
        save_state(self.root, self.state)
        ensure_submission(self.root, self.problem, self.state.settings)
        self.refresh_view(f"Language: {self.state.settings.language}")

    def action_toggle_ui_language(self) -> None:
        current = UI_LANGUAGES.index(self.state.settings.ui_language)
        self.state.settings.ui_language = UI_LANGUAGES[(current + 1) % len(UI_LANGUAGES)]
        save_state(self.root, self.state)
        self.refresh_view(f"UI language: {self.state.settings.ui_language}")

    def action_toggle_next_source(self) -> None:
        self.state.settings.next_source = "codex" if self.state.settings.next_source == "bank" else "bank"
        save_state(self.root, self.state)
        self.refresh_view(f"Next source: {self.state.settings.next_source}")

    def on_input_submitted(self, event: Input.Submitted) -> None:
        value = event.value.strip()
        event.input.value = ""
        event.input.blur()
        if value.startswith("/"):
            value = value[1:].strip()
        self.handle_command(value)

    def handle_command(self, value: str) -> None:
        if not value or value in {"help", "h", "?"}:
            self.refresh_view(HELP)
            return
        if value.startswith("vim"):
            self.refresh_view(VIM_HELP)
            return
        parts = value.split(maxsplit=1)
        command, arg = parts[0], parts[1] if len(parts) > 1 else ""
        if command in {"run", "r"}:
            self.action_run()
        elif command in {"edit", "e"}:
            self.action_edit()
        elif command in {"next", "n"}:
            self.action_next()
        elif command in {"prev", "previous", "p"}:
            self.action_previous()
        elif command in {"giveup", "give", "g"}:
            self.action_give_up()
        elif command == "list":
            self.refresh_view(self.render_problem_list())
        elif command in {"open", "o"} and arg:
            self.open_problem(arg)
        elif command == "lang" and not arg:
            self.action_cycle_language()
        elif command == "lang" and arg in LANGUAGES:
            self.set_language(arg)
        elif command == "ui" and not arg:
            self.action_toggle_ui_language()
        elif command == "ui" and arg in UI_LANGUAGES:
            self.set_ui_language(arg)
        elif command in {"source", "next-source"} and arg in ("bank", "codex"):
            self.state.settings.next_source = arg
            save_state(self.root, self.state)
            self.refresh_view(f"Next source: {arg}")
        elif command == "next-command" and arg:
            self.state.settings.codex_next_command = arg
            self.state.settings.next_source = "codex"
            save_state(self.root, self.state)
            self.refresh_view("Codex next command saved.")
        elif command == "codex" and arg:
            self.start_codex_prompt(arg)
        else:
            self.refresh_view(f"Unknown command: {value}")

    def start_codex_prompt(self, prompt: str) -> None:
        self.write_output("Thinking...", loading=True)
        self.run_worker(lambda: self.ask_codex(prompt), thread=True, exclusive=True, exit_on_error=False)

    def ask_codex(self, prompt: str) -> None:
        try:
            output = run_codex_prompt(self.root, self.problem, self.state.settings, prompt)
        except Exception as error:
            output = f"Codex prompt failed\n{error}"
        self.call_from_thread(self.finish_codex_prompt, output)

    def finish_codex_prompt(self, output: str) -> None:
        self.write_output(output)

    def set_language(self, language: str) -> None:
        self.state.settings.language = language
        save_state(self.root, self.state)
        ensure_submission(self.root, self.problem, self.state.settings)
        self.refresh_view(f"Language: {language}")

    def set_ui_language(self, language: str) -> None:
        self.state.settings.ui_language = language
        save_state(self.root, self.state)
        self.refresh_view(f"UI language: {language}")

    def render_problem_list(self) -> str:
        status_by_id = {item.get("id"): item.get("status", "-") for item in self.state.history}
        lines = ["Problems", "", "  ID                 Difficulty  Status    Title"]
        for problem in self.bank:
            marker = ">" if problem.id == self.problem.id else " "
            title = problem.title[self.state.settings.ui_language]
            lines.append(
                f"{marker} {problem.id:<18} {problem.difficulty:<10} {status_by_id.get(problem.id, '-'):<8} {title}"
            )
        lines.append("\nOpen with /open 2, /open 002-running-sum, or /open running-sum.")
        return "\n".join(lines)

    def open_problem(self, query: str) -> None:
        problem = self.find_problem(query)
        if problem is None:
            self.refresh_view(f"Problem not found: {query}")
            return
        self.problem = problem
        self.state.current_problem = problem.id
        if not any(item.get("id") == problem.id for item in self.state.history):
            self.state.history.append({"id": problem.id, "status": "assigned"})
        save_state(self.root, self.state)
        ensure_problem_files(self.root, problem)
        self.refresh_view(f"Opened {problem.id}")

    def find_problem(self, query: str):
        needle = query.strip().lower()
        if needle.isdigit():
            needle = f"{int(needle):03d}"
        for problem in self.bank:
            if needle in {problem.id.lower(), problem.slug.lower()} or problem.id.startswith(needle):
                return problem
        return None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    if args.smoke:
        bank = load_bank()
        state = load_state(Path.cwd(), bank)
        print(problem_by_id(bank, state.current_problem).title[state.settings.ui_language])
        return
    CodeCodeApp().run()


if __name__ == "__main__":
    main()
