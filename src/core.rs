use crate::process::{CommandSpec, run_capture, which};
use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

mod bank;
mod judge;
mod language;
mod model;
mod problem_files;
mod profile;
mod progress;
mod render;
mod state;

pub use crate::i18n::{UI_LANGUAGES, normalize_ui_language, ui_text};
pub use bank::*;
pub use judge::*;
pub use language::*;
pub use model::*;
pub use problem_files::*;
pub use profile::{
    DIFFICULTIES, default_difficulty, normalize_difficulty, normalize_topic_list, parse_topic_list,
};
pub use progress::*;
pub use render::*;
pub use state::*;
