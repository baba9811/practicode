use crate::core::{
    AI_PROVIDERS, AppState, CLAUDE_AI_EFFORTS, CODEX_AI_EFFORTS, DIFFICULTIES, LANGUAGES, THEMES,
    UI_LANGUAGES, normalize_ai_effort, ui_text,
};

pub(super) struct SettingsChange {
    pub reload_editor: bool,
    pub provider_changed: bool,
    pub edit_notes: bool,
}

const AI_PROVIDER_ROW: usize = 4;
pub(super) const AI_MODEL_ROW: usize = 5;
const AI_EFFORT_ROW: usize = 6;
const NOTE_ROW: usize = 7;
const TOGGLE_START: usize = 8;

pub(super) fn row_count() -> usize {
    TOGGLE_START + LANGUAGES.len() + UI_LANGUAGES.len()
}

pub(super) fn render(
    state: &AppState,
    cursor: Option<usize>,
    available_models: &[String],
    models_loading: bool,
) -> String {
    let settings = &state.settings;
    let ui_language = settings.ui_language.as_str();
    let topics = list_or_none(&settings.topics, ui_language);
    let avoid = list_or_none(&settings.avoid_topics, ui_language);
    let generate_languages = list_or_all(&settings.generate_languages, ui_language);
    let generate_ui_languages = list_or_all(&settings.generate_ui_languages, ui_language);
    let mut lines = vec![
        ui_text(ui_language, "settings_title").to_string(),
        String::new(),
        ui_text(ui_language, "settings_instructions").to_string(),
        String::new(),
        row(
            cursor,
            0,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_code_language"),
                settings.language
            ),
        ),
        row(
            cursor,
            1,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_ui_language"),
                settings.ui_language
            ),
        ),
        row(
            cursor,
            2,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_theme"),
                settings.theme
            ),
        ),
        row(
            cursor,
            3,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_difficulty"),
                settings.difficulty
            ),
        ),
        String::new(),
        format!(
            "{}: {topics}",
            ui_text(ui_language, "settings_preferred_topics")
        ),
        format!("{}: {avoid}", ui_text(ui_language, "settings_avoid_topics")),
        format!(
            "{}: {generate_languages}",
            ui_text(ui_language, "settings_generated_answer_languages")
        ),
        format!(
            "{}: {generate_ui_languages}",
            ui_text(ui_language, "settings_generated_ui_languages")
        ),
        row(
            cursor,
            AI_PROVIDER_ROW,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_ai_provider"),
                settings.ai_provider
            ),
        ),
        row(
            cursor,
            AI_MODEL_ROW,
            &format!(
                "{}: {}{}",
                ui_text(ui_language, "settings_ai_model"),
                if settings.ai_model == "auto" {
                    ui_text(ui_language, "settings_provider_default")
                } else {
                    settings.ai_model.as_str()
                },
                if models_loading {
                    format!(" ({})", ui_text(ui_language, "settings_model_loading"))
                } else if available_models.is_empty() {
                    format!(" ({})", ui_text(ui_language, "settings_model_load_hint"))
                } else {
                    String::new()
                }
            ),
        ),
        row(
            cursor,
            AI_EFFORT_ROW,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_ai_effort"),
                if settings.ai_effort == "auto" {
                    ui_text(ui_language, "settings_provider_default")
                } else {
                    settings.ai_effort.as_str()
                }
            ),
        ),
        row(
            cursor,
            NOTE_ROW,
            &format!(
                "{}: {}",
                ui_text(ui_language, "settings_problem_notes"),
                ui_text(ui_language, "settings_note_action")
            ),
        ),
        String::new(),
        ui_text(ui_language, "settings_answer_toggles").to_string(),
    ];
    for (index, language) in LANGUAGES.iter().enumerate() {
        let row_index = TOGGLE_START + index;
        let checked = generate_language_enabled(state, language);
        lines.push(row(
            cursor,
            row_index,
            &format!("{} {language}", checkbox(checked)),
        ));
    }
    lines.push(String::new());
    lines.push(ui_text(ui_language, "settings_ui_toggles").to_string());
    for (index, language) in UI_LANGUAGES.iter().enumerate() {
        let row_index = TOGGLE_START + LANGUAGES.len() + index;
        let checked = generate_ui_language_enabled(state, language);
        lines.push(row(
            cursor,
            row_index,
            &format!("{} {language}", checkbox(checked)),
        ));
    }
    lines.extend([
        String::new(),
        ui_text(ui_language, "settings_commands").to_string(),
        "/profile".to_string(),
        "/difficulty auto|easy|medium|hard".to_string(),
        "/topics arrays, strings".to_string(),
        "/avoid dp, graph".to_string(),
        "/generate-languages all|python, rust".to_string(),
        "/generate-ui all|en, ko".to_string(),
        "/provider codex|claude".to_string(),
        "/model auto".to_string(),
        "/effort auto|low|medium|high|xhigh|max".to_string(),
        "/note".to_string(),
        "/notes".to_string(),
    ]);
    lines.join("\n")
}

pub(super) fn apply_selected(
    state: &mut AppState,
    selected: usize,
    available_models: &[String],
) -> SettingsChange {
    let mut change = SettingsChange {
        reload_editor: false,
        provider_changed: false,
        edit_notes: false,
    };
    match selected {
        0 => {
            let current = LANGUAGES
                .iter()
                .position(|language| language == &state.settings.language)
                .unwrap_or(0);
            state.settings.language = LANGUAGES[(current + 1) % LANGUAGES.len()].to_string();
            change.reload_editor = true;
        }
        1 => {
            let current = UI_LANGUAGES
                .iter()
                .position(|language| language == &state.settings.ui_language)
                .unwrap_or(0);
            state.settings.ui_language =
                UI_LANGUAGES[(current + 1) % UI_LANGUAGES.len()].to_string();
        }
        2 => {
            let current = THEMES
                .iter()
                .position(|theme| theme == &state.settings.theme)
                .unwrap_or(0);
            state.settings.theme = THEMES[(current + 1) % THEMES.len()].to_string();
        }
        3 => {
            let current = DIFFICULTIES
                .iter()
                .position(|difficulty| difficulty == &state.settings.difficulty)
                .unwrap_or(0);
            let difficulty = DIFFICULTIES[(current + 1) % DIFFICULTIES.len()].to_string();
            state.settings.difficulty = difficulty.clone();
            if difficulty != "auto" {
                state.suggested_next_difficulty = difficulty;
            }
        }
        AI_PROVIDER_ROW => {
            let current = AI_PROVIDERS
                .iter()
                .position(|provider| provider == &state.settings.ai_provider)
                .unwrap_or(0);
            state.settings.ai_provider =
                AI_PROVIDERS[(current + 1) % AI_PROVIDERS.len()].to_string();
            state.settings.ai_model = "auto".to_string();
            state.settings.ai_effort =
                normalize_ai_effort(&state.settings.ai_provider, &state.settings.ai_effort);
            change.provider_changed = true;
        }
        AI_MODEL_ROW => cycle_ai_model(state, available_models),
        AI_EFFORT_ROW => cycle_ai_effort(state),
        NOTE_ROW => change.edit_notes = true,
        row if row < TOGGLE_START + LANGUAGES.len() => {
            toggle_generate_language(state, LANGUAGES[row - TOGGLE_START]);
        }
        row if row < row_count() => {
            toggle_generate_ui_language(state, UI_LANGUAGES[row - TOGGLE_START - LANGUAGES.len()]);
        }
        _ => {}
    }
    change
}

fn cycle_ai_model(state: &mut AppState, available_models: &[String]) {
    let mut models = vec!["auto"];
    models.extend(available_models.iter().map(String::as_str));
    let current = models
        .iter()
        .position(|model| model == &state.settings.ai_model)
        .unwrap_or(0);
    state.settings.ai_model = models[(current + 1) % models.len()].to_string();
}

fn cycle_ai_effort(state: &mut AppState) {
    let efforts = if state.settings.ai_provider == "claude" {
        CLAUDE_AI_EFFORTS
    } else {
        CODEX_AI_EFFORTS
    };
    let current = efforts
        .iter()
        .position(|effort| effort == &state.settings.ai_effort)
        .unwrap_or(0);
    state.settings.ai_effort = efforts[(current + 1) % efforts.len()].to_string();
}

fn row(cursor: Option<usize>, index: usize, text: &str) -> String {
    let marker = if cursor == Some(index) { ">" } else { " " };
    format!("{marker} {text}")
}

fn generate_language_enabled(state: &AppState, language: &str) -> bool {
    state.settings.generate_languages.is_empty()
        || state
            .settings
            .generate_languages
            .iter()
            .any(|value| value == language)
}

fn generate_ui_language_enabled(state: &AppState, language: &str) -> bool {
    state.settings.generate_ui_languages.is_empty()
        || state
            .settings
            .generate_ui_languages
            .iter()
            .any(|value| value == language)
}

fn toggle_generate_language(state: &mut AppState, language: &str) {
    if state.settings.generate_languages.is_empty() {
        state.settings.generate_languages = LANGUAGES
            .iter()
            .filter(|value| **value != language)
            .map(|value| (*value).to_string())
            .collect();
        return;
    }
    if generate_language_enabled(state, language) {
        if state.settings.generate_languages.len() > 1 {
            state
                .settings
                .generate_languages
                .retain(|value| value != language);
        }
    } else {
        state.settings.generate_languages.push(language.to_string());
        state.settings.generate_languages = LANGUAGES
            .iter()
            .filter(|value| {
                state
                    .settings
                    .generate_languages
                    .iter()
                    .any(|selected| selected == *value)
            })
            .map(|value| (*value).to_string())
            .collect();
        if state.settings.generate_languages.len() == LANGUAGES.len() {
            state.settings.generate_languages.clear();
        }
    }
}

fn toggle_generate_ui_language(state: &mut AppState, language: &str) {
    if state.settings.generate_ui_languages.is_empty() {
        state.settings.generate_ui_languages = UI_LANGUAGES
            .iter()
            .filter(|value| **value != language)
            .map(|value| (*value).to_string())
            .collect();
        return;
    }
    if generate_ui_language_enabled(state, language) {
        if state.settings.generate_ui_languages.len() > 1 {
            state
                .settings
                .generate_ui_languages
                .retain(|value| value != language);
        }
    } else {
        state
            .settings
            .generate_ui_languages
            .push(language.to_string());
        state.settings.generate_ui_languages = UI_LANGUAGES
            .iter()
            .filter(|value| {
                state
                    .settings
                    .generate_ui_languages
                    .iter()
                    .any(|selected| selected == *value)
            })
            .map(|value| (*value).to_string())
            .collect();
        if state.settings.generate_ui_languages.len() == UI_LANGUAGES.len() {
            state.settings.generate_ui_languages.clear();
        }
    }
}

fn list_or_none(values: &[String], ui_language: &str) -> String {
    if values.is_empty() {
        ui_text(ui_language, "settings_none").to_string()
    } else {
        values.join(", ")
    }
}

fn list_or_all(values: &[String], ui_language: &str) -> String {
    if values.is_empty() {
        ui_text(ui_language, "settings_all").to_string()
    } else {
        values.join(", ")
    }
}

fn checkbox(checked: bool) -> &'static str {
    if checked { "[x]" } else { "[ ]" }
}
