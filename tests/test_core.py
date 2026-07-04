import json
from dataclasses import replace
from pathlib import Path
import subprocess
import sys

import pytest

from codecode.core import (
    AppState,
    Problem,
    Settings,
    edit_command,
    ensure_edit_files,
    ensure_submission,
    give_up,
    judge,
    load_bank,
    load_state,
    next_problem,
    previous_problem,
    record_pass,
    render_problem,
    run_codex_next,
    run_codex_prompt,
    save_bank,
    save_state,
)


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


def test_load_state_uses_first_problem_when_state_file_is_missing(tmp_path: Path):
    bank = load_bank(tmp_path)

    state = load_state(tmp_path, bank)

    assert state.current_problem == "001-hello-world"
    assert state.settings.language == "python"
    assert state.settings.ui_language == "ko"


def test_save_bank_creates_local_custom_problem_bank(tmp_path: Path):
    bank = two_problem_bank(tmp_path)

    loaded = load_bank(tmp_path)

    assert (tmp_path / ".codecode" / "problem_bank.json").exists()
    assert [problem.id for problem in loaded] == [problem.id for problem in bank]


def test_ensure_submission_creates_language_template(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", settings=Settings(language="rust"))

    path = ensure_submission(tmp_path, bank[0], state.settings)

    assert path == tmp_path / "submissions" / "001-hello-world" / "solution.rs"
    assert "fn main()" in path.read_text()


def test_ensure_edit_files_creates_problem_statement_and_vim_split_command(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", settings=Settings(language="python"))

    statement, solution = ensure_edit_files(tmp_path, bank[0], state.settings)

    assert statement == tmp_path / "submissions" / "001-hello-world" / "problem.md"
    assert solution == tmp_path / "submissions" / "001-hello-world" / "solution.py"
    assert "Hello World" in statement.read_text()
    assert edit_command("vim", statement, solution) == [
        "vim",
        "-O",
        str(statement),
        str(solution),
        "-c",
        "wincmd h | setlocal readonly nomodifiable | wincmd l",
    ]


def test_render_problem_separates_input_output_blocks(tmp_path: Path):
    problem = load_bank(tmp_path)[0]

    rendered = render_problem(problem, "ko")

    assert "## Input\n\n입력은 없습니다.\n\n## Output\n\n`Hello, World!` 한 줄" in rendered
    assert "Input:" not in rendered
    assert "Output:" not in rendered
    assert "```text\n\n```\n" in rendered


def test_judge_runs_python_solution_against_cases(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", settings=Settings(language="python"))
    path = ensure_submission(tmp_path, bank[0], state.settings)
    path.write_text("print('Hello, World!')\n")

    result = judge(tmp_path, bank[0], state.settings)

    assert result.passed
    assert result.passed_cases == result.total_cases


def test_judge_shows_debug_stdout_on_failure(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", settings=Settings(language="python"))
    path = ensure_submission(tmp_path, bank[0], state.settings)
    path.write_text("print('debug')\nprint('Hello, World!')\n")

    result = judge(tmp_path, bank[0], state.settings)

    assert not result.passed
    assert "stdout:\ndebug\nHello, World!" in result.output


def test_judge_shows_debug_stderr_without_failing_case(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", settings=Settings(language="python"))
    path = ensure_submission(tmp_path, bank[0], state.settings)
    path.write_text("import sys\nprint('debug', file=sys.stderr)\nprint('Hello, World!')\n")

    result = judge(tmp_path, bank[0], state.settings)

    assert result.passed
    assert "stderr:\ndebug" in result.output


def test_give_up_marks_problem_and_returns_answer(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world")

    answer = give_up(tmp_path, bank[0], state)
    saved = json.loads((tmp_path / ".codex" / "problem-state.json").read_text())

    assert "Hello, World!" in answer
    assert saved["history"][0]["status"] == "gave_up"


def test_next_problem_skips_history_and_saves_new_current(tmp_path: Path):
    bank = two_problem_bank(tmp_path)
    state = AppState(
        current_problem="001-hello-world",
        history=[{"id": "001-hello-world", "status": "solved"}],
    )
    save_state(tmp_path, state)

    problem = next_problem(tmp_path, bank, state)
    saved = load_state(tmp_path, bank)

    assert problem.id == "002-echo"
    assert saved.current_problem == "002-echo"
    assert "002 | echo" in (tmp_path / "problems" / "INDEX.md").read_text()


def test_next_problem_returns_none_when_bank_is_exhausted(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(
        current_problem=bank[-1].id,
        history=[{"id": problem.id, "status": "solved"} for problem in bank],
    )
    save_state(tmp_path, state)

    problem = next_problem(tmp_path, bank, state)
    saved = load_state(tmp_path, bank)

    assert problem is None
    assert saved.current_problem == bank[-1].id


def test_previous_problem_uses_history_order(tmp_path: Path):
    bank = two_problem_bank(tmp_path)
    state = AppState(
        current_problem="002-echo",
        history=[
            {"id": "001-hello-world", "status": "solved"},
            {"id": "002-echo", "status": "assigned"},
        ],
    )

    problem = previous_problem(tmp_path, bank, state)
    saved = load_state(tmp_path, bank)

    assert problem.id == "001-hello-world"
    assert saved.current_problem == "001-hello-world"


def test_record_pass_marks_solved_and_raises_suggested_difficulty_after_two_solves(tmp_path: Path):
    bank = load_bank(tmp_path)
    state = AppState(current_problem="001-hello-world", solved=["000-warmup"])

    record_pass(tmp_path, bank[0], state)
    saved = load_state(tmp_path, bank)

    assert "001-hello-world" in saved.solved
    assert saved.history[0]["status"] == "solved"
    assert saved.suggested_next_difficulty == "medium"


def test_run_codex_next_executes_configured_command_in_repo_root(tmp_path: Path):
    command = (
        f"{sys.executable} -c "
        "\"from pathlib import Path; Path('codex-made.txt').write_text('ok')\""
    )
    state = AppState(
        current_problem="001-hello-world",
        settings=Settings(next_source="codex", codex_next_command=command),
    )

    output = run_codex_next(tmp_path, state)

    assert "finished" in output
    assert (tmp_path / "codex-made.txt").read_text() == "ok"


def test_run_codex_next_can_be_forced_from_bank_mode(tmp_path: Path):
    command = (
        f"{sys.executable} -c "
        "\"from pathlib import Path; Path('codex-forced.txt').write_text('ok')\""
    )
    state = AppState(
        current_problem="001-hello-world",
        settings=Settings(next_source="bank", codex_next_command=command),
    )

    output = run_codex_next(tmp_path, state, force=True)

    assert "finished" in output
    assert (tmp_path / "codex-forced.txt").read_text() == "ok"


def test_run_codex_prompt_includes_problem_and_submission_context(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    bank = load_bank(tmp_path)
    problem = bank[0]
    settings = Settings(language="python")
    ensure_submission(tmp_path, problem, settings).write_text("print('work in progress')\n")
    captured = {}

    def fake_run(command, cwd, text, capture_output, timeout):
        captured["command"] = command
        captured["cwd"] = cwd
        return subprocess.CompletedProcess(command, 0, stdout="hint response\n", stderr="")

    monkeypatch.setattr("codecode.core.subprocess.run", fake_run)

    output = run_codex_prompt(tmp_path, problem, settings, "give me a hint")

    assert output == "hint response"
    assert captured["cwd"] == tmp_path
    assert captured["command"][:6] == [
        "codex",
        "exec",
        "--cd",
        str(tmp_path),
        "--sandbox",
        "read-only",
    ]
    prompt = captured["command"][-1]
    assert "give me a hint" in prompt
    assert "Hello World" in prompt
    assert "print('work in progress')" in prompt


def test_run_codex_prompt_returns_only_last_message_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    bank = load_bank(tmp_path)
    problem = bank[0]
    settings = Settings(language="python")

    def fake_run(command, cwd, text, capture_output, timeout):
        output_path = Path(command[command.index("-o") + 1])
        output_path.write_text("final hint only\n")
        return subprocess.CompletedProcess(
            command,
            0,
            stdout="final hint only\n",
            stderr="workdir: /tmp\nsandbox: read-only\nuser\nfull prompt echo\n",
        )

    monkeypatch.setattr("codecode.core.subprocess.run", fake_run)

    output = run_codex_prompt(tmp_path, problem, settings, "hint")

    assert output == "final hint only"
    assert "workdir:" not in output
    assert "full prompt echo" not in output
