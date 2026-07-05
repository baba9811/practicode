mod common;

use common::tmp_root;
use practicode::{
    ai::{
        append_problem_note, default_ai_next_command, default_ai_next_prompt, read_problem_notes,
        run_ai_next,
    },
    core::{AppState, Settings},
};
use std::fs;

#[test]
fn default_ai_next_prompt_reads_notes_and_includes_request() {
    let prompt = default_ai_next_prompt("그래프 쉬운 문제");
    assert!(prompt.contains("docs/problem-authoring-notes.md"));
    assert!(prompt.contains(".practicode/problem_notes.md"));
    assert!(prompt.contains("그래프 쉬운 문제"));
}

#[test]
fn default_codex_command_uses_model_when_set() {
    let root = tmp_root("codex-command");
    let command = default_ai_next_command(
        &root,
        &Settings {
            next_source: "ai".to_string(),
            ai_model: "gpt-5-codex".to_string(),
            ..Settings::default()
        },
        "strings",
    );
    assert!(command.contains("codex app-server daemon start"));
    assert!(command.contains("--model 'gpt-5-codex'"));
    assert!(command.contains("strings"));
}

#[test]
fn default_claude_command_uses_print_mode_and_accepts_edits() {
    let root = tmp_root("claude-command");
    let command = default_ai_next_command(
        &root,
        &Settings {
            next_source: "ai".to_string(),
            ai_provider: "claude".to_string(),
            ai_model: "sonnet".to_string(),
            ..Settings::default()
        },
        "arrays",
    );
    assert!(command.contains("claude daemon status"));
    assert!(command.contains("claude --permission-mode acceptEdits"));
    assert!(command.contains("--model 'sonnet'"));
    assert!(command.contains("arrays"));
}

#[test]
#[cfg(unix)]
fn run_ai_next_exposes_request_provider_and_model_to_custom_command() {
    let root = tmp_root("ai-env");
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings {
            next_source: "ai".to_string(),
            ai_provider: "claude".to_string(),
            ai_model: "sonnet".to_string(),
            ai_next_command:
                "printf '%s|%s|%s' \"$PRACTICODE_NEXT_REQUEST\" \"$PRACTICODE_AI_PROVIDER\" \"$PRACTICODE_AI_MODEL\" > request.txt"
                    .to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
    };
    let output = run_ai_next(&root, &state, false, "문자열 쉬운 문제");
    assert!(output.contains("finished"));
    assert_eq!(
        fs::read_to_string(root.join("request.txt")).unwrap(),
        "문자열 쉬운 문제|claude|sonnet"
    );
}

#[test]
fn problem_notes_append_and_read_local_file() {
    let root = tmp_root("notes");
    append_problem_note(&root, "Prefer string problems.").unwrap();
    append_problem_note(&root, "Avoid DP.").unwrap();
    assert_eq!(
        read_problem_notes(&root).unwrap(),
        "Prefer string problems.\nAvoid DP."
    );
}
