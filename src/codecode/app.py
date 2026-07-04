from __future__ import annotations

import argparse
from pathlib import Path

from textual import events
from textual.app import App, ComposeResult
from textual.timer import Timer
from textual.containers import Container, Horizontal
from textual.widgets import Markdown, Static, TextArea

from codecode.core import (
    EXT,
    LANGUAGES,
    THEMES,
    UI_LANGUAGES,
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
    normalize_language,
    run_codex_next,
    run_codex_prompt,
    save_state,
    template_for,
)


HELP = """# Help

## Daily loop

1. Type code in the right pane.
2. Press `Esc`, then `/run`.
3. Use `/next` when it passes.

## Commands

- `/run` judge current submission
- `/edit` focus the code editor
- `/next` next problem
- `/prev` previous problem
- `/list` choose from problem list
- `/open 2` open by number, id, or slug
- `/giveup` show answer
- `/codex hint` ask Codex about current problem + code
- `/lang python|ts|java|rust`
- `/ui ko|en`
- `/theme dark|light`
- `/source bank|codex`
- `/exit` quit

## Keys

- `Esc` leaves the editor or output pane
- `/` opens the command bar when the editor is not focused
- `?` opens this help when the editor is not focused
- `up/down` or `j/k` move in `/list`

## Debug prints

- stdout prints are shown when a case fails
- stderr prints are shown without affecting the expected stdout
"""


CODE_LANGUAGES = {"python": "python", "ts": "javascript", "java": "java", "rust": "rust"}


class CommandBar(TextArea):
    def _on_key(self, event: events.Key) -> None:
        if event.key == "enter":
            event.prevent_default()
            event.stop()
            getattr(self.app, "submit_command")()
        elif event.key == "escape":
            event.prevent_default()
            event.stop()
            self.load_text("")
            self.blur()
        elif event.key in {"?", "question_mark"} and self.text.strip() in {"", "/"}:
            event.prevent_default()
            event.stop()
            self.load_text("")
            self.blur()
            getattr(self.app, "handle_command")("help")


class CodeCodeApp(App[None]):
    CSS = """
    Screen {
        layout: vertical;
        background: #090d12;
        color: #e5eef8;
    }
    #body {
        height: 1fr;
        padding: 1 1 1 1;
    }
    #problem {
        width: 58%;
        height: 100%;
        padding: 1 2;
        border: tall #2f6f82;
        background: #0f1720;
        color: #f8fafc;
        overflow-y: auto;
        scrollbar-color: #4fd1c5;
        scrollbar-background: #17202b;
    }
    #side {
        width: 42%;
        height: 100%;
        margin-left: 1;
    }
    #code, #output {
        width: 100%;
        height: 100%;
        padding: 1 2;
        border: tall #31536b;
        background: #0d141c;
        color: #dbe4f0;
        overflow-y: auto;
        scrollbar-color: #f6c177;
        scrollbar-background: #17202b;
    }
    #code {
        padding: 0 1;
    }
    .hidden {
        display: none;
    }
    #status {
        height: 1;
        margin: 0 1;
        padding: 0 1;
        background: #152033;
        color: #c8d3f5;
        text-style: bold;
    }
    #command {
        height: 3;
        margin: 1 1 1 1;
        border: tall #3b82f6;
        background: #0b1017;
        color: #f8fafc;
    }
    Screen.theme-light {
        background: #f4f7fb;
        color: #111827;
    }
    Screen.theme-light #problem {
        border: tall #0f766e;
        background: #ffffff;
        color: #111827;
        scrollbar-color: #0f766e;
        scrollbar-background: #dbe4ef;
    }
    Screen.theme-light #code, Screen.theme-light #output {
        border: tall #2563eb;
        background: #f8fafc;
        color: #1f2937;
        scrollbar-color: #2563eb;
        scrollbar-background: #dbe4ef;
    }
    Screen.theme-light #status {
        background: #dbeafe;
        color: #1e3a8a;
    }
    Screen.theme-light #command {
        border: tall #2563eb;
        background: #ffffff;
        color: #111827;
    }
    """
    BUSY_FRAMES = ("", ".", "..", "...")
    BINDINGS = []

    def __init__(self, root: Path | None = None) -> None:
        super().__init__()
        self.root = root or Path.cwd()
        self.bank = load_bank(self.root)
        self.state = load_state(self.root, self.bank)
        self.problem = problem_by_id(self.bank, self.state.current_problem)
        self.busy_label = ""
        self.busy_body = ""
        self.busy_frame = 0
        self.busy_timer: Timer | None = None
        self.list_cursor: int | None = None
        self.loading_code = False

    def compose(self) -> ComposeResult:
        with Horizontal(id="body"):
            yield Markdown(id="problem")
            with Container(id="side"):
                yield TextArea(
                    id="code",
                    language=CODE_LANGUAGES[normalize_language(self.state.settings.language)],
                    tab_behavior="indent",
                    show_line_numbers=True,
                    soft_wrap=False,
                )
                output = Markdown(id="output", classes="hidden")
                output.can_focus = True
                yield output
        yield Static(id="status")
        yield CommandBar(placeholder="/run, /next, /list, /codex hint, /help", id="command")

    def on_mount(self) -> None:
        self.apply_theme()
        self.refresh_view()
        self.load_code_editor(focus=True)

    def on_unmount(self) -> None:
        if self.busy_timer is not None:
            self.busy_timer.stop()

    def refresh_view(self, output: str | None = None) -> None:
        self.query_one("#status", Static).update(self.status_text())
        self.query_one("#problem", Markdown).update(render_problem(self.problem, self.state.settings.ui_language))
        if output is not None:
            self.write_output(output)

    def status_text(self) -> str:
        return (
            f" CODECODE | {self.problem.id} | {self.problem.difficulty} | {self.busy_status()} | "
            f"{self.problem_status()} | code:{self.submission_status()[0]} | "
            f"{self.state.settings.language} | next:{self.state.settings.next_source} | {self.mode_hint()} "
        )

    def write_output(self, output: str) -> None:
        self.show_output()
        markdown = self.query_one("#output", Markdown)
        markdown.loading = False
        markdown.update(output)

    def write_text_output(self, output: str) -> None:
        self.write_output(f"```text\n{output.rstrip()}\n```")

    def mode_hint(self) -> str:
        command = self.query_one("#command", TextArea)
        code = self.query_one("#code", TextArea)
        output = self.query_one("#output", Markdown)
        if self.focused is command:
            return "Enter submit | Esc cancel"
        if self.list_cursor is not None:
            return "up/down move | Enter open | Esc close"
        if not output.has_class("hidden"):
            return "Esc code | / command | ? help"
        if self.focused is code:
            return "Esc then / command"
        return "/ command | ? help"

    def load_code_editor(self, focus: bool = False) -> None:
        path = ensure_submission(self.root, self.problem, self.state.settings)
        editor = self.query_one("#code", TextArea)
        self.loading_code = True
        editor.language = CODE_LANGUAGES[normalize_language(self.state.settings.language)]
        editor.load_text(path.read_text())
        self.loading_code = False
        self.query_one("#status", Static).update(self.status_text())
        self.show_code(focus=focus)

    def show_code(self, focus: bool = False) -> None:
        code = self.query_one("#code", TextArea)
        output = self.query_one("#output", Markdown)
        code.set_class(False, "hidden")
        output.set_class(True, "hidden")
        if focus:
            code.focus()

    def show_output(self) -> None:
        code = self.query_one("#code", TextArea)
        output = self.query_one("#output", Markdown)
        code.set_class(True, "hidden")
        output.set_class(False, "hidden")
        output.focus()

    def save_code(self) -> None:
        path = ensure_submission(self.root, self.problem, self.state.settings)
        path.write_text(self.query_one("#code", TextArea).text)
        self.query_one("#status", Static).update(self.status_text())

    def on_text_area_changed(self, event: TextArea.Changed) -> None:
        if event.text_area.id == "code" and not self.loading_code:
            self.save_code()

    def on_focus(self, event: events.Focus) -> None:
        if getattr(event.control, "id", None) in {"code", "output", "command"}:
            self.query_one("#status", Static).update(self.status_text())

    def on_blur(self, event: events.Blur) -> None:
        if getattr(event.control, "id", None) in {"code", "output", "command"}:
            self.query_one("#status", Static).update(self.status_text())

    def busy_status(self) -> str:
        if not self.busy_label:
            return "idle"
        return f"busy:{self.busy_label}{self.BUSY_FRAMES[self.busy_frame]}"

    def start_busy(self, label: str, body: str) -> None:
        self.busy_label = label
        self.busy_body = body
        self.busy_frame = 0
        self.update_busy()
        if self.busy_timer is None:
            self.busy_timer = self.set_interval(0.2, self.update_busy, pause=True)
        self.busy_timer.resume()

    def stop_busy(self) -> None:
        if self.busy_timer is not None:
            self.busy_timer.pause()
        self.busy_label = ""
        self.busy_body = ""
        self.busy_frame = 0

    def update_busy(self) -> None:
        self.busy_frame = (self.busy_frame + 1) % len(self.BUSY_FRAMES)
        self.query_one("#status", Static).update(self.status_text())
        self.write_text_output(f"{self.busy_body}{self.BUSY_FRAMES[self.busy_frame]}")

    def on_key(self, event: events.Key) -> None:
        command = self.query_one("#command", TextArea)
        code = self.query_one("#code", TextArea)
        if event.key == "escape" and self.focused is command:
            command.load_text("")
            command.blur()
            event.stop()
            return
        if event.key == "escape" and self.focused is code:
            self.set_focus(None)
            event.stop()
            return
        if event.key == "escape" and not self.query_one("#output", Markdown).has_class("hidden"):
            self.show_code(focus=True)
            event.stop()
            return
        if self.focused not in {command, code}:
            if event.key in {"?", "question_mark"} or event.character == "?":
                event.prevent_default()
                self.handle_command("help")
                event.stop()
                return
            if event.key == "slash" or event.character == "/":
                event.prevent_default()
                self.action_focus_command()
                event.stop()
                return
            shortcuts = {
                "r": self.action_run,
                "n": self.action_next,
                "p": self.action_previous,
                "g": self.action_give_up,
                "e": self.action_edit,
                "l": self.action_cycle_language,
                "u": self.action_toggle_ui_language,
                "q": self.exit,
            }
            if event.key in shortcuts:
                shortcuts[event.key]()
                event.stop()
                return
        if self.list_cursor is not None and self.focused is not command:
            if event.key in {"up", "k"}:
                self.move_list_cursor(-1)
                event.stop()
            elif event.key in {"down", "j"}:
                self.move_list_cursor(1)
                event.stop()
            elif event.key == "enter":
                self.open_selected_problem()
                event.stop()
            elif event.key == "escape":
                self.list_cursor = None
                self.refresh_view("Closed list.")
                event.stop()

    def action_focus_command(self) -> None:
        self.query_one("#command", TextArea).focus()

    def action_edit(self) -> None:
        self.load_code_editor(focus=True)

    def action_run(self) -> None:
        self.save_code()
        result = judge(self.root, self.problem, self.state.settings)
        if result.passed:
            record_pass(self.root, self.problem, self.state)
        headline = ("PASS" if result.passed else "FAIL") + f" {result.passed_cases}/{result.total_cases}"
        next_step = "Next: /next" if result.passed else "Fix code, then /run"
        self.refresh_view()
        self.write_text_output(f"{headline}\n{result.output}\n\n{next_step}")

    def action_next(self) -> None:
        old_problem = self.state.current_problem
        if self.state.settings.next_source == "codex":
            self.start_next_problem(old_problem, force=False)
            return
        problem = next_problem(self.root, self.bank, self.state)
        if problem is None:
            self.start_next_problem(old_problem, force=True)
            return
        self.problem = problem
        self.refresh_view()
        self.load_code_editor(focus=True)

    def start_next_problem(self, old_problem: str, force: bool) -> None:
        self.start_busy("next", "Generating next problem")
        self.run_worker(lambda: self.ask_next_problem(old_problem, force), thread=True, exclusive=True, exit_on_error=False)

    def ask_next_problem(self, old_problem: str, force: bool) -> None:
        try:
            output = run_codex_next(self.root, self.state, force=force)
        except Exception as error:
            output = f"Codex next failed\n{error}"
        self.call_from_thread(self.finish_next_problem, output, old_problem, force)

    def finish_next_problem(self, output: str, old_problem: str, force: bool) -> None:
        self.stop_busy()
        if self.state.settings.next_source == "codex" or force:
            self.bank = load_bank(self.root)
            self.state = load_state(self.root, self.bank)
        self.problem = problem_by_id(self.bank, self.state.current_problem)
        if self.state.current_problem == old_problem:
            problem = next_problem(self.root, self.bank, self.state)
            if problem is None:
                self.refresh_view()
                self.write_text_output((output + "\n\n" if output else "") + "No next problem is available yet.")
                return
            self.problem = problem
        self.refresh_view()
        self.load_code_editor(focus=True)

    def action_previous(self) -> None:
        old_problem = self.state.current_problem
        self.problem = previous_problem(self.root, self.bank, self.state)
        if self.state.current_problem == old_problem:
            self.refresh_view("Already at the first known problem.")
        else:
            self.refresh_view()
            self.load_code_editor(focus=True)

    def action_give_up(self) -> None:
        answer = give_up(self.root, self.problem, self.state)
        language = normalize_language(self.state.settings.language)
        self.refresh_view()
        self.write_output(f"Answer for {language}:\n\n```{language}\n{answer.rstrip()}\n```")

    def action_cycle_language(self) -> None:
        current = LANGUAGES.index(self.state.settings.language)
        self.state.settings.language = LANGUAGES[(current + 1) % len(LANGUAGES)]
        save_state(self.root, self.state)
        self.refresh_view()
        self.load_code_editor(focus=True)

    def action_toggle_ui_language(self) -> None:
        current = UI_LANGUAGES.index(self.state.settings.ui_language)
        self.state.settings.ui_language = UI_LANGUAGES[(current + 1) % len(UI_LANGUAGES)]
        save_state(self.root, self.state)
        self.refresh_view(f"UI language: {self.state.settings.ui_language}")

    def action_toggle_theme(self) -> None:
        current = THEMES.index(self.state.settings.theme)
        self.set_theme(THEMES[(current + 1) % len(THEMES)])

    def action_toggle_next_source(self) -> None:
        self.state.settings.next_source = "codex" if self.state.settings.next_source == "bank" else "bank"
        save_state(self.root, self.state)
        self.refresh_view(f"Next source: {self.state.settings.next_source}")

    def submit_command(self) -> None:
        command = self.query_one("#command", TextArea)
        value = command.text.strip()
        command.load_text("")
        command.blur()
        if value.startswith("/"):
            value = value[1:].strip()
        self.handle_command(value)

    def handle_command(self, value: str) -> None:
        if not value or value in {"help", "h", "?"}:
            self.list_cursor = None
            self.refresh_view(HELP)
            return
        if value.startswith("vim"):
            self.list_cursor = None
            self.refresh_view()
            self.write_text_output("The code editor is already open on the right.")
            return
        parts = value.split(maxsplit=1)
        command, arg = parts[0], parts[1] if len(parts) > 1 else ""
        if command != "list":
            self.list_cursor = None
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
            self.start_problem_list()
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
        elif command == "theme" and not arg:
            self.action_toggle_theme()
        elif command == "theme" and arg in THEMES:
            self.set_theme(arg)
        elif command in {"source", "next-source"} and arg in ("bank", "codex"):
            self.state.settings.next_source = arg
            save_state(self.root, self.state)
            self.refresh_view(f"Next source: {arg}")
        elif command == "next-command" and arg:
            self.state.settings.codex_next_command = arg
            self.state.settings.next_source = "codex"
            save_state(self.root, self.state)
            self.refresh_view()
            self.write_text_output("Codex next command saved.")
        elif command == "codex" and arg:
            self.start_codex_prompt(arg)
        elif command in {"exit", "quit", "q"}:
            self.exit()
        else:
            self.refresh_view()
            self.write_text_output(f"Unknown command: {value}\nTry /help.")

    def start_codex_prompt(self, prompt: str) -> None:
        self.save_code()
        self.start_busy("codex", "Codex is thinking")
        self.run_worker(lambda: self.ask_codex(prompt), thread=True, exclusive=True, exit_on_error=False)

    def ask_codex(self, prompt: str) -> None:
        try:
            output = run_codex_prompt(self.root, self.problem, self.state.settings, prompt)
        except Exception as error:
            output = f"Codex prompt failed\n{error}"
        self.call_from_thread(self.finish_codex_prompt, output)

    def finish_codex_prompt(self, output: str) -> None:
        self.stop_busy()
        self.refresh_view(output)

    def set_language(self, language: str) -> None:
        self.state.settings.language = language
        save_state(self.root, self.state)
        self.refresh_view()
        self.load_code_editor(focus=True)

    def set_ui_language(self, language: str) -> None:
        self.state.settings.ui_language = language
        save_state(self.root, self.state)
        self.refresh_view(f"UI language: {language}")

    def set_theme(self, theme: str) -> None:
        self.state.settings.theme = theme
        self.apply_theme()
        save_state(self.root, self.state)
        self.refresh_view(f"Theme: {theme}")

    def apply_theme(self) -> None:
        self.screen.set_class(self.state.settings.theme == "light", "theme-light")

    def render_problem_list(self) -> str:
        status_by_id = {item.get("id"): item.get("status", "-") for item in self.state.history}
        cursor = self.list_cursor if self.list_cursor is not None else self.current_problem_index()
        lines = ["Problems", "", "    # ID                 Difficulty  Status      Code      Title"]
        for index, problem in enumerate(self.bank):
            marker = ">" if index == cursor else " "
            current = "*" if problem.id == self.problem.id else " "
            title = problem.title[self.state.settings.ui_language]
            code_status = self.submission_status(problem)[0]
            lines.append(
                f"{marker} {current} {index + 1:>2} {problem.id:<18} {problem.difficulty:<10} "
                f"{status_by_id.get(problem.id, '-'):<10} {code_status:<9} {title}"
            )
        lines.append("\nup/down or j/k select | enter open | esc close")
        return "\n".join(lines)

    def start_problem_list(self) -> None:
        self.list_cursor = self.current_problem_index()
        self.refresh_view()
        self.write_text_output(self.render_problem_list())

    def current_problem_index(self) -> int:
        for index, problem in enumerate(self.bank):
            if problem.id == self.problem.id:
                return index
        return 0

    def move_list_cursor(self, delta: int) -> None:
        if not self.bank:
            return
        cursor = self.list_cursor if self.list_cursor is not None else self.current_problem_index()
        self.list_cursor = (cursor + delta) % len(self.bank)
        self.query_one("#status", Static).update(self.status_text())
        self.write_text_output(self.render_problem_list())

    def open_selected_problem(self) -> None:
        if self.list_cursor is None:
            return
        problem = self.bank[self.list_cursor]
        self.list_cursor = None
        self.open_problem(problem.id)

    def open_problem(self, query: str) -> None:
        self.list_cursor = None
        problem = self.find_problem(query)
        if problem is None:
            self.refresh_view()
            self.write_text_output(f"Problem not found: {query}\nTry /list.")
            return
        self.problem = problem
        self.state.current_problem = problem.id
        if not any(item.get("id") == problem.id for item in self.state.history):
            self.state.history.append({"id": problem.id, "status": "assigned"})
        save_state(self.root, self.state)
        ensure_problem_files(self.root, problem)
        self.refresh_view()
        self.load_code_editor(focus=True)

    def problem_summary(self, problem=None) -> str:
        problem = problem or self.problem
        code_status, code_note = self.submission_status(problem)
        return "\n".join(
            [
                f"Status: {self.problem_status(problem)}",
                f"Submission: {code_status} {code_note}".rstrip(),
                f"Difficulty: {problem.difficulty}",
                f"Topics: {', '.join(problem.topics)}",
            ]
        )

    def problem_status(self, problem=None) -> str:
        problem = problem or self.problem
        if problem.id in self.state.solved:
            return "solved"
        for item in reversed(self.state.history):
            if item.get("id") == problem.id:
                return item.get("status", "assigned")
        return "not_started"

    def submission_status(self, problem=None) -> tuple[str, str]:
        problem = problem or self.problem
        language = normalize_language(self.state.settings.language)
        path = self.root / "submissions" / problem.id / f"solution.{EXT[language]}"
        if not path.exists():
            return "missing", f"({language})"
        content = path.read_text()
        if content == template_for(language):
            return "template", f"({path.relative_to(self.root)})"
        if not content.strip():
            return "empty", f"({path.relative_to(self.root)})"
        return "written", f"({path.relative_to(self.root)})"

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
        bank = load_bank(Path.cwd())
        state = load_state(Path.cwd(), bank)
        print(problem_by_id(bank, state.current_problem).title[state.settings.ui_language])
        return
    CodeCodeApp().run()


if __name__ == "__main__":
    main()
