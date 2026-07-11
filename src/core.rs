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
mod learning;
mod model;
mod problem_files;
mod profile;
mod progress;
mod render;
mod state;
mod syntax;

pub use crate::i18n::{UI_LANGUAGES, normalize_ui_language, ui_text};
pub use bank::*;
pub use judge::*;
pub use language::*;
#[cfg(test)]
pub(crate) use learning::record_syntax_result_for_lessons;
pub(crate) use learning::syntax_review_due_at;
pub use learning::*;
pub use model::*;
pub use problem_files::*;
pub use profile::{
    DIFFICULTIES, default_difficulty, normalize_difficulty, normalize_topic_list, parse_topic_list,
};
pub use progress::*;
pub use render::*;
pub use state::*;
pub use syntax::*;
pub(crate) use syntax::{
    localized_syntax_exercise_prompt, localized_syntax_language_delta, localized_syntax_objective,
    localized_syntax_prediction_prompt, localized_syntax_title, localized_syntax_transfer_trap,
    syntax_core_progress_count,
};
