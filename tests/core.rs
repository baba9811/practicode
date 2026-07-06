mod common;

use common::{tmp_root, two_problem_bank};
use practicode::{
    core::{
        AppState, HistoryItem, Settings, ensure_submission, judge, load_bank, load_state,
        localized, next_problem, parse_language_list, parse_ui_language_list, problem_by_id,
        record_pass, render_problem, render_problem_tui, save_bank, save_state, syntax_lesson_text,
        syntax_progress_count,
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
    assert_eq!(state.settings.ui_language, "en");
    assert_eq!(state.settings.difficulty, "auto");
    assert!(state.settings.topics.is_empty());
    assert!(state.settings.avoid_topics.is_empty());
    assert_eq!(state.settings.ai_provider, "codex");
    assert_eq!(state.settings.ai_model, "auto");
    assert_eq!(state.settings.ai_effort, "auto");
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
fn load_bank_accepts_partial_answers_for_generation_profile() {
    let root = tmp_root("partial-answers");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.answers.retain(|language, _| language == "python");
    save_bank(&root, &[problem]).unwrap();
    let loaded = load_bank(&root).unwrap();
    assert_eq!(loaded[0].answers.len(), 1);
    assert!(loaded[0].answers.contains_key("python"));
}

#[test]
fn generation_language_lists_accept_all_or_known_values_only() {
    assert_eq!(
        parse_language_list("python, rust, ruby"),
        vec!["python", "rust"]
    );
    assert!(parse_language_list("all").is_empty());
    assert_eq!(parse_ui_language_list("ko, en, xx"), vec!["ko", "en"]);
    assert!(parse_ui_language_list("all").is_empty());
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
fn load_state_normalizes_practice_profile() {
    let root = tmp_root("state-profile");
    let bank = load_bank(&root).unwrap();
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(
        root.join(".practicode/problem-state.json"),
        r##"{
  "current_problem": "001-hello-world",
  "settings": {
    "difficulty": "weird",
    "topics": [" Arrays ", "#Strings", "arrays"],
    "avoid_topics": [" DP ", ""]
  }
}"##,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.difficulty, "auto");
    assert_eq!(state.settings.topics, vec!["arrays", "strings"]);
    assert_eq!(state.settings.avoid_topics, vec!["dp"]);
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
            ai_effort: "max".to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
    };
    save_state(&root, &state).unwrap();
    let saved = fs::read_to_string(root.join(".practicode/problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_provider\": \"claude\""));
    assert!(saved.contains("\"ai_model\": \"sonnet\""));
    assert!(saved.contains("\"ai_effort\": \"max\""));
    assert_eq!(load_state(&root, &bank).unwrap().settings.next_source, "ai");
}

#[test]
fn load_state_normalizes_ai_effort_by_provider() {
    let root = tmp_root("state-ai-effort");
    let bank = load_bank(&root).unwrap();
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(
        root.join(".practicode/problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "settings": {
    "ai_provider": "codex",
    "ai_effort": "max"
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.ai_effort, "xhigh");
}

#[test]
fn load_state_normalizes_syntax_progress() {
    let root = tmp_root("state-syntax-progress");
    let bank = load_bank(&root).unwrap();
    fs::create_dir_all(root.join(".practicode")).unwrap();
    fs::write(
        root.join(".practicode/problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "syntax_progress": {
    "python": ["io", "unknown", "io"],
    "ruby": ["io"]
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.syntax_progress["python"], vec!["io"]);
    assert!(!state.syntax_progress.contains_key("ruby"));
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
    assert!(rendered.contains("## 입력\n\n입력은 없습니다.\n\n## 출력\n\n`Hello, World!` 한 줄"));
    assert!(rendered.contains("```text\n\n```"));
}

#[test]
fn render_problem_defaults_to_english_and_supports_common_ui_languages() {
    let root = tmp_root("render-i18n");
    let problem = load_bank(&root).unwrap().remove(0);
    assert!(render_problem(&problem, "xx").contains("## Input\n\nNo input."));
    assert!(render_problem(&problem, "ja").contains("入力はありません。"));
    assert!(render_problem(&problem, "zh-CN").contains("没有输入。"));
    assert!(render_problem(&problem, "es").contains("No hay entrada."));
}

#[test]
fn render_problem_tui_is_scannable_plain_text() {
    let root = tmp_root("render-tui");
    let problem = load_bank(&root).unwrap().remove(0);
    let rendered = render_problem_tui(&problem, "en");
    assert!(rendered.contains("001. Hello World"));
    assert!(rendered.contains("Difficulty: easy    Topics: io"));
    assert!(rendered.contains("Input\n  No input."));
    assert!(rendered.contains("Examples\n  Example 1"));
    assert!(!rendered.contains("```"));
    assert!(!rendered.contains("##"));
}

#[test]
fn render_markdown_plain_hides_problem_markdown_syntax() {
    let root = tmp_root("render-plain");
    let problem = load_bank(&root).unwrap().remove(0);
    let rendered = render_markdown_plain(&render_problem(&problem, "ko"));
    assert!(rendered.contains("001. Hello World"));
    assert!(rendered.contains("입력"));
    assert!(rendered.contains("출력"));
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
    assert!(result.output.contains("Got\n  debug\n  Hello, World!"));
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
        syntax_progress: Default::default(),
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
fn next_problem_prefers_profile_difficulty_when_fixed() {
    let root = tmp_root("next-profile-difficulty");
    let mut bank = two_problem_bank(&root);
    bank[1].difficulty = "medium".to_string();
    save_bank(&root, &bank).unwrap();
    let mut state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings {
            difficulty: "medium".to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: vec![HistoryItem {
            id: "001-hello-world".to_string(),
            status: "solved".to_string(),
        }],
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
    };
    let next = next_problem(&root, &bank, &mut state).unwrap().unwrap();
    assert_eq!(next.difficulty, "medium");
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
        syntax_progress: Default::default(),
    };
    record_pass(&root, &bank[0], &mut state).unwrap();
    let saved = load_state(&root, &bank).unwrap();
    assert!(saved.solved.contains(&"001-hello-world".to_string()));
    assert_eq!(saved.history[0].status, "solved");
    assert_eq!(saved.suggested_next_difficulty, "medium");
}

#[test]
fn record_pass_tracks_syntax_progress_for_current_language_topics() {
    let root = tmp_root("record-pass-syntax");
    let bank = two_problem_bank(&root);
    let mut state = AppState {
        current_problem: "002-echo".to_string(),
        settings: Settings {
            language: "rust".to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
    };
    record_pass(&root, &bank[1], &mut state).unwrap();
    let saved = load_state(&root, &bank).unwrap();
    assert_eq!(syntax_progress_count(&saved, "rust"), (2, 5));
    assert_eq!(saved.syntax_progress["rust"], vec!["io", "strings"]);
    assert!(!saved.syntax_progress.contains_key("python"));
}

#[test]
fn syntax_lesson_text_uses_problem_topics_and_language_examples() {
    let root = tmp_root("syntax-lesson-text");
    let bank = two_problem_bank(&root);
    let state = AppState {
        current_problem: "002-echo".to_string(),
        settings: Settings::default(),
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
    };
    let text = syntax_lesson_text(&bank[1], "rust", "ko", &state);
    assert!(text.contains("Rust"));
    assert!(text.contains("read_to_string"));
    assert!(text.contains("문법"));
    assert!(text.contains("표준 입출력"));
    assert!(text.contains("문자열"));
    assert!(text.contains("[ ]"));
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
