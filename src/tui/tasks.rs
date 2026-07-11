use super::*;

impl PracticodeApp {
    pub(super) fn start_ai_prompt(&mut self, prompt: &str) -> Result<()> {
        if self.task_rx.is_some() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "already_busy"));
            return Ok(());
        }
        self.save_code()?;
        if self.mode == AppMode::Learn {
            self.learning_session.mark_assisted();
        }
        #[cfg(test)]
        if self.ai_spawn_disabled {
            return Ok(());
        }
        let label = normalize_ai_provider(&self.state.settings.ai_provider);
        self.start_busy("ai", &label);
        let root = self.root.clone();
        let problem = self.problem.clone();
        let settings = self.state.settings.clone();
        let mode = self.mode;
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let latest_result = self.learn_result.clone();
        let prompt = prompt.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = if mode == AppMode::Learn {
                run_ai_lesson_prompt(&root, lesson, &settings, &prompt, &latest_result)
            } else {
                run_ai_prompt(&root, &problem, &settings, &prompt)
            };
            let _ = tx.send(TaskResult::AiPrompt(output));
        });
        self.task_rx = Some(rx);
        Ok(())
    }

    pub(super) fn check_task(&mut self) {
        let task = self.task_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(task) = task {
            self.task_rx = None;
            self.stop_busy();
            match task {
                TaskResult::AiPrompt(output) => self.write_output(&output),
                TaskResult::Next {
                    output,
                    old_problem,
                    fallback_to_local,
                } => {
                    if let Err(error) =
                        self.finish_next_problem(output, old_problem, fallback_to_local)
                    {
                        self.write_text_output(&format!(
                            "{}\n{error}",
                            ui_text(&self.state.settings.ui_language, "next_failed")
                        ));
                    }
                }
            }
        }
    }

    pub(super) fn check_background_generation(&mut self) {
        let output = self.generate_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        let Some(result) = output else {
            return;
        };
        self.generate_rx = None;
        self.generate_started = None;
        let old_len = self.generate_bank_len;
        let (added, reload_error) = match load_bank(&self.root) {
            Ok(bank) => {
                let added = bank.len().saturating_sub(old_len);
                self.bank = bank;
                let _ = save_state(&self.root, &self.state);
                (added, None)
            }
            Err(error) => (0, Some(error.to_string())),
        };
        self.generate_notice = Some(match result {
            AiGenerationResult::Failed { status, detail } => GenerationNotice::Failed {
                status,
                detail,
                added,
                reload_error,
            },
            AiGenerationResult::FailedToRun(detail) => GenerationNotice::Failed {
                status: None,
                detail,
                added,
                reload_error,
            },
            AiGenerationResult::Succeeded(_) => {
                if let Some(error) = reload_error {
                    GenerationNotice::ReloadFailed(error)
                } else if added > 0 {
                    GenerationNotice::Generated(added)
                } else {
                    GenerationNotice::Finished
                }
            }
        });
    }

    pub(super) fn check_update(&mut self) {
        let result = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        let showing_update_check =
            self.output == ui_text(&self.state.settings.ui_language, "update_checking");
        if let Some(result) = result {
            self.update_rx = None;
            self.update_check = Some(result.clone());
            match &result {
                UpdateCheck::Available(version) => self.update_notice = Some(version.clone()),
                UpdateCheck::Current | UpdateCheck::Disabled => self.update_notice = None,
                UpdateCheck::Failed => {}
            }
            if showing_update_check {
                self.show_update_notice();
            }
        }
    }

    pub(super) fn start_update_check(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        self.last_update_check = Some(Instant::now());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(check_latest_version());
        });
        self.update_rx = Some(rx);
    }

    pub(super) fn maybe_start_periodic_update_check(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        if self
            .last_update_check
            .is_none_or(|last| last.elapsed() >= UPDATE_CHECK_INTERVAL)
        {
            self.start_update_check();
        }
    }

    pub(super) fn start_model_check(&mut self) {
        let provider = self.state.settings.ai_provider.clone();
        if self.model_rx.is_some() || self.available_models_provider == provider {
            return;
        }
        let query_provider = provider.clone();
        let ui_language = self.state.settings.ui_language.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(available_models(&query_provider, &ui_language));
        });
        self.available_models_provider = provider;
        self.model_rx = Some(rx);
    }

    pub(super) fn check_models(&mut self) {
        let models = self.model_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(catalog) = models {
            self.model_rx = None;
            self.available_models = catalog.models;
            self.model_message = catalog.message;
            if self.showing_model_status {
                self.output = self.model_status_text();
                self.output_scroll = 0;
                self.output_is_markdown = false;
                self.show_output = true;
            } else if self.settings_cursor.is_some() {
                self.output = self.profile_text();
                self.output_scroll = 0;
                self.output_is_markdown = false;
                self.show_output = true;
            }
        }
    }

    pub(super) fn model_status_text(&self) -> String {
        let lang = &self.state.settings.ui_language;
        let mut lines = vec![
            format!(
                "{}: {}",
                ui_text(lang, "settings_ai_provider"),
                self.state.settings.ai_provider
            ),
            format!(
                "{}: {}",
                ui_text(lang, "settings_ai_model"),
                if self.state.settings.ai_model == "auto" {
                    ui_text(lang, "settings_provider_default")
                } else {
                    self.state.settings.ai_model.as_str()
                }
            ),
            format!(
                "{}: {}",
                ui_text(lang, "settings_ai_effort"),
                if self.state.settings.ai_effort == "auto" {
                    ui_text(lang, "settings_provider_default")
                } else {
                    self.state.settings.ai_effort.as_str()
                }
            ),
            ui_text(lang, "model_use_default_model").to_string(),
            ui_text(lang, "model_use_default_effort").to_string(),
        ];
        if self.model_rx.is_some() {
            lines.push(ui_text(lang, "model_loading").to_string());
        } else if self.available_models.is_empty() {
            lines.push(
                self.model_message
                    .clone()
                    .unwrap_or_else(|| ui_text(lang, "model_unavailable").to_string()),
            );
            lines.push(ui_text(lang, "model_custom_hint").to_string());
        } else {
            if let Some(message) = &self.model_message {
                lines.push(message.clone());
            }
            let efforts = if self.state.settings.ai_provider == "claude" {
                CLAUDE_AI_EFFORTS
            } else {
                CODEX_AI_EFFORTS
            };
            lines.push(
                ui_text(lang, "model_available_efforts").replace("{efforts}", &efforts.join(", ")),
            );
            lines.push(ui_text(lang, "model_available_models").to_string());
            lines.extend(
                self.available_models
                    .iter()
                    .map(|model| format!("- /model {model}")),
            );
        }
        lines.join("\n")
    }

    pub(super) fn start_busy(&mut self, label: &str, arg: &str) {
        self.settings_cursor = None;
        self.busy_label = label.to_string();
        self.busy_arg = arg.to_string();
        self.busy_started = Some(Instant::now());
        self.busy_frame = 0;
        self.busy_hits = 0;
        self.busy_misses = 0;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn stop_busy(&mut self) {
        self.busy_label.clear();
        self.busy_arg.clear();
        self.busy_started = None;
        self.busy_frame = 0;
    }

    pub(super) fn handle_busy_key(&mut self, key: KeyEvent) -> bool {
        if self.task_rx.is_none() {
            return false;
        }
        if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
            self.should_quit = true;
        } else if self.busy_label == "next"
            && key.code == KeyCode::Char(' ')
            && key.modifiers.is_empty()
        {
            if self.busy_game_on_target() {
                self.busy_hits += 1;
            } else {
                self.busy_misses += 1;
            }
        }
        self.focus = Focus::Output;
        true
    }

    pub(super) fn write_output(&mut self, output: &str) {
        self.settings_cursor = None;
        self.editing_notes = false;
        self.showing_model_status = false;
        self.output = output.to_string();
        self.output_scroll = 0;
        self.output_is_markdown = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn write_text_output(&mut self, output: &str) {
        self.settings_cursor = None;
        self.editing_notes = false;
        self.showing_model_status = false;
        self.output = output.trim_end().to_string();
        self.output_scroll = 0;
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn write_model_status(&mut self) {
        self.output = self.model_status_text();
        self.output_scroll = 0;
        self.output_is_markdown = false;
        self.showing_model_status = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn refresh_update_notice(&mut self) {
        self.update_check = None;
        self.update_notice = None;
        self.start_update_check();
        self.show_update_notice();
    }

    pub(super) fn show_update_notice(&mut self) {
        let lang = self.state.settings.ui_language.clone();
        if let Some(version) = &self.update_notice {
            self.write_text_output(&format!(
                "{}: practicode {version} (current {CURRENT_VERSION})\n\nnpm update -g practicode\ncargo install --force practicode",
                ui_text(&lang, "update_available")
            ));
        } else if self.update_rx.is_some() {
            self.write_text_output(ui_text(&lang, "update_checking"));
        } else if matches!(self.update_check, Some(UpdateCheck::Disabled)) {
            self.write_text_output(ui_text(&lang, "update_check_disabled"));
        } else if matches!(self.update_check, Some(UpdateCheck::Failed)) {
            self.write_text_output(ui_text(&lang, "update_check_failed"));
        } else {
            self.write_text_output(ui_text(&lang, "update_none"));
        }
    }

    pub(super) fn append_note(&mut self, note: &str) -> Result<()> {
        append_problem_note(&self.root, note)?;
        self.write_text_output(
            &ui_text(&self.state.settings.ui_language, "note_saved")
                .replace("{path}", PROBLEM_NOTES_PATH),
        );
        Ok(())
    }

    pub(super) fn show_notes(&mut self) -> Result<()> {
        let notes = read_problem_notes(&self.root)?;
        if notes.is_empty() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "notes_empty"));
        } else {
            self.write_text_output(&format!(
                "{}\n\n{notes}",
                ui_text(&self.state.settings.ui_language, "notes_title")
                    .replace("{path}", PROBLEM_NOTES_PATH)
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_text_content(app: &PracticodeApp) -> String {
        app.output_text()
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn finish_generation(app: &mut PracticodeApp, result: AiGenerationResult, old_len: usize) {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(result).unwrap();
        app.generate_bank_len = old_len;
        app.generate_rx = Some(rx);
        app.check_background_generation();
    }

    fn learning_app(name: &str) -> PracticodeApp {
        let root = crate::process::unique_temp_path(name, "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.ai_spawn_disabled = true;
        app.handle_command("learn python").unwrap();
        app
    }

    fn localized_app(name: &str, language: &str) -> PracticodeApp {
        let root = crate::process::unique_temp_path(name, "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.state.settings.ui_language = language.to_string();
        app
    }

    fn assert_learning_exit_clears_session(
        name: &str,
        leave: impl FnOnce(&mut PracticodeApp) -> Result<()>,
    ) {
        let mut app = learning_app(name);
        app.handle_command("hint").unwrap();
        assert!(app.learning_session.assisted());

        leave(&mut app).unwrap();

        assert!(!app.learning_session.is_guided());
        assert!(!app.learning_session.assisted());
        app.handle_command("learn python").unwrap();
        assert!(!app.learning_session.assisted());
    }

    #[test]
    fn update_check_refreshes_visible_checking_notice() {
        let root = crate::process::unique_temp_path("practicode-update-test", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(UpdateCheck::Disabled).unwrap();
        app.update_rx = Some(rx);
        app.write_text_output(ui_text("en", "update_checking"));

        app.check_update();

        assert_eq!(app.output, ui_text("en", "update_check_disabled"));
    }

    #[test]
    fn visible_update_check_uses_the_selected_locale() {
        let mut app = localized_app("practicode-update-checking-locale", "ko");
        let (_tx, rx) = std::sync::mpsc::channel();
        app.update_rx = Some(rx);

        app.show_update_notice();

        assert_eq!(app.output, ui_text("ko", "update_checking"));
    }

    #[test]
    fn next_failure_heading_uses_the_selected_locale() {
        let mut app = localized_app("practicode-next-failed-locale", "ja");
        std::fs::write(app.root.join("problem_bank.json"), "not json").unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(TaskResult::Next {
            output: String::new(),
            old_problem: app.state.current_problem.clone(),
            fallback_to_local: false,
        })
        .unwrap();
        app.task_rx = Some(rx);

        app.check_task();

        assert!(
            app.output.starts_with(ui_text("ja", "next_failed")),
            "{}",
            app.output
        );
        assert!(app.output.contains("parse"), "{}", app.output);
    }

    #[test]
    fn profile_copy_renders_in_the_selected_locale() {
        let app = localized_app("practicode-profile-locale", "ja");

        let profile = app.profile_text();

        for key in [
            "settings_title",
            "settings_instructions",
            "settings_code_language",
            "settings_ai_provider",
            "settings_ai_model",
            "settings_model_load_hint",
            "settings_problem_notes",
        ] {
            assert!(profile.contains(ui_text("ja", key)), "{key}: {profile}");
        }
        assert!(!profile.contains("User profile"), "{profile}");
        assert!(!profile.contains("AI provider"), "{profile}");
    }

    #[test]
    fn model_status_renders_app_owned_copy_in_the_selected_locale() {
        let app = localized_app("practicode-model-status-locale", "zh");

        let status = app.model_status_text();

        for key in [
            "settings_ai_provider",
            "settings_ai_model",
            "settings_ai_effort",
            "model_use_default_model",
            "model_use_default_effort",
            "model_unavailable",
            "model_custom_hint",
        ] {
            assert!(status.contains(ui_text("zh", key)), "{key}: {status}");
        }
        assert!(!status.contains("AI provider"), "{status}");
        assert!(!status.contains("Provider model list"), "{status}");
    }

    #[test]
    fn notes_feedback_renders_in_the_selected_locale() {
        let mut app = localized_app("practicode-notes-locale", "es");

        app.show_notes().unwrap();
        assert_eq!(app.output, ui_text("es", "notes_empty"));

        app.append_note("prioriza límites").unwrap();
        assert_eq!(
            app.output,
            ui_text("es", "note_saved").replace("{path}", PROBLEM_NOTES_PATH)
        );

        app.show_notes().unwrap();
        assert!(
            app.output
                .starts_with(&ui_text("es", "notes_title").replace("{path}", PROBLEM_NOTES_PATH)),
            "{}",
            app.output
        );
        assert!(app.output.contains("prioriza límites"), "{}", app.output);
    }

    #[test]
    fn busy_copy_and_elapsed_time_render_in_the_selected_locale() {
        let root = crate::process::unique_temp_path("practicode-busy-locale", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.state.settings.ui_language = "ko".to_string();

        app.start_busy("ai", "codex");

        let output = output_text_content(&app);
        let status = app.status_text();
        assert!(output.contains("codex가 생각 중"), "{output}");
        assert!(output.contains("0초"), "{output}");
        assert!(status.contains("codex가 생각 중"), "{status}");
        assert!(status.contains("0초"), "{status}");
        assert!(!output.contains("is thinking"), "{output}");
        assert!(!output.contains("0s"), "{output}");

        app.state.settings.ui_language = "ja".to_string();
        app.start_busy("next", "");

        let output = output_text_content(&app);
        assert!(output.contains("次の問題を生成中"), "{output}");
        assert!(output.contains("0秒"), "{output}");
        assert!(!output.contains("Generating next problem"), "{output}");
    }

    #[test]
    fn background_generation_notices_render_in_the_selected_locale() {
        let root = crate::process::unique_temp_path("practicode-generation-locale", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root.clone()).unwrap();
        app.state.settings.ui_language = "ko".to_string();

        finish_generation(&mut app, AiGenerationResult::Succeeded(String::new()), 0);
        let generated = app.background_generation_status().unwrap();
        assert!(generated.contains("문제 1개"), "{generated}");
        assert!(!generated.contains("Generated"), "{generated}");

        finish_generation(
            &mut app,
            AiGenerationResult::Failed {
                status: Some(7),
                detail: "raw provider detail".to_string(),
            },
            0,
        );
        let failed = app.background_generation_status().unwrap();
        assert!(failed.contains("백그라운드 생성에 실패"), "{failed}");
        assert!(failed.contains("raw provider detail"), "{failed}");
        assert!(failed.contains("문제 1개"), "{failed}");
        assert!(!failed.contains("Background generation failed"), "{failed}");

        app.state.settings.ui_language = "ja".to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        app.generate_rx = Some(rx);
        app.generate_started = Some(Instant::now());
        app.generate_notice = Some(GenerationNotice::Started);
        let running = app.background_generation_status().unwrap();
        assert!(running.contains("バックグラウンド生成"), "{running}");
        assert!(running.contains("0秒"), "{running}");
        assert!(!running.contains("background generation"), "{running}");
        assert_eq!(
            app.generation_notice_text(&GenerationNotice::Started),
            "バックグラウンドで生成中です。"
        );

        app.action_generate("");
        assert!(app.output.contains("重複した /generate"), "{}", app.output);
        assert!(!app.output.contains("duplicate"), "{}", app.output);
        drop(tx);
        app.generate_rx = None;

        let bank_len = app.bank.len();
        finish_generation(
            &mut app,
            AiGenerationResult::Succeeded(String::new()),
            bank_len,
        );
        let finished = app.background_generation_status().unwrap();
        assert!(
            finished.contains("バックグラウンド生成が完了"),
            "{finished}"
        );
        assert!(
            !finished.contains("Background generation finished"),
            "{finished}"
        );

        std::fs::write(root.join("problem_bank.json"), "not json").unwrap();
        let bank_len = app.bank.len();
        finish_generation(
            &mut app,
            AiGenerationResult::Succeeded(String::new()),
            bank_len,
        );
        let reload_failed = app.background_generation_status().unwrap();
        assert!(
            reload_failed.contains("問題バンクを再読み込みできませんでした"),
            "{reload_failed}"
        );
        assert!(reload_failed.contains("parse"), "{reload_failed}");
        assert!(
            !reload_failed.contains("bank reload failed"),
            "{reload_failed}"
        );

        finish_generation(
            &mut app,
            AiGenerationResult::Failed {
                status: Some(9),
                detail: "raw failed-generation detail".to_string(),
            },
            bank_len,
        );
        let failed_reload = app.background_generation_status().unwrap();
        assert!(
            failed_reload.contains("バックグラウンド生成に失敗"),
            "{failed_reload}"
        );
        assert!(
            failed_reload.contains("raw failed-generation detail"),
            "{failed_reload}"
        );
        assert!(
            failed_reload.contains("問題バンクを再読み込みできませんでした"),
            "{failed_reload}"
        );
        assert!(failed_reload.contains("parse"), "{failed_reload}");
    }

    #[test]
    fn every_lesson_ai_command_marks_the_live_attempt_at_start() {
        for (index, command) in ["hint", "hint one clue", "ask", "ask why", "ai explain this"]
            .into_iter()
            .enumerate()
        {
            let mut app = learning_app(&format!("practicode-ai-command-{index}"));

            app.handle_command(command).unwrap();

            assert!(app.learning_session.assisted(), "{command}");
            assert!(app.task_rx.is_none(), "{command}");
        }
    }

    #[test]
    fn every_manual_lesson_ai_command_marks_the_attempt_at_start() {
        for (index, command) in ["hint", "hint one clue", "ask", "ask why", "ai explain this"]
            .into_iter()
            .enumerate()
        {
            let mut app = learning_app(&format!("practicode-manual-ai-command-{index}"));
            app.handle_command("back").unwrap();
            assert!(!app.learning_session.is_guided());

            app.handle_command(command).unwrap();

            assert!(app.learning_session.assisted(), "{command}");
            assert!(app.task_rx.is_none(), "{command}");
        }
    }

    #[test]
    fn lesson_ai_at_reflect_cannot_leak_into_the_next_item() {
        let root = crate::process::unique_temp_path("practicode-ai-boundary", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let bank = load_bank(&root).unwrap();
        let mut state = load_state(&root, &bank).unwrap();
        state.syntax_mastery.insert(
            "python".to_string(),
            HashMap::from([(
                "py-output".to_string(),
                crate::core::LessonMastery {
                    stage: crate::core::MasteryStage::Practiced,
                    review_due_at: 1,
                    attempts: 1,
                },
            )]),
        );
        save_state(&root, &state).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.ai_spawn_disabled = true;
        app.handle_command("learn python").unwrap();
        app.handle_command("next").unwrap();
        app.handle_command("next").unwrap();
        app.handle_command("ai explain this").unwrap();
        assert!(app.learning_session.assisted());
        app.learning_session.finish_judge(true);
        assert!(!app.learning_session.assisted());

        app.handle_command("hint reflect").unwrap();
        assert!(!app.learning_session.assisted());
        app.handle_command("next").unwrap();

        assert_eq!(app.learning_session.current_lesson_id(), Some("py-input"));
        assert!(!app.learning_session.assisted());
    }

    #[test]
    fn problem_ai_does_not_mark_a_suspended_lesson_attempt() {
        let mut app = learning_app("practicode-ai-suspended-lesson");
        app.handle_command("home").unwrap();

        app.handle_command("ai explain this problem").unwrap();

        assert!(!app.learning_session.assisted());
        assert!(app.task_rx.is_none());
    }

    #[test]
    fn home_clears_the_assisted_learning_session() {
        assert_learning_exit_clears_session("practicode-ai-home-boundary", |app| {
            app.handle_command("home")
        });
    }

    #[test]
    fn practice_clears_the_assisted_learning_session() {
        assert_learning_exit_clears_session("practicode-ai-practice-boundary", |app| {
            app.action_practice()
        });
    }

    #[test]
    fn problem_list_clears_the_assisted_learning_session() {
        assert_learning_exit_clears_session("practicode-ai-list-boundary", |app| {
            app.handle_command("problems")
        });
    }

    #[test]
    fn direct_problem_open_clears_the_assisted_learning_session() {
        assert_learning_exit_clears_session("practicode-ai-open-boundary", |app| {
            app.open_problem("1")
        });
    }

    #[test]
    fn generation_clears_the_assisted_learning_session() {
        assert_learning_exit_clears_session("practicode-ai-generate-boundary", |app| {
            app.state.settings.ai_next_command = "true".to_string();
            app.start_background_generation(String::new());
            Ok(())
        });
    }
}
