use crate::{
    core::{
        AppState, PROBLEM_NOTES_PATH, Problem, Settings, ensure_submission, normalize_ai_provider,
        render_problem,
    },
    process::{run_capture, sh_quote, shell_process, unique_temp_path},
};
use anyhow::Result;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    time::Duration,
};

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
        return "AI next is disabled; using local problem bank.".to_string();
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

pub fn default_ai_next_prompt(request: &str) -> String {
    format!(
        "Read AGENTS.md, docs/problem-authoring-notes.md if present, .practicode/problem_notes.md if present, problems/INDEX.md if present, .practicode/problem_bank.json if present, and .practicode/problem-state.json. The app has a built-in starter problem 001-hello-world, so do not duplicate it. Create exactly one new non-duplicate coding practice problem. User request for this problem: {}. Update .practicode/problem_bank.json, the local problem files, the index, and state files. Do not include the answer in the problem statement.",
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
    let start = "codex app-server daemon start >/dev/null 2>&1 || true";
    let mut exec = format!(
        "codex exec --cd {} --sandbox workspace-write",
        sh_quote(&root.display().to_string())
    );
    if let Some(model) = settings.model_arg() {
        exec.push_str(&format!(" --model {}", sh_quote(model)));
    }
    exec.push(' ');
    exec.push_str(&sh_quote(&default_ai_next_prompt(request)));
    format!("{start}; {exec}")
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
