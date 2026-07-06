use super::*;

impl PracticodeApp {
    pub(super) fn handle_command(&mut self, value: &str) -> Result<()> {
        if self.task_rx.is_some() {
            let command = value
                .trim()
                .strip_prefix('/')
                .unwrap_or(value.trim())
                .split_whitespace()
                .next()
                .unwrap_or("");
            if matches!(command, "exit" | "quit" | "q") {
                self.should_quit = true;
            } else {
                self.focus = Focus::Output;
            }
            return Ok(());
        }
        if value.is_empty() || matches!(value, "help" | "h" | "?") {
            self.list_cursor = None;
            self.write_output(&self.help_text());
            return Ok(());
        }
        if value.starts_with("vim") {
            self.list_cursor = None;
            self.write_text_output("The code editor is already open on the right.");
            return Ok(());
        }
        let (command, arg) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
        let arg = arg.trim();
        if !matches!(command, "list" | "problems") {
            self.list_cursor = None;
        }
        match command {
            "run" | "r" => self.action_run()?,
            "code" | "edit" | "e" => self.action_edit()?,
            "learn" => self.action_learn(arg)?,
            "drill" => self.action_drill()?,
            "next-lesson" => self.action_next_lesson()?,
            "prev-lesson" => self.action_prev_lesson()?,
            "next" | "n" => self.action_next(arg)?,
            "generate" | "gen" | "new" => self.action_generate(arg),
            "back" | "prev" | "previous" | "p" => self.action_previous()?,
            "answer" | "giveup" | "give" | "g" => self.action_give_up()?,
            "problems" | "list" => self.start_problem_list(),
            "open" | "o" if !arg.is_empty() => self.open_problem(arg)?,
            "language" | "lang" if arg.is_empty() => self.action_cycle_language()?,
            "language" | "lang" if LANGUAGES.contains(&arg) => self.set_language(arg)?,
            "ui" if arg.is_empty() => self.action_toggle_ui_language()?,
            "ui" => self.set_ui_language(&normalize_ui_language(arg))?,
            "theme" if arg.is_empty() => self.action_toggle_theme()?,
            "theme" if THEMES.contains(&arg) => self.set_theme(arg)?,
            "profile" | "settings" if arg.is_empty() => self.show_profile(),
            "profile" | "settings" if arg == "reset" => self.reset_profile()?,
            "difficulty" | "level" if arg.is_empty() => self.show_profile(),
            "difficulty" | "level" => self.set_difficulty(arg)?,
            "topics" | "topic" if arg.is_empty() => self.show_profile(),
            "topics" | "topic" => self.set_topics(arg, false)?,
            "avoid" | "skip" if arg.is_empty() => self.show_profile(),
            "avoid" | "skip" => self.set_topics(arg, true)?,
            "generate-languages" | "gen-languages" | "gen-lang" if arg.is_empty() => {
                self.show_profile()
            }
            "generate-languages" | "gen-languages" | "gen-lang" => {
                self.set_generate_languages(arg, false)?
            }
            "generate-ui" | "gen-ui" if arg.is_empty() => self.show_profile(),
            "generate-ui" | "gen-ui" => self.set_generate_languages(arg, true)?,
            "source" | "next-source" if arg.is_empty() => {
                self.write_text_output(&self.next_source_help());
            }
            "source" | "next-source" if matches!(arg, "bank" | "local" | "ai") => {
                self.state.settings.next_source = normalize_next_source(arg);
                save_state(&self.root, &self.state)?;
                self.write_text_output(&self.next_source_help());
            }
            "ai-next-command" if !arg.is_empty() => {
                self.state.settings.ai_next_command = arg.to_string();
                self.state.settings.next_source = "ai".to_string();
                save_state(&self.root, &self.state)?;
                self.write_text_output("AI next command saved.");
            }
            "provider" | "ai-provider" if arg.is_empty() => {
                self.write_text_output(&format!(
                    "AI provider: {}\n{}",
                    self.state.settings.ai_provider,
                    provider_status(&self.state.settings.ai_provider)
                ));
            }
            "provider" | "ai-provider" if AI_PROVIDERS.contains(&arg) => {
                self.state.settings.ai_provider = normalize_ai_provider(arg);
                self.state.settings.ai_model = "auto".to_string();
                self.state.settings.ai_effort = normalize_ai_effort(
                    &self.state.settings.ai_provider,
                    &self.state.settings.ai_effort,
                );
                self.model_rx = None;
                self.available_models.clear();
                self.available_models_provider.clear();
                self.model_message = None;
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!(
                    "AI provider: {}\n{}",
                    self.state.settings.ai_provider,
                    provider_status(&self.state.settings.ai_provider)
                ));
            }
            "model" if arg.is_empty() => {
                self.start_model_check();
                self.check_models();
                self.write_model_status();
            }
            "model" => {
                self.state.settings.ai_model = if arg == "auto" {
                    "auto".to_string()
                } else {
                    arg.to_string()
                };
                save_state(&self.root, &self.state)?;
                self.start_model_check();
                self.check_models();
                self.write_model_status();
            }
            "effort" | "reasoning" | "ai-effort" if arg.is_empty() => {
                self.write_model_status();
            }
            "effort" | "reasoning" | "ai-effort" => self.set_ai_effort(arg)?,
            "hint" if arg.is_empty() => {
                self.start_ai_prompt("Give one concise hint for the current problem.")?
            }
            "hint" | "ask" | "ai" if !arg.is_empty() => self.start_ai_prompt(arg)?,
            "note" if !arg.is_empty() => self.append_note(arg)?,
            "note" => self.start_note_editor()?,
            "notes" => self.show_notes()?,
            "update" => self.refresh_update_notice(),
            "exit" | "quit" | "q" => self.should_quit = true,
            _ => self.write_text_output(&format!("Unknown command: {value}\nTry /help.")),
        }
        Ok(())
    }
}
