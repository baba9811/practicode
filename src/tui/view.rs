use super::*;

impl PracticodeApp {
    pub(super) fn home_preview_text(&self) -> String {
        match self.home_choice {
            HomeChoice::Learn => {
                let (done, total) =
                    syntax_core_progress_count(&self.state, &self.state.settings.language);
                let now = unix_time_now();
                let due = crate::core::due_syntax_lesson_count(
                    &self.state,
                    &self.state.settings.language,
                    now,
                );
                let next = LearningSession::start(&self.state, &self.state.settings.language, now);
                let lang = &self.state.settings.ui_language;
                format!(
                    "{}\n\n{}: {}\n{}: {done}/{total}\n{}: {due}\n{}: {}",
                    ui_text(lang, "home_learn_choice"),
                    ui_text(lang, "progress_language"),
                    syntax_language_name(&self.state.settings.language),
                    ui_text(lang, "syntax_progress"),
                    ui_text(lang, "learning_due_reviews"),
                    ui_text(lang, "home_next_step"),
                    learning_step_label(lang, next.step()),
                )
            }
            HomeChoice::Problems => {
                let lang = &self.state.settings.ui_language;
                format!(
                    "{}\n\n{}: {}\n{}: {}\n{}: {}\n\n{}\n{}",
                    ui_text(lang, "home_practice_preview_title"),
                    ui_text(lang, "home_current"),
                    self.problem.id,
                    ui_text(lang, "difficulty"),
                    localized_status(lang, &self.problem.difficulty),
                    ui_text(lang, "home_status"),
                    localized_status(lang, &self.problem_status(&self.problem)),
                    ui_text(lang, "home_practice_run"),
                    ui_text(lang, "home_practice_next"),
                )
            }
        }
    }

    pub(super) fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();
        self.home_area = Rect::default();
        self.home_learn_area = Rect::default();
        self.home_problems_area = Rect::default();
        self.left_area = Rect::default();
        self.code_area = Rect::default();
        self.output_area = Rect::default();
        self.command_area = Rect::default();
        self.command_palette_area = Rect::default();
        let light = self.state.settings.theme == "light";
        if size.width < 60 || size.height < 16 {
            frame.render_widget(
                Paragraph::new(ui_text(&self.state.settings.ui_language, "resize_required"))
                    .style(Self::pane_style(light))
                    .wrap(Wrap { trim: false }),
                size,
            );
            return;
        }

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);
        let body = vertical[0];
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(body);
        let wide = size.width >= 100;

        if self.show_output {
            self.output_area = body;
        } else {
            match self.mode {
                AppMode::Home => {
                    self.home_area = if wide { panes[0] } else { body };
                    if wide {
                        self.output_area = panes[1];
                    }
                }
                AppMode::Learn if wide => {
                    self.left_area = panes[0];
                    if self.learning_session.view() == LearningView::Result {
                        self.output_area = panes[1];
                    } else {
                        self.code_area = panes[1];
                    }
                }
                AppMode::Learn => match self.learning_session.view() {
                    LearningView::Lesson => self.left_area = body,
                    LearningView::Code => self.code_area = body,
                    LearningView::Result => self.output_area = body,
                },
                AppMode::Problems if wide => {
                    self.left_area = panes[0];
                    self.code_area = panes[1];
                }
                AppMode::Problems => match self.practice_view {
                    PracticeView::Problem => self.left_area = body,
                    PracticeView::Code => self.code_area = body,
                },
            }
        }

        if self.mode == AppMode::Home && !self.show_output {
            let choices = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if wide {
                    [
                        Constraint::Length(7),
                        Constraint::Length(7),
                        Constraint::Min(1),
                    ]
                } else {
                    [
                        Constraint::Length(6),
                        Constraint::Length(6),
                        Constraint::Min(1),
                    ]
                })
                .split(self.home_area);
            self.home_learn_area = choices[0];
            self.home_problems_area = choices[1];
        }
        self.command_area = vertical[2];

        if !self.show_output {
            if self.mode == AppMode::Home {
                for (area, choice, title, description) in [
                    (
                        self.home_learn_area,
                        HomeChoice::Learn,
                        ui_text(&self.state.settings.ui_language, "home_learn_choice"),
                        ui_text(&self.state.settings.ui_language, "home_learn_description"),
                    ),
                    (
                        self.home_problems_area,
                        HomeChoice::Problems,
                        ui_text(&self.state.settings.ui_language, "home_practice_choice"),
                        ui_text(
                            &self.state.settings.ui_language,
                            "home_practice_description",
                        ),
                    ),
                ] {
                    frame.render_widget(
                        Paragraph::new(description)
                            .style(Self::pane_style(light))
                            .block(Self::block(
                                title,
                                light,
                                self.focus == Focus::Home && self.home_choice == choice,
                            ))
                            .wrap(Wrap { trim: false }),
                        area,
                    );
                }
                let help_area = Rect::new(
                    self.home_area.x,
                    self.home_problems_area.bottom(),
                    self.home_area.width,
                    self.home_area
                        .bottom()
                        .saturating_sub(self.home_problems_area.bottom()),
                );
                frame.render_widget(
                    Paragraph::new(ui_text(&self.state.settings.ui_language, "home_help"))
                        .style(Self::pane_style(light)),
                    help_area,
                );

                if self.output_area.width > 0 {
                    let right = Paragraph::new(self.home_preview_text())
                        .style(Self::pane_style(light))
                        .block(Self::block(
                            ui_text(&self.state.settings.ui_language, "home_preview"),
                            light,
                            false,
                        ))
                        .wrap(Wrap { trim: false });
                    frame.render_widget(right, self.output_area);
                }
            } else if self.left_area.width > 0 {
                let left = if self.mode == AppMode::Learn {
                    markdown_text(
                        &self.output,
                        light,
                        ui_text(&self.state.settings.ui_language, "empty_value"),
                    )
                } else {
                    problem_view::render(&self.problem, &self.state.settings.ui_language, light)
                };
                let title = if self.mode == AppMode::Learn {
                    ui_text(&self.state.settings.ui_language, "learning_view_lesson")
                } else {
                    ui_text(&self.state.settings.ui_language, "problem")
                };
                let problem = Paragraph::new(left)
                    .style(Self::pane_style(light))
                    .block(Self::block(title, light, self.focus == Focus::Left))
                    .wrap(Wrap { trim: false })
                    .scroll((self.left_scroll, 0));
                frame.render_widget(problem, self.left_area);
            }
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
                .wrap(Wrap { trim: false })
                .scroll((self.output_scroll, 0));
            frame.render_widget(output, self.output_area);
        } else if self.mode != AppMode::Home && self.code_area.width > 0 {
            let code = self
                .editor
                .visible_text(self.code_area.height.saturating_sub(2) as usize);
            let title = if self.mode == AppMode::Learn {
                format!(
                    "{} · {}.{}",
                    ui_text(&self.state.settings.ui_language, "learning_view_code"),
                    ui_text(&self.state.settings.ui_language, "pane_exercise"),
                    ext_for(&self.state.settings.language)
                )
            } else {
                format!(
                    "{}.{}",
                    ui_text(&self.state.settings.ui_language, "pane_solution"),
                    ext_for(&self.state.settings.language)
                )
            };
            let code = Paragraph::new(code)
                .style(Self::pane_style(light))
                .block(Self::block(&title, light, self.focus == Focus::Code));
            frame.render_widget(code, self.code_area);
        } else if self.mode == AppMode::Learn && self.output_area.width > 0 {
            let text = if self.learn_result.is_empty() {
                ui_text(&self.state.settings.ui_language, "result_empty").to_string()
            } else {
                self.learn_result.clone()
            };
            let result = Paragraph::new(text)
                .style(Self::pane_style(light))
                .block(Self::block(
                    ui_text(&self.state.settings.ui_language, "learning_view_result"),
                    light,
                    self.focus == Focus::Output,
                ))
                .wrap(Wrap { trim: false })
                .scroll((self.output_scroll, 0));
            frame.render_widget(result, self.output_area);
        }

        let status = Paragraph::new(self.status_text_for_width(size.width)).style(if light {
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

        let command_text = if self.focus == Focus::Command {
            format!(
                "[{}] {}",
                ui_text(&self.state.settings.ui_language, "focus_active"),
                self.command
            )
        } else if !self.command.is_empty() {
            self.command.clone()
        } else {
            format!(
                "{}: {}",
                ui_text(&self.state.settings.ui_language, "command"),
                ui_text(&self.state.settings.ui_language, "command_placeholder")
            )
        };
        let command = Paragraph::new(command_text)
            .style(Self::pane_style(light))
            .wrap(Wrap { trim: false });
        frame.render_widget(command, vertical[2]);
        self.draw_command_palette(frame, vertical[2]);
        self.set_terminal_cursor(frame, self.code_area, vertical[2]);
    }

    pub(super) fn wants_mouse_capture(&self) -> bool {
        !(self.show_output
            || (self.mode == AppMode::Learn
                && self.learning_session.view() == LearningView::Result
                && self.focus != Focus::Command))
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
                format!(
                    "{}{}  {}",
                    self.busy_text(),
                    self.busy_dots(),
                    self.elapsed_text(elapsed)
                ),
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
        if self.output_is_markdown {
            return markdown_text(
                &self.output,
                light,
                ui_text(&self.state.settings.ui_language, "empty_value"),
            );
        }
        let output = self.output.clone();
        let mut lines = Vec::new();
        let pass = format!(
            "{} ",
            ui_text(&self.state.settings.ui_language, "result_pass")
        );
        let fail = format!(
            "{} ",
            ui_text(&self.state.settings.ui_language, "result_fail")
        );
        let case = format!(
            "{} ",
            ui_text(&self.state.settings.ui_language, "judge_case")
        );
        for line in output.lines() {
            if line.is_empty() {
                lines.push(Line::default());
            } else if line.starts_with(&pass)
                || line.starts_with(&fail)
                || line.starts_with(&case)
                || line == ui_text(&self.state.settings.ui_language, "run_pass_next")
                || line == ui_text(&self.state.settings.ui_language, "run_fail_next")
            {
                lines.push(Line::from(Span::styled(line.to_string(), title_style)));
            } else if [
                "judge_input",
                "judge_expected",
                "judge_got",
                "judge_stdout",
                "judge_stderr",
                "judge_compile",
                "judge_error",
            ]
            .into_iter()
            .any(|key| line == ui_text(&self.state.settings.ui_language, key))
            {
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

    pub(super) fn draw_command_palette(&mut self, frame: &mut Frame, command_area: Rect) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() || command_area.y < 3 {
            return;
        }
        let show_ai_disclosure = suggestions
            .iter()
            .any(|hint| matches!(hint.desc_key, "cmd_hint" | "cmd_ask" | "cmd_ai"));
        let disclosure_rows = usize::from(show_ai_disclosure);
        let height = ((suggestions.len() + disclosure_rows + 3) as u16)
            .min(14)
            .min(command_area.y);
        let area = Rect::new(
            command_area.x,
            command_area.y - height,
            command_area.width,
            height,
        );
        self.command_palette_area = area;
        let selected = self.command_palette_cursor.min(suggestions.len() - 1);
        let visible = height.saturating_sub(3 + disclosure_rows as u16).max(1) as usize;
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
        if show_ai_disclosure {
            lines.push(
                ui_text(&self.state.settings.ui_language, "ai_context_disclosure").to_string(),
            );
        }
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
                let marker = format!(
                    "[{}] ",
                    ui_text(&self.state.settings.ui_language, "focus_active")
                );
                let x = command_area
                    .x
                    .saturating_add(display_width(&marker) as u16)
                    .saturating_add(display_width(&before) as u16)
                    .min(command_area.right().saturating_sub(1));
                frame.set_cursor_position(Position::new(x, command_area.y));
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

fn markdown_text(markdown: &str, light: bool, empty_label: &str) -> Text<'static> {
    let title_style = if light {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };
    let section_style = if light {
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

    let mut lines = Vec::new();
    let mut in_fence = false;
    let mut code_lines = Vec::new();
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                push_markdown_code_block(&mut lines, &code_lines, code_style, empty_label);
                code_lines.clear();
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            code_lines.push(line.to_string());
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim_start().to_string();
            let style = if trimmed.starts_with("# ") {
                title_style
            } else {
                section_style
            };
            lines.push(Line::from(Span::styled(heading, style)));
        } else if line.is_empty() {
            lines.push(Line::default());
        } else {
            lines.push(Line::from(Span::styled(line.replace('`', ""), body_style)));
        }
    }
    if in_fence {
        push_markdown_code_block(&mut lines, &code_lines, code_style, empty_label);
    }
    Text::from(lines)
}

fn push_markdown_code_block(
    lines: &mut Vec<Line<'static>>,
    code_lines: &[String],
    code_style: Style,
    empty_label: &str,
) {
    if code_lines.iter().all(|line| line.is_empty()) {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {empty_label} "), code_style),
        ]));
        return;
    }
    for line in code_lines {
        let body = if line.is_empty() { " " } else { line };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {body} "), code_style),
        ]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::which;
    use crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use ratatui::{Terminal, backend::TestBackend};

    fn tmp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("practicode-view-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    fn status_row_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let row = buffer.area.height - 2;
        (0..buffer.area.width)
            .map(|x| buffer[(x, row)].symbol())
            .collect()
    }

    fn draw_at(app: &mut PracticodeApp, width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        terminal
    }

    #[test]
    fn learning_layout_switches_at_the_exact_width_boundary() {
        for (width, height) in [(60, 16), (80, 24), (99, 30)] {
            let mut app = PracticodeApp::new(tmp_root(&format!("narrow-{width}"))).unwrap();
            app.handle_command("learn python").unwrap();

            let _terminal = draw_at(&mut app, width, height);

            assert_eq!(
                app.left_area,
                Rect::new(0, 0, width, height - 2),
                "{width}x{height}"
            );
            assert_eq!(app.output_area, Rect::default(), "{width}x{height}");
            assert_eq!(app.code_area, Rect::default(), "{width}x{height}");
            assert_eq!(app.command_area.height, 1);
            if width == 80 {
                assert!(app.left_area.width.saturating_sub(2) > 60);
            }
        }

        for (width, height) in [(100, 30), (140, 40)] {
            let mut app = PracticodeApp::new(tmp_root(&format!("wide-{width}"))).unwrap();
            app.handle_command("learn python").unwrap();

            let _terminal = draw_at(&mut app, width, height);

            assert!(app.left_area.width > 0, "{width}x{height}");
            assert!(app.code_area.width > 0, "{width}x{height}");
            assert!(
                app.code_area.width > app.left_area.width,
                "{width}x{height}"
            );
            assert!(app.left_area.right() <= app.code_area.x, "{width}x{height}");
            assert_eq!(app.left_area.y, app.code_area.y);
            assert_eq!(app.left_area.height, app.code_area.height);
            assert_eq!(app.command_area.height, 1);
        }
    }

    #[test]
    fn undersized_terminal_renders_only_the_localized_resize_message() {
        let mut app = PracticodeApp::new(tmp_root("resize-ko")).unwrap();
        app.set_ui_language("ko").unwrap();
        let _terminal = draw_at(&mut app, 60, 16);

        for (width, height) in [(59, 15), (59, 16), (60, 15)] {
            let terminal = draw_at(&mut app, width, height);
            let text = buffer_text(&terminal);
            let compact = text.replace(' ', "");
            assert!(
                compact.contains("터미널크기를60x16이상으로조정하세요."),
                "{width}x{height}: {text}"
            );
            assert!(!text.contains("PRACTICODE"), "{width}x{height}: {text}");
            assert_eq!(app.home_area, Rect::default());
            assert_eq!(app.home_learn_area, Rect::default());
            assert_eq!(app.home_problems_area, Rect::default());
            assert_eq!(app.left_area, Rect::default());
            assert_eq!(app.code_area, Rect::default());
            assert_eq!(app.output_area, Rect::default());
            assert_eq!(app.command_area, Rect::default());
        }

        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .unwrap();
        assert!(app.should_quit);
    }

    #[test]
    fn narrow_f6_selects_one_full_width_learning_view() {
        let mut app = PracticodeApp::new(tmp_root("narrow-f6")).unwrap();
        app.handle_command("learn python").unwrap();

        let _terminal = draw_at(&mut app, 80, 24);
        assert_eq!(app.left_area, Rect::new(0, 0, 80, 22));

        app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE))
            .unwrap();
        let _terminal = draw_at(&mut app, 80, 24);
        assert_eq!(app.code_area, Rect::new(0, 0, 80, 22));
        assert_eq!(app.left_area, Rect::default());

        app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE))
            .unwrap();
        let terminal = draw_at(&mut app, 80, 24);
        assert_eq!(app.left_area, Rect::default());
        assert_eq!(app.code_area, Rect::default());
        assert_eq!(app.output_area, Rect::new(0, 0, 80, 22));
        assert!(buffer_text(&terminal).contains("No result yet."));
        assert!(!app.wants_mouse_capture());
        assert!(app.status_text().contains("drag select to copy"));
        app.focus_command();
        assert!(app.wants_mouse_capture());
        assert!(app.status_text().contains("Enter submit | Esc cancel"));
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
            .unwrap();

        app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE))
            .unwrap();
        let _terminal = draw_at(&mut app, 80, 24);
        assert_eq!(app.left_area, Rect::new(0, 0, 80, 22));
        assert_eq!(app.code_area, Rect::default());
        assert_eq!(app.output_area, Rect::default());
    }

    #[test]
    fn command_palette_overlays_the_body_and_keeps_a_one_row_input() {
        let mut app = PracticodeApp::new(tmp_root("palette-overlay")).unwrap();
        app.handle_command("learn python").unwrap();
        let _terminal = draw_at(&mut app, 80, 24);
        let body = app.code_area;
        assert_eq!(app.command_area.height, 1);

        app.focus_command();
        let terminal = draw_at(&mut app, 80, 24);

        assert_eq!(app.code_area, body);
        assert_eq!(app.command_area.height, 1);
        assert!(buffer_text(&terminal).contains("Commands"));
        assert!(
            buffer_text(&terminal).contains("up/down select | Enter run | Esc cancel"),
            "palette hint was clipped"
        );

        app.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 2,
            row: 10,
            modifiers: KeyModifiers::NONE,
        })
        .unwrap();
        assert_eq!(app.focus, Focus::Command);
    }

    #[test]
    fn ai_palette_disclosure_is_fully_visible_at_supported_widths() {
        for language in UI_LANGUAGES {
            for command in ["/hint", "/ask"] {
                for width in [60, 80, 100] {
                    let mut app = PracticodeApp::new(tmp_root(&format!(
                        "ai-disclosure-{language}-{command}-{width}"
                    )))
                    .unwrap();
                    app.set_ui_language(language).unwrap();
                    app.focus_command();
                    app.command = command.to_string();
                    app.command_cursor = char_len(command);

                    let terminal = draw_at(&mut app, width, 24);
                    let rendered = buffer_text(&terminal).replace(' ', "");
                    let expected = ui_text(language, "ai_context_disclosure").replace(' ', "");
                    assert!(!expected.is_empty(), "{language}: missing disclosure copy");
                    assert!(
                        rendered.contains(&expected),
                        "{language} {command} {width}: {rendered}"
                    );
                }
            }
        }
    }

    #[test]
    fn narrow_home_cards_match_their_mouse_hitboxes_in_long_locales() {
        for language in ["ko", "es"] {
            let mut app = PracticodeApp::new(tmp_root(&format!("home-cards-{language}"))).unwrap();
            app.set_ui_language(language).unwrap();
            app.action_home().unwrap();
            let terminal = draw_at(&mut app, 60, 16);
            let buffer = terminal.backend().buffer();

            assert_eq!(
                buffer[(app.home_learn_area.x, app.home_learn_area.y)].symbol(),
                "┌"
            );
            assert_eq!(
                buffer[(app.home_problems_area.x, app.home_problems_area.y)].symbol(),
                "┌"
            );
            assert!(app.home_learn_area.bottom() <= app.home_problems_area.y);

            app.handle_mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: app.home_problems_area.x + 2,
                row: app.home_problems_area.y + 2,
                modifiers: KeyModifiers::NONE,
            })
            .unwrap();
            assert_eq!(app.mode, AppMode::Problems, "{language}");
        }
    }

    #[test]
    fn wide_home_cards_match_their_mouse_hitboxes_in_all_locales() {
        for language in UI_LANGUAGES {
            let mut app =
                PracticodeApp::new(tmp_root(&format!("wide-home-cards-{language}"))).unwrap();
            app.set_ui_language(language).unwrap();
            app.action_home().unwrap();
            let terminal = draw_at(&mut app, 140, 30);
            let buffer = terminal.backend().buffer();

            assert_eq!(
                buffer[(app.home_problems_area.x, app.home_problems_area.y)].symbol(),
                "┌",
                "{language}"
            );
            app.handle_mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: app.home_problems_area.x + 2,
                row: app.home_problems_area.y + 2,
                modifiers: KeyModifiers::NONE,
            })
            .unwrap();
            assert_eq!(app.mode, AppMode::Problems, "{language}");
        }
    }

    #[test]
    fn narrow_practice_opens_on_problem_and_f6_toggles_full_width_panes() {
        let mut app = PracticodeApp::new(tmp_root("narrow-practice-toggle")).unwrap();
        app.action_practice().unwrap();

        let _terminal = draw_at(&mut app, 60, 16);
        assert_eq!(app.left_area, Rect::new(0, 0, 60, 14));
        assert_eq!(app.code_area, Rect::default());

        app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE))
            .unwrap();
        let _terminal = draw_at(&mut app, 60, 16);
        assert_eq!(app.left_area, Rect::default());
        assert_eq!(app.code_area, Rect::new(0, 0, 60, 14));

        app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE))
            .unwrap();
        let _terminal = draw_at(&mut app, 60, 16);
        assert_eq!(app.left_area, Rect::new(0, 0, 60, 14));
        assert_eq!(app.code_area, Rect::default());
    }

    #[test]
    fn narrow_problem_scroll_uses_wrapped_visual_rows() {
        let mut app = PracticodeApp::new(tmp_root("wrapped-problem-scroll")).unwrap();
        app.problem.statement.insert(
            "en".to_string(),
            "Read every wrapped word before editing. ".repeat(80),
        );
        app.action_practice().unwrap();
        let _terminal = draw_at(&mut app, 60, 16);

        for _ in 0..3 {
            app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
                .unwrap();
        }

        assert!(
            app.left_scroll > 10,
            "scroll stopped at {}",
            app.left_scroll
        );
    }

    #[test]
    fn command_cursor_stays_on_the_single_input_row_with_hangul() {
        let mut app = PracticodeApp::new(tmp_root("command-cursor-ko")).unwrap();
        app.set_ui_language("ko").unwrap();
        app.focus_command();
        for char in "ㅇㅏㄴ".chars() {
            app.insert_command_char(char);
        }

        let terminal = draw_at(&mut app, 60, 16);
        let cursor = terminal.backend().cursor_position();
        assert_eq!(cursor.y, app.command_area.y);
        assert!(cursor.x < app.command_area.right());
    }

    #[test]
    fn supported_locales_and_themes_render_at_boundary_sizes_without_panics() {
        for language in UI_LANGUAGES {
            for theme in THEMES {
                let mut app =
                    PracticodeApp::new(tmp_root(&format!("matrix-{language}-{theme}"))).unwrap();
                app.set_ui_language(language).unwrap();
                app.set_theme(theme).unwrap();
                app.action_learn("python").unwrap();
                for (width, height) in [(60, 16), (80, 24), (100, 30), (140, 40)] {
                    let _terminal = draw_at(&mut app, width, height);
                }
            }
        }
    }

    #[test]
    fn compact_learning_status_keeps_every_primary_action_visible() {
        for language in UI_LANGUAGES {
            for width in [60, 80, 99, 100] {
                let mut app =
                    PracticodeApp::new(tmp_root(&format!("compact-status-{language}-{width}")))
                        .unwrap();
                app.set_ui_language(language).unwrap();
                app.action_learn("python").unwrap();
                let terminal = draw_at(&mut app, width, 24);
                let row = terminal.backend().buffer().area.height - 2;
                let rendered = (0..width)
                    .map(|x| terminal.backend().buffer()[(x, row)].symbol())
                    .collect::<String>();

                for action in ["/next", "F5", "F6", "F1"] {
                    assert!(
                        rendered.contains(action),
                        "{language} {width}: missing {action} in {rendered:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn compact_practice_status_keeps_navigation_and_judging_visible() {
        for language in UI_LANGUAGES {
            let mut app =
                PracticodeApp::new(tmp_root(&format!("compact-practice-status-{language}")))
                    .unwrap();
            app.set_ui_language(language).unwrap();
            app.action_practice().unwrap();
            let terminal = draw_at(&mut app, 60, 24);
            let row = terminal.backend().buffer().area.height - 2;
            let rendered = (0..60)
                .map(|x| terminal.backend().buffer()[(x, row)].symbol())
                .collect::<String>();

            for action in ["F1", "F6", "/run", "/next"] {
                assert!(
                    rendered.contains(action),
                    "{language}: missing {action} in {rendered:?}"
                );
            }
        }
    }

    #[test]
    fn overlay_status_takes_priority_over_mode_at_narrow_and_wide_widths() {
        for language in UI_LANGUAGES {
            for width in [60, 121] {
                let cases = [
                    ("help", "hint_output"),
                    ("profile", "hint_settings"),
                    ("problems", "hint_list"),
                    ("note", "hint_notes"),
                ];
                for (command, hint_key) in cases {
                    let mut app = PracticodeApp::new(tmp_root(&format!(
                        "overlay-status-{language}-{width}-{command}"
                    )))
                    .unwrap();
                    app.set_ui_language(language).unwrap();
                    app.action_learn("python").unwrap();
                    app.handle_command(command).unwrap();

                    let terminal = draw_at(&mut app, width, 24);
                    let rendered = status_row_text(&terminal).replace(' ', "");
                    let expected = ui_text(language, hint_key).replace(' ', "");
                    assert!(
                        rendered.contains(&expected),
                        "{language} {width} {command}: {rendered}"
                    );
                    assert!(
                        !rendered.contains("F5"),
                        "{language} {width} {command}: mode hint leaked: {rendered}"
                    );
                }
            }
        }
    }

    #[test]
    fn output_uses_full_body_so_terminal_drag_selection_has_no_side_pane() {
        let mut app = PracticodeApp::new(tmp_root("full-output")).unwrap();
        app.handle_command("help").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert_eq!(app.output_area, Rect::new(0, 0, 80, 22));
        assert_eq!(app.code_area, Rect::default());
    }

    #[test]
    fn narrow_learn_result_uses_the_full_body() {
        let mut app = PracticodeApp::new(tmp_root("learn-result-split")).unwrap();
        app.handle_command("learn python").unwrap();
        app.handle_command("next").unwrap();
        app.handle_command("next").unwrap();
        app.handle_command("run").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert!(app.output.contains("Exercise"));
        assert!(app.learn_result.contains("FAIL"));
        assert_eq!(app.output_area, Rect::new(0, 0, 80, 22));
        assert_eq!(app.left_area, Rect::default());
        assert_eq!(app.code_area, Rect::default());
    }

    #[test]
    fn learning_gate_selects_visible_result_at_narrow_and_wide_widths() {
        for (width, height) in [(80, 24), (100, 30)] {
            let mut app = PracticodeApp::new(tmp_root(&format!("gate-visible-{width}"))).unwrap();
            app.handle_command("learn python").unwrap();

            app.handle_command("run").unwrap();
            let terminal = draw_at(&mut app, width, height);
            let text = buffer_text(&terminal);

            assert_eq!(app.learning_session.view(), LearningView::Result);
            assert!(
                text.contains("Next: use /next until Exercise"),
                "{width}: {text}"
            );
            assert!(app.output_area.width > 0, "{width}");
            assert_eq!(app.code_area, Rect::default(), "{width}");
        }
    }

    #[test]
    fn manual_judge_selects_visible_result_at_narrow_and_wide_widths() {
        if which("python3").or_else(|| which("python")).is_none() {
            return;
        }
        for (width, height) in [(80, 24), (100, 30)] {
            let mut app = PracticodeApp::new(tmp_root(&format!("manual-visible-{width}"))).unwrap();
            app.handle_command("learn python").unwrap();
            app.handle_command("back").unwrap();

            app.handle_command("run").unwrap();
            let terminal = draw_at(&mut app, width, height);
            let text = buffer_text(&terminal);

            assert!(!app.learning_session.is_guided());
            assert_eq!(app.learning_session.view(), LearningView::Result);
            assert!(text.contains("FAIL"), "{width}: {text}");
            assert!(
                app.learn_result
                    .contains("Retry this exercise; no review is scheduled."),
                "{width}: {}",
                app.learn_result
            );
            assert!(!app.learn_result.contains("Next review (days)"));
            assert!(app.output_area.width > 0, "{width}");
            assert_eq!(app.code_area, Rect::default(), "{width}");
        }
    }

    #[test]
    fn reference_lesson_scrolls_vertically() {
        let mut app = PracticodeApp::new(tmp_root("lesson-scroll")).unwrap();
        app.handle_command("learn python").unwrap();
        app.handle_command("lesson").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(buffer_text(&terminal).contains("Language: Python"));

        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
            .unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert!(!buffer_text(&terminal).contains("Language: Python"));
    }

    #[test]
    fn lesson_markdown_renders_code_without_ascii_boxes() {
        let mut app = PracticodeApp::new(tmp_root("lesson-code-style")).unwrap();
        app.handle_command("learn rust").unwrap();

        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let text = buffer_text(&terminal);

        assert!(text.contains("fn main()"));
        assert!(!text.contains("+--"));
        let border_x = app.left_area.right().saturating_sub(1);
        let buffer = terminal.backend().buffer();
        for y in app.left_area.y + 1..app.left_area.bottom().saturating_sub(1) {
            assert_eq!(buffer[(border_x, y)].symbol(), "│");
        }
    }

    #[test]
    fn output_pane_scrolls_vertically() {
        let mut app = PracticodeApp::new(tmp_root("output-scroll")).unwrap();
        app.handle_command("help").unwrap();

        let backend = TestBackend::new(80, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(buffer_text(&terminal).contains("Help"));

        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
            .unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        assert!(!buffer_text(&terminal).contains("Help"));
    }

    #[test]
    fn output_scroll_uses_wrapped_visual_rows() {
        let mut app = PracticodeApp::new(tmp_root("wrapped-output-scroll")).unwrap();
        app.write_text_output(&"Inspect this wrapped output safely. ".repeat(80));
        let _terminal = draw_at(&mut app, 60, 16);

        for _ in 0..3 {
            app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
                .unwrap();
        }

        assert!(
            app.output_scroll > 10,
            "scroll stopped at {}",
            app.output_scroll
        );
    }

    #[test]
    fn home_pane_title_is_active_when_home_has_focus() {
        let mut app = PracticodeApp::new(tmp_root("home-active-title")).unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("> Continue today's session"));
    }

    #[test]
    fn home_learning_preview_describes_the_guided_session() {
        let app = PracticodeApp::new(tmp_root("home-learning-preview")).unwrap();
        let preview = app.home_preview_text();

        assert!(preview.contains("Continue today's session"));
        assert!(preview.contains("Due: 0"));
        assert!(preview.contains("Next step: Language delta"));
        assert!(!preview.contains("moves to the next lesson"));
    }

    #[test]
    fn home_preview_and_status_localize_problem_state_tokens() {
        let mut app = PracticodeApp::new(tmp_root("home-problem-tokens-ko")).unwrap();
        app.set_ui_language("ko").unwrap();
        app.action_home().unwrap();
        app.home_choice = HomeChoice::Problems;

        let preview = app.home_preview_text();
        assert!(preview.contains("난이도: 쉬움"), "{preview}");
        assert!(preview.contains("상태: 배정됨"), "{preview}");
        assert!(!preview.contains("easy"), "{preview}");

        app.action_practice().unwrap();
        let status = app.status_text();
        assert!(status.contains("| 쉬움 |"), "{status}");
        assert!(!status.contains("| easy |"), "{status}");
    }

    #[test]
    fn localized_judge_case_keeps_the_result_emphasis() {
        let mut app = PracticodeApp::new(tmp_root("judge-case-style-ko")).unwrap();
        app.set_ui_language("ko").unwrap();
        app.output = "케이스 1: 실패".to_string();
        app.output_is_markdown = false;

        let text = app.output_text();

        assert_eq!(text.lines[0].spans[0].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn editor_pane_titles_are_localized_in_every_supported_locale() {
        for language in UI_LANGUAGES {
            let mut app =
                PracticodeApp::new(tmp_root(&format!("localized-pane-title-{language}"))).unwrap();
            app.set_ui_language(language).unwrap();
            app.action_learn("rust").unwrap();
            let learn = draw_at(&mut app, 120, 30);
            let learn_text = buffer_text(&learn).replace(' ', "");
            let expected = ui_text(language, "pane_exercise").replace(' ', "");
            assert!(learn_text.contains(&expected), "{language}: {learn_text}");

            app.action_practice().unwrap();
            app.action_edit().unwrap();
            let practice = draw_at(&mut app, 120, 30);
            let practice_text = buffer_text(&practice).replace(' ', "");
            let expected = ui_text(language, "pane_solution").replace(' ', "");
            assert!(
                practice_text.contains(&expected),
                "{language}: {practice_text}"
            );
        }
    }
}
