mod common;

use common::{tmp_root, two_problem_bank};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use practicode::tui::{PracticodeApp, TextEditor};

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
fn app_command_next_request_starts_forced_ai_task() {
    let root = tmp_root("app-next-request");
    two_problem_bank(&root);
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("ai-next-command true").unwrap();
    app.handle_command_for_test("next 해시맵 쉬운 문제")
        .unwrap();
    assert!(app.has_task());
    assert_eq!(app.busy_label(), "next");
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
    app.handle_key_for_test(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.command_text(), "/hint ");
}

#[test]
fn slash_command_palette_surfaces_settings_commands() {
    let root = tmp_root("command-palette-settings");
    let mut app = PracticodeApp::new(root).unwrap();
    app.focus_command_for_test();
    let suggestions = app.command_suggestions_for_test();
    assert!(suggestions.contains(&"/provider codex".to_string()));
    assert!(suggestions.contains(&"/model auto".to_string()));
    assert!(suggestions.contains(&"/hint <request>".to_string()));
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
    assert!(
        app.output_for_test()
            .contains("provider does not expose model list")
    );
    assert!(app.output_for_test().contains("/model <name>"));
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
