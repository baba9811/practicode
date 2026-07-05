mod common;

use common::{tmp_root, two_problem_bank};
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
fn codex_command_surface_is_replaced_by_ai() {
    let root = tmp_root("no-codex-command");
    let mut app = PracticodeApp::new(root).unwrap();
    app.handle_command_for_test("codex hint").unwrap();
    assert!(!app.has_task());
}
