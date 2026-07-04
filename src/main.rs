use anyhow::{Context, Result, anyhow};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use unicode_width::UnicodeWidthStr;
use wait_timeout::ChildExt;

const LANGUAGES: &[&str] = &["python", "ts", "java", "rust"];
const UI_LANGUAGES: &[&str] = &["ko", "en"];
const THEMES: &[&str] = &["dark", "light"];
const BANK_PATH: &str = ".codecode/problem_bank.json";
const HELP: &str = r#"# Help

## Daily loop

1. Type code in the right pane.
2. Press `Esc`, then `/run`.
3. Use `/next` when it passes.

## Commands

- `/run` judge current submission
- `/edit` focus the code editor
- `/next [request]` next problem, optionally with a request
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
"#;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Settings {
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_ui_language")]
    ui_language: String,
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default = "default_editor")]
    editor: String,
    #[serde(default = "default_next_source")]
    next_source: String,
    #[serde(default)]
    codex_next_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: default_language(),
            ui_language: default_ui_language(),
            theme: default_theme(),
            editor: default_editor(),
            next_source: default_next_source(),
            codex_next_command: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HistoryItem {
    id: String,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppState {
    current_problem: String,
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    solved: Vec<String>,
    #[serde(default)]
    history: Vec<HistoryItem>,
    #[serde(default = "default_suggested_difficulty")]
    suggested_next_difficulty: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Problem {
    id: String,
    slug: String,
    difficulty: String,
    topics: Vec<String>,
    title: HashMap<String, String>,
    statement: HashMap<String, String>,
    input: HashMap<String, String>,
    output: HashMap<String, String>,
    examples: Vec<IoCase>,
    cases: Vec<IoCase>,
    answers: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IoCase {
    input: String,
    output: String,
}

#[derive(Clone, Debug)]
struct JudgeResult {
    passed: bool,
    passed_cases: usize,
    total_cases: usize,
    output: String,
}

#[derive(Clone, Debug)]
struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
}

#[derive(Clone, Debug)]
struct RunOutput {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

fn default_language() -> String {
    "python".to_string()
}

fn default_ui_language() -> String {
    "ko".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_editor() -> String {
    "vim".to_string()
}

fn default_next_source() -> String {
    "bank".to_string()
}

fn default_suggested_difficulty() -> String {
    "easy".to_string()
}

fn main() -> Result<()> {
    let root = env::current_dir().context("read current directory")?;
    if env::args().any(|arg| arg == "--smoke") {
        let bank = load_bank(&root)?;
        let state = load_state(&root, &bank)?;
        let problem = problem_by_id(&bank, &state.current_problem).unwrap_or(&bank[0]);
        println!("{}", localized(&problem.title, &state.settings.ui_language));
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let result = CodeCodeApp::new(root)?.run(&mut terminal);
    ratatui::restore();
    result
}

fn ext_for(language: &str) -> &'static str {
    match normalize_language(language).as_str() {
        "python" => "py",
        "ts" => "ts",
        "java" => "java",
        "rust" => "rs",
        _ => "py",
    }
}

fn starter_problem() -> Problem {
    Problem {
        id: "001-hello-world".to_string(),
        slug: "hello-world".to_string(),
        difficulty: "easy".to_string(),
        topics: vec!["io".to_string()],
        title: map2("ko", "Hello World", "en", "Hello World"),
        statement: map2(
            "ko",
            "표준 출력으로 정확히 `Hello, World!`를 출력하세요.",
            "en",
            "Print exactly `Hello, World!` to stdout.",
        ),
        input: map2("ko", "입력은 없습니다.", "en", "No input."),
        output: map2(
            "ko",
            "`Hello, World!` 한 줄",
            "en",
            "One line: `Hello, World!`",
        ),
        examples: vec![IoCase {
            input: String::new(),
            output: "Hello, World!\n".to_string(),
        }],
        cases: vec![IoCase {
            input: String::new(),
            output: "Hello, World!\n".to_string(),
        }],
        answers: HashMap::from([
            ("python".to_string(), "print('Hello, World!')\n".to_string()),
            (
                "ts".to_string(),
                "console.log('Hello, World!');\n".to_string(),
            ),
            (
                "java".to_string(),
                "class Solution {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, World!\");\n    }\n}\n".to_string(),
            ),
            (
                "rust".to_string(),
                "fn main() {\n    println!(\"Hello, World!\");\n}\n".to_string(),
            ),
        ]),
    }
}

fn map2(k1: &str, v1: &str, k2: &str, v2: &str) -> HashMap<String, String> {
    HashMap::from([
        (k1.to_string(), v1.to_string()),
        (k2.to_string(), v2.to_string()),
    ])
}

fn load_bank(root: &Path) -> Result<Vec<Problem>> {
    let path = root.join(BANK_PATH);
    if path.exists() {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let bank: Vec<Problem> =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        Ok(bank)
    } else {
        Ok(vec![starter_problem()])
    }
}

#[cfg(test)]
fn save_bank(root: &Path, bank: &[Problem]) -> Result<()> {
    let path = root.join(BANK_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(bank)? + "\n")?;
    Ok(())
}

fn load_state(root: &Path, bank: &[Problem]) -> Result<AppState> {
    let path = root.join(".codex/problem-state.json");
    if !path.exists() {
        return Ok(AppState {
            current_problem: bank[0].id.clone(),
            settings: Settings::default(),
            solved: Vec::new(),
            history: vec![HistoryItem {
                id: bank[0].id.clone(),
                status: "assigned".to_string(),
            }],
            suggested_next_difficulty: "easy".to_string(),
        });
    }

    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut state: AppState =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if !bank
        .iter()
        .any(|problem| problem.id == state.current_problem)
    {
        state.current_problem = bank[0].id.clone();
    }
    state.settings.language = normalize_language(&state.settings.language);
    if !UI_LANGUAGES.contains(&state.settings.ui_language.as_str()) {
        state.settings.ui_language = "ko".to_string();
    }
    if !THEMES.contains(&state.settings.theme.as_str()) {
        state.settings.theme = "dark".to_string();
    }
    if state.history.is_empty() {
        state.history.push(HistoryItem {
            id: state.current_problem.clone(),
            status: "assigned".to_string(),
        });
    }
    Ok(state)
}

fn save_state(root: &Path, state: &AppState) -> Result<()> {
    #[derive(Serialize)]
    struct StateFile<'a> {
        current_problem: &'a str,
        next_number: usize,
        suggested_next_difficulty: &'a str,
        settings: &'a Settings,
        solved: &'a [String],
        history: &'a [HistoryItem],
    }

    let path = root.join(".codex/problem-state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = StateFile {
        current_problem: &state.current_problem,
        next_number: state.history.len() + 1,
        suggested_next_difficulty: &state.suggested_next_difficulty,
        settings: &state.settings,
        solved: &state.solved,
        history: &state.history,
    };
    fs::write(path, serde_json::to_string_pretty(&file)? + "\n")?;
    Ok(())
}

fn problem_by_id<'a>(bank: &'a [Problem], problem_id: &str) -> Option<&'a Problem> {
    bank.iter().find(|problem| problem.id == problem_id)
}

fn normalize_language(language: &str) -> String {
    if LANGUAGES.contains(&language) {
        language.to_string()
    } else {
        "python".to_string()
    }
}

fn localized(map: &HashMap<String, String>, lang: &str) -> String {
    map.get(lang)
        .or_else(|| map.get("ko"))
        .or_else(|| map.get("en"))
        .or_else(|| map.values().next())
        .cloned()
        .unwrap_or_default()
}

fn template_for(language: &str) -> String {
    match normalize_language(language).as_str() {
        "python" => "# Read from stdin and print to stdout.\nimport sys\n\n\n".to_string(),
        "ts" => "const fs = require('fs');\nconst input = fs.readFileSync(0, 'utf8');\n\n".to_string(),
        "java" => "import java.io.*;\n\nclass Solution {\n    public static void main(String[] args) throws Exception {\n    }\n}\n".to_string(),
        "rust" => "fn main() {\n}\n".to_string(),
        _ => String::new(),
    }
}

fn ensure_submission(root: &Path, problem: &Problem, settings: &Settings) -> Result<PathBuf> {
    let language = normalize_language(&settings.language);
    let path = root
        .join("submissions")
        .join(&problem.id)
        .join(format!("solution.{}", ext_for(&language)));
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, template_for(&language))?;
    }
    Ok(path)
}

fn render_problem(problem: &Problem, ui_language: &str) -> String {
    let lang = if UI_LANGUAGES.contains(&ui_language) {
        ui_language
    } else {
        "ko"
    };
    let examples = problem
        .examples
        .iter()
        .enumerate()
        .map(|(index, case)| {
            format!(
                "### Example {}\n\nInput\n\n{}\n\nOutput\n\n{}",
                index + 1,
                fenced_text(&case.input),
                fenced_text(&case.output)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    format!(
        "# {number}. {}\n\nDifficulty: {}\nTopics: {}\n\n{}\n\n## Input\n\n{}\n\n## Output\n\n{}\n\n## Examples\n\n{}",
        localized(&problem.title, lang),
        problem.difficulty,
        problem.topics.join(", "),
        localized(&problem.statement, lang),
        localized(&problem.input, lang),
        localized(&problem.output, lang),
        examples
    )
}

fn fenced_text(value: &str) -> String {
    let mut body = value.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    format!("```text\n{body}```")
}

fn render_markdown_plain(markdown: &str) -> String {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push(line.to_string());
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push(trimmed.trim_start_matches('#').trim_start().to_string());
        } else {
            out.push(line.replace('`', ""));
        }
    }
    out.join("\n").trim_end().to_string()
}

fn judge(root: &Path, problem: &Problem, settings: &Settings) -> JudgeResult {
    let path = match ensure_submission(root, problem, settings) {
        Ok(path) => path,
        Err(error) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: problem.cases.len(),
                output: error.to_string(),
            };
        }
    };
    let language = normalize_language(&settings.language);
    let command = match command_for(root, &path, &language) {
        Ok(Some(command)) => command,
        Ok(None) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: problem.cases.len(),
                output: format!("Missing runtime for {}", settings.language),
            };
        }
        Err(error) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: problem.cases.len(),
                output: format!("compile failed\n{error}"),
            };
        }
    };

    let mut passed = 0;
    let mut lines = Vec::new();
    for (index, case) in problem.cases.iter().enumerate() {
        let mut process = Command::new(&command.program);
        process.args(&command.args).current_dir(root);
        let run = match run_capture(&mut process, &case.input, Duration::from_secs(5)) {
            Ok(run) => run,
            Err(error) => {
                lines.push(format!("case {}: FAIL", index + 1));
                lines.push(error.to_string());
                break;
            }
        };
        let got = run.stdout.trim();
        let expected = case.output.trim();
        if !run.timed_out && run.code == Some(0) && got == expected {
            passed += 1;
            lines.push(format!("case {}: PASS", index + 1));
            if !run.stderr.trim().is_empty() {
                lines.push("stderr:".to_string());
                lines.push(run.stderr.trim_end().to_string());
            }
        } else {
            lines.push(format!("case {}: FAIL", index + 1));
            if run.timed_out {
                lines.push("timeout: 5s".to_string());
            }
            lines.push(format!("input: {:?}", case.input));
            lines.push(format!("expected: {:?}", expected));
            lines.push(format!("got: {:?}", got));
            lines.push("stdout:".to_string());
            lines.push(if run.stdout.trim_end().is_empty() {
                "<empty>".to_string()
            } else {
                run.stdout.trim_end().to_string()
            });
            if !run.stderr.trim().is_empty() {
                lines.push("stderr:".to_string());
                lines.push(run.stderr.trim_end().to_string());
            }
            break;
        }
    }

    JudgeResult {
        passed: passed == problem.cases.len(),
        passed_cases: passed,
        total_cases: problem.cases.len(),
        output: lines.join("\n"),
    }
}

fn command_for(root: &Path, path: &Path, language: &str) -> Result<Option<CommandSpec>> {
    match language {
        "python" => Ok(which("python3")
            .or_else(|| which("python"))
            .map(|program| CommandSpec {
                program,
                args: vec![path.display().to_string()],
            })),
        "ts" => Ok(which("node").map(|program| CommandSpec {
            program,
            args: vec![
                "--experimental-strip-types".to_string(),
                path.display().to_string(),
            ],
        })),
        "java" => compile_java(root, path),
        "rust" => compile_rust(root, path),
        _ => Ok(None),
    }
}

fn compile_java(root: &Path, path: &Path) -> Result<Option<CommandSpec>> {
    let Some(javac) = which("javac") else {
        return Ok(None);
    };
    let Some(java) = which("java") else {
        return Ok(None);
    };
    let build = root
        .join(".codex/build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default())
        .join("java");
    fs::create_dir_all(&build)?;
    let mut compile = Command::new(javac);
    compile
        .args([
            "-d",
            &build.display().to_string(),
            &path.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(anyhow!(output.stderr.trim().to_string()));
    }
    Ok(Some(CommandSpec {
        program: java,
        args: vec![
            "-cp".to_string(),
            build.display().to_string(),
            "Solution".to_string(),
        ],
    }))
}

fn compile_rust(root: &Path, path: &Path) -> Result<Option<CommandSpec>> {
    let Some(rustc) = which("rustc") else {
        return Ok(None);
    };
    let build = root
        .join(".codex/build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default());
    fs::create_dir_all(&build)?;
    let exe = build.join(if cfg!(windows) {
        "solution.exe"
    } else {
        "solution"
    });
    let mut compile = Command::new(rustc);
    compile
        .args([
            path.display().to_string(),
            "-o".to_string(),
            exe.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(anyhow!(output.stderr.trim().to_string()));
    }
    Ok(Some(CommandSpec {
        program: exe,
        args: Vec::new(),
    }))
}

fn run_capture(command: &mut Command, input: &str, timeout: Duration) -> Result<RunOutput> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().context("spawn command")?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes()).context("write stdin")?;
    }
    drop(child.stdin.take());

    let timed_out = match child.wait_timeout(timeout).context("wait for command")? {
        Some(_) => false,
        None => {
            let _ = child.kill();
            true
        }
    };
    let output = child.wait_with_output().context("read command output")?;
    Ok(RunOutput {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        timed_out,
    })
}

fn which(name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths).find_map(|dir| {
        let path = dir.join(name);
        if path.is_file() { Some(path) } else { None }
    })
}

fn give_up(root: &Path, problem: &Problem, state: &mut AppState) -> Result<String> {
    let language = normalize_language(&state.settings.language);
    let answer = problem
        .answers
        .get(&language)
        .cloned()
        .unwrap_or_else(|| problem.answers.values().next().cloned().unwrap_or_default());
    mark_history(state, &problem.id, "gave_up");
    upsert_problem_index(root, problem, "gave_up")?;
    save_state(root, state)?;
    Ok(answer)
}

fn next_problem(root: &Path, bank: &[Problem], state: &mut AppState) -> Result<Option<Problem>> {
    let seen = state
        .history
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    let preferred = &state.suggested_next_difficulty;
    let problem = bank
        .iter()
        .find(|item| !seen.contains(&item.id.as_str()) && &item.difficulty == preferred)
        .or_else(|| bank.iter().find(|item| !seen.contains(&item.id.as_str())));
    let Some(problem) = problem.cloned() else {
        return Ok(None);
    };
    state.current_problem = problem.id.clone();
    mark_history(state, &problem.id, "assigned");
    save_state(root, state)?;
    ensure_problem_files(root, &problem)?;
    upsert_problem_index(root, &problem, "assigned")?;
    Ok(Some(problem))
}

fn previous_problem(root: &Path, bank: &[Problem], state: &mut AppState) -> Result<Problem> {
    let known_ids = bank
        .iter()
        .map(|problem| problem.id.as_str())
        .collect::<Vec<_>>();
    let history = state
        .history
        .iter()
        .filter(|item| known_ids.contains(&item.id.as_str()))
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let Some(index) = history.iter().position(|id| id == &state.current_problem) else {
        return problem_by_id(bank, &state.current_problem)
            .cloned()
            .ok_or_else(|| anyhow!("current problem missing"));
    };
    if index == 0 {
        return problem_by_id(bank, &state.current_problem)
            .cloned()
            .ok_or_else(|| anyhow!("current problem missing"));
    }
    state.current_problem = history[index - 1].clone();
    save_state(root, state)?;
    problem_by_id(bank, &state.current_problem)
        .cloned()
        .ok_or_else(|| anyhow!("current problem missing"))
}

fn record_pass(root: &Path, problem: &Problem, state: &mut AppState) -> Result<()> {
    if !state.solved.contains(&problem.id) {
        state.solved.push(problem.id.clone());
    }
    mark_history(state, &problem.id, "solved");
    upsert_problem_index(root, problem, "solved")?;
    state.suggested_next_difficulty = if state.solved.len() >= 2 {
        "medium".to_string()
    } else {
        "easy".to_string()
    };
    save_state(root, state)
}

fn mark_history(state: &mut AppState, problem_id: &str, status: &str) {
    if let Some(item) = state.history.iter_mut().find(|item| item.id == problem_id) {
        item.status = status.to_string();
    } else {
        state.history.push(HistoryItem {
            id: problem_id.to_string(),
            status: status.to_string(),
        });
    }
}

fn ensure_problem_files(root: &Path, problem: &Problem) -> Result<()> {
    let problem_dir = root.join("problems").join(&problem.id);
    fs::create_dir_all(&problem_dir)?;
    let readme = problem_dir.join("README.md");
    if readme.exists() {
        return Ok(());
    }
    let examples = problem
        .examples
        .iter()
        .map(|case| format!("input:\n{}output:\n{}", case.input, case.output))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        readme,
        format!(
            "# {}. {}\n\n난이도: {}\n\n{}\n\n## 입력\n\n{}\n\n## 출력\n\n{}\n\n## 예시\n\n```text\n{}\n```\n",
            problem.id,
            localized(&problem.title, "ko"),
            problem.difficulty,
            localized(&problem.statement, "ko"),
            localized(&problem.input, "ko"),
            localized(&problem.output, "ko"),
            examples
        ),
    )?;
    Ok(())
}

fn upsert_problem_index(root: &Path, problem: &Problem, status: &str) -> Result<()> {
    let index = root.join("problems/INDEX.md");
    if let Some(parent) = index.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut rows: HashMap<String, (String, String, String, String)> = HashMap::new();
    if index.exists() {
        for line in fs::read_to_string(&index)?.lines() {
            let parts = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if parts.len() == 5 && parts[0].chars().all(|c| c.is_ascii_digit()) {
                rows.insert(
                    parts[0].to_string(),
                    (
                        parts[1].to_string(),
                        parts[2].to_string(),
                        parts[3].to_string(),
                        parts[4].to_string(),
                    ),
                );
            }
        }
    }
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    rows.insert(
        number.to_string(),
        (
            problem.slug.clone(),
            problem.difficulty.clone(),
            problem.topics.join(", "),
            status.to_string(),
        ),
    );
    let mut numbers = rows.keys().cloned().collect::<Vec<_>>();
    numbers.sort();
    let body = numbers
        .into_iter()
        .filter_map(|number| {
            rows.get(&number)
                .map(|(slug, difficulty, topics, row_status)| {
                    format!("| {number} | {slug} | {difficulty} | {topics} | {row_status} |")
                })
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        index,
        format!(
            "# Problem Index\n\n| # | Slug | Difficulty | Topics | Status |\n|---|------|------------|--------|--------|\n{body}\n"
        ),
    )?;
    Ok(())
}

fn run_codex_prompt(root: &Path, problem: &Problem, settings: &Settings, prompt: &str) -> String {
    let solution = match ensure_submission(root, problem, settings) {
        Ok(path) => path,
        Err(error) => return format!("Codex prompt failed\n{error}"),
    };
    let code = fs::read_to_string(&solution).unwrap_or_default();
    let relative = solution
        .strip_prefix(root)
        .unwrap_or(&solution)
        .display()
        .to_string();
    let full_prompt = format!(
        "You are a concise coding-test coach. Help with the current problem and current submission. Prefer hints over full answers unless the user explicitly asks for the answer.\n\nUser request:\n{prompt}\n\nProblem:\n{}\n\nCurrent {} submission ({relative}):\n```{}\n{code}\n```",
        render_problem(problem, &settings.ui_language),
        settings.language,
        normalize_language(&settings.language)
    );
    let output_path = unique_temp_path("codecode-last-message", "txt");
    let mut command = Command::new("codex");
    command
        .args([
            "exec",
            "--cd",
            &root.display().to_string(),
            "--sandbox",
            "read-only",
            "-o",
            &output_path.display().to_string(),
            &full_prompt,
        ])
        .current_dir(root);
    let result = run_capture(&mut command, "", Duration::from_secs(600));
    let last_message = fs::read_to_string(&output_path).unwrap_or_default();
    let _ = fs::remove_file(&output_path);
    match result {
        Ok(run) if run.code == Some(0) => {
            let output = [run.stdout.trim(), run.stderr.trim()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            let last = last_message.trim();
            if !last.is_empty() {
                last.to_string()
            } else if !output.is_empty() {
                output
            } else {
                "Codex returned no output.".to_string()
            }
        }
        Ok(run) => {
            let output = [run.stdout.trim(), run.stderr.trim()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            format!("Codex prompt failed ({})\n{output}", run.code.unwrap_or(-1))
        }
        Err(error) => format!("Codex prompt failed\n{error}"),
    }
}

fn run_codex_next(root: &Path, state: &AppState, force: bool, request: &str) -> String {
    if state.settings.next_source != "codex" && !force {
        return "Codex next is disabled; using local problem bank.".to_string();
    }
    let command = if state.settings.codex_next_command.trim().is_empty() {
        default_codex_next_command(root, request)
    } else {
        state.settings.codex_next_command.clone()
    };
    let mut process = if cfg!(windows) {
        let mut command_process = Command::new("cmd");
        command_process.args(["/C", &command]);
        command_process
    } else {
        let mut command_process = Command::new("sh");
        command_process.args(["-c", &command]);
        command_process
    };
    process
        .current_dir(root)
        .env("CODECODE_NEXT_REQUEST", request);
    match run_capture(&mut process, "", Duration::from_secs(600)) {
        Ok(run) if run.code == Some(0) => {
            let output = [run.stdout.trim(), run.stderr.trim()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            format!("Codex command finished\n{output}")
                .trim()
                .to_string()
        }
        Ok(run) => {
            let output = [run.stdout.trim(), run.stderr.trim()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "Codex command failed ({})\n{output}",
                run.code.unwrap_or(-1)
            )
        }
        Err(error) => format!("Codex command failed\n{error}"),
    }
}

fn default_codex_next_command(root: &Path, request: &str) -> String {
    let prompt = format!(
        "Read AGENTS.md, docs/problem-authoring-notes.md if present, .codecode/problem_notes.md if present, problems/INDEX.md if present, .codecode/problem_bank.json if present, and .codex/problem-state.json. The app has a built-in starter problem 001-hello-world, so do not duplicate it. Create exactly one new non-duplicate coding practice problem. User request for this problem: {}. Update .codecode/problem_bank.json, the local problem files, the index, and state files. Do not include the answer in the problem statement.",
        if request.is_empty() {
            "(none)"
        } else {
            request
        }
    );
    format!(
        "codex app-server daemon start >/dev/null 2>&1; codex exec --cd {} --sandbox workspace-write {}",
        sh_quote(&root.display().to_string()),
        sh_quote(&prompt)
    )
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn unique_temp_path(prefix: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{nanos}.{ext}", std::process::id()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Code,
    Command,
    Output,
    None,
}

struct CodeCodeApp {
    root: PathBuf,
    bank: Vec<Problem>,
    state: AppState,
    problem: Problem,
    editor: TextEditor,
    command: String,
    command_cursor: usize,
    output: String,
    output_is_markdown: bool,
    show_output: bool,
    focus: Focus,
    list_cursor: Option<usize>,
    busy_label: String,
    busy_body: String,
    busy_frame: usize,
    task_rx: Option<Receiver<TaskResult>>,
    should_quit: bool,
}

enum TaskResult {
    CodexPrompt(String),
    Next {
        output: String,
        old_problem: String,
        force: bool,
    },
}

impl CodeCodeApp {
    fn new(root: PathBuf) -> Result<Self> {
        let bank = load_bank(&root)?;
        let state = load_state(&root, &bank)?;
        let problem = problem_by_id(&bank, &state.current_problem)
            .cloned()
            .unwrap_or_else(|| bank[0].clone());
        let mut app = Self {
            root,
            bank,
            state,
            problem,
            editor: TextEditor::default(),
            command: String::new(),
            command_cursor: 0,
            output: String::new(),
            output_is_markdown: false,
            show_output: false,
            focus: Focus::Code,
            list_cursor: None,
            busy_label: String::new(),
            busy_body: String::new(),
            busy_frame: 0,
            task_rx: None,
            should_quit: false,
        };
        app.load_code_editor()?;
        Ok(app)
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.check_task();
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                self.handle_key(key)?;
            }
            if !self.busy_label.is_empty() {
                self.busy_frame = (self.busy_frame + 1) % 4;
            }
        }
        self.save_code().ok();
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(size);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(vertical[0]);

        let problem = Paragraph::new(render_markdown_plain(&render_problem(
            &self.problem,
            &self.state.settings.ui_language,
        )))
        .block(Self::block("Problem", self.state.settings.theme == "light"))
        .wrap(Wrap { trim: false });
        frame.render_widget(problem, body[0]);

        if self.show_output {
            let text = if !self.busy_label.is_empty() {
                format!("{}{}", self.busy_body, ".".repeat(self.busy_frame))
            } else if self.output_is_markdown {
                render_markdown_plain(&self.output)
            } else {
                self.output.clone()
            };
            let output = Paragraph::new(text)
                .block(Self::block("Output", self.state.settings.theme == "light"))
                .wrap(Wrap { trim: false });
            frame.render_widget(output, body[1]);
        } else {
            let code = self
                .editor
                .visible_text(body[1].height.saturating_sub(2) as usize);
            let title = format!("solution.{}", ext_for(&self.state.settings.language));
            let code = Paragraph::new(code)
                .block(Self::block(&title, self.state.settings.theme == "light"));
            frame.render_widget(code, body[1]);
        }

        let status =
            Paragraph::new(self.status_text()).style(if self.state.settings.theme == "light" {
                Style::default()
                    .fg(Color::Blue)
                    .bg(Color::Rgb(219, 234, 254))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Rgb(200, 211, 245))
                    .bg(Color::Rgb(21, 32, 51))
                    .add_modifier(Modifier::BOLD)
            });
        frame.render_widget(status, vertical[1]);

        let command_text = if self.focus == Focus::Command || !self.command.is_empty() {
            self.command.clone()
        } else {
            "/run, /next easy string problem, /codex hint, /help".to_string()
        };
        let command = Paragraph::new(command_text)
            .block(Self::block("Command", self.state.settings.theme == "light"))
            .wrap(Wrap { trim: false });
        frame.render_widget(command, vertical[2]);
        self.set_terminal_cursor(frame, body[1], vertical[2]);
    }

    fn block(title: &str, light: bool) -> Block<'_> {
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(if light {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Cyan)
            })
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.focus {
            Focus::Command => self.handle_command_key(key),
            Focus::Code => self.handle_code_key(key),
            _ => self.handle_global_key(key),
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command.clear();
                self.command_cursor = 0;
                self.focus = Focus::None;
            }
            KeyCode::Enter => {
                let value = self.command.trim().to_string();
                self.command.clear();
                self.command_cursor = 0;
                self.focus = Focus::None;
                self.submit_command(&value)?;
            }
            KeyCode::Backspace => {
                self.delete_command_before_cursor();
            }
            KeyCode::Delete => {
                self.delete_command_at_cursor();
            }
            KeyCode::Left => {
                self.command_cursor = self.command_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.command_cursor = (self.command_cursor + 1).min(char_len(&self.command));
            }
            KeyCode::Home => {
                self.command_cursor = 0;
            }
            KeyCode::End => {
                self.command_cursor = char_len(&self.command);
            }
            KeyCode::Char('?') if self.command.trim().is_empty() || self.command.trim() == "/" => {
                self.command.clear();
                self.command_cursor = 0;
                self.focus = Focus::None;
                self.handle_command("help")?;
            }
            KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_command_char(char);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_code_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.focus = Focus::None,
            KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.insert_char(char);
                self.save_code()?;
            }
            KeyCode::Enter => {
                self.editor.insert_newline();
                self.save_code()?;
            }
            KeyCode::Backspace => {
                self.editor.backspace();
                self.save_code()?;
            }
            KeyCode::Delete => {
                self.editor.delete();
                self.save_code()?;
            }
            KeyCode::Tab => {
                for _ in 0..4 {
                    self.editor.insert_char(' ');
                }
                self.save_code()?;
            }
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            _ => {}
        }
        Ok(())
    }

    fn handle_global_key(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(cursor) = self.list_cursor {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.move_list_cursor(-1),
                KeyCode::Down | KeyCode::Char('j') => self.move_list_cursor(1),
                KeyCode::Enter => self.open_selected_problem()?,
                KeyCode::Esc => {
                    self.list_cursor = None;
                    self.write_text_output("Closed list.");
                }
                _ => {
                    self.list_cursor = Some(cursor);
                    self.handle_global_shortcut(key)?;
                }
            }
            return Ok(());
        }
        if key.code == KeyCode::Esc && self.show_output {
            self.show_output = false;
            self.focus = Focus::Code;
            return Ok(());
        }
        self.handle_global_shortcut(key)
    }

    fn handle_global_shortcut(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('/') => self.focus_command(),
            KeyCode::Char('?') => self.handle_command("help")?,
            KeyCode::Char('r') => self.action_run()?,
            KeyCode::Char('n') => self.action_next("")?,
            KeyCode::Char('p') => self.action_previous()?,
            KeyCode::Char('g') => self.action_give_up()?,
            KeyCode::Char('e') => self.action_edit()?,
            KeyCode::Char('l') => self.action_cycle_language()?,
            KeyCode::Char('u') => self.action_toggle_ui_language()?,
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
        Ok(())
    }

    fn focus_command(&mut self) {
        if self.command.is_empty() {
            self.command.push('/');
            self.command_cursor = 1;
        }
        self.focus = Focus::Command;
    }

    fn submit_command(&mut self, value: &str) -> Result<()> {
        let value = value
            .trim()
            .strip_prefix('/')
            .unwrap_or(value.trim())
            .trim();
        self.handle_command(value)
    }

    fn handle_command(&mut self, value: &str) -> Result<()> {
        if value.is_empty() || matches!(value, "help" | "h" | "?") {
            self.list_cursor = None;
            self.write_output(HELP);
            return Ok(());
        }
        if value.starts_with("vim") {
            self.list_cursor = None;
            self.write_text_output("The code editor is already open on the right.");
            return Ok(());
        }
        let (command, arg) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
        let arg = arg.trim();
        if command != "list" {
            self.list_cursor = None;
        }
        match command {
            "run" | "r" => self.action_run()?,
            "edit" | "e" => self.action_edit()?,
            "next" | "n" => self.action_next(arg)?,
            "prev" | "previous" | "p" => self.action_previous()?,
            "giveup" | "give" | "g" => self.action_give_up()?,
            "list" => self.start_problem_list(),
            "open" | "o" if !arg.is_empty() => self.open_problem(arg)?,
            "lang" if arg.is_empty() => self.action_cycle_language()?,
            "lang" if LANGUAGES.contains(&arg) => self.set_language(arg)?,
            "ui" if arg.is_empty() => self.action_toggle_ui_language()?,
            "ui" if UI_LANGUAGES.contains(&arg) => self.set_ui_language(arg)?,
            "theme" if arg.is_empty() => self.action_toggle_theme()?,
            "theme" if THEMES.contains(&arg) => self.set_theme(arg)?,
            "source" | "next-source" if matches!(arg, "bank" | "codex") => {
                self.state.settings.next_source = arg.to_string();
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!("Next source: {arg}"));
            }
            "next-command" if !arg.is_empty() => {
                self.state.settings.codex_next_command = arg.to_string();
                self.state.settings.next_source = "codex".to_string();
                save_state(&self.root, &self.state)?;
                self.write_text_output("Codex next command saved.");
            }
            "codex" if !arg.is_empty() => self.start_codex_prompt(arg)?,
            "exit" | "quit" | "q" => self.should_quit = true,
            _ => self.write_text_output(&format!("Unknown command: {value}\nTry /help.")),
        }
        Ok(())
    }

    fn action_edit(&mut self) -> Result<()> {
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn action_run(&mut self) -> Result<()> {
        self.save_code()?;
        let result = judge(&self.root, &self.problem, &self.state.settings);
        if result.passed {
            record_pass(&self.root, &self.problem, &mut self.state)?;
        }
        let headline = format!(
            "{} {}/{}",
            if result.passed { "PASS" } else { "FAIL" },
            result.passed_cases,
            result.total_cases
        );
        let next_step = if result.passed {
            "Next: /next"
        } else {
            "Fix code, then /run"
        };
        self.write_text_output(&format!("{headline}\n{}\n\n{next_step}", result.output));
        Ok(())
    }

    fn action_next(&mut self, request: &str) -> Result<()> {
        let request = request.trim();
        let old_problem = self.state.current_problem.clone();
        if !request.is_empty() {
            self.start_next_problem(old_problem, true, request.to_string());
            return Ok(());
        }
        if self.state.settings.next_source == "codex" {
            self.start_next_problem(old_problem, false, String::new());
            return Ok(());
        }
        if let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)? {
            self.problem = problem;
            self.load_code_editor()?;
            self.show_output = false;
            self.focus = Focus::Code;
            return Ok(());
        }
        self.start_next_problem(old_problem, true, String::new());
        Ok(())
    }

    fn start_next_problem(&mut self, old_problem: String, force: bool, request: String) {
        if self.task_rx.is_some() {
            self.write_text_output("Already busy.");
            return;
        }
        self.start_busy("next", "Generating next problem");
        let root = self.root.clone();
        let state = self.state.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_codex_next(&root, &state, force, &request);
            let _ = tx.send(TaskResult::Next {
                output,
                old_problem,
                force,
            });
        });
        self.task_rx = Some(rx);
    }

    fn finish_next_problem(
        &mut self,
        output: String,
        old_problem: String,
        force: bool,
    ) -> Result<()> {
        if self.state.settings.next_source == "codex" || force {
            self.bank = load_bank(&self.root)?;
            self.state = load_state(&self.root, &self.bank)?;
        }
        self.problem = problem_by_id(&self.bank, &self.state.current_problem)
            .cloned()
            .unwrap_or_else(|| self.bank[0].clone());
        if self.state.current_problem == old_problem {
            if let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)? {
                self.problem = problem;
            } else {
                self.write_text_output(&format!(
                    "{}{}No next problem is available yet.",
                    if output.is_empty() { "" } else { &output },
                    if output.is_empty() { "" } else { "\n\n" }
                ));
                return Ok(());
            }
        }
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn action_previous(&mut self) -> Result<()> {
        let old_problem = self.state.current_problem.clone();
        self.problem = previous_problem(&self.root, &self.bank, &mut self.state)?;
        if self.state.current_problem == old_problem {
            self.write_text_output("Already at the first known problem.");
        } else {
            self.load_code_editor()?;
            self.show_output = false;
            self.focus = Focus::Code;
        }
        Ok(())
    }

    fn action_give_up(&mut self) -> Result<()> {
        let answer = give_up(&self.root, &self.problem, &mut self.state)?;
        let language = normalize_language(&self.state.settings.language);
        self.write_output(&format!(
            "Answer for {language}:\n\n```{language}\n{}\n```",
            answer.trim_end()
        ));
        Ok(())
    }

    fn action_cycle_language(&mut self) -> Result<()> {
        let current = LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.language)
            .unwrap_or(0);
        self.set_language(LANGUAGES[(current + 1) % LANGUAGES.len()])
    }

    fn action_toggle_ui_language(&mut self) -> Result<()> {
        let current = UI_LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.ui_language)
            .unwrap_or(0);
        self.set_ui_language(UI_LANGUAGES[(current + 1) % UI_LANGUAGES.len()])
    }

    fn action_toggle_theme(&mut self) -> Result<()> {
        let current = THEMES
            .iter()
            .position(|theme| theme == &self.state.settings.theme)
            .unwrap_or(0);
        self.set_theme(THEMES[(current + 1) % THEMES.len()])
    }

    fn set_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.language = language.to_string();
        save_state(&self.root, &self.state)?;
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn set_ui_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.ui_language = language.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("UI language: {language}"));
        Ok(())
    }

    fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.state.settings.theme = theme.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("Theme: {theme}"));
        Ok(())
    }

    fn start_codex_prompt(&mut self, prompt: &str) -> Result<()> {
        if self.task_rx.is_some() {
            self.write_text_output("Already busy.");
            return Ok(());
        }
        self.save_code()?;
        self.start_busy("codex", "Codex is thinking");
        let root = self.root.clone();
        let problem = self.problem.clone();
        let settings = self.state.settings.clone();
        let prompt = prompt.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_codex_prompt(&root, &problem, &settings, &prompt);
            let _ = tx.send(TaskResult::CodexPrompt(output));
        });
        self.task_rx = Some(rx);
        Ok(())
    }

    fn check_task(&mut self) {
        let task = self.task_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(task) = task {
            self.task_rx = None;
            self.stop_busy();
            match task {
                TaskResult::CodexPrompt(output) => self.write_output(&output),
                TaskResult::Next {
                    output,
                    old_problem,
                    force,
                } => {
                    if let Err(error) = self.finish_next_problem(output, old_problem, force) {
                        self.write_text_output(&format!("Next failed\n{error}"));
                    }
                }
            }
        }
    }

    fn start_busy(&mut self, label: &str, body: &str) {
        self.busy_label = label.to_string();
        self.busy_body = body.to_string();
        self.busy_frame = 0;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn stop_busy(&mut self) {
        self.busy_label.clear();
        self.busy_body.clear();
        self.busy_frame = 0;
    }

    fn write_output(&mut self, output: &str) {
        self.output = output.to_string();
        self.output_is_markdown = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn write_text_output(&mut self, output: &str) {
        self.output = output.trim_end().to_string();
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn insert_command_char(&mut self, char: char) {
        let byte = byte_index(&self.command, self.command_cursor);
        self.command.insert(byte, char);
        self.command_cursor += 1;
        self.normalize_command_input();
    }

    fn delete_command_before_cursor(&mut self) {
        if self.command_cursor == 0 {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor - 1);
        let end = byte_index(&self.command, self.command_cursor);
        self.command.replace_range(start..end, "");
        self.command_cursor -= 1;
        self.normalize_command_input();
    }

    fn delete_command_at_cursor(&mut self) {
        if self.command_cursor >= char_len(&self.command) {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor);
        let end = byte_index(&self.command, self.command_cursor + 1);
        self.command.replace_range(start..end, "");
        self.normalize_command_input();
    }

    fn normalize_command_input(&mut self) {
        let normalized = compose_hangul_jamo(&self.command);
        if normalized == self.command {
            self.command_cursor = self.command_cursor.min(char_len(&self.command));
            return;
        }
        let prefix = command_prefix(&self.command, self.command_cursor);
        self.command = normalized;
        self.command_cursor = char_len(&compose_hangul_jamo(&prefix)).min(char_len(&self.command));
    }

    fn set_terminal_cursor(&self, frame: &mut Frame, code_area: Rect, command_area: Rect) {
        match self.focus {
            Focus::Command => {
                let prefix = command_prefix(&self.command, self.command_cursor);
                let x = command_area
                    .x
                    .saturating_add(1)
                    .saturating_add(display_width(&prefix) as u16)
                    .min(command_area.right().saturating_sub(2));
                frame.set_cursor_position(Position::new(x, command_area.y.saturating_add(1)));
            }
            Focus::Code if !self.show_output => {
                if let Some(position) = self.editor.cursor_position(code_area) {
                    frame.set_cursor_position(position);
                }
            }
            _ => {}
        }
    }

    fn load_code_editor(&mut self) -> Result<()> {
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        let text = fs::read_to_string(path).unwrap_or_default();
        self.editor.set_text(&text);
        Ok(())
    }

    fn save_code(&self) -> Result<()> {
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        fs::write(path, self.editor.text())?;
        Ok(())
    }

    fn start_problem_list(&mut self) {
        self.list_cursor = Some(self.current_problem_index());
        self.write_text_output(&self.render_problem_list());
    }

    fn render_problem_list(&self) -> String {
        let status_by_id = self
            .state
            .history
            .iter()
            .map(|item| (item.id.as_str(), item.status.as_str()))
            .collect::<HashMap<_, _>>();
        let cursor = self
            .list_cursor
            .unwrap_or_else(|| self.current_problem_index());
        let mut lines = vec![
            "Problems".to_string(),
            String::new(),
            "    # ID                 Difficulty  Status      Code      Title".to_string(),
        ];
        for (index, problem) in self.bank.iter().enumerate() {
            let marker = if index == cursor { ">" } else { " " };
            let current = if problem.id == self.problem.id {
                "*"
            } else {
                " "
            };
            let title = localized(&problem.title, &self.state.settings.ui_language);
            let code_status = self.submission_status(problem).0;
            lines.push(format!(
                "{marker} {current} {:>2} {:<18} {:<10} {:<10} {:<9} {title}",
                index + 1,
                problem.id,
                problem.difficulty,
                status_by_id
                    .get(problem.id.as_str())
                    .copied()
                    .unwrap_or("-"),
                code_status,
            ));
        }
        lines.push("\nup/down or j/k select | enter open | esc close".to_string());
        lines.join("\n")
    }

    fn current_problem_index(&self) -> usize {
        self.bank
            .iter()
            .position(|problem| problem.id == self.problem.id)
            .unwrap_or(0)
    }

    fn move_list_cursor(&mut self, delta: isize) {
        if self.bank.is_empty() {
            return;
        }
        let cursor = self
            .list_cursor
            .unwrap_or_else(|| self.current_problem_index()) as isize;
        let len = self.bank.len() as isize;
        self.list_cursor = Some(((cursor + delta).rem_euclid(len)) as usize);
        self.write_text_output(&self.render_problem_list());
    }

    fn open_selected_problem(&mut self) -> Result<()> {
        if let Some(cursor) = self.list_cursor {
            let problem_id = self.bank[cursor].id.clone();
            self.list_cursor = None;
            self.open_problem(&problem_id)?;
        }
        Ok(())
    }

    fn open_problem(&mut self, query: &str) -> Result<()> {
        self.list_cursor = None;
        let Some(problem) = self.find_problem(query).cloned() else {
            self.write_text_output(&format!("Problem not found: {query}\nTry /list."));
            return Ok(());
        };
        self.problem = problem;
        self.state.current_problem = self.problem.id.clone();
        if !self
            .state
            .history
            .iter()
            .any(|item| item.id == self.problem.id)
        {
            self.state.history.push(HistoryItem {
                id: self.problem.id.clone(),
                status: "assigned".to_string(),
            });
        }
        save_state(&self.root, &self.state)?;
        ensure_problem_files(&self.root, &self.problem)?;
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn find_problem(&self, query: &str) -> Option<&Problem> {
        let needle = if query.trim().chars().all(|c| c.is_ascii_digit()) {
            format!("{:03}", query.trim().parse::<usize>().ok()?)
        } else {
            query.trim().to_lowercase()
        };
        self.bank.iter().find(|problem| {
            needle == problem.id.to_lowercase()
                || needle == problem.slug.to_lowercase()
                || problem.id.starts_with(&needle)
        })
    }

    fn problem_status(&self, problem: &Problem) -> String {
        if self.state.solved.contains(&problem.id) {
            return "solved".to_string();
        }
        self.state
            .history
            .iter()
            .rev()
            .find(|item| item.id == problem.id)
            .map(|item| item.status.clone())
            .unwrap_or_else(|| "not_started".to_string())
    }

    fn submission_status(&self, problem: &Problem) -> (String, String) {
        let language = normalize_language(&self.state.settings.language);
        let path = self
            .root
            .join("submissions")
            .join(&problem.id)
            .join(format!("solution.{}", ext_for(&language)));
        if !path.exists() {
            return ("missing".to_string(), format!("({language})"));
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        let relative = path.strip_prefix(&self.root).unwrap_or(&path).display();
        if content == template_for(&language) {
            ("template".to_string(), format!("({relative})"))
        } else if content.trim().is_empty() {
            ("empty".to_string(), format!("({relative})"))
        } else {
            ("written".to_string(), format!("({relative})"))
        }
    }

    fn status_text(&self) -> String {
        let code_status = self.submission_status(&self.problem).0;
        format!(
            " CODECODE | {} | {} | {} | {} | code:{} | {} | next:{} | {} ",
            self.problem.id,
            self.problem.difficulty,
            self.busy_status(),
            self.problem_status(&self.problem),
            code_status,
            self.state.settings.language,
            self.state.settings.next_source,
            self.mode_hint(),
        )
    }

    fn busy_status(&self) -> String {
        if self.busy_label.is_empty() {
            "idle".to_string()
        } else {
            format!("busy:{}{}", self.busy_label, ".".repeat(self.busy_frame))
        }
    }

    fn mode_hint(&self) -> &'static str {
        match (self.focus, self.list_cursor.is_some(), self.show_output) {
            (Focus::Command, _, _) => "Enter submit | Esc cancel",
            (_, true, _) => "up/down move | Enter open | Esc close",
            (_, _, true) => "Esc code | / command | ? help",
            (Focus::Code, _, _) => "Esc then / command",
            _ => "/ command | ? help",
        }
    }
}

#[derive(Clone, Debug)]
struct TextEditor {
    lines: Vec<String>,
    row: usize,
    col: usize,
    scroll: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            row: 0,
            col: 0,
            scroll: 0,
        }
    }
}

impl TextEditor {
    fn set_text(&mut self, text: &str) {
        self.lines = text.split('\n').map(str::to_string).collect();
        if text.ends_with('\n') {
            self.lines.pop();
            self.lines.push(String::new());
        }
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = 0;
        self.col = 0;
        self.scroll = 0;
    }

    fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn visible_text(&mut self, height: usize) -> String {
        if self.row < self.scroll {
            self.scroll = self.row;
        } else if height > 0 && self.row >= self.scroll + height {
            self.scroll = self.row + 1 - height;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        self.lines
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height.max(1))
            .map(|(index, line)| {
                let cursor = if index == self.row { ">" } else { " " };
                format!("{cursor}{:>width$} {line}", index + 1, width = line_width)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cursor_position(&self, area: Rect) -> Option<Position> {
        if self.row < self.scroll {
            return None;
        }
        let visible_row = self.row - self.scroll;
        let inner_height = area.height.saturating_sub(2) as usize;
        if visible_row >= inner_height {
            return None;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        let prefix_width = 1 + line_width + 1;
        let line = self.lines.get(self.row)?;
        let text_before_cursor = command_prefix(line, self.col);
        let x = area
            .x
            .saturating_add(1)
            .saturating_add((prefix_width + display_width(&text_before_cursor)) as u16)
            .min(area.right().saturating_sub(2));
        let y = area.y.saturating_add(1).saturating_add(visible_row as u16);
        Some(Position::new(x, y))
    }

    fn insert_char(&mut self, char: char) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        self.lines[self.row].insert(byte, char);
        self.col += 1;
    }

    fn insert_newline(&mut self) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        let rest = self.lines[self.row].split_off(byte);
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    fn backspace(&mut self) {
        self.ensure_cursor();
        if self.col > 0 {
            let start = byte_index(&self.lines[self.row], self.col - 1);
            let end = byte_index(&self.lines[self.row], self.col);
            self.lines[self.row].replace_range(start..end, "");
            self.col -= 1;
        } else if self.row > 0 {
            let current = self.lines.remove(self.row);
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
            self.lines[self.row].push_str(&current);
        }
    }

    fn delete(&mut self) {
        self.ensure_cursor();
        if self.col < char_len(&self.lines[self.row]) {
            let start = byte_index(&self.lines[self.row], self.col);
            let end = byte_index(&self.lines[self.row], self.col + 1);
            self.lines[self.row].replace_range(start..end, "");
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        }
    }

    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
        }
    }

    fn move_right(&mut self) {
        if self.col < char_len(&self.lines[self.row]) {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    fn ensure_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = self.row.min(self.lines.len() - 1);
        self.col = self.col.min(char_len(&self.lines[self.row]));
    }
}

fn byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

fn char_len(value: &str) -> usize {
    value.chars().count()
}

fn command_prefix(value: &str, char_index: usize) -> String {
    value.chars().take(char_index).collect()
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

const CHO: &[char] = &[
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];
const JUNG: &[char] = &[
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];
const JONG: &[char] = &[
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ',
    'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

fn compose_hangul_jamo(value: &str) -> String {
    let mut out = String::new();
    let mut run = Vec::new();
    for char in decompose_hangul(value).chars() {
        if is_hangul_jamo(char) {
            run.push(char);
        } else {
            if !run.is_empty() {
                out.push_str(&compose_hangul_run(&run));
                run.clear();
            }
            out.push(char);
        }
    }
    if !run.is_empty() {
        out.push_str(&compose_hangul_run(&run));
    }
    out
}

fn decompose_hangul(value: &str) -> String {
    let mut chars = String::new();
    for char in value.chars() {
        let code = char as u32;
        if (0xAC00..=0xD7A3).contains(&code) {
            let offset = code - 0xAC00;
            let lead = (offset / 588) as usize;
            let vowel = ((offset % 588) / 28) as usize;
            let tail = (offset % 28) as usize;
            chars.push(CHO[lead]);
            chars.push(JUNG[vowel]);
            if tail != 0 {
                chars.push(JONG[tail]);
            }
        } else {
            chars.push(char);
        }
    }
    chars
}

fn compose_hangul_run(chars: &[char]) -> String {
    let mut out = String::new();
    let mut lead = None;
    let mut vowel = None;
    let mut tail = None;

    fn emit(
        out: &mut String,
        lead: &mut Option<char>,
        vowel: &mut Option<char>,
        tail: &mut Option<char>,
    ) {
        match (*lead, *vowel) {
            (Some(l), Some(v)) => {
                if let (Some(l_index), Some(v_index)) = (cho_index(l), jung_index(v)) {
                    let code = 0xAC00
                        + ((l_index * 21 + v_index) * 28 + tail.and_then(jong_index).unwrap_or(0))
                            as u32;
                    if let Some(char) = char::from_u32(code) {
                        out.push(char);
                    }
                } else {
                    for part in [*lead, *vowel, *tail].into_iter().flatten() {
                        out.push(part);
                    }
                }
            }
            _ => {
                for part in [*lead, *vowel, *tail].into_iter().flatten() {
                    out.push(part);
                }
            }
        }
        *lead = None;
        *vowel = None;
        *tail = None;
    }

    for &char in chars {
        if jung_index(char).is_some() {
            if lead.is_none() {
                out.push(char);
            } else if vowel.is_none() {
                vowel = Some(char);
            } else if tail.is_none() {
                if let Some(combined) = combine_jung(vowel.unwrap(), char) {
                    vowel = Some(combined);
                } else {
                    emit(&mut out, &mut lead, &mut vowel, &mut tail);
                    out.push(char);
                }
            } else if let Some((first, second)) = split_jong(tail.unwrap()) {
                tail = Some(first);
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(second);
                vowel = Some(char);
            } else {
                let next_lead = tail;
                tail = None;
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = next_lead;
                vowel = Some(char);
            }
        } else if lead.is_none() {
            lead = Some(char);
        } else if vowel.is_none() {
            if let Some(combined) = combine_cho(lead.unwrap(), char) {
                lead = Some(combined);
            } else {
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(char);
            }
        } else if tail.is_none() && jong_index(char).is_some() {
            tail = Some(char);
        } else if tail.is_some() {
            if let Some(combined) = combine_jong(tail.unwrap(), char) {
                tail = Some(combined);
            } else {
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(char);
            }
        } else {
            emit(&mut out, &mut lead, &mut vowel, &mut tail);
            lead = Some(char);
        }
    }
    emit(&mut out, &mut lead, &mut vowel, &mut tail);
    out
}

fn is_hangul_jamo(char: char) -> bool {
    cho_index(char).is_some() || jung_index(char).is_some() || jong_index(char).is_some()
}

fn cho_index(char: char) -> Option<usize> {
    CHO.iter().position(|value| *value == char)
}

fn jung_index(char: char) -> Option<usize> {
    JUNG.iter().position(|value| *value == char)
}

fn jong_index(char: char) -> Option<usize> {
    JONG.iter()
        .position(|value| *value == char)
        .filter(|index| *index != 0)
}

fn combine_cho(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㄱ', 'ㄱ') => Some('ㄲ'),
        ('ㄷ', 'ㄷ') => Some('ㄸ'),
        ('ㅂ', 'ㅂ') => Some('ㅃ'),
        ('ㅅ', 'ㅅ') => Some('ㅆ'),
        ('ㅈ', 'ㅈ') => Some('ㅉ'),
        _ => None,
    }
}

fn combine_jung(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

fn combine_jong(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㄱ', 'ㅅ') => Some('ㄳ'),
        ('ㄴ', 'ㅈ') => Some('ㄵ'),
        ('ㄴ', 'ㅎ') => Some('ㄶ'),
        ('ㄹ', 'ㄱ') => Some('ㄺ'),
        ('ㄹ', 'ㅁ') => Some('ㄻ'),
        ('ㄹ', 'ㅂ') => Some('ㄼ'),
        ('ㄹ', 'ㅅ') => Some('ㄽ'),
        ('ㄹ', 'ㅌ') => Some('ㄾ'),
        ('ㄹ', 'ㅍ') => Some('ㄿ'),
        ('ㄹ', 'ㅎ') => Some('ㅀ'),
        ('ㅂ', 'ㅅ') => Some('ㅄ'),
        _ => None,
    }
}

fn split_jong(char: char) -> Option<(char, char)> {
    match char {
        'ㄳ' => Some(('ㄱ', 'ㅅ')),
        'ㄵ' => Some(('ㄴ', 'ㅈ')),
        'ㄶ' => Some(('ㄴ', 'ㅎ')),
        'ㄺ' => Some(('ㄹ', 'ㄱ')),
        'ㄻ' => Some(('ㄹ', 'ㅁ')),
        'ㄼ' => Some(('ㄹ', 'ㅂ')),
        'ㄽ' => Some(('ㄹ', 'ㅅ')),
        'ㄾ' => Some(('ㄹ', 'ㅌ')),
        'ㄿ' => Some(('ㄹ', 'ㅍ')),
        'ㅀ' => Some(('ㄹ', 'ㅎ')),
        'ㅄ' => Some(('ㅂ', 'ㅅ')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("codecode-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn two_problem_bank(root: &Path) -> Vec<Problem> {
        let mut first = starter_problem();
        first.id = "001-hello-world".to_string();
        let mut second = first.clone();
        second.id = "002-echo".to_string();
        second.slug = "echo".to_string();
        second.topics = vec!["io".to_string(), "string".to_string()];
        second.title = map2("ko", "그대로 출력", "en", "Echo");
        second.statement = map2(
            "ko",
            "입력을 그대로 출력하세요.",
            "en",
            "Print stdin unchanged.",
        );
        second.input = map2("ko", "문자열", "en", "A string");
        second.output = map2("ko", "입력과 같은 문자열", "en", "The same string");
        second.examples = vec![IoCase {
            input: "code\n".to_string(),
            output: "code\n".to_string(),
        }];
        second.cases = second.examples.clone();
        second.answers = HashMap::from([
            (
                "python".to_string(),
                "import sys\nprint(sys.stdin.read(), end='')\n".to_string(),
            ),
            (
                "ts".to_string(),
                "const fs = require('fs');\nprocess.stdout.write(fs.readFileSync(0, 'utf8'));\n".to_string(),
            ),
            (
                "java".to_string(),
                "class Solution { public static void main(String[] args) throws Exception { System.out.print(new String(System.in.readAllBytes())); } }\n".to_string(),
            ),
            (
                "rust".to_string(),
                "use std::io::{self, Read};\nfn main() { let mut s = String::new(); io::stdin().read_to_string(&mut s).unwrap(); print!(\"{}\", s); }\n".to_string(),
            ),
        ]);
        let bank = vec![first, second];
        save_bank(root, &bank).unwrap();
        bank
    }

    #[test]
    fn load_state_uses_first_problem_when_state_file_is_missing() {
        let root = tmp_root("state-missing");
        let bank = load_bank(&root).unwrap();
        let state = load_state(&root, &bank).unwrap();
        assert_eq!(state.current_problem, "001-hello-world");
        assert_eq!(state.settings.language, "python");
        assert_eq!(state.settings.ui_language, "ko");
    }

    #[test]
    fn save_bank_creates_local_custom_problem_bank() {
        let root = tmp_root("save-bank");
        let bank = two_problem_bank(&root);
        let loaded = load_bank(&root).unwrap();
        assert!(root.join(".codecode/problem_bank.json").exists());
        assert_eq!(
            loaded.iter().map(|problem| &problem.id).collect::<Vec<_>>(),
            bank.iter().map(|problem| &problem.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ensure_submission_creates_language_template() {
        let root = tmp_root("submission");
        let bank = load_bank(&root).unwrap();
        let settings = Settings {
            language: "rust".to_string(),
            ..Settings::default()
        };
        let path = ensure_submission(&root, &bank[0], &settings).unwrap();
        assert_eq!(path, root.join("submissions/001-hello-world/solution.rs"));
        assert!(fs::read_to_string(path).unwrap().contains("fn main()"));
    }

    #[test]
    fn render_problem_separates_input_output_blocks() {
        let root = tmp_root("render");
        let problem = load_bank(&root).unwrap().remove(0);
        let rendered = render_problem(&problem, "ko");
        assert!(
            rendered.contains("## Input\n\n입력은 없습니다.\n\n## Output\n\n`Hello, World!` 한 줄")
        );
        assert!(rendered.contains("```text\n\n```"));
    }

    #[test]
    fn render_markdown_plain_hides_problem_markdown_syntax() {
        let root = tmp_root("render-plain");
        let problem = load_bank(&root).unwrap().remove(0);
        let rendered = render_markdown_plain(&render_problem(&problem, "ko"));
        assert!(rendered.contains("001. Hello World"));
        assert!(rendered.contains("Input"));
        assert!(rendered.contains("Output"));
        assert!(rendered.contains("Hello, World!"));
        assert!(!rendered.contains("```"));
        assert!(!rendered.contains("##"));
        assert!(!rendered.contains("`Hello, World!`"));
    }

    #[test]
    fn render_markdown_plain_preserves_fenced_code_body() {
        let rendered =
            render_markdown_plain("## Answer\n\n```python\n# keep comment\nprint('x')\n```");
        assert!(rendered.contains("Answer"));
        assert!(rendered.contains("# keep comment"));
        assert!(rendered.contains("print('x')"));
        assert!(!rendered.contains("```"));
    }

    #[test]
    fn judge_runs_python_solution_against_cases() {
        if which("python3").or_else(|| which("python")).is_none() {
            return;
        }
        let root = tmp_root("judge-pass");
        let bank = load_bank(&root).unwrap();
        let settings = Settings::default();
        let path = ensure_submission(&root, &bank[0], &settings).unwrap();
        fs::write(path, "print('Hello, World!')\n").unwrap();
        let result = judge(&root, &bank[0], &settings);
        assert!(result.passed, "{}", result.output);
        assert_eq!(result.passed_cases, result.total_cases);
    }

    #[test]
    fn judge_shows_debug_stdout_on_failure() {
        if which("python3").or_else(|| which("python")).is_none() {
            return;
        }
        let root = tmp_root("judge-fail");
        let bank = load_bank(&root).unwrap();
        let settings = Settings::default();
        let path = ensure_submission(&root, &bank[0], &settings).unwrap();
        fs::write(path, "print('debug')\nprint('Hello, World!')\n").unwrap();
        let result = judge(&root, &bank[0], &settings);
        assert!(!result.passed);
        assert!(result.output.contains("stdout:\ndebug\nHello, World!"));
    }

    #[test]
    fn next_problem_skips_history_and_saves_new_current() {
        let root = tmp_root("next");
        let bank = two_problem_bank(&root);
        let mut state = AppState {
            current_problem: "001-hello-world".to_string(),
            settings: Settings::default(),
            solved: Vec::new(),
            history: vec![HistoryItem {
                id: "001-hello-world".to_string(),
                status: "solved".to_string(),
            }],
            suggested_next_difficulty: "easy".to_string(),
        };
        save_state(&root, &state).unwrap();
        let problem = next_problem(&root, &bank, &mut state).unwrap().unwrap();
        let saved = load_state(&root, &bank).unwrap();
        assert_eq!(problem.id, "002-echo");
        assert_eq!(saved.current_problem, "002-echo");
        assert!(
            fs::read_to_string(root.join("problems/INDEX.md"))
                .unwrap()
                .contains("002 | echo")
        );
    }

    #[test]
    fn record_pass_marks_solved_and_raises_suggested_difficulty_after_two_solves() {
        let root = tmp_root("record-pass");
        let bank = load_bank(&root).unwrap();
        let mut state = AppState {
            current_problem: "001-hello-world".to_string(),
            settings: Settings::default(),
            solved: vec!["000-warmup".to_string()],
            history: Vec::new(),
            suggested_next_difficulty: "easy".to_string(),
        };
        record_pass(&root, &bank[0], &mut state).unwrap();
        let saved = load_state(&root, &bank).unwrap();
        assert!(saved.solved.contains(&"001-hello-world".to_string()));
        assert_eq!(saved.history[0].status, "solved");
        assert_eq!(saved.suggested_next_difficulty, "medium");
    }

    #[test]
    fn default_codex_next_command_reads_notes_and_includes_request() {
        let root = tmp_root("codex-command");
        let command = default_codex_next_command(&root, "그래프 쉬운 문제");
        assert!(command.contains("docs/problem-authoring-notes.md"));
        assert!(command.contains(".codecode/problem_notes.md"));
        assert!(command.contains("그래프 쉬운 문제"));
    }

    #[test]
    #[cfg(unix)]
    fn run_codex_next_exposes_request_to_custom_command() {
        let root = tmp_root("codex-env");
        let state = AppState {
            current_problem: "001-hello-world".to_string(),
            settings: Settings {
                next_source: "codex".to_string(),
                codex_next_command: "printf '%s' \"$CODECODE_NEXT_REQUEST\" > request.txt"
                    .to_string(),
                ..Settings::default()
            },
            solved: Vec::new(),
            history: Vec::new(),
            suggested_next_difficulty: "easy".to_string(),
        };
        let output = run_codex_next(&root, &state, false, "문자열 쉬운 문제");
        assert!(output.contains("finished"));
        assert_eq!(
            fs::read_to_string(root.join("request.txt")).unwrap(),
            "문자열 쉬운 문제"
        );
    }

    #[test]
    fn compose_hangul_jamo_handles_korean_command_text() {
        assert_eq!(
            compose_hangul_jamo("ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"),
            "안녕하세요"
        );
        assert_eq!(
            compose_hangul_jamo("/next ㅎㅐㅅㅣ맵 쉬운 문제"),
            "/next 해시맵 쉬운 문제"
        );
        let mut value = String::new();
        for char in "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ".chars() {
            value.push(char);
            value = compose_hangul_jamo(&value);
        }
        assert_eq!(value, "안녕하세요");
        assert_eq!(compose_hangul_jamo("ㄳㅏ"), "ㄳㅏ");
    }

    #[test]
    fn text_editor_preserves_utf8_while_editing() {
        let mut editor = TextEditor::default();
        editor.set_text("");
        for char in "안녕".chars() {
            editor.insert_char(char);
        }
        editor.insert_newline();
        editor.insert_char('!');
        assert_eq!(editor.text(), "안녕\n!");
        editor.backspace();
        assert_eq!(editor.text(), "안녕\n");
    }

    #[test]
    fn app_command_next_request_starts_forced_codex_task() {
        let root = tmp_root("app-next-request");
        two_problem_bank(&root);
        let mut app = CodeCodeApp::new(root).unwrap();
        app.state.settings.codex_next_command = "true".to_string();
        app.handle_command("next 해시맵 쉬운 문제").unwrap();
        assert!(app.task_rx.is_some());
        assert_eq!(app.busy_label, "next");
    }

    #[test]
    fn command_input_tracks_cursor_after_hangul_composition() {
        let root = tmp_root("command-cursor");
        let mut app = CodeCodeApp::new(root).unwrap();
        app.focus_command();
        for char in "ㅇㅏㄴㄴㅕㅇ".chars() {
            app.insert_command_char(char);
        }
        assert_eq!(app.command, "/안녕");
        assert_eq!(app.command_cursor, 3);

        app.command_cursor = 1;
        app.insert_command_char('x');
        assert_eq!(app.command, "/x안녕");
        assert_eq!(app.command_cursor, 2);
    }

    #[test]
    fn display_width_counts_hangul_as_wide() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("안녕"), 4);
    }

    #[test]
    fn smoke_title_comes_from_current_problem() {
        let root = tmp_root("smoke");
        let bank = load_bank(&root).unwrap();
        let state = load_state(&root, &bank).unwrap();
        let problem = problem_by_id(&bank, &state.current_problem).unwrap();
        assert_eq!(
            localized(&problem.title, &state.settings.ui_language),
            "Hello World"
        );
    }
}
