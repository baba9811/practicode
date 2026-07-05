use anyhow::{Context, Result};
use crossterm::{cursor::SetCursorStyle, execute};
use std::{env, io::stdout};

pub mod ai;
pub mod core;
pub mod process;
pub mod text;
pub mod tui;

pub fn run_cli() -> Result<()> {
    let root = env::current_dir().context("read current directory")?;
    if env::args().any(|arg| arg == "--smoke") {
        let bank = core::load_bank(&root)?;
        let state = core::load_state(&root, &bank)?;
        let problem = core::problem_by_id(&bank, &state.current_problem).unwrap_or(&bank[0]);
        println!(
            "{}",
            core::localized(&problem.title, &state.settings.ui_language)
        );
        return Ok(());
    }

    let mut app = tui::PracticodeApp::new(root)?;
    let mut terminal = ratatui::init();
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    let result = app.run(&mut terminal);
    ratatui::restore();
    let _ = execute!(stdout(), SetCursorStyle::DefaultUserShape);
    result
}
