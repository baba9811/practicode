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
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return Ok(());
        }
        let position = Position::new(mouse.column, mouse.row);
        if self.command_area.contains(position) {
            self.focus_command();
        } else if self.mode == AppMode::Home && self.home_learn_area.contains(position) {
            self.home_choice = HomeChoice::Learn;
            self.open_home_choice()?;
        } else if self.mode == AppMode::Home && self.home_problems_area.contains(position) {
            self.home_choice = HomeChoice::Problems;
            self.open_home_choice()?;
        } else if self.show_output && self.output_area.contains(position) {
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
                self.focus = Focus::None;
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
                    self.focus = Focus::Code;
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
                    self.write_text_output("Closed list.");
                }
                _ => {
                    self.list_cursor = Some(cursor);
                    self.handle_global_shortcut(key)?;
                }
            }
            return Ok(());
        }
        if self.mode == AppMode::Home {
            match key.code {
                KeyCode::Left | KeyCode::Right => self.move_home_choice(),
                KeyCode::Enter | KeyCode::Char(' ') => self.open_home_choice()?,
                _ => self.handle_global_shortcut(key)?,
            }
            return Ok(());
        }
        if key.code == KeyCode::Esc && self.show_output {
            if self.mode == AppMode::Learn {
                self.show_current_syntax_lesson();
            } else {
                self.show_output = false;
                self.focus = Focus::Code;
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

    pub(super) fn focus_command(&mut self) {
        if self.command.is_empty() {
            self.command.push('/');
            self.command_cursor = 1;
        }
        self.command_palette_cursor = 0;
        self.focus = Focus::Command;
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
