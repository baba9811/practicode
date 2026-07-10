use super::*;

impl PracticodeApp {
    pub(super) fn load_code_editor(&mut self) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.load_syntax_editor();
        }
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        let text = fs::read_to_string(path).unwrap_or_default();
        self.editor.set_text(&text);
        Ok(())
    }

    pub(super) fn save_code(&self) -> Result<()> {
        if self.mode == AppMode::Learn {
            return self.save_syntax_code();
        }
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        fs::write(path, self.editor.text())?;
        Ok(())
    }

    pub(super) fn load_syntax_editor(&mut self) -> Result<()> {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let path = ensure_syntax_submission(&self.root, lesson)?;
        let text = fs::read_to_string(path).unwrap_or_default();
        self.editor.set_text(&text);
        Ok(())
    }

    pub(super) fn save_syntax_code(&self) -> Result<()> {
        let lesson = current_syntax_lesson(&self.state, &self.state.settings.language);
        let path = ensure_syntax_submission(&self.root, lesson)?;
        fs::write(path, self.editor.text())?;
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
        let status_by_id = self
            .state
            .history
            .iter()
            .map(|item| (item.id.as_str(), item.status.as_str()))
            .collect::<HashMap<_, _>>();
        let cursor = self
            .list_cursor
            .unwrap_or_else(|| self.current_problem_index());
        let mut lines = vec![
            "Problems".to_string(),
            String::new(),
            "    # ID                 Difficulty  Status      Code      Title".to_string(),
        ];
        for (index, problem) in self.bank.iter().enumerate() {
            let marker = if index == cursor { ">" } else { " " };
            let current = if problem.id == self.problem.id {
                "*"
            } else {
                " "
            };
            let title = localized(&problem.title, &self.state.settings.ui_language);
            let code_status = self.submission_status(problem).0;
            lines.push(format!(
                "{marker} {current} {:>2} {:<18} {:<10} {:<10} {:<9} {title}",
                index + 1,
                problem.id,
                problem.difficulty,
                status_by_id
                    .get(problem.id.as_str())
                    .copied()
                    .unwrap_or("-"),
                code_status,
            ));
        }
        lines.push("\nup/down or j/k select | enter open | esc close".to_string());
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
            .unwrap_or_else(|| self.current_problem_index()) as isize;
        let len = self.bank.len() as isize;
        self.list_cursor = Some(((cursor + delta).rem_euclid(len)) as usize);
        self.write_text_output(&self.render_problem_list());
    }

    pub(super) fn open_selected_problem(&mut self) -> Result<()> {
        if let Some(cursor) = self.list_cursor {
            let problem_id = self.bank[cursor].id.clone();
            self.list_cursor = None;
            self.open_problem(&problem_id)?;
        }
        Ok(())
    }

    pub(super) fn open_problem(&mut self, query: &str) -> Result<()> {
        self.list_cursor = None;
        let Some(problem) = self.find_problem(query).cloned() else {
            self.write_text_output(&format!("Problem not found: {query}\nTry /problems."));
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
        save_state(&self.root, &self.state)?;
        ensure_problem_files(&self.root, &self.problem)?;
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
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
        if !path.exists() {
            return ("missing".to_string(), format!("({language})"));
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        let relative = path.strip_prefix(&self.root).unwrap_or(&path).display();
        if content == template_for(&language) {
            ("template".to_string(), format!("({relative})"))
        } else if content.trim().is_empty() {
            ("empty".to_string(), format!("({relative})"))
        } else {
            ("written".to_string(), format!("({relative})"))
        }
    }
}
