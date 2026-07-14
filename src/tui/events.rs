use super::*;

impl PracticodeApp {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }
        if self.handle_busy_key(key) {
            return Ok(());
        }
        match key.code {
            KeyCode::F(1) => {
                self.handle_command("help")?;
                return Ok(());
            }
            KeyCode::F(5) => {
                self.action_run()?;
                return Ok(());
            }
            KeyCode::F(6) => {
                self.cycle_learning_view();
                return Ok(());
            }
            _ => {}
        }
        if self.editing_notes {
            return self.handle_note_key(key);
        }
        match self.focus {
            Focus::Command => self.handle_command_key(key),
            Focus::Code => self.handle_code_key(key),
            _ => self.handle_global_key(key),
        }
    }

    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        if self.task_rx.is_some() {
            self.focus = Focus::Output;
            return Ok(());
        }
        let position = Position::new(mouse.column, mouse.row);
        if self.command_palette_area.contains(position) {
            return Ok(());
        }
        if matches!(
            mouse.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        ) {
            let delta = if matches!(mouse.kind, MouseEventKind::ScrollUp) {
                -3
            } else {
                3
            };
            if self.show_output && self.output_area.contains(position) {
                self.focus = Focus::Output;
                self.scroll_output(delta);
            } else if !self.show_output && self.left_area.contains(position) {
                self.focus = Focus::Left;
                self.scroll_left(delta);
            } else if !self.show_output
                && self.mode == AppMode::Learn
                && !self.learn_result.is_empty()
                && self.output_area.contains(position)
            {
                self.focus = Focus::Output;
                self.scroll_output(delta);
            } else if !self.show_output && self.code_area.contains(position) {
                self.focus = Focus::Code;
                if delta < 0 {
                    self.editor.move_page_up(delta.unsigned_abs());
                } else {
                    self.editor.move_page_down(delta as usize);
                }
            }
            return Ok(());
        }
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return Ok(());
        }
        if self.command_area.contains(position) {
            self.focus_command();
        } else if self.mode == AppMode::Home && self.home_learn_area.contains(position) {
            self.focus = Focus::Home;
            self.home_choice = HomeChoice::Learn;
            self.open_home_choice()?;
        } else if self.mode == AppMode::Home && self.home_problems_area.contains(position) {
            self.focus = Focus::Home;
            self.home_choice = HomeChoice::Problems;
            self.open_home_choice()?;
        } else if self.mode == AppMode::Home
            && !self.show_output
            && self.home_area.contains(position)
        {
            self.focus = Focus::Home;
        } else if self.show_output && self.output_area.contains(position) {
            self.focus = Focus::Output;
        } else if !self.show_output
            && self.mode != AppMode::Home
            && self.left_area.contains(position)
        {
            if self.mode == AppMode::Problems {
                self.practice_view = PracticeView::Problem;
            }
            self.focus = Focus::Left;
        } else if !self.show_output
            && self.mode == AppMode::Learn
            && !self.learn_result.is_empty()
            && self.output_area.contains(position)
        {
            self.focus = Focus::Output;
        } else if self.code_area.contains(position) {
            self.action_edit()?;
        }
        Ok(())
    }

    pub(super) fn handle_command_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command.clear();
                self.command_cursor = 0;
                self.command_palette_cursor = 0;
                self.focus = self.resting_focus();
            }
            KeyCode::Enter => {
                if !self.accept_command_palette()? {
                    let value = self.command.trim().to_string();
                    self.command.clear();
                    self.command_cursor = 0;
                    self.command_palette_cursor = 0;
                    self.focus = Focus::None;
                    self.submit_command(&value)?;
                }
            }
            KeyCode::Backspace => self.delete_command_before_cursor(),
            KeyCode::Delete => self.delete_command_at_cursor(),
            KeyCode::Left => self.command_cursor = self.command_cursor.saturating_sub(1),
            KeyCode::Right => {
                self.command_cursor = (self.command_cursor + 1).min(char_len(&self.command));
            }
            KeyCode::Up => self.move_command_palette(-1),
            KeyCode::Down => self.move_command_palette(1),
            KeyCode::Home => self.command_cursor = 0,
            KeyCode::End => self.command_cursor = char_len(&self.command),
            KeyCode::Char('?') if self.command.trim().is_empty() || self.command.trim() == "/" => {
                self.command.clear();
                self.command_cursor = 0;
                self.command_palette_cursor = 0;
                self.focus = Focus::None;
                self.handle_command("help")?;
            }
            KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_command_char(char);
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_code_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.focus = Focus::None,
            KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.insert_char(char);
                self.save_code()?;
            }
            KeyCode::Enter => {
                self.editor.insert_newline();
                self.save_code()?;
            }
            KeyCode::Backspace => {
                self.editor.backspace();
                self.save_code()?;
            }
            KeyCode::Delete => {
                self.editor.delete();
                self.save_code()?;
            }
            KeyCode::Tab => {
                for _ in 0..4 {
                    self.editor.insert_char(' ');
                }
                self.save_code()?;
            }
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            KeyCode::PageUp => self
                .editor
                .move_page_up(self.code_area.height.saturating_sub(2) as usize),
            KeyCode::PageDown => self
                .editor
                .move_page_down(self.code_area.height.saturating_sub(2) as usize),
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_note_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.close_note_editor()?,
            KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.note_editor.insert_char(char);
                self.save_notes()?;
            }
            KeyCode::Enter => {
                self.note_editor.insert_newline();
                self.save_notes()?;
            }
            KeyCode::Backspace => {
                self.note_editor.backspace();
                self.save_notes()?;
            }
            KeyCode::Delete => {
                self.note_editor.delete();
                self.save_notes()?;
            }
            KeyCode::Tab => {
                for _ in 0..4 {
                    self.note_editor.insert_char(' ');
                }
                self.save_notes()?;
            }
            KeyCode::Left => self.note_editor.move_left(),
            KeyCode::Right => self.note_editor.move_right(),
            KeyCode::Up => self.note_editor.move_up(),
            KeyCode::Down => self.note_editor.move_down(),
            KeyCode::PageUp => self
                .note_editor
                .move_page_up(self.output_area.height.saturating_sub(2) as usize),
            KeyCode::PageDown => self
                .note_editor
                .move_page_down(self.output_area.height.saturating_sub(2) as usize),
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_global_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.settings_cursor.is_some() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.move_settings_cursor(-1),
                KeyCode::Down | KeyCode::Char('j') => self.move_settings_cursor(1),
                KeyCode::Char(' ') | KeyCode::Enter => self.change_selected_setting()?,
                KeyCode::Esc => {
                    self.settings_cursor = None;
                    self.show_output = false;
                    self.focus = if self.mode == AppMode::Problems
                        && self.practice_view == PracticeView::Problem
                    {
                        Focus::Left
                    } else {
                        Focus::Code
                    };
                }
                _ => self.handle_global_shortcut(key)?,
            }
            return Ok(());
        }
        if let Some(cursor) = self.list_cursor {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.move_list_cursor(-1),
                KeyCode::Down | KeyCode::Char('j') => self.move_list_cursor(1),
                KeyCode::Enter => self.open_selected_problem()?,
                KeyCode::Esc => {
                    self.list_cursor = None;
                    self.write_text_output(ui_text(
                        &self.state.settings.ui_language,
                        "list_closed",
                    ));
                }
                _ => {
                    self.list_cursor = Some(cursor);
                    self.handle_global_shortcut(key)?;
                }
            }
            return Ok(());
        }
        if self.handle_scroll_key(key) {
            return Ok(());
        }
        if key.code == KeyCode::Esc && self.show_output {
            if self.mode == AppMode::Home {
                self.action_home()?;
            } else if self.mode == AppMode::Learn {
                self.show_current_syntax_lesson();
            } else {
                self.show_output = false;
                self.practice_view = PracticeView::Code;
                self.focus = Focus::Code;
            }
            return Ok(());
        }
        if self.mode == AppMode::Home {
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    self.move_home_choice()
                }
                KeyCode::Enter | KeyCode::Char(' ') => self.open_home_choice()?,
                _ => self.handle_global_shortcut(key)?,
            }
            return Ok(());
        }
        if self.mode == AppMode::Learn && key.code == KeyCode::Enter {
            match self.learning_session.step() {
                LearningStep::Complete => self.action_home()?,
                LearningStep::Exercise if self.learning_session.view() == LearningView::Result => {
                    self.action_edit()?
                }
                LearningStep::Exercise => {}
                _ => self.action_next_learning()?,
            }
            return Ok(());
        }
        self.handle_global_shortcut(key)
    }

    pub(super) fn handle_global_shortcut(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('/') => self.focus_command(),
            KeyCode::Char('?') => self.handle_command("help")?,
            KeyCode::Char('r') => self.action_run()?,
            KeyCode::Char('n') => self.action_next("")?,
            KeyCode::Char('p') => self.action_previous()?,
            KeyCode::Char('g') => self.action_give_up()?,
            KeyCode::Char('e') => self.action_edit()?,
            KeyCode::Char('l') => self.action_cycle_language()?,
            KeyCode::Char('u') => self.action_toggle_ui_language()?,
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
        Ok(())
    }

    fn handle_scroll_key(&mut self, key: KeyEvent) -> bool {
        let Some(delta) = self.scroll_delta_for_key(key) else {
            return false;
        };
        match self.focus {
            Focus::Left => self.scroll_left(delta),
            Focus::Output => self.scroll_output(delta),
            _ => return false,
        }
        true
    }

    fn scroll_delta_for_key(&self, key: KeyEvent) -> Option<isize> {
        let page = match self.focus {
            Focus::Left => self.left_area.height.saturating_sub(2).max(1) as isize,
            Focus::Output => self.output_area.height.saturating_sub(2).max(1) as isize,
            _ => 1,
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => Some(-1),
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => Some(1),
            KeyCode::PageUp => Some(-page),
            KeyCode::PageDown => Some(page),
            _ => None,
        }
    }

    fn scroll_left(&mut self, delta: isize) {
        let width = self.left_area.width.saturating_sub(2).max(1);
        let lines = if self.mode == AppMode::Learn {
            Paragraph::new(render_markdown_plain(&self.output))
                .wrap(Wrap { trim: false })
                .line_count(width)
        } else {
            Paragraph::new(problem_view::render(
                &self.problem,
                &self.state.settings.ui_language,
                self.state.settings.theme == "light",
            ))
            .wrap(Wrap { trim: false })
            .line_count(width)
        };
        self.left_scroll = scrolled(self.left_scroll, delta, lines, self.left_area);
    }

    fn scroll_output(&mut self, delta: isize) {
        let width = self.output_area.width.saturating_sub(2).max(1);
        let lines = if !self.show_output && self.mode == AppMode::Learn {
            Paragraph::new(self.learn_result.clone())
                .wrap(Wrap { trim: false })
                .line_count(width)
        } else {
            Paragraph::new(self.output_text())
                .wrap(Wrap { trim: false })
                .line_count(width)
        };
        self.output_scroll = scrolled(self.output_scroll, delta, lines, self.output_area);
    }

    pub(super) fn focus_command(&mut self) {
        if self.command.is_empty() {
            self.command.push('/');
            self.command_cursor = 1;
        }
        self.command_palette_cursor = 0;
        self.focus = Focus::Command;
    }

    pub(super) fn resting_focus(&self) -> Focus {
        if self.mode == AppMode::Home && !self.show_output {
            Focus::Home
        } else {
            Focus::None
        }
    }

    pub(super) fn submit_command(&mut self, value: &str) -> Result<()> {
        let value = value
            .trim()
            .strip_prefix('/')
            .unwrap_or(value.trim())
            .trim();
        self.handle_command(value)
    }
}

fn scrolled(current: u16, delta: isize, lines: usize, area: Rect) -> u16 {
    let viewport = area.height.saturating_sub(2).max(1) as usize;
    let max = lines.saturating_sub(viewport).min(u16::MAX as usize) as u16;
    let next = if delta < 0 {
        current.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        current.saturating_add(delta as u16)
    };
    next.min(max)
}
