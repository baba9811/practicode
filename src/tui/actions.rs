use super::*;
use crate::core::{JudgeFailureKind, JudgeResult};

fn judge_headline(result: &JudgeResult, language: &str) -> String {
    let failure = result.failure_kind.map(|kind| {
        ui_text(
            language,
            match kind {
                JudgeFailureKind::Compile => "judge_failure_compile",
                JudgeFailureKind::TypeCheck => "judge_failure_typecheck",
                JudgeFailureKind::Runtime => "judge_failure_runtime",
                JudgeFailureKind::Timeout => "judge_failure_timeout",
                JudgeFailureKind::Output => "judge_failure_output",
            },
        )
    });
    format!(
        "{} {}/{}{}",
        ui_text(
            language,
            if result.passed {
                "result_pass"
            } else {
                "result_fail"
            }
        ),
        result.passed_cases,
        result.total_cases,
        failure.map(|kind| format!(" [{kind}]")).unwrap_or_default()
    )
}

fn judge_label_key(label: &str) -> Option<&'static str> {
    match label {
        "Input" => Some("judge_input"),
        "Expected" => Some("judge_expected"),
        "Got" => Some("judge_got"),
        "Stdout" => Some("judge_stdout"),
        "Stderr" => Some("judge_stderr"),
        "Error" => Some("judge_error"),
        "Compile" => Some("judge_compile"),
        _ => None,
    }
}

fn localized_judge_result_output(result: &JudgeResult, language: &str) -> String {
    let output = &result.output;
    if output == "problem has no judge cases" {
        return ui_text(language, "judge_no_cases").to_string();
    }
    if let Some(tool) = output.strip_prefix("Missing runtime for TypeScript: ")
        && matches!(tool, "tsc" | "node")
    {
        return ui_text(language, "judge_missing_typescript_tool").replace("{tool}", tool);
    }
    if let Some(runtime) = output.strip_prefix("Missing runtime for ")
        && LANGUAGES.contains(&runtime)
    {
        return ui_text(language, "judge_missing_runtime").replace("{runtime}", runtime);
    }
    if matches!(
        result.failure_kind,
        Some(JudgeFailureKind::Compile | JudgeFailureKind::TypeCheck)
    ) {
        return output.clone();
    }
    let structured = output.lines().any(|line| {
        line.strip_prefix("Case ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(case, outcome)| {
                case.parse::<usize>().is_ok() && matches!(outcome, "PASS" | "FAIL")
            })
    });
    if !structured {
        return output.to_string();
    }
    let mut body_label = "";
    output
        .lines()
        .map(|line| {
            if let Some((case, outcome)) = line
                .strip_prefix("Case ")
                .and_then(|rest| rest.split_once(": "))
                && case.parse::<usize>().is_ok()
                && matches!(outcome, "PASS" | "FAIL")
            {
                body_label = "";
                let outcome = ui_text(
                    language,
                    if outcome == "PASS" {
                        "result_pass"
                    } else {
                        "result_fail"
                    },
                );
                return format!("{} {case}: {outcome}", ui_text(language, "judge_case"));
            }
            if let Some(label) = line.strip_suffix(": <empty>")
                && let Some(key) = judge_label_key(label)
            {
                body_label = "";
                return format!(
                    "{}: {}",
                    ui_text(language, key),
                    ui_text(language, "empty_value")
                );
            }
            if let Some(key) = judge_label_key(line) {
                body_label = line;
                return ui_text(language, key).to_string();
            }
            if matches!(body_label, "Input" | "Expected") && line == "  <hidden>" {
                return format!("  <{}>", ui_text(language, "judge_hidden"));
            }
            if body_label == "Error" && line == "  timeout: 5s" {
                return format!("  {}", ui_text(language, "judge_timeout_detail"));
            }
            if body_label == "Error"
                && let Some(status) = line.strip_prefix("  process exited with status ")
                && (status == "unknown" || status.parse::<i32>().is_ok())
            {
                let status = if status == "unknown" {
                    ui_text(language, "judge_unknown_status")
                } else {
                    status
                };
                return format!(
                    "  {}",
                    ui_text(language, "judge_process_exit").replace("{status}", status)
                );
            }
            if !line.is_empty() && !line.starts_with("  ") {
                body_label = "";
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn learning_state_text(
    lesson: &crate::core::SyntaxLesson,
    mastery: &crate::core::LessonMastery,
    now: u64,
    language: &str,
) -> String {
    let stage = ui_text(
        language,
        match mastery.stage {
            crate::core::MasteryStage::New => "mastery_new",
            crate::core::MasteryStage::Practiced => "mastery_practiced",
            crate::core::MasteryStage::Retained => "mastery_retained",
            crate::core::MasteryStage::Mastered => "mastery_mastered",
        },
    );
    let review_due_at = if lesson.track == crate::core::SyntaxTrack::Core {
        syntax_review_due_at(mastery, now)
    } else {
        None
    };
    let review = if let Some(due_at) = review_due_at {
        let days = due_at.saturating_sub(now).saturating_add(86_399) / 86_400;
        format!("{}: {days}", ui_text(language, "result_review_days"))
    } else {
        ui_text(language, "result_retry_no_review").to_string()
    };
    format!("{}: {stage}\n{review}", ui_text(language, "result_mastery"))
}

impl PracticodeApp {
    pub(super) fn transition_mode(&mut self, mode: AppMode) {
        if self.mode == AppMode::Learn && mode != AppMode::Learn {
            self.learning_session = LearningSession::inactive();
        }
        self.mode = mode;
    }

    pub(super) fn home_text(&self) -> String {
        let lang = &self.state.settings.ui_language;
        let learn_label = ui_text(lang, "home_learn_choice");
        let practice_label = ui_text(lang, "home_practice_choice");
        let help = ui_text(lang, "home_help");
        let learn = if self.home_choice == HomeChoice::Learn {
            format!("> {learn_label}")
        } else {
            format!("  {learn_label}")
        };
        let problems = if self.home_choice == HomeChoice::Problems {
            format!("> {practice_label}")
        } else {
            format!("  {practice_label}")
        };
        format!(
            "Practicode\n\n{learn}\n  {}\n\n{problems}\n  {}\n\n{help}",
            ui_text(lang, "home_learn_description"),
            ui_text(lang, "home_practice_description")
        )
    }

    pub(super) fn action_home(&mut self) -> Result<()> {
        self.transition_mode(AppMode::Home);
        self.state.settings.start_mode = "home".to_string();
        save_state(&self.root, &self.state)?;
        self.learn_result.clear();
        self.left_scroll = 0;
        self.output_scroll = 0;
        self.show_output = false;
        self.settings_cursor = None;
        self.list_cursor = None;
        self.focus = Focus::Home;
        self.output = self.home_text();
        self.output_is_markdown = false;
        Ok(())
    }

    pub(super) fn action_practice(&mut self) -> Result<()> {
        self.transition_mode(AppMode::Problems);
        self.practice_view = PracticeView::Problem;
        self.state.settings.start_mode = "problems".to_string();
        save_state(&self.root, &self.state)?;
        self.load_code_editor()?;
        self.settings_cursor = None;
        self.list_cursor = None;
        self.show_output = false;
        self.focus = Focus::Left;
        Ok(())
    }

    pub(super) fn move_home_choice(&mut self) {
        self.home_choice = match self.home_choice {
            HomeChoice::Learn => HomeChoice::Problems,
            HomeChoice::Problems => HomeChoice::Learn,
        };
        self.output = self.home_text();
    }

    pub(super) fn open_home_choice(&mut self) -> Result<()> {
        match self.home_choice {
            HomeChoice::Learn => self.action_learn(""),
            HomeChoice::Problems => self.action_practice(),
        }
    }

    pub(super) fn action_edit(&mut self) -> Result<()> {
        if self.mode == AppMode::Home {
            self.action_practice()?;
        }
        self.editing_notes = false;
        self.load_code_editor()?;
        if self.mode == AppMode::Learn {
            self.learning_session.set_view(LearningView::Code);
            self.show_current_syntax_lesson();
            return Ok(());
        }
        self.settings_cursor = None;
        self.left_scroll = 0;
        self.output_scroll = 0;
        self.show_output = false;
        self.practice_view = PracticeView::Code;
        self.focus = Focus::Code;
        Ok(())
    }

    pub(super) fn action_run(&mut self) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.action_exercise();
        }
        if self.mode == AppMode::Home {
            self.transition_mode(AppMode::Problems);
            self.state.settings.start_mode = "problems".to_string();
            save_state(&self.root, &self.state)?;
        }
        self.save_code()?;
        let result = judge(&self.root, &self.problem, &self.state.settings);
        if result.passed {
            record_pass(&self.root, &self.problem, &mut self.state)?;
        }
        let headline = judge_headline(&result, &self.state.settings.ui_language);
        let next_step = if result.passed {
            ui_text(&self.state.settings.ui_language, "run_pass_next")
        } else {
            ui_text(&self.state.settings.ui_language, "run_fail_next")
        };
        let detail = localized_judge_result_output(&result, &self.state.settings.ui_language);
        self.write_text_output(&format!("{headline}\n{detail}\n\n{next_step}"));
        Ok(())
    }

    pub(super) fn action_next(&mut self, request: &str) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.action_next_learning();
        }
        self.transition_mode(AppMode::Problems);
        self.state.settings.start_mode = "problems".to_string();
        save_state(&self.root, &self.state)?;
        self.check_background_generation();
        let request = request.trim();
        let old_problem = self.state.current_problem.clone();
        if let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)? {
            self.generate_notice = None;
            self.problem = problem;
            self.load_code_editor()?;
            self.settings_cursor = None;
            self.show_output = false;
            self.practice_view = PracticeView::Problem;
            self.focus = Focus::Left;
            return Ok(());
        }
        if self.generate_rx.is_some() {
            self.write_text_output(ui_text(
                &self.state.settings.ui_language,
                "generation_already_running",
            ));
            return Ok(());
        }
        self.start_next_problem(old_problem, true, request.to_string());
        Ok(())
    }

    pub(super) fn action_generate(&mut self, request: &str) {
        self.check_background_generation();
        if self.task_rx.is_some() || self.generate_rx.is_some() {
            let notice = GenerationNotice::Duplicate;
            let message = self.generation_notice_text(&notice);
            self.generate_notice = Some(notice);
            self.write_text_output(&message);
            return;
        }
        self.start_background_generation(request.trim().to_string());
    }

    pub(super) fn start_background_generation(&mut self, request: String) {
        self.transition_mode(AppMode::Problems);
        self.state.settings.start_mode = "problems".to_string();
        if let Err(error) = save_state(&self.root, &self.state) {
            self.write_text_output(&format!(
                "{}\n{error}",
                ui_text(&self.state.settings.ui_language, "generation_save_failed")
            ));
            return;
        }
        let root = self.root.clone();
        let state = self.state.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(run_ai_generate_result(&root, &state, &request));
        });
        self.generate_bank_len = self.bank.len();
        self.generate_started = Some(Instant::now());
        self.generate_notice = Some(GenerationNotice::Started);
        self.generate_rx = Some(rx);
        self.settings_cursor = None;
        self.left_scroll = 0;
        self.output_scroll = 0;
        self.show_output = false;
        self.focus = Focus::Code;
    }

    pub(super) fn start_next_problem(
        &mut self,
        old_problem: String,
        fallback_to_local: bool,
        request: String,
    ) {
        if self.task_rx.is_some() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "already_busy"));
            return;
        }
        self.start_busy("next", "");
        let root = self.root.clone();
        let state = self.state.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_ai_next(&root, &state, true, &request);
            let _ = tx.send(TaskResult::Next {
                output,
                old_problem,
                fallback_to_local,
            });
        });
        self.task_rx = Some(rx);
    }

    pub(super) fn finish_next_problem(
        &mut self,
        output: String,
        old_problem: String,
        fallback_to_local: bool,
    ) -> Result<()> {
        self.bank = load_bank(&self.root)?;
        self.state = load_state(&self.root, &self.bank)?;
        self.problem = problem_by_id(&self.bank, &self.state.current_problem)
            .cloned()
            .unwrap_or_else(|| self.bank[0].clone());
        if self.state.current_problem == old_problem {
            if fallback_to_local
                && let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)?
            {
                self.problem = problem;
            } else {
                let unavailable = ui_text(&self.state.settings.ui_language, "next_unavailable");
                self.write_text_output(&format!(
                    "{}{}{unavailable}",
                    if output.is_empty() { "" } else { &output },
                    if output.is_empty() { "" } else { "\n\n" }
                ));
                return Ok(());
            }
        }
        self.load_code_editor()?;
        self.settings_cursor = None;
        self.show_output = false;
        self.practice_view = PracticeView::Problem;
        self.focus = Focus::Left;
        Ok(())
    }

    pub(super) fn action_previous(&mut self) -> Result<()> {
        if self.mode == AppMode::Learn {
            self.learning_session = LearningSession::inactive();
            return self.action_prev_lesson();
        }
        if self.mode == AppMode::Home {
            self.transition_mode(AppMode::Problems);
            self.state.settings.start_mode = "problems".to_string();
            save_state(&self.root, &self.state)?;
        }
        let old_problem = self.state.current_problem.clone();
        self.problem = previous_problem(&self.root, &self.bank, &mut self.state)?;
        if self.state.current_problem == old_problem {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "first_problem"));
        } else {
            self.load_code_editor()?;
            self.settings_cursor = None;
            self.show_output = false;
            self.practice_view = PracticeView::Problem;
            self.focus = Focus::Left;
        }
        Ok(())
    }

    pub(super) fn action_give_up(&mut self) -> Result<()> {
        if self.mode == AppMode::Home {
            self.transition_mode(AppMode::Problems);
            self.state.settings.start_mode = "problems".to_string();
            save_state(&self.root, &self.state)?;
        }
        let answer = give_up(&self.root, &self.problem, &mut self.state)?;
        let language = normalize_language(&self.state.settings.language);
        let heading = ui_text(&self.state.settings.ui_language, "answer_for_language")
            .replace("{language}", &language);
        self.write_output(&format!(
            "{heading}\n\n```{language}\n{}\n```",
            answer.trim_end()
        ));
        Ok(())
    }

    pub(super) fn action_learn(&mut self, language: &str) -> Result<()> {
        let language = language.trim();
        if !language.is_empty() && !LANGUAGES.contains(&language) {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "learn_usage"));
            return Ok(());
        }
        if !language.is_empty() {
            self.state.settings.language = language.to_string();
        }
        self.transition_mode(AppMode::Learn);
        self.state.settings.start_mode = "learn".to_string();
        self.learn_result.clear();
        self.left_scroll = 0;
        self.output_scroll = 0;
        let language = self.state.settings.language.clone();
        self.learning_session = LearningSession::start(&self.state, &language, unix_time_now());
        if let Some(lesson_id) = self.learning_session.current_lesson_id() {
            set_current_syntax_lesson(&mut self.state, &language, lesson_id);
        }
        self.load_syntax_editor()?;
        save_state(&self.root, &self.state)?;
        self.show_current_syntax_lesson();
        Ok(())
    }

    pub(super) fn action_exercise(&mut self) -> Result<()> {
        if self.mode != AppMode::Learn {
            self.action_learn("")?;
        }
        if !self.learning_session.can_judge() {
            self.learn_result =
                ui_text(&self.state.settings.ui_language, "learning_run_gate").to_string();
            self.learning_session.set_view(LearningView::Result);
            self.show_current_syntax_lesson();
            return Ok(());
        }
        self.save_syntax_code()?;
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let path = ensure_syntax_submission(&self.root, lesson)?;
        let cases = syntax_cases(lesson);
        let result = judge_path(
            &self.root,
            &format!(".syntax-{}-{}", lesson.language, lesson.id),
            &path,
            lesson.language,
            &cases,
        );
        let assisted = self.learning_session.assisted();
        let now = unix_time_now();
        record_syntax_result(
            &mut self.state,
            lesson.language,
            lesson.id,
            result.passed,
            now,
            assisted,
        );
        let learning_state = learning_state_text(
            lesson,
            &self.state.syntax_mastery[lesson.language][lesson.id],
            now,
            &self.state.settings.ui_language,
        );
        self.learning_session.finish_judge(result.passed);
        save_state(&self.root, &self.state)?;
        self.output_scroll = 0;
        let headline = judge_headline(&result, &self.state.settings.ui_language);
        let next_step = ui_text(
            &self.state.settings.ui_language,
            if result.passed {
                "run_pass_next"
            } else {
                "run_fail_next"
            },
        );
        self.learn_result = format!(
            "{headline}\n{}\n\n{learning_state}\n\n{next_step}",
            localized_judge_result_output(&result, &self.state.settings.ui_language).trim_end()
        );
        self.show_current_syntax_lesson();
        Ok(())
    }

    pub(super) fn action_next_learning(&mut self) -> Result<()> {
        match self.learning_session.advance() {
            LearningAdvance::Step | LearningAdvance::Blocked | LearningAdvance::Complete => {
                self.show_current_syntax_lesson();
            }
            LearningAdvance::Item(lesson_id) => {
                let language = self.state.settings.language.clone();
                set_current_syntax_lesson(&mut self.state, &language, lesson_id);
                self.load_syntax_editor()?;
                save_state(&self.root, &self.state)?;
                self.learn_result.clear();
                self.show_current_syntax_lesson();
            }
            LearningAdvance::Manual => {
                self.learning_session = LearningSession::inactive();
                self.action_next_lesson()?;
            }
        }
        Ok(())
    }

    pub(super) fn action_next_lesson(&mut self) -> Result<()> {
        self.transition_mode(AppMode::Learn);
        self.learning_session = LearningSession::inactive();
        let language = self.state.settings.language.clone();
        next_syntax_lesson(&mut self.state, &language, 1);
        self.load_syntax_editor()?;
        save_state(&self.root, &self.state)?;
        self.learn_result.clear();
        self.left_scroll = 0;
        self.output_scroll = 0;
        self.show_current_syntax_lesson();
        Ok(())
    }

    pub(super) fn action_prev_lesson(&mut self) -> Result<()> {
        self.transition_mode(AppMode::Learn);
        self.learning_session = LearningSession::inactive();
        let language = self.state.settings.language.clone();
        next_syntax_lesson(&mut self.state, &language, -1);
        self.load_syntax_editor()?;
        save_state(&self.root, &self.state)?;
        self.learn_result.clear();
        self.left_scroll = 0;
        self.output_scroll = 0;
        self.show_current_syntax_lesson();
        Ok(())
    }

    pub(super) fn show_current_syntax_lesson(&mut self) {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        self.output = if self.learning_session.is_guided() {
            render_learning_step(Some(lesson), &self.state, self.learning_session.step())
        } else {
            render_syntax_lesson(lesson, &self.state)
        };
        self.left_scroll = 0;
        self.output_is_markdown = true;
        self.show_output = false;
        self.settings_cursor = None;
        self.list_cursor = None;
        self.focus = match self.learning_session.view() {
            LearningView::Lesson => Focus::Left,
            LearningView::Code => Focus::Code,
            LearningView::Result => Focus::Output,
        };
    }

    pub(super) fn action_lesson(&mut self) {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let output = render_syntax_lesson(lesson, &self.state);
        self.write_output(&output);
    }

    pub(super) fn action_progress(&mut self) {
        let output = progress_text(&self.state, unix_time_now());
        self.write_text_output(&output);
    }

    pub(super) fn cycle_learning_view(&mut self) {
        if self.mode == AppMode::Problems {
            self.practice_view = match self.practice_view {
                PracticeView::Problem => PracticeView::Code,
                PracticeView::Code => PracticeView::Problem,
            };
            self.focus = match self.practice_view {
                PracticeView::Problem => Focus::Left,
                PracticeView::Code => Focus::Code,
            };
        } else if self.mode == AppMode::Learn {
            self.learning_session.cycle_view();
            self.show_current_syntax_lesson();
        }
    }

    pub(super) fn action_cycle_language(&mut self) -> Result<()> {
        let current = LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.language)
            .unwrap_or(0);
        self.set_language(LANGUAGES[(current + 1) % LANGUAGES.len()])
    }

    pub(super) fn action_toggle_ui_language(&mut self) -> Result<()> {
        let current = UI_LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.ui_language)
            .unwrap_or(0);
        self.set_ui_language(UI_LANGUAGES[(current + 1) % UI_LANGUAGES.len()])
    }

    pub(super) fn action_toggle_theme(&mut self) -> Result<()> {
        let current = THEMES
            .iter()
            .position(|theme| theme == &self.state.settings.theme)
            .unwrap_or(0);
        self.set_theme(THEMES[(current + 1) % THEMES.len()])
    }

    pub(super) fn set_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.language = language.to_string();
        if self.mode == AppMode::Learn {
            return self.action_learn(language);
        }
        self.load_code_editor()?;
        save_state(&self.root, &self.state)?;
        self.settings_cursor = None;
        self.show_output = false;
        self.focus = match self.practice_view {
            PracticeView::Problem => Focus::Left,
            PracticeView::Code => Focus::Code,
        };
        Ok(())
    }

    pub(super) fn set_ui_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.ui_language = normalize_ui_language(language);
        save_state(&self.root, &self.state)?;
        if self.mode == AppMode::Learn {
            self.left_scroll = 0;
            self.show_current_syntax_lesson();
        } else {
            self.write_text_output(
                &ui_text(&self.state.settings.ui_language, "ui_language_set")
                    .replace("{language}", &self.state.settings.ui_language),
            );
        }
        Ok(())
    }

    pub(super) fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.state.settings.theme = theme.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(
            &ui_text(&self.state.settings.ui_language, "theme_set").replace("{theme}", theme),
        );
        Ok(())
    }

    pub(super) fn set_difficulty(&mut self, difficulty: &str) -> Result<()> {
        let difficulty = difficulty.trim().to_lowercase();
        if !DIFFICULTIES.contains(&difficulty.as_str()) {
            self.write_text_output(ui_text(
                &self.state.settings.ui_language,
                "difficulty_options",
            ));
            return Ok(());
        }
        let normalized = normalize_difficulty(&difficulty);
        self.state.settings.difficulty = normalized.clone();
        if normalized != "auto" {
            self.state.suggested_next_difficulty = normalized;
        }
        save_state(&self.root, &self.state)?;
        self.show_profile();
        Ok(())
    }

    pub(super) fn set_topics(&mut self, topics: &str, avoid: bool) -> Result<()> {
        let topics = parse_topic_list(topics);
        if avoid {
            self.state.settings.avoid_topics = topics;
        } else {
            self.state.settings.topics = topics;
        }
        save_state(&self.root, &self.state)?;
        self.show_profile();
        Ok(())
    }

    pub(super) fn set_generate_languages(&mut self, value: &str, ui: bool) -> Result<()> {
        let all = value.trim().eq_ignore_ascii_case("all");
        let valid = all
            || value.split(',').all(|language| {
                let language = language.trim().to_lowercase();
                if ui {
                    language
                        .split(['-', '_'])
                        .next()
                        .is_some_and(|language| UI_LANGUAGES.contains(&language))
                } else {
                    LANGUAGES.contains(&language.as_str())
                }
            });
        if !valid {
            self.show_profile();
            return Ok(());
        }
        if ui {
            self.state.settings.generate_ui_languages = parse_ui_language_list(value);
        } else {
            self.state.settings.generate_languages = parse_language_list(value);
        }
        save_state(&self.root, &self.state)?;
        self.show_profile();
        Ok(())
    }

    pub(super) fn set_ai_effort(&mut self, effort: &str) -> Result<()> {
        let effort = effort.trim().to_lowercase();
        let allowed = if self.state.settings.ai_provider == "claude" {
            CLAUDE_AI_EFFORTS
        } else {
            CODEX_AI_EFFORTS
        };
        if !(allowed.contains(&effort.as_str())
            || self.state.settings.ai_provider == "codex" && effort == "max")
        {
            self.write_model_status();
            return Ok(());
        }
        self.state.settings.ai_effort =
            normalize_ai_effort(&self.state.settings.ai_provider, &effort);
        save_state(&self.root, &self.state)?;
        self.write_model_status();
        Ok(())
    }

    pub(super) fn reset_profile(&mut self) -> Result<()> {
        self.state.settings.difficulty = "auto".to_string();
        self.state.settings.topics.clear();
        self.state.settings.avoid_topics.clear();
        self.state.settings.generate_languages.clear();
        self.state.settings.generate_ui_languages.clear();
        save_state(&self.root, &self.state)?;
        self.show_profile();
        Ok(())
    }

    pub(super) fn show_profile(&mut self) {
        self.show_profile_with_intro("");
    }

    pub(super) fn show_profile_with_intro(&mut self, intro: &str) {
        self.editing_notes = false;
        self.showing_model_status = false;
        if self.settings_cursor.is_none() {
            self.settings_cursor = Some(0);
        }
        let profile = self.profile_text();
        self.output = if intro.trim().is_empty() {
            profile
        } else {
            format!("{}\n\n{profile}", intro.trim_end())
        };
        self.output_scroll = 0;
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn profile_text(&self) -> String {
        settings_panel::render(
            &self.state,
            self.settings_cursor,
            &self.available_models,
            self.model_rx.is_some(),
        )
    }

    pub(super) fn settings_row_count(&self) -> usize {
        settings_panel::row_count()
    }

    pub(super) fn move_settings_cursor(&mut self, delta: isize) {
        let len = self.settings_row_count() as isize;
        let cursor = self.settings_cursor.unwrap_or(0) as isize;
        self.settings_cursor = Some(((cursor + delta).rem_euclid(len)) as usize);
        self.show_profile();
    }

    pub(super) fn change_selected_setting(&mut self) -> Result<()> {
        let Some(row) = self.settings_cursor else {
            return Ok(());
        };
        if row == settings_panel::AI_MODEL_ROW
            && self.available_models_provider != self.state.settings.ai_provider
        {
            self.start_model_check();
            self.check_models();
            if self.model_rx.is_some() {
                self.show_profile();
                return Ok(());
            }
        }
        let change = settings_panel::apply_selected(&mut self.state, row, &self.available_models);
        if change.edit_notes {
            self.start_note_editor()?;
            return Ok(());
        }
        if change.provider_changed {
            self.model_rx = None;
            self.available_models.clear();
            self.available_models_provider.clear();
            self.model_message = None;
        }
        if change.reload_editor {
            self.load_code_editor()?;
        }
        save_state(&self.root, &self.state)?;
        self.show_profile();
        Ok(())
    }

    pub(super) fn start_note_editor(&mut self) -> Result<()> {
        self.save_code()?;
        let notes = read_problem_notes(&self.root)?;
        self.note_editor.set_text(&notes);
        self.settings_cursor = None;
        self.showing_model_status = false;
        self.editing_notes = true;
        self.output_scroll = 0;
        self.show_output = true;
        self.focus = Focus::Output;
        Ok(())
    }

    pub(super) fn save_notes(&self) -> Result<()> {
        let path = self.root.join(PROBLEM_NOTES_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = self.note_editor.text();
        let text = text.trim_end();
        save_user_text(&path, if text.is_empty() { "" } else { text })?;
        Ok(())
    }

    pub(super) fn close_note_editor(&mut self) -> Result<()> {
        self.save_notes()?;
        self.editing_notes = false;
        self.show_profile();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_lesson(track: crate::core::SyntaxTrack) -> crate::core::SyntaxLesson {
        crate::core::SyntaxLesson {
            id: "synthetic",
            aliases: &[],
            language: "rust",
            track,
            kind: crate::core::SyntaxKind::Lesson,
            level: "basic",
            title: "synthetic",
            body: "synthetic",
            example: "fn main() {}",
            exercise: crate::core::SyntaxExercise {
                prompt: "synthetic",
                starter: "fn main() {}",
                cases: &[],
            },
            refs: &[],
        }
    }

    fn localize(output: &str, kind: Option<JudgeFailureKind>) -> String {
        localized_judge_result_output(
            &JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: 1,
                failure_kind: kind,
                output: output.to_string(),
            },
            "ko",
        )
    }

    #[test]
    fn judge_localization_preserves_tool_owned_compiler_diagnostics() {
        let raw = "Case 1: compiler-owned diagnostic\nTS2322: raw detail";
        assert_eq!(localize(raw, Some(JudgeFailureKind::TypeCheck)), raw);
        assert_eq!(
            localize(
                "Case 1: FAIL\n\nGot\n  value",
                Some(JudgeFailureKind::Output)
            ),
            "케이스 1: 실패\n\n실제 출력\n  value"
        );

        let structured_looking = "Case 1: FAIL\nTS2322: compiler-owned detail";
        let result = JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: 1,
            failure_kind: Some(JudgeFailureKind::TypeCheck),
            output: structured_looking.to_string(),
        };
        assert_eq!(
            localized_judge_result_output(&result, "ko"),
            structured_looking
        );

        let missing_tool = "Missing runtime for TypeScript: tsc 5.9";
        assert_eq!(
            localize(missing_tool, Some(JudgeFailureKind::TypeCheck)),
            missing_tool
        );

        for (tool, kind) in [
            ("tsc", JudgeFailureKind::TypeCheck),
            ("node", JudgeFailureKind::Runtime),
        ] {
            let raw = format!("Missing runtime for TypeScript: {tool}");
            assert_eq!(
                localize(&raw, Some(kind)),
                format!("TypeScript 런타임이 없습니다: {tool}")
            );
        }
    }

    #[test]
    fn judge_localization_preserves_indented_program_output_byte_for_byte() {
        let output = "Case 1: PASS\n\nGot\n  <empty>\n\nStdout\n  <hidden>";

        assert_eq!(
            localize(output, None),
            "케이스 1: 통과\n\n실제 출력\n  <empty>\n\n표준 출력\n  <hidden>"
        );

        let non_numeric_case = "Case literal: PASS\n\nGot\n  value";
        assert_eq!(localize(non_numeric_case, None), non_numeric_case);
    }

    #[test]
    fn judge_localization_uses_provenance_for_empty_and_hidden_markers() {
        let output = "Case 1: FAIL\n\nGot: <empty>\n\nInput\n  <hidden>\n\nExpected\n  <hidden>";

        assert_eq!(
            localize(output, Some(JudgeFailureKind::Output)),
            "케이스 1: 실패\n\n실제 출력: <비어 있음>\n\n입력\n  <숨김>\n\n기대 출력\n  <숨김>"
        );
    }

    #[test]
    fn judge_localization_translates_only_known_error_provenance() {
        let output = "Case 1: FAIL\n\nError\n  process exited with status 7\n\nGot\n  literal";
        assert_eq!(
            localize(output, Some(JudgeFailureKind::Runtime)),
            "케이스 1: 실패\n\n오류\n  프로세스 종료 상태 7\n\n실제 출력\n  literal"
        );

        let unknown = "Case 1: FAIL\n\nError\n  No such file or directory";
        assert_eq!(
            localize(unknown, Some(JudgeFailureKind::Runtime)),
            "케이스 1: 실패\n\n오류\n  No such file or directory"
        );

        let malformed = "Case 1: FAIL\n\nError\n  process exited with status 7 extra";
        assert_eq!(
            localize(malformed, Some(JudgeFailureKind::Runtime)),
            "케이스 1: 실패\n\n오류\n  process exited with status 7 extra"
        );

        let unknown_status = "Case 1: FAIL\n\nError\n  process exited with status unknown";
        assert_eq!(
            localize(unknown_status, Some(JudgeFailureKind::Runtime)),
            "케이스 1: 실패\n\n오류\n  프로세스 종료 상태 알 수 없음"
        );
    }

    #[test]
    fn judge_localization_handles_no_cases_and_missing_runtime_app_prose() {
        assert_eq!(
            localize("problem has no judge cases", None),
            "채점 케이스가 없습니다."
        );
        assert_eq!(
            localize("Missing runtime for python", None),
            "python 런타임이 없습니다."
        );
        for malformed in [
            "Missing runtime for ruby",
            "Missing runtime for python\nraw detail",
        ] {
            assert_eq!(localize(malformed, None), malformed);
        }
    }

    #[test]
    fn assisted_new_capstone_result_has_retry_instead_of_fake_review() {
        let mastery = crate::core::LessonMastery {
            stage: crate::core::MasteryStage::New,
            review_due_at: 0,
            attempts: 1,
        };

        let lesson = synthetic_lesson(crate::core::SyntaxTrack::Core);
        let text = learning_state_text(&lesson, &mastery, 1_000, "en");

        assert!(text.contains("Mastery: New"), "{text}");
        assert!(
            text.contains("Retry this exercise; no review is scheduled."),
            "{text}"
        );
        assert!(!text.contains("Next review (days)"), "{text}");
    }

    #[test]
    fn learning_state_schedules_core_pass_but_not_lab_pass() {
        let now = 1_000;
        let passed = crate::core::LessonMastery {
            stage: crate::core::MasteryStage::Practiced,
            review_due_at: now + 86_400,
            attempts: 1,
        };
        let lab = synthetic_lesson(crate::core::SyntaxTrack::Lab);
        let core = synthetic_lesson(crate::core::SyntaxTrack::Core);

        let lab_text = learning_state_text(&lab, &passed, now, "en");
        let core_text = learning_state_text(&core, &passed, now, "en");

        assert!(
            lab_text.contains("Retry this exercise; no review is scheduled."),
            "{lab_text}"
        );
        assert!(!lab_text.contains("Next review (days)"), "{lab_text}");
        assert!(core_text.contains("Next review (days): 1"), "{core_text}");
    }

    fn localized_app(name: &str, language: &str) -> PracticodeApp {
        let root = crate::process::unique_temp_path(name, "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.state.settings.ui_language = language.to_string();
        app
    }

    #[test]
    fn first_problem_feedback_uses_the_selected_locale() {
        let mut app = localized_app("practicode-first-problem-locale", "ko");

        app.action_previous().unwrap();

        assert_eq!(app.output, ui_text("ko", "first_problem"));
    }

    #[test]
    fn answer_heading_uses_the_selected_locale() {
        let mut app = localized_app("practicode-answer-locale", "ja");

        app.action_give_up().unwrap();

        let heading = ui_text("ja", "answer_for_language").replace("{language}", "python");
        assert!(app.output.starts_with(&heading), "{}", app.output);
        assert!(!app.output.starts_with("Answer for"), "{}", app.output);
    }

    #[test]
    fn command_feedback_uses_the_selected_locale() {
        let mut app = localized_app("practicode-command-feedback-locale", "es");

        app.action_learn("ruby").unwrap();
        assert_eq!(app.output, ui_text("es", "learn_usage"));

        app.set_theme("light").unwrap();
        assert_eq!(
            app.output,
            ui_text("es", "theme_set").replace("{theme}", "light")
        );

        app.set_difficulty("impossible").unwrap();
        assert_eq!(app.output, ui_text("es", "difficulty_options"));

        app.set_ui_language("zh").unwrap();
        assert_eq!(
            app.output,
            ui_text("zh", "ui_language_set").replace("{language}", "zh")
        );
    }

    #[test]
    fn unavailable_next_problem_uses_the_selected_locale() {
        let mut app = localized_app("practicode-next-unavailable-locale", "zh");
        save_state(&app.root, &app.state).unwrap();
        let current = app.state.current_problem.clone();

        app.finish_next_problem(String::new(), current, false)
            .unwrap();

        assert_eq!(app.output, ui_text("zh", "next_unavailable"));
    }

    #[test]
    fn unreadable_notes_do_not_clear_the_note_editor() {
        let mut app = localized_app("practicode-unreadable-notes", "en");
        app.note_editor.set_text("keep this note");
        let path = app.root.join(PROBLEM_NOTES_PATH);
        std::fs::write(&path, [0xff]).unwrap();

        assert!(app.start_note_editor().is_err());
        assert_eq!(app.note_editor.text(), "keep this note");
        assert_eq!(std::fs::read(path).unwrap(), [0xff]);
    }
}
