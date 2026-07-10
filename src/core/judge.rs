use super::*;
use std::env;

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
                output: format!("Missing runtime for {language}"),
            };
        }
        Err(error) => {
            return JudgeResult {
                passed: false,
                passed_cases: 0,
                total_cases: cases.len(),
                output: format!("compile failed\n{error}"),
            };
        }
    };
    let run_dir = root.join("build").join(id).join("run");
    if let Err(error) = fs::create_dir_all(&run_dir) {
        return JudgeResult {
            passed: false,
            passed_cases: 0,
            total_cases: cases.len(),
            output: error.to_string(),
        };
    }

    let mut passed = 0;
    let mut lines = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let mut process = Command::new(&command.program);
        process.args(&command.args).current_dir(&run_dir);
        apply_judge_env(&mut process);
        let run = match run_capture(&mut process, &case.input, Duration::from_secs(5)) {
            Ok(run) => run,
            Err(error) => {
                lines.push(format!("Case {}: FAIL", index + 1));
                push_labeled_block(&mut lines, "Error", &error.to_string());
                break;
            }
        };
        let got = run.stdout.trim();
        let expected = case.output.trim();
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
            lines.push(format!("Case {}: FAIL", index + 1));
            if run.timed_out {
                push_labeled_block(&mut lines, "Error", "timeout: 5s");
            }
            push_labeled_block(&mut lines, "Got", run.stdout.trim_end());
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
        "ts" => Ok(which("node").map(|program| CommandSpec {
            program,
            args: vec![
                "--experimental-strip-types".to_string(),
                path.display().to_string(),
            ],
        })),
        "java" => compile_java(root, path),
        "rust" => compile_rust(root, path),
        _ => Ok(None),
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
        .args([
            "-d",
            &build.display().to_string(),
            &path.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(anyhow!(output.stderr.trim().to_string()));
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
        .args([
            "--edition=2024".to_string(),
            path.display().to_string(),
            "-o".to_string(),
            exe.display().to_string(),
        ])
        .current_dir(root);
    let output = run_capture(&mut compile, "", Duration::from_secs(30))?;
    if output.code != Some(0) {
        return Err(anyhow!(output.stderr.trim().to_string()));
    }
    Ok(Some(CommandSpec {
        program: exe,
        args: Vec::new(),
    }))
}
