use super::*;
#[cfg(test)]
use crate::core::SyntaxKind;
use crate::core::{
    MasteryStage, SyntaxLesson, SyntaxTrack, due_syntax_lesson_count, due_syntax_lessons,
    syntax_lessons_for,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LearningStep {
    Review,
    Delta,
    Predict,
    Exercise,
    Reflect,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LearningView {
    Lesson,
    Code,
    Result,
}

pub(super) fn unix_time_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(super) fn learning_step_label(language: &str, step: LearningStep) -> &'static str {
    ui_text(
        language,
        match step {
            LearningStep::Review => "learning_step_review",
            LearningStep::Delta => "learning_step_delta",
            LearningStep::Predict => "learning_step_predict",
            LearningStep::Exercise => "learning_step_exercise",
            LearningStep::Reflect => "learning_step_reflect",
            LearningStep::Complete => "learning_step_complete",
        },
    )
}

#[derive(Clone, Copy, Debug)]
struct LearningItem {
    lesson_id: &'static str,
    review: bool,
}

#[derive(Debug)]
pub(super) struct LearningSession {
    guided: bool,
    queue: Vec<LearningItem>,
    index: usize,
    step: LearningStep,
    view: LearningView,
    assisted: bool,
}

pub(super) enum LearningAdvance {
    Step,
    Item(&'static str),
    Blocked,
    Complete,
    Manual,
}

impl LearningSession {
    pub(super) fn inactive() -> Self {
        Self {
            guided: false,
            queue: Vec::new(),
            index: 0,
            step: LearningStep::Complete,
            view: LearningView::Code,
            assisted: false,
        }
    }

    pub(super) fn start(state: &AppState, language: &str, now: u64) -> Self {
        let language = normalize_language(language);
        let lessons = syntax_lessons_for(&language);
        let due = due_syntax_lessons(state, &language, now, 2);
        let queue = session_queue(state, &language, &lessons, &due);
        let step = queue
            .first()
            .map(|item| {
                if item.review {
                    LearningStep::Review
                } else {
                    LearningStep::Delta
                }
            })
            .unwrap_or(LearningStep::Complete);
        Self {
            guided: true,
            queue,
            index: 0,
            step,
            view: LearningView::Code,
            assisted: false,
        }
    }

    pub(super) fn step(&self) -> LearningStep {
        self.step
    }

    pub(super) fn is_guided(&self) -> bool {
        self.guided
    }

    pub(super) fn view(&self) -> LearningView {
        self.view
    }

    pub(super) fn set_view(&mut self, view: LearningView) {
        self.view = view;
    }

    pub(super) fn current_lesson_id(&self) -> Option<&'static str> {
        self.queue.get(self.index).map(|item| item.lesson_id)
    }

    #[cfg(test)]
    fn queue_ids(&self) -> Vec<&'static str> {
        self.queue.iter().map(|item| item.lesson_id).collect()
    }

    pub(super) fn assisted(&self) -> bool {
        self.assisted
    }

    pub(super) fn can_judge(&self) -> bool {
        !self.guided || self.step == LearningStep::Exercise
    }

    pub(super) fn mark_assisted(&mut self) {
        if !self.guided
            || (self.queue.get(self.index).is_some()
                && matches!(
                    self.step,
                    LearningStep::Review
                        | LearningStep::Delta
                        | LearningStep::Predict
                        | LearningStep::Exercise
                ))
        {
            self.assisted = true;
        }
    }

    pub(super) fn finish_judge(&mut self, passed: bool) {
        self.assisted = false;
        if !self.guided || self.step == LearningStep::Complete || self.queue.is_empty() {
            return;
        }
        self.step = if passed {
            LearningStep::Reflect
        } else {
            LearningStep::Exercise
        };
        self.view = LearningView::Result;
    }

    pub(super) fn advance(&mut self) -> LearningAdvance {
        match self.step {
            LearningStep::Review | LearningStep::Delta => {
                self.step = LearningStep::Predict;
                self.view = LearningView::Lesson;
                LearningAdvance::Step
            }
            LearningStep::Predict => {
                self.step = LearningStep::Exercise;
                self.view = LearningView::Code;
                LearningAdvance::Step
            }
            LearningStep::Exercise => LearningAdvance::Blocked,
            LearningStep::Reflect if self.index + 1 < self.queue.len() => {
                self.assisted = false;
                self.index += 1;
                let item = self.queue[self.index];
                self.step = if item.review {
                    LearningStep::Review
                } else {
                    LearningStep::Delta
                };
                self.view = LearningView::Lesson;
                LearningAdvance::Item(item.lesson_id)
            }
            LearningStep::Reflect => {
                self.assisted = false;
                self.step = LearningStep::Complete;
                self.view = LearningView::Lesson;
                LearningAdvance::Complete
            }
            LearningStep::Complete => {
                self.assisted = false;
                LearningAdvance::Manual
            }
        }
    }

    pub(super) fn cycle_view(&mut self) {
        self.view = match self.view {
            LearningView::Lesson => LearningView::Code,
            LearningView::Code => LearningView::Result,
            LearningView::Result => LearningView::Lesson,
        };
    }
}

fn session_queue(
    state: &AppState,
    language: &str,
    lessons: &[&SyntaxLesson],
    due: &[&SyntaxLesson],
) -> Vec<LearningItem> {
    let mut queue = due
        .iter()
        .copied()
        .map(|lesson| LearningItem {
            lesson_id: lesson.id,
            review: true,
        })
        .collect::<Vec<_>>();
    if let Some(lesson) = lessons.iter().copied().find(|lesson| {
        lesson.track == SyntaxTrack::Core
            && state
                .syntax_mastery
                .get(language)
                .and_then(|mastery| mastery.get(lesson.id))
                .is_none_or(|progress| progress.stage == MasteryStage::New)
            && !queue.iter().any(|item| item.lesson_id == lesson.id)
    }) {
        queue.push(LearningItem {
            lesson_id: lesson.id,
            review: false,
        });
    }
    queue
}

pub(super) fn render_learning_step(
    lesson: Option<&SyntaxLesson>,
    state: &AppState,
    step: LearningStep,
) -> String {
    let language = &state.settings.ui_language;
    if step == LearningStep::Complete {
        return format!(
            "# {}\n\n{}",
            learning_step_label(language, step),
            ui_text(language, "learning_complete_body")
        );
    }
    let Some(lesson) = lesson else {
        return String::new();
    };
    let title = crate::core::localized_syntax_title(lesson, language);
    let body = match step {
        LearningStep::Review => crate::core::localized_syntax_objective(lesson, language),
        LearningStep::Delta => crate::core::localized_syntax_language_delta(lesson, language),
        LearningStep::Predict => crate::core::localized_syntax_prediction_prompt(lesson, language),
        LearningStep::Exercise => learning_exercise(lesson, language),
        LearningStep::Reflect => crate::core::localized_syntax_transfer_trap(lesson, language),
        LearningStep::Complete => unreachable!(),
    };
    format!(
        "# {}: {title}\n\n{body}",
        learning_step_label(language, step)
    )
}

fn learning_exercise(lesson: &SyntaxLesson, language: &str) -> String {
    let prompt = crate::core::localized_syntax_exercise_prompt(lesson, language);
    let Some(case) = lesson.exercise.cases.first() else {
        return prompt;
    };
    format!(
        "{}\n\n{}\n\n```\n{}\n```\n\n{}\n\n```\n{}\n```",
        prompt,
        ui_text(language, "input"),
        case.input.trim_end(),
        ui_text(language, "output"),
        case.output.trim_end()
    )
}

pub(super) fn progress_text(state: &AppState, now: u64) -> String {
    let language = normalize_language(&state.settings.language);
    let lessons = syntax_lessons_for(&language);
    let mastery = state.syntax_mastery.get(&language);
    let count = |stage| {
        lessons
            .iter()
            .filter(|lesson| lesson.track == SyntaxTrack::Core)
            .filter(|lesson| {
                mastery
                    .and_then(|items| items.get(lesson.id))
                    .is_some_and(|item| item.stage == stage)
            })
            .count()
    };
    let (_, core) = crate::core::syntax_core_progress_count(state, &language);
    let due = due_syntax_lesson_count(state, &language, now);
    let ui_language = &state.settings.ui_language;
    format!(
        "{}\n{}: {}\n{}: {core}\n{}: {}\n{}: {}\n{}: {}\n{}: {due}",
        ui_text(ui_language, "progress_title"),
        ui_text(ui_language, "progress_language"),
        syntax_language_name(&language),
        ui_text(ui_language, "progress_core"),
        ui_text(ui_language, "progress_practiced"),
        count(MasteryStage::Practiced),
        ui_text(ui_language, "progress_retained"),
        count(MasteryStage::Retained),
        ui_text(ui_language, "progress_mastered"),
        count(MasteryStage::Mastered),
        ui_text(ui_language, "progress_due"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
            exercise: crate::core::SyntaxExercise {
                prompt: id,
                starter: id,
                cases: &[],
            },
            refs: &[],
        }
    }

    fn state() -> AppState {
        AppState {
            current_problem: "001-hello-world".to_string(),
            settings: crate::core::Settings::default(),
            solved: Vec::new(),
            history: Vec::new(),
            suggested_next_difficulty: "easy".to_string(),
            syntax_progress: HashMap::new(),
            current_syntax_lesson: HashMap::new(),
            syntax_mastery: HashMap::new(),
            completed_syntax_courses: Vec::new(),
        }
    }

    #[test]
    fn synthetic_checkpoint_and_capstone_are_queued_and_ai_cannot_grant_capstone_progress() {
        let lab = lesson("lab", SyntaxTrack::Lab, SyntaxKind::Lesson);
        let checkpoint = lesson("checkpoint", SyntaxTrack::Core, SyntaxKind::Checkpoint);
        let capstone = lesson("capstone", SyntaxTrack::Core, SyntaxKind::Capstone);
        let lessons = [&lab, &checkpoint, &capstone];
        let mut state = state();

        let checkpoint_queue = session_queue(&state, "rust", &lessons, &[]);
        assert_eq!(checkpoint_queue[0].lesson_id, "checkpoint");
        state
            .syntax_mastery
            .entry("rust".to_string())
            .or_default()
            .insert(
                "checkpoint".to_string(),
                crate::core::LessonMastery {
                    stage: MasteryStage::Practiced,
                    review_due_at: u64::MAX,
                    attempts: 1,
                },
            );
        let capstone_queue = session_queue(&state, "rust", &lessons, &[]);
        assert_eq!(capstone_queue[0].lesson_id, "capstone");

        let mut session = LearningSession {
            guided: true,
            queue: capstone_queue,
            index: 0,
            step: LearningStep::Exercise,
            view: LearningView::Code,
            assisted: false,
        };

        session.mark_assisted();
        let assisted = session.assisted();
        crate::core::record_syntax_result_for_lessons(
            &mut state, "rust", "capstone", true, 1_000, assisted, &lessons,
        );
        session.finish_judge(true);

        assert!(!session.assisted());
        assert_eq!(session.step(), LearningStep::Reflect);
        assert_eq!(
            state.syntax_mastery["rust"]["capstone"].stage,
            MasteryStage::New
        );

        crate::core::record_syntax_result_for_lessons(
            &mut state, "rust", "capstone", true, 1_000, false, &lessons,
        );
        assert_eq!(
            state.syntax_mastery["rust"]["capstone"].stage,
            MasteryStage::Practiced
        );
    }

    #[test]
    fn assistance_lives_only_until_the_current_item_judge_or_boundary() {
        let queue = vec![
            LearningItem {
                lesson_id: "first",
                review: false,
            },
            LearningItem {
                lesson_id: "second",
                review: false,
            },
        ];
        let mut session = LearningSession {
            guided: true,
            queue,
            index: 0,
            step: LearningStep::Delta,
            view: LearningView::Code,
            assisted: false,
        };

        for step in [
            LearningStep::Review,
            LearningStep::Delta,
            LearningStep::Predict,
            LearningStep::Exercise,
        ] {
            session.step = step;
            session.assisted = false;
            session.mark_assisted();
            assert!(session.assisted(), "{step:?}");
        }
        session.step = LearningStep::Delta;
        session.assisted = false;
        session.mark_assisted();
        assert!(session.assisted());
        assert!(matches!(session.advance(), LearningAdvance::Step));
        assert_eq!(session.step(), LearningStep::Predict);
        assert!(session.assisted());
        assert!(matches!(session.advance(), LearningAdvance::Step));
        assert_eq!(session.step(), LearningStep::Exercise);
        assert!(session.assisted());

        session.finish_judge(true);
        assert_eq!(session.step(), LearningStep::Reflect);
        assert!(!session.assisted());
        session.mark_assisted();
        assert!(!session.assisted());

        session.assisted = true;
        assert!(matches!(session.advance(), LearningAdvance::Item("second")));
        assert!(!session.assisted());
        session.step = LearningStep::Reflect;
        session.assisted = true;
        assert!(matches!(session.advance(), LearningAdvance::Complete));
        assert!(!session.assisted());
        session.assisted = true;
        assert!(matches!(session.advance(), LearningAdvance::Manual));
        assert!(!session.assisted());

        let mut inactive = LearningSession::inactive();
        inactive.mark_assisted();
        assert!(inactive.assisted());
        inactive.finish_judge(true);
        assert!(!inactive.assisted());
    }

    #[test]
    fn manual_assisted_capstone_pass_cannot_advance_mastery() {
        let capstone = lesson("capstone", SyntaxTrack::Core, SyntaxKind::Capstone);
        let lessons = [&capstone];
        let mut state = state();
        let mut session = LearningSession::inactive();

        session.mark_assisted();
        crate::core::record_syntax_result_for_lessons(
            &mut state,
            "rust",
            "capstone",
            true,
            1_000,
            session.assisted(),
            &lessons,
        );
        session.finish_judge(true);

        assert_eq!(state.syntax_mastery["rust"]["capstone"].attempts, 1);
        assert_eq!(
            state.syntax_mastery["rust"]["capstone"].stage,
            MasteryStage::New
        );
        assert!(!session.assisted());
    }

    #[test]
    fn guided_judge_gate_allows_only_exercise_while_inactive_browsing_can_run() {
        let mut session = LearningSession {
            guided: true,
            queue: vec![LearningItem {
                lesson_id: "lesson",
                review: false,
            }],
            index: 0,
            step: LearningStep::Review,
            view: LearningView::Code,
            assisted: false,
        };

        for step in [
            LearningStep::Review,
            LearningStep::Delta,
            LearningStep::Predict,
            LearningStep::Reflect,
            LearningStep::Complete,
        ] {
            session.step = step;
            assert!(!session.can_judge(), "{step:?}");
        }
        session.step = LearningStep::Exercise;
        assert!(session.can_judge());
        assert!(LearningSession::inactive().can_judge());
    }

    #[test]
    fn session_queue_orders_two_due_reviews_then_one_new_core_item() {
        let mut state = state();
        state.syntax_mastery.insert(
            "python".to_string(),
            [
                ("py-output", 300),
                ("py-variables", 100),
                ("py-numbers", 200),
            ]
            .into_iter()
            .map(|(id, review_due_at)| {
                (
                    id.to_string(),
                    crate::core::LessonMastery {
                        stage: MasteryStage::Practiced,
                        review_due_at,
                        attempts: 1,
                    },
                )
            })
            .collect(),
        );

        let session = LearningSession::start(&state, "python", 100_000);

        assert_eq!(
            session.queue_ids(),
            ["py-variables", "py-numbers", "py-strings"]
        );
        assert_eq!(session.step(), LearningStep::Review);
    }

    #[test]
    fn learning_view_cycles_code_result_lesson() {
        let mut session = LearningSession {
            guided: true,
            queue: vec![LearningItem {
                lesson_id: "lesson",
                review: false,
            }],
            index: 0,
            step: LearningStep::Delta,
            view: LearningView::Code,
            assisted: false,
        };

        session.cycle_view();
        assert_eq!(session.view(), LearningView::Result);
        session.cycle_view();
        assert_eq!(session.view(), LearningView::Lesson);
        session.cycle_view();
        assert_eq!(session.view(), LearningView::Code);
    }
}
