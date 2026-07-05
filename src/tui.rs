use crate::{
    ai::{
        ModelCatalog, append_problem_note, available_models, provider_status, read_problem_notes,
        run_ai_next, run_ai_prompt,
    },
    core::{
        AI_PROVIDERS, AppState, HistoryItem, LANGUAGES, PROBLEM_NOTES_PATH, Problem, THEMES,
        UI_LANGUAGES, ensure_problem_files, ensure_submission, ext_for, give_up, judge, load_bank,
        load_state, localized, next_problem, normalize_ai_provider, normalize_language,
        normalize_next_source, normalize_ui_language, previous_problem, problem_by_id, record_pass,
        save_state, template_for, ui_text,
    },
    text::{
        byte_index, char_len, compose_hangul_jamo, display_width, prefix, render_markdown_plain,
    },
    update::{CURRENT_VERSION, UpdateCheck, check_latest_version},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

#[derive(Clone, Copy)]
struct CommandHint {
    insert: &'static str,
    display: &'static str,
    desc_key: &'static str,
    keep_open: bool,
    help: bool,
}

#[derive(Clone)]
struct CommandChoice {
    insert: String,
    display: String,
    desc_key: &'static str,
    keep_open: bool,
}

const COMMAND_HINTS: &[CommandHint] = &[
    CommandHint {
        insert: "run",
        display: "/run",
        desc_key: "cmd_run",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "edit",
        display: "/edit",
        desc_key: "cmd_edit",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "next",
        display: "/next",
        desc_key: "cmd_next",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "prev",
        display: "/prev",
        desc_key: "cmd_prev",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "list",
        display: "/list",
        desc_key: "cmd_list",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "open ",
        display: "/open <id>",
        desc_key: "cmd_open",
        keep_open: true,
        help: true,
    },
    CommandHint {
        insert: "giveup",
        display: "/giveup",
        desc_key: "cmd_giveup",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "hint ",
        display: "/hint <request>",
        desc_key: "cmd_hint",
        keep_open: true,
        help: true,
    },
    CommandHint {
        insert: "provider codex",
        display: "/provider codex",
        desc_key: "cmd_provider",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "provider claude",
        display: "/provider claude",
        desc_key: "cmd_provider",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "model auto",
        display: "/model auto",
        desc_key: "cmd_model_auto",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "model ",
        display: "/model <name>",
        desc_key: "cmd_model_custom",
        keep_open: true,
        help: false,
    },
    CommandHint {
        insert: "note ",
        display: "/note <text>",
        desc_key: "cmd_note",
        keep_open: true,
        help: true,
    },
    CommandHint {
        insert: "notes",
        display: "/notes",
        desc_key: "cmd_notes",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "lang python",
        display: "/lang python",
        desc_key: "cmd_lang",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "lang ts",
        display: "/lang ts",
        desc_key: "cmd_lang",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "lang java",
        display: "/lang java",
        desc_key: "cmd_lang",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "lang rust",
        display: "/lang rust",
        desc_key: "cmd_lang",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "ui en",
        display: "/ui en",
        desc_key: "cmd_ui",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "ui ko",
        display: "/ui ko",
        desc_key: "cmd_ui",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "ui ja",
        display: "/ui ja",
        desc_key: "cmd_ui",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "ui zh",
        display: "/ui zh",
        desc_key: "cmd_ui",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "ui es",
        display: "/ui es",
        desc_key: "cmd_ui",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "theme dark",
        display: "/theme dark",
        desc_key: "cmd_theme",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "theme light",
        display: "/theme light",
        desc_key: "cmd_theme",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "source local",
        display: "/source local",
        desc_key: "cmd_source",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "source ai",
        display: "/source ai",
        desc_key: "cmd_source",
        keep_open: false,
        help: false,
    },
    CommandHint {
        insert: "update",
        display: "/update",
        desc_key: "cmd_update",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "help",
        display: "/help",
        desc_key: "cmd_help",
        keep_open: false,
        help: true,
    },
    CommandHint {
        insert: "exit",
        display: "/exit",
        desc_key: "cmd_exit",
        keep_open: false,
        help: true,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Code,
    Command,
    Output,
    None,
}

pub struct PracticodeApp {
    root: PathBuf,
    bank: Vec<Problem>,
    state: AppState,
    problem: Problem,
    editor: TextEditor,
    command: String,
    command_cursor: usize,
    command_palette_cursor: usize,
    output: String,
    output_is_markdown: bool,
    showing_model_status: bool,
    show_output: bool,
    focus: Focus,
    list_cursor: Option<usize>,
    busy_label: String,
    busy_body: String,
    busy_frame: usize,
    task_rx: Option<Receiver<TaskResult>>,
    update_rx: Option<Receiver<UpdateCheck>>,
    model_rx: Option<Receiver<ModelCatalog>>,
    available_models: Vec<String>,
    available_models_provider: String,
    model_message: Option<String>,
    update_check: Option<UpdateCheck>,
    update_notice: Option<String>,
    should_quit: bool,
}

enum TaskResult {
    AiPrompt(String),
    Next {
        output: String,
        old_problem: String,
        force: bool,
    },
}

impl PracticodeApp {
    pub fn new(root: PathBuf) -> Result<Self> {
        let bank = load_bank(&root)?;
        let state = load_state(&root, &bank)?;
        let problem = problem_by_id(&bank, &state.current_problem)
            .cloned()
            .unwrap_or_else(|| bank[0].clone());
        let mut app = Self {
            root,
            bank,
            state,
            problem,
            editor: TextEditor::default(),
            command: String::new(),
            command_cursor: 0,
            command_palette_cursor: 0,
            output: String::new(),
            output_is_markdown: false,
            showing_model_status: false,
            show_output: false,
            focus: Focus::Code,
            list_cursor: None,
            busy_label: String::new(),
            busy_body: String::new(),
            busy_frame: 0,
            task_rx: None,
            update_rx: None,
            model_rx: None,
            available_models: Vec::new(),
            available_models_provider: String::new(),
            model_message: None,
            update_check: None,
            update_notice: None,
            should_quit: false,
        };
        app.load_code_editor()?;
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.start_update_check();
        self.start_model_check();
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.check_task();
            self.check_update();
            self.start_model_check();
            self.check_models();
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                self.handle_key(key)?;
            }
            if !self.busy_label.is_empty() {
                self.busy_frame = (self.busy_frame + 1) % 16;
            }
        }
        self.save_code().ok();
        Ok(())
    }

    pub fn handle_command_for_test(&mut self, value: &str) -> Result<()> {
        self.handle_command(value)
    }

    pub fn focus_command_for_test(&mut self) {
        self.focus_command();
    }

    pub fn insert_command_char_for_test(&mut self, char: char) {
        self.insert_command_char(char);
    }

    pub fn command_text(&self) -> &str {
        &self.command
    }

    pub fn command_cursor(&self) -> usize {
        self.command_cursor
    }

    pub fn handle_key_for_test(&mut self, key: KeyEvent) -> Result<()> {
        self.handle_key(key)
    }

    pub fn busy_label(&self) -> &str {
        &self.busy_label
    }

    pub fn has_task(&self) -> bool {
        self.task_rx.is_some()
    }

    pub fn status_text_for_test(&self) -> String {
        self.status_text()
    }

    pub fn output_for_test(&self) -> &str {
        &self.output
    }

    pub fn command_suggestions_for_test(&self) -> Vec<String> {
        self.command_suggestions()
            .into_iter()
            .map(|choice| choice.display)
            .collect()
    }

    pub fn set_available_models_for_test(&mut self, models: Vec<&str>) {
        self.available_models = models.into_iter().map(str::to_string).collect();
        self.available_models_provider = self.state.settings.ai_provider.clone();
        self.model_message = None;
    }

    pub fn set_model_message_for_test(&mut self, message: &str) {
        self.available_models.clear();
        self.available_models_provider = self.state.settings.ai_provider.clone();
        self.model_message = Some(message.to_string());
    }

    pub fn pane_title_for_test(title: &str, active: bool) -> String {
        Self::pane_title(title, active)
    }

    fn draw(&mut self, frame: &mut Frame) {
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

        let problem = Paragraph::new(self.problem_text())
            .block(Self::block(
                ui_text(&self.state.settings.ui_language, "problem"),
                self.state.settings.theme == "light",
                false,
            ))
            .wrap(Wrap { trim: false });
        frame.render_widget(problem, body[0]);

        if self.show_output {
            let text = if !self.busy_label.is_empty() {
                format!("{}{}", self.busy_body, self.busy_dots())
            } else if self.output_is_markdown {
                render_markdown_plain(&self.output)
            } else {
                self.output.clone()
            };
            let output = Paragraph::new(text)
                .block(Self::block(
                    ui_text(&self.state.settings.ui_language, "output"),
                    self.state.settings.theme == "light",
                    self.focus != Focus::Command,
                ))
                .wrap(Wrap { trim: false });
            frame.render_widget(output, body[1]);
        } else {
            let code = self
                .editor
                .visible_text(body[1].height.saturating_sub(2) as usize);
            let title = format!("solution.{}", ext_for(&self.state.settings.language));
            let code = Paragraph::new(code).block(Self::block(
                &title,
                self.state.settings.theme == "light",
                self.focus == Focus::Code,
            ));
            frame.render_widget(code, body[1]);
        }

        let status =
            Paragraph::new(self.status_text()).style(if self.state.settings.theme == "light" {
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
            .block(Self::block(
                ui_text(&self.state.settings.ui_language, "command"),
                self.state.settings.theme == "light",
                self.focus == Focus::Command,
            ))
            .wrap(Wrap { trim: false });
        frame.render_widget(command, vertical[2]);
        self.draw_command_palette(frame, vertical[2]);
        self.set_terminal_cursor(frame, body[1], vertical[2]);
    }

    fn problem_text(&self) -> Text<'static> {
        let lang = normalize_ui_language(&self.state.settings.ui_language);
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
        let meta_style = if light {
            Style::default().fg(Color::Rgb(75, 85, 99))
        } else {
            Style::default().fg(Color::Rgb(156, 163, 175))
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
        let number = self
            .problem
            .id
            .split_once('-')
            .map(|(number, _)| number)
            .unwrap_or(&self.problem.id);
        let mut lines = vec![
            Line::from(Span::styled(
                format!("{number}. {}", localized(&self.problem.title, &lang)),
                title_style,
            )),
            Line::from(Span::styled(
                format!(
                    "{}: {}    {}: {}",
                    ui_text(&lang, "difficulty"),
                    self.problem.difficulty,
                    ui_text(&lang, "topics"),
                    self.problem.topics.join(", ")
                ),
                meta_style,
            )),
        ];
        lines.push(Line::default());
        for line in localized(&self.problem.statement, &lang).trim_end().lines() {
            lines.push(Line::from(Span::styled(line.to_string(), body_style)));
        }
        Self::push_problem_section(
            &mut lines,
            ui_text(&lang, "input"),
            &localized(&self.problem.input, &lang),
            section_style,
            body_style,
        );
        Self::push_problem_section(
            &mut lines,
            ui_text(&lang, "output"),
            &localized(&self.problem.output, &lang),
            section_style,
            body_style,
        );
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            ui_text(&lang, "examples").to_string(),
            section_style,
        )));
        for (index, case) in self.problem.examples.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!("  {} {}", ui_text(&lang, "example"), index + 1),
                meta_style.add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("    {}", ui_text(&lang, "input")),
                meta_style,
            )));
            Self::push_code_lines(&mut lines, &case.input, code_style);
            lines.push(Line::from(Span::styled(
                format!("    {}", ui_text(&lang, "output")),
                meta_style,
            )));
            Self::push_code_lines(&mut lines, &case.output, code_style);
        }
        Text::from(lines)
    }

    fn push_problem_section(
        lines: &mut Vec<Line<'static>>,
        title: &str,
        body: &str,
        section_style: Style,
        body_style: Style,
    ) {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(title.to_string(), section_style)));
        for line in body.trim_end().lines() {
            lines.push(Line::from(Span::styled(format!("  {line}"), body_style)));
        }
    }

    fn push_code_lines(lines: &mut Vec<Line<'static>>, body: &str, code_style: Style) {
        let body = body.trim_end();
        if body.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled("<empty>".to_string(), code_style),
            ]));
            return;
        }
        for line in body.lines() {
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(line.to_string(), code_style),
            ]));
        }
    }

    fn draw_command_palette(&self, frame: &mut Frame, command_area: Rect) {
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
        frame.render_widget(
            Paragraph::new(lines.join("\n")).block(Self::block(
                ui_text(&self.state.settings.ui_language, "commands"),
                self.state.settings.theme == "light",
                true,
            )),
            area,
        );
    }

    fn block(title: &str, light: bool, active: bool) -> Block<'static> {
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
        Block::default()
            .borders(Borders::ALL)
            .title(Self::pane_title(title, active))
            .border_style(border)
    }

    fn pane_title(title: &str, active: bool) -> String {
        if active {
            format!("> {title}")
        } else {
            title.to_string()
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.focus {
            Focus::Command => self.handle_command_key(key),
            Focus::Code => self.handle_code_key(key),
            _ => self.handle_global_key(key),
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<()> {
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

    fn handle_code_key(&mut self, key: KeyEvent) -> Result<()> {
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

    fn handle_global_key(&mut self, key: KeyEvent) -> Result<()> {
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
        if key.code == KeyCode::Esc && self.show_output {
            self.show_output = false;
            self.focus = Focus::Code;
            return Ok(());
        }
        self.handle_global_shortcut(key)
    }

    fn handle_global_shortcut(&mut self, key: KeyEvent) -> Result<()> {
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

    fn focus_command(&mut self) {
        if self.command.is_empty() {
            self.command.push('/');
            self.command_cursor = 1;
        }
        self.command_palette_cursor = 0;
        self.focus = Focus::Command;
    }

    fn submit_command(&mut self, value: &str) -> Result<()> {
        let value = value
            .trim()
            .strip_prefix('/')
            .unwrap_or(value.trim())
            .trim();
        self.handle_command(value)
    }

    fn handle_command(&mut self, value: &str) -> Result<()> {
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
        if command != "list" {
            self.list_cursor = None;
        }
        match command {
            "run" | "r" => self.action_run()?,
            "edit" | "e" => self.action_edit()?,
            "next" | "n" => self.action_next(arg)?,
            "prev" | "previous" | "p" => self.action_previous()?,
            "giveup" | "give" | "g" => self.action_give_up()?,
            "list" => self.start_problem_list(),
            "open" | "o" if !arg.is_empty() => self.open_problem(arg)?,
            "lang" if arg.is_empty() => self.action_cycle_language()?,
            "lang" if LANGUAGES.contains(&arg) => self.set_language(arg)?,
            "ui" if arg.is_empty() => self.action_toggle_ui_language()?,
            "ui" => self.set_ui_language(&normalize_ui_language(arg))?,
            "theme" if arg.is_empty() => self.action_toggle_theme()?,
            "theme" if THEMES.contains(&arg) => self.set_theme(arg)?,
            "source" | "next-source" if arg.is_empty() => {
                self.write_text_output(&format!("Next source: {}", self.next_source_label()));
            }
            "source" | "next-source" if matches!(arg, "bank" | "local" | "ai") => {
                self.state.settings.next_source = normalize_next_source(arg);
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!("Next source: {}", self.next_source_label()));
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
            "hint" if arg.is_empty() => {
                self.start_ai_prompt("Give one concise hint for the current problem.")?
            }
            "hint" | "ask" | "ai" if !arg.is_empty() => self.start_ai_prompt(arg)?,
            "note" if !arg.is_empty() => self.append_note(arg)?,
            "note" | "notes" => self.show_notes()?,
            "update" => self.show_update_notice(),
            "exit" | "quit" | "q" => self.should_quit = true,
            _ => self.write_text_output(&format!("Unknown command: {value}\nTry /help.")),
        }
        Ok(())
    }

    fn action_edit(&mut self) -> Result<()> {
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn action_run(&mut self) -> Result<()> {
        self.save_code()?;
        let result = judge(&self.root, &self.problem, &self.state.settings);
        if result.passed {
            record_pass(&self.root, &self.problem, &mut self.state)?;
        }
        let headline = format!(
            "{} {}/{}",
            if result.passed { "PASS" } else { "FAIL" },
            result.passed_cases,
            result.total_cases
        );
        let next_step = if result.passed {
            "Next: /next"
        } else {
            "Fix code, then /run"
        };
        self.write_text_output(&format!("{headline}\n{}\n\n{next_step}", result.output));
        Ok(())
    }

    fn action_next(&mut self, request: &str) -> Result<()> {
        let request = request.trim();
        let old_problem = self.state.current_problem.clone();
        if !request.is_empty() {
            self.start_next_problem(old_problem, true, request.to_string());
            return Ok(());
        }
        if self.state.settings.next_source == "ai" {
            self.start_next_problem(old_problem, false, String::new());
            return Ok(());
        }
        if let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)? {
            self.problem = problem;
            self.load_code_editor()?;
            self.show_output = false;
            self.focus = Focus::Code;
            return Ok(());
        }
        self.start_next_problem(old_problem, true, String::new());
        Ok(())
    }

    fn start_next_problem(&mut self, old_problem: String, force: bool, request: String) {
        if self.task_rx.is_some() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "already_busy"));
            return;
        }
        self.start_busy(
            "next",
            ui_text(&self.state.settings.ui_language, "generating_next"),
        );
        let root = self.root.clone();
        let state = self.state.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_ai_next(&root, &state, force, &request);
            let _ = tx.send(TaskResult::Next {
                output,
                old_problem,
                force,
            });
        });
        self.task_rx = Some(rx);
    }

    fn finish_next_problem(
        &mut self,
        output: String,
        old_problem: String,
        force: bool,
    ) -> Result<()> {
        if self.state.settings.next_source == "ai" || force {
            self.bank = load_bank(&self.root)?;
            self.state = load_state(&self.root, &self.bank)?;
        }
        self.problem = problem_by_id(&self.bank, &self.state.current_problem)
            .cloned()
            .unwrap_or_else(|| self.bank[0].clone());
        if self.state.current_problem == old_problem {
            if let Some(problem) = next_problem(&self.root, &self.bank, &mut self.state)? {
                self.problem = problem;
            } else {
                self.write_text_output(&format!(
                    "{}{}No next problem is available yet.",
                    if output.is_empty() { "" } else { &output },
                    if output.is_empty() { "" } else { "\n\n" }
                ));
                return Ok(());
            }
        }
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn action_previous(&mut self) -> Result<()> {
        let old_problem = self.state.current_problem.clone();
        self.problem = previous_problem(&self.root, &self.bank, &mut self.state)?;
        if self.state.current_problem == old_problem {
            self.write_text_output("Already at the first known problem.");
        } else {
            self.load_code_editor()?;
            self.show_output = false;
            self.focus = Focus::Code;
        }
        Ok(())
    }

    fn action_give_up(&mut self) -> Result<()> {
        let answer = give_up(&self.root, &self.problem, &mut self.state)?;
        let language = normalize_language(&self.state.settings.language);
        self.write_output(&format!(
            "Answer for {language}:\n\n```{language}\n{}\n```",
            answer.trim_end()
        ));
        Ok(())
    }

    fn action_cycle_language(&mut self) -> Result<()> {
        let current = LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.language)
            .unwrap_or(0);
        self.set_language(LANGUAGES[(current + 1) % LANGUAGES.len()])
    }

    fn action_toggle_ui_language(&mut self) -> Result<()> {
        let current = UI_LANGUAGES
            .iter()
            .position(|language| language == &self.state.settings.ui_language)
            .unwrap_or(0);
        self.set_ui_language(UI_LANGUAGES[(current + 1) % UI_LANGUAGES.len()])
    }

    fn action_toggle_theme(&mut self) -> Result<()> {
        let current = THEMES
            .iter()
            .position(|theme| theme == &self.state.settings.theme)
            .unwrap_or(0);
        self.set_theme(THEMES[(current + 1) % THEMES.len()])
    }

    fn set_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.language = language.to_string();
        save_state(&self.root, &self.state)?;
        self.load_code_editor()?;
        self.show_output = false;
        self.focus = Focus::Code;
        Ok(())
    }

    fn set_ui_language(&mut self, language: &str) -> Result<()> {
        self.state.settings.ui_language = normalize_ui_language(language);
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("UI language: {}", self.state.settings.ui_language));
        Ok(())
    }

    fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.state.settings.theme = theme.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("Theme: {theme}"));
        Ok(())
    }

    fn start_ai_prompt(&mut self, prompt: &str) -> Result<()> {
        if self.task_rx.is_some() {
            self.write_text_output(ui_text(&self.state.settings.ui_language, "already_busy"));
            return Ok(());
        }
        self.save_code()?;
        let label = normalize_ai_provider(&self.state.settings.ai_provider);
        self.start_busy("ai", &format!("{label} is thinking"));
        let root = self.root.clone();
        let problem = self.problem.clone();
        let settings = self.state.settings.clone();
        let prompt = prompt.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let output = run_ai_prompt(&root, &problem, &settings, &prompt);
            let _ = tx.send(TaskResult::AiPrompt(output));
        });
        self.task_rx = Some(rx);
        Ok(())
    }

    fn check_task(&mut self) {
        let task = self.task_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(task) = task {
            self.task_rx = None;
            self.stop_busy();
            match task {
                TaskResult::AiPrompt(output) => self.write_output(&output),
                TaskResult::Next {
                    output,
                    old_problem,
                    force,
                } => {
                    if let Err(error) = self.finish_next_problem(output, old_problem, force) {
                        self.write_text_output(&format!("Next failed\n{error}"));
                    }
                }
            }
        }
    }

    fn check_update(&mut self) {
        let result = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.update_rx = None;
            self.update_check = Some(result.clone());
            if let UpdateCheck::Available(version) = &result {
                self.update_notice = Some(version.clone());
            }
        }
    }

    fn start_update_check(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(check_latest_version());
        });
        self.update_rx = Some(rx);
    }

    fn start_model_check(&mut self) {
        let provider = self.state.settings.ai_provider.clone();
        if self.model_rx.is_some() || self.available_models_provider == provider {
            return;
        }
        let query_provider = provider.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(available_models(&query_provider));
        });
        self.available_models_provider = provider;
        self.model_rx = Some(rx);
    }

    fn check_models(&mut self) {
        let models = self.model_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(catalog) = models {
            self.model_rx = None;
            self.available_models = catalog.models;
            self.model_message = catalog.message;
            if self.showing_model_status {
                self.output = self.model_status_text();
                self.output_is_markdown = false;
                self.show_output = true;
            }
        }
    }

    fn model_status_text(&self) -> String {
        let mut lines = vec![
            format!("AI provider: {}", self.state.settings.ai_provider),
            format!(
                "AI model: {}",
                if self.state.settings.ai_model == "auto" {
                    "auto (provider default)"
                } else {
                    self.state.settings.ai_model.as_str()
                }
            ),
            "Use /model auto to let the provider choose its default.".to_string(),
        ];
        if self.model_rx.is_some() {
            lines.push("Loading provider model list...".to_string());
        } else if self.available_models.is_empty() {
            lines.push(
                self.model_message
                    .clone()
                    .unwrap_or_else(|| "Provider model list is unavailable.".to_string()),
            );
            lines.push("Use /model <name> for a known model.".to_string());
        } else {
            lines.push("Available models:".to_string());
            lines.extend(
                self.available_models
                    .iter()
                    .map(|model| format!("- /model {model}")),
            );
        }
        lines.join("\n")
    }

    fn start_busy(&mut self, label: &str, body: &str) {
        self.busy_label = label.to_string();
        self.busy_body = body.to_string();
        self.busy_frame = 0;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn stop_busy(&mut self) {
        self.busy_label.clear();
        self.busy_body.clear();
        self.busy_frame = 0;
    }

    fn write_output(&mut self, output: &str) {
        self.showing_model_status = false;
        self.output = output.to_string();
        self.output_is_markdown = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn write_text_output(&mut self, output: &str) {
        self.showing_model_status = false;
        self.output = output.trim_end().to_string();
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn write_model_status(&mut self) {
        self.output = self.model_status_text();
        self.output_is_markdown = false;
        self.showing_model_status = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn show_update_notice(&mut self) {
        let lang = self.state.settings.ui_language.clone();
        if let Some(version) = &self.update_notice {
            self.write_text_output(&format!(
                "{}: practicode {version} (current {CURRENT_VERSION})\n\nnpm update -g practicode\ncargo install --force practicode",
                ui_text(&lang, "update_available")
            ));
        } else if self.update_rx.is_some() {
            self.write_text_output("Checking for updates...");
        } else if matches!(self.update_check, Some(UpdateCheck::Disabled)) {
            self.write_text_output(ui_text(&lang, "update_check_disabled"));
        } else if matches!(self.update_check, Some(UpdateCheck::Failed)) {
            self.write_text_output(ui_text(&lang, "update_check_failed"));
        } else {
            self.write_text_output(ui_text(&lang, "update_none"));
        }
    }

    fn append_note(&mut self, note: &str) -> Result<()> {
        append_problem_note(&self.root, note)?;
        self.write_text_output(&format!("Problem note saved to {PROBLEM_NOTES_PATH}."));
        Ok(())
    }

    fn show_notes(&mut self) -> Result<()> {
        let notes = read_problem_notes(&self.root)?;
        if notes.is_empty() {
            self.write_text_output("No notes yet. Add one with /note <text>.");
        } else {
            self.write_text_output(&format!("Problem notes ({PROBLEM_NOTES_PATH})\n\n{notes}"));
        }
        Ok(())
    }

    fn insert_command_char(&mut self, char: char) {
        let byte = byte_index(&self.command, self.command_cursor);
        self.command.insert(byte, char);
        self.command_cursor += 1;
        self.command_palette_cursor = 0;
        self.normalize_command_input();
    }

    fn delete_command_before_cursor(&mut self) {
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

    fn delete_command_at_cursor(&mut self) {
        if self.command_cursor >= char_len(&self.command) {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor);
        let end = byte_index(&self.command, self.command_cursor + 1);
        self.command.replace_range(start..end, "");
        self.command_palette_cursor = 0;
        self.normalize_command_input();
    }

    fn command_suggestions(&self) -> Vec<CommandChoice> {
        if self.focus != Focus::Command {
            return Vec::new();
        }
        let Some(query) = self.command.trim_start().strip_prefix('/') else {
            return Vec::new();
        };
        let query = query.to_lowercase();
        self.command_choices()
            .into_iter()
            .filter(|hint| hint.insert.starts_with(query.trim_start()))
            .collect()
    }

    fn command_choices(&self) -> Vec<CommandChoice> {
        let mut choices = Vec::new();
        for hint in COMMAND_HINTS {
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

    fn move_command_palette(&mut self, delta: isize) {
        let len = self.command_suggestions().len();
        if len == 0 {
            return;
        }
        let cursor = self.command_palette_cursor as isize;
        self.command_palette_cursor = ((cursor + delta).rem_euclid(len as isize)) as usize;
    }

    fn accept_command_palette(&mut self) -> Result<bool> {
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

    fn normalize_command_input(&mut self) {
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

    fn set_terminal_cursor(&self, frame: &mut Frame, code_area: Rect, command_area: Rect) {
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
            _ => {}
        }
    }

    fn load_code_editor(&mut self) -> Result<()> {
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        let text = fs::read_to_string(path).unwrap_or_default();
        self.editor.set_text(&text);
        Ok(())
    }

    fn save_code(&self) -> Result<()> {
        let path = ensure_submission(&self.root, &self.problem, &self.state.settings)?;
        fs::write(path, self.editor.text())?;
        Ok(())
    }

    fn start_problem_list(&mut self) {
        self.list_cursor = Some(self.current_problem_index());
        self.write_text_output(&self.render_problem_list());
    }

    fn render_problem_list(&self) -> String {
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

    fn current_problem_index(&self) -> usize {
        self.bank
            .iter()
            .position(|problem| problem.id == self.problem.id)
            .unwrap_or(0)
    }

    fn move_list_cursor(&mut self, delta: isize) {
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

    fn open_selected_problem(&mut self) -> Result<()> {
        if let Some(cursor) = self.list_cursor {
            let problem_id = self.bank[cursor].id.clone();
            self.list_cursor = None;
            self.open_problem(&problem_id)?;
        }
        Ok(())
    }

    fn open_problem(&mut self, query: &str) -> Result<()> {
        self.list_cursor = None;
        let Some(problem) = self.find_problem(query).cloned() else {
            self.write_text_output(&format!("Problem not found: {query}\nTry /list."));
            return Ok(());
        };
        self.problem = problem;
        self.state.current_problem = self.problem.id.clone();
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

    fn find_problem(&self, query: &str) -> Option<&Problem> {
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

    fn problem_status(&self, problem: &Problem) -> String {
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

    fn submission_status(&self, problem: &Problem) -> (String, String) {
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

    fn status_text(&self) -> String {
        let code_status = self.submission_status(&self.problem).0;
        let activity = if self.busy_label.is_empty() {
            "idle".to_string()
        } else {
            format!("{}{}", self.busy_body, self.busy_dots())
        };
        let tail = self
            .update_notice
            .as_ref()
            .map(|version| {
                format!(
                    "{}:{version} /update",
                    ui_text(&self.state.settings.ui_language, "update")
                )
            })
            .unwrap_or_else(|| self.mode_hint().to_string());
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

    fn next_source_label(&self) -> &'static str {
        if self.state.settings.next_source == "ai" {
            "ai"
        } else {
            "local"
        }
    }

    fn busy_dots(&self) -> String {
        ".".repeat(self.busy_frame / 4)
    }

    fn mode_hint(&self) -> &'static str {
        let lang = &self.state.settings.ui_language;
        match (self.focus, self.list_cursor.is_some(), self.show_output) {
            (Focus::Command, _, _) => ui_text(lang, "hint_command"),
            (_, true, _) => ui_text(lang, "hint_list"),
            (_, _, true) => ui_text(lang, "hint_output"),
            (Focus::Code, _, _) => ui_text(lang, "hint_code"),
            _ => ui_text(lang, "hint_idle"),
        }
    }

    fn help_text(&self) -> String {
        let lang = &self.state.settings.ui_language;
        let commands = COMMAND_HINTS
            .iter()
            .filter(|hint| hint.help)
            .map(|hint| format!("- `{}` {}", hint.display, ui_text(lang, hint.desc_key)))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "# {}\n\n## {}\n\n1. Type code in the right pane.\n2. Press `Esc`, then choose `/run` from the command palette.\n3. Use `/next` when it passes.\n\n## {}\n\n{}\n\n## {}\n\n- `/` opens the command palette outside the editor.\n- `↑/↓` selects a command and `Enter` accepts it.\n- `Esc` cancels the command palette or leaves output.\n\n## {}\n\n- stdout is shown when a case fails.\n- stderr is shown without affecting the expected stdout.",
            ui_text(lang, "help_title"),
            ui_text(lang, "daily_loop"),
            ui_text(lang, "commands"),
            commands,
            ui_text(lang, "keys"),
            ui_text(lang, "debug_prints"),
        )
    }
}

#[derive(Clone, Debug)]
pub struct TextEditor {
    lines: Vec<String>,
    row: usize,
    col: usize,
    scroll: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            row: 0,
            col: 0,
            scroll: 0,
        }
    }
}

impl TextEditor {
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.split('\n').map(str::to_string).collect();
        if text.ends_with('\n') {
            self.lines.pop();
            self.lines.push(String::new());
        }
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = 0;
        self.col = 0;
        self.scroll = 0;
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn visible_text(&mut self, height: usize) -> String {
        if self.row < self.scroll {
            self.scroll = self.row;
        } else if height > 0 && self.row >= self.scroll + height {
            self.scroll = self.row + 1 - height;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        self.lines
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height.max(1))
            .map(|(index, line)| {
                let cursor = if index == self.row { ">" } else { " " };
                format!("{cursor}{:>width$} {line}", index + 1, width = line_width)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cursor_position(&self, area: Rect) -> Option<Position> {
        if self.row < self.scroll {
            return None;
        }
        let visible_row = self.row - self.scroll;
        let inner_height = area.height.saturating_sub(2) as usize;
        if visible_row >= inner_height {
            return None;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        let prefix_width = 1 + line_width + 1;
        let line = self.lines.get(self.row)?;
        let text_before_cursor = prefix(line, self.col);
        let x = area
            .x
            .saturating_add(1)
            .saturating_add((prefix_width + display_width(&text_before_cursor)) as u16)
            .min(area.right().saturating_sub(2));
        let y = area.y.saturating_add(1).saturating_add(visible_row as u16);
        Some(Position::new(x, y))
    }

    pub fn insert_char(&mut self, char: char) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        self.lines[self.row].insert(byte, char);
        self.col += 1;
        self.normalize_current_line();
    }

    pub fn insert_newline(&mut self) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        let rest = self.lines[self.row].split_off(byte);
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    pub fn backspace(&mut self) {
        self.ensure_cursor();
        if self.col > 0 {
            let start = byte_index(&self.lines[self.row], self.col - 1);
            let end = byte_index(&self.lines[self.row], self.col);
            self.lines[self.row].replace_range(start..end, "");
            self.col -= 1;
            self.normalize_current_line();
        } else if self.row > 0 {
            let current = self.lines.remove(self.row);
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
            self.lines[self.row].push_str(&current);
        }
    }

    fn delete(&mut self) {
        self.ensure_cursor();
        if self.col < char_len(&self.lines[self.row]) {
            let start = byte_index(&self.lines[self.row], self.col);
            let end = byte_index(&self.lines[self.row], self.col + 1);
            self.lines[self.row].replace_range(start..end, "");
            self.normalize_current_line();
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        }
    }

    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
        }
    }

    fn move_right(&mut self) {
        if self.col < char_len(&self.lines[self.row]) {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    fn ensure_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = self.row.min(self.lines.len() - 1);
        self.col = self.col.min(char_len(&self.lines[self.row]));
    }

    fn normalize_current_line(&mut self) {
        let normalized = compose_hangul_jamo(&self.lines[self.row]);
        if normalized == self.lines[self.row] {
            self.col = self.col.min(char_len(&self.lines[self.row]));
            return;
        }
        let old_prefix = prefix(&self.lines[self.row], self.col);
        self.lines[self.row] = normalized;
        self.col = char_len(&compose_hangul_jamo(&old_prefix)).min(char_len(&self.lines[self.row]));
    }
}
