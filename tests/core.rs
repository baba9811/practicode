mod common;

use common::{tmp_root, two_problem_bank};
use practicode::{
    core::{
        AppState, HistoryItem, Settings, ensure_submission, judge, load_bank, load_state,
        localized, next_problem, problem_by_id, record_pass, render_problem, save_bank, save_state,
    },
    process::which,
    text::render_markdown_plain,
};
use std::fs;

#[test]
fn load_state_uses_first_problem_when_state_file_is_missing() {
    let root = tmp_root("state-missing");
    let bank = load_bank(&root).unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.current_problem, "001-hello-world");
    assert_eq!(state.settings.language, "python");
    assert_eq!(state.settings.ui_language, "ko");
    assert_eq!(state.settings.ai_provider, "codex");
    assert_eq!(state.settings.ai_model, "auto");
}

#[test]
fn save_bank_creates_local_custom_problem_bank() {
    let root = tmp_root("save-bank");
    let bank = two_problem_bank(&root);
    let loaded = load_bank(&root).unwrap();
    assert!(root.join(".practicode/problem_bank.json").exists());
    assert_eq!(
        loaded.iter().map(|problem| &problem.id).collect::<Vec<_>>(),
        bank.iter().map(|problem| &problem.id).collect::<Vec<_>>()
    );
}

#[test]
fn load_bank_rejects_empty_custom_bank() {
    let root = tmp_root("empty-bank");
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(root.join(".practicode/problem_bank.json"), "[]").unwrap();
    let error = load_bank(&root).unwrap_err().to_string();
    assert!(error.contains("at least one problem"));
}

#[test]
fn load_bank_rejects_invalid_problem_shape() {
    let root = tmp_root("invalid-bank");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.id = "../bad".to_string();
    problem.cases.clear();
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(
        root.join(".practicode/problem_bank.json"),
        serde_json::to_string_pretty(&vec![problem]).unwrap(),
    )
    .unwrap();
    let error = load_bank(&root).unwrap_err().to_string();
    assert!(error.contains("invalid problem id"));
}

#[test]
fn load_state_keeps_next_source_to_current_values_only() {
    let root = tmp_root("state-source");
    let bank = load_bank(&root).unwrap();
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(
        root.join(".practicode/problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "settings": {
    "next_source": "codex"
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.next_source, "bank");
}

#[test]
fn save_state_writes_ai_settings_without_deprecated_empty_field() {
    let root = tmp_root("state-save");
    let bank = load_bank(&root).unwrap();
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings {
            next_source: "ai".to_string(),
            ai_provider: "claude".to_string(),
            ai_model: "sonnet".to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
    };
    save_state(&root, &state).unwrap();
    let saved = fs::read_to_string(root.join(".practicode/problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_provider\": \"claude\""));
    assert!(saved.contains("\"ai_model\": \"sonnet\""));
    assert_eq!(load_state(&root, &bank).unwrap().settings.next_source, "ai");
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
fn judge_rejects_problem_without_cases() {
    let root = tmp_root("judge-empty-cases");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.cases.clear();
    let result = judge(&root, &problem, &Settings::default());
    assert!(!result.passed);
    assert_eq!(result.total_cases, 0);
    assert!(result.output.contains("no judge cases"));
}

#[test]
fn judge_runs_submission_from_build_directory() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-cwd");
    let bank = load_bank(&root).unwrap();
    let settings = Settings::default();
    let path = ensure_submission(&root, &bank[0], &settings).unwrap();
    fs::write(
        path,
        "open('touch.txt', 'w').write('x')\nprint('Hello, World!')\n",
    )
    .unwrap();
    let result = judge(&root, &bank[0], &settings);
    assert!(result.passed, "{}", result.output);
    assert!(!root.join("touch.txt").exists());
    assert!(
        root.join(".practicode/build/001-hello-world/run/touch.txt")
            .exists()
    );
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
fn smoke_title_comes_from_current_problem() {
    let root = tmp_root("smoke");
    let bank = load_bank(&root).unwrap();
    save_bank(&root, &bank).unwrap();
    let state = load_state(&root, &bank).unwrap();
    let problem = problem_by_id(&bank, &state.current_problem).unwrap();
    assert_eq!(
        localized(&problem.title, &state.settings.ui_language),
        "Hello World"
    );
}
