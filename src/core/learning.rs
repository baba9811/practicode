use super::*;

const DAY_SECONDS: u64 = 86_400;
const THREE_DAYS_SECONDS: u64 = 259_200;
const SEVEN_DAYS_SECONDS: u64 = 604_800;

pub(super) fn unix_timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn record_syntax_result(
    state: &mut AppState,
    language: &str,
    lesson_id: &str,
    passed: bool,
    now: u64,
    assisted: bool,
) {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    record_syntax_result_for_lessons(state, &language, lesson_id, passed, now, assisted, &lessons);
}

pub(crate) fn record_syntax_result_for_lessons(
    state: &mut AppState,
    language: &str,
    lesson_id: &str,
    passed: bool,
    now: u64,
    assisted: bool,
    lessons: &[&SyntaxLesson],
) {
    let Some(lesson) = lessons.iter().find(|lesson| lesson.id == lesson_id) else {
        return;
    };

    let mastery = state
        .syntax_mastery
        .entry(language.to_string())
        .or_default()
        .entry(lesson_id.to_string())
        .or_default();
    mastery.attempts = mastery.attempts.saturating_add(1);
    if (mastery.stage != MasteryStage::New
        && syntax_review_due_at(mastery, now).is_none_or(|due_at| due_at > now))
        || (passed && assisted && lesson.kind == SyntaxKind::Capstone)
    {
        return;
    }
    if passed {
        let (stage, delay) = match mastery.stage {
            MasteryStage::New => (MasteryStage::Practiced, DAY_SECONDS),
            MasteryStage::Practiced => (MasteryStage::Retained, THREE_DAYS_SECONDS),
            MasteryStage::Retained | MasteryStage::Mastered => {
                (MasteryStage::Mastered, SEVEN_DAYS_SECONDS)
            }
        };
        mastery.stage = stage;
        mastery.review_due_at = now.saturating_add(delay);
    } else {
        mastery.stage = match mastery.stage {
            MasteryStage::Mastered => MasteryStage::Retained,
            MasteryStage::Retained => MasteryStage::Practiced,
            MasteryStage::Practiced | MasteryStage::New => MasteryStage::New,
        };
        mastery.review_due_at = now;
    }
    if syntax_course_completed(state, language, lessons)
        && !state
            .completed_syntax_courses
            .iter()
            .any(|completed| completed == language)
    {
        state.completed_syntax_courses.push(language.to_string());
    }
}

pub(crate) fn syntax_review_due_at(mastery: &LessonMastery, now: u64) -> Option<u64> {
    let max_delay = match mastery.stage {
        MasteryStage::New => return None,
        MasteryStage::Practiced => DAY_SECONDS,
        MasteryStage::Retained => THREE_DAYS_SECONDS,
        MasteryStage::Mastered => SEVEN_DAYS_SECONDS,
    };
    Some(if mastery.review_due_at > now.saturating_add(max_delay) {
        now
    } else {
        mastery.review_due_at
    })
}

fn syntax_course_completed(state: &AppState, language: &str, lessons: &[&SyntaxLesson]) -> bool {
    let Some(mastery) = state.syntax_mastery.get(language) else {
        return false;
    };
    let retained = |lesson: &&SyntaxLesson| {
        mastery.get(lesson.id).is_some_and(|progress| {
            matches!(
                progress.stage,
                MasteryStage::Retained | MasteryStage::Mastered
            )
        })
    };
    let all_core_retained = lessons
        .iter()
        .filter(|lesson| lesson.track == SyntaxTrack::Core)
        .all(retained);
    let checkpoints_passed = lessons
        .iter()
        .filter(|lesson| lesson.track == SyntaxTrack::Core && lesson.kind == SyntaxKind::Checkpoint)
        .all(retained);
    let capstone_retained = lessons
        .iter()
        .find(|lesson| lesson.track == SyntaxTrack::Core && lesson.kind == SyntaxKind::Capstone)
        .is_some_and(retained);
    all_core_retained && checkpoints_passed && capstone_retained
}

pub fn due_syntax_lessons(
    state: &AppState,
    language: &str,
    now: u64,
    limit: usize,
) -> Vec<&'static SyntaxLesson> {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    due_syntax_lessons_for(state, &language, now, limit, &lessons)
}

fn due_syntax_lessons_for<'a>(
    state: &AppState,
    language: &str,
    now: u64,
    limit: usize,
    lessons: &[&'a SyntaxLesson],
) -> Vec<&'a SyntaxLesson> {
    let mut due = due_syntax_lesson_candidates(state, language, now, lessons);
    due.sort_by_key(|&(review_due_at, index, _)| (review_due_at, index));
    due.into_iter()
        .take(limit.min(2))
        .map(|(_, _, lesson)| lesson)
        .collect()
}

pub fn due_syntax_lesson_count(state: &AppState, language: &str, now: u64) -> usize {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    due_syntax_lesson_candidates(state, &language, now, &lessons).len()
}

fn due_syntax_lesson_candidates<'a>(
    state: &AppState,
    language: &str,
    now: u64,
    lessons: &[&'a SyntaxLesson],
) -> Vec<(u64, usize, &'a SyntaxLesson)> {
    let Some(mastery) = state.syntax_mastery.get(language) else {
        return Vec::new();
    };
    lessons
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, lesson)| {
            let progress = mastery.get(lesson.id)?;
            let review_due_at = syntax_review_due_at(progress, now)?;
            (lesson.track != SyntaxTrack::Lab && review_due_at <= now).then_some((
                review_due_at,
                index,
                lesson,
            ))
        })
        .collect()
}

pub fn record_syntax_test_out(state: &mut AppState, language: &str, lesson_ids: &[&str], now: u64) {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    record_syntax_test_out_for_lessons(state, &language, lesson_ids, now, &lessons);
}

fn record_syntax_test_out_for_lessons(
    state: &mut AppState,
    language: &str,
    lesson_ids: &[&str],
    now: u64,
    lessons: &[&SyntaxLesson],
) {
    for lesson_id in lesson_ids {
        let Some(lesson) = super::syntax::resolve_syntax_lesson(lessons, lesson_id) else {
            continue;
        };
        if lesson.track != SyntaxTrack::Core || lesson.kind != SyntaxKind::Lesson {
            continue;
        }
        let mastery = state
            .syntax_mastery
            .entry(language.to_string())
            .or_default()
            .entry(lesson.id.to_string())
            .or_default();
        mastery.attempts = mastery.attempts.saturating_add(1);
        if mastery.stage == MasteryStage::New {
            mastery.stage = MasteryStage::Practiced;
            mastery.review_due_at = now.saturating_add(DAY_SECONDS);
        }
    }
}

pub fn migrate_syntax_mastery(state: &mut AppState, now: u64) {
    for language in LANGUAGES {
        normalize_syntax_mastery_for_lessons(state, language, &syntax_lessons_for(language));
    }
    let legacy = std::mem::take(&mut state.syntax_progress);
    for (language, ids) in legacy {
        if !LANGUAGES.contains(&language.as_str()) {
            state.syntax_progress.insert(language, ids);
            continue;
        }
        let lessons = syntax_lessons_for(&language);
        let unknown = migrate_syntax_mastery_for_lessons(state, &language, ids, now, &lessons);
        if !unknown.is_empty() {
            state.syntax_progress.insert(language, unknown);
        }
    }
}

fn normalize_syntax_mastery_for_lessons(
    state: &mut AppState,
    language: &str,
    lessons: &[&SyntaxLesson],
) {
    let Some(mut stored) = state.syntax_mastery.remove(language) else {
        return;
    };
    let mut normalized = HashMap::new();
    for lesson in lessons {
        let mut current = stored.remove(lesson.id);
        for alias in lesson.aliases {
            if let Some(alias_mastery) = stored.remove(*alias) {
                current = Some(match current {
                    Some(canonical) => merge_lesson_mastery(canonical, alias_mastery),
                    None => alias_mastery,
                });
            }
        }
        if let Some(mastery) = current {
            normalized.insert(lesson.id.to_string(), mastery);
        }
    }
    normalized.extend(stored);
    if !normalized.is_empty() {
        state
            .syntax_mastery
            .insert(language.to_string(), normalized);
    }
}

fn merge_lesson_mastery(mut canonical: LessonMastery, mut alias: LessonMastery) -> LessonMastery {
    let rank = |stage| match stage {
        MasteryStage::New => 0,
        MasteryStage::Practiced => 1,
        MasteryStage::Retained => 2,
        MasteryStage::Mastered => 3,
    };
    let attempts = canonical.attempts.max(alias.attempts);
    if rank(alias.stage) > rank(canonical.stage) {
        alias.attempts = attempts;
        alias
    } else {
        canonical.attempts = attempts;
        canonical
    }
}

fn migrate_syntax_mastery_for_lessons(
    state: &mut AppState,
    language: &str,
    ids: Vec<String>,
    now: u64,
    lessons: &[&SyntaxLesson],
) -> Vec<String> {
    let mut unknown = Vec::new();
    for id in ids {
        let Some(canonical) =
            super::syntax::resolve_syntax_lesson(lessons, &id).map(|lesson| lesson.id)
        else {
            unknown.push(id);
            continue;
        };
        state
            .syntax_mastery
            .entry(language.to_string())
            .or_default()
            .entry(canonical.to_string())
            .and_modify(|mastery| {
                if mastery.stage == MasteryStage::New {
                    mastery.stage = MasteryStage::Practiced;
                    mastery.review_due_at = now;
                    mastery.attempts = mastery.attempts.saturating_add(1);
                }
            })
            .or_insert(LessonMastery {
                stage: MasteryStage::Practiced,
                review_due_at: now,
                attempts: 1,
            });
    }
    unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> AppState {
        AppState {
            current_problem: "001-hello-world".to_string(),
            settings: Settings::default(),
            solved: Vec::new(),
            history: Vec::new(),
            suggested_next_difficulty: "easy".to_string(),
            syntax_progress: HashMap::new(),
            current_syntax_lesson: HashMap::new(),
            syntax_mastery: HashMap::new(),
            completed_syntax_courses: Vec::new(),
        }
    }

    fn lesson(id: &'static str, track: SyntaxTrack, kind: SyntaxKind) -> SyntaxLesson {
        SyntaxLesson {
            id,
            aliases: &[],
            language: "rust",
            track,
            kind,
            level: "basic",
            title: id,
            body: id,
            example: "fn main() {}",
            exercise: SyntaxExercise {
                prompt: id,
                starter: id,
                cases: &[],
            },
            refs: &[],
        }
    }

    #[test]
    fn syntax_mastery_assistance_capstone_checkpoints_and_labs_gate_completion() {
        let course = [
            lesson("core", SyntaxTrack::Core, SyntaxKind::Lesson),
            lesson("checkpoint", SyntaxTrack::Core, SyntaxKind::Checkpoint),
            lesson("capstone", SyntaxTrack::Core, SyntaxKind::Capstone),
            lesson("lab", SyntaxTrack::Lab, SyntaxKind::Lesson),
        ];
        let lessons = course.iter().collect::<Vec<_>>();
        let mut state = state();

        for now in [1_000, 87_400] {
            record_syntax_result_for_lessons(&mut state, "rust", "core", true, now, true, &lessons);
            record_syntax_result_for_lessons(
                &mut state,
                "rust",
                "checkpoint",
                true,
                now,
                false,
                &lessons,
            );
        }
        record_syntax_result_for_lessons(
            &mut state, "rust", "capstone", true, 1_000, true, &lessons,
        );
        assert_eq!(
            state.syntax_mastery["rust"]["core"].stage,
            MasteryStage::Retained
        );
        assert_eq!(
            state.syntax_mastery["rust"]["capstone"].stage,
            MasteryStage::New
        );
        assert!(state.completed_syntax_courses.is_empty());

        for now in [1_000, 87_400] {
            record_syntax_result_for_lessons(
                &mut state, "rust", "capstone", true, now, false, &lessons,
            );
        }
        assert_eq!(
            state.syntax_mastery["rust"]["capstone"].stage,
            MasteryStage::Retained
        );
        assert_eq!(state.completed_syntax_courses, ["rust"]);
    }

    #[test]
    fn syntax_mastery_due_reviews_exclude_labs() {
        let course = [
            lesson("core", SyntaxTrack::Core, SyntaxKind::Lesson),
            lesson("lab", SyntaxTrack::Lab, SyntaxKind::Lesson),
        ];
        let lessons = course.iter().collect::<Vec<_>>();
        let mut state = state();
        record_syntax_result_for_lessons(&mut state, "rust", "core", true, 1_000, false, &lessons);
        record_syntax_result_for_lessons(&mut state, "rust", "lab", true, 1_000, false, &lessons);

        let due = due_syntax_lessons_for(&state, "rust", 100_000, 10, &lessons)
            .into_iter()
            .map(|lesson| lesson.id)
            .collect::<Vec<_>>();

        assert_eq!(due, ["core"]);
    }

    #[test]
    fn syntax_mastery_test_out_seeds_only_core_lessons() {
        let course = [
            lesson("core", SyntaxTrack::Core, SyntaxKind::Lesson),
            lesson("checkpoint", SyntaxTrack::Core, SyntaxKind::Checkpoint),
            lesson("capstone", SyntaxTrack::Core, SyntaxKind::Capstone),
            lesson("lab", SyntaxTrack::Lab, SyntaxKind::Lesson),
        ];
        let lessons = course.iter().collect::<Vec<_>>();
        let mut state = state();

        record_syntax_test_out_for_lessons(
            &mut state,
            "rust",
            &["core", "checkpoint", "capstone", "lab"],
            1_000,
            &lessons,
        );

        assert_eq!(
            state.syntax_mastery["rust"].keys().collect::<Vec<_>>(),
            ["core"]
        );
        assert_eq!(
            state.syntax_mastery["rust"]["core"].stage,
            MasteryStage::Practiced
        );
    }

    #[test]
    fn syntax_mastery_migration_maps_aliases_to_current_ids() {
        let mut current = lesson("current", SyntaxTrack::Core, SyntaxKind::Lesson);
        current.aliases = &["legacy"];
        let lessons = [&current];
        let mut state = state();

        let unknown = migrate_syntax_mastery_for_lessons(
            &mut state,
            "rust",
            vec!["legacy".to_string()],
            1_000,
            &lessons,
        );

        assert!(unknown.is_empty());
        assert_eq!(
            state.syntax_mastery["rust"]["current"],
            LessonMastery {
                stage: MasteryStage::Practiced,
                review_due_at: 1_000,
                attempts: 1,
            }
        );
    }

    #[test]
    fn syntax_mastery_migration_prefers_an_exact_id_over_an_alias() {
        let mut first = lesson("first", SyntaxTrack::Core, SyntaxKind::Lesson);
        first.aliases = &["second"];
        let second = lesson("second", SyntaxTrack::Core, SyntaxKind::Lesson);
        let lessons = [&first, &second];
        let mut state = state();

        migrate_syntax_mastery_for_lessons(
            &mut state,
            "rust",
            vec!["second".to_string()],
            1_000,
            &lessons,
        );

        assert!(state.syntax_mastery["rust"].contains_key("second"));
        assert!(!state.syntax_mastery["rust"].contains_key("first"));
    }

    #[test]
    fn syntax_mastery_migration_normalizes_existing_alias_keys_idempotently() {
        let mut first = lesson("first", SyntaxTrack::Core, SyntaxKind::Lesson);
        first.aliases = &["old-first"];
        let mut second = lesson("second", SyntaxTrack::Core, SyntaxKind::Lesson);
        second.aliases = &["old-second"];
        let lessons = [&first, &second];
        let mut state = state();
        state.syntax_mastery.insert(
            "rust".to_string(),
            HashMap::from([
                (
                    "first".to_string(),
                    LessonMastery {
                        stage: MasteryStage::Retained,
                        review_due_at: 700,
                        attempts: 4,
                    },
                ),
                (
                    "old-first".to_string(),
                    LessonMastery {
                        stage: MasteryStage::Practiced,
                        review_due_at: 500,
                        attempts: 2,
                    },
                ),
                (
                    "second".to_string(),
                    LessonMastery {
                        stage: MasteryStage::Practiced,
                        review_due_at: 500,
                        attempts: 2,
                    },
                ),
                (
                    "old-second".to_string(),
                    LessonMastery {
                        stage: MasteryStage::Retained,
                        review_due_at: 700,
                        attempts: 4,
                    },
                ),
                (
                    "unknown".to_string(),
                    LessonMastery {
                        stage: MasteryStage::Mastered,
                        review_due_at: 900,
                        attempts: 9,
                    },
                ),
            ]),
        );

        normalize_syntax_mastery_for_lessons(&mut state, "rust", &lessons);
        let normalized = state.syntax_mastery["rust"].clone();
        normalize_syntax_mastery_for_lessons(&mut state, "rust", &lessons);

        assert_eq!(state.syntax_mastery["rust"], normalized);
        assert_eq!(normalized["first"].stage, MasteryStage::Retained);
        assert_eq!(normalized["first"].attempts, 4);
        assert_eq!(normalized["first"].review_due_at, 700);
        assert_eq!(normalized["second"].stage, MasteryStage::Retained);
        assert_eq!(normalized["second"].attempts, 4);
        assert_eq!(normalized["second"].review_due_at, 700);
        assert_eq!(normalized["unknown"].stage, MasteryStage::Mastered);
        assert!(!normalized.contains_key("old-first"));
        assert!(!normalized.contains_key("old-second"));
    }
}
