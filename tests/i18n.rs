use practicode::i18n::{UI_LANGUAGES, normalize_ui_language, ui_text};
use std::collections::HashMap;

#[test]
fn ui_catalogs_load_and_fallback_to_english() {
    for lang in UI_LANGUAGES {
        assert!(!ui_text(lang, "cmd_run").is_empty(), "{lang}");
        assert!(!ui_text(lang, "update_available").is_empty(), "{lang}");
    }
    assert_eq!(normalize_ui_language("zh-CN"), "zh");
    assert_eq!(ui_text("xx", "cmd_run"), "Judge the current submission");
}

#[test]
fn syntax_copy_exists_in_every_ui_catalog() {
    for lang in UI_LANGUAGES {
        let text = std::fs::read_to_string(format!("assets/i18n/{lang}.json")).unwrap();
        let catalog: HashMap<String, String> = serde_json::from_str(&text).unwrap();
        for key in [
            "syntax",
            "syntax_no_lesson",
            "syntax_practice",
            "syntax_usage",
            "syntax_progress",
            "syntax_lesson",
            "cmd_lesson",
        ] {
            assert!(
                catalog.get(key).is_some_and(|value| !value.is_empty()),
                "{lang} missing {key}"
            );
        }
    }
}
