use crate::{
    ai::{
        ModelCatalog, append_problem_note, available_models, provider_status, read_problem_notes,
        run_ai_generate, run_ai_next, run_ai_prompt,
    },
    core::{
        AI_PROVIDERS, AppState, CLAUDE_AI_EFFORTS, CODEX_AI_EFFORTS, DIFFICULTIES, HistoryItem,
        LANGUAGES, PROBLEM_NOTES_PATH, Problem, THEMES, UI_LANGUAGES, current_syntax_lesson,
        ensure_problem_files, ensure_submission, ensure_syntax_submission, ext_for, give_up, judge,
        judge_path, load_bank, load_state, localized, next_problem, next_syntax_lesson,
        normalize_ai_effort, normalize_ai_provider, normalize_difficulty, normalize_language,
        normalize_next_source, normalize_ui_language, parse_language_list, parse_topic_list,
        parse_ui_language_list, previous_problem, problem_by_id, record_pass, record_syntax_pass,
        render_syntax_lesson, save_state, set_current_syntax_lesson, syntax_cases,
        syntax_language_name, syntax_progress_count, template_for, ui_text,
    },
    text::{
        byte_index, char_len, compose_hangul_jamo, display_width, prefix, render_markdown_plain,
    },
    update::{CURRENT_VERSION, UpdateCheck, check_latest_version},
};
use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
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
    io::stdout,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

mod actions;
mod command_handlers;
mod command_input;
mod commands;
mod editor;
mod events;
mod problem_list;
mod problem_view;
mod settings_panel;
mod status;
mod tasks;
mod view;
use self::commands::COMMAND_HINTS;
pub use self::editor::TextEditor;

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);

#[derive(Clone)]
struct CommandChoice {
    insert: String,
    display: String,
    desc_key: &'static str,
    keep_open: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Home,
    Code,
    Command,
    Output,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppMode {
    Home,
    Problems,
    Learn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HomeChoice {
    Learn,
    Problems,
}

pub struct PracticodeApp {
    root: PathBuf,
    bank: Vec<Problem>,
    state: AppState,
    problem: Problem,
    editor: TextEditor,
    note_editor: TextEditor,
    command: String,
    command_cursor: usize,
    command_palette_cursor: usize,
    output: String,
    learn_result: String,
    output_is_markdown: bool,
    showing_model_status: bool,
    editing_notes: bool,
    show_output: bool,
    focus: Focus,
    mode: AppMode,
    home_choice: HomeChoice,
    home_area: Rect,
    home_learn_area: Rect,
    home_problems_area: Rect,
    list_cursor: Option<usize>,
    settings_cursor: Option<usize>,
    busy_label: String,
    busy_body: String,
    busy_started: Option<Instant>,
    busy_frame: usize,
    busy_hits: usize,
    busy_misses: usize,
    task_rx: Option<Receiver<TaskResult>>,
    generate_rx: Option<Receiver<String>>,
    generate_bank_len: usize,
    generate_started: Option<Instant>,
    generate_notice: Option<String>,
    update_rx: Option<Receiver<UpdateCheck>>,
    model_rx: Option<Receiver<ModelCatalog>>,
    available_models: Vec<String>,
    available_models_provider: String,
    model_message: Option<String>,
    update_check: Option<UpdateCheck>,
    update_notice: Option<String>,
    last_update_check: Option<Instant>,
    code_area: Rect,
    output_area: Rect,
    command_area: Rect,
    mouse_capture: bool,
    should_quit: bool,
}

enum TaskResult {
    AiPrompt(String),
    Next {
        output: String,
        old_problem: String,
        fallback_to_local: bool,
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
            note_editor: TextEditor::default(),
            command: String::new(),
            command_cursor: 0,
            command_palette_cursor: 0,
            output: String::new(),
            learn_result: String::new(),
            output_is_markdown: false,
            showing_model_status: false,
            editing_notes: false,
            show_output: false,
            focus: Focus::Code,
            mode: AppMode::Problems,
            home_choice: HomeChoice::Learn,
            home_area: Rect::default(),
            home_learn_area: Rect::default(),
            home_problems_area: Rect::default(),
            list_cursor: None,
            settings_cursor: None,
            busy_label: String::new(),
            busy_body: String::new(),
            busy_started: None,
            busy_frame: 0,
            busy_hits: 0,
            busy_misses: 0,
            task_rx: None,
            generate_rx: None,
            generate_bank_len: 0,
            generate_started: None,
            generate_notice: None,
            update_rx: None,
            model_rx: None,
            available_models: Vec::new(),
            available_models_provider: String::new(),
            model_message: None,
            update_check: None,
            update_notice: None,
            last_update_check: None,
            code_area: Rect::default(),
            output_area: Rect::default(),
            command_area: Rect::default(),
            mouse_capture: false,
            should_quit: false,
        };
        app.load_code_editor()?;
        save_state(&app.root, &app.state)?;
        match app.state.settings.start_mode.as_str() {
            "learn" => app.action_learn("")?,
            "problems" => app.action_practice()?,
            _ => app.action_home()?,
        }
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.start_update_check();
        self.start_model_check();
        while !self.should_quit {
            self.sync_mouse_capture();
            terminal.draw(|frame| self.draw(frame))?;
            self.check_task();
            self.check_background_generation();
            self.check_update();
            self.maybe_start_periodic_update_check();
            self.start_model_check();
            self.check_models();
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => self.handle_key(key)?,
                    Event::Mouse(mouse) => self.handle_mouse(mouse)?,
                    _ => {}
                }
            }
            if !self.busy_label.is_empty() {
                self.busy_frame = (self.busy_frame + 1) % 32;
            }
        }
        self.save_code().ok();
        self.disable_mouse_capture();
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

    pub fn handle_mouse_for_test(&mut self, mouse: MouseEvent) -> Result<()> {
        self.handle_mouse(mouse)
    }

    pub fn set_pane_areas_for_test(&mut self, code: Rect, output: Rect, command: Rect) {
        self.code_area = code;
        self.output_area = output;
        self.command_area = command;
    }

    pub fn set_home_choice_areas_for_test(&mut self, learn: Rect, problems: Rect) {
        self.home_learn_area = learn;
        self.home_problems_area = problems;
    }

    pub fn set_home_area_for_test(&mut self, area: Rect) {
        self.home_area = area;
    }

    pub fn busy_label(&self) -> &str {
        &self.busy_label
    }

    pub fn busy_attempts_for_test(&self) -> usize {
        self.busy_hits + self.busy_misses
    }

    pub fn has_task(&self) -> bool {
        self.task_rx.is_some()
    }

    pub fn has_background_generation_for_test(&self) -> bool {
        self.generate_rx.is_some()
    }

    pub fn check_background_generation_for_test(&mut self) {
        self.check_background_generation();
    }

    pub fn should_quit_for_test(&self) -> bool {
        self.should_quit
    }

    pub fn status_text_for_test(&self) -> String {
        self.status_text()
    }

    pub fn wants_mouse_capture_for_test(&self) -> bool {
        self.wants_mouse_capture()
    }

    pub fn output_for_test(&self) -> &str {
        &self.output
    }

    pub fn learn_result_for_test(&self) -> &str {
        &self.learn_result
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
}
