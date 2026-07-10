use super::*;

pub const LANGUAGES: &[&str] = &["python", "ts", "java", "rust"];
pub const THEMES: &[&str] = &["dark", "light"];
pub const AI_PROVIDERS: &[&str] = &["codex", "claude"];
pub const CODEX_AI_EFFORTS: &[&str] = &["auto", "low", "medium", "high", "xhigh"];
pub const CLAUDE_AI_EFFORTS: &[&str] = &["auto", "low", "medium", "high", "xhigh", "max"];
pub const BANK_PATH: &str = "problem_bank.json";
pub const STATE_PATH: &str = "problem-state.json";
pub const PROBLEM_NOTES_PATH: &str = "problem_notes.md";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_start_mode")]
    pub start_mode: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avoid_topics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generate_languages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generate_ui_languages: Vec<String>,
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default = "default_next_source")]
    pub next_source: String,
    #[serde(default = "default_ai_provider")]
    pub ai_provider: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "default_ai_effort")]
    pub ai_effort: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ai_next_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: default_language(),
            ui_language: default_ui_language(),
            theme: default_theme(),
            start_mode: default_start_mode(),
            difficulty: default_difficulty(),
            topics: Vec::new(),
            avoid_topics: Vec::new(),
            generate_languages: Vec::new(),
            generate_ui_languages: Vec::new(),
            editor: default_editor(),
            next_source: default_next_source(),
            ai_provider: default_ai_provider(),
            ai_model: default_ai_model(),
            ai_effort: default_ai_effort(),
            ai_next_command: String::new(),
        }
    }
}

impl Settings {
    pub fn next_ai_command(&self) -> &str {
        &self.ai_next_command
    }

    pub fn model_arg(&self) -> Option<&str> {
        let model = self.ai_model.trim();
        if model.is_empty() || model == "auto" {
            None
        } else {
            Some(model)
        }
    }

    pub fn effort_arg(&self) -> Option<&str> {
        let effort = self.ai_effort.trim();
        if effort.is_empty() || effort == "auto" {
            None
        } else {
            Some(effort)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub current_problem: String,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub solved: Vec<String>,
    #[serde(default)]
    pub history: Vec<HistoryItem>,
    #[serde(default = "default_suggested_difficulty")]
    pub suggested_next_difficulty: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub syntax_progress: HashMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub current_syntax_lesson: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub slug: String,
    pub difficulty: String,
    pub topics: Vec<String>,
    pub title: HashMap<String, String>,
    pub statement: HashMap<String, String>,
    pub input: HashMap<String, String>,
    pub output: HashMap<String, String>,
    pub examples: Vec<IoCase>,
    pub cases: Vec<IoCase>,
    pub answers: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IoCase {
    pub input: String,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct JudgeResult {
    pub passed: bool,
    pub passed_cases: usize,
    pub total_cases: usize,
    pub output: String,
}

pub fn default_language() -> String {
    "python".to_string()
}

pub fn default_ui_language() -> String {
    "en".to_string()
}

pub fn default_theme() -> String {
    "dark".to_string()
}

pub fn default_start_mode() -> String {
    "home".to_string()
}

pub fn default_editor() -> String {
    "vim".to_string()
}

pub fn default_next_source() -> String {
    "bank".to_string()
}

pub fn default_ai_provider() -> String {
    "codex".to_string()
}

pub fn default_ai_model() -> String {
    "auto".to_string()
}

pub fn default_ai_effort() -> String {
    "auto".to_string()
}

pub fn default_suggested_difficulty() -> String {
    "easy".to_string()
}
