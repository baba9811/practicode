use super::*;

impl PracticodeApp {
    pub(super) fn status_text(&self) -> String {
        if self.mode == AppMode::Home && !self.show_output {
            return format!(" PRACTICODE | home | {} ", self.mode_hint());
        }
        if self.mode == AppMode::Learn {
            let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
            let (done, total) = syntax_progress_count(&self.state, &self.state.settings.language);
            return format!(
                " PRACTICODE | learn | {} | {} | {done}/{total} | code:{} | {} ",
                syntax_language_name(&self.state.settings.language),
                lesson.id,
                self.state.settings.language,
                self.mode_hint(),
            );
        }
        let code_status = self.submission_status(&self.problem).0;
        let activity = if self.busy_label.is_empty() {
            "idle".to_string()
        } else {
            format!("{}{}", self.busy_body, self.busy_dots())
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
            " PRACTICODE | {} | {} | {} | {} | code:{} | {} | {} ",
            self.problem.id,
            self.problem.difficulty,
            self.problem_status(&self.problem),
            activity,
            code_status,
            self.state.settings.language,
            tail,
        )
    }

    pub(super) fn next_source_help(&self) -> String {
        "Next behavior: /next opens unsolved local problems first and asks AI only when none remain. Use /generate <request> to create a problem in the background.".to_string()
    }

    pub(super) fn background_generation_status(&self) -> Option<String> {
        if self.generate_rx.is_some() {
            let elapsed = self
                .generate_started
                .map(|started| started.elapsed().as_secs())
                .unwrap_or_default();
            Some(format!("bg generate {elapsed}s"))
        } else {
            self.generate_notice.clone()
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
            return "notes: type to edit, Esc profile";
        }
        if self.mode == AppMode::Home && !self.show_output {
            return ui_text(lang, "home_help");
        }
        if self.mode == AppMode::Learn && self.focus == Focus::Code {
            return ui_text(lang, "hint_learn");
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
        let daily_loop = match self.mode {
            AppMode::Home => {
                "1. Choose Learn syntax or Practice coding tests.\n2. Use Left/Right to move and Enter/Space to open.\n3. Press `/` for commands."
            }
            AppMode::Learn => {
                "1. Read the lesson on the left.\n2. Edit the drill on the right.\n3. Use `/run`, then `/next` or `/back`."
            }
            AppMode::Problems => {
                "1. Type code in the right pane.\n2. Press `Esc`, then choose `/run` from the command palette.\n3. Use `/next` when it passes."
            }
        };
        format!(
            "# {}\n\n## {}\n\n{}\n\n## {}\n\n{}\n\n## {}\n\n- `/` opens the command palette outside the editor.\n- `↑/↓` selects a command and `Enter` accepts it.\n- `Esc` cancels the command palette or leaves output.\n\n## {}\n\n- stdout is shown when a case fails.\n- stderr is shown without affecting the expected stdout.",
            ui_text(lang, "help_title"),
            ui_text(lang, "daily_loop"),
            daily_loop,
            ui_text(lang, "commands"),
            commands,
            ui_text(lang, "keys"),
            ui_text(lang, "debug_prints"),
        )
    }
}
