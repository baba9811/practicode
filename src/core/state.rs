use super::*;
use std::{fs::OpenOptions, io::Write};

pub fn load_state(root: &Path, bank: &[Problem]) -> Result<AppState> {
    let path = root.join(STATE_PATH);
    let backup = sibling_path(&path, ".bak");
    if !path.exists() {
        match fs::symlink_metadata(&backup) {
            Ok(metadata) if metadata.file_type().is_file() => {
                let contents =
                    fs::read(&backup).with_context(|| format!("read {}", backup.display()))?;
                replace_state_file(&path, &contents).with_context(|| {
                    format!("restore {} from {}", path.display(), backup.display())
                })?;
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect backup {}", backup.display()));
            }
        }
    }
    if !path.exists() {
        return Ok(AppState {
            current_problem: bank[0].id.clone(),
            settings: Settings::default(),
            solved: Vec::new(),
            history: vec![HistoryItem {
                id: bank[0].id.clone(),
                status: "assigned".to_string(),
            }],
            suggested_next_difficulty: "easy".to_string(),
            syntax_progress: HashMap::new(),
            current_syntax_lesson: HashMap::new(),
            syntax_mastery: HashMap::new(),
            completed_syntax_courses: Vec::new(),
        });
    }

    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut state: AppState =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if !bank
        .iter()
        .any(|problem| problem.id == state.current_problem)
    {
        state.current_problem = bank[0].id.clone();
    }
    normalize_settings(&mut state.settings);
    state.suggested_next_difficulty =
        normalize_suggested_difficulty(&state.suggested_next_difficulty);
    migrate_syntax_mastery(&mut state, super::learning::unix_timestamp_now());
    state.current_syntax_lesson = normalize_current_syntax_lessons(&state.current_syntax_lesson);
    if state.history.is_empty() {
        state.history.push(HistoryItem {
            id: state.current_problem.clone(),
            status: "assigned".to_string(),
        });
    }
    Ok(state)
}

pub fn save_state(root: &Path, state: &AppState) -> Result<()> {
    #[derive(Serialize)]
    struct StateFile<'a> {
        current_problem: &'a str,
        next_number: usize,
        suggested_next_difficulty: &'a str,
        settings: &'a Settings,
        solved: &'a [String],
        history: &'a [HistoryItem],
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        syntax_progress: &'a HashMap<String, Vec<String>>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        current_syntax_lesson: &'a HashMap<String, String>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        syntax_mastery: &'a HashMap<String, HashMap<String, LessonMastery>>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        completed_syntax_courses: &'a Vec<String>,
    }

    let path = root.join(STATE_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create state directory {}", parent.display()))?;
    }
    let file = StateFile {
        current_problem: &state.current_problem,
        next_number: state.history.len() + 1,
        suggested_next_difficulty: &state.suggested_next_difficulty,
        settings: &state.settings,
        solved: &state.solved,
        history: &state.history,
        syntax_progress: &state.syntax_progress,
        current_syntax_lesson: &state.current_syntax_lesson,
        syntax_mastery: &state.syntax_mastery,
        completed_syntax_courses: &state.completed_syntax_courses,
    };
    let text = serde_json::to_string_pretty(&file)
        .with_context(|| format!("serialize {}", path.display()))?
        + "\n";
    replace_state_file(&path, text.as_bytes())
}

fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

fn replace_state_file(path: &Path, contents: &[u8]) -> Result<()> {
    let temporary = sibling_path(path, ".tmp");
    let backup = sibling_path(path, ".bak");
    remove_file_if_exists(&temporary)?;

    let result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .with_context(|| format!("open {}", temporary.display()))?;
        file.write_all(contents)
            .with_context(|| format!("write {}", temporary.display()))?;
        file.flush()
            .with_context(|| format!("flush {}", temporary.display()))?;
        file.sync_all()
            .with_context(|| format!("sync {}", temporary.display()))?;
        drop(file);
        replace_state_file_inner(path, &temporary, &backup)
    })();

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("remove stale {}", path.display())),
    }
}

#[cfg(unix)]
fn replace_state_file_inner(path: &Path, temporary: &Path, backup: &Path) -> Result<()> {
    if path.exists() {
        remove_file_if_exists(backup)?;
        fs::copy(path, backup)
            .with_context(|| format!("preserve {} as {}", path.display(), backup.display()))?;
    }
    fs::rename(temporary, path)
        .with_context(|| format!("replace {} with {}", path.display(), temporary.display()))
}

#[cfg(windows)]
fn replace_state_file_inner(path: &Path, temporary: &Path, backup: &Path) -> Result<()> {
    let had_primary = path.exists();
    replace_state_file_windows_with(
        path,
        temporary,
        backup,
        had_primary,
        remove_file_if_exists,
        |from, to| {
            fs::rename(from, to)?;
            Ok(())
        },
    )
}

#[cfg(any(windows, test))]
fn replace_state_file_windows_with(
    path: &Path,
    temporary: &Path,
    backup: &Path,
    had_primary: bool,
    mut remove_backup: impl FnMut(&Path) -> Result<()>,
    mut rename: impl FnMut(&Path, &Path) -> Result<()>,
) -> Result<()> {
    if had_primary {
        remove_backup(backup)?;
        rename(path, backup)
            .with_context(|| format!("preserve {} as {}", path.display(), backup.display()))?;
    }
    if let Err(replace_error) = rename(temporary, path) {
        if had_primary && let Err(restore_error) = rename(backup, path) {
            bail!(
                "replace {}: {replace_error}; restore {}: {restore_error}",
                path.display(),
                backup.display()
            );
        }
        bail!(
            "replace {} with {}: {replace_error}",
            path.display(),
            temporary.display()
        );
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn replace_state_file_inner(path: &Path, temporary: &Path, backup: &Path) -> Result<()> {
    if path.exists() {
        remove_file_if_exists(backup)?;
        fs::copy(path, backup)
            .with_context(|| format!("preserve {} as {}", path.display(), backup.display()))?;
    }
    fs::rename(temporary, path)
        .with_context(|| format!("replace {} with {}", path.display(), temporary.display()))
}

pub fn normalize_settings(settings: &mut Settings) {
    settings.language = normalize_language(&settings.language);
    settings.ui_language = normalize_ui_language(&settings.ui_language);
    settings.theme = settings.theme.trim().to_lowercase();
    if !THEMES.contains(&settings.theme.as_str()) {
        settings.theme = "dark".to_string();
    }
    settings.start_mode = normalize_start_mode(&settings.start_mode);
    settings.difficulty = normalize_difficulty(&settings.difficulty);
    settings.topics = normalize_topic_list(&settings.topics);
    settings.avoid_topics = normalize_topic_list(&settings.avoid_topics);
    settings.generate_languages = normalize_language_list(&settings.generate_languages);
    settings.generate_ui_languages = normalize_ui_language_list(&settings.generate_ui_languages);
    settings.next_source = normalize_next_source(&settings.next_source);
    settings.ai_provider = normalize_ai_provider(&settings.ai_provider);
    if settings.ai_model.trim().is_empty() {
        settings.ai_model = default_ai_model();
    }
    settings.ai_effort = normalize_ai_effort(&settings.ai_provider, &settings.ai_effort);
}

pub fn normalize_start_mode(mode: &str) -> String {
    let mode = mode.trim().to_lowercase();
    match mode.as_str() {
        "home" | "learn" | "problems" => mode,
        _ => "home".to_string(),
    }
}

fn normalize_suggested_difficulty(difficulty: &str) -> String {
    match normalize_difficulty(difficulty).as_str() {
        "medium" => "medium".to_string(),
        "hard" => "hard".to_string(),
        _ => "easy".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn windows_recovery_keeps_backup_when_primary_is_absent_and_replace_fails() {
        let operations = RefCell::new(Vec::new());
        let error = replace_state_file_windows_with(
            Path::new("problem-state.json"),
            Path::new("problem-state.json.tmp"),
            Path::new("problem-state.json.bak"),
            false,
            |path| {
                operations
                    .borrow_mut()
                    .push(format!("remove {}", path.display()));
                Ok(())
            },
            |from, to| {
                operations
                    .borrow_mut()
                    .push(format!("rename {} {}", from.display(), to.display()));
                Err(anyhow!("injected replacement failure"))
            },
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("injected replacement failure"));
        assert_eq!(
            operations.into_inner(),
            ["rename problem-state.json.tmp problem-state.json"]
        );
    }

    #[test]
    fn windows_replacement_restores_primary_after_new_file_rename_fails() {
        let operations = RefCell::new(Vec::new());
        let error = replace_state_file_windows_with(
            Path::new("problem-state.json"),
            Path::new("problem-state.json.tmp"),
            Path::new("problem-state.json.bak"),
            true,
            |path| {
                operations
                    .borrow_mut()
                    .push(format!("remove {}", path.display()));
                Ok(())
            },
            |from, to| {
                operations
                    .borrow_mut()
                    .push(format!("rename {} {}", from.display(), to.display()));
                if from == Path::new("problem-state.json.tmp") {
                    Err(anyhow!("injected replacement failure"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            operations.into_inner(),
            [
                "remove problem-state.json.bak",
                "rename problem-state.json problem-state.json.bak",
                "rename problem-state.json.tmp problem-state.json",
                "rename problem-state.json.bak problem-state.json",
            ]
        );
        assert_eq!(
            error,
            "replace problem-state.json with problem-state.json.tmp: injected replacement failure"
        );
    }
}
