use practicode::core::{LANGUAGES, syntax_lessons_for};
use practicode::i18n::{UI_LANGUAGES, normalize_ui_language, ui_text};
use practicode::text::display_width;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    process::Command,
};

fn lesson_asset_dir(language: &str) -> &str {
    match language {
        "ts" => "typescript",
        _ => language,
    }
}

fn assert_no_english_scaffolding_terms(ui_language: &str, lesson_id: &str, text: &str) {
    if ui_language == "en" {
        return;
    }
    let text = text.to_ascii_lowercase();
    for term in ["worked example", "starter"] {
        assert!(
            !text.contains(term),
            "{ui_language}:{lesson_id} contains untranslated scaffolding term: {term}"
        );
    }
}

fn assert_not_generic_lesson_copy(ui_language: &str, lesson_id: &str, text: &str) {
    for phrase in [
        "is easiest to learn by tracing the value flow",
        "with the smallest code shape",
        "Memorizing the example instead of explaining the value flow",
        "What value exists immediately before",
        "문법 이름을 외우기보다 예제의 값 흐름",
        "가장 작은 관찰 가능한 코드",
        "用語を暗記するよりも",
        "最小限のコードで示しています",
        "不要只记语法名称",
        "最小但可观察的代码",
        "se aprende mejor siguiendo el recorrido del valor",
        "con el código mínimo que produce un resultado observable",
    ] {
        assert!(
            !text.contains(phrase),
            "{ui_language}:{lesson_id} contains generic lesson-copy phrase: {phrase}"
        );
    }
}

fn assert_no_unformatted_english_terms(ui_language: &str, lesson_id: &str, text: &str) {
    if !["ko", "ja", "zh"].contains(&ui_language) {
        return;
    }
    let prose = text.split('`').step_by(2).collect::<Vec<_>>().join(" ");
    let banned = [
        "alias",
        "annotation",
        "array",
        "assertion",
        "await",
        "awaitable",
        "branch",
        "callback",
        "capstone",
        "case",
        "casting",
        "catch",
        "chain",
        "check",
        "checker",
        "class",
        "coalescing",
        "command",
        "compile",
        "compiler",
        "context",
        "coercion",
        "control",
        "data",
        "dataclass",
        "default",
        "demo",
        "dispatch",
        "editor",
        "eager",
        "error",
        "executor",
        "exception",
        "fallback",
        "feature",
        "field",
        "flow",
        "frontier",
        "function",
        "future",
        "generic",
        "generator",
        "handle",
        "hint",
        "import",
        "indexed",
        "instance",
        "interface",
        "introspection",
        "interpreter",
        "iterable",
        "iterator",
        "key",
        "label",
        "literal",
        "loop",
        "materialize",
        "mapped",
        "member",
        "membership",
        "metadata",
        "method",
        "model",
        "mutable",
        "narrowing",
        "object",
        "one-poll",
        "overload",
        "parser",
        "patch",
        "path",
        "payload",
        "predicate",
        "prefix",
        "promise",
        "property",
        "protocol",
        "readonly",
        "record",
        "rebinding",
        "result",
        "route",
        "runtime",
        "starter",
        "static",
        "stdin",
        "stdout",
        "stem",
        "strict",
        "string",
        "sorting",
        "surrogate",
        "text",
        "todo",
        "token",
        "toolchain",
        "tuple",
        "type",
        "unit",
        "union",
        "validation",
        "variant",
        "worker",
        "wrapper",
        "yield",
    ];
    for word in prose
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
        .filter(|word| !word.is_empty())
    {
        assert!(
            !banned.contains(&word.to_ascii_lowercase().as_str()),
            "{ui_language}:{lesson_id} contains unformatted English prose term: {word}"
        );
    }
}

fn normalized_character_ngrams(
    copy: &serde_json::Map<String, Value>,
    width: usize,
) -> HashSet<String> {
    let mut prose = String::new();
    for value in copy.values() {
        match value {
            Value::String(text) => {
                prose.push_str(text);
                prose.push(' ');
            }
            Value::Array(items) => {
                for item in items {
                    if let Some(text) = item.as_str() {
                        prose.push_str(text);
                        prose.push(' ');
                    }
                }
            }
            _ => {}
        }
    }
    let characters = prose
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect::<Vec<_>>();
    characters
        .windows(width)
        .map(|window| window.iter().collect())
        .collect()
}

fn normalized_word_ngrams(copy: &serde_json::Map<String, Value>, width: usize) -> HashSet<String> {
    let mut words = Vec::new();
    for value in copy.values() {
        let texts = match value {
            Value::String(text) => vec![text.as_str()],
            Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        for text in texts {
            words.extend(
                text.split(|character: char| !character.is_alphanumeric())
                    .filter(|word| !word.is_empty())
                    .map(str::to_lowercase),
            );
        }
    }
    words
        .windows(width)
        .map(|window| window.join(" "))
        .collect()
}

#[test]
fn ui_catalogs_load_and_fallback_to_english() {
    for lang in UI_LANGUAGES {
        assert!(!ui_text(lang, "cmd_run").is_empty(), "{lang}");
        assert!(!ui_text(lang, "cmd_home").is_empty(), "{lang}");
        assert!(!ui_text(lang, "cmd_doctor").is_empty(), "{lang}");
        assert!(!ui_text(lang, "home_learn_choice").is_empty(), "{lang}");
        assert!(!ui_text(lang, "home_practice_choice").is_empty(), "{lang}");
        assert!(!ui_text(lang, "update_available").is_empty(), "{lang}");
    }
    assert_eq!(normalize_ui_language("zh-CN"), "zh");
    assert_eq!(ui_text("xx", "cmd_run"), "Judge the current submission");
}

#[test]
fn ui_catalogs_have_complete_localized_learning_ui_copy() {
    let required = [
        "learning_step_review",
        "learning_step_delta",
        "learning_step_predict",
        "learning_step_exercise",
        "learning_step_reflect",
        "learning_step_complete",
        "learning_due_reviews",
        "mastery_new",
        "mastery_practiced",
        "mastery_retained",
        "mastery_mastered",
        "judge_failure_compile",
        "judge_failure_typecheck",
        "judge_failure_runtime",
        "judge_failure_timeout",
        "judge_failure_output",
        "judge_case",
        "judge_input",
        "judge_expected",
        "judge_got",
        "judge_stdout",
        "judge_stderr",
        "judge_error",
        "judge_compile",
        "judge_hidden",
        "judge_timeout_detail",
        "home_learn_choice",
        "learning_view_lesson",
        "learning_view_code",
        "learning_view_result",
        "hint_result",
        "resize_required",
        "learning_shortcuts",
        "focus_active",
        "result_pass",
        "result_fail",
        "result_empty",
        "help_home_loop",
        "help_learn_loop",
        "help_problem_loop",
        "help_palette_open",
        "help_palette_move",
        "help_palette_close",
        "help_stdout",
        "help_stderr",
        "empty_value",
        "list_closed",
        "result_mastery",
        "result_review_days",
        "busy_ai_thinking",
        "elapsed_seconds",
        "generation_started",
        "generation_duplicate",
        "generation_generated",
        "generation_failed",
        "generation_finished",
        "generation_reload_failed",
        "generation_partial_count",
        "judge_unknown_status",
        "judge_missing_typescript_tool",
        "hint_learn_compact",
        "hint_problem_compact",
        "hint_home_compact",
        "pane_exercise",
        "pane_solution",
        "settings_title",
        "settings_instructions",
        "settings_code_language",
        "settings_ui_language",
        "settings_theme",
        "settings_difficulty",
        "settings_preferred_topics",
        "settings_avoid_topics",
        "settings_generated_answer_languages",
        "settings_generated_ui_languages",
        "settings_provider_default",
        "settings_problem_notes",
        "settings_answer_toggles",
        "settings_ui_toggles",
        "settings_commands",
        "settings_none",
        "settings_all",
        "settings_ai_provider",
        "settings_ai_model",
        "settings_ai_effort",
        "settings_model_loading",
        "settings_model_load_hint",
        "settings_note_action",
        "model_use_default_model",
        "model_use_default_effort",
        "model_loading",
        "model_unavailable",
        "model_custom_hint",
        "model_available_efforts",
        "model_available_models",
        "note_saved",
        "notes_empty",
        "notes_title",
        "first_problem",
        "answer_for_language",
        "learn_usage",
        "ui_language_set",
        "theme_set",
        "difficulty_options",
        "practice_shortcuts",
        "problem_list_title",
        "problem_list_id",
        "problem_list_difficulty",
        "problem_list_status",
        "problem_list_code",
        "problem_list_name",
        "problem_list_hint",
        "problem_not_found",
        "update_checking",
        "ai_next_command_saved",
        "unknown_command",
        "next_unavailable",
        "next_failed",
        "provider_cli_found",
        "provider_cli_missing",
        "provider_codex_daemon_available",
        "provider_codex_direct_fallback",
        "model_cli_missing",
        "model_claude_presets",
        "model_codex_daemon_unavailable",
        "model_codex_query_failed",
        "model_codex_empty",
        "doctor_missing_tool",
        "doctor_node_required",
        "doctor_tsc_unreadable",
        "doctor_tsc_required",
        "doctor_unknown_version",
        "doctor_node_install_linux",
        "doctor_codex_install",
        "doctor_claude_install",
        "ai_context_disclosure",
    ];
    let catalogs = UI_LANGUAGES
        .iter()
        .map(|lang| {
            let text = fs::read_to_string(format!("assets/i18n/{lang}.json")).unwrap();
            (
                *lang,
                serde_json::from_str::<HashMap<String, String>>(&text).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let english_keys = catalogs[0]
        .1
        .keys()
        .collect::<std::collections::HashSet<_>>();

    for (lang, catalog) in &catalogs {
        assert_eq!(
            catalog.keys().collect::<std::collections::HashSet<_>>(),
            english_keys,
            "{lang}: UI catalog key mismatch"
        );
        for key in required {
            let value = catalog
                .get(key)
                .unwrap_or_else(|| panic!("{lang}: missing {key}"));
            assert!(!value.trim().is_empty(), "{lang}:{key}");
        }
    }

    for (lang, catalog) in catalogs.iter().skip(1) {
        for key in [
            "resize_required",
            "result_pass",
            "result_fail",
            "result_empty",
            "help_home_loop",
            "help_learn_loop",
            "help_problem_loop",
            "provider_cli_found",
            "provider_cli_missing",
            "provider_codex_daemon_available",
            "provider_codex_direct_fallback",
            "model_cli_missing",
            "model_claude_presets",
            "model_codex_daemon_unavailable",
            "model_codex_query_failed",
            "model_codex_empty",
            "doctor_missing_tool",
            "doctor_node_required",
            "doctor_tsc_unreadable",
            "doctor_tsc_required",
            "doctor_unknown_version",
            "doctor_node_install_linux",
            "doctor_codex_install",
            "doctor_claude_install",
        ] {
            assert_ne!(
                catalog[key], catalogs[0].1[key],
                "{lang}:{key} leaked English"
            );
        }
    }
}

#[test]
fn compact_hints_fit_sixty_columns_and_keep_essential_shortcuts() {
    for lang in UI_LANGUAGES {
        let learn = ui_text(lang, "hint_learn_compact");
        for token in ["/next", "F5", "F6", "F1"] {
            assert!(
                learn.contains(token),
                "{lang}:hint_learn_compact needs {token}"
            );
        }
        let problem = ui_text(lang, "hint_problem_compact");
        for token in ["F6", "/run", "/next", "F1"] {
            assert!(
                problem.contains(token),
                "{lang}:hint_problem_compact needs {token}"
            );
        }
        for key in [
            "hint_learn_compact",
            "hint_problem_compact",
            "hint_home_compact",
        ] {
            let value = ui_text(lang, key);
            assert!(
                display_width(value) <= 60,
                "{lang}:{key} is {} columns: {value}",
                display_width(value)
            );
        }
    }
}

#[test]
fn practice_shortcuts_keep_the_problem_loop_commands() {
    for lang in UI_LANGUAGES {
        let value = ui_text(lang, "practice_shortcuts");
        for token in ["F1", "F6", "/run", "/next"] {
            assert!(
                value.contains(token),
                "{lang}:practice_shortcuts needs {token}"
            );
        }
    }
}

#[test]
fn ai_palette_copy_discloses_the_context_sent_to_the_selected_provider() {
    let markers = [
        (
            "en",
            ["problem/lesson", "code", "result/context", "provider"],
        ),
        ("ko", ["문제/레슨", "코드", "결과/맥락", "제공자"]),
        (
            "ja",
            ["問題/レッスン", "コード", "結果/文脈", "プロバイダー"],
        ),
        ("zh", ["题目/课程", "代码", "结果/上下文", "提供商"]),
        (
            "es",
            [
                "problema/lección",
                "código",
                "resultado/contexto",
                "proveedor",
            ],
        ),
    ];
    for (lang, expected) in markers {
        for key in ["cmd_hint", "cmd_ask", "cmd_ai"] {
            let value = ui_text(lang, key);
            for marker in expected {
                assert!(
                    value.contains(marker),
                    "{lang}:{key} needs {marker}: {value}"
                );
            }
        }
    }
}

#[test]
fn compact_ai_disclosure_fits_the_palette_and_names_the_sent_context() {
    for lang in UI_LANGUAGES {
        let value = ui_text(lang, "ai_context_disclosure");
        assert!(!value.trim().is_empty(), "{lang}");
        assert!(
            display_width(value) <= 58,
            "{lang}: disclosure is {} columns: {value}",
            display_width(value)
        );
    }
}

#[test]
fn ui_catalogs_reject_known_english_scaffolding_and_spanish_misspellings() {
    for lang in ["ko", "ja", "zh", "es"] {
        let text = fs::read_to_string(format!("assets/i18n/{lang}.json")).unwrap();
        let catalog = serde_json::from_str::<HashMap<String, String>>(&text).unwrap();
        for phrase in [
            "Choose Learn or Practice",
            "No result yet.",
            "Resize the terminal",
            "Retry this exercise",
        ] {
            assert!(
                catalog.values().all(|value| !value.contains(phrase)),
                "{lang}: UI catalog contains English scaffolding: {phrase}"
            );
        }
    }

    let spanish_catalog = serde_json::from_str::<HashMap<String, String>>(
        &fs::read_to_string("assets/i18n/es.json").unwrap(),
    )
    .unwrap();
    let spanish = spanish_catalog
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let mut spanish_prose = String::with_capacity(spanish.len());
    let mut inside_placeholder = false;
    for character in spanish.chars() {
        match character {
            '{' => inside_placeholder = true,
            '}' => inside_placeholder = false,
            _ if !inside_placeholder => spanish_prose.push(character),
            _ => {}
        }
    }
    let spanish_words = spanish_prose
        .split(|character: char| !character.is_alphabetic())
        .collect::<std::collections::HashSet<_>>();
    for misspelling in [
        "actualizacion",
        "codigo",
        "numero",
        "version",
        "Diagnostico",
        "generacion",
        "solucion",
        "Evalua",
        "depuracion",
    ] {
        assert!(
            !spanish_words.contains(misspelling),
            "es UI contains {misspelling}"
        );
    }
}

#[test]
fn ui_catalogs_do_not_store_syntax_curriculum_copy() {
    for ui_language in UI_LANGUAGES {
        for language in LANGUAGES {
            for lesson in syntax_lessons_for(language) {
                let id = lesson.id.replace('-', "_");
                assert!(
                    ui_text(ui_language, &format!("syntax_{id}_title")).is_empty(),
                    "{ui_language}:{id}:title"
                );
                assert!(
                    ui_text(ui_language, &format!("syntax_{id}_body")).is_empty(),
                    "{ui_language}:{id}:body"
                );
            }
        }
    }
}

#[test]
fn lesson_catalogs_have_complete_study_copy_for_every_language() {
    for &ui_language in UI_LANGUAGES {
        let legacy_path = format!("assets/lessons/{ui_language}.json");
        assert!(
            !Path::new(&legacy_path).exists(),
            "legacy lesson catalog should be removed: {legacy_path}"
        );
    }

    for &ui_language in UI_LANGUAGES {
        for &language in LANGUAGES {
            let path = format!(
                "assets/lessons/{}/{ui_language}.json",
                lesson_asset_dir(language)
            );
            let catalog: Value =
                serde_json::from_str(&fs::read_to_string(&path).unwrap()).expect(&path);
            let catalog_object = catalog
                .as_object()
                .unwrap_or_else(|| panic!("{path}: catalog should be an object"));
            for key in catalog_object.keys() {
                assert!(
                    [
                        "schema_version",
                        "programming_language",
                        "ui_language",
                        "lessons"
                    ]
                    .contains(&key.as_str()),
                    "{path}: unexpected top-level key {key}"
                );
            }
            assert_eq!(
                catalog
                    .get("schema_version")
                    .and_then(Value::as_u64)
                    .unwrap_or_default(),
                1,
                "{path}: schema_version"
            );
            assert_eq!(
                catalog
                    .get("programming_language")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                language,
                "{path}: programming_language"
            );
            assert_eq!(
                catalog
                    .get("ui_language")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                ui_language,
                "{path}: ui_language"
            );
            let lessons = catalog
                .get("lessons")
                .and_then(Value::as_object)
                .unwrap_or_else(|| panic!("{path}: missing lessons object"));
            assert_eq!(
                lessons.len(),
                syntax_lessons_for(language).len(),
                "{path}: unexpected lesson count"
            );
            for lesson in syntax_lessons_for(language) {
                let copy = lessons.get(lesson.id).unwrap_or_else(|| {
                    panic!("{ui_language}: missing lesson copy for {}", lesson.id)
                });
                let copy_object = copy.as_object().unwrap_or_else(|| {
                    panic!("{ui_language}:{} copy should be an object", lesson.id)
                });
                for key in copy_object.keys() {
                    assert!(
                        [
                            "title",
                            "concept",
                            "worked_example",
                            "common_mistakes",
                            "self_check",
                            "exercise_prompt",
                            "objective",
                            "language_delta",
                            "prediction_prompt",
                            "transfer_trap",
                        ]
                        .contains(&key.as_str()),
                        "{ui_language}:{} unexpected lesson-copy key {key}",
                        lesson.id
                    );
                }
                let minimum_prose_length = if ["ko", "ja", "zh"].contains(&ui_language) {
                    30
                } else {
                    45
                };
                for field in ["title", "concept", "worked_example", "exercise_prompt"] {
                    let text = copy
                        .get(field)
                        .and_then(Value::as_str)
                        .unwrap_or_else(|| panic!("{ui_language}:{} missing {field}", lesson.id));
                    assert!(
                        text.chars().count() >= minimum_prose_length || field == "title",
                        "{ui_language}:{} {field} too short: {text}",
                        lesson.id
                    );
                    assert_no_english_scaffolding_terms(ui_language, lesson.id, text);
                    assert_not_generic_lesson_copy(ui_language, lesson.id, text);
                }
                for field in ["common_mistakes", "self_check"] {
                    let items = copy
                        .get(field)
                        .and_then(Value::as_array)
                        .unwrap_or_else(|| panic!("{ui_language}:{} missing {field}", lesson.id));
                    assert!(
                        items.len() >= 2,
                        "{ui_language}:{} {field} needs at least 2 items",
                        lesson.id
                    );
                    for item in items {
                        let text = item
                            .as_str()
                            .unwrap_or_else(|| panic!("{ui_language}:{} bad {field}", lesson.id));
                        assert!(
                            text.chars().count() >= 12,
                            "{ui_language}:{} {field} item too short: {text}",
                            lesson.id
                        );
                        assert_no_english_scaffolding_terms(ui_language, lesson.id, text);
                        assert_not_generic_lesson_copy(ui_language, lesson.id, text);
                    }
                }
                if ["en", "ko", "ja", "zh", "es"].contains(&ui_language) {
                    for field in [
                        "objective",
                        "language_delta",
                        "prediction_prompt",
                        "transfer_trap",
                    ] {
                        let text = copy.get(field).and_then(Value::as_str).unwrap_or_else(|| {
                            panic!("{ui_language}:{} missing {field}", lesson.id)
                        });
                        assert!(
                            text.chars().count() >= 20,
                            "{ui_language}:{} {field} too short: {text}",
                            lesson.id
                        );
                        assert_no_english_scaffolding_terms(ui_language, lesson.id, text);
                        assert_not_generic_lesson_copy(ui_language, lesson.id, text);
                    }
                    assert_ne!(
                        copy["objective"], copy["concept"],
                        "{ui_language}:{} objective repeats concept",
                        lesson.id
                    );
                    assert_ne!(
                        copy["language_delta"], copy["concept"],
                        "{ui_language}:{} language_delta repeats concept",
                        lesson.id
                    );
                    assert_ne!(
                        copy["prediction_prompt"], copy["exercise_prompt"],
                        "{ui_language}:{} prediction repeats exercise",
                        lesson.id
                    );
                    for value in copy_object.values() {
                        let texts = match value {
                            Value::String(text) => vec![text.as_str()],
                            Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
                            _ => Vec::new(),
                        };
                        for text in texts {
                            assert!(
                                !text.to_ascii_lowercase().contains("example:"),
                                "{ui_language}:{} references a nonexistent example: label",
                                lesson.id
                            );
                            if ["ko", "ja", "zh"].contains(&ui_language) {
                                assert!(
                                    !text.to_ascii_lowercase().contains("judge"),
                                    "{ui_language}:{} contains untranslated judge prose",
                                    lesson.id
                                );
                                assert_no_unformatted_english_terms(ui_language, lesson.id, text);
                            }
                            if ui_language == "ko" {
                                assert!(
                                    !text.contains("흐름 흐름"),
                                    "ko:{} contains duplicated 흐름",
                                    lesson.id
                                );
                            }
                        }
                    }
                }
            }

            let mut repeated = HashMap::<String, usize>::new();
            for copy in lessons.values() {
                let copy = copy.as_object().expect("lesson copy object");
                for value in copy.values() {
                    match value {
                        Value::String(text) => {
                            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                            if text.chars().count() >= 45 {
                                *repeated.entry(text).or_default() += 1;
                            }
                        }
                        Value::Array(items) => {
                            for item in items {
                                let text = item
                                    .as_str()
                                    .expect("lesson list item string")
                                    .split_whitespace()
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if text.chars().count() >= 45 {
                                    *repeated.entry(text).or_default() += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            for (text, count) in repeated {
                assert!(
                    count <= 3,
                    "{path}: repeated lesson copy {count} times: {text}"
                );
            }
            if ["ko", "ja", "zh"].contains(&ui_language) {
                let mut frequencies = HashMap::<String, usize>::new();
                for copy in lessons.values() {
                    for gram in normalized_character_ngrams(
                        copy.as_object().expect("lesson copy object"),
                        24,
                    ) {
                        *frequencies.entry(gram).or_default() += 1;
                    }
                }
                let mut repeated = frequencies
                    .into_iter()
                    .filter(|(_, count)| count * 5 > lessons.len())
                    .collect::<Vec<_>>();
                repeated
                    .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
                repeated.truncate(3);
                assert!(
                    repeated.is_empty(),
                    "{path}: repeated 24-character lesson templates: {repeated:?}"
                );
            }
            if ui_language == "es" {
                let mut frequencies = HashMap::<String, usize>::new();
                for copy in lessons.values() {
                    for gram in
                        normalized_word_ngrams(copy.as_object().expect("lesson copy object"), 8)
                    {
                        *frequencies.entry(gram).or_default() += 1;
                    }
                }
                let mut repeated = frequencies
                    .into_iter()
                    .filter(|(_, count)| count * 5 > lessons.len())
                    .collect::<Vec<_>>();
                repeated
                    .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
                repeated.truncate(3);
                assert!(
                    repeated.is_empty(),
                    "{path}: repeated 8-word lesson templates: {repeated:?}"
                );
            }
        }
    }
}

#[test]
fn review_manifest_covers_every_final_lesson_catalog_hash() {
    let output = Command::new("node")
        .arg("scripts/check-lessons.js")
        .output()
        .expect("run lesson review manifest checker");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("20 catalogs, 550 records"),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
