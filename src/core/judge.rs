use super::*;
use std::env;

const TYPESCRIPT_TYPECHECK_FLAGS: [&str; 8] = [
    "--noEmit",
    "--strict",
    "--target",
    "ES2022",
    "--module",
    "nodenext",
    "--moduleResolution",
    "nodenext",
];
const JAVA_RELEASE_FLAGS: [&str; 2] = ["--release", "21"];
const RUST_EDITION_FLAG: &str = "--edition=2024";

#[derive(Debug)]
struct CommandFailure {
    kind: JudgeFailureKind,
    detail: String,
}

impl std::fmt::Display for CommandFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for CommandFailure {}

pub fn normalize_judge_output(output: &str) -> String {
    output.replace("\r\n", "\n")
}

pub fn ensure_submission(root: &Path, problem: &Problem, settings: &Settings) -> Result<PathBuf> {
    let language = normalize_language(&settings.language);
    let path = root
        .join("submissions")
        .join(&problem.id)
        .join(format!("solution.{}", ext_for(&language)));
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, template_for(&language))?;
    }
    Ok(path)
}
pub fn judge(root: &Path, problem: &Problem, settings: &Settings) -> JudgeResult {
    if problem.cases.is_empty() {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: 0,
            failure_kind: None,
            output: "problem has no judge cases".to_string(),
        };
    }
    let path = match ensure_submission(root, problem, settings) {
        Ok(path) => path,
        Err(error) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: problem.cases.len(),
                failure_kind: None,
                output: error.to_string(),
            };
        }
    };
    let language = normalize_language(&settings.language);
    judge_path(root, &problem.id, &path, &language, &problem.cases)
}

pub fn judge_path(
    root: &Path,
    id: &str,
    path: &Path,
    language: &str,
    cases: &[IoCase],
) -> JudgeResult {
    if cases.is_empty() {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: 0,
            failure_kind: None,
            output: "problem has no judge cases".to_string(),
        };
    }
    let language = normalize_language(language);
    let command = match command_for(root, path, &language) {
        Ok(Some(command)) => command,
        Ok(None) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: cases.len(),
                failure_kind: None,
                output: format!("Missing runtime for {language}"),
            };
        }
        Err(error) => {
            let failure_kind = error
                .downcast_ref::<CommandFailure>()
                .map(|failure| failure.kind)
                .unwrap_or(if language == "ts" {
                    JudgeFailureKind::TypeCheck
                } else {
                    JudgeFailureKind::Compile
                });
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: cases.len(),
                failure_kind: Some(failure_kind),
                output: error.to_string(),
            };
        }
    };
    let run_dir = root.join("build").join(id).join("run");
    if let Err(error) = fs::create_dir_all(&run_dir) {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: cases.len(),
            failure_kind: None,
            output: error.to_string(),
        };
    }

    let mut passed = 0;
    let mut lines = Vec::new();
    let mut failure_kind = None;
    for (index, case) in cases.iter().enumerate() {
        let mut process = Command::new(&command.program);
        process.args(&command.args).current_dir(&run_dir);
        apply_judge_env(&mut process);
        let run = match run_capture(&mut process, &case.input, Duration::from_secs(5)) {
            Ok(run) => run,
            Err(error) => {
                failure_kind = Some(JudgeFailureKind::Runtime);
                lines.push(format!("Case {}: FAIL", index + 1));
                push_labeled_block(&mut lines, "Error", &error.to_string());
                break;
            }
        };
        let got = normalize_judge_output(&run.stdout);
        let expected = normalize_judge_output(&case.output);
        if !run.timed_out && run.code == Some(0) && got == expected {
            passed += 1;
            lines.push(format!("Case {}: PASS", index + 1));
            if !run.stdout.trim().is_empty() {
                push_labeled_block(&mut lines, "Stdout", run.stdout.trim_end());
            }
            if !run.stderr.trim().is_empty() {
                push_labeled_block(&mut lines, "Stderr", run.stderr.trim_end());
            }
        } else {
            failure_kind = Some(if run.timed_out {
                JudgeFailureKind::Timeout
            } else if run.code != Some(0) {
                JudgeFailureKind::Runtime
            } else {
                JudgeFailureKind::Output
            });
            lines.push(format!("Case {}: FAIL", index + 1));
            if run.timed_out {
                push_labeled_block(&mut lines, "Error", "timeout: 5s");
            } else if run.code != Some(0) {
                push_labeled_block(
                    &mut lines,
                    "Error",
                    &format!(
                        "process exited with status {}",
                        run.code
                            .map(|code| code.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ),
                );
            }
            push_labeled_block(&mut lines, "Got", &got);
            push_labeled_block(&mut lines, "Input", "<hidden>");
            push_labeled_block(&mut lines, "Expected", "<hidden>");
            if !run.stderr.trim().is_empty() {
                push_labeled_block(&mut lines, "Stderr", run.stderr.trim_end());
            }
            break;
        }
    }

    JudgeResult {
        passed: passed == cases.len(),
        passed_cases: passed,
        total_cases: cases.len(),
        failure_kind,
        output: lines.join("\n"),
    }
}

fn push_labeled_block(lines: &mut Vec<String>, label: &str, body: &str) {
    lines.push(String::new());
    lines.push(label.to_string());
    if body.is_empty() {
        lines.push("  <empty>".to_string());
    } else {
        lines.extend(body.lines().map(|line| format!("  {line}")));
    }
}

fn apply_judge_env(process: &mut Command) {
    process.env_clear();
    for key in [
        "PATH",
        "HOME",
        "USER",
        "USERNAME",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TMPDIR",
        "TEMP",
        "TMP",
        "PYENV_ROOT",
        "ASDF_DIR",
        "ASDF_DATA_DIR",
        "NVM_DIR",
        "VOLTA_HOME",
    ] {
        if let Some(value) = env::var_os(key) {
            process.env(key, value);
        }
    }
    #[cfg(windows)]
    for key in ["COMSPEC", "PATHEXT", "SYSTEMROOT", "WINDIR"] {
        if let Some(value) = env::var_os(key) {
            process.env(key, value);
        }
    }
}

pub fn command_for(root: &Path, path: &Path, language: &str) -> Result<Option<CommandSpec>> {
    match language {
        "python" => Ok(which("python3")
            .or_else(|| which("python"))
            .map(|program| CommandSpec {
                program,
                args: vec![path.display().to_string()],
            })),
        "ts" => compile_typescript(root, path),
        "java" => compile_java(root, path),
        "rust" => compile_rust(root, path),
        _ => Ok(None),
    }
}

fn compile_typescript(root: &Path, path: &Path) -> Result<Option<CommandSpec>> {
    let Some(tsc) = which("tsc") else {
        return Err(missing_typescript_tool_failure("tsc").into());
    };
    let Some(node) = which("node") else {
        return Err(missing_typescript_tool_failure("node").into());
    };
    let build = root
        .join("build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default())
        .join("typescript");
    fs::create_dir_all(&build)?;
    let shim = build.join("node-shim.d.ts");
    fs::write(
        &shim,
        include_str!("../../assets/typescript/node-shim.d.ts"),
    )?;
    let mut compile = Command::new(tsc);
    compile
        .args(TYPESCRIPT_TYPECHECK_FLAGS)
        .args([&shim.display().to_string(), &path.display().to_string()])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(compiler_failure(JudgeFailureKind::TypeCheck, &output).into());
    }
    Ok(Some(CommandSpec {
        program: node,
        args: vec![
            "--experimental-strip-types".to_string(),
            path.display().to_string(),
        ],
    }))
}

fn compiler_detail(output: &crate::process::RunOutput) -> String {
    let stdout = normalize_judge_output(&output.stdout);
    let stderr = normalize_judge_output(&output.stderr);
    let detail = match (stdout.trim_end(), stderr.trim_end()) {
        ("", "") => String::new(),
        ("", stderr) => stderr.to_string(),
        (stdout, "") => stdout.to_string(),
        (stdout, stderr) => format!("{stdout}\n{stderr}"),
    };
    if output.timed_out {
        if detail.is_empty() {
            "timeout: 30s".to_string()
        } else {
            format!("timeout: 30s\n{detail}")
        }
    } else if detail.is_empty() {
        "compiler exited without a diagnostic".to_string()
    } else {
        detail
    }
}

fn compiler_failure(kind: JudgeFailureKind, output: &crate::process::RunOutput) -> CommandFailure {
    CommandFailure {
        kind: if output.timed_out {
            JudgeFailureKind::Timeout
        } else {
            kind
        },
        detail: compiler_detail(output),
    }
}

fn missing_typescript_tool_failure(tool: &str) -> CommandFailure {
    CommandFailure {
        kind: if tool == "tsc" {
            JudgeFailureKind::TypeCheck
        } else {
            JudgeFailureKind::Runtime
        },
        detail: format!("Missing runtime for TypeScript: {tool}"),
    }
}

fn compile_java(root: &Path, path: &Path) -> Result<Option<CommandSpec>> {
    let Some(javac) = which("javac") else {
        return Ok(None);
    };
    let Some(java) = which("java") else {
        return Ok(None);
    };
    let build = root
        .join("build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default())
        .join("java");
    fs::create_dir_all(&build)?;
    let mut compile = Command::new(javac);
    compile
        .args(JAVA_RELEASE_FLAGS)
        .args([
            "-d",
            &build.display().to_string(),
            &path.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(compiler_failure(JudgeFailureKind::Compile, &output).into());
    }
    Ok(Some(CommandSpec {
        program: java,
        args: vec![
            "-cp".to_string(),
            build.display().to_string(),
            "Solution".to_string(),
        ],
    }))
}

fn compile_rust(root: &Path, path: &Path) -> Result<Option<CommandSpec>> {
    let Some(rustc) = which("rustc") else {
        return Ok(None);
    };
    let build = root
        .join("build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default());
    fs::create_dir_all(&build)?;
    let exe = build.join(if cfg!(windows) {
        "solution.exe"
    } else {
        "solution"
    });
    let mut compile = Command::new(rustc);
    compile
        .arg(RUST_EDITION_FLAG)
        .args([
            path.display().to_string(),
            "-o".to_string(),
            exe.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(compiler_failure(JudgeFailureKind::Compile, &output).into());
    }
    Ok(Some(CommandSpec {
        program: exe,
        args: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_contract_flags_are_explicit() {
        assert_eq!(
            TYPESCRIPT_TYPECHECK_FLAGS,
            [
                "--noEmit",
                "--strict",
                "--target",
                "ES2022",
                "--module",
                "nodenext",
                "--moduleResolution",
                "nodenext",
            ]
        );
        assert_eq!(JAVA_RELEASE_FLAGS, ["--release", "21"]);
        assert_eq!(RUST_EDITION_FLAG, "--edition=2024");
    }

    #[test]
    fn compiler_timeout_takes_precedence_over_compile_kind() {
        let output = crate::process::RunOutput {
            code: None,
            stdout: String::new(),
            stderr: "partial diagnostic\n".to_string(),
            timed_out: true,
        };
        let failure = compiler_failure(JudgeFailureKind::Compile, &output);

        assert_eq!(failure.kind, JudgeFailureKind::Timeout);
        assert!(failure.detail.starts_with("timeout: 30s\n"));
        assert!(failure.detail.contains("partial diagnostic"));
    }

    #[test]
    fn missing_typescript_tools_are_classified_by_stage() {
        assert_eq!(
            missing_typescript_tool_failure("tsc").kind,
            JudgeFailureKind::TypeCheck
        );
        assert_eq!(
            missing_typescript_tool_failure("node").kind,
            JudgeFailureKind::Runtime
        );
    }
}
