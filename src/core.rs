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

pub(crate) fn regular_file_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => bail!("{} is not a regular file", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

pub(crate) fn create_dir_all_beneath(root: &Path, directory: &Path) -> Result<()> {
    fs::create_dir_all(root).with_context(|| format!("create data root {}", root.display()))?;
    let relative = directory.strip_prefix(root).with_context(|| {
        format!(
            "directory {} is outside data root {}",
            directory.display(),
            root.display()
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            bail!("unsafe directory path {}", directory.display());
        };
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => bail!("{} is not a regular directory", current.display()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .with_context(|| format!("create directory {}", current.display()))?;
            }
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", current.display()));
            }
        }
    }
    Ok(())
}
