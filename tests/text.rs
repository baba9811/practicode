use practicode::text::{compose_hangul_jamo, display_width, render_markdown_plain};

#[test]
fn compose_hangul_jamo_handles_korean_command_text() {
    assert_eq!(
        compose_hangul_jamo("ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"),
        "안녕하세요"
    );
    assert_eq!(
        compose_hangul_jamo("/next ㅎㅐㅅㅣ맵 쉬운 문제"),
        "/next 해시맵 쉬운 문제"
    );
    let mut value = String::new();
    for char in "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ".chars() {
        value.push(char);
        value = compose_hangul_jamo(&value);
    }
    assert_eq!(value, "안녕하세요");
    assert_eq!(compose_hangul_jamo("ㄳㅏ"), "ㄳㅏ");
}

#[test]
fn render_markdown_plain_preserves_fenced_code_body() {
    let rendered = render_markdown_plain("## Answer\n\n```python\n# keep comment\nprint('x')\n```");
    assert!(rendered.contains("Answer"));
    assert!(rendered.contains("# keep comment"));
    assert!(rendered.contains("print('x')"));
    assert!(!rendered.contains("```"));
}

#[test]
fn display_width_counts_hangul_as_wide() {
    assert_eq!(display_width("abc"), 3);
    assert_eq!(display_width("안녕"), 4);
}
