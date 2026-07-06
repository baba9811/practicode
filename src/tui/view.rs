use super::*;

impl PracticodeApp {
    pub(super) fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(size);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(vertical[0]);
        let right_panes = if !self.show_output
            && self.mode == AppMode::Learn
            && !self.learn_result.is_empty()
            && body[1].height >= 6
        {
            let result_height = (body[1].height / 3).clamp(3, 7);
            let panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(result_height)])
                .split(body[1]);
            Some((panes[0], panes[1]))
        } else {
            None
        };
        self.code_area = if self.show_output {
            Rect::default()
        } else if let Some((code_area, _)) = right_panes {
            code_area
        } else {
            body[1]
        };
        self.output_area = if self.show_output {
            vertical[0]
        } else if let Some((_, result_area)) = right_panes {
            result_area
        } else {
            self.code_area
        };
        self.command_area = vertical[2];

        let light = self.state.settings.theme == "light";
        if !self.show_output {
            let left = if self.mode == AppMode::Learn {
                Text::from(render_markdown_plain(&self.output))
            } else {
                problem_view::render(&self.problem, &self.state.settings.ui_language, light)
            };
            let title = if self.mode == AppMode::Learn {
                ui_text(&self.state.settings.ui_language, "syntax")
            } else {
                ui_text(&self.state.settings.ui_language, "problem")
            };
            let problem = Paragraph::new(left)
                .style(Self::pane_style(light))
                .block(Self::block(title, light, false))
                .wrap(Wrap { trim: false });
            frame.render_widget(problem, body[0]);
        }

        if self.show_output {
            let text = if self.editing_notes {
                Text::from(
                    self.note_editor
                        .visible_text(self.output_area.height.saturating_sub(2) as usize),
                )
            } else {
                self.output_text()
            };
            let output = Paragraph::new(text)
                .style(Self::pane_style(light))
                .block(Self::block(
                    if self.editing_notes {
                        PROBLEM_NOTES_PATH
                    } else {
                        ui_text(&self.state.settings.ui_language, "output")
                    },
                    light,
                    self.focus != Focus::Command,
                ))
                .wrap(Wrap { trim: false });
            frame.render_widget(output, self.output_area);
        } else {
            let code = self
                .editor
                .visible_text(self.code_area.height.saturating_sub(2) as usize);
            let title = if self.mode == AppMode::Learn {
                format!("drill.{}", ext_for(&self.state.settings.language))
            } else {
                format!("solution.{}", ext_for(&self.state.settings.language))
            };
            let code = Paragraph::new(code)
                .style(Self::pane_style(light))
                .block(Self::block(&title, light, self.focus == Focus::Code));
            frame.render_widget(code, self.code_area);

            if self.mode == AppMode::Learn && !self.learn_result.is_empty() && right_panes.is_some()
            {
                let result = Paragraph::new(self.learn_result.clone())
                    .style(Self::pane_style(light))
                    .block(Self::block(
                        ui_text(&self.state.settings.ui_language, "drill_result"),
                        light,
                        false,
                    ))
                    .wrap(Wrap { trim: false });
                frame.render_widget(result, self.output_area);
            }
        }

        let status = Paragraph::new(self.status_text()).style(if light {
            Style::default()
                .fg(Color::Blue)
                .bg(Color::Rgb(219, 234, 254))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Rgb(200, 211, 245))
                .bg(Color::Rgb(21, 32, 51))
                .add_modifier(Modifier::BOLD)
        });
        frame.render_widget(status, vertical[1]);

        let command_text = if self.focus == Focus::Command || !self.command.is_empty() {
            self.command.clone()
        } else {
            ui_text(&self.state.settings.ui_language, "command_placeholder").to_string()
        };
        let command = Paragraph::new(command_text)
            .style(Self::pane_style(light))
            .block(Self::block(
                ui_text(&self.state.settings.ui_language, "command"),
                light,
                self.focus == Focus::Command,
            ))
            .wrap(Wrap { trim: false });
        frame.render_widget(command, vertical[2]);
        self.draw_command_palette(frame, vertical[2]);
        self.set_terminal_cursor(frame, self.code_area, vertical[2]);
    }

    pub(super) fn wants_mouse_capture(&self) -> bool {
        !self.show_output
    }

    pub(super) fn sync_mouse_capture(&mut self) {
        let want = self.wants_mouse_capture();
        if want == self.mouse_capture {
            return;
        }
        let result = if want {
            execute!(stdout(), EnableMouseCapture)
        } else {
            execute!(stdout(), DisableMouseCapture)
        };
        if result.is_ok() {
            self.mouse_capture = want;
        }
    }

    pub(super) fn disable_mouse_capture(&mut self) {
        if self.mouse_capture {
            let _ = execute!(stdout(), DisableMouseCapture);
            self.mouse_capture = false;
        }
    }

    pub(super) fn output_text(&self) -> Text<'static> {
        let light = self.state.settings.theme == "light";
        let title_style = if light {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };
        let label_style = if light {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };
        let body_style = if light {
            Style::default().fg(Color::Black)
        } else {
            Style::default().fg(Color::Rgb(229, 231, 235))
        };
        let code_style = if light {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(229, 231, 235))
        } else {
            Style::default()
                .fg(Color::Rgb(243, 244, 246))
                .bg(Color::Rgb(31, 41, 55))
        };
        if !self.busy_label.is_empty() {
            let elapsed = self
                .busy_started
                .map(|started| started.elapsed().as_secs())
                .unwrap_or_default();
            let mut lines = vec![Line::from(Span::styled(
                format!("{}{}  {}s", self.busy_body, self.busy_dots(), elapsed),
                title_style,
            ))];
            if self.busy_label == "next" {
                lines.extend([
                    Line::default(),
                    Line::from(Span::styled(self.busy_game_track(), code_style)),
                    Line::from(Span::styled(
                        ui_text(&self.state.settings.ui_language, "busy_warmup").to_string(),
                        body_style,
                    )),
                    Line::from(Span::styled(
                        format!(
                            "{}: {}    {}: {}",
                            ui_text(&self.state.settings.ui_language, "hits"),
                            self.busy_hits,
                            ui_text(&self.state.settings.ui_language, "misses"),
                            self.busy_misses
                        ),
                        label_style,
                    )),
                    Line::from(Span::styled(
                        ui_text(&self.state.settings.ui_language, "busy_commands_paused")
                            .to_string(),
                        body_style,
                    )),
                ]);
            }
            return Text::from(lines);
        }
        let output = if self.output_is_markdown {
            render_markdown_plain(&self.output)
        } else {
            self.output.clone()
        };
        let mut lines = Vec::new();
        for line in output.lines() {
            if line.is_empty() {
                lines.push(Line::default());
            } else if line.starts_with("PASS ")
                || line.starts_with("FAIL ")
                || line.starts_with("Case ")
                || line.starts_with("Next:")
                || line.starts_with("Fix:")
            {
                lines.push(Line::from(Span::styled(line.to_string(), title_style)));
            } else if matches!(
                line,
                "Input" | "Expected" | "Got" | "Stdout" | "Stderr" | "Compile" | "Error"
            ) {
                lines.push(Line::from(Span::styled(line.to_string(), label_style)));
            } else if line.starts_with("  ") {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line.trim_start().to_string(), code_style),
                ]));
            } else {
                lines.push(Line::from(Span::styled(line.to_string(), body_style)));
            }
        }
        Text::from(lines)
    }

    pub(super) fn draw_command_palette(&self, frame: &mut Frame, command_area: Rect) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() || command_area.y < 3 {
            return;
        }
        let height = ((suggestions.len() + 3) as u16).min(14).min(command_area.y);
        let area = Rect::new(
            command_area.x,
            command_area.y - height,
            command_area.width,
            height,
        );
        let selected = self.command_palette_cursor.min(suggestions.len() - 1);
        let visible = height.saturating_sub(2) as usize;
        let start = selected.saturating_sub(visible.saturating_sub(1));
        let mut lines = suggestions
            .iter()
            .enumerate()
            .skip(start)
            .take(visible)
            .map(|(index, hint)| {
                let marker = if index == selected { ">" } else { " " };
                format!(
                    "{marker} {:<16} {}",
                    hint.display,
                    ui_text(&self.state.settings.ui_language, hint.desc_key)
                )
            })
            .collect::<Vec<_>>();
        lines.push(ui_text(&self.state.settings.ui_language, "palette_hint").to_string());
        frame.render_widget(Clear, area);
        let light = self.state.settings.theme == "light";
        frame.render_widget(
            Paragraph::new(lines.join("\n"))
                .style(Self::pane_style(light))
                .block(Self::block(
                    ui_text(&self.state.settings.ui_language, "commands"),
                    light,
                    true,
                )),
            area,
        );
    }

    pub(super) fn block(title: &str, light: bool, active: bool) -> Block<'static> {
        let border = if active {
            if light {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            }
        } else if light {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let border = border.bg(Self::pane_bg(light));
        Block::default()
            .borders(Borders::ALL)
            .title(Self::pane_title(title, active))
            .style(Self::pane_style(light))
            .border_style(border)
    }

    pub(super) fn pane_style(light: bool) -> Style {
        if light {
            Style::default()
                .fg(Color::Rgb(17, 24, 39))
                .bg(Self::pane_bg(light))
        } else {
            Style::default()
                .fg(Color::Rgb(229, 231, 235))
                .bg(Self::pane_bg(light))
        }
    }

    pub(super) fn pane_bg(light: bool) -> Color {
        if light {
            Color::Rgb(248, 250, 252)
        } else {
            Color::Rgb(17, 24, 39)
        }
    }

    pub fn pane_style_for_test(light: bool) -> Style {
        Self::pane_style(light)
    }

    pub(super) fn pane_title(title: &str, active: bool) -> String {
        if active {
            format!("> {title}")
        } else {
            title.to_string()
        }
    }
    pub(super) fn set_terminal_cursor(
        &self,
        frame: &mut Frame,
        code_area: Rect,
        command_area: Rect,
    ) {
        match self.focus {
            Focus::Command => {
                let before = prefix(&self.command, self.command_cursor);
                let x = command_area
                    .x
                    .saturating_add(1)
                    .saturating_add(display_width(&before) as u16)
                    .min(command_area.right().saturating_sub(2));
                frame.set_cursor_position(Position::new(x, command_area.y.saturating_add(1)));
            }
            Focus::Code if !self.show_output => {
                if let Some(position) = self.editor.cursor_position(code_area) {
                    frame.set_cursor_position(position);
                }
            }
            Focus::Output if self.editing_notes => {
                if let Some(position) = self.note_editor.cursor_position(self.output_area) {
                    frame.set_cursor_position(position);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn tmp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("practicode-view-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn output_uses_full_body_so_terminal_drag_selection_has_no_side_pane() {
        let mut app = PracticodeApp::new(tmp_root("full-output")).unwrap();
        app.handle_command("help").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert_eq!(app.output_area, Rect::new(0, 0, 80, 20));
        assert_eq!(app.code_area, Rect::default());
    }

    #[test]
    fn learn_result_keeps_lesson_and_splits_right_pane() {
        let mut app = PracticodeApp::new(tmp_root("learn-result-split")).unwrap();
        app.handle_command("learn python").unwrap();
        app.handle_command("run").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert!(app.output.contains("Syntax"));
        assert!(app.learn_result.contains("PASS"));
        assert_ne!(app.output_area, Rect::new(0, 0, 80, 20));
        assert!(app.output_area.y > app.code_area.y);
        assert_eq!(app.output_area.x, app.code_area.x);
    }
}
