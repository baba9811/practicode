from pathlib import Path

import pytest
from textual.widgets import Button, Input, Markdown, Static

from codecode.app import CodeCodeApp
from codecode.core import AppState, save_state


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
        output = app.query_one("#output", Static)
        assert "Commands" in str(output.content)

        await pilot.press("/")
        await pilot.pause()
        await pilot.press("r", "u", "n", "enter")
        await pilot.pause()
        assert "case 1:" in str(output.content)

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
        output = app.query_one("#output", Static)
        assert "001-running-sum" in str(output.content)
        assert "002-count-vowels" in str(output.content)

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
        await pilot.pause()
        output = app.query_one("#output", Static)

    assert "Codex says: hello" in str(output.content)
    assert captured == {"problem": "001-running-sum", "language": "python", "prompt": "hello"}
    assert app.state.settings.next_source == "bank"
    assert app.state.settings.codex_next_command == ""


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
