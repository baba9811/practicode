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

fn localized_judge_output(output: &str, language: &str) -> String {
    let structured = output.lines().any(|line| {
        line.strip_prefix("Case ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, outcome)| matches!(outcome, "PASS" | "FAIL"))
    });
    if !structured {
        return output.to_string();
    }
    output
        .lines()
        .map(|line| {
            if let Some((case, outcome)) = line
                .strip_prefix("Case ")
                .and_then(|rest| rest.split_once(": "))
            {
                let outcome = match outcome {
                    "PASS" => ui_text(language, "result_pass"),
                    "FAIL" => ui_text(language, "result_fail"),
                    _ => outcome,
                };
                return format!("{} {case}: {outcome}", ui_text(language, "judge_case"));
            }
            match line {
                "Input" => ui_text(language, "judge_input").to_string(),
                "Expected" => ui_text(language, "judge_expected").to_string(),
                "Got" => ui_text(language, "judge_got").to_string(),
                "Stdout" => ui_text(language, "judge_stdout").to_string(),
                "Stderr" => ui_text(language, "judge_stderr").to_string(),
                "Error" => ui_text(language, "judge_error").to_string(),
                "Compile" => ui_text(language, "judge_compile").to_string(),
                "  <hidden>" => format!("  <{}>", ui_text(language, "judge_hidden")),
                "  <empty>" => format!("  {}", ui_text(language, "empty_value")),
                "  timeout: 5s" => format!("  {}", ui_text(language, "judge_timeout_detail")),
                _ => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
        self.state.settings.start_mode = "problems".to_string();
        save_state(&self.root, &self.state)?;
        self.load_code_editor()?;
        self.settings_cursor = None;
        self.list_cursor = None;
        self.show_output = false;
        self.focus = Focus::Code;
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
            return self.action_practice();
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
        let detail = localized_judge_output(&result.output, &self.state.settings.ui_language);
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
            self.focus = Focus::Code;
            return Ok(());
        }
        if self.generate_rx.is_some() {
            self.write_text_output(
                "A background generation is already running. Keep solving; /next will pick up the new problem when it finishes.",
            );
            return Ok(());
        }
        self.start_next_problem(old_problem, true, request.to_string());
        Ok(())
    }

    pub(super) fn action_generate(&mut self, request: &str) {
        self.check_background_generation();
        if self.task_rx.is_some() || self.generate_rx.is_some() {
            let message = "Generation is already running; skipped duplicate /generate.";
            self.generate_notice = Some(message.to_string());
            self.write_text_output(message);
            return;
        }
        self.start_background_generation(request.trim().to_string());
    }

    pub(super) fn start_background_generation(&mut self, request: String) {
        self.transition_mode(AppMode::Problems);
        self.state.settings.start_mode = "problems".to_string();
        if save_state(&self.root, &self.state).is_err() {
            self.write_text_output("Could not save practice mode before generation.");
            return;
        }
        let root = self.root.clone();
        let state = self.state.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(run_ai_generate(&root, &state, &request));
        });
        self.generate_bank_len = self.bank.len();
        self.generate_started = Some(Instant::now());
        self.generate_notice = Some("Generating in background.".to_string());
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
        self.start_busy(
            "next",
            ui_text(&self.state.settings.ui_language, "generating_next"),
        );
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
                self.write_text_output(&format!(
                    "{}{}No next problem is available yet.",
                    if output.is_empty() { "" } else { &output },
                    if output.is_empty() { "" } else { "\n\n" }
                ));
                return Ok(());
            }
        }
        self.load_code_editor()?;
        self.settings_cursor = None;
        self.show_output = false;
        self.focus = Focus::Code;
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
            self.write_text_output("Already at the first known problem.");
        } else {
            self.load_code_editor()?;
            self.settings_cursor = None;
            self.show_output = false;
            self.focus = Focus::Code;
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
        self.write_output(&format!(
            "Answer for {language}:\n\n```{language}\n{}\n```",
            answer.trim_end()
        ));
        Ok(())
    }

    pub(super) fn action_learn(&mut self, language: &str) -> Result<()> {
        let language = language.trim();
        if !language.is_empty() && !LANGUAGES.contains(&language) {
            self.write_text_output("Usage: /learn or /learn python|ts|java|rust");
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
        save_state(&self.root, &self.state)?;
        self.load_syntax_editor()?;
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
        let mastery = &self.state.syntax_mastery[lesson.language][lesson.id];
        let stage = ui_text(
            &self.state.settings.ui_language,
            match mastery.stage {
                crate::core::MasteryStage::New => "mastery_new",
                crate::core::MasteryStage::Practiced => "mastery_practiced",
                crate::core::MasteryStage::Retained => "mastery_retained",
                crate::core::MasteryStage::Mastered => "mastery_mastered",
            },
        );
        let review_days = mastery
            .review_due_at
            .saturating_sub(now)
            .saturating_add(86_399)
            / 86_400;
        let learning_state = format!(
            "{}: {stage}\n{}: {review_days}",
            ui_text(&self.state.settings.ui_language, "result_mastery"),
            ui_text(&self.state.settings.ui_language, "result_review_days"),
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
            localized_judge_output(&result.output, &self.state.settings.ui_language).trim_end()
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
                save_state(&self.root, &self.state)?;
                self.load_syntax_editor()?;
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
        save_state(&self.root, &self.state)?;
        self.load_syntax_editor()?;
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
        save_state(&self.root, &self.state)?;
        self.load_syntax_editor()?;
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
        self.focus = if self.learning_session.is_guided() {
            match self.learning_session.view() {
                LearningView::Lesson => Focus::Left,
                LearningView::Code => Focus::Code,
                LearningView::Result => Focus::Output,
            }
        } else {
            Focus::Code
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
        if self.mode != AppMode::Learn {
            return;
        }
        self.learning_session.cycle_view();
        self.show_current_syntax_lesson();
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
        save_state(&self.root, &self.state)?;
        self.load_code_editor()?;
        self.settings_cursor = None;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    pub(super) fn set_ui_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.ui_language = normalize_ui_language(language);
        save_state(&self.root, &self.state)?;
        if self.mode == AppMode::Learn {
            self.left_scroll = 0;
            self.show_current_syntax_lesson();
        } else {
            self.write_text_output(&format!("UI language: {}", self.state.settings.ui_language));
        }
        Ok(())
    }

    pub(super) fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.state.settings.theme = theme.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("Theme: {theme}"));
        Ok(())
    }

    pub(super) fn set_difficulty(&mut self, difficulty: &str) -> Result<()> {
        let difficulty = difficulty.trim().to_lowercase();
        if !DIFFICULTIES.contains(&difficulty.as_str()) {
            self.write_text_output("Difficulty: auto, easy, medium, or hard.");
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
        self.state.settings.ai_effort =
            normalize_ai_effort(&self.state.settings.ai_provider, effort);
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
        self.note_editor
            .set_text(&read_problem_notes(&self.root).unwrap_or_default());
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
        fs::write(path, if text.is_empty() { "" } else { text })?;
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

    #[test]
    fn judge_localization_preserves_tool_owned_compiler_diagnostics() {
        let raw = "Case 1: compiler-owned diagnostic\nTS2322: raw detail";
        assert_eq!(localized_judge_output(raw, "ko"), raw);
        assert_eq!(
            localized_judge_output("Case 1: FAIL\n\nGot\n  value", "ko"),
            "케이스 1: 실패\n\n실제 출력\n  value"
        );
    }
}
