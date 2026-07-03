from __future__ import annotations

from dataclasses import dataclass, field
from importlib.resources import files
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import sys


LANGUAGES = ("python", "ts", "java", "rust")
UI_LANGUAGES = ("ko", "en")
EXT = {"python": "py", "ts": "ts", "java": "java", "rust": "rs"}


@dataclass
class Settings:
    language: str = "python"
    ui_language: str = "ko"
    editor: str = "vim"
    next_source: str = "bank"
    codex_next_command: str = ""


@dataclass
class AppState:
    current_problem: str
    settings: Settings = field(default_factory=Settings)
    solved: list[str] = field(default_factory=list)
    history: list[dict] = field(default_factory=list)
    suggested_next_difficulty: str = "easy"


@dataclass(frozen=True)
class Problem:
    id: str
    slug: str
    difficulty: str
    topics: list[str]
    title: dict[str, str]
    statement: dict[str, str]
    input: dict[str, str]
    output: dict[str, str]
    examples: list[dict[str, str]]
    cases: list[dict[str, str]]
    answers: dict[str, str]


@dataclass(frozen=True)
class JudgeResult:
    passed: bool
    passed_cases: int
    total_cases: int
    output: str


def load_bank() -> list[Problem]:
    data = json.loads(files("codecode").joinpath("problem_bank.json").read_text())
    return [Problem(**item) for item in data]


def load_state(root: Path, bank: list[Problem]) -> AppState:
    path = root / ".codex" / "problem-state.json"
    if not path.exists():
        return AppState(current_problem=bank[0].id, history=[{"id": bank[0].id, "status": "assigned"}])
    raw = json.loads(path.read_text())
    return AppState(
        current_problem=raw.get("current_problem", bank[0].id),
        settings=Settings(**raw.get("settings", {})),
        solved=raw.get("solved", []),
        history=raw.get("history", []),
        suggested_next_difficulty=raw.get("suggested_next_difficulty", "easy"),
    )


def save_state(root: Path, state: AppState) -> None:
    path = root / ".codex" / "problem-state.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            {
                "current_problem": state.current_problem,
                "next_number": len(state.history) + 1,
                "suggested_next_difficulty": state.suggested_next_difficulty,
                "settings": vars(state.settings),
                "solved": state.solved,
                "history": state.history,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n"
    )


def problem_by_id(bank: list[Problem], problem_id: str) -> Problem:
    return next(problem for problem in bank if problem.id == problem_id)


def ensure_submission(root: Path, problem: Problem, settings: Settings) -> Path:
    language = normalize_language(settings.language)
    path = root / "submissions" / problem.id / f"solution.{EXT[language]}"
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(template_for(language))
    return path


def ensure_edit_files(root: Path, problem: Problem, settings: Settings) -> tuple[Path, Path]:
    solution = ensure_submission(root, problem, settings)
    statement = solution.parent / "problem.md"
    statement.write_text(render_problem(problem, settings.ui_language) + "\n")
    return statement, solution


def edit_command(editor: str, statement: Path, solution: Path) -> list[str]:
    editor_name = Path(editor).name
    if editor_name in {"vim", "nvim", "vi"}:
        return [editor, "-O", str(statement), str(solution), "-c", "wincmd h | setlocal readonly nomodifiable | wincmd l"]
    return [editor, str(statement), str(solution)]


def template_for(language: str) -> str:
    if language == "python":
        return "# Read from stdin and print to stdout.\nimport sys\n\n\n"
    if language == "ts":
        return "const fs = require('fs');\nconst input = fs.readFileSync(0, 'utf8');\n\n"
    if language == "java":
        return "import java.io.*;\n\nclass Solution {\n    public static void main(String[] args) throws Exception {\n    }\n}\n"
    return "fn main() {\n}\n"


def judge(root: Path, problem: Problem, settings: Settings) -> JudgeResult:
    path = ensure_submission(root, problem, settings)
    command = command_for(root, path, normalize_language(settings.language))
    if not command:
        return JudgeResult(False, 0, len(problem.cases), f"Missing runtime for {settings.language}")

    passed = 0
    lines: list[str] = []
    for index, case in enumerate(problem.cases, 1):
        run = subprocess.run(command, input=case["input"], text=True, capture_output=True, timeout=5)
        got = run.stdout.strip()
        expected = case["output"].strip()
        if run.returncode == 0 and got == expected:
            passed += 1
            lines.append(f"case {index}: PASS")
        else:
            lines.append(f"case {index}: FAIL")
            lines.append(f"expected: {expected!r}")
            lines.append(f"got: {got!r}")
            if run.stderr.strip():
                lines.append(run.stderr.strip())
            break
    return JudgeResult(passed == len(problem.cases), passed, len(problem.cases), "\n".join(lines))


def command_for(root: Path, path: Path, language: str) -> list[str]:
    if language == "python":
        return [sys.executable, str(path)]
    if language == "ts":
        node = shutil.which("node")
        return [node, "--experimental-strip-types", str(path)] if node else []
    if language == "java":
        return compile_java(root, path)
    if language == "rust":
        return compile_rust(root, path)
    return []


def compile_java(root: Path, path: Path) -> list[str]:
    javac = shutil.which("javac")
    java = shutil.which("java")
    if not javac or not java:
        return []
    build = root / ".codex" / "build" / path.parent.name / "java"
    build.mkdir(parents=True, exist_ok=True)
    compiled = subprocess.run([javac, "-d", str(build), str(path)], capture_output=True, text=True)
    if compiled.returncode != 0:
        return ["sh", "-c", f"printf '%s' {json.dumps(compiled.stderr)} >&2; exit 1"]
    return [java, "-cp", str(build), "Solution"]


def compile_rust(root: Path, path: Path) -> list[str]:
    rustc = shutil.which("rustc")
    if not rustc:
        return []
    build = root / ".codex" / "build" / path.parent.name
    build.mkdir(parents=True, exist_ok=True)
    exe = build / "solution"
    compiled = subprocess.run([rustc, str(path), "-o", str(exe)], capture_output=True, text=True)
    if compiled.returncode != 0:
        return ["sh", "-c", f"printf '%s' {json.dumps(compiled.stderr)} >&2; exit 1"]
    return [str(exe)]


def give_up(root: Path, problem: Problem, state: AppState) -> str:
    language = normalize_language(state.settings.language)
    answer = problem.answers[language]
    mark_history(state, problem.id, "gave_up")
    upsert_problem_index(root, problem, "gave_up")
    save_state(root, state)
    return answer


def next_problem(root: Path, bank: list[Problem], state: AppState) -> Problem:
    seen = {item.get("id") for item in state.history}
    preferred = state.suggested_next_difficulty
    problem = next((item for item in bank if item.id not in seen and item.difficulty == preferred), None)
    if problem is None:
        problem = next((item for item in bank if item.id not in seen), bank[0])
    state.current_problem = problem.id
    mark_history(state, problem.id, "assigned")
    save_state(root, state)
    ensure_problem_files(root, problem)
    upsert_problem_index(root, problem, "assigned")
    return problem


def previous_problem(root: Path, bank: list[Problem], state: AppState) -> Problem:
    known_ids = {problem.id for problem in bank}
    history = [item.get("id") for item in state.history if item.get("id") in known_ids]
    if state.current_problem not in history:
        return problem_by_id(bank, state.current_problem)
    index = history.index(state.current_problem)
    if index == 0:
        return problem_by_id(bank, state.current_problem)
    state.current_problem = history[index - 1]
    save_state(root, state)
    return problem_by_id(bank, state.current_problem)


def record_pass(root: Path, problem: Problem, state: AppState) -> None:
    if problem.id not in state.solved:
        state.solved.append(problem.id)
    mark_history(state, problem.id, "solved")
    upsert_problem_index(root, problem, "solved")
    state.suggested_next_difficulty = "medium" if len(state.solved) >= 2 else "easy"
    save_state(root, state)


def run_codex_prompt(root: Path, problem: Problem, settings: Settings, prompt: str) -> str:
    solution = ensure_submission(root, problem, settings)
    code = solution.read_text()
    full_prompt = (
        "You are a concise coding-test coach. Help with the current problem and current submission. "
        "Prefer hints over full answers unless the user explicitly asks for the answer.\n\n"
        f"User request:\n{prompt}\n\n"
        f"Problem:\n{render_problem(problem, settings.ui_language)}\n\n"
        f"Current {settings.language} submission ({solution.relative_to(root)}):\n"
        f"```{normalize_language(settings.language)}\n{code}\n```"
    )
    command = ["codex", "exec", "--cd", str(root), "--sandbox", "read-only", full_prompt]
    result = subprocess.run(command, cwd=root, text=True, capture_output=True, timeout=600)
    output = "\n".join(part for part in [result.stdout.strip(), result.stderr.strip()] if part)
    if result.returncode != 0:
        return f"Codex prompt failed ({result.returncode})\n{output}"
    return output or "Codex returned no output."


def run_codex_next(root: Path, state: AppState) -> str:
    if state.settings.next_source != "codex":
        return "Codex next is disabled; using local problem bank."
    command = state.settings.codex_next_command or default_codex_next_command(root)
    # ponytail: trusted local hook; replace with app-server JSON-RPC when we need streamed progress.
    result = subprocess.run(command, cwd=root, shell=True, text=True, capture_output=True, timeout=600)
    output = "\n".join(part for part in [result.stdout.strip(), result.stderr.strip()] if part)
    if result.returncode != 0:
        return f"Codex command failed ({result.returncode})\n{output}"
    return f"Codex command finished\n{output}".strip()


def default_codex_next_command(root: Path) -> str:
    prompt = (
        "Read AGENTS.md, problems/INDEX.md, src/codecode/problem_bank.json, and .codex/problem-state.json. "
        "Create exactly one new non-duplicate coding practice problem. "
        "Update problem_bank.json, the index, and state files. Do not include the answer in the problem statement."
    )
    return (
        "codex app-server daemon start >/dev/null 2>&1; "
        f"codex exec --cd {shlex.quote(str(root))} --sandbox workspace-write "
        f"{shlex.quote(prompt)}"
    )


def ensure_problem_files(root: Path, problem: Problem) -> None:
    problem_dir = root / "problems" / problem.id
    problem_dir.mkdir(parents=True, exist_ok=True)
    readme = problem_dir / "README.md"
    if readme.exists():
        return
    examples = "\n".join(f"input:\n{ex['input']}output:\n{ex['output']}" for ex in problem.examples)
    readme.write_text(
        f"# {problem.id}. {problem.title['ko']}\n\n"
        f"난이도: {problem.difficulty}\n\n"
        f"{problem.statement['ko']}\n\n"
        f"## 입력\n\n{problem.input['ko']}\n\n"
        f"## 출력\n\n{problem.output['ko']}\n\n"
        f"## 예시\n\n```text\n{examples}\n```\n"
    )


def upsert_problem_index(root: Path, problem: Problem, status: str) -> None:
    index = root / "problems" / "INDEX.md"
    index.parent.mkdir(parents=True, exist_ok=True)
    rows: dict[str, tuple[str, str, str, str]] = {}
    if index.exists():
        for line in index.read_text().splitlines():
            parts = [part.strip() for part in line.strip().strip("|").split("|")]
            if len(parts) == 5 and parts[0].isdigit():
                rows[parts[0]] = (parts[1], parts[2], parts[3], parts[4])
    number = problem.id.split("-", 1)[0]
    rows[number] = (problem.slug, problem.difficulty, ", ".join(problem.topics), status)
    body = "\n".join(f"| {num} | {slug} | {difficulty} | {topics} | {row_status} |" for num, (slug, difficulty, topics, row_status) in sorted(rows.items()))
    index.write_text(
        "# Problem Index\n\n"
        "| # | Slug | Difficulty | Topics | Status |\n"
        "|---|------|------------|--------|--------|\n"
        f"{body}\n"
    )


def mark_history(state: AppState, problem_id: str, status: str) -> None:
    for item in state.history:
        if item.get("id") == problem_id:
            item["status"] = status
            return
    state.history.append({"id": problem_id, "status": status})


def normalize_language(language: str) -> str:
    return language if language in LANGUAGES else "python"


def render_problem(problem: Problem, ui_language: str) -> str:
    lang = ui_language if ui_language in UI_LANGUAGES else "ko"
    examples = "\n".join(f"> input\n{ex['input']}> output\n{ex['output']}" for ex in problem.examples)
    return (
        f"# {problem.title[lang]}\n\n"
        f"Difficulty: {problem.difficulty}\n"
        f"Topics: {', '.join(problem.topics)}\n\n"
        f"{problem.statement[lang]}\n\n"
        f"Input: {problem.input[lang]}\n"
        f"Output: {problem.output[lang]}\n\n"
        f"{examples}"
    )
