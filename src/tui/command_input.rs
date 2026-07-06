use super::*;

impl PracticodeApp {
    pub(super) fn insert_command_char(&mut self, char: char) {
        let byte = byte_index(&self.command, self.command_cursor);
        self.command.insert(byte, char);
        self.command_cursor += 1;
        self.command_palette_cursor = 0;
        self.normalize_command_input();
    }

    pub(super) fn delete_command_before_cursor(&mut self) {
        if self.command_cursor == 0 {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor - 1);
        let end = byte_index(&self.command, self.command_cursor);
        self.command.replace_range(start..end, "");
        self.command_cursor -= 1;
        self.command_palette_cursor = 0;
        self.normalize_command_input();
    }

    pub(super) fn delete_command_at_cursor(&mut self) {
        if self.command_cursor >= char_len(&self.command) {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor);
        let end = byte_index(&self.command, self.command_cursor + 1);
        self.command.replace_range(start..end, "");
        self.command_palette_cursor = 0;
        self.normalize_command_input();
    }

    pub(super) fn command_suggestions(&self) -> Vec<CommandChoice> {
        if self.focus != Focus::Command {
            return Vec::new();
        }
        let Some(query) = self.command.trim_start().strip_prefix('/') else {
            return Vec::new();
        };
        let query = query.to_lowercase();
        let trimmed = query.trim_start();
        let choices = self.command_choices();
        if trimmed.is_empty() {
            return self
                .default_command_inserts()
                .iter()
                .filter_map(|insert| choices.iter().find(|hint| hint.insert == *insert).cloned())
                .collect();
        }
        choices
            .into_iter()
            .filter(|hint| hint.insert.starts_with(trimmed))
            .collect()
    }

    pub(super) fn default_command_inserts(&self) -> &'static [&'static str] {
        match self.mode {
            AppMode::Home => &["learn", "problems", "profile", "help", "quit"],
            AppMode::Problems => &[
                "run",
                "next",
                "back",
                "problems",
                "answer",
                "hint ",
                "generate ",
                "profile",
                "home",
            ],
            AppMode::Learn => &[
                "run", "next", "back", "learn", "problems", "profile", "home",
            ],
        }
    }

    pub(super) fn command_choices(&self) -> Vec<CommandChoice> {
        let mut choices = Vec::new();
        for hint in COMMAND_HINTS {
            if !hint.help && matches!(hint.insert, "drill" | "next-lesson" | "prev-lesson") {
                continue;
            }
            if hint.insert == "effort max" && self.state.settings.ai_provider != "claude" {
                continue;
            }
            if hint.insert == "model " {
                for model in self
                    .available_models
                    .iter()
                    .filter(|model| *model != "auto")
                {
                    choices.push(CommandChoice {
                        insert: format!("model {model}"),
                        display: format!("/model {model}"),
                        desc_key: "cmd_model_available",
                        keep_open: false,
                    });
                }
            }
            choices.push(CommandChoice {
                insert: hint.insert.to_string(),
                display: hint.display.to_string(),
                desc_key: hint.desc_key,
                keep_open: hint.keep_open,
            });
        }
        choices
    }

    pub(super) fn move_command_palette(&mut self, delta: isize) {
        let len = self.command_suggestions().len();
        if len == 0 {
            return;
        }
        let cursor = self.command_palette_cursor as isize;
        self.command_palette_cursor = ((cursor + delta).rem_euclid(len as isize)) as usize;
    }

    pub(super) fn accept_command_palette(&mut self) -> Result<bool> {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return Ok(false);
        }
        let hint = &suggestions[self.command_palette_cursor.min(suggestions.len() - 1)];
        if hint.keep_open {
            self.command = format!("/{}", hint.insert);
            self.command_cursor = char_len(&self.command);
            self.command_palette_cursor = 0;
            return Ok(true);
        }
        let value = hint.insert.clone();
        self.command.clear();
        self.command_cursor = 0;
        self.command_palette_cursor = 0;
        self.focus = Focus::None;
        self.submit_command(&value)?;
        Ok(true)
    }

    pub(super) fn normalize_command_input(&mut self) {
        let normalized = compose_hangul_jamo(&self.command);
        if normalized == self.command {
            self.command_cursor = self.command_cursor.min(char_len(&self.command));
            return;
        }
        let old_prefix = prefix(&self.command, self.command_cursor);
        self.command = normalized;
        self.command_cursor =
            char_len(&compose_hangul_jamo(&old_prefix)).min(char_len(&self.command));
    }
}
