mod common;

use common::tmp_root;
use practicode::{
    ai::{
        append_problem_note, build_lesson_ai_prompt, default_ai_generate_prompt_with_settings,
        default_ai_next_command, default_ai_next_prompt, default_ai_next_prompt_with_settings,
        provider_status, read_problem_notes, run_ai_generate, run_ai_next,
    },
    core::{AppState, Settings, syntax_lessons_for},
};

#[test]
fn tui_run_does_not_eagerly_start_ai_model_discovery() {
    let source = include_str!("../src/tui.rs");
    let run_body = source
        .split_once("pub fn run")
        .unwrap()
        .1
        .split_once("pub fn handle_command_for_test")
        .unwrap()
        .0;

    assert!(
        !run_body.contains("start_model_check"),
        "AI model discovery must stay lazy until /model or the AI model setting is used"
    );
}
use std::fs;

fn app_state(settings: Settings) -> AppState {
    AppState {
        current_problem: "001-hello-world".to_string(),
        settings,
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
    }
}

#[test]
fn default_ai_next_prompt_reads_notes_and_includes_request() {
    let prompt = default_ai_next_prompt("그래프 쉬운 문제");
    assert!(prompt.contains("problem_notes.md"));
    assert!(prompt.contains("problem_bank.json"));
    assert!(prompt.contains("problem-state.json"));
    assert!(!prompt.contains(".practicode/problem"));
    assert!(prompt.contains("그래프 쉬운 문제"));
}

#[test]
fn default_ai_next_prompt_includes_user_profile() {
    let prompt = default_ai_next_prompt_with_settings(
        &Settings {
            language: "rust".to_string(),
            ui_language: "ko".to_string(),
            difficulty: "medium".to_string(),
            topics: vec!["strings".to_string(), "hashmap".to_string()],
            avoid_topics: vec!["dp".to_string()],
            ..Settings::default()
        },
        "more parsing practice",
    );
    assert!(prompt.contains("User profile"));
    assert!(prompt.contains("difficulty preference: medium"));
    assert!(prompt.contains("preferred topics: strings, hashmap"));
    assert!(prompt.contains("avoid topics: dp"));
    assert!(prompt.contains("code language: rust"));
    assert!(prompt.contains("UI language: ko"));
}

#[test]
fn default_ai_prompts_include_generation_language_scope() {
    let settings = Settings {
        generate_languages: vec!["python".to_string(), "rust".to_string()],
        generate_ui_languages: vec!["ko".to_string(), "en".to_string()],
        ..Settings::default()
    };
    let prompt = default_ai_next_prompt_with_settings(&settings, "strings");
    assert!(prompt.contains("generated answer languages: python, rust"));
    assert!(prompt.contains("generated UI languages: ko, en"));

    let background = default_ai_generate_prompt_with_settings(&settings, "strings");
    assert!(background.contains("for later use"));
    assert!(background.contains("Preserve problem-state.json current_problem"));
}

#[test]
fn default_ai_prompts_forbid_answer_files_in_problem_dirs() {
    let next = default_ai_next_prompt("arrays");
    assert!(next.contains("Do not create solution.*"));
    assert!(next.contains("test_solution.*"));

    let background = default_ai_generate_prompt_with_settings(&Settings::default(), "arrays");
    assert!(background.contains("Do not create solution.*"));
    assert!(background.contains("test_solution.*"));
}

#[test]
fn default_codex_command_uses_model_when_set() {
    let root = tmp_root("codex-command");
    let command = default_ai_next_command(
        &root,
        &Settings {
            next_source: "ai".to_string(),
            ai_model: "gpt-5-codex".to_string(),
            ai_effort: "high".to_string(),
            ..Settings::default()
        },
        "strings",
    );
    assert!(command.contains("codex app-server daemon start"));
    assert!(command.contains("--ephemeral"));
    assert!(command.contains("--skip-git-repo-check"));
    assert!(command.contains("--model 'gpt-5-codex'"));
    assert!(command.contains("-c 'model_reasoning_effort=\"high\"'"));
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
            ai_effort: "max".to_string(),
            ..Settings::default()
        },
        "arrays",
    );
    assert!(command.contains("claude daemon status"));
    assert!(command.contains("claude --permission-mode acceptEdits"));
    assert!(command.contains("--model 'sonnet'"));
    assert!(command.contains("--effort 'max'"));
    assert!(command.contains("arrays"));
}

#[test]
fn provider_status_reports_cli_and_daemon_state() {
    let status = provider_status("codex", "en");
    assert!(status.contains("Codex CLI"));
    assert!(
        status.contains("daemon")
            || status.contains("Install Codex CLI")
            || status.contains("codex exec")
    );
}

#[test]
fn lesson_ask_prompt_uses_lesson_tutor_context() {
    let lesson = syntax_lessons_for("python")
        .into_iter()
        .find(|lesson| lesson.id == "py-lists-dicts")
        .unwrap();
    let prompt = build_lesson_ai_prompt(
        lesson,
        &Settings {
            language: "python".to_string(),
            ui_language: "ko".to_string(),
            ..Settings::default()
        },
        "딕셔너리 키가 왜 필요한지 모르겠어",
        "submissions/.syntax/python/py-lists-dicts/exercise.py",
        "nums = [2, 3]\n# TODO: nums의 합계를 출력하세요.\n",
        "FAIL 0/1\nExpected 5, got empty output.",
    );

    assert!(prompt.contains("Socratic programming tutor"));
    assert!(prompt.contains("current syntax lesson"));
    assert!(prompt.contains("딕셔너리 키가 왜 필요한지 모르겠어"));
    assert!(prompt.contains("py-lists-dicts"));
    assert!(prompt.contains("딕셔너리 키별로 점수 모으기"));
    assert!(prompt.contains("submissions/.syntax/python/py-lists-dicts/exercise.py"));
    assert!(prompt.contains("FAIL 0/1"));
    assert!(prompt.contains("Do not give the full exercise solution"));
    assert!(!prompt.contains("current problem"));
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
            ai_effort: "high".to_string(),
            ai_next_command:
                "printf '%s|%s|%s|%s' \"$PRACTICODE_NEXT_REQUEST\" \"$PRACTICODE_AI_PROVIDER\" \"$PRACTICODE_AI_MODEL\" \"$PRACTICODE_AI_EFFORT\" > request.txt"
                    .to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
    };
    let output = run_ai_next(&root, &state, false, "문자열 쉬운 문제");
    assert!(output.contains("finished"));
    assert_eq!(
        fs::read_to_string(root.join("request.txt")).unwrap(),
        "문자열 쉬운 문제|claude|sonnet|high"
    );
}

#[test]
#[cfg(unix)]
fn run_ai_generate_preserves_public_success_output_and_runs_once() {
    let root = tmp_root("ai-generate-success-api");
    let state = app_state(Settings {
        ai_provider: "claude".to_string(),
        ai_next_command: "printf x >> generate-count.txt; printf 'generated detail'".to_string(),
        ..Settings::default()
    });

    let output: String = run_ai_generate(&root, &state, "arrays");

    assert_eq!(
        output,
        "claude background generation finished\ngenerated detail"
    );
    assert_eq!(
        fs::read_to_string(root.join("generate-count.txt")).unwrap(),
        "x"
    );
}

#[test]
#[cfg(unix)]
fn run_ai_generate_preserves_public_failure_output() {
    let root = tmp_root("ai-generate-failure-api");
    let state = app_state(Settings {
        ai_provider: "codex".to_string(),
        ai_next_command: "printf 'failure detail' >&2; exit 7".to_string(),
        ..Settings::default()
    });

    let output: String = run_ai_generate(&root, &state, "arrays");

    assert_eq!(
        output,
        "codex background generation failed (7)\nfailure detail"
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

#[test]
fn appending_a_note_preserves_existing_spacing() {
    let root = tmp_root("notes-spacing");
    fs::write(root.join("problem_notes.md"), "First note.\n\n").unwrap();

    append_problem_note(&root, "Second note.").unwrap();

    assert_eq!(
        fs::read_to_string(root.join("problem_notes.md")).unwrap(),
        "First note.\n\nSecond note.\n"
    );
}
