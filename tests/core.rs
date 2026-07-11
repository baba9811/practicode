mod common;

use common::{tmp_root, two_problem_bank};
use practicode::{
    core::{
        AppState, HistoryItem, JudgeFailureKind, LANGUAGES, LessonMastery, MasteryStage, Settings,
        SyntaxKind, SyntaxTrack, due_syntax_lessons, ensure_submission, ensure_syntax_submission,
        judge, judge_path, load_bank, load_state, localized, migrate_syntax_mastery, next_problem,
        normalize_judge_output, parse_language_list, parse_ui_language_list, problem_by_id,
        record_pass, record_syntax_pass, record_syntax_result, record_syntax_test_out,
        render_problem, render_problem_tui, render_syntax_lesson, save_bank, save_state,
        syntax_cases, syntax_lesson_completed, syntax_lessons_for, syntax_progress_count,
    },
    process::which,
    text::render_markdown_plain,
};
use std::{collections::HashSet, fs, process::Command};

fn test_state() -> AppState {
    AppState {
        current_problem: "001-hello-world".to_string(),
        settings: Settings::default(),
        solved: Vec::new(),
        history: Vec::new(),
        suggested_next_difficulty: "easy".to_string(),
        syntax_progress: Default::default(),
        current_syntax_lesson: Default::default(),
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
    }
}

#[test]
fn syntax_mastery_uses_one_day_initial_schedule() {
    let mut state = test_state();

    record_syntax_result(&mut state, "rust", "rust-ownership", true, 1_000, false);

    let mastery = &state.syntax_mastery["rust"]["rust-ownership"];
    assert_eq!(mastery.stage, MasteryStage::Practiced);
    assert_eq!(mastery.review_due_at, 1_000 + 86_400);
    assert_eq!(mastery.attempts, 1);
    assert_eq!(
        serde_json::to_string(&MasteryStage::Practiced).unwrap(),
        "\"practiced\""
    );
    assert_eq!(
        serde_json::from_str::<LessonMastery>("{}").unwrap(),
        LessonMastery::default()
    );
}

#[test]
fn syntax_mastery_scheduling_saturates_at_u64_max() {
    let now = u64::MAX - 1;
    let mut result_state = test_state();

    record_syntax_result(&mut result_state, "rust", "rust-output", true, now, false);

    assert_eq!(
        result_state.syntax_mastery["rust"]["rust-output"],
        LessonMastery {
            stage: MasteryStage::Practiced,
            review_due_at: u64::MAX,
            attempts: 1,
        }
    );

    let mut test_out_state = test_state();
    record_syntax_test_out(&mut test_out_state, "rust", &["rust-output"], now);

    assert_eq!(
        test_out_state.syntax_mastery["rust"]["rust-output"],
        LessonMastery {
            stage: MasteryStage::Practiced,
            review_due_at: u64::MAX,
            attempts: 1,
        }
    );
}

#[test]
fn syntax_mastery_uses_three_and_seven_day_follow_up_schedule() {
    let mut state = test_state();
    let lesson_id = "rust-ownership";

    record_syntax_result(&mut state, "rust", lesson_id, true, 1_000, false);
    record_syntax_result(&mut state, "rust", lesson_id, true, 1_000 + 86_400, false);
    let retained = &state.syntax_mastery["rust"][lesson_id];
    assert_eq!(retained.stage, MasteryStage::Retained);
    assert_eq!(retained.review_due_at, 1_000 + 86_400 + 259_200);
    let retained_due_at = retained.review_due_at;

    record_syntax_result(&mut state, "rust", lesson_id, true, retained_due_at, false);
    let mastered = &state.syntax_mastery["rust"][lesson_id];
    assert_eq!(mastered.stage, MasteryStage::Mastered);
    assert_eq!(mastered.review_due_at, 1_000 + 86_400 + 259_200 + 604_800);

    let maintenance_at = mastered.review_due_at;
    record_syntax_result(&mut state, "rust", lesson_id, true, maintenance_at, false);
    let maintained = &state.syntax_mastery["rust"][lesson_id];
    assert_eq!(maintained.stage, MasteryStage::Mastered);
    assert_eq!(maintained.review_due_at, maintenance_at + 604_800);
    assert_eq!(maintained.attempts, 4);
}

#[test]
fn syntax_mastery_failure_demotes_one_stage_without_revoking_completion() {
    let mut state = test_state();
    let lesson_id = "rust-ownership";
    for now in [1_000, 87_400, 346_600] {
        record_syntax_result(&mut state, "rust", lesson_id, true, now, false);
    }
    state.completed_syntax_courses.push("rust".to_string());

    record_syntax_result(&mut state, "rust", lesson_id, false, 2_000_000, false);
    let retained = &state.syntax_mastery["rust"][lesson_id];
    assert_eq!(retained.stage, MasteryStage::Retained);
    assert_eq!(retained.review_due_at, 2_000_000);
    assert_eq!(retained.attempts, 4);
    assert_eq!(state.completed_syntax_courses, ["rust"]);

    record_syntax_result(&mut state, "rust", lesson_id, false, 2_000_001, false);
    assert_eq!(
        state.syntax_mastery["rust"][lesson_id].stage,
        MasteryStage::Practiced
    );
    record_syntax_result(&mut state, "rust", lesson_id, false, 2_000_002, false);
    assert_eq!(
        state.syntax_mastery["rust"][lesson_id].stage,
        MasteryStage::New
    );
    record_syntax_result(&mut state, "rust", lesson_id, false, 2_000_003, false);
    assert_eq!(
        state.syntax_mastery["rust"][lesson_id].stage,
        MasteryStage::New
    );
    assert_eq!(
        state.syntax_mastery["rust"][lesson_id].review_due_at,
        2_000_003
    );
    assert!(due_syntax_lessons(&state, "rust", 2_000_003, 2).is_empty());
}

#[test]
fn syntax_mastery_due_reviews_are_ordered_and_capped_at_two() {
    let mut state = test_state();
    record_syntax_result(&mut state, "rust", "rust-output", true, 100, false);
    record_syntax_result(&mut state, "rust", "rust-variables", true, 50, false);
    record_syntax_result(&mut state, "rust", "rust-input", true, 50, false);

    let due = due_syntax_lessons(&state, "rust", 100_000, 10)
        .into_iter()
        .map(|lesson| lesson.id)
        .collect::<Vec<_>>();
    assert_eq!(due, ["rust-variables", "rust-input"]);

    let limited = due_syntax_lessons(&state, "rust", 100_000, 1)
        .into_iter()
        .map(|lesson| lesson.id)
        .collect::<Vec<_>>();
    assert_eq!(limited, ["rust-variables"]);
    assert!(due_syntax_lessons(&state, "rust", 100_000, 0).is_empty());
}

#[test]
fn syntax_mastery_clock_regression_clamps_review_into_current_session() {
    let mut state = test_state();
    record_syntax_result(&mut state, "rust", "rust-ownership", true, 1_000_000, false);

    let due = due_syntax_lessons(&state, "rust", 100, 2)
        .into_iter()
        .map(|lesson| lesson.id)
        .collect::<Vec<_>>();

    assert_eq!(due, ["rust-ownership"]);
}

#[test]
fn syntax_mastery_early_retry_waits_but_clock_regression_is_due() {
    let mut state = test_state();
    record_syntax_result(&mut state, "rust", "rust-ownership", true, 1_000, false);
    let scheduled = state.syntax_mastery["rust"]["rust-ownership"].clone();

    record_syntax_result(&mut state, "rust", "rust-ownership", true, 2_000, false);
    record_syntax_result(&mut state, "rust", "rust-ownership", false, 3_000, false);
    let early = &state.syntax_mastery["rust"]["rust-ownership"];
    assert_eq!(early.stage, MasteryStage::Practiced);
    assert_eq!(early.review_due_at, scheduled.review_due_at);
    assert_eq!(early.attempts, 3);

    let mut regressed = test_state();
    record_syntax_result(
        &mut regressed,
        "rust",
        "rust-ownership",
        true,
        1_000_000,
        false,
    );
    record_syntax_result(&mut regressed, "rust", "rust-ownership", true, 100, false);
    let reviewed = &regressed.syntax_mastery["rust"]["rust-ownership"];
    assert_eq!(reviewed.stage, MasteryStage::Retained);
    assert_eq!(reviewed.review_due_at, 100 + 259_200);
    assert_eq!(reviewed.attempts, 2);
}

#[test]
fn syntax_mastery_test_out_only_schedules_new_coverage() {
    let mut state = test_state();
    for now in [10, 86_410, 345_610] {
        record_syntax_result(&mut state, "rust", "rust-output", true, now, false);
    }
    for now in [10, 86_410] {
        record_syntax_result(&mut state, "rust", "rust-variables", true, now, false);
    }
    record_syntax_result(&mut state, "rust", "rust-input", true, 10, false);
    let before = state.syntax_mastery["rust"].clone();

    record_syntax_test_out(
        &mut state,
        "rust",
        &[
            "rust-output",
            "rust-variables",
            "rust-input",
            "rust-strings",
        ],
        1_000,
    );

    for id in ["rust-output", "rust-variables", "rust-input"] {
        assert_eq!(state.syntax_mastery["rust"][id].stage, before[id].stage);
        assert_eq!(
            state.syntax_mastery["rust"][id].review_due_at,
            before[id].review_due_at
        );
        assert_eq!(
            state.syntax_mastery["rust"][id].attempts,
            before[id].attempts + 1
        );
    }
    let new_coverage = &state.syntax_mastery["rust"]["rust-strings"];
    assert_eq!(new_coverage.stage, MasteryStage::Practiced);
    assert_eq!(new_coverage.review_due_at, 1_000 + 86_400);
    assert_eq!(new_coverage.attempts, 1);
}

#[test]
fn syntax_mastery_legacy_migration_is_fixed_time_and_idempotent() {
    let mut state = test_state();
    state.syntax_progress.insert(
        "rust".to_string(),
        vec![
            "rust-ownership".to_string(),
            "unknown-a".to_string(),
            "unknown-b".to_string(),
            "rust-ownership".to_string(),
        ],
    );
    state
        .syntax_progress
        .insert("ruby".to_string(), vec!["legacy-ruby".to_string()]);

    migrate_syntax_mastery(&mut state, 1_000);

    let migrated = &state.syntax_mastery["rust"]["rust-ownership"];
    assert_eq!(migrated.stage, MasteryStage::Practiced);
    assert_eq!(migrated.review_due_at, 1_000);
    assert_eq!(migrated.attempts, 1);
    assert_eq!(state.syntax_progress["rust"], ["unknown-a", "unknown-b"]);
    assert_eq!(state.syntax_progress["ruby"], ["legacy-ruby"]);
    assert_eq!(syntax_progress_count(&state, "rust"), (1, 29));
    assert!(syntax_lesson_completed(&state, "rust", "rust-ownership"));

    migrate_syntax_mastery(&mut state, 2_000);

    let repeated = &state.syntax_mastery["rust"]["rust-ownership"];
    assert_eq!(repeated.review_due_at, 1_000);
    assert_eq!(repeated.attempts, 1);
}

#[test]
fn syntax_mastery_legacy_pass_repairs_mixed_new_state_once() {
    let mut state = test_state();
    state.syntax_mastery.insert(
        "rust".to_string(),
        std::collections::HashMap::from([(
            "rust-ownership".to_string(),
            LessonMastery {
                stage: MasteryStage::New,
                review_due_at: 500,
                attempts: 1,
            },
        )]),
    );
    state.syntax_progress.insert(
        "rust".to_string(),
        vec!["rust-ownership".to_string(), "rust-ownership".to_string()],
    );

    migrate_syntax_mastery(&mut state, 1_000);
    migrate_syntax_mastery(&mut state, 2_000);

    assert_eq!(
        state.syntax_mastery["rust"]["rust-ownership"],
        LessonMastery {
            stage: MasteryStage::Practiced,
            review_due_at: 1_000,
            attempts: 2,
        }
    );
}

#[test]
fn syntax_mastery_migrates_every_known_legacy_id() {
    let mut state = test_state();
    for language in LANGUAGES {
        state.syntax_progress.insert(
            (*language).to_string(),
            syntax_lessons_for(language)
                .into_iter()
                .map(|lesson| lesson.id.to_string())
                .collect(),
        );
    }

    migrate_syntax_mastery(&mut state, 1_000);

    assert!(state.syntax_progress.is_empty());
    for language in LANGUAGES {
        for lesson in syntax_lessons_for(language) {
            assert_eq!(
                state.syntax_mastery[*language][lesson.id],
                LessonMastery {
                    stage: MasteryStage::Practiced,
                    review_due_at: 1_000,
                    attempts: 1,
                },
                "{language}:{}",
                lesson.id
            );
        }
    }
}

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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
    };
    save_state(&root, &state).unwrap();
    let saved = fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert!(saved.contains("\"ai_provider\": \"claude\""));
    assert!(saved.contains("\"ai_model\": \"sonnet\""));
    assert!(saved.contains("\"ai_effort\": \"max\""));
    assert_eq!(load_state(&root, &bank).unwrap().settings.next_source, "ai");
}

#[test]
fn state_save_preserves_previous_backup_and_cleans_temporary_file() {
    let root = tmp_root("state-safe-save");
    let submission = root.join("submissions/.syntax/rust/rust-output/exercise.rs");
    fs::create_dir_all(submission.parent().unwrap()).unwrap();
    fs::write(&submission, "fn main() { /* learner edit */ }\n").unwrap();
    let mut state = test_state();

    save_state(&root, &state).unwrap();
    let first = fs::read_to_string(root.join("problem-state.json")).unwrap();
    fs::write(root.join("problem-state.json.tmp"), "stale").unwrap();
    state.settings.theme = "light".to_string();
    save_state(&root, &state).unwrap();

    let second = fs::read_to_string(root.join("problem-state.json")).unwrap();
    assert_ne!(second, first);
    assert_eq!(
        fs::read_to_string(root.join("problem-state.json.bak")).unwrap(),
        first
    );
    assert!(!root.join("problem-state.json.tmp").exists());
    assert_eq!(
        fs::read_to_string(submission).unwrap(),
        "fn main() { /* learner edit */ }\n"
    );
}

#[test]
fn state_save_failure_preserves_primary_submission_and_removes_temporary() {
    let root = tmp_root("state-safe-save-failure");
    let submission = root.join("submissions/.syntax/rust/rust-output/exercise.rs");
    fs::create_dir_all(submission.parent().unwrap()).unwrap();
    fs::write(&submission, "fn main() { /* learner edit */ }\n").unwrap();
    let mut state = test_state();
    save_state(&root, &state).unwrap();
    let primary = root.join("problem-state.json");
    let original = fs::read(&primary).unwrap();
    fs::create_dir(root.join("problem-state.json.bak")).unwrap();

    state.settings.theme = "light".to_string();
    let error = save_state(&root, &state).unwrap_err().to_string();

    assert!(error.contains("problem-state.json"));
    assert!(error.contains("problem-state.json.bak"));
    assert_eq!(fs::read(primary).unwrap(), original);
    assert!(!root.join("problem-state.json.tmp").exists());
    assert_eq!(
        fs::read_to_string(submission).unwrap(),
        "fn main() { /* learner edit */ }\n"
    );
}

#[test]
fn state_load_restores_backup_when_primary_is_missing() {
    let root = tmp_root("state-backup-recovery");
    let bank = load_bank(&root).unwrap();
    let mut state = test_state();
    state.settings.theme = "light".to_string();
    save_state(&root, &state).unwrap();
    let primary = root.join("problem-state.json");
    let backup = root.join("problem-state.json.bak");
    let expected = fs::read(&primary).unwrap();
    fs::rename(&primary, &backup).unwrap();
    fs::write(root.join("problem-state.json.tmp"), "partial").unwrap();

    let recovered = load_state(&root, &bank).unwrap();

    assert_eq!(recovered.settings.theme, "light");
    assert_eq!(fs::read(&primary).unwrap(), expected);
    assert_eq!(fs::read(&backup).unwrap(), expected);
    assert!(!root.join("problem-state.json.tmp").exists());
}

#[test]
#[cfg(unix)]
fn state_save_does_not_follow_a_backup_symlink_into_submissions() {
    use std::os::unix::fs::symlink;

    let root = tmp_root("state-backup-symlink");
    let submission = root.join("submissions/.syntax/rust/rust-output/exercise.rs");
    fs::create_dir_all(submission.parent().unwrap()).unwrap();
    fs::write(&submission, "fn main() { /* learner edit */ }\n").unwrap();
    let mut state = test_state();
    save_state(&root, &state).unwrap();
    let previous = fs::read(root.join("problem-state.json")).unwrap();
    let backup = root.join("problem-state.json.bak");
    symlink(&submission, &backup).unwrap();

    state.settings.theme = "light".to_string();
    save_state(&root, &state).unwrap();

    assert_eq!(
        fs::read_to_string(submission).unwrap(),
        "fn main() { /* learner edit */ }\n"
    );
    assert!(
        !fs::symlink_metadata(&backup)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(backup).unwrap(), previous);
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
fn syntax_mastery_load_migrates_progress_idempotently_without_touching_submissions() {
    let root = tmp_root("state-syntax-progress");
    let bank = load_bank(&root).unwrap();
    let submission = root.join("submissions/.syntax/python/py-variables/exercise.py");
    fs::create_dir_all(submission.parent().unwrap()).unwrap();
    fs::write(&submission, "print('learner edit')\n").unwrap();
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
    let migrated = &state.syntax_mastery["python"]["py-variables"];
    assert_eq!(migrated.stage, MasteryStage::Practiced);
    assert_eq!(migrated.attempts, 1);
    assert_eq!(state.syntax_progress["python"], ["unknown"]);
    assert_eq!(state.syntax_progress["ruby"], ["variables"]);
    assert_eq!(state.current_syntax_lesson["python"], "py-functions");
    assert!(!state.current_syntax_lesson.contains_key("ruby"));
    let due_at = migrated.review_due_at;

    save_state(&root, &state).unwrap();
    let reloaded = load_state(&root, &bank).unwrap();
    let repeated = &reloaded.syntax_mastery["python"]["py-variables"];
    assert_eq!(repeated.review_due_at, due_at);
    assert_eq!(repeated.attempts, 1);
    assert_eq!(
        fs::read_to_string(submission).unwrap(),
        "print('learner edit')\n"
    );
}

#[test]
fn syntax_mastery_legacy_record_pass_updates_current_ui_without_legacy_data() {
    let mut state = test_state();

    record_syntax_pass(&mut state, "rust", "rust-ownership");

    assert_eq!(
        state.syntax_mastery["rust"]["rust-ownership"].stage,
        MasteryStage::Practiced
    );
    assert_eq!(state.syntax_mastery["rust"]["rust-ownership"].attempts, 1);
    assert!(state.syntax_progress.is_empty());
    assert_eq!(syntax_progress_count(&state, "rust"), (1, 29));
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
fn render_problem_tui_localizes_empty_example_values() {
    let root = tmp_root("render-tui-empty-ko");
    let mut problem = load_bank(&root).unwrap().remove(0);
    problem.examples[0].input.clear();
    problem.examples[0].output.clear();

    let rendered = render_problem_tui(&problem, "ko");

    assert!(rendered.contains("      <비어 있음>"), "{rendered}");
    assert!(!rendered.contains("<empty>"), "{rendered}");
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
fn judge_preserves_meaningful_whitespace() {
    assert_eq!(normalize_judge_output("ok\r\n"), "ok\n");
    assert_eq!(normalize_judge_output("ok\r"), "ok\r");
    assert_ne!(normalize_judge_output(" ok\n"), "ok\n");
    assert_ne!(normalize_judge_output("ok\n\n"), "ok\n");
}

#[test]
fn judge_compares_normalized_output_without_trimming() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-exact-output");
    let path = root.join("solution.py");
    fs::write(&path, "import sys\nsys.stdout.write('ok\\n\\n')\n").unwrap();
    let result = judge_path(
        &root,
        "exact-output",
        &path,
        "python",
        &[practicode::core::IoCase {
            input: String::new(),
            output: "ok\n".to_string(),
        }],
    );

    assert!(!result.passed, "extra output newline must be meaningful");
    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Output));
    assert!(result.output.contains("Got\n  ok\n  "));
}

#[test]
fn judge_classifies_nonzero_exit_as_runtime_failure() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-runtime-failure");
    let path = root.join("solution.py");
    fs::write(
        &path,
        "import sys\nprint('runtime detail', file=sys.stderr)\nsys.exit(7)\n",
    )
    .unwrap();
    let result = judge_path(
        &root,
        "runtime-failure",
        &path,
        "python",
        &[practicode::core::IoCase {
            input: String::new(),
            output: "private expected\n".to_string(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Runtime));
    assert!(result.output.contains("runtime detail"));
    assert!(!result.output.contains("private expected"));
}

#[test]
fn judge_classifies_timeout_before_runtime_failure() {
    if which("python3").or_else(|| which("python")).is_none() {
        return;
    }
    let root = tmp_root("judge-timeout-failure");
    let path = root.join("solution.py");
    fs::write(&path, "import time\ntime.sleep(10)\n").unwrap();
    let result = judge_path(
        &root,
        "timeout-failure",
        &path,
        "python",
        &[practicode::core::IoCase {
            input: String::new(),
            output: String::new(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Timeout));
    assert!(result.output.contains("timeout: 5s"));
}

#[test]
fn java_compiler_errors_are_compile_failures() {
    if which("javac").is_none() || which("java").is_none() {
        return;
    }
    let root = tmp_root("judge-java-compile-failure");
    let path = root.join("Solution.java");
    fs::write(
        &path,
        "class Solution { public static void main(String[] args) { nope } }\n",
    )
    .unwrap();
    let result = judge_path(
        &root,
        "java-compile-failure",
        &path,
        "java",
        &[practicode::core::IoCase {
            input: String::new(),
            output: String::new(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Compile));
    assert!(result.output.contains("Solution.java"));
}

#[test]
fn rust_compiler_errors_are_compile_failures() {
    if which("rustc").is_none() {
        return;
    }
    let root = tmp_root("judge-rust-compile-failure");
    let path = root.join("solution.rs");
    fs::write(&path, "fn main() { let value: = 1; }\n").unwrap();
    let result = judge_path(
        &root,
        "rust-compile-failure",
        &path,
        "rust",
        &[practicode::core::IoCase {
            input: String::new(),
            output: String::new(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Compile));
    assert!(result.output.contains("solution.rs"));
}

#[test]
fn rust_judge_explicitly_uses_edition_2024() {
    if which("rustc").is_none() {
        return;
    }
    let root = tmp_root("judge-rust-edition-2024");
    let path = root.join("solution.rs");
    fs::write(&path, "fn main() { let gen = 1; println!(\"{gen}\"); }\n").unwrap();
    let result = judge_path(
        &root,
        "rust-edition-2024",
        &path,
        "rust",
        &[practicode::core::IoCase {
            input: String::new(),
            output: "1\n".to_string(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Compile));
    assert!(result.output.contains("reserved keyword"));
}

#[test]
fn typescript_type_errors_fail_before_matching_stdout_runs() {
    if which("node").is_none() || which("tsc").is_none() {
        return;
    }
    let root = tmp_root("judge-typescript-typecheck-failure");
    let dir = root.join("submissions/typecheck");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("solution.ts");
    fs::write(
        &path,
        "const value: number = \"ok\";\nconsole.log(value);\n",
    )
    .unwrap();
    let result = judge_path(
        &root,
        "typescript-typecheck-failure",
        &path,
        "ts",
        &[practicode::core::IoCase {
            input: String::new(),
            output: "ok\n".to_string(),
        }],
    );

    assert!(!result.passed);
    assert_eq!(result.failure_kind, Some(JudgeFailureKind::TypeCheck));
    assert!(result.output.contains("TS2322"), "{}", result.output);
    assert!(
        root.join("build/typecheck/typescript/node-shim.d.ts")
            .exists()
    );
    assert!(!dir.join("solution.js").exists());
}

#[test]
fn typescript_typecheck_ignores_ambient_node_types() {
    if which("node").is_none() || which("tsc").is_none() {
        return;
    }
    let root = tmp_root("judge-typescript-isolated-types");
    let ambient = root.join("node_modules/@types/ambient");
    fs::create_dir_all(&ambient).unwrap();
    fs::write(
        ambient.join("index.d.ts"),
        "declare const Buffer: { from(value: string): { toString(): string } };\n",
    )
    .unwrap();
    let dir = root.join("submissions/isolated-types");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("solution.ts");
    fs::write(&path, "console.log(Buffer.from('ok').toString());\n").unwrap();

    let result = judge_path(
        &root,
        "typescript-isolated-types",
        &path,
        "ts",
        &[practicode::core::IoCase {
            input: String::new(),
            output: "ok\n".to_string(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::TypeCheck));
    assert!(result.output.contains("Buffer"), "{}", result.output);
}

#[test]
fn typescript_nonzero_exit_is_a_runtime_failure_after_typecheck() {
    if which("node").is_none() || which("tsc").is_none() {
        return;
    }
    let root = tmp_root("judge-typescript-runtime-failure");
    let dir = root.join("submissions/runtime");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("solution.ts");
    fs::write(&path, "throw new Error('runtime detail');\n").unwrap();
    let result = judge_path(
        &root,
        "typescript-runtime-failure",
        &path,
        "ts",
        &[practicode::core::IoCase {
            input: String::new(),
            output: String::new(),
        }],
    );

    assert_eq!(result.failure_kind, Some(JudgeFailureKind::Runtime));
    assert!(result.output.contains("runtime detail"));
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
fn embedded_courses_preserve_every_existing_lesson() {
    assert_eq!(syntax_lessons_for("python").len(), 25);
    assert_eq!(syntax_lessons_for("ts").len(), 28);
    assert_eq!(syntax_lessons_for("java").len(), 28);
    assert_eq!(syntax_lessons_for("rust").len(), 29);

    let mut all_ids = HashSet::new();
    for language in LANGUAGES {
        let lessons = syntax_lessons_for(language);
        assert!(lessons.iter().all(|lesson| lesson.language == *language));
        assert!(lessons.iter().all(|lesson| !lesson.id.is_empty()));
        assert_eq!(
            lessons
                .iter()
                .map(|lesson| lesson.id)
                .collect::<HashSet<_>>()
                .len(),
            lessons.len()
        );
        for lesson in lessons {
            assert!(
                all_ids.insert(lesson.id),
                "duplicate lesson ID: {}",
                lesson.id
            );
            assert!(matches!(lesson.track, SyntaxTrack::Core | SyntaxTrack::Lab));
            assert!(matches!(
                lesson.kind,
                SyntaxKind::Lesson | SyntaxKind::Checkpoint | SyntaxKind::Capstone
            ));
            assert!(!lesson.example.trim().is_empty(), "{} example", lesson.id);
            assert!(
                !lesson.exercise.starter.trim().is_empty(),
                "{} starter",
                lesson.id
            );
            assert!(!lesson.refs.is_empty(), "{} refs", lesson.id);
            assert!(
                lesson.refs.iter().all(|url| url.starts_with("https://")),
                "{} refs",
                lesson.id
            );
        }
    }
    assert_eq!(all_ids.len(), 110);
}

#[test]
fn embedded_course_assets_use_the_versioned_contract() {
    for (runtime, path) in [
        ("python", "assets/lessons/python/course.json"),
        ("ts", "assets/lessons/typescript/course.json"),
        ("java", "assets/lessons/java/course.json"),
        ("rust", "assets/lessons/rust/course.json"),
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(catalog["schema_version"], 1);
        assert_eq!(catalog["runtime"], runtime);
        for lesson in catalog["lessons"].as_array().unwrap() {
            assert!(lesson["aliases"].is_array());
            assert!(matches!(lesson["track"].as_str(), Some("core" | "lab")));
            assert!(matches!(
                lesson["kind"].as_str(),
                Some("lesson" | "checkpoint" | "capstone")
            ));
        }
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
    };

    let rendered = render_syntax_lesson(lesson, &state);

    assert!(rendered.contains("## Exercise"));
    assert!(rendered.find("## Exercise") < rendered.find("## Common mistakes"));
    assert!(rendered.contains("Input\n\n```text\nAda 7\n```"));
    assert!(rendered.contains("Output\n\n```text\nAda:7\n```"));
    let plain = render_markdown_plain(&rendered);
    assert!(plain.contains("  name = \"Mina\""));
    assert!(plain.contains("  print(name, score, sep=\"=\", end=\"!\\n\")"));
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
            syntax_mastery: Default::default(),
            completed_syntax_courses: Default::default(),
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
        assert_eq!(
            result.failure_kind,
            Some(JudgeFailureKind::Output),
            "{} worked example should run and differ only by output:\n{}",
            lesson.id,
            result.output
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
    if which("node").is_none() || which("tsc").is_none() {
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
        assert_eq!(
            result.failure_kind,
            Some(JudgeFailureKind::Output),
            "{} worked example should typecheck, run, and differ only by output:\n{}",
            lesson.id,
            result.output
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
        assert_eq!(
            result.failure_kind,
            Some(JudgeFailureKind::Output),
            "{} worked example should compile, run, and differ only by output:\n{}",
            lesson.id,
            result.output
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
        assert_eq!(
            result.failure_kind,
            Some(JudgeFailureKind::Output),
            "{} worked example should compile, run, and differ only by output:\n{}",
            lesson.id,
            result.output
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
        syntax_mastery: Default::default(),
        completed_syntax_courses: Default::default(),
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
