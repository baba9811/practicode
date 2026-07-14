use super::*;

impl PracticodeApp {
    pub(super) fn status_text_for_width(&self, width: u16) -> String {
        if self.task_rx.is_some()
            || self.editing_notes
            || self.focus == Focus::Command
            || self.list_cursor.is_some()
            || self.show_output
        {
            return format!(" {} ", self.mode_hint());
        }
        if width > 120 {
            return self.status_text();
        }
        if self.mode == AppMode::Learn {
            return format!(" {} ", self.learning_primary_hint());
        }
        let lang = &self.state.settings.ui_language;
        let key = match self.mode {
            AppMode::Home => "hint_home_compact",
            AppMode::Learn => "hint_learn_compact",
            AppMode::Problems => "hint_problem_compact",
        };
        format!(" {} ", ui_text(lang, key))
    }

    pub(super) fn status_text(&self) -> String {
        if self.task_rx.is_some()
            || self.editing_notes
            || self.focus == Focus::Command
            || self.list_cursor.is_some()
            || (self.show_output && self.busy_label.is_empty())
        {
            return format!(" {} ", self.mode_hint());
        }
        let lang = &self.state.settings.ui_language;
        if self.mode == AppMode::Home && !self.show_output {
            return format!(
                " PRACTICODE | {} | {} ",
                ui_text(lang, "mode_home"),
                self.mode_hint()
            );
        }
        if self.mode == AppMode::Learn {
            let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
            let (done, total) =
                syntax_core_progress_count(&self.state, &self.state.settings.language);
            let selected_focus = match self.learning_session.view() {
                LearningView::Lesson => Focus::Left,
                LearningView::Code => Focus::Code,
                LearningView::Result => Focus::Output,
            };
            let focus = if !self.show_output && self.focus == selected_focus {
                format!(
                    " | {}: {}",
                    ui_text(lang, "focus_active"),
                    learning_view_label(lang, self.learning_session.view()),
                )
            } else {
                String::new()
            };
            return format!(
                " {} | PRACTICODE | {} | {} | {} | {done}/{total}{focus} | {}:{} ",
                self.learning_primary_hint(),
                ui_text(lang, "mode_learn"),
                syntax_language_name(&self.state.settings.language),
                lesson.id,
                ui_text(lang, "status_code"),
                self.state.settings.language,
            );
        }
        let code_status = localized_status(lang, &self.submission_status(&self.problem).0);
        let activity = if self.busy_label.is_empty() {
            ui_text(lang, "status_idle").to_string()
        } else {
            let elapsed = self
                .busy_started
                .map(|started| started.elapsed().as_secs())
                .unwrap_or_default();
            format!(
                "{}{} {}",
                self.busy_text(),
                self.busy_dots(),
                self.elapsed_text(elapsed)
            )
        };
        let tail = if let Some(version) = self.update_notice.as_ref() {
            format!(
                "{}:{version} /update",
                ui_text(&self.state.settings.ui_language, "update")
            )
        } else if self.task_rx.is_some() {
            self.mode_hint().to_string()
        } else if let Some(status) = self.background_generation_status() {
            status
        } else {
            self.mode_hint().to_string()
        };
        format!(
            " PRACTICODE | {} | {} | {} | {} | {}:{} | {} | {} ",
            self.problem.id,
            localized_status(lang, &self.problem.difficulty),
            localized_status(lang, &self.problem_status(&self.problem)),
            activity,
            ui_text(lang, "status_code"),
            code_status,
            self.state.settings.language,
            tail,
        )
    }

    pub(super) fn next_source_help(&self) -> String {
        ui_text(&self.state.settings.ui_language, "next_source_help").to_string()
    }

    pub(super) fn background_generation_status(&self) -> Option<String> {
        if self.generate_rx.is_some() {
            let elapsed = self
                .generate_started
                .map(|started| started.elapsed().as_secs())
                .unwrap_or_default();
            Some(format!(
                "{} {}",
                ui_text(
                    &self.state.settings.ui_language,
                    "status_background_generation"
                ),
                self.elapsed_text(elapsed)
            ))
        } else {
            self.generate_notice
                .as_ref()
                .map(|notice| self.generation_notice_text(notice))
        }
    }

    pub(super) fn busy_text(&self) -> String {
        let lang = &self.state.settings.ui_language;
        match self.busy_label.as_str() {
            "ai" => ui_text(lang, "busy_ai_thinking").replace("{provider}", &self.busy_arg),
            "next" => ui_text(lang, "generating_next").to_string(),
            _ => self.busy_arg.clone(),
        }
    }

    pub(super) fn elapsed_text(&self, seconds: u64) -> String {
        ui_text(&self.state.settings.ui_language, "elapsed_seconds")
            .replace("{seconds}", &seconds.to_string())
    }

    pub(super) fn generation_notice_text(&self, notice: &GenerationNotice) -> String {
        let lang = &self.state.settings.ui_language;
        match notice {
            GenerationNotice::Started => ui_text(lang, "generation_started").to_string(),
            GenerationNotice::Duplicate => ui_text(lang, "generation_duplicate").to_string(),
            GenerationNotice::Generated(count) => {
                ui_text(lang, "generation_generated").replace("{count}", &count.to_string())
            }
            GenerationNotice::Failed {
                status,
                detail,
                added,
                reload_error,
            } => {
                let mut lines = vec![ui_text(lang, "generation_failed").to_string()];
                if let Some(status) = status {
                    lines.push(
                        ui_text(lang, "generation_exit_status")
                            .replace("{status}", &status.to_string()),
                    );
                }
                if !detail.is_empty() {
                    lines.push(detail.clone());
                }
                if *added > 0 {
                    lines.push(
                        ui_text(lang, "generation_partial_count")
                            .replace("{count}", &added.to_string()),
                    );
                }
                if let Some(error) = reload_error {
                    lines.push(ui_text(lang, "generation_reload_failed").to_string());
                    if !error.is_empty() {
                        lines.push(error.clone());
                    }
                }
                lines.join("\n")
            }
            GenerationNotice::Finished => ui_text(lang, "generation_finished").to_string(),
            GenerationNotice::ReloadFailed(detail) => {
                let mut text = ui_text(lang, "generation_reload_failed").to_string();
                if !detail.is_empty() {
                    text.push('\n');
                    text.push_str(detail);
                }
                text
            }
        }
    }

    pub(super) fn busy_dots(&self) -> String {
        ".".repeat((self.busy_frame / 8) % 4)
    }

    pub(super) fn busy_game_track(&self) -> String {
        let width = 9;
        let target = width / 2;
        let position = (self.busy_frame / 2) % width;
        let mut cells = vec!['-'; width];
        cells[target] = '|';
        cells[position] = if position == target { 'X' } else { '*' };
        format!("[{}]", cells.into_iter().collect::<String>())
    }

    pub(super) fn busy_game_on_target(&self) -> bool {
        (self.busy_frame / 2) % 9 == 4
    }

    pub(super) fn mode_hint(&self) -> &'static str {
        let lang = &self.state.settings.ui_language;
        if self.task_rx.is_some() {
            return if self.busy_label == "next" {
                ui_text(lang, "hint_busy_next")
            } else {
                ui_text(lang, "hint_busy")
            };
        }
        if self.editing_notes {
            return ui_text(lang, "hint_notes");
        }
        if self.mode == AppMode::Home && !self.show_output {
            return ui_text(lang, "home_help");
        }
        if self.mode == AppMode::Learn && !self.show_output && self.focus != Focus::Command {
            return self.learning_primary_hint();
        }
        if self.mode == AppMode::Problems && !self.show_output && self.focus != Focus::Command {
            return ui_text(lang, "hint_problem");
        }
        match (self.focus, self.list_cursor.is_some(), self.show_output) {
            (Focus::Command, _, _) => ui_text(lang, "hint_command"),
            (_, true, _) => ui_text(lang, "hint_list"),
            (_, _, true) if self.settings_cursor.is_some() => ui_text(lang, "hint_settings"),
            (_, _, true) => ui_text(lang, "hint_output"),
            (Focus::Code, _, _) => ui_text(lang, "hint_problem"),
            _ => ui_text(lang, "hint_idle"),
        }
    }

    fn learning_primary_hint(&self) -> &'static str {
        let key = if self.focus == Focus::Code
            && self.learning_session.step() != LearningStep::Exercise
        {
            "learning_primary_leave_code"
        } else {
            match self.learning_session.step() {
                LearningStep::Exercise if self.learning_session.view() == LearningView::Result => {
                    "learning_primary_edit"
                }
                LearningStep::Exercise => "learning_primary_run",
                LearningStep::Complete => "learning_primary_home",
                _ => "learning_primary_next",
            }
        };
        ui_text(&self.state.settings.ui_language, key)
    }

    pub(super) fn help_text(&self) -> String {
        let lang = &self.state.settings.ui_language;
        let choices = self.command_choices();
        let commands = self
            .default_command_inserts()
            .iter()
            .filter_map(|insert| choices.iter().find(|hint| hint.insert == *insert))
            .map(|hint| format!("- `{}` {}", hint.display, ui_text(lang, hint.desc_key)))
            .collect::<Vec<_>>()
            .join("\n");
        let (daily_loop, shortcuts) = match self.mode {
            AppMode::Home => (ui_text(lang, "help_home_loop"), ui_text(lang, "home_help")),
            AppMode::Learn => (
                ui_text(lang, "help_learn_loop"),
                ui_text(lang, "learning_shortcuts"),
            ),
            AppMode::Problems => (
                ui_text(lang, "help_problem_loop"),
                ui_text(lang, "practice_shortcuts"),
            ),
        };
        format!(
            "# {}\n\n## {}\n\n{}\n\n## {}\n\n{}\n\n## {}\n\n- {}\n- {}\n- {}\n- {}\n\n## {}\n\n- {}\n- {}",
            ui_text(lang, "help_title"),
            ui_text(lang, "daily_loop"),
            daily_loop,
            ui_text(lang, "commands"),
            commands,
            ui_text(lang, "keys"),
            shortcuts,
            ui_text(lang, "help_palette_open"),
            ui_text(lang, "help_palette_move"),
            ui_text(lang, "help_palette_close"),
            ui_text(lang, "debug_prints"),
            ui_text(lang, "help_stdout"),
            ui_text(lang, "help_stderr"),
        )
    }
}
