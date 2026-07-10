mod common;

use common::{tmp_root, two_problem_bank};
use practicode::{
    core::{
        AppState, HistoryItem, LANGUAGES, Settings, ensure_submission, ensure_syntax_submission,
        judge, judge_path, load_bank, load_state, localized, next_problem, parse_language_list,
        parse_ui_language_list, problem_by_id, record_pass, render_problem, render_problem_tui,
        render_syntax_lesson, save_bank, save_state, syntax_cases, syntax_lessons_for,
        syntax_progress_count,
    },
    process::which,
    text::render_markdown_plain,
};
use std::{fs, process::Command};

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
fn load_state_defaults_start_mode_to_home() {
    let root = tmp_root("state-start-mode-default");
    let bank = load_bank(&root).unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.start_mode, "home");
}

#[test]
fn load_state_normalizes_start_mode() {
    let root = tmp_root("state-start-mode-normalize");
    let bank = load_bank(&root).unwrap();
    fs::write(
        root.join("problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "settings": {
    "start_mode": "weird"
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.start_mode, "home");
}

#[test]
fn save_bank_creates_local_custom_problem_bank() {
    let root = tmp_root("save-bank");
    let bank = two_problem_bank(&root);
    let loaded = load_bank(&root).unwrap();
    assert!(root.join("problem_bank.json").exists());
    assert_eq!(
        loaded.iter().map(|problem| &problem.id).collect::<Vec<_>>(),
        bank.iter().map(|problem| &problem.id).collect::<Vec<_>>()
    );
}

#[test]
fn load_bank_rejects_empty_custom_bank() {
    let root = tmp_root("empty-bank");
    fs::write(root.join("problem_bank.json"), "[]").unwrap();
    let error = load_bank(&root).unwrap_err().to_string();
    assert!(error.contains("at least one problem"));
}

#[test]
fn load_bank_rejects_invalid_problem_shape() {
    let root = tmp_root("invalid-bank");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.id = "../bad".to_string();
    problem.cases.clear();
    fs::write(
        root.join("problem_bank.json"),
        serde_json::to_string_pretty(&vec![problem]).unwrap(),
    )
    .unwrap();
    let error = load_bank(&root).unwrap_err().to_string();
    assert!(error.contains("invalid problem id"));
}

#[test]
fn load_bank_rejects_duplicate_ids_and_slugs() {
    let root = tmp_root("duplicate-bank");
    let mut bank = two_problem_bank(&root);
    bank[1].id = bank[0].id.clone();
    let error = save_bank(&root, &bank).unwrap_err().to_string();
    assert!(error.contains("duplicate problem id"));

    bank[1].id = "002-other".to_string();
    bank[1].slug = bank[0].slug.clone();
    let error = save_bank(&root, &bank).unwrap_err().to_string();
    assert!(error.contains("duplicate slug"));
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
    fs::write(
        root.join("problem-state.json"),
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
    fs::write(
        root.join("problem-state.json"),
        r##"{
  "current_problem": "001-hello-world",
  "suggested_next_difficulty": "weird",
  "settings": {
    "difficulty": "weird",
    "theme": " Light ",
    "topics": [" Arrays ", "#Strings", "arrays"],
    "avoid_topics": [" DP ", ""]
  }
}"##,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.difficulty, "auto");
    assert_eq!(state.settings.theme, "light");
    assert_eq!(state.settings.topics, vec!["arrays", "strings"]);
    assert_eq!(state.settings.avoid_topics, vec!["dp"]);
    assert_eq!(state.suggested_next_difficulty, "easy");
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
        current_syntax_lesson: Default::default(),
    };
    save_state(&root, &state).unwrap();
    let saved = fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_provider\": \"claude\""));
    assert!(saved.contains("\"ai_model\": \"sonnet\""));
    assert!(saved.contains("\"ai_effort\": \"max\""));
    assert_eq!(load_state(&root, &bank).unwrap().settings.next_source, "ai");
}

#[test]
fn load_state_normalizes_ai_effort_by_provider() {
    let root = tmp_root("state-ai-effort");
    let bank = load_bank(&root).unwrap();
    fs::write(
        root.join("problem-state.json"),
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
fn load_state_normalizes_ai_provider_case_and_spaces() {
    let root = tmp_root("state-ai-provider");
    let bank = load_bank(&root).unwrap();
    fs::write(
        root.join("problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "settings": {
    "next_source": " AI ",
    "ai_provider": " Claude ",
    "ai_effort": " max "
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.settings.next_source, "ai");
    assert_eq!(state.settings.ai_provider, "claude");
    assert_eq!(state.settings.ai_effort, "max");
}

#[test]
fn load_state_normalizes_syntax_progress_for_learn_mode() {
    let root = tmp_root("state-syntax-progress");
    let bank = load_bank(&root).unwrap();
    fs::write(
        root.join("problem-state.json"),
        r#"{
  "current_problem": "001-hello-world",
  "syntax_progress": {
    "python": ["py-variables", "unknown", "py-variables"],
    "ruby": ["variables"]
  },
  "current_syntax_lesson": {
    "python": "py-functions",
    "ruby": "variables"
  }
}"#,
    )
    .unwrap();
    let state = load_state(&root, &bank).unwrap();
    assert_eq!(state.syntax_progress["python"], vec!["py-variables"]);
    assert_eq!(state.current_syntax_lesson["python"], "py-functions");
    assert!(!state.syntax_progress.contains_key("ruby"));
    assert!(!state.current_syntax_lesson.contains_key("ruby"));
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
fn judge_shows_stdout_on_pass() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-pass-stdout");
    let bank = load_bank(&root).unwrap();
    let settings = Settings::default();
    let path = ensure_submission(&root, &bank[0], &settings).unwrap();
    fs::write(path, "print('Hello, World!')\n").unwrap();
    let result = judge(&root, &bank[0], &settings);
    assert!(result.passed, "{}", result.output);
    assert!(result.output.contains("Stdout\n  Hello, World!"));
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
    assert!(result.output.find("Got").unwrap() < result.output.find("Expected").unwrap());
}

#[test]
fn judge_hides_case_input_and_expected_on_failure() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-hide-cases");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.cases = vec![practicode::core::IoCase {
        input: "private input".to_string(),
        output: "private expected".to_string(),
    }];
    let settings = Settings::default();
    let path = ensure_submission(&root, &problem, &settings).unwrap();
    fs::write(path, "print('wrong')\n").unwrap();

    let result = judge(&root, &problem, &settings);

    assert!(!result.passed);
    assert!(result.output.contains("Input\n  <hidden>"));
    assert!(result.output.contains("Expected\n  <hidden>"));
    assert!(!result.output.contains("private input"));
    assert!(!result.output.contains("private expected"));
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
    assert!(root.join("build/001-hello-world/run/touch.txt").exists());
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
        current_syntax_lesson: Default::default(),
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
        current_syntax_lesson: Default::default(),
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
        current_syntax_lesson: Default::default(),
    };
    record_pass(&root, &bank[0], &mut state).unwrap();
    let saved = load_state(&root, &bank).unwrap();
    assert!(saved.solved.contains(&"001-hello-world".to_string()));
    assert_eq!(saved.history[0].status, "solved");
    assert_eq!(saved.suggested_next_difficulty, "medium");
    assert!(saved.syntax_progress.is_empty());
}

#[test]
fn syntax_curriculum_covers_basic_to_advanced_for_every_supported_language() {
    for language in LANGUAGES {
        let lessons = syntax_lessons_for(language);
        assert!(
            lessons.len() >= 12,
            "{language} should have a real syntax course"
        );
        assert!(
            lessons.iter().all(|lesson| lesson.language == *language),
            "{language} should not fall back to another language's lessons"
        );
        for level in ["basic", "intermediate", "advanced"] {
            assert!(
                lessons.iter().any(|lesson| lesson.level == level),
                "{language} missing {level} syntax lessons"
            );
        }
        assert_eq!(
            lessons
                .iter()
                .filter(|lesson| lesson.exercise.cases.is_empty())
                .count(),
            0
        );
    }
}

#[test]
fn rust_syntax_curriculum_covers_core_book_topics() {
    let lesson_ids: Vec<_> = syntax_lessons_for("rust")
        .into_iter()
        .map(|lesson| lesson.id)
        .collect();

    assert!(lesson_ids.len() >= 28, "rust curriculum is too shallow");

    for id in [
        "rust-numbers-tuples",
        "rust-structs-impl",
        "rust-modules-use",
        "rust-option",
        "rust-borrowing-slices",
        "rust-generics",
        "rust-traits",
        "rust-lifetimes",
        "rust-testing",
        "rust-smart-pointers",
        "rust-interior-mutability",
        "rust-concurrency",
        "rust-shared-state",
        "rust-async-await",
        "rust-macros",
        "rust-unsafe",
        "rust-cargo-workspaces",
    ] {
        assert!(lesson_ids.contains(&id), "missing {id}");
    }
}

#[test]
fn rust_lesson_copy_is_topic_specific() {
    let banned = [
        "locating three concrete pieces",
        "Use this example to place",
        "edit only the part tied to this lesson's rule",
    ];
    for path in [
        "assets/lessons/rust/en.json",
        "assets/lessons/rust/ko.json",
        "assets/lessons/rust/ja.json",
        "assets/lessons/rust/zh.json",
        "assets/lessons/rust/es.json",
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for (id, copy) in catalog["lessons"].as_object().unwrap() {
            let text = copy.to_string();
            for phrase in banned {
                assert!(!text.contains(phrase), "{path}:{id}: generic copy");
            }
        }
    }
}

#[test]
fn java_syntax_curriculum_covers_official_java_topics() {
    let lessons = syntax_lessons_for("java");
    let lesson_ids: Vec<_> = lessons.iter().map(|lesson| lesson.id).collect();

    assert!(lesson_ids.len() >= 27, "java curriculum is too shallow");

    for id in [
        "java-output",
        "java-variables-types",
        "java-strings",
        "java-control-flow",
        "java-methods",
        "java-input",
        "java-arrays-collections",
        "java-classes-objects",
        "java-constructors",
        "java-encapsulation",
        "java-static-members",
        "java-enum-switch",
        "java-exceptions",
        "java-generics",
        "java-interfaces",
        "java-inheritance-composition",
        "java-records",
        "java-optional",
        "java-streams-lambdas",
        "java-comparators-sorting",
        "java-try-with-resources",
        "java-packages-imports",
        "java-annotations",
        "java-sealed-classes",
        "java-testing-assert",
    ] {
        assert!(lesson_ids.contains(&id), "missing {id}");
    }

    let refs = lessons
        .iter()
        .flat_map(|lesson| lesson.refs.iter().copied())
        .collect::<Vec<_>>()
        .join("\n");
    for required_ref in [
        "https://dev.java/learn/",
        "https://docs.oracle.com/javase/tutorial/",
        "https://docs.oracle.com/javase/specs/jls/se21/html/index.html",
        "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        "https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html",
        "https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/List.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Map.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Set.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Optional.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/stream/Stream.html",
        "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Comparator.html",
    ] {
        assert!(refs.contains(required_ref), "missing ref {required_ref}");
    }
}

#[test]
fn java_lesson_copy_is_topic_specific() {
    let banned = [
        "locating three concrete pieces",
        "Use this example to place",
        "Copying the shape of the example",
        "edit only the part tied to this lesson's rule",
        "Do not write the expected output as a constant",
        "matters when",
        "세 가지 구체적인 부분",
        "이 예제를 사용해",
        "三つの具体的な部分",
        "この例を使って",
        "三个具体部分",
        "用这个例子",
        "tres piezas concretas",
        "Usa este ejemplo",
    ];
    for path in [
        "assets/lessons/java/en.json",
        "assets/lessons/java/ko.json",
        "assets/lessons/java/ja.json",
        "assets/lessons/java/zh.json",
        "assets/lessons/java/es.json",
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for (id, copy) in catalog["lessons"].as_object().unwrap() {
            let text = copy.to_string();
            for phrase in banned {
                assert!(!text.contains(phrase), "{path}:{id}: generic copy");
            }
        }
    }
}

#[test]
fn python_syntax_curriculum_covers_official_python_topics() {
    let lessons = syntax_lessons_for("python");
    let lesson_ids: Vec<_> = lessons.iter().map(|lesson| lesson.id).collect();

    assert!(lesson_ids.len() >= 24, "python curriculum is too shallow");

    for id in [
        "py-output",
        "py-variables",
        "py-numbers",
        "py-strings",
        "py-control-flow",
        "py-functions",
        "py-input",
        "py-lists-dicts",
        "py-tuples-sets",
        "py-comprehensions",
        "py-errors",
        "py-files-context",
        "py-modules-imports",
        "py-dataclasses",
        "py-typing",
        "py-generators",
        "py-lambdas-closures",
        "py-decorators",
        "py-sorting-keys",
        "py-counter-defaultdict",
        "py-deque",
        "py-itertools",
        "py-pathlib",
        "py-testing-assert",
        "py-async",
    ] {
        assert!(lesson_ids.contains(&id), "missing {id}");
    }

    let refs = lessons
        .iter()
        .flat_map(|lesson| lesson.refs.iter().copied())
        .collect::<Vec<_>>()
        .join("\n");
    for required_ref in [
        "https://docs.python.org/3/tutorial/index.html",
        "https://docs.python.org/3/reference/index.html",
        "https://docs.python.org/3/library/index.html",
        "https://peps.python.org/pep-0008/",
        "https://docs.python.org/3/library/typing.html",
        "https://docs.python.org/3/library/pathlib.html",
        "https://docs.python.org/3/library/collections.html",
        "https://docs.python.org/3/library/itertools.html",
        "https://docs.python.org/3/library/contextlib.html",
        "https://docs.python.org/3/library/dataclasses.html",
        "https://docs.python.org/3/library/asyncio.html",
    ] {
        assert!(refs.contains(required_ref), "missing ref {required_ref}");
    }
}

#[test]
fn typescript_syntax_curriculum_covers_ts_and_node_topics() {
    let lessons = syntax_lessons_for("ts");
    let lesson_ids: Vec<_> = lessons.iter().map(|lesson| lesson.id).collect();

    assert!(
        lesson_ids.len() >= 28,
        "typescript curriculum is too shallow"
    );

    for id in [
        "ts-output",
        "ts-let-const",
        "ts-primitives",
        "ts-strings-templates",
        "ts-arrays-tuples",
        "ts-objects",
        "ts-functions",
        "ts-input",
        "ts-control-flow",
        "ts-union-narrowing",
        "ts-literal-types",
        "ts-optional-nullish",
        "ts-interfaces-aliases",
        "ts-generics",
        "ts-keyof-typeof",
        "ts-indexed-access",
        "ts-mapped-types",
        "ts-conditional-types",
        "ts-utility-types",
        "ts-discriminated-unions",
        "ts-async-promise",
        "ts-error-handling",
        "ts-modules",
        "ts-classes",
        "ts-readonly",
        "ts-satisfies-as-const",
        "ts-iterables",
        "ts-array-methods",
    ] {
        assert!(lesson_ids.contains(&id), "missing {id}");
    }

    let refs = lessons
        .iter()
        .flat_map(|lesson| lesson.refs.iter().copied())
        .collect::<Vec<_>>()
        .join("\n");
    for required_ref in [
        "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html",
        "https://www.typescriptlang.org/docs/handbook/2/narrowing.html",
        "https://www.typescriptlang.org/docs/handbook/2/generics.html",
        "https://www.typescriptlang.org/docs/handbook/2/keyof-types.html",
        "https://www.typescriptlang.org/docs/handbook/2/typeof-types.html",
        "https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html",
        "https://www.typescriptlang.org/docs/handbook/2/mapped-types.html",
        "https://www.typescriptlang.org/docs/handbook/2/conditional-types.html",
        "https://www.typescriptlang.org/docs/handbook/utility-types.html",
        "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-0.html",
        "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-4.html",
        "https://nodejs.org/api/typescript.html",
        "https://nodejs.org/api/fs.html#fsreadfilesyncpath-options",
        "https://nodejs.org/api/process.html#processstdout",
        "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reduce",
    ] {
        assert!(refs.contains(required_ref), "missing ref {required_ref}");
    }
}

#[test]
fn typescript_lesson_copy_is_topic_specific() {
    let banned = [
        "locating three concrete pieces",
        "Use this example to place",
        "Copying the shape of the example",
        "edit only the part tied to this lesson's rule",
        "Do not write the expected output as a constant",
        "세 가지 구체적인 부분",
        "이 예제를 사용해",
        "三つの具体的な部分",
        "この例を使って",
        "三个具体部分",
        "用这个例子",
        "tres piezas concretas",
        "Usa este ejemplo",
    ];
    for path in [
        "assets/lessons/typescript/en.json",
        "assets/lessons/typescript/ko.json",
        "assets/lessons/typescript/ja.json",
        "assets/lessons/typescript/zh.json",
        "assets/lessons/typescript/es.json",
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for (id, copy) in catalog["lessons"].as_object().unwrap() {
            let text = copy.to_string();
            for phrase in banned {
                assert!(!text.contains(phrase), "{path}:{id}: generic copy");
            }
        }
    }
}

#[test]
fn python_lesson_copy_is_topic_specific() {
    let banned = [
        "focuses on this Python skill",
        "Complete the exercise around this skill",
        "Keep the intended Python construct",
        "locating three concrete pieces",
        "edit only the part tied to this lesson's rule",
        "この構文が実際の問題でどの値を読み",
        "例は ",
        "这一课关注解题时真实会遇到的用法",
        "示例把",
        "se practica con el uso que aparece",
        "El ejemplo muestra",
    ];
    for path in [
        "assets/lessons/python/en.json",
        "assets/lessons/python/ko.json",
        "assets/lessons/python/ja.json",
        "assets/lessons/python/zh.json",
        "assets/lessons/python/es.json",
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for (id, copy) in catalog["lessons"].as_object().unwrap() {
            let text = copy.to_string();
            for phrase in banned {
                assert!(!text.contains(phrase), "{path}:{id}: generic copy");
            }
        }
    }
}

#[test]
fn rust_syntax_starters_compile_to_useful_failures() {
    if std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }

    let root = tmp_root("rust-syntax-starters-compile");
    for lesson in syntax_lessons_for("rust") {
        let path = ensure_syntax_submission(&root, lesson).unwrap();
        let result = judge_path(
            &root,
            lesson.id,
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("compile failed"),
            "{} starter should compile:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} starter should still require the learner edit",
            lesson.id
        );
    }
}

#[test]
fn render_syntax_lesson_uses_exercise_copy() {
    let lesson = syntax_lessons_for("python")[0];
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings::default(),
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
    };
    let english = render_syntax_lesson(lesson, &state);
    assert!(english.contains("Worked example"));
    assert!(english.contains("Exercise"));
    assert!(!english.contains("Drill"));

    let mut ko_state = state.clone();
    ko_state.settings.ui_language = "ko".to_string();
    let korean = render_syntax_lesson(lesson, &ko_state);
    assert!(korean.contains("풀이 예제"));
    assert!(korean.contains("실습"));
    assert!(!korean.contains("예제 풀이"));
}

#[test]
fn render_syntax_lesson_shows_exercise_io_goal() {
    let lesson = syntax_lessons_for("python")
        .into_iter()
        .find(|lesson| lesson.id == "py-output")
        .unwrap();
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings::default(),
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
    };

    let rendered = render_syntax_lesson(lesson, &state);

    assert!(rendered.contains("## Exercise"));
    assert!(rendered.find("## Exercise") < rendered.find("## Common mistakes"));
    assert!(rendered.contains("Input\n\n```text\n\n```"));
    assert!(rendered.contains("Output\n\n```text\nAda:7\n```"));
    let plain = render_markdown_plain(&rendered);
    assert!(plain.contains("  name = 'Ada'"));
    assert!(plain.contains("  score = 7"));
    assert!(plain.contains("Output\n\n  Ada:7"));
    assert!(plain.find("Output") < plain.find("Common mistakes"));
}

#[test]
fn lessons_use_rich_split_copy_for_all_code_languages() {
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings {
            ui_language: "ko".to_string(),
            ..Settings::default()
        },
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
    };

    for (ui_language, language, id, title, concept, mistakes, check) in [
        (
            "ko",
            "ts",
            "ts-arrays-tuples",
            "# 문법: 배열과 튜플",
            "배열은 순서가 있는 값의 묶음",
            "흔한 실수",
            "자가 점검",
        ),
        (
            "ja",
            "java",
            "java-arrays-collections",
            "# 文法: 配列とコレクション",
            "配列は長さが固定された値のまとまり",
            "よくある間違い",
            "セルフチェック",
        ),
        (
            "zh",
            "rust",
            "rust-vec-hashmap",
            "# 语法: Vec 与 HashMap",
            "有顺序的数据使用 Vec",
            "常见错误",
            "自我检查",
        ),
        (
            "es",
            "python",
            "py-lists-dicts",
            "# Sintaxis: Listas y diccionarios",
            "Las listas guardan valores en orden",
            "Errores frecuentes",
            "Autoevaluación",
        ),
    ] {
        let mut state = state.clone();
        state.settings.ui_language = ui_language.to_string();
        let lesson = syntax_lessons_for(language)
            .into_iter()
            .find(|lesson| lesson.id == id)
            .unwrap();
        let rendered = render_syntax_lesson(lesson, &state);

        assert!(
            rendered.contains(title),
            "{ui_language}:{language}: {rendered}"
        );
        assert!(
            rendered.contains(concept),
            "{ui_language}:{language}: {rendered}"
        );
        assert!(
            rendered.contains(mistakes),
            "{ui_language}:{language}: {rendered}"
        );
        assert!(
            rendered.contains(check),
            "{ui_language}:{language}: {rendered}"
        );
    }
}

#[test]
fn split_lesson_copy_covers_every_lesson_in_every_ui_language() {
    for (ui_language, mistakes, check) in [
        ("en", "Common mistakes", "Self-check"),
        ("ko", "흔한 실수", "자가 점검"),
        ("ja", "よくある間違い", "セルフチェック"),
        ("zh", "常见错误", "自我检查"),
        ("es", "Errores frecuentes", "Autoevaluación"),
    ] {
        let state = AppState {
            current_problem: "001-hello-world".to_string(),
            settings: Settings {
                ui_language: ui_language.to_string(),
                ..Settings::default()
            },
            solved: Vec::new(),
            history: Vec::new(),
            suggested_next_difficulty: "easy".to_string(),
            syntax_progress: Default::default(),
            current_syntax_lesson: Default::default(),
        };

        for language in ["python", "ts", "java", "rust"] {
            for lesson in syntax_lessons_for(language) {
                let rendered = render_syntax_lesson(lesson, &state);
                assert!(rendered.contains(mistakes), "{ui_language}:{}", lesson.id);
                assert!(rendered.contains(check), "{ui_language}:{}", lesson.id);
            }
        }
    }
}

#[test]
fn syntax_exercise_starters_require_user_edit_for_every_language() {
    for &language in LANGUAGES {
        for lesson in syntax_lessons_for(language) {
            assert!(
                lesson.exercise.starter.contains("TODO"),
                "{} starter should require a user edit",
                lesson.id
            );
            assert_ne!(
                lesson.exercise.starter.trim(),
                lesson.example.trim(),
                "{} starter should not be the worked example",
                lesson.id
            );
        }
    }
}

#[test]
fn syntax_exercise_todos_do_not_spell_out_the_answer() {
    let banned = [
        "print exactly",
        "so the output is",
        "output is",
        "expected text",
        "expected value",
        "expected fallback",
        "expected output",
        "produce Ada",
        "produces Ada",
        "choose the literal",
        "key whose value",
        "users route",
        "score=",
        "Ada:",
        "app:.txt",
        "cargo check --workspace",
    ];

    for &language in LANGUAGES {
        for lesson in syntax_lessons_for(language) {
            for line in lesson
                .exercise
                .starter
                .lines()
                .filter(|line| line.contains("TODO"))
            {
                let lower = line.to_lowercase();
                for phrase in banned {
                    assert!(
                        !lower.contains(&phrase.to_lowercase()),
                        "{} TODO gives away the answer with {phrase}: {line}",
                        lesson.id
                    );
                }
            }
        }
    }
}

#[test]
fn python_syntax_starters_fail_by_output_not_runtime_error() {
    let root = tmp_root("python-syntax-starters-run-cleanly");
    for lesson in syntax_lessons_for("python") {
        let path = ensure_syntax_submission(&root, lesson).unwrap();
        let result = judge_path(
            &root,
            lesson.id,
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("Stderr"),
            "{} starter should run without a runtime traceback:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} starter should still require the learner edit",
            lesson.id
        );
    }
}

#[test]
fn typescript_syntax_starters_run_under_node_strip_types() {
    if which("node").is_none() {
        return;
    }

    let root = tmp_root("typescript-syntax-starters-run-cleanly");
    for lesson in syntax_lessons_for("ts") {
        let path = ensure_syntax_submission(&root, lesson).unwrap();
        let result = judge_path(
            &root,
            lesson.id,
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("Stderr"),
            "{} starter should run without a Node/TypeScript runtime error:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} starter should still require the learner edit",
            lesson.id
        );
    }
}

#[test]
fn java_syntax_starters_compile_to_useful_failures() {
    if which("javac").is_none() || which("java").is_none() {
        return;
    }

    let root = tmp_root("java-syntax-starters-compile");
    for lesson in syntax_lessons_for("java") {
        let path = ensure_syntax_submission(&root, lesson).unwrap();
        let result = judge_path(
            &root,
            lesson.id,
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("compile failed"),
            "{} starter should compile:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.output.contains("Stderr"),
            "{} starter should fail by expected output, not runtime stderr:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} starter should still require the learner edit",
            lesson.id
        );
    }
}

#[test]
fn python_syntax_examples_run_cleanly() {
    let Some(python) = which("python3").or_else(|| which("python")) else {
        return;
    };
    let root = tmp_root("python-syntax-examples-run");
    for lesson in syntax_lessons_for("python") {
        let path = root.join(format!("{}.py", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let output = Command::new(&python).arg(&path).output().unwrap();
        assert!(
            output.status.success(),
            "{} example should exit successfully\nstdout:\n{}\nstderr:\n{}",
            lesson.id,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).trim().is_empty(),
            "{} example should not write stderr\n{}",
            lesson.id,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn python_syntax_examples_are_not_answer_keys() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }

    let root = tmp_root("python-syntax-examples-not-answer-keys");
    for lesson in syntax_lessons_for("python") {
        let path = root.join(format!("{}.py", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let result = judge_path(
            &root,
            &format!("{}-example", lesson.id),
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.passed,
            "{} worked example should teach the concept with different data, not pass the exercise case",
            lesson.id
        );
    }
}

#[test]
fn typescript_syntax_examples_run_under_node_strip_types() {
    let Some(node) = which("node") else {
        return;
    };
    let root = tmp_root("typescript-syntax-examples-run");
    for lesson in syntax_lessons_for("ts") {
        let path = root.join(format!("{}.ts", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let output = Command::new(&node)
            .arg("--experimental-strip-types")
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} example should exit successfully\nstdout:\n{}\nstderr:\n{}",
            lesson.id,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).trim().is_empty(),
            "{} example should not write stderr\n{}",
            lesson.id,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn typescript_syntax_examples_are_not_answer_keys() {
    if which("node").is_none() {
        return;
    }

    let root = tmp_root("typescript-syntax-examples-not-answer-keys");
    for lesson in syntax_lessons_for("ts") {
        let path = root.join(format!("{}.ts", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let result = judge_path(
            &root,
            &format!("{}-example", lesson.id),
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.passed,
            "{} worked example should teach the concept with different data, not pass the exercise case",
            lesson.id
        );
    }
}

#[test]
fn java_syntax_examples_run_cleanly_without_being_answer_keys() {
    if which("javac").is_none() || which("java").is_none() {
        return;
    }

    let root = tmp_root("java-syntax-examples-run");
    for lesson in syntax_lessons_for("java") {
        let path = root.join(format!("{}.java", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let result = judge_path(
            &root,
            &format!("{}-example", lesson.id),
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("compile failed") && !result.output.contains("Stderr"),
            "{} example should compile and run cleanly:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} worked example should teach the concept with different data, not pass the exercise case:\n{}",
            lesson.id, result.output
        );
    }
}

#[test]
fn rust_syntax_examples_run_cleanly_without_being_answer_keys() {
    if which("rustc").is_none() {
        return;
    }

    let root = tmp_root("rust-syntax-examples-run");
    for lesson in syntax_lessons_for("rust") {
        let path = root.join(format!("{}.rs", lesson.id));
        fs::write(&path, lesson.example).unwrap();
        let result = judge_path(
            &root,
            &format!("{}-example", lesson.id),
            &path,
            lesson.language,
            &syntax_cases(lesson),
        );
        assert!(
            !result.output.contains("compile failed") && !result.output.contains("Stderr"),
            "{} example should compile and run cleanly:\n{}",
            lesson.id,
            result.output
        );
        assert!(
            !result.passed,
            "{} worked example should teach the concept with different data, not pass the exercise case:\n{}",
            lesson.id, result.output
        );
    }
}

#[test]
fn syntax_exercise_starter_preserves_user_edit() {
    let root = tmp_root("syntax-exercise-preserve-user-edit");
    let lesson = syntax_lessons_for("python")
        .into_iter()
        .find(|lesson| lesson.id == "py-lists-dicts")
        .unwrap();
    let dir = root
        .join("submissions/.syntax")
        .join(lesson.language)
        .join(lesson.id);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("exercise.py");
    fs::write(&path, "nums = [2, 3]\nprint(0)\n").unwrap();

    let ensured = ensure_syntax_submission(&root, lesson).unwrap();

    assert_eq!(ensured, path);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "nums = [2, 3]\nprint(0)\n"
    );
}

#[test]
fn syntax_lessons_include_learning_scaffolding() {
    let state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings::default(),
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
    };
    for language in ["python", "ts", "java", "rust"] {
        for lesson in syntax_lessons_for(language) {
            let rendered = render_syntax_lesson(lesson, &state);
            assert!(
                rendered.contains("Concept") && rendered.contains("Exercise"),
                "{} is missing learning scaffolding",
                lesson.id
            );
            assert!(!lesson.refs.is_empty(), "{} has no references", lesson.id);
        }
    }
}

#[test]
fn ensure_syntax_submission_does_not_migrate_legacy_drill_file() {
    let root = tmp_root("syntax-exercise-no-migration");
    let lesson = syntax_lessons_for("python")[0];
    let dir = root
        .join("submissions/.syntax")
        .join(lesson.language)
        .join(lesson.id);
    fs::create_dir_all(&dir).unwrap();
    let legacy = dir.join("drill.py");
    fs::write(&legacy, "print('custom')\n").unwrap();

    let path = ensure_syntax_submission(&root, lesson).unwrap();

    assert_eq!(path, dir.join("exercise.py"));
    assert_eq!(fs::read_to_string(path).unwrap(), lesson.exercise.starter);
    assert_eq!(fs::read_to_string(legacy).unwrap(), "print('custom')\n");
}

#[test]
fn syntax_progress_count_is_separate_from_problem_progress() {
    let root = tmp_root("syntax-progress-count");
    let bank = load_bank(&root).unwrap();
    let mut state = AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings::default(),
        solved: vec!["001-hello-world".to_string()],
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
    };
    record_pass(&root, &bank[0], &mut state).unwrap();
    assert_eq!(syntax_progress_count(&state, "python").0, 0);
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
