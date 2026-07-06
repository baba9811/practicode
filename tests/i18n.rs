use practicode::core::{LANGUAGES, syntax_lessons_for};
use practicode::i18n::{UI_LANGUAGES, normalize_ui_language, ui_text};

#[test]
fn ui_catalogs_load_and_fallback_to_english() {
    for lang in UI_LANGUAGES {
        assert!(!ui_text(lang, "cmd_run").is_empty(), "{lang}");
        assert!(!ui_text(lang, "cmd_home").is_empty(), "{lang}");
        assert!(!ui_text(lang, "home_learn_choice").is_empty(), "{lang}");
        assert!(!ui_text(lang, "home_practice_choice").is_empty(), "{lang}");
        assert!(!ui_text(lang, "update_available").is_empty(), "{lang}");
    }
    assert_eq!(normalize_ui_language("zh-CN"), "zh");
    assert_eq!(ui_text("xx", "cmd_run"), "Judge the current submission");
}

#[test]
fn supported_ui_catalogs_cover_syntax_curriculum_copy() {
    for ui_language in ["ko", "ja", "zh", "es"] {
        for language in LANGUAGES {
            for lesson in syntax_lessons_for(language) {
                let id = lesson.id.replace('-', "_");
                let title_key = format!("syntax_{id}_title");
                let body_key = format!("syntax_{id}_body");
                assert!(
                    !ui_text(ui_language, &title_key).is_empty(),
                    "{ui_language}:{title_key}"
                );
                assert!(
                    !ui_text(ui_language, &body_key).is_empty(),
                    "{ui_language}:{body_key}"
                );
            }
        }
    }
}
