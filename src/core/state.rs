use super::*;

pub fn load_state(root: &Path, bank: &[Problem]) -> Result<AppState> {
    let path = root.join(STATE_PATH);
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
    state.syntax_progress = normalize_syntax_progress(&state.syntax_progress);
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
    }

    let path = root.join(STATE_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = StateFile {
        current_problem: &state.current_problem,
        next_number: state.history.len() + 1,
        suggested_next_difficulty: &state.suggested_next_difficulty,
        settings: &state.settings,
        solved: &state.solved,
        history: &state.history,
        syntax_progress: &state.syntax_progress,
    };
    fs::write(path, serde_json::to_string_pretty(&file)? + "\n")?;
    Ok(())
}

pub fn normalize_settings(settings: &mut Settings) {
    settings.language = normalize_language(&settings.language);
    settings.ui_language = normalize_ui_language(&settings.ui_language);
    if !THEMES.contains(&settings.theme.as_str()) {
        settings.theme = "dark".to_string();
    }
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
