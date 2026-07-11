use super::*;
use crate::{
    core::typescript_version_is_supported,
    process::{run_capture, which},
};
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
    install: Option<InstallHelp>,
}

#[derive(Clone, Copy)]
enum InstallHelp {
    Python,
    Node,
    NodeAndTypeScript,
    TypeScript,
    Java,
    Rust,
    Codex,
    Claude,
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

    for check in runtime_checks(lang, &mut has_command, &mut command_version) {
        push_check(lang, &mut lines, check);
    }

    lines.push(String::new());
    lines.push(ui_text(lang, "doctor_optional_ai").to_string());
    push_check(
        lang,
        &mut lines,
        ai_check(lang, ai_provider, &mut has_command),
    );

    lines.join("\n")
}

fn runtime_checks<F, V>(
    lang: &str,
    has_command: &mut F,
    command_version: &mut V,
) -> Vec<DoctorCheck>
where
    F: FnMut(&str) -> bool,
    V: FnMut(&str, &[&str]) -> Option<String>,
{
    vec![
        python_check(lang, has_command),
        node_check(lang, has_command, command_version),
        java_check(lang, has_command),
        rust_check(lang, has_command),
    ]
}

fn python_check<F>(lang: &str, has_command: &mut F) -> DoctorCheck
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
        detail: command.map(str::to_string).unwrap_or_else(|| {
            format!(
                "{}; {}",
                missing_tool(lang, "python3"),
                missing_tool(lang, "python")
            )
        }),
        install: command.is_none().then_some(InstallHelp::Python),
    }
}

fn node_check<F, V>(lang: &str, has_command: &mut F, command_version: &mut V) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
    V: FnMut(&str, &[&str]) -> Option<String>,
{
    let has_node = has_command("node");
    let has_tsc = has_command("tsc");
    if !has_tsc {
        let mut detail = missing_tool(lang, "tsc");
        if !has_node {
            detail.push_str(&format!("; {}", ui_text(lang, "doctor_node_required")));
        }
        return DoctorCheck {
            status: DoctorStatus::Missing,
            name: "TypeScript",
            detail,
            install: Some(if has_node {
                InstallHelp::TypeScript
            } else {
                InstallHelp::NodeAndTypeScript
            }),
        };
    }
    let Some(tsc_version) = command_version("tsc", &["--version"]) else {
        let mut detail = ui_text(lang, "doctor_tsc_unreadable").to_string();
        if !has_node {
            detail.push_str(&format!("; {}", ui_text(lang, "doctor_node_required")));
        }
        return DoctorCheck {
            status: DoctorStatus::Update,
            name: "TypeScript",
            detail,
            install: Some(if has_node {
                InstallHelp::TypeScript
            } else {
                InstallHelp::NodeAndTypeScript
            }),
        };
    };
    if !typescript_version_is_supported(&tsc_version) {
        let mut detail = ui_text(lang, "doctor_tsc_required").replace("{version}", &tsc_version);
        if !has_node {
            detail.push_str(&format!("; {}", ui_text(lang, "doctor_node_required")));
        }
        return DoctorCheck {
            status: DoctorStatus::Update,
            name: "TypeScript",
            detail,
            install: Some(if has_node {
                InstallHelp::TypeScript
            } else {
                InstallHelp::NodeAndTypeScript
            }),
        };
    }
    if !has_node {
        return DoctorCheck {
            status: DoctorStatus::Missing,
            name: "TypeScript",
            detail: format!(
                "{}; tsc {tsc_version}",
                ui_text(lang, "doctor_node_required")
            ),
            install: Some(InstallHelp::Node),
        };
    }
    let version = command_version("node", &["--version"])
        .unwrap_or_else(|| ui_text(lang, "doctor_unknown_version").to_string());
    let ok = node_supports_strip_types(&version);
    DoctorCheck {
        status: if ok {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Update
        },
        name: "TypeScript",
        detail: if ok {
            format!("node {version} + tsc {tsc_version}")
        } else {
            format!(
                "{} ({version}); tsc {tsc_version}",
                ui_text(lang, "doctor_node_required")
            )
        },
        install: (!ok).then_some(InstallHelp::Node),
    }
}

fn java_check<F>(lang: &str, has_command: &mut F) -> DoctorCheck
where
    F: FnMut(&str) -> bool,
{
    let has_javac = has_command("javac");
    let has_java = has_command("java");
    let detail = match (has_javac, has_java) {
        (true, true) => "javac + java".to_string(),
        (false, true) => missing_tool(lang, "javac"),
        (true, false) => missing_tool(lang, "java"),
        (false, false) => format!(
            "{}; {}",
            missing_tool(lang, "javac"),
            missing_tool(lang, "java")
        ),
    };
    DoctorCheck {
        status: if has_javac && has_java {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Missing
        },
        name: "Java",
        detail,
        install: (!(has_javac && has_java)).then_some(InstallHelp::Java),
    }
}

fn rust_check<F>(lang: &str, has_command: &mut F) -> DoctorCheck
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
        detail: if ok {
            "rustc".to_string()
        } else {
            missing_tool(lang, "rustc")
        },
        install: (!ok).then_some(InstallHelp::Rust),
    }
}

fn ai_check<F>(lang: &str, ai_provider: &str, has_command: &mut F) -> DoctorCheck
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
            missing_tool(lang, command)
        },
        install: (!ok).then_some(if provider == "claude" {
            InstallHelp::Claude
        } else {
            InstallHelp::Codex
        }),
    }
}

fn missing_tool(lang: &str, tool: &str) -> String {
    ui_text(lang, "doctor_missing_tool").replace("{tool}", tool)
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
        lines.extend(
            install_lines(lang, install)
                .into_iter()
                .map(|line| format!("  {line}")),
        );
    }
}

fn install_lines(lang: &str, install: InstallHelp) -> Vec<String> {
    match install {
        InstallHelp::Python => PYTHON_INSTALL.lines().map(str::to_string).collect(),
        InstallHelp::Node | InstallHelp::NodeAndTypeScript => {
            let mut lines = NODE_INSTALL.lines().map(str::to_string).collect::<Vec<_>>();
            lines.push(
                ui_text(lang, "doctor_node_install_linux")
                    .replace("{url}", "https://nodejs.org/en/download"),
            );
            if matches!(install, InstallHelp::NodeAndTypeScript) {
                lines.push(TYPESCRIPT_INSTALL.to_string());
            }
            lines
        }
        InstallHelp::TypeScript => vec![TYPESCRIPT_INSTALL.to_string()],
        InstallHelp::Java => JAVA_INSTALL.lines().map(str::to_string).collect(),
        InstallHelp::Rust => RUST_INSTALL.lines().map(str::to_string).collect(),
        InstallHelp::Codex => vec![ui_text(lang, "doctor_codex_install").to_string()],
        InstallHelp::Claude => vec![ui_text(lang, "doctor_claude_install").to_string()],
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
    let mut command = Command::new(program);
    command.args(args);
    let output = run_capture(&mut command, "", Duration::from_secs(5)).ok()?;
    if output.timed_out || output.code != Some(0) {
        return None;
    }
    let output = output.stdout.trim().to_string();
    (!output.is_empty()).then_some(output)
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
const NODE_INSTALL: &str =
    "macOS: brew install node\nWindows: winget install -e --id OpenJS.NodeJS.LTS";
const TYPESCRIPT_INSTALL: &str = "npm install -g typescript@5.9";
const JAVA_INSTALL: &str = "macOS: brew install --cask temurin@21\nWindows: winget install -e --id EclipseAdoptium.Temurin.21.JDK\nUbuntu/Debian: sudo apt install -y openjdk-21-jdk";
const RUST_INSTALL: &str = "macOS/Linux: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\nWindows: winget install -e --id Rustlang.Rustup";

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
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| match name {
                "node" => Some("v22.5.0".to_string()),
                "tsc" => Some("Version 5.9.3".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("UPDATE TypeScript"));
        assert!(output.contains("node >= 22.6.0 required"));
    }

    #[test]
    fn doctor_reports_missing_tsc_when_node_is_ready() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "codex"),
            |name, _| (name == "node").then(|| "v22.6.0".to_string()),
        );

        assert!(output.contains("MISSING TypeScript"));
        assert!(output.contains("missing tsc"));
    }

    #[test]
    fn doctor_reports_missing_node_when_tsc_is_ready() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "tsc" | "codex"),
            |name, _| (name == "tsc").then(|| "Version 5.9.3".to_string()),
        );

        assert!(output.contains("MISSING TypeScript"));
        assert!(output.contains("node >= 22.6.0 required"));
        assert!(!output.contains("missing tsc"));
    }

    #[test]
    fn doctor_accepts_typescript_5_9_when_node_is_ready() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| match name {
                "node" => Some("v22.6.0".to_string()),
                "tsc" => Some("Version 5.9.3".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("OK TypeScript"));
        assert!(output.contains("tsc Version 5.9.3"));
    }

    #[test]
    fn doctor_rejects_old_typescript() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| match name {
                "node" => Some("v22.6.0".to_string()),
                "tsc" => Some("Version 5.8.4".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("UPDATE TypeScript"));
        assert!(output.contains("TypeScript 5.9.x required"));
        assert!(output.contains("Version 5.8.4"));
    }

    #[test]
    fn doctor_rejects_future_typescript_major() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| match name {
                "node" => Some("v22.6.0".to_string()),
                "tsc" => Some("Version 6.0.0".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("UPDATE TypeScript"));
        assert!(output.contains("TypeScript 5.9.x required"));
        assert!(output.contains("Version 6.0.0"));
    }

    #[test]
    fn doctor_rejects_unreadable_typescript_version() {
        let output = doctor_text_with(
            "en",
            "ts",
            "codex",
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| (name == "node").then(|| "v22.6.0".to_string()),
        );

        assert!(output.contains("UPDATE TypeScript"));
        assert!(output.contains("unreadable tsc version"));
    }

    #[test]
    fn doctor_missing_guidance_localizes_prose_and_preserves_install_commands() {
        let output = doctor_text_with("ja", "python", "codex", |_| false, |_, _| None);

        for expected in [
            "python3がありません",
            "tscがありません",
            "node >= 22.6.0が必要",
            "javacがありません",
            "rustcがありません",
            "Codex CLIをインストール",
            "Ubuntu/Debian: https://nodejs.org/en/downloadからNode.js LTSをダウンロード",
        ] {
            assert!(output.contains(expected), "{expected}: {output}");
        }
        for command in [
            "brew install python",
            "winget install -e --id OpenJS.NodeJS.LTS",
            "npm install -g typescript@5.9",
            "sudo apt install -y openjdk-21-jdk",
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            "/provider claude",
        ] {
            assert!(output.contains(command), "{command}: {output}");
        }
        assert!(!output.contains("missing "), "{output}");
        assert!(!output.contains("Install Codex"), "{output}");
    }

    #[test]
    fn doctor_update_guidance_localizes_prose_and_preserves_raw_versions() {
        let output = doctor_text_with(
            "es",
            "ts",
            "codex",
            |name| matches!(name, "node" | "tsc" | "codex"),
            |name, _| match name {
                "node" => Some("v22.5.0 raw".to_string()),
                "tsc" => Some("Version 5.8.4 raw".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("se requiere TypeScript 5.9.x"), "{output}");
        assert!(output.contains("Version 5.8.4 raw"), "{output}");
        assert!(!output.contains("TypeScript 5.9.x required"), "{output}");
    }
}
