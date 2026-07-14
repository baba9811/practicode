use super::*;
use std::{env, ffi::OsString};

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
const MAX_COMPILER_DIAGNOSTIC_LINES: usize = 12;
const MAX_COMPILER_DIAGNOSTIC_BYTES: usize = 4_096;
const DIAGNOSTIC_TRUNCATED_MARKER: &str = "[diagnostic truncated]";

#[derive(Clone, Debug)]
struct PreparedCommand {
    program: PathBuf,
    args: Vec<OsString>,
}

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
    if !regular_file_exists(&path)? {
        if let Some(parent) = path.parent() {
            create_dir_all_beneath(root, parent)?;
        }
        save_user_text(&path, &template_for(&language))?;
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
    judge_path_with(root, id, path, language, cases, command_for_os)
}

fn judge_path_with<F>(
    root: &Path,
    id: &str,
    path: &Path,
    language: &str,
    cases: &[IoCase],
    mut prepare: F,
) -> JudgeResult
where
    F: FnMut(&Path, &Path, &str) -> Result<Option<PreparedCommand>>,
{
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
    let command = match prepare(root, path, &language) {
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
    if let Err(error) = create_dir_all_beneath(root, &run_dir) {
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
    if body.is_empty() {
        lines.push(format!("{label}: <empty>"));
    } else {
        lines.push(label.to_string());
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
    command_for_os(root, path, language)?
        .map(|command| {
            let args = command
                .args
                .into_iter()
                .map(|arg| {
                    arg.into_string()
                        .map_err(|arg| anyhow!("command argument {arg:?} is not valid UTF-8"))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(CommandSpec {
                program: command.program,
                args,
            })
        })
        .transpose()
}

fn command_for_os(root: &Path, path: &Path, language: &str) -> Result<Option<PreparedCommand>> {
    match language {
        "python" => {
            Ok(which("python3")
                .or_else(|| which("python"))
                .map(|program| PreparedCommand {
                    program,
                    args: vec![path.as_os_str().to_owned()],
                }))
        }
        "ts" => compile_typescript(root, path),
        "java" => compile_java(root, path),
        "rust" => compile_rust(root, path),
        _ => Ok(None),
    }
}

fn compile_typescript(root: &Path, path: &Path) -> Result<Option<PreparedCommand>> {
    compile_typescript_with(root, path, which, |program, args| {
        let mut command = Command::new(program);
        command.args(args).current_dir(root);
        run_capture(&mut command, "", Duration::from_secs(30))
    })
}

fn compile_typescript_with<F, R>(
    root: &Path,
    path: &Path,
    mut find: F,
    mut run: R,
) -> Result<Option<PreparedCommand>>
where
    F: FnMut(&str) -> Option<PathBuf>,
    R: FnMut(&Path, &[OsString]) -> Result<crate::process::RunOutput>,
{
    let Some(tsc) = find("tsc") else {
        return Err(missing_typescript_tool_failure("tsc").into());
    };
    let version = run(&tsc, &[OsString::from("--version")])?;
    validate_typescript_version(&version)?;

    let build = root
        .join("build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default())
        .join("typescript");
    create_dir_all_beneath(root, &build)?;
    let shim = build.join("node-shim.d.ts");
    let type_root = build.join("type-roots");
    match fs::symlink_metadata(&type_root) {
        Ok(metadata) if metadata.file_type().is_dir() => fs::remove_dir_all(&type_root)?,
        Ok(_) => bail!("{} is not a regular directory", type_root.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    create_dir_all_beneath(root, &type_root)?;
    regular_file_exists(&shim)?;
    fs::write(
        &shim,
        include_str!("../../assets/typescript/node-shim.d.ts"),
    )?;
    let mut args = TYPESCRIPT_TYPECHECK_FLAGS.map(OsString::from).to_vec();
    args.extend([
        OsString::from("--typeRoots"),
        type_root.as_os_str().to_owned(),
        shim.as_os_str().to_owned(),
        path.as_os_str().to_owned(),
    ]);
    let output = run(&tsc, &args)?;
    if compiler_gate_failed(&output) {
        return Err(compiler_failure(JudgeFailureKind::TypeCheck, &output).into());
    }
    let Some(node) = find("node") else {
        return Err(missing_typescript_tool_failure("node").into());
    };
    Ok(Some(PreparedCommand {
        program: node,
        args: vec![
            OsString::from("--experimental-strip-types"),
            path.as_os_str().to_owned(),
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
    let detail = if output.timed_out {
        if detail.is_empty() {
            "timeout: 30s".to_string()
        } else {
            format!("timeout: 30s\n{detail}")
        }
    } else if detail.is_empty() {
        "compiler exited without a diagnostic".to_string()
    } else {
        detail
    };
    bound_compiler_detail(&detail)
}

fn bound_compiler_detail(detail: &str) -> String {
    let mut lines = 1;
    let mut end = detail.len();
    for (index, char) in detail.char_indices() {
        if index + char.len_utf8() > MAX_COMPILER_DIAGNOSTIC_BYTES
            || (char == '\n' && lines == MAX_COMPILER_DIAGNOSTIC_LINES)
        {
            end = index;
            break;
        }
        if char == '\n' {
            lines += 1;
        }
    }
    if end == detail.len() {
        detail.to_string()
    } else {
        format!(
            "{}\n{DIAGNOSTIC_TRUNCATED_MARKER}",
            detail[..end].trim_end()
        )
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

fn compiler_gate_failed(output: &crate::process::RunOutput) -> bool {
    output.timed_out || output.code != Some(0)
}

pub(crate) fn typescript_version_is_supported(output: &str) -> bool {
    let Some(version) = output.trim().strip_prefix("Version ") else {
        return false;
    };
    let mut parts = version.split('.');
    matches!(
        (
            parts.next().and_then(|part| part.parse::<u64>().ok()),
            parts.next().and_then(|part| part.parse::<u64>().ok()),
            parts.next().and_then(|part| part.parse::<u64>().ok()),
            parts.next(),
        ),
        (Some(5), Some(9), Some(_), None)
    )
}

fn validate_typescript_version(
    output: &crate::process::RunOutput,
) -> std::result::Result<(), CommandFailure> {
    if compiler_gate_failed(output) {
        return Err(compiler_failure(JudgeFailureKind::TypeCheck, output));
    }
    if typescript_version_is_supported(&output.stdout) {
        return Ok(());
    }
    let reported = output.stdout.trim();
    let detail = format!(
        "TypeScript 5.9.x required; found {}",
        if reported.is_empty() {
            "unreadable version output"
        } else {
            reported
        }
    );
    Err(CommandFailure {
        kind: JudgeFailureKind::TypeCheck,
        detail: bound_compiler_detail(&detail),
    })
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

fn compile_java(root: &Path, path: &Path) -> Result<Option<PreparedCommand>> {
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
    let build_parent = build.parent().context("find Java build parent")?;
    create_dir_all_beneath(root, build_parent)?;
    match fs::symlink_metadata(&build) {
        Ok(metadata) if metadata.file_type().is_dir() => fs::remove_dir_all(&build)?,
        Ok(_) => bail!("{} is not a regular directory", build.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    create_dir_all_beneath(root, &build)?;
    let mut compile = Command::new(javac);
    compile
        .args(JAVA_RELEASE_FLAGS)
        .arg("-d")
        .arg(&build)
        .arg(path)
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if compiler_gate_failed(&output) {
        return Err(compiler_failure(JudgeFailureKind::Compile, &output).into());
    }
    Ok(Some(PreparedCommand {
        program: java,
        args: vec![
            OsString::from("-cp"),
            build.as_os_str().to_owned(),
            OsString::from("Solution"),
        ],
    }))
}

fn compile_rust(root: &Path, path: &Path) -> Result<Option<PreparedCommand>> {
    let Some(rustc) = which("rustc") else {
        return Ok(None);
    };
    let build = root
        .join("build")
        .join(path.parent().and_then(Path::file_name).unwrap_or_default());
    create_dir_all_beneath(root, &build)?;
    let exe = build.join(if cfg!(windows) {
        "solution.exe"
    } else {
        "solution"
    });
    regular_file_exists(&exe)?;
    let mut compile = Command::new(rustc);
    compile
        .arg(RUST_EDITION_FLAG)
        .arg(path)
        .arg("-o")
        .arg(&exe)
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if compiler_gate_failed(&output) {
        return Err(compiler_failure(JudgeFailureKind::Compile, &output).into());
    }
    Ok(Some(PreparedCommand {
        program: exe,
        args: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labeled_blocks_distinguish_semantic_empty_from_literal_empty_output() {
        let mut semantic_empty = Vec::new();
        push_labeled_block(&mut semantic_empty, "Got", "");
        assert_eq!(semantic_empty, ["", "Got: <empty>"]);

        let mut literal_empty = Vec::new();
        push_labeled_block(&mut literal_empty, "Got", "<empty>");
        assert_eq!(literal_empty, ["", "Got", "  <empty>"]);
    }

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
    fn compiler_gate_rejects_timeout_even_with_zero_exit_code() {
        let output = crate::process::RunOutput {
            code: Some(0),
            stdout: String::new(),
            stderr: "partial diagnostic\n".to_string(),
            timed_out: true,
        };
        let failure = compiler_failure(JudgeFailureKind::Compile, &output);

        assert!(compiler_gate_failed(&output));
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

    #[test]
    fn compiler_diagnostics_are_bounded_without_splitting_utf8() {
        let mut stderr = "solution.ts(1,1): error TS9999: first useful message".to_string();
        for line in 0..40 {
            stderr.push_str(&format!("\n{line}: {}", "한".repeat(300)));
        }
        let output = crate::process::RunOutput {
            code: Some(1),
            stdout: String::new(),
            stderr,
            timed_out: false,
        };

        let detail = compiler_detail(&output);

        assert!(detail.starts_with("solution.ts(1,1): error TS9999: first useful message"));
        assert!(detail.contains("[diagnostic truncated]"));
        assert!(detail.lines().count() <= 13);
        assert!(detail.len() <= 4_128);
        assert!(!detail.contains('\u{fffd}'));
    }

    #[test]
    fn typescript_version_contract_accepts_only_5_9() {
        assert!(typescript_version_is_supported("Version 5.9.0"));
        assert!(typescript_version_is_supported("Version 5.9.99\n"));
        assert!(!typescript_version_is_supported("Version 5.8.4"));
        assert!(!typescript_version_is_supported("Version 6.0.0"));
        assert!(!typescript_version_is_supported("Version 5.9"));
        assert!(!typescript_version_is_supported("garbled"));
    }

    #[test]
    fn typescript_version_probe_rejects_bad_exit_output_and_timeout() {
        let output = |code, stdout: &str, stderr: &str, timed_out| crate::process::RunOutput {
            code,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            timed_out,
        };

        assert!(
            validate_typescript_version(&output(Some(0), "Version 5.9.3\n", "", false)).is_ok()
        );
        for reported in ["Version 5.8.4\n", "Version 6.0.0\n", "garbled\n"] {
            let failure =
                validate_typescript_version(&output(Some(0), reported, "", false)).unwrap_err();
            assert_eq!(failure.kind, JudgeFailureKind::TypeCheck);
            assert!(failure.detail.contains("TypeScript 5.9.x"));
        }
        let failure =
            validate_typescript_version(&output(Some(1), "", "version probe failed\n", false))
                .unwrap_err();
        assert_eq!(failure.kind, JudgeFailureKind::TypeCheck);
        assert!(failure.detail.contains("version probe failed"));

        let failure =
            validate_typescript_version(&output(Some(0), "Version 5.9.3\n", "", true)).unwrap_err();
        assert_eq!(failure.kind, JudgeFailureKind::Timeout);
        assert!(failure.detail.contains("timeout: 30s"));
    }

    #[test]
    fn typescript_typecheck_runs_before_resolving_node_with_isolated_types() {
        use std::{cell::RefCell, rc::Rc};

        let root = crate::process::unique_temp_path("practicode-ts-staging", "dir");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("solution.ts");
        fs::write(&path, "const value: number = 'bad';\nconsole.log(value);\n").unwrap();
        let stale_type_root = root
            .join("build")
            .join(root.file_name().unwrap())
            .join("typescript/type-roots");
        fs::create_dir_all(&stale_type_root).unwrap();
        fs::write(
            stale_type_root.join("stale.d.ts"),
            "declare const stale: true;\n",
        )
        .unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let prepare_events = Rc::clone(&events);

        let result = judge_path_with(
            &root,
            "injected-typescript-staging",
            &path,
            "ts",
            &[IoCase {
                input: String::new(),
                output: "bad\n".to_string(),
            }],
            move |root, path, _| {
                let find_events = Rc::clone(&prepare_events);
                let run_events = Rc::clone(&prepare_events);
                compile_typescript_with(
                    root,
                    path,
                    move |name| {
                        find_events.borrow_mut().push(format!("find:{name}"));
                        (name == "tsc").then(|| PathBuf::from("fake-tsc"))
                    },
                    move |_, args| {
                        if args == ["--version"] {
                            run_events.borrow_mut().push("run:version".to_string());
                            return Ok(crate::process::RunOutput {
                                code: Some(0),
                                stdout: "Version 5.9.3\n".to_string(),
                                stderr: String::new(),
                                timed_out: false,
                            });
                        }
                        run_events.borrow_mut().push("run:typecheck".to_string());
                        let type_root = Path::new(
                            &args[args.iter().position(|arg| arg == "--typeRoots").unwrap() + 1],
                        );
                        assert!(type_root.is_dir());
                        assert_eq!(fs::read_dir(type_root).unwrap().count(), 0);
                        Ok(crate::process::RunOutput {
                            code: Some(1),
                            stdout: "solution.ts(1,7): error TS2322: bad type\n".to_string(),
                            stderr: String::new(),
                            timed_out: false,
                        })
                    },
                )
            },
        );

        assert_eq!(result.failure_kind, Some(JudgeFailureKind::TypeCheck));
        assert!(result.output.contains("TS2322"));
        assert_eq!(
            events.borrow().as_slice(),
            ["find:tsc", "run:version", "run:typecheck"]
        );
        let _ = fs::remove_dir_all(root);
    }
}
