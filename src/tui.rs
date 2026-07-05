use crate::{
    ai::{append_problem_note, read_problem_notes, run_ai_next, run_ai_prompt},
    core::{
        AI_PROVIDERS, AppState, HistoryItem, LANGUAGES, PROBLEM_NOTES_PATH, Problem, THEMES,
        UI_LANGUAGES, ensure_problem_files, ensure_submission, ext_for, give_up, judge, load_bank,
        load_state, localized, next_problem, normalize_ai_provider, normalize_language,
        normalize_next_source, previous_problem, problem_by_id, record_pass, render_problem,
        save_state, template_for,
    },
    text::{
        byte_index, char_len, compose_hangul_jamo, display_width, prefix, render_markdown_plain,
    },
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

pub const HELP: &str = r#"# Help

## Daily loop

1. Type code in the right pane.
2. Press `Esc`, then `/run`.
3. Use `/next` when it passes.

## Commands

- `/run` judge current submission
- `/edit` focus the code editor
- `/next [request]` next problem, optionally with a request
- `/prev` previous problem
- `/list` choose from problem list
- `/open 2` open by number, id, or slug
- `/giveup` show answer
- `/ai hint` ask the selected AI about current problem + code
- `/provider codex|claude`
- `/model auto|sonnet|opus|...`
- `/note prefer strings this week`
- `/notes` show next-problem notes
- `/lang python|ts|java|rust`
- `/ui ko|en`
- `/theme dark|light`
- `/source bank|ai`
- `/exit` quit

## Keys

- `Esc` leaves the editor or output pane
- `/` opens the command bar when the editor is not focused
- `?` opens this help when the editor is not focused
- `up/down` or `j/k` move in `/list`

## Debug prints

- stdout prints are shown when a case fails
- stderr prints are shown without affecting the expected stdout
"#;

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
    output: String,
    output_is_markdown: bool,
    show_output: bool,
    focus: Focus,
    list_cursor: Option<usize>,
    busy_label: String,
    busy_body: String,
    busy_frame: usize,
    task_rx: Option<Receiver<TaskResult>>,
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
            output: String::new(),
            output_is_markdown: false,
            show_output: false,
            focus: Focus::Code,
            list_cursor: None,
            busy_label: String::new(),
            busy_body: String::new(),
            busy_frame: 0,
            task_rx: None,
            should_quit: false,
        };
        app.load_code_editor()?;
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.check_task();
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                self.handle_key(key)?;
            }
            if !self.busy_label.is_empty() {
                self.busy_frame = (self.busy_frame + 1) % 4;
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

    pub fn busy_label(&self) -> &str {
        &self.busy_label
    }

    pub fn has_task(&self) -> bool {
        self.task_rx.is_some()
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

        let problem = Paragraph::new(render_markdown_plain(&render_problem(
            &self.problem,
            &self.state.settings.ui_language,
        )))
        .block(Self::block("Problem", self.state.settings.theme == "light"))
        .wrap(Wrap { trim: false });
        frame.render_widget(problem, body[0]);

        if self.show_output {
            let text = if !self.busy_label.is_empty() {
                format!("{}{}", self.busy_body, ".".repeat(self.busy_frame))
            } else if self.output_is_markdown {
                render_markdown_plain(&self.output)
            } else {
                self.output.clone()
            };
            let output = Paragraph::new(text)
                .block(Self::block("Output", self.state.settings.theme == "light"))
                .wrap(Wrap { trim: false });
            frame.render_widget(output, body[1]);
        } else {
            let code = self
                .editor
                .visible_text(body[1].height.saturating_sub(2) as usize);
            let title = format!("solution.{}", ext_for(&self.state.settings.language));
            let code = Paragraph::new(code)
                .block(Self::block(&title, self.state.settings.theme == "light"));
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
            "/run, /next easy string problem, /ai hint, /help".to_string()
        };
        let command = Paragraph::new(command_text)
            .block(Self::block("Command", self.state.settings.theme == "light"))
            .wrap(Wrap { trim: false });
        frame.render_widget(command, vertical[2]);
        self.set_terminal_cursor(frame, body[1], vertical[2]);
    }

    fn block(title: &str, light: bool) -> Block<'_> {
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(if light {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Cyan)
            })
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
                self.focus = Focus::None;
            }
            KeyCode::Enter => {
                let value = self.command.trim().to_string();
                self.command.clear();
                self.command_cursor = 0;
                self.focus = Focus::None;
                self.submit_command(&value)?;
            }
            KeyCode::Backspace => self.delete_command_before_cursor(),
            KeyCode::Delete => self.delete_command_at_cursor(),
            KeyCode::Left => self.command_cursor = self.command_cursor.saturating_sub(1),
            KeyCode::Right => {
                self.command_cursor = (self.command_cursor + 1).min(char_len(&self.command));
            }
            KeyCode::Home => self.command_cursor = 0,
            KeyCode::End => self.command_cursor = char_len(&self.command),
            KeyCode::Char('?') if self.command.trim().is_empty() || self.command.trim() == "/" => {
                self.command.clear();
                self.command_cursor = 0;
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
            self.write_output(HELP);
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
            "ui" if UI_LANGUAGES.contains(&arg) => self.set_ui_language(arg)?,
            "theme" if arg.is_empty() => self.action_toggle_theme()?,
            "theme" if THEMES.contains(&arg) => self.set_theme(arg)?,
            "source" | "next-source" if arg.is_empty() => {
                self.write_text_output(&format!(
                    "Next source: {}",
                    self.state.settings.next_source
                ));
            }
            "source" | "next-source" if matches!(arg, "bank" | "ai") => {
                self.state.settings.next_source = normalize_next_source(arg);
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!(
                    "Next source: {}",
                    self.state.settings.next_source
                ));
            }
            "ai-next-command" if !arg.is_empty() => {
                self.state.settings.ai_next_command = arg.to_string();
                self.state.settings.next_source = "ai".to_string();
                save_state(&self.root, &self.state)?;
                self.write_text_output("AI next command saved.");
            }
            "provider" | "ai-provider" if arg.is_empty() => {
                self.write_text_output(&format!(
                    "AI provider: {}",
                    self.state.settings.ai_provider
                ));
            }
            "provider" | "ai-provider" if AI_PROVIDERS.contains(&arg) => {
                self.state.settings.ai_provider = normalize_ai_provider(arg);
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!(
                    "AI provider: {}",
                    self.state.settings.ai_provider
                ));
            }
            "model" if arg.is_empty() => {
                self.write_text_output(&format!("AI model: {}", self.state.settings.ai_model));
            }
            "model" => {
                self.state.settings.ai_model = arg.to_string();
                save_state(&self.root, &self.state)?;
                self.write_text_output(&format!("AI model: {arg}"));
            }
            "ai" if !arg.is_empty() => self.start_ai_prompt(arg)?,
            "note" if !arg.is_empty() => self.append_note(arg)?,
            "note" | "notes" => self.show_notes()?,
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
            self.write_text_output("Already busy.");
            return;
        }
        self.start_busy("next", "Generating next problem");
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
        self.state.settings.ui_language = language.to_string();
        save_state(&self.root, &self.state)?;
        self.write_text_output(&format!("UI language: {language}"));
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
            self.write_text_output("Already busy.");
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
        self.output = output.to_string();
        self.output_is_markdown = true;
        self.show_output = true;
        self.focus = Focus::Output;
    }

    fn write_text_output(&mut self, output: &str) {
        self.output = output.trim_end().to_string();
        self.output_is_markdown = false;
        self.show_output = true;
        self.focus = Focus::Output;
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
        self.normalize_command_input();
    }

    fn delete_command_at_cursor(&mut self) {
        if self.command_cursor >= char_len(&self.command) {
            return;
        }
        let start = byte_index(&self.command, self.command_cursor);
        let end = byte_index(&self.command, self.command_cursor + 1);
        self.command.replace_range(start..end, "");
        self.normalize_command_input();
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
        format!(
            " PRACTICODE | {} | {} | {} | {} | code:{} | {} | next:{} | ai:{}/{} | {} ",
            self.problem.id,
            self.problem.difficulty,
            self.busy_status(),
            self.problem_status(&self.problem),
            code_status,
            self.state.settings.language,
            self.state.settings.next_source,
            self.state.settings.ai_provider,
            self.state.settings.ai_model,
            self.mode_hint(),
        )
    }

    fn busy_status(&self) -> String {
        if self.busy_label.is_empty() {
            "idle".to_string()
        } else {
            format!("busy:{}{}", self.busy_label, ".".repeat(self.busy_frame))
        }
    }

    fn mode_hint(&self) -> &'static str {
        match (self.focus, self.list_cursor.is_some(), self.show_output) {
            (Focus::Command, _, _) => "Enter submit | Esc cancel",
            (_, true, _) => "up/down move | Enter open | Esc close",
            (_, _, true) => "Esc code | / command | ? help",
            (Focus::Code, _, _) => "Esc then / command",
            _ => "/ command | ? help",
        }
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
