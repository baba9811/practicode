use crate::process::{CommandSpec, run_capture, which};
use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub const LANGUAGES: &[&str] = &["python", "ts", "java", "rust"];
pub const UI_LANGUAGES: &[&str] = &["ko", "en"];
pub const THEMES: &[&str] = &["dark", "light"];
pub const AI_PROVIDERS: &[&str] = &["codex", "claude"];
pub const BANK_PATH: &str = ".practicode/problem_bank.json";
pub const STATE_PATH: &str = ".practicode/problem-state.json";
pub const PROBLEM_NOTES_PATH: &str = ".practicode/problem_notes.md";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default = "default_next_source")]
    pub next_source: String,
    #[serde(default = "default_ai_provider")]
    pub ai_provider: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ai_next_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: default_language(),
            ui_language: default_ui_language(),
            theme: default_theme(),
            editor: default_editor(),
            next_source: default_next_source(),
            ai_provider: default_ai_provider(),
            ai_model: default_ai_model(),
            ai_next_command: String::new(),
        }
    }
}

impl Settings {
    pub fn next_ai_command(&self) -> &str {
        &self.ai_next_command
    }

    pub fn model_arg(&self) -> Option<&str> {
        let model = self.ai_model.trim();
        if model.is_empty() || model == "auto" {
            None
        } else {
            Some(model)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub current_problem: String,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub solved: Vec<String>,
    #[serde(default)]
    pub history: Vec<HistoryItem>,
    #[serde(default = "default_suggested_difficulty")]
    pub suggested_next_difficulty: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub slug: String,
    pub difficulty: String,
    pub topics: Vec<String>,
    pub title: HashMap<String, String>,
    pub statement: HashMap<String, String>,
    pub input: HashMap<String, String>,
    pub output: HashMap<String, String>,
    pub examples: Vec<IoCase>,
    pub cases: Vec<IoCase>,
    pub answers: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IoCase {
    pub input: String,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct JudgeResult {
    pub passed: bool,
    pub passed_cases: usize,
    pub total_cases: usize,
    pub output: String,
}

pub fn default_language() -> String {
    "python".to_string()
}

pub fn default_ui_language() -> String {
    "ko".to_string()
}

pub fn default_theme() -> String {
    "dark".to_string()
}

pub fn default_editor() -> String {
    "vim".to_string()
}

pub fn default_next_source() -> String {
    "bank".to_string()
}

pub fn default_ai_provider() -> String {
    "codex".to_string()
}

pub fn default_ai_model() -> String {
    "auto".to_string()
}

pub fn default_suggested_difficulty() -> String {
    "easy".to_string()
}

pub fn ext_for(language: &str) -> &'static str {
    match normalize_language(language).as_str() {
        "python" => "py",
        "ts" => "ts",
        "java" => "java",
        "rust" => "rs",
        _ => "py",
    }
}

pub fn starter_problem() -> Problem {
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

pub fn map2(k1: &str, v1: &str, k2: &str, v2: &str) -> HashMap<String, String> {
    HashMap::from([
        (k1.to_string(), v1.to_string()),
        (k2.to_string(), v2.to_string()),
    ])
}

pub fn load_bank(root: &Path) -> Result<Vec<Problem>> {
    let path = root.join(BANK_PATH);
    if path.exists() {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let bank: Vec<Problem> =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        validate_bank(&bank, &path)?;
        Ok(bank)
    } else {
        Ok(vec![starter_problem()])
    }
}

pub fn save_bank(root: &Path, bank: &[Problem]) -> Result<()> {
    let path = root.join(BANK_PATH);
    validate_bank(bank, &path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(bank)? + "\n")?;
    Ok(())
}

fn validate_bank(bank: &[Problem], path: &Path) -> Result<()> {
    if bank.is_empty() {
        bail!("{} must contain at least one problem", path.display());
    }
    for problem in bank {
        if !is_safe_name(&problem.id) {
            bail!("{} has invalid problem id {:?}", path.display(), problem.id);
        }
        if !is_safe_name(&problem.slug) {
            bail!(
                "{} has invalid slug {:?} for {}",
                path.display(),
                problem.slug,
                problem.id
            );
        }
        if problem.cases.is_empty() {
            bail!(
                "{} problem {} has no judge cases",
                path.display(),
                problem.id
            );
        }
        for language in LANGUAGES {
            if !problem.answers.contains_key(*language) {
                bail!(
                    "{} problem {} missing {language} answer",
                    path.display(),
                    problem.id
                );
            }
        }
    }
    Ok(())
}

fn is_safe_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_'))
}

pub fn load_state(root: &Path, bank: &[Problem]) -> Result<AppState> {
    let path = root.join(STATE_PATH);
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
    normalize_settings(&mut state.settings);
    if state.history.is_empty() {
        state.history.push(HistoryItem {
            id: state.current_problem.clone(),
            status: "assigned".to_string(),
        });
    }
    Ok(state)
}

pub fn save_state(root: &Path, state: &AppState) -> Result<()> {
    #[derive(Serialize)]
    struct StateFile<'a> {
        current_problem: &'a str,
        next_number: usize,
        suggested_next_difficulty: &'a str,
        settings: &'a Settings,
        solved: &'a [String],
        history: &'a [HistoryItem],
    }

    let path = root.join(STATE_PATH);
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

pub fn normalize_settings(settings: &mut Settings) {
    settings.language = normalize_language(&settings.language);
    if !UI_LANGUAGES.contains(&settings.ui_language.as_str()) {
        settings.ui_language = "ko".to_string();
    }
    if !THEMES.contains(&settings.theme.as_str()) {
        settings.theme = "dark".to_string();
    }
    settings.next_source = normalize_next_source(&settings.next_source);
    settings.ai_provider = normalize_ai_provider(&settings.ai_provider);
    if settings.ai_model.trim().is_empty() {
        settings.ai_model = default_ai_model();
    }
}

pub fn problem_by_id<'a>(bank: &'a [Problem], problem_id: &str) -> Option<&'a Problem> {
    bank.iter().find(|problem| problem.id == problem_id)
}

pub fn normalize_language(language: &str) -> String {
    if LANGUAGES.contains(&language) {
        language.to_string()
    } else {
        "python".to_string()
    }
}

pub fn normalize_next_source(source: &str) -> String {
    if source == "ai" {
        "ai".to_string()
    } else {
        "bank".to_string()
    }
}

pub fn normalize_ai_provider(provider: &str) -> String {
    if provider == "claude" {
        "claude".to_string()
    } else {
        "codex".to_string()
    }
}

pub fn localized(map: &HashMap<String, String>, lang: &str) -> String {
    map.get(lang)
        .or_else(|| map.get("ko"))
        .or_else(|| map.get("en"))
        .or_else(|| map.values().next())
        .cloned()
        .unwrap_or_default()
}

pub fn template_for(language: &str) -> String {
    match normalize_language(language).as_str() {
        "python" => "# Read from stdin and print to stdout.\nimport sys\n\n\n".to_string(),
        "ts" => "const fs = require('fs');\nconst input = fs.readFileSync(0, 'utf8');\n\n".to_string(),
        "java" => "import java.io.*;\n\nclass Solution {\n    public static void main(String[] args) throws Exception {\n    }\n}\n".to_string(),
        "rust" => "fn main() {\n}\n".to_string(),
        _ => String::new(),
    }
}

pub fn ensure_submission(root: &Path, problem: &Problem, settings: &Settings) -> Result<PathBuf> {
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

pub fn render_problem(problem: &Problem, ui_language: &str) -> String {
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

pub fn fenced_text(value: &str) -> String {
    let mut body = value.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    format!("```text\n{body}```")
}

pub fn judge(root: &Path, problem: &Problem, settings: &Settings) -> JudgeResult {
    if problem.cases.is_empty() {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: 0,
            output: "problem has no judge cases".to_string(),
        };
    }
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
    let run_dir = root.join(".practicode/build").join(&problem.id).join("run");
    if let Err(error) = fs::create_dir_all(&run_dir) {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: problem.cases.len(),
            output: error.to_string(),
        };
    }

    let mut passed = 0;
    let mut lines = Vec::new();
    for (index, case) in problem.cases.iter().enumerate() {
        let mut process = Command::new(&command.program);
        process.args(&command.args).current_dir(&run_dir);
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

pub fn command_for(root: &Path, path: &Path, language: &str) -> Result<Option<CommandSpec>> {
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
        .join(".practicode/build")
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
        .join(".practicode/build")
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

pub fn give_up(root: &Path, problem: &Problem, state: &mut AppState) -> Result<String> {
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

pub fn next_problem(
    root: &Path,
    bank: &[Problem],
    state: &mut AppState,
) -> Result<Option<Problem>> {
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

pub fn previous_problem(root: &Path, bank: &[Problem], state: &mut AppState) -> Result<Problem> {
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

pub fn record_pass(root: &Path, problem: &Problem, state: &mut AppState) -> Result<()> {
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

pub fn mark_history(state: &mut AppState, problem_id: &str, status: &str) {
    if let Some(item) = state.history.iter_mut().find(|item| item.id == problem_id) {
        item.status = status.to_string();
    } else {
        state.history.push(HistoryItem {
            id: problem_id.to_string(),
            status: status.to_string(),
        });
    }
}

pub fn ensure_problem_files(root: &Path, problem: &Problem) -> Result<()> {
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

pub fn upsert_problem_index(root: &Path, problem: &Problem, status: &str) -> Result<()> {
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
