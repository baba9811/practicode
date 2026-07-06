use super::*;
use crate::process::which;
use std::process::Command;

#[derive(Clone, Copy, Eq, PartialEq)]
enum DoctorStatus {
    Ok,
    Missing,
    Update,
}

struct DoctorCheck {
    status: DoctorStatus,
    name: &'static str,
    detail: String,
    install: Option<&'static str>,
}

impl PracticodeApp {
    pub(super) fn action_doctor(&mut self) {
        let output = doctor_text(
            &self.state.settings.ui_language,
            &self.state.settings.language,
            &self.state.settings.ai_provider,
        );
        self.write_text_output(&output);
    }
}

fn doctor_text(lang: &str, current_language: &str, ai_provider: &str) -> String {
    doctor_text_with(
        lang,
        current_language,
        ai_provider,
        |name| which(name).is_some(),
        command_version,
    )
}

fn doctor_text_with<F, V>(
    lang: &str,
    current_language: &str,
    ai_provider: &str,
    mut has_command: F,
    mut command_version: V,
) -> String
where
    F: FnMut(&str) -> bool,
    V: FnMut(&str, &[&str]) -> Option<String>,
{
    let mut lines = vec![
        ui_text(lang, "doctor_title").to_string(),
        String::new(),
        format!(
            "{}: {}",
            ui_text(lang, "doctor_current_language"),
            syntax_language_name(&normalize_language(current_language))
        ),
        String::new(),
        ui_text(lang, "doctor_runtime_checks").to_string(),
    ];

    for check in runtime_checks(&mut has_command, &mut command_version) {
        push_check(lang, &mut lines, check);
    }

    lines.push(String::new());
    lines.push(ui_text(lang, "doctor_optional_ai").to_string());
    push_check(lang, &mut lines, ai_check(ai_provider, &mut has_command));

    lines.join("\n")
}

fn runtime_checks<F, V>(has_command: &mut F, command_version: &mut V) -> Vec<DoctorCheck>
where
    F: FnMut(&str) -> bool,
    V: FnMut(&str, &[&str]) -> Option<String>,
{
    vec![
        python_check(has_command),
        node_check(has_command, command_version),
        java_check(has_command),
        rust_check(has_command),
    ]
}

fn python_check<F>(has_command: &mut F) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
{
    let command = if has_command("python3") {
        Some("python3")
    } else if has_command("python") {
        Some("python")
    } else {
        None
    };
    DoctorCheck {
        status: if command.is_some() {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Missing
        },
        name: "Python",
        detail: command.unwrap_or("python3 or python").to_string(),
        install: command.is_none().then_some(PYTHON_INSTALL),
    }
}

fn node_check<F, V>(has_command: &mut F, command_version: &mut V) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
    V: FnMut(&str, &[&str]) -> Option<String>,
{
    if !has_command("node") {
        return DoctorCheck {
            status: DoctorStatus::Missing,
            name: "TypeScript",
            detail: "node >= 22.6.0".to_string(),
            install: Some(NODE_INSTALL),
        };
    }
    let version = command_version("node", &["--version"]).unwrap_or_else(|| "unknown".to_string());
    let ok = node_supports_strip_types(&version);
    DoctorCheck {
        status: if ok {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Update
        },
        name: "TypeScript",
        detail: if ok {
            format!("node {version}")
        } else {
            format!("Node.js >= 22.6.0 ({version})")
        },
        install: (!ok).then_some(NODE_INSTALL),
    }
}

fn java_check<F>(has_command: &mut F) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
{
    let has_javac = has_command("javac");
    let has_java = has_command("java");
    let missing = match (has_javac, has_java) {
        (true, true) => "javac + java",
        (false, true) => "missing javac",
        (true, false) => "missing java",
        (false, false) => "missing javac and java",
    };
    DoctorCheck {
        status: if has_javac && has_java {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Missing
        },
        name: "Java",
        detail: missing.to_string(),
        install: (!(has_javac && has_java)).then_some(JAVA_INSTALL),
    }
}

fn rust_check<F>(has_command: &mut F) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
{
    let ok = has_command("rustc");
    DoctorCheck {
        status: if ok {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Missing
        },
        name: "Rust",
        detail: if ok { "rustc" } else { "missing rustc" }.to_string(),
        install: (!ok).then_some(RUST_INSTALL),
    }
}

fn ai_check<F>(ai_provider: &str, has_command: &mut F) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
{
    let provider = normalize_ai_provider(ai_provider);
    let command = if provider == "claude" {
        "claude"
    } else {
        "codex"
    };
    let ok = has_command(command);
    DoctorCheck {
        status: if ok {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Missing
        },
        name: if provider == "claude" {
            "Claude Code"
        } else {
            "Codex"
        },
        detail: if ok {
            command.to_string()
        } else {
            format!("missing {command}")
        },
        install: (!ok).then_some(if provider == "claude" {
            CLAUDE_INSTALL
        } else {
            CODEX_INSTALL
        }),
    }
}

fn push_check(lang: &str, lines: &mut Vec<String>, check: DoctorCheck) {
    lines.push(format!(
        "{} {}: {}",
        status_label(lang, check.status),
        check.name,
        check.detail
    ));
    if let Some(install) = check.install {
        lines.push(format!("  {}:", ui_text(lang, "doctor_install")));
        lines.extend(install.lines().map(|line| format!("  {line}")));
    }
}

fn status_label(lang: &str, status: DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Ok => ui_text(lang, "doctor_ok"),
        DoctorStatus::Missing => ui_text(lang, "doctor_missing"),
        DoctorStatus::Update => ui_text(lang, "doctor_update"),
    }
}

fn command_version(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|output| !output.is_empty())
}

fn node_supports_strip_types(version: &str) -> bool {
    version_at_least(version.trim_start_matches('v'), 22, 6, 0)
}

fn version_at_least(version: &str, major: u64, minor: u64, patch: u64) -> bool {
    let mut parts = version
        .split(['.', '-'])
        .take(3)
        .map(|part| part.parse::<u64>().unwrap_or(0));
    let found = (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    );
    found >= (major, minor, patch)
}

const PYTHON_INSTALL: &str = "macOS: brew install python\nWindows: winget install -e --id Python.Python.3.12\nUbuntu/Debian: sudo apt install -y python3";
const NODE_INSTALL: &str = "macOS: brew install node\nWindows: winget install -e --id OpenJS.NodeJS.LTS\nUbuntu/Debian: install Node.js LTS from https://nodejs.org/en/download";
const JAVA_INSTALL: &str = "macOS: brew install --cask temurin@21\nWindows: winget install -e --id EclipseAdoptium.Temurin.21.JDK\nUbuntu/Debian: sudo apt install -y openjdk-21-jdk";
const RUST_INSTALL: &str = "macOS/Linux: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\nWindows: winget install -e --id Rustlang.Rustup";
const CODEX_INSTALL: &str = "Install Codex CLI, or switch with /provider claude.";
const CLAUDE_INSTALL: &str = "Install Claude Code, or switch with /provider codex.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_output_includes_install_help_for_missing_runtimes() {
        let output = doctor_text_with("en", "python", "codex", |_| false, |_, _| None);

        assert!(output.contains("Doctor"));
        assert!(output.contains("Runtime checks"));
        assert!(output.contains("MISSING Python"));
        assert!(output.contains("Install"));
        assert!(output.contains("brew install python"));
        assert!(output.contains("brew install node"));
        assert!(output.contains("winget install -e --id Python.Python.3.12"));
    }

    #[test]
    fn doctor_marks_old_node_as_update_needed() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "codex"),
            |name, _| (name == "node").then(|| "v22.5.0".to_string()),
        );

        assert!(output.contains("UPDATE TypeScript"));
        assert!(output.contains("Node.js >= 22.6.0"));
    }
}
