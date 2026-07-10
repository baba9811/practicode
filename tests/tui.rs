mod common;

use common::{tmp_root, two_problem_bank};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use practicode::tui::{PracticodeApp, TextEditor};
use ratatui::{layout::Rect, style::Color};

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
fn text_editor_composes_jamo_input_on_current_line() {
    let mut editor = TextEditor::default();
    editor.set_text("");
    for char in "ㅇㅏㄴㄴㅕㅇ".chars() {
        editor.insert_char(char);
    }
    assert_eq!(editor.text(), "안녕");
}

#[test]
fn first_run_shows_home_once() {
    let root = tmp_root("first-run-home");
    let app = PracticodeApp::new(root.clone()).unwrap();
    assert!(app.status_text_for_test().contains("home"));
    assert!(app.output_for_test().contains("Learn syntax"));
    assert!(app.output_for_test().contains("Practice coding tests"));
    assert!(root.join("problem-state.json").exists());

    let app = PracticodeApp::new(root).unwrap();
    assert!(app.status_text_for_test().contains("home"));
}

#[test]
fn home_command_opens_home_and_persists_it() {
    let root = tmp_root("home-command");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("learn").unwrap();
    app.handle_command_for_test("home").unwrap();

    assert!(app.status_text_for_test().contains("home"));
    assert!(app.output_for_test().contains("Learn syntax"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"home\""));
}

#[test]
fn home_arrows_and_enter_open_selected_mode() {
    let root = tmp_root("home-keyboard-enter");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    assert!(!app.status_text_for_test().contains("| home |"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn home_vertical_arrows_and_enter_open_selected_mode() {
    let root = tmp_root("home-keyboard-down");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));

    let root = tmp_root("home-keyboard-up");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn home_space_opens_learn_mode() {
    let root = tmp_root("home-keyboard-space");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("learn"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"learn\""));
}

#[test]
fn home_command_escape_returns_to_home_focus() {
    let root = tmp_root("home-command-escape");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn home_output_escape_returns_to_home_focus() {
    let root = tmp_root("home-output-escape");
    let mut app = PracticodeApp::new(root.clone()).unwrap();

    app.handle_command_for_test("help").unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn clicking_home_area_returns_to_home_focus() {
    let root = tmp_root("home-click-focus");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.set_home_area_for_test(Rect::new(0, 0, 20, 10));
    app.set_home_choice_areas_for_test(Rect::new(0, 2, 20, 2), Rect::new(0, 5, 20, 2));
    app.set_pane_areas_for_test(
        Rect::default(),
        Rect::new(20, 0, 20, 10),
        Rect::new(0, 11, 40, 3),
    );

    app.handle_key_for_test(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE))
        .unwrap();
    app.handle_mouse_for_test(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 1,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn home_mouse_click_opens_clicked_choice() {
    let root = tmp_root("home-mouse-click");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.set_home_choice_areas_for_test(Rect::new(0, 2, 20, 3), Rect::new(22, 2, 20, 3));

    app.handle_mouse_for_test(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 25,
        row: 3,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();

    assert!(app.status_text_for_test().contains("001-hello-world"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"start_mode\": \"problems\""));
}

#[test]
fn app_start_resumes_learn_mode() {
    let root = tmp_root("resume-learn-mode");
    {
        let mut app = PracticodeApp::new(root.clone()).unwrap();
        app.handle_command_for_test("learn").unwrap();
    }

    let app = PracticodeApp::new(root).unwrap();
    assert!(app.status_text_for_test().contains("learn"));
    assert!(app.output_for_test().contains("Syntax"));
}

#[test]
fn app_start_resumes_problem_mode() {
    let root = tmp_root("resume-problem-mode");
    {
        let mut app = PracticodeApp::new(root.clone()).unwrap();
        app.handle_key_for_test(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
            .unwrap();
        app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .unwrap();
    }

    let app = PracticodeApp::new(root).unwrap();
    assert!(app.status_text_for_test().contains("001-hello-world"));
    assert!(!app.status_text_for_test().contains("| home |"));
}

#[test]
fn app_command_next_opens_local_problem_before_ai() {
    let root = tmp_root("app-next-local-first");
    two_problem_bank(&root);
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("next 해시맵 쉬운 문제")
        .unwrap();
    assert!(!app.has_task());
    assert!(app.status_text_for_test().contains("002-echo"));
}

#[test]
fn app_command_generate_request_starts_background_generation() {
    let root = tmp_root("app-generate-request");
    two_problem_bank(&root);
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("generate 해시맵 쉬운 문제")
        .unwrap();
    assert!(!app.has_task());
    assert!(app.has_background_generation_for_test());
    assert!(app.status_text_for_test().contains("bg generate"));
}

#[test]
fn next_fallback_generation_blocks_commands_but_keeps_warmup_active() {
    let root = tmp_root("busy-blocks-commands");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("next").unwrap();

    app.handle_command_for_test("language rust").unwrap();
    assert!(app.status_text_for_test().contains("python"));
    assert!(app.status_text_for_test().contains("Space warmup"));

    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.busy_attempts_for_test(), 1);
    assert!(app.has_task());
}

#[test]
fn next_fallback_generation_ignores_palette_and_mouse_editing() {
    let root = tmp_root("busy-ignores-palette-mouse");
    let mut app = PracticodeApp::new(root).unwrap();
    app.set_pane_areas_for_test(
        Rect::new(20, 0, 20, 10),
        Rect::new(20, 0, 20, 10),
        Rect::new(0, 11, 40, 3),
    );
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("next").unwrap();

    app.handle_key_for_test(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE))
        .unwrap();
    assert!(app.command_text().is_empty());
    app.handle_mouse_for_test(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 21,
        row: 1,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    assert!(app.status_text_for_test().contains("Space warmup"));
    assert!(app.has_task());
}

#[test]
fn background_generate_allows_solving_and_next_uses_local_problem() {
    let root = tmp_root("background-generate-next-local");
    two_problem_bank(&root);
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("generate 문자열").unwrap();
    app.handle_command_for_test("language rust").unwrap();
    assert!(app.status_text_for_test().contains("rust"));
    app.handle_command_for_test("next").unwrap();
    assert!(app.status_text_for_test().contains("002-echo"));
}

#[test]
fn next_waits_when_background_generate_is_running_and_no_local_problem_exists() {
    let root = tmp_root("background-generate-next-waits");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command sleep 1")
        .unwrap();
    app.handle_command_for_test("generate 문자열").unwrap();
    app.handle_command_for_test("next").unwrap();
    assert!(!app.has_task());
    assert!(app.has_background_generation_for_test());
    assert!(app.output_for_test().contains("background generation"));
}

#[test]
fn command_input_tracks_cursor_after_hangul_composition() {
    let root = tmp_root("command-cursor");
    let mut app = PracticodeApp::new(root).unwrap();
    app.focus_command_for_test();
    for char in "ㅇㅏㄴㄴㅕㅇ".chars() {
        app.insert_command_char_for_test(char);
    }
    assert_eq!(app.command_text(), "/안녕");
    assert_eq!(app.command_cursor(), 3);
}

#[test]
fn slash_command_palette_completes_prompt_commands() {
    let root = tmp_root("command-palette");
    let mut app = PracticodeApp::new(root).unwrap();
    app.focus_command_for_test();
    app.insert_command_char_for_test('h');
    app.insert_command_char_for_test('i');
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.command_text(), "/hint ");
}

#[test]
fn slash_command_palette_surfaces_problem_mode_commands() {
    let root = tmp_root("command-palette-problems");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    app.focus_command_for_test();
    let suggestions = app.command_suggestions_for_test();
    assert!(suggestions.contains(&"/run".to_string()));
    assert!(suggestions.contains(&"/next".to_string()));
    assert!(suggestions.contains(&"/back".to_string()));
    assert!(suggestions.contains(&"/problems".to_string()));
    assert!(suggestions.contains(&"/answer".to_string()));
    assert!(suggestions.contains(&"/hint <request>".to_string()));
    assert!(suggestions.contains(&"/generate <request>".to_string()));
    assert!(suggestions.contains(&"/profile".to_string()));
    assert!(suggestions.contains(&"/doctor".to_string()));
    assert!(suggestions.contains(&"/home".to_string()));
    assert!(!suggestions.contains(&"/drill".to_string()));
    assert!(!suggestions.contains(&"/next-lesson".to_string()));
    assert!(!suggestions.contains(&"/prev-lesson".to_string()));
    assert!(!suggestions.contains(&"/difficulty auto".to_string()));
    assert!(!suggestions.contains(&"/model auto".to_string()));
}

#[test]
fn home_command_palette_shows_entry_commands() {
    let root = tmp_root("command-palette-home");
    let mut app = PracticodeApp::new(root).unwrap();
    app.focus_command_for_test();
    let suggestions = app.command_suggestions_for_test();
    assert_eq!(suggestions[0], "/learn");
    assert!(suggestions.contains(&"/problems".to_string()));
    assert!(suggestions.contains(&"/doctor".to_string()));
    assert!(suggestions.contains(&"/profile".to_string()));
    assert!(suggestions.contains(&"/help".to_string()));
    assert!(suggestions.contains(&"/quit".to_string()));
    assert!(!suggestions.contains(&"/run".to_string()));
}

#[test]
fn learn_command_palette_uses_run_next_back_aliases() {
    let root = tmp_root("command-palette-learn-mode");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("learn python").unwrap();
    app.focus_command_for_test();
    let suggestions = app.command_suggestions_for_test();
    assert!(suggestions.contains(&"/run".to_string()));
    assert!(suggestions.contains(&"/next".to_string()));
    assert!(suggestions.contains(&"/back".to_string()));
    assert!(suggestions.contains(&"/ask <question>".to_string()));
    assert!(suggestions.contains(&"/doctor".to_string()));
    assert!(suggestions.contains(&"/home".to_string()));
    assert!(!suggestions.contains(&"/drill".to_string()));
    assert!(!suggestions.contains(&"/next-lesson".to_string()));
    assert!(!suggestions.contains(&"/prev-lesson".to_string()));
}

#[test]
fn typed_secondary_command_remains_discoverable() {
    let root = tmp_root("command-palette-typed-secondary");
    let mut app = PracticodeApp::new(root).unwrap();
    app.focus_command_for_test();
    for char in "model ".chars() {
        app.insert_command_char_for_test(char);
    }
    assert!(
        app.command_suggestions_for_test()
            .iter()
            .any(|command| command.starts_with("/model"))
    );
}

#[test]
fn next_and_back_are_mode_aware() {
    let root = tmp_root("mode-aware-next-back");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("learn python").unwrap();
    let first = app.output_for_test().to_string();

    app.handle_command_for_test("next").unwrap();
    let second = app.output_for_test().to_string();
    assert_ne!(first, second);

    app.handle_command_for_test("back").unwrap();
    assert_eq!(app.output_for_test(), first);
}

#[test]
fn learn_command_opens_syntax_course_separate_from_problem_mode() {
    let root = tmp_root("learn-command");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("learn").unwrap();
    assert!(app.output_for_test().contains("Syntax"));
    assert!(app.output_for_test().contains("Python"));
    assert!(app.status_text_for_test().contains("learn"));

    app.handle_command_for_test("problems").unwrap();
    assert!(app.status_text_for_test().contains("001-hello-world"));
    assert!(!app.status_text_for_test().contains("learn"));
}

#[test]
fn old_lesson_aliases_are_removed() {
    let root = tmp_root("old-lesson-aliases-removed");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("learn python").unwrap();
    for command in ["drill", "exercise", "next-lesson", "prev-lesson"] {
        app.handle_command_for_test(command).unwrap();
        assert!(app.output_for_test().contains("Unknown command"));
    }
    assert!(app.learn_result_for_test().is_empty());
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"python\": \"py-output\""));
    assert!(!saved.contains("\"syntax_progress\": {\n    \"python\""));
}

#[test]
fn run_in_learn_keeps_lesson_pane_visible() {
    let root = tmp_root("learn-run-keeps-lesson");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("learn python").unwrap();
    app.handle_command_for_test("run").unwrap();
    assert!(app.output_for_test().contains("Syntax"));
    assert!(app.learn_result_for_test().contains("FAIL"));
    assert!(app.learn_result_for_test().contains("Got\n  TODO"));
    assert!(app.status_text_for_test().contains("learn"));
}

#[test]
fn learn_command_uses_korean_syntax_copy() {
    let root = tmp_root("learn-korean-copy");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ui ko").unwrap();
    app.handle_command_for_test("learn python").unwrap();

    let output = app.output_for_test();
    assert!(output.contains("문법"));
    assert!(output.contains("언어"));
    assert!(output.contains("진도"));
    assert!(output.contains("출력"));
    assert!(!output.contains("# Syntax: Output"));
}

#[test]
fn learn_command_uses_supported_ui_language_syntax_copy() {
    let cases = [
        ("ko", "# 문법: print와 표준 출력", "표준 출력"),
        ("ja", "# 文法: print と標準出力", "`print` は"),
        ("zh", "# 语法: print 与标准输出", "`print` 是"),
        ("es", "# Sintaxis: print y stdout", "`print` es"),
    ];

    for (lang, title, body) in cases {
        let root = tmp_root(&format!("learn-{lang}-copy"));
        let mut app = PracticodeApp::new(root).unwrap();
        app.handle_command_for_test(&format!("ui {lang}")).unwrap();
        app.handle_command_for_test("learn python").unwrap();

        let output = app.output_for_test();
        assert!(output.contains(title), "{lang}: {output}");
        assert!(output.contains(body), "{lang}: {output}");
        assert!(!output.contains("# Syntax: Output"), "{lang}: {output}");
    }
}

#[test]
fn profile_commands_update_saved_preferences() {
    let root = tmp_root("profile-commands");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("difficulty medium").unwrap();
    app.handle_command_for_test("topics arrays, strings, arrays")
        .unwrap();
    app.handle_command_for_test("avoid dp, graph").unwrap();
    app.handle_command_for_test("generate-languages python, rust")
        .unwrap();
    app.handle_command_for_test("generate-ui ko, en").unwrap();
    app.handle_command_for_test("profile").unwrap();
    let output = app.output_for_test();
    assert!(output.contains("Difficulty: medium"));
    assert!(output.contains("Preferred topics: arrays, strings"));
    assert!(output.contains("Avoid topics: dp, graph"));
    assert!(output.contains("Generated answer languages: python, rust"));
    assert!(output.contains("Generated UI languages: ko, en"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"difficulty\": \"medium\""));
    assert!(saved.contains("\"topics\": ["));
    assert!(saved.contains("\"avoid_topics\": ["));
    assert!(saved.contains("\"generate_languages\": ["));
    assert!(saved.contains("\"generate_ui_languages\": ["));
}

#[test]
fn profile_panel_uses_korean_user_profile_copy() {
    let root = tmp_root("profile-korean-copy");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ui ko").unwrap();
    app.handle_command_for_test("profile").unwrap();

    let output = app.output_for_test();
    assert!(output.contains("사용자 프로필"));
    assert!(output.contains("생성 정답 언어"));
    assert!(!output.contains("연습 프로파일"));
}

#[test]
fn profile_panel_toggles_generation_languages_with_keyboard() {
    let root = tmp_root("profile-keyboard-toggles");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("profile").unwrap();
    for _ in 0..8 {
        app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .unwrap();
    }
    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();

    let output = app.output_for_test();
    assert!(output.contains("[ ] python"));
    assert!(output.contains("Generated answer languages: ts, java, rust"));
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("problem-state.json")).unwrap())
            .unwrap();
    let languages = saved["settings"]["generate_languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(languages, vec!["ts", "java", "rust"]);
}

#[test]
fn profile_panel_cycles_ai_settings_with_keyboard() {
    let root = tmp_root("profile-ai-settings");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("profile").unwrap();
    for _ in 0..4 {
        app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .unwrap();
    }
    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();
    app.set_available_models_for_test(vec!["claude-test"]);
    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();

    let output = app.output_for_test();
    assert!(output.contains("AI provider: claude"));
    assert!(output.contains("AI model: claude-test"));
    assert!(output.contains("AI effort: low"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_provider\": \"claude\""));
    assert!(saved.contains("\"ai_model\": \"claude-test\""));
    assert!(saved.contains("\"ai_effort\": \"low\""));
}

#[test]
fn profile_panel_opens_problem_notes_editor() {
    let root = tmp_root("profile-notes-editor");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("profile").unwrap();
    for _ in 0..7 {
        app.handle_key_for_test(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .unwrap();
    }
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    for char in "Prefer strings".chars() {
        app.handle_key_for_test(KeyEvent::new(KeyCode::Char(char), KeyModifiers::NONE))
            .unwrap();
    }
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    for char in "Avoid DP".chars() {
        app.handle_key_for_test(KeyEvent::new(KeyCode::Char(char), KeyModifiers::NONE))
            .unwrap();
    }
    app.handle_key_for_test(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(root.join("problem_notes.md")).unwrap(),
        "Prefer strings\nAvoid DP"
    );
    assert!(app.output_for_test().contains("User profile"));
}

#[test]
fn slash_command_palette_uses_provider_models_when_available() {
    let root = tmp_root("command-palette-models");
    let mut app = PracticodeApp::new(root).unwrap();
    app.set_available_models_for_test(vec!["provider-model"]);
    app.focus_command_for_test();
    for char in "model ".chars() {
        app.insert_command_char_for_test(char);
    }
    assert!(
        app.command_suggestions_for_test()
            .contains(&"/model provider-model".to_string())
    );
}

#[test]
fn model_command_explains_unavailable_provider_models() {
    let root = tmp_root("model-status");
    let mut app = PracticodeApp::new(root).unwrap();
    app.set_model_message_for_test("provider does not expose model list");
    app.handle_command_for_test("model").unwrap();
    assert!(app.output_for_test().contains("AI provider:"));
    assert!(app.output_for_test().contains("AI effort:"));
    assert!(
        app.output_for_test()
            .contains("provider does not expose model list")
    );
    assert!(app.output_for_test().contains("/model <name>"));
}

#[test]
fn doctor_command_reports_runtime_checks() {
    let root = tmp_root("doctor-command");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("doctor").unwrap();

    assert!(app.output_for_test().contains("Doctor"));
    assert!(app.output_for_test().contains("Runtime checks"));
    assert!(app.output_for_test().contains("Python"));
    assert!(app.output_for_test().contains("TypeScript"));
}

#[test]
fn doctor_command_is_localized() {
    let root = tmp_root("doctor-localized");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ui ko").unwrap();
    app.handle_command_for_test("doctor").unwrap();

    assert!(app.output_for_test().contains("환경 진단"));
    assert!(app.output_for_test().contains("런타임 확인"));
}

#[test]
fn effort_command_updates_saved_ai_effort() {
    let root = tmp_root("effort-command");
    let mut app = PracticodeApp::new(root.clone()).unwrap();
    app.handle_command_for_test("effort high").unwrap();

    assert!(app.output_for_test().contains("AI effort: high"));
    let saved = std::fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_effort\": \"high\""));
}

#[test]
fn focused_pane_title_has_text_indicator() {
    assert_eq!(
        PracticodeApp::pane_title_for_test("Command", true),
        "> Command"
    );
    assert_eq!(
        PracticodeApp::pane_title_for_test("Command", false),
        "Command"
    );
}

#[test]
fn pane_styles_fill_light_and_dark_backgrounds() {
    let light = PracticodeApp::pane_style_for_test(true);
    assert_eq!(light.bg, Some(Color::Rgb(248, 250, 252)));
    assert_eq!(light.fg, Some(Color::Rgb(17, 24, 39)));

    let dark = PracticodeApp::pane_style_for_test(false);
    assert_eq!(dark.bg, Some(Color::Rgb(17, 24, 39)));
    assert_eq!(dark.fg, Some(Color::Rgb(229, 231, 235)));
}

#[test]
fn codex_command_surface_is_replaced_by_ai() {
    let root = tmp_root("no-codex-command");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("codex hint").unwrap();
    assert!(!app.has_task());
}

#[test]
fn status_text_hides_internal_problem_source() {
    let root = tmp_root("status-source");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("source local").unwrap();
    let status = app.status_text_for_test();
    assert!(!status.contains("bank"));
    assert!(!status.contains("next:"));
}

#[test]
fn clicking_output_keeps_output_for_copying() {
    let root = tmp_root("mouse-output-edit");
    let mut app = PracticodeApp::new(root).unwrap();
    app.set_pane_areas_for_test(
        Rect::new(20, 0, 20, 10),
        Rect::new(20, 0, 20, 10),
        Rect::new(0, 11, 40, 3),
    );
    app.handle_command_for_test("help").unwrap();
    app.handle_mouse_for_test(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 21,
        row: 1,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    assert!(app.status_text_for_test().contains("drag select to copy"));
    assert!(!app.wants_mouse_capture_for_test());
}

#[test]
fn clicking_visible_code_editor_focuses_editor() {
    let root = tmp_root("mouse-code-edit");
    let mut app = PracticodeApp::new(root).unwrap();
    app.set_pane_areas_for_test(
        Rect::new(20, 0, 20, 10),
        Rect::new(20, 0, 20, 10),
        Rect::new(0, 11, 40, 3),
    );
    app.handle_command_for_test("code").unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();
    app.handle_mouse_for_test(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 21,
        row: 1,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    assert!(app.status_text_for_test().contains("/run judge"));
    assert!(app.wants_mouse_capture_for_test());
}

#[test]
fn ctrl_c_quits_from_editor() {
    let root = tmp_root("ctrl-c-quit");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_key_for_test(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
        .unwrap();
    assert!(app.should_quit_for_test());
}
