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
        let (command, arg) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
        let arg = arg.trim();
        if !matches!(command, "list" | "problems") {
            self.list_cursor = None;
        }
        match command {
            "run" | "r" => self.action_run()?,
            "code" | "edit" | "e" | "vim" => self.action_edit()?,
            "home" => self.action_home()?,
            "doctor" => self.action_doctor(),
            "learn" => self.action_learn(arg)?,
            "lesson" => self.action_lesson(),
            "progress" => self.action_progress(),
            "next" | "n" => self.action_next(arg)?,
            "generate" | "gen" | "new" => self.action_generate(arg),
            "back" | "prev" | "previous" | "p" => self.action_previous()?,
            "answer" | "giveup" | "give" | "g" => self.action_give_up()?,
            "problems" | "list" => self.start_problem_list()?,
            "open" | "o" if !arg.is_empty() => self.open_problem(arg)?,
            "language" | "lang" if arg.is_empty() => self.action_cycle_language()?,
            "language" | "lang" if LANGUAGES.contains(&arg) => self.set_language(arg)?,
            "ui" if arg.is_empty() => self.action_toggle_ui_language()?,
            "ui" if arg
                .to_lowercase()
                .split(['-', '_'])
                .next()
                .is_some_and(|language| UI_LANGUAGES.contains(&language)) =>
            {
                self.set_ui_language(&normalize_ui_language(arg))?
            }
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
                self.write_text_output(ui_text(
                    &self.state.settings.ui_language,
                    "ai_next_command_saved",
                ));
            }
            "provider" | "ai-provider" if arg.is_empty() => {
                self.write_text_output(&format!(
                    "{}: {}\n{}",
                    ui_text(&self.state.settings.ui_language, "settings_ai_provider"),
                    self.state.settings.ai_provider,
                    provider_status(
                        &self.state.settings.ai_provider,
                        &self.state.settings.ui_language
                    )
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
                    "{}: {}\n{}",
                    ui_text(&self.state.settings.ui_language, "settings_ai_provider"),
                    self.state.settings.ai_provider,
                    provider_status(
                        &self.state.settings.ai_provider,
                        &self.state.settings.ui_language
                    )
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
            "hint" | "ask" if arg.is_empty() => {
                let prompt = match (command, self.mode) {
                    ("hint", AppMode::Learn) => {
                        "Give one concise hint about the current lesson exercise without giving the full solution."
                    }
                    ("ask", AppMode::Learn) => {
                        "Explain the current learning step with one guiding question and no full solution."
                    }
                    ("hint", _) => "Give one concise hint for the current problem.",
                    _ => {
                        "Explain the current problem with one guiding question and no full solution."
                    }
                };
                self.start_ai_prompt(prompt)?
            }
            "hint" | "ask" | "ai" if !arg.is_empty() => self.start_ai_prompt(arg)?,
            "note" if !arg.is_empty() => self.append_note(arg)?,
            "note" => self.start_note_editor()?,
            "notes" => self.show_notes()?,
            "update" => self.refresh_update_notice(),
            "exit" | "quit" | "q" => self.should_quit = true,
            _ => self.write_text_output(
                &ui_text(&self.state.settings.ui_language, "unknown_command")
                    .replace("{command}", value),
            ),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(language: &str) -> PracticodeApp {
        let root = crate::process::unique_temp_path("practicode-command-handler-locale", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.state.settings.ui_language = language.to_string();
        app
    }

    #[test]
    fn command_handler_feedback_uses_the_selected_locale() {
        let mut app = app("ja");

        app.handle_command("vim").unwrap();
        assert_eq!(app.mode, AppMode::Problems);
        assert_eq!(app.practice_view, PracticeView::Code);
        assert_eq!(app.focus, Focus::Code);
        assert!(!app.show_output);

        app.handle_command("ai-next-command true").unwrap();
        assert_eq!(app.output, ui_text("ja", "ai_next_command_saved"));

        app.handle_command("unknown-example").unwrap();
        assert_eq!(
            app.output,
            ui_text("ja", "unknown_command").replace("{command}", "unknown-example")
        );

        app.handle_command("provider").unwrap();
        assert!(
            app.output
                .starts_with(&format!("{}: ", ui_text("ja", "settings_ai_provider"))),
            "{}",
            app.output
        );
    }

    #[test]
    fn vim_alias_matches_only_the_exact_command_from_every_view() {
        for setup in [None, Some("learn"), Some("help")] {
            let mut app = app("en");
            if let Some(command) = setup {
                app.handle_command(command).unwrap();
            }

            app.handle_command("vim").unwrap();

            if setup == Some("learn") {
                assert_eq!(app.mode, AppMode::Learn);
                assert_eq!(app.learning_session.view(), LearningView::Code);
            } else {
                assert_eq!(app.mode, AppMode::Problems, "setup: {setup:?}");
                assert_eq!(app.practice_view, PracticeView::Code, "setup: {setup:?}");
            }
            assert_eq!(app.focus, Focus::Code, "setup: {setup:?}");
            assert!(!app.show_output, "setup: {setup:?}");
        }

        let mut app = app("en");
        app.handle_command("vimbad").unwrap();
        assert_eq!(
            app.output,
            ui_text("en", "unknown_command").replace("{command}", "vimbad")
        );
    }

    #[test]
    fn invalid_setting_commands_leave_settings_unchanged() {
        let mut app = app("ko");
        app.state.settings.ai_effort = "high".to_string();
        app.state.settings.generate_languages = vec!["rust".to_string()];
        app.state.settings.generate_ui_languages = vec!["ko".to_string()];

        app.handle_command("ui klingon").unwrap();
        app.handle_command("effort impossible").unwrap();
        app.handle_command("generate-languages ruby").unwrap();
        app.handle_command("generate-ui klingon").unwrap();

        assert_eq!(app.state.settings.ui_language, "ko");
        assert_eq!(app.state.settings.ai_effort, "high");
        assert_eq!(app.state.settings.generate_languages, ["rust"]);
        assert_eq!(app.state.settings.generate_ui_languages, ["ko"]);
    }
}
