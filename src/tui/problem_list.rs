use super::*;

impl PracticodeApp {
    pub(super) fn load_code_editor(&mut self) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.load_syntax_editor();
        }
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        self.editor.set_text(&text);
        Ok(())
    }

    pub(super) fn save_code(&self) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.save_syntax_code();
        }
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        save_user_text(&path, &self.editor.text())?;
        Ok(())
    }

    pub(super) fn load_syntax_editor(&mut self) -> Result<()> {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let path = ensure_syntax_submission(&self.root, lesson)?;
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        self.editor.set_text(&text);
        Ok(())
    }

    pub(super) fn save_syntax_code(&self) -> Result<()> {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let path = ensure_syntax_submission(&self.root, lesson)?;
        save_user_text(&path, &self.editor.text())?;
        Ok(())
    }

    pub(super) fn start_problem_list(&mut self) -> Result<()> {
        self.transition_mode(AppMode::Problems);
        self.state.settings.start_mode = "problems".to_string();
        save_state(&self.root, &self.state)?;
        self.list_cursor = Some(self.current_problem_index());
        self.write_text_output(&self.render_problem_list());
        Ok(())
    }

    pub(super) fn render_problem_list(&self) -> String {
        let lang = &self.state.settings.ui_language;
        let status_by_id = self
            .state
            .history
            .iter()
            .map(|item| (item.id.as_str(), item.status.as_str()))
            .collect::<HashMap<_, _>>();
        let cursor = self
            .list_cursor
            .unwrap_or_else(|| self.current_problem_index())
            .min(self.bank.len().saturating_sub(1));
        let mut lines = vec![
            ui_text(lang, "problem_list_title").to_string(),
            String::new(),
            format!(
                "    # {} {} {} {} {}",
                cell(ui_text(lang, "problem_list_id"), 18),
                cell(ui_text(lang, "problem_list_difficulty"), 10),
                cell(ui_text(lang, "problem_list_status"), 10),
                cell(ui_text(lang, "problem_list_code"), 9),
                ui_text(lang, "problem_list_name")
            ),
        ];
        for (index, problem) in self.bank.iter().enumerate() {
            let marker = if index == cursor { ">" } else { " " };
            let current = if problem.id == self.problem.id {
                "*"
            } else {
                " "
            };
            let title = localized(&problem.title, &self.state.settings.ui_language);
            let difficulty = localized_status(lang, &problem.difficulty);
            let status = localized_status(
                lang,
                status_by_id
                    .get(problem.id.as_str())
                    .copied()
                    .unwrap_or("-"),
            );
            let code_status = localized_status(lang, &self.submission_status(problem).0);
            lines.push(format!(
                "{marker} {current} {:>2} {} {} {} {} {title}",
                index + 1,
                cell(&problem.id, 18),
                cell(&difficulty, 10),
                cell(&status, 10),
                cell(&code_status, 9),
            ));
        }
        lines.push(format!("\n{}", ui_text(lang, "problem_list_hint")));
        lines.join("\n")
    }

    pub(super) fn current_problem_index(&self) -> usize {
        self.bank
            .iter()
            .position(|problem| problem.id == self.problem.id)
            .unwrap_or(0)
    }

    pub(super) fn move_list_cursor(&mut self, delta: isize) {
        if self.bank.is_empty() {
            return;
        }
        let cursor = self
            .list_cursor
            .unwrap_or_else(|| self.current_problem_index())
            .min(self.bank.len() - 1) as isize;
        let len = self.bank.len() as isize;
        self.list_cursor = Some(((cursor + delta).rem_euclid(len)) as usize);
        self.write_text_output(&self.render_problem_list());
    }

    pub(super) fn open_selected_problem(&mut self) -> Result<()> {
        if let Some(cursor) = self.list_cursor.take()
            && let Some(problem) = self.bank.get(cursor).or_else(|| self.bank.last())
        {
            let problem_id = problem.id.clone();
            self.open_problem(&problem_id)?;
        }
        Ok(())
    }

    pub(super) fn open_problem(&mut self, query: &str) -> Result<()> {
        self.list_cursor = None;
        let Some(problem) = self.find_problem(query).cloned() else {
            self.write_text_output(
                &ui_text(&self.state.settings.ui_language, "problem_not_found")
                    .replace("{query}", query),
            );
            return Ok(());
        };
        self.problem = problem;
        self.state.current_problem = self.problem.id.clone();
        self.transition_mode(AppMode::Problems);
        self.state.settings.start_mode = "problems".to_string();
        if !self
            .state
            .history
            .iter()
            .any(|item| item.id == self.problem.id)
        {
            self.state.history.push(HistoryItem {
                id: self.problem.id.clone(),
                status: "assigned".to_string(),
            });
        }
        ensure_problem_files(&self.root, &self.problem)?;
        self.load_code_editor()?;
        save_state(&self.root, &self.state)?;
        self.show_output = false;
        self.practice_view = PracticeView::Problem;
        self.focus = Focus::Left;
        Ok(())
    }

    pub(super) fn find_problem(&self, query: &str) -> Option<&Problem> {
        let needle = if query.trim().chars().all(|c| c.is_ascii_digit()) {
            format!("{:03}", query.trim().parse::<usize>().ok()?)
        } else {
            query.trim().to_lowercase()
        };
        self.bank.iter().find(|problem| {
            needle == problem.id.to_lowercase()
                || needle == problem.slug.to_lowercase()
                || problem.id.starts_with(&needle)
        })
    }

    pub(super) fn problem_status(&self, problem: &Problem) -> String {
        if self.state.solved.contains(&problem.id) {
            return "solved".to_string();
        }
        self.state
            .history
            .iter()
            .rev()
            .find(|item| item.id == problem.id)
            .map(|item| item.status.clone())
            .unwrap_or_else(|| "not_started".to_string())
    }

    pub(super) fn submission_status(&self, problem: &Problem) -> (String, String) {
        let language = normalize_language(&self.state.settings.language);
        let path = self
            .root
            .join("submissions")
            .join(&problem.id)
            .join(format!("solution.{}", ext_for(&language)));
        let relative = path.strip_prefix(&self.root).unwrap_or(&path).display();
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return ("missing".to_string(), format!("({language})"));
            }
            Ok(metadata) if metadata.file_type().is_file() => {}
            Ok(_) | Err(_) => {
                return ("unreadable".to_string(), format!("({relative})"));
            }
        }
        let Ok(content) = fs::read_to_string(&path) else {
            return ("unreadable".to_string(), format!("({relative})"));
        };
        if content == template_for(&language) {
            ("template".to_string(), format!("({relative})"))
        } else if content.trim().is_empty() {
            ("empty".to_string(), format!("({relative})"))
        } else {
            ("written".to_string(), format!("({relative})"))
        }
    }
}

fn cell(value: &str, width: usize) -> String {
    format!(
        "{value}{}",
        " ".repeat(width.saturating_sub(display_width(value)))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(language: &str) -> PracticodeApp {
        let root = crate::process::unique_temp_path("practicode-problem-list-locale", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = PracticodeApp::new(root).unwrap();
        app.state.settings.ui_language = language.to_string();
        app
    }

    #[test]
    fn problem_list_localizes_app_owned_columns_and_values() {
        let app = app("zh");

        let list = app.render_problem_list();

        for key in [
            "problem_list_title",
            "problem_list_id",
            "problem_list_difficulty",
            "problem_list_status",
            "problem_list_code",
            "problem_list_name",
            "problem_list_hint",
            "status_easy",
            "status_assigned",
            "status_template",
        ] {
            assert!(list.contains(ui_text("zh", key)), "{key}: {list}");
        }
        assert!(!list.contains("Problems"), "{list}");
        assert!(!list.contains("Difficulty"), "{list}");
    }

    #[test]
    fn missing_problem_feedback_uses_the_selected_locale() {
        let mut app = app("ja");

        app.open_problem("does-not-exist").unwrap();

        assert_eq!(
            app.output,
            ui_text("ja", "problem_not_found").replace("{query}", "does-not-exist")
        );
    }

    #[test]
    fn unreadable_submission_does_not_clear_the_editor() {
        let mut app = app("en");
        app.editor.set_text("keep this text");
        let path = ensure_submission(&app.root, &app.problem, &app.state.settings).unwrap();
        std::fs::write(&path, [0xff]).unwrap();

        assert!(app.load_code_editor().is_err());
        assert_eq!(app.editor.text(), "keep this text");
        assert_eq!(std::fs::read(path).unwrap(), [0xff]);
        assert_eq!(app.submission_status(&app.problem).0, "unreadable");
    }

    #[test]
    fn opening_a_stale_problem_cursor_clamps_to_the_bank() {
        let mut app = app("en");
        let expected = app.bank.last().unwrap().id.clone();
        app.list_cursor = Some(app.bank.len() + 10);

        app.open_selected_problem().unwrap();

        assert_eq!(app.problem.id, expected);
        assert!(app.list_cursor.is_none());
    }
}
