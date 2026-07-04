from dataclasses import replace
from pathlib import Path
import time

import pytest
from textual.widgets import Button, Markdown, Static, TextArea

from codecode.app import CodeCodeApp
from codecode.core import AppState, Problem, Settings, load_bank, save_bank, save_state


def output_text(app: CodeCodeApp) -> str:
    return app.query_one("#output", Markdown).source or ""


async def submit_command(app: CodeCodeApp, pilot, value: str, wait: bool = True) -> None:
    if app.focused is app.query_one("#code", TextArea):
        await pilot.press("escape")
        await pilot.pause()
    command = app.query_one("#command", TextArea)
    command.load_text(value)
    command.focus()
    await pilot.press("enter")
    if wait:
        await pilot.pause()


def two_problem_bank(root: Path) -> list[Problem]:
    first = load_bank(root)[0]
    second = replace(
        first,
        id="002-echo",
        slug="echo",
        topics=["io", "string"],
        title={"ko": "그대로 출력", "en": "Echo"},
        statement={"ko": "입력을 그대로 출력하세요.", "en": "Print stdin unchanged."},
        input={"ko": "문자열", "en": "A string"},
        output={"ko": "입력과 같은 문자열", "en": "The same string"},
        examples=[{"input": "code\n", "output": "code\n"}],
        cases=[{"input": "code\n", "output": "code\n"}],
        answers={
            "python": "import sys\nprint(sys.stdin.read(), end='')\n",
            "ts": "const fs = require('fs');\nprocess.stdout.write(fs.readFileSync(0, 'utf8'));\n",
            "java": "class Solution { public static void main(String[] args) throws Exception { System.out.print(new String(System.in.readAllBytes())); } }\n",
            "rust": "use std::io::{self, Read};\nfn main() { let mut s = String::new(); io::stdin().read_to_string(&mut s).unwrap(); print!(\"{}\", s); }\n",
        },
    )
    bank = [first, second]
    save_bank(root, bank)
    return bank


@pytest.mark.asyncio
async def test_app_renders_current_problem(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        problem = app.query_one("#problem", Markdown)
        code = app.query_one("#code", TextArea)
        status = app.query_one("#status", Static)
        focused = app.focused

    assert problem is not None
    assert code.text.startswith("# Read from stdin")
    assert focused is code
    assert app.problem.title["ko"] == "Hello World"
    assert "CODECODE" in str(status.content)
    assert "python" in str(status.content)


@pytest.mark.asyncio
async def test_main_panes_have_matching_height(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        problem = app.query_one("#problem", Markdown)
        code = app.query_one("#code", TextArea)
        status = app.query_one("#status", Static)
        command = app.query_one("#command", TextArea)
        problem_region = problem.region
        code_region = code.region
        status_region = status.region
        command_region = command.region

    assert problem_region.height == code_region.height
    assert status_region.y > problem_region.y + problem_region.height
    assert command_region.y > status_region.y + status_region.height


@pytest.mark.asyncio
async def test_code_editor_saves_current_submission(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        code = app.query_one("#code", TextArea)
        code.load_text("")
        code.insert("print('Hello, World!')\n")
        await pilot.pause()

    assert (tmp_path / "submissions" / "001-hello-world" / "solution.py").read_text() == "print('Hello, World!')\n"


@pytest.mark.asyncio
async def test_question_mark_opens_help_outside_editor(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("escape")
        await pilot.pause()
        await pilot.press("?")
        await pilot.pause()
        source = output_text(app)

    assert "# Help" in source


@pytest.mark.asyncio
async def test_next_key_loads_next_problem(tmp_path: Path):
    two_problem_bank(tmp_path)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "next")
        problem = app.query_one("#problem", Markdown)

    assert problem is not None
    assert app.problem.title["ko"] == "그대로 출력"


@pytest.mark.asyncio
async def test_codex_next_shows_loading_without_blocking(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    save_state(
        tmp_path,
        AppState(current_problem="001-hello-world", settings=Settings(next_source="codex")),
    )

    def slow_next(*args, **kwargs):
        time.sleep(1)
        return "Codex command finished"

    monkeypatch.setattr("codecode.app.run_codex_next", slow_next)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "next", wait=False)
        await pilot.pause(0.1)
        output = app.query_one("#output", Markdown)

        assert not output.loading
        assert "Generating next problem" in output_text(app)
        assert "busy:next" in str(app.query_one("#status", Static).content)


@pytest.mark.asyncio
async def test_bank_next_flows_to_codex_when_bank_is_exhausted(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    bank = load_bank(tmp_path)
    save_state(
        tmp_path,
        AppState(
            current_problem=bank[-1].id,
            settings=Settings(next_source="bank"),
            history=[{"id": problem.id, "status": "solved"} for problem in bank],
        ),
    )
    captured = {}

    def slow_next(root, state, force=False):
        captured["force"] = force
        time.sleep(1)
        return "Codex command finished"

    monkeypatch.setattr("codecode.app.run_codex_next", slow_next)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "next", wait=False)
        await pilot.pause(0.1)
        output = app.query_one("#output", Markdown)

        assert not output.loading
        assert "Generating next problem" in output_text(app)
        assert captured == {"force": True}


@pytest.mark.asyncio
async def test_codex_next_falls_back_to_bank_when_current_problem_does_not_change(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    two_problem_bank(tmp_path)
    save_state(
        tmp_path,
        AppState(
            current_problem="001-hello-world",
            settings=Settings(next_source="codex"),
            history=[{"id": "001-hello-world", "status": "assigned"}],
        ),
    )
    monkeypatch.setattr("codecode.app.run_codex_next", lambda *args, **kwargs: "Codex command finished")
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "next")
        await pilot.pause(0.1)
        assert app.query_one("#code", TextArea).text.startswith("# Read from stdin")

    assert app.problem.title["ko"] == "그대로 출력"


@pytest.mark.asyncio
async def test_previous_key_loads_previous_problem(tmp_path: Path):
    two_problem_bank(tmp_path)
    state = AppState(
        current_problem="002-echo",
        history=[
            {"id": "001-hello-world", "status": "solved"},
            {"id": "002-echo", "status": "assigned"},
        ],
    )
    save_state(tmp_path, state)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "prev")

    assert app.problem.title["ko"] == "Hello World"


@pytest.mark.asyncio
async def test_escape_exits_command_input(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("escape")
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        command = app.query_one("#command", TextArea)
        assert app.focused is command

        await pilot.press("h", "e", "l", "p")
        assert command.text == "/help"

        await pilot.press("escape")
        await pilot.pause()

        assert command.text == ""
        assert app.focused is not command


@pytest.mark.asyncio
async def test_exit_command_quits_app(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    app = CodeCodeApp(root=tmp_path)
    called = {}

    def fake_exit(*args, **kwargs):
        called["exit"] = True

    monkeypatch.setattr(app, "exit", fake_exit)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "exit")

    assert called == {"exit": True}


@pytest.mark.asyncio
async def test_slash_commands_run_actions(tmp_path: Path):
    two_problem_bank(tmp_path)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        app.query_one("#code", TextArea).load_text("print('Hello, World!')\n")
        await pilot.pause()
        await submit_command(app, pilot, "help")
        assert "Commands" in output_text(app)

        await submit_command(app, pilot, "run")
        assert "case 1:" in output_text(app)
        assert "PASS 1/1" in output_text(app)
        assert "Next: /next" in output_text(app)

        await submit_command(app, pilot, "next")
        assert app.problem.title["ko"] == "그대로 출력"

        await submit_command(app, pilot, "prev")

    assert app.problem.title["ko"] == "Hello World"


@pytest.mark.asyncio
async def test_list_and_open_commands_show_and_load_problems(tmp_path: Path):
    two_problem_bank(tmp_path)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "list")
        assert "001-hello-world" in output_text(app)
        assert "002-echo" in output_text(app)

        await submit_command(app, pilot, "open 2")

    assert app.problem.title["ko"] == "그대로 출력"


@pytest.mark.asyncio
async def test_list_command_selects_problem_with_arrows(tmp_path: Path):
    two_problem_bank(tmp_path)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "list")
        assert "> *  1 001-hello-world" in output_text(app)

        await pilot.press("down")
        await pilot.pause()
        assert ">    2 002-echo" in output_text(app)

        await pilot.press("enter")
        await pilot.pause()

    assert app.problem.title["ko"] == "그대로 출력"


@pytest.mark.asyncio
async def test_problem_view_shows_problem_number(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        problem = app.query_one("#problem", Markdown)

    assert "# 001. Hello World" in (problem.source or "")


@pytest.mark.asyncio
async def test_open_command_shows_problem_status_and_submission_state(tmp_path: Path):
    two_problem_bank(tmp_path)
    state = AppState(
        current_problem="002-echo",
        history=[
            {"id": "001-hello-world", "status": "solved"},
            {"id": "002-echo", "status": "assigned"},
        ],
    )
    save_state(tmp_path, state)
    submission = tmp_path / "submissions" / "001-hello-world" / "solution.py"
    submission.parent.mkdir(parents=True)
    submission.write_text("print('done')\n")
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "open 1")
        status = app.query_one("#status", Static)
        code = app.query_one("#code", TextArea)

    assert app.problem.title["ko"] == "Hello World"
    assert "print('done')" in code.text
    assert "| solved |" in str(status.content)
    assert "code:written" in str(status.content)


@pytest.mark.asyncio
async def test_open_command_reports_missing_submission_without_creating_it(tmp_path: Path):
    two_problem_bank(tmp_path)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "open 2")
        code = app.query_one("#code", TextArea)

    assert code.text.startswith("# Read from stdin")
    assert (tmp_path / "submissions" / "002-echo" / "solution.py").exists()


@pytest.mark.asyncio
async def test_codex_command_prints_response_without_changing_next_settings(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    captured = {}

    def fake_codex(root, problem, settings, prompt):
        captured["problem"] = problem.id
        captured["language"] = settings.language
        captured["prompt"] = prompt
        return f"Codex says: {prompt}"

    monkeypatch.setattr("codecode.app.run_codex_prompt", fake_codex)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "codex hello")
        await pilot.pause(0.1)
        assert "Codex says: hello" in output_text(app)

    assert captured == {"problem": "001-hello-world", "language": "python", "prompt": "hello"}
    assert app.state.settings.next_source == "bank"
    assert app.state.settings.codex_next_command == ""


@pytest.mark.asyncio
async def test_command_editor_accepts_korean_codex_prompt(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    captured = {}

    def fake_codex(root, problem, settings, prompt):
        captured["prompt"] = prompt
        return "ok"

    monkeypatch.setattr("codecode.app.run_codex_prompt", fake_codex)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("escape")
        await pilot.pause()
        command = app.query_one("#command", TextArea)
        command.load_text("/codex 한글 힌트 줘")
        command.focus()
        await pilot.press("enter")
        await pilot.pause(0.1)

    assert captured == {"prompt": "한글 힌트 줘"}


@pytest.mark.asyncio
async def test_codex_command_shows_loading_in_scrollable_output(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    def slow_codex(*args):
        time.sleep(1)
        return "later"

    monkeypatch.setattr("codecode.app.run_codex_prompt", slow_codex)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await submit_command(app, pilot, "codex hi", wait=False)
        await pilot.pause(0.1)
        output = app.query_one("#output", Markdown)
        assert output.can_focus
        assert not output.loading
        assert "Codex is thinking" in output_text(app)
        assert "busy:codex" in str(app.query_one("#status", Static).content)


@pytest.mark.asyncio
async def test_output_renders_markdown_for_codex_answers(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        app.write_output("Use `stdin`:\n\n```python\nseq = sys.stdin.read().split()\n```")
        await pilot.pause()
        output = app.query_one("#output", Markdown)

    assert output.can_focus
    assert "```python" in (output.source or "")


@pytest.mark.asyncio
async def test_tui_uses_statusline_instead_of_buttons(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        status = app.query_one("#status", Static)
        buttons = app.query(Button)

    assert "python" in str(status.content)
    assert "Esc then / command" in str(status.content)
    assert len(buttons) == 0


@pytest.mark.asyncio
async def test_theme_command_toggles_and_saves_theme(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        assert app.state.settings.theme == "dark"

        await submit_command(app, pilot, "theme")
        status = app.query_one("#status", Static)
        has_light_class = app.screen.has_class("theme-light")

    saved = (tmp_path / ".codex" / "problem-state.json").read_text()
    assert app.state.settings.theme == "light"
    assert has_light_class
    assert "CODECODE" in str(status.content)
    assert '"theme": "light"' in saved
