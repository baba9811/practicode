use super::*;

impl PracticodeApp {
    pub(super) fn start_ai_prompt(&mut self, prompt: &str) -> Result<()> {
        if self.task_rx.is_some() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "already_busy"));
            return Ok(());
        }
        self.save_code()?;
        let label = normalize_ai_provider(&self.state.settings.ai_provider);
        self.start_busy("ai", &format!("{label} is thinking"));
        let root = self.root.clone();
        let problem = self.problem.clone();
        let settings = self.state.settings.clone();
        let prompt = prompt.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_ai_prompt(&root, &problem, &settings, &prompt);
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
                        self.write_text_output(&format!("Next failed\n{error}"));
                    }
                }
            }
        }
    }

    pub(super) fn check_background_generation(&mut self) {
        let output = self.generate_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        let Some(output) = output else {
            return;
        };
        self.generate_rx = None;
        self.generate_started = None;
        let old_len = self.generate_bank_len;
        match load_bank(&self.root) {
            Ok(bank) => {
                let added = bank.len().saturating_sub(old_len);
                self.bank = bank;
                let _ = save_state(&self.root, &self.state);
                self.generate_notice = Some(if added > 0 {
                    format!("Generated {added} problem in background. Use /next.")
                } else if output.contains("failed") {
                    "Background generation failed. Use /generate to retry.".to_string()
                } else {
                    "Background generation finished. Use /problems to review.".to_string()
                });
            }
            Err(error) => {
                self.generate_notice = Some(format!(
                    "Background generation finished, but bank reload failed: {error}"
                ));
            }
        }
    }

    pub(super) fn check_update(&mut self) {
        let result = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.update_rx = None;
            self.update_check = Some(result.clone());
            match &result {
                UpdateCheck::Available(version) => self.update_notice = Some(version.clone()),
                UpdateCheck::Current | UpdateCheck::Disabled => self.update_notice = None,
                UpdateCheck::Failed => {}
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
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(available_models(&query_provider));
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
                self.output_is_markdown = false;
                self.show_output = true;
            } else if self.settings_cursor.is_some() {
                self.output = self.profile_text();
                self.output_is_markdown = false;
                self.show_output = true;
            }
        }
    }

    pub(super) fn model_status_text(&self) -> String {
        let mut lines = vec![
            format!("AI provider: {}", self.state.settings.ai_provider),
            format!(
                "AI model: {}",
                if self.state.settings.ai_model == "auto" {
                    "auto (provider default)"
                } else {
                    self.state.settings.ai_model.as_str()
                }
            ),
            format!(
                "AI effort: {}",
                if self.state.settings.ai_effort == "auto" {
                    "auto (provider default)"
                } else {
                    self.state.settings.ai_effort.as_str()
                }
            ),
            "Use /model auto to let the provider choose its default.".to_string(),
            "Use /effort auto to let the provider choose its default.".to_string(),
        ];
        if self.model_rx.is_some() {
            lines.push("Loading provider model list...".to_string());
        } else if self.available_models.is_empty() {
            lines.push(
                self.model_message
                    .clone()
                    .unwrap_or_else(|| "Provider model list is unavailable.".to_string()),
            );
            lines.push("Use /model <name> for a known model.".to_string());
        } else {
            if let Some(message) = &self.model_message {
                lines.push(message.clone());
            }
            let efforts = if self.state.settings.ai_provider == "claude" {
                CLAUDE_AI_EFFORTS
            } else {
                CODEX_AI_EFFORTS
            };
            lines.push(format!("Available efforts: {}", efforts.join(", ")));
            lines.push("Available models:".to_string());
            lines.extend(
                self.available_models
                    .iter()
                    .map(|model| format!("- /model {model}")),
            );
        }
        lines.join("\n")
    }

    pub(super) fn start_busy(&mut self, label: &str, body: &str) {
        self.settings_cursor = None;
        self.busy_label = label.to_string();
        self.busy_body = body.to_string();
        self.busy_started = Some(Instant::now());
        self.busy_frame = 0;
        self.busy_hits = 0;
        self.busy_misses = 0;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn stop_busy(&mut self) {
        self.busy_label.clear();
        self.busy_body.clear();
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
        self.output_is_markdown = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn write_text_output(&mut self, output: &str) {
        self.settings_cursor = None;
        self.editing_notes = false;
        self.showing_model_status = false;
        self.output = output.trim_end().to_string();
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    pub(super) fn write_model_status(&mut self) {
        self.output = self.model_status_text();
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
            self.write_text_output("Checking for updates...");
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
        self.write_text_output(&format!("Problem note saved to {PROBLEM_NOTES_PATH}."));
        Ok(())
    }

    pub(super) fn show_notes(&mut self) -> Result<()> {
        let notes = read_problem_notes(&self.root)?;
        if notes.is_empty() {
            self.write_text_output("No notes yet. Use /note to edit problem-generation notes.");
        } else {
            self.write_text_output(&format!("Problem notes ({PROBLEM_NOTES_PATH})\n\n{notes}"));
        }
        Ok(())
    }
}
