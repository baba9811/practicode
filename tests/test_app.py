from pathlib import Path
import time

import pytest
from textual.widgets import Button, Input, Markdown, Static

from codecode.app import CodeCodeApp
from codecode.core import AppState, Settings, save_state


def output_text(app: CodeCodeApp) -> str:
    return app.query_one("#output", Markdown).source or ""


@pytest.mark.asyncio
async def test_app_renders_current_problem(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        problem = app.query_one("#problem", Markdown)
        status = app.query_one("#status", Static)

    assert problem is not None
    assert app.problem.title["ko"] == "누적 합"
    assert "CODECODE" in str(status.content)
    assert "python" in str(status.content)


@pytest.mark.asyncio
async def test_next_key_loads_next_problem(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("n")
        await pilot.pause()
        problem = app.query_one("#problem", Markdown)

    assert problem is not None
    assert app.problem.title["ko"] == "모음 세기"


@pytest.mark.asyncio
async def test_codex_next_shows_loading_without_blocking(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    save_state(
        tmp_path,
        AppState(current_problem="001-running-sum", settings=Settings(next_source="codex")),
    )

    def slow_next(*args):
        time.sleep(0.5)
        return "Codex command finished"

    monkeypatch.setattr("codecode.app.run_codex_next", slow_next)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("n", "e", "x", "t", "enter")
        await pilot.pause()
        output = app.query_one("#output", Markdown)

        assert output.loading
        assert "Loading next problem..." in output_text(app)


@pytest.mark.asyncio
async def test_codex_next_falls_back_to_bank_when_current_problem_does_not_change(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    save_state(
        tmp_path,
        AppState(
            current_problem="001-running-sum",
            settings=Settings(next_source="codex"),
            history=[{"id": "001-running-sum", "status": "assigned"}],
        ),
    )
    monkeypatch.setattr("codecode.app.run_codex_next", lambda *args: "Codex command finished")
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("n", "e", "x", "t", "enter")
        await pilot.pause(0.1)
        assert "Codex command finished" in output_text(app)

    assert app.problem.title["ko"] == "모음 세기"


@pytest.mark.asyncio
async def test_previous_key_loads_previous_problem(tmp_path: Path):
    state = AppState(
        current_problem="002-count-vowels",
        history=[
            {"id": "001-running-sum", "status": "solved"},
            {"id": "002-count-vowels", "status": "assigned"},
        ],
    )
    save_state(tmp_path, state)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("p")
        await pilot.pause()

    assert app.problem.title["ko"] == "누적 합"


@pytest.mark.asyncio
async def test_escape_exits_command_input(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        command = app.query_one("#command", Input)
        assert app.focused is command

        await pilot.press("h", "e", "l", "p")
        assert command.value == "help"

        await pilot.press("escape")
        await pilot.pause()

        assert command.value == ""
        assert app.focused is not command


@pytest.mark.asyncio
async def test_slash_commands_run_actions(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("h", "e", "l", "p", "enter")
        await pilot.pause()
        assert "Commands" in output_text(app)

        await pilot.press("/")
        await pilot.pause()
        await pilot.press("r", "u", "n", "enter")
        await pilot.pause()
        assert "case 1:" in output_text(app)

        await pilot.press("/")
        await pilot.pause()
        await pilot.press("n", "e", "x", "t", "enter")
        await pilot.pause()
        assert app.problem.title["ko"] == "모음 세기"

        await pilot.press("/")
        await pilot.pause()
        await pilot.press("p", "r", "e", "v", "enter")
        await pilot.pause()

    assert app.problem.title["ko"] == "누적 합"


@pytest.mark.asyncio
async def test_list_and_open_commands_show_and_load_problems(tmp_path: Path):
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("l", "i", "s", "t", "enter")
        await pilot.pause()
        assert "001-running-sum" in output_text(app)
        assert "002-count-vowels" in output_text(app)

        await pilot.press("/")
        await pilot.pause()
        await pilot.press("o", "p", "e", "n", " ", "2", "enter")
        await pilot.pause()

    assert app.problem.title["ko"] == "모음 세기"


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
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("c", "o", "d", "e", "x", " ", "h", "e", "l", "l", "o", "enter")
        await pilot.pause(0.1)
        assert "Codex says: hello" in output_text(app)

    assert captured == {"problem": "001-running-sum", "language": "python", "prompt": "hello"}
    assert app.state.settings.next_source == "bank"
    assert app.state.settings.codex_next_command == ""


@pytest.mark.asyncio
async def test_codex_command_shows_loading_in_scrollable_output(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    def slow_codex(*args):
        time.sleep(0.5)
        return "later"

    monkeypatch.setattr("codecode.app.run_codex_prompt", slow_codex)
    app = CodeCodeApp(root=tmp_path)

    async with app.run_test(size=(100, 35)) as pilot:
        await pilot.pause()
        await pilot.press("/")
        await pilot.pause()
        await pilot.press("c", "o", "d", "e", "x", " ", "h", "i", "enter")
        await pilot.pause()
        output = app.query_one("#output", Markdown)
        assert output.can_focus
        assert output.loading
        assert "Thinking..." in output_text(app)


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

    assert "lang:python" in str(status.content)
    assert "ui:ko" in str(status.content)
    assert len(buttons) == 0
