use crate::{
    core::{
        AppState, CLAUDE_AI_EFFORTS, LANGUAGES, PROBLEM_NOTES_PATH, Problem, Settings,
        SyntaxLesson, UI_LANGUAGES, ensure_submission, ensure_syntax_submission,
        normalize_ai_provider, regular_file_exists, render_problem, save_user_text,
        syntax_lesson_study_context, ui_text,
    },
    process::{run_capture, sh_quote, shell_process, unique_temp_path, which},
};
use anyhow::{Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Clone, Debug, Default)]
pub struct ModelCatalog {
    pub models: Vec<String>,
    pub message: Option<String>,
}

const CODEX_MODEL_FALLBACKS: &[&str] =
    &["gpt-5.5", "gpt-5.4", "gpt-5.4-mini", "gpt-5.3-codex-spark"];
const CLAUDE_MODEL_FALLBACKS: &[&str] = &["sonnet", "opus", "fable", "claude-fable-5"];

pub fn build_lesson_ai_prompt(
    lesson: &SyntaxLesson,
    settings: &Settings,
    prompt: &str,
    relative: &str,
    code: &str,
    latest_result: &str,
) -> String {
    let latest_result = if latest_result.trim().is_empty() {
        "(none yet)"
    } else {
        latest_result.trim()
    };
    format!(
        "You are a concise Socratic programming tutor for the current syntax lesson. Answer conceptual questions about the lesson, worked example, and learner's current exercise. Prefer explanation, guiding questions, and tiny examples. Do not give the full exercise solution unless the user explicitly asks for the answer.\n\nUser question:\n{prompt}\n\nCurrent syntax lesson context:\n{}\n\nCurrent {} exercise ({relative}):\n```{}\n{code}\n```\n\nLatest validation result:\n{latest_result}",
        syntax_lesson_study_context(lesson, &settings.ui_language),
        settings.language,
        settings.language
    )
}

pub fn run_ai_prompt(root: &Path, problem: &Problem, settings: &Settings, prompt: &str) -> String {
    let solution = match ensure_submission(root, problem, settings) {
        Ok(path) => path,
        Err(error) => return format!("AI prompt failed\n{error}"),
    };
    let code = match fs::read_to_string(&solution)
        .with_context(|| format!("read {}", solution.display()))
    {
        Ok(code) => code,
        Err(error) => return format!("AI prompt failed\n{error}"),
    };
    let relative = solution
        .strip_prefix(root)
        .unwrap_or(&solution)
        .display()
        .to_string();
    let full_prompt = format!(
        "You are a concise coding-test coach. Help with the current problem and current submission. Prefer hints over full answers unless the user explicitly asks for the answer.\n\nUser request:\n{prompt}\n\nProblem:\n{}\n\nCurrent {} submission ({relative}):\n```{}\n{code}\n```",
        render_problem(problem, &settings.ui_language),
        settings.language,
        settings.language
    );
    match normalize_ai_provider(&settings.ai_provider).as_str() {
        "claude" => run_claude_prompt(root, settings, &full_prompt),
        _ => run_codex_prompt(root, settings, &full_prompt),
    }
}

pub fn run_ai_lesson_prompt(
    root: &Path,
    lesson: &SyntaxLesson,
    settings: &Settings,
    prompt: &str,
    latest_result: &str,
) -> String {
    let exercise = match ensure_syntax_submission(root, lesson) {
        Ok(path) => path,
        Err(error) => return format!("AI prompt failed\n{error}"),
    };
    let code = match fs::read_to_string(&exercise)
        .with_context(|| format!("read {}", exercise.display()))
    {
        Ok(code) => code,
        Err(error) => return format!("AI prompt failed\n{error}"),
    };
    let relative = exercise
        .strip_prefix(root)
        .unwrap_or(&exercise)
        .display()
        .to_string();
    let full_prompt =
        build_lesson_ai_prompt(lesson, settings, prompt, &relative, &code, latest_result);
    match normalize_ai_provider(&settings.ai_provider).as_str() {
        "claude" => run_claude_prompt(root, settings, &full_prompt),
        _ => run_codex_prompt(root, settings, &full_prompt),
    }
}

pub fn run_ai_next(root: &Path, state: &AppState, force: bool, request: &str) -> String {
    if state.settings.next_source != "ai" && !force {
        return "AI next is disabled; using local problems.".to_string();
    }
    let provider = normalize_ai_provider(&state.settings.ai_provider);
    let mut process = if state.settings.next_ai_command().trim().is_empty() {
        builtin_ai_process(
            root,
            &state.settings,
            &default_ai_next_prompt_with_settings(&state.settings, request),
        )
    } else {
        shell_process(state.settings.next_ai_command())
    };
    process
        .current_dir(root)
        .env("PRACTICODE_NEXT_REQUEST", request)
        .env("PRACTICODE_AI_PROVIDER", &provider)
        .env("PRACTICODE_AI_MODEL", &state.settings.ai_model)
        .env("PRACTICODE_AI_EFFORT", &state.settings.ai_effort);
    match run_capture(&mut process, "", Duration::from_secs(900)) {
        Ok(run) if run.code == Some(0) => {
            let output = output_text(&run.stdout, &run.stderr);
            format!("{provider} command finished\n{output}")
                .trim()
                .to_string()
        }
        Ok(run) => {
            let output = output_text(&run.stdout, &run.stderr);
            format!(
                "{provider} command failed ({})\n{output}",
                run.code.unwrap_or(-1)
            )
        }
        Err(error) => format!("{provider} command failed\n{error}"),
    }
}

pub(crate) enum AiGenerationResult {
    Succeeded(String),
    Failed { status: Option<i32>, detail: String },
    FailedToRun(String),
}

pub fn run_ai_generate(root: &Path, state: &AppState, request: &str) -> String {
    let provider = normalize_ai_provider(&state.settings.ai_provider);
    match run_ai_generate_result(root, state, request) {
        AiGenerationResult::Succeeded(detail) => {
            format!("{provider} background generation finished\n{detail}")
                .trim()
                .to_string()
        }
        AiGenerationResult::Failed { status, detail } => format!(
            "{provider} background generation failed ({})\n{detail}",
            status.unwrap_or(-1)
        ),
        AiGenerationResult::FailedToRun(detail) => {
            format!("{provider} background generation failed\n{detail}")
        }
    }
}

pub(crate) fn run_ai_generate_result(
    root: &Path,
    state: &AppState,
    request: &str,
) -> AiGenerationResult {
    let provider = normalize_ai_provider(&state.settings.ai_provider);
    let mut process = if state.settings.next_ai_command().trim().is_empty() {
        builtin_ai_process(
            root,
            &state.settings,
            &default_ai_generate_prompt_with_settings(&state.settings, request),
        )
    } else {
        shell_process(state.settings.next_ai_command())
    };
    process
        .current_dir(root)
        .env("PRACTICODE_NEXT_REQUEST", request)
        .env("PRACTICODE_GENERATE_BACKGROUND", "1")
        .env("PRACTICODE_AI_PROVIDER", &provider)
        .env("PRACTICODE_AI_MODEL", &state.settings.ai_model)
        .env("PRACTICODE_AI_EFFORT", &state.settings.ai_effort);
    match run_capture(&mut process, "", Duration::from_secs(900)) {
        Ok(run) if run.code == Some(0) => {
            AiGenerationResult::Succeeded(output_text(&run.stdout, &run.stderr))
        }
        Ok(run) => AiGenerationResult::Failed {
            status: run.code,
            detail: output_text(&run.stdout, &run.stderr),
        },
        Err(error) => AiGenerationResult::FailedToRun(error.to_string()),
    }
}

pub fn default_ai_next_command(root: &Path, settings: &Settings, request: &str) -> String {
    match normalize_ai_provider(&settings.ai_provider).as_str() {
        "claude" => default_claude_next_command(root, settings, request),
        _ => default_codex_next_command(root, settings, request),
    }
}

pub fn default_ai_generate_command(root: &Path, settings: &Settings, request: &str) -> String {
    match normalize_ai_provider(&settings.ai_provider).as_str() {
        "claude" => default_claude_generate_command(root, settings, request),
        _ => default_codex_generate_command(root, settings, request),
    }
}

pub fn provider_status(provider: &str, lang: &str) -> String {
    let provider = normalize_ai_provider(provider);
    let command = if provider == "claude" {
        "claude"
    } else {
        "codex"
    };
    provider_status_with(
        &provider,
        lang,
        which(command).is_some(),
        provider == "codex" && codex_daemon_path().is_some_and(|path| path.exists()),
    )
}

fn provider_status_with(provider: &str, lang: &str, found: bool, daemon: bool) -> String {
    if provider == "claude" {
        if found {
            return ui_text(lang, "provider_cli_found").replace("{provider}", "Claude");
        }
        return ui_text(lang, "provider_cli_missing")
            .replace("{provider}", "Claude")
            .replace("{install}", "Claude Code")
            .replace("{command}", "/provider codex");
    }
    if !found {
        return ui_text(lang, "provider_cli_missing")
            .replace("{provider}", "Codex")
            .replace("{install}", "Codex CLI")
            .replace("{command}", "/provider claude");
    }
    ui_text(
        lang,
        if daemon {
            "provider_codex_daemon_available"
        } else {
            "provider_codex_direct_fallback"
        },
    )
    .to_string()
}

pub fn available_models(provider: &str, lang: &str) -> ModelCatalog {
    match normalize_ai_provider(provider).as_str() {
        "codex" => codex_models(lang),
        "claude" => ModelCatalog {
            models: CLAUDE_MODEL_FALLBACKS
                .iter()
                .map(|model| (*model).to_string())
                .collect(),
            message: Some(
                ui_text(lang, "model_claude_presets")
                    .replace(
                        "{version}",
                        &claude_version().unwrap_or_else(|| "?".to_string()),
                    )
                    .replace("{efforts}", &CLAUDE_AI_EFFORTS.join(", ")),
            ),
        },
        _ => ModelCatalog::default(),
    }
}

pub fn default_ai_next_prompt(request: &str) -> String {
    default_ai_next_prompt_with_settings(&Settings::default(), request)
}

pub fn default_ai_next_prompt_with_settings(settings: &Settings, request: &str) -> String {
    format!(
        "Read problem_notes.md if present, problems/INDEX.md if present, problem_bank.json if present, and problem-state.json. Create exactly one new non-duplicate coding practice problem. The built-in 001-hello-world already exists, so do not duplicate it. User request: {}. User profile: difficulty preference: {}; preferred topics: {}; avoid topics: {}; code language: {}; UI language: {}; generated answer languages: {}; generated UI languages: {}. Treat difficulty auto as gradual progression from state; otherwise prefer the requested difficulty unless the direct user request conflicts. Make the smallest valid edits: update problem_bank.json, one problem directory, problems/INDEX.md, and problem-state.json. Do not include the answer in the problem statement. Do not create solution.*, test_solution.*, or any answer-revealing file inside the problem directory.",
        if request.is_empty() {
            "(none)"
        } else {
            request
        },
        settings.difficulty,
        list_or_none(&settings.topics),
        list_or_none(&settings.avoid_topics),
        settings.language,
        settings.ui_language,
        list_or_all(&settings.generate_languages, LANGUAGES),
        list_or_all(&settings.generate_ui_languages, UI_LANGUAGES)
    )
}

pub fn default_ai_generate_prompt_with_settings(settings: &Settings, request: &str) -> String {
    format!(
        "Read problem_notes.md if present, problems/INDEX.md if present, problem_bank.json if present, and problem-state.json. Create exactly one new non-duplicate coding practice problem for later use. The built-in 001-hello-world already exists, so do not duplicate it. User request: {}. User profile: difficulty preference: {}; preferred topics: {}; avoid topics: {}; current code language: {}; current UI language: {}; generated answer languages: {}; generated UI languages: {}. Treat difficulty auto as gradual progression from state; otherwise prefer the requested difficulty unless the direct user request conflicts. Make the smallest valid edits: update problem_bank.json, one problem directory, and problems/INDEX.md. Preserve problem-state.json current_problem, history, solved, and settings; do not switch the current problem. Do not include the answer in the problem statement. Do not create solution.*, test_solution.*, or any answer-revealing file inside the problem directory.",
        if request.is_empty() {
            "(none)"
        } else {
            request
        },
        settings.difficulty,
        list_or_none(&settings.topics),
        list_or_none(&settings.avoid_topics),
        settings.language,
        settings.ui_language,
        list_or_all(&settings.generate_languages, LANGUAGES),
        list_or_all(&settings.generate_ui_languages, UI_LANGUAGES)
    )
}

pub fn append_problem_note(root: &Path, note: &str) -> Result<()> {
    let path = root.join(PROBLEM_NOTES_PATH);
    let existing = read_problem_notes_file(root)?;
    let separator = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    save_user_text(&path, &format!("{existing}{separator}{}\n", note.trim()))
}

pub fn read_problem_notes(root: &Path) -> Result<String> {
    Ok(read_problem_notes_file(root)?.trim_end().to_string())
}

fn read_problem_notes_file(root: &Path) -> Result<String> {
    let path = root.join(PROBLEM_NOTES_PATH);
    if regular_file_exists(&path)? {
        fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
    } else {
        Ok(String::new())
    }
}

fn codex_models(lang: &str) -> ModelCatalog {
    if which("codex").is_none() {
        return ModelCatalog {
            models: Vec::new(),
            message: Some(
                ui_text(lang, "model_cli_missing")
                    .replace("{provider}", "Codex")
                    .replace("{command}", "/provider claude")
                    .replace("{install}", "Codex CLI"),
            ),
        };
    }
    if codex_daemon_path().is_none_or(|path| !path.exists()) {
        return ModelCatalog {
            models: codex_cached_models(),
            message: Some(ui_text(lang, "model_codex_daemon_unavailable").to_string()),
        };
    }
    let mut start = Command::new("codex");
    start.args(["app-server", "daemon", "start"]);
    let _ = run_capture(&mut start, "", Duration::from_secs(5));
    let mut command = Command::new("codex");
    command.args(["app-server", "proxy"]);
    let input = r#"{"id":1,"method":"model/list","params":{"limit":25}}"#;
    let Ok(run) = run_capture(&mut command, &format!("{input}\n"), Duration::from_secs(2)) else {
        return ModelCatalog {
            models: codex_cached_models(),
            message: Some(ui_text(lang, "model_codex_query_failed").to_string()),
        };
    };
    if run.code != Some(0) {
        let detail = output_text(&run.stdout, &run.stderr);
        return ModelCatalog {
            models: codex_cached_models(),
            message: Some(if detail.is_empty() {
                ui_text(lang, "model_codex_query_failed").to_string()
            } else {
                format!("{}: {detail}", ui_text(lang, "model_codex_query_failed"))
            }),
        };
    }
    let models = parse_model_list(&run.stdout);
    if models.is_empty() {
        ModelCatalog {
            models,
            message: Some(ui_text(lang, "model_codex_empty").to_string()),
        }
    } else {
        ModelCatalog {
            models,
            message: None,
        }
    }
}

fn codex_cached_models() -> Vec<String> {
    let mut models = env::var_os("HOME")
        .and_then(|home| {
            fs::read_to_string(PathBuf::from(home).join(".codex/models_cache.json")).ok()
        })
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|value| {
            value
                .get("models")
                .and_then(|models| models.as_array())
                .cloned()
        })
        .unwrap_or_default()
        .into_iter()
        .filter_map(|model| {
            model
                .get("slug")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    if models.is_empty() {
        models = CODEX_MODEL_FALLBACKS
            .iter()
            .map(|model| (*model).to_string())
            .collect();
    }
    models
}

fn claude_version() -> Option<String> {
    which("claude").and_then(|_| {
        let mut command = Command::new("claude");
        command.arg("--version");
        run_capture(&mut command, "", Duration::from_secs(2))
            .ok()
            .and_then(|run| {
                output_text(&run.stdout, &run.stderr)
                    .lines()
                    .next()
                    .map(str::to_string)
            })
    })
}

fn parse_model_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .flat_map(|value| {
            value
                .pointer("/result/data")
                .or_else(|| value.get("data"))
                .and_then(|data| data.as_array())
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|model| {
            model
                .get("model")
                .or_else(|| model.get("id"))
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .collect()
}

fn run_codex_prompt(root: &Path, settings: &Settings, prompt: &str) -> String {
    let output_path = unique_temp_path("practicode-last-message", "txt");
    let mut command = Command::new("codex");
    command
        .args(["exec", "--skip-git-repo-check", "--cd"])
        .arg(root)
        .args(["--sandbox", "read-only"])
        .current_dir(root);
    if let Some(model) = settings.model_arg() {
        command.args(["--model", model]);
    }
    add_codex_effort_args(&mut command, settings);
    command.arg("-o").arg(&output_path).arg(prompt);
    let result = run_capture(&mut command, "", Duration::from_secs(600));
    let last_message = fs::read_to_string(&output_path).unwrap_or_default();
    let _ = fs::remove_file(&output_path);
    match result {
        Ok(run) if run.code == Some(0) => {
            let output = output_text(&run.stdout, &run.stderr);
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
            let output = output_text(&run.stdout, &run.stderr);
            format!("Codex prompt failed ({})\n{output}", run.code.unwrap_or(-1))
        }
        Err(error) => format!("Codex prompt failed\n{error}"),
    }
}

fn run_claude_prompt(root: &Path, settings: &Settings, prompt: &str) -> String {
    let mut command = Command::new("claude");
    command
        .args(["--permission-mode", "plan"])
        .current_dir(root);
    if let Some(model) = settings.model_arg() {
        command.args(["--model", model]);
    }
    if let Some(effort) = settings.effort_arg() {
        command.args(["--effort", effort]);
    }
    command.args(["-p", prompt]);
    match run_capture(&mut command, "", Duration::from_secs(600)) {
        Ok(run) if run.code == Some(0) => {
            let output = output_text(&run.stdout, &run.stderr);
            if output.is_empty() {
                "Claude returned no output.".to_string()
            } else {
                output
            }
        }
        Ok(run) => {
            let output = output_text(&run.stdout, &run.stderr);
            format!(
                "Claude prompt failed ({})\n{output}",
                run.code.unwrap_or(-1)
            )
        }
        Err(error) => format!("Claude prompt failed\n{error}"),
    }
}

fn builtin_ai_process(root: &Path, settings: &Settings, prompt: &str) -> Command {
    if normalize_ai_provider(&settings.ai_provider) == "claude" {
        let mut command = Command::new("claude");
        command
            .args(["--permission-mode", "acceptEdits"])
            .current_dir(root);
        if let Some(model) = settings.model_arg() {
            command.args(["--model", model]);
        }
        if let Some(effort) = settings.effort_arg() {
            command.args(["--effort", effort]);
        }
        command.args(["-p", prompt]);
        command
    } else {
        let mut command = Command::new("codex");
        command
            .args(["exec", "--ephemeral", "--skip-git-repo-check", "--cd"])
            .arg(root)
            .args(["--sandbox", "workspace-write"])
            .current_dir(root);
        if let Some(model) = settings.model_arg() {
            command.args(["--model", model]);
        }
        add_codex_effort_args(&mut command, settings);
        command.arg(prompt);
        command
    }
}

fn default_codex_next_command(root: &Path, settings: &Settings, request: &str) -> String {
    let start = "if [ -x \"$HOME/.codex/packages/standalone/current/codex\" ]; then codex app-server daemon start >/dev/null 2>&1 || true; fi";
    let mut exec = format!(
        "codex exec --ephemeral --skip-git-repo-check --cd {} --sandbox workspace-write",
        sh_quote(&root.display().to_string())
    );
    if let Some(model) = settings.model_arg() {
        exec.push_str(&format!(" --model {}", sh_quote(model)));
    }
    push_codex_effort_arg(&mut exec, settings);
    exec.push(' ');
    exec.push_str(&sh_quote(&default_ai_next_prompt_with_settings(
        settings, request,
    )));
    format!("{start}; {exec}")
}

fn default_codex_generate_command(root: &Path, settings: &Settings, request: &str) -> String {
    let start = "if [ -x \"$HOME/.codex/packages/standalone/current/codex\" ]; then codex app-server daemon start >/dev/null 2>&1 || true; fi";
    let mut exec = format!(
        "codex exec --ephemeral --skip-git-repo-check --cd {} --sandbox workspace-write",
        sh_quote(&root.display().to_string())
    );
    if let Some(model) = settings.model_arg() {
        exec.push_str(&format!(" --model {}", sh_quote(model)));
    }
    push_codex_effort_arg(&mut exec, settings);
    exec.push(' ');
    exec.push_str(&sh_quote(&default_ai_generate_prompt_with_settings(
        settings, request,
    )));
    format!("{start}; {exec}")
}

fn codex_daemon_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".codex/packages/standalone/current")
            .join(if cfg!(windows) { "codex.exe" } else { "codex" })
    })
}

fn default_claude_next_command(root: &Path, settings: &Settings, request: &str) -> String {
    let mut claude = "claude --permission-mode acceptEdits".to_string();
    if let Some(model) = settings.model_arg() {
        claude.push_str(&format!(" --model {}", sh_quote(model)));
    }
    if let Some(effort) = settings.effort_arg() {
        claude.push_str(&format!(" --effort {}", sh_quote(effort)));
    }
    claude.push_str(" -p ");
    claude.push_str(&sh_quote(&default_ai_next_prompt_with_settings(
        settings, request,
    )));
    format!(
        "claude daemon status >/dev/null 2>&1 || true; cd {}; {}",
        sh_quote(&root.display().to_string()),
        claude
    )
}

fn default_claude_generate_command(root: &Path, settings: &Settings, request: &str) -> String {
    let mut claude = "claude --permission-mode acceptEdits".to_string();
    if let Some(model) = settings.model_arg() {
        claude.push_str(&format!(" --model {}", sh_quote(model)));
    }
    if let Some(effort) = settings.effort_arg() {
        claude.push_str(&format!(" --effort {}", sh_quote(effort)));
    }
    claude.push_str(" -p ");
    claude.push_str(&sh_quote(&default_ai_generate_prompt_with_settings(
        settings, request,
    )));
    format!(
        "claude daemon status >/dev/null 2>&1 || true; cd {}; {}",
        sh_quote(&root.display().to_string()),
        claude
    )
}

fn output_text(stdout: &str, stderr: &str) -> String {
    [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn add_codex_effort_args(command: &mut Command, settings: &Settings) {
    if let Some(effort) = settings.effort_arg() {
        let effort = if effort == "max" { "xhigh" } else { effort };
        command.args(["-c", &format!("model_reasoning_effort=\"{effort}\"")]);
    }
}

fn push_codex_effort_arg(command: &mut String, settings: &Settings) {
    if let Some(effort) = settings.effort_arg() {
        let effort = if effort == "max" { "xhigh" } else { effort };
        command.push_str(&format!(
            " -c {}",
            sh_quote(&format!("model_reasoning_effort=\"{effort}\""))
        ));
    }
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

fn list_or_all(values: &[String], all: &[&str]) -> String {
    if values.is_empty() {
        all.join(", ")
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_codex_command_keeps_values_as_separate_arguments() {
        let root = Path::new("workspace with spaces");
        let settings = Settings {
            ai_model: "model with spaces".to_string(),
            ai_effort: "high".to_string(),
            ..Settings::default()
        };
        let command = builtin_ai_process(root, &settings, "quote ' & percent %");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), "codex");
        assert_eq!(command.get_current_dir(), Some(root));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--cd", "workspace with spaces"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--model", "model with spaces"])
        );
        assert_eq!(args.last().unwrap(), "quote ' & percent %");
    }

    #[cfg(unix)]
    #[test]
    fn builtin_codex_command_preserves_a_non_utf8_workspace_path() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let root = PathBuf::from(OsString::from_vec(b"workspace-\xff".to_vec()));
        let command = builtin_ai_process(&root, &Settings::default(), "prompt");
        let args = command.get_args().collect::<Vec<_>>();
        let cd = args.iter().position(|arg| *arg == "--cd").unwrap();

        assert_eq!(args[cd + 1], root.as_os_str());
    }

    #[test]
    fn builtin_claude_command_keeps_values_as_separate_arguments() {
        let root = Path::new("workspace with spaces");
        let settings = Settings {
            ai_provider: "claude".to_string(),
            ai_model: "model with spaces".to_string(),
            ai_effort: "max".to_string(),
            ..Settings::default()
        };
        let command = builtin_ai_process(root, &settings, "quote ' & percent %");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), "claude");
        assert_eq!(command.get_current_dir(), Some(root));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--model", "model with spaces"])
        );
        assert!(args.windows(2).any(|pair| pair == ["--effort", "max"]));
        assert_eq!(args.last().unwrap(), "quote ' & percent %");
    }

    #[test]
    fn parses_codex_model_list_response() {
        let output =
            r#"{"id":1,"result":{"data":[{"model":"gpt-test","displayName":"GPT Test"}]}}"#;
        assert_eq!(parse_model_list(output), vec!["gpt-test"]);
    }

    #[test]
    fn provider_status_localizes_every_app_owned_branch() {
        let cases = [
            ("claude", true, false, "CLI를 찾았습니다"),
            ("claude", false, false, "CLI가 없습니다"),
            ("codex", false, false, "CLI가 없습니다"),
            ("codex", true, true, "데몬을 사용할 수 있습니다"),
            ("codex", true, false, "codex exec를 직접 사용합니다"),
        ];
        for (provider, found, daemon, expected) in cases {
            let status = provider_status_with(provider, "ko", found, daemon);
            assert!(status.contains(expected), "{provider}:{expected}: {status}");
            assert!(!status.contains("not found"), "{status}");
            assert!(!status.contains("is available"), "{status}");
        }
    }

    #[test]
    fn bundled_model_notice_localizes_copy_and_preserves_raw_tokens() {
        let catalog = available_models("claude", "zh");
        let message = catalog.message.unwrap();

        assert!(message.contains("内置 Claude 预设"), "{message}");
        assert!(message.contains("--help"), "{message}");
        assert!(message.contains("auto, low, medium"), "{message}");
        assert!(!message.contains("Bundled Claude presets"), "{message}");
    }
}
