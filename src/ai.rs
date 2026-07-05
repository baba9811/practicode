use crate::{
    core::{
        AppState, PROBLEM_NOTES_PATH, Problem, Settings, ensure_submission, normalize_ai_provider,
        render_problem,
    },
    process::{run_capture, sh_quote, shell_process, unique_temp_path, which},
};
use anyhow::Result;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Clone, Debug, Default)]
pub struct ModelCatalog {
    pub models: Vec<String>,
    pub message: Option<String>,
}

pub fn run_ai_prompt(root: &Path, problem: &Problem, settings: &Settings, prompt: &str) -> String {
    let solution = match ensure_submission(root, problem, settings) {
        Ok(path) => path,
        Err(error) => return format!("AI prompt failed\n{error}"),
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
        settings.language
    );
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
    let command = if state.settings.next_ai_command().trim().is_empty() {
        default_ai_next_command(root, &state.settings, request)
    } else {
        state.settings.next_ai_command().to_string()
    };
    let mut process = shell_process(&command);
    process
        .current_dir(root)
        .env("PRACTICODE_NEXT_REQUEST", request)
        .env("PRACTICODE_AI_PROVIDER", &provider)
        .env("PRACTICODE_AI_MODEL", &state.settings.ai_model);
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

pub fn default_ai_next_command(root: &Path, settings: &Settings, request: &str) -> String {
    match normalize_ai_provider(&settings.ai_provider).as_str() {
        "claude" => default_claude_next_command(root, settings, request),
        _ => default_codex_next_command(root, settings, request),
    }
}

pub fn provider_status(provider: &str) -> String {
    match normalize_ai_provider(provider).as_str() {
        "claude" => {
            if which("claude").is_some() {
                "Claude CLI found.".to_string()
            } else {
                "Claude CLI not found. Install Claude Code or choose /provider codex.".to_string()
            }
        }
        _ => {
            if which("codex").is_none() {
                return "Codex CLI not found. Install Codex CLI or choose /provider claude."
                    .to_string();
            }
            if codex_daemon_path().is_some_and(|path| path.exists()) {
                "Codex CLI found. App-server daemon is available.".to_string()
            } else {
                "Codex CLI found. App-server daemon is not available; practicode will use codex exec directly.".to_string()
            }
        }
    }
}

pub fn available_models(provider: &str) -> ModelCatalog {
    match normalize_ai_provider(provider).as_str() {
        "codex" => codex_models(),
        "claude" => ModelCatalog {
            models: Vec::new(),
            message: Some(
                "Claude CLI does not expose a model list; use /model <name> for a known model."
                    .to_string(),
            ),
        },
        _ => ModelCatalog::default(),
    }
}

pub fn default_ai_next_prompt(request: &str) -> String {
    format!(
        "Read AGENTS.md, docs/problem-authoring-notes.md if present, .practicode/problem_notes.md if present, problems/INDEX.md if present, .practicode/problem_bank.json if present, and .practicode/problem-state.json. Create exactly one new non-duplicate coding practice problem. The built-in 001-hello-world already exists, so do not duplicate it. User request: {}. Make the smallest valid edits: update .practicode/problem_bank.json, one problem directory, problems/INDEX.md, and .practicode/problem-state.json. Do not include the answer in the problem statement.",
        if request.is_empty() {
            "(none)"
        } else {
            request
        }
    )
}

pub fn append_problem_note(root: &Path, note: &str) -> Result<()> {
    let path = root.join(PROBLEM_NOTES_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", note.trim())?;
    Ok(())
}

pub fn read_problem_notes(root: &Path) -> Result<String> {
    let path = root.join(PROBLEM_NOTES_PATH);
    if path.exists() {
        Ok(fs::read_to_string(path)?.trim_end().to_string())
    } else {
        Ok(String::new())
    }
}

fn codex_models() -> ModelCatalog {
    if which("codex").is_none() {
        return ModelCatalog {
            models: Vec::new(),
            message: Some(
                "Codex CLI not found; choose /provider claude or install Codex CLI.".to_string(),
            ),
        };
    }
    if codex_daemon_path().is_none_or(|path| !path.exists()) {
        return ModelCatalog {
            models: Vec::new(),
            message: Some("Codex app-server daemon is unavailable; install the standalone Codex app to list models, or use /model <name>.".to_string()),
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
            models: Vec::new(),
            message: Some("Could not query Codex model list.".to_string()),
        };
    };
    if run.code != Some(0) {
        let detail = output_text(&run.stdout, &run.stderr);
        return ModelCatalog {
            models: Vec::new(),
            message: Some(if detail.is_empty() {
                "Could not query Codex model list.".to_string()
            } else {
                format!("Could not query Codex model list: {detail}")
            }),
        };
    }
    let models = parse_model_list(&run.stdout);
    if models.is_empty() {
        ModelCatalog {
            models,
            message: Some("Codex app-server returned no models.".to_string()),
        }
    } else {
        ModelCatalog {
            models,
            message: None,
        }
    }
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
        .args([
            "exec",
            "--cd",
            &root.display().to_string(),
            "--sandbox",
            "read-only",
        ])
        .current_dir(root);
    if let Some(model) = settings.model_arg() {
        command.args(["--model", model]);
    }
    command.args(["-o", &output_path.display().to_string(), prompt]);
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

fn default_codex_next_command(root: &Path, settings: &Settings, request: &str) -> String {
    let start = "if [ -x \"$HOME/.codex/packages/standalone/current/codex\" ]; then codex app-server daemon start >/dev/null 2>&1 || true; fi";
    let mut exec = format!(
        "codex exec --ephemeral --cd {} --sandbox workspace-write",
        sh_quote(&root.display().to_string())
    );
    if let Some(model) = settings.model_arg() {
        exec.push_str(&format!(" --model {}", sh_quote(model)));
    }
    exec.push(' ');
    exec.push_str(&sh_quote(&default_ai_next_prompt(request)));
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
    claude.push_str(" -p ");
    claude.push_str(&sh_quote(&default_ai_next_prompt(request)));
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

#[cfg(test)]
mod tests {
    use super::parse_model_list;

    #[test]
    fn parses_codex_model_list_response() {
        let output =
            r#"{"id":1,"result":{"data":[{"model":"gpt-test","displayName":"GPT Test"}]}}"#;
        assert_eq!(parse_model_list(output), vec!["gpt-test"]);
    }
}
