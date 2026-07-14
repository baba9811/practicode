use anyhow::{Context, Result, anyhow};
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread::{self, ScopedJoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use wait_timeout::ChildExt;

const MAX_CAPTURE_BYTES: usize = 4 * 1024 * 1024;
const OUTPUT_TRUNCATED_MARKER: &[u8] = b"\n[output truncated]\n";
static ACTIVE_COMMANDS: OnceLock<Mutex<CommandRegistry>> = OnceLock::new();

#[derive(Default)]
struct CommandRegistry {
    pids: HashSet<u32>,
    shutting_down: bool,
}

impl CommandRegistry {
    fn register(&mut self, pid: u32) -> bool {
        if self.shutting_down {
            return false;
        }
        self.pids.insert(pid);
        true
    }

    fn remove(&mut self, pid: u32) {
        self.pids.remove(&pid);
    }

    fn begin_shutdown(&mut self) -> Vec<u32> {
        self.shutting_down = true;
        self.pids.iter().copied().collect()
    }
}

struct ActiveCommand(u32);

impl ActiveCommand {
    fn register(pid: u32) -> Self {
        let registered = active_commands()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(pid);
        if !registered {
            let _ = terminate_process_group(pid);
        }
        Self(pid)
    }
}

impl Drop for ActiveCommand {
    fn drop(&mut self) {
        active_commands()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(self.0);
    }
}

fn active_commands() -> &'static Mutex<CommandRegistry> {
    ACTIVE_COMMANDS.get_or_init(|| Mutex::new(CommandRegistry::default()))
}

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct RunOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub fn run_capture(command: &mut Command, input: &str, timeout: Duration) -> Result<RunOutput> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().context("spawn command")?;
    let active = ActiveCommand::register(child.id());
    let stdin = child.stdin.take();
    let stdout = child.stdout.take().context("capture stdout")?;
    let stderr = child.stderr.take().context("capture stderr")?;

    thread::scope(|scope| {
        let input_thread = stdin.map(|mut stdin| {
            scope.spawn(move || match stdin.write_all(input.as_bytes()) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
                Err(error) => Err(error),
            })
        });
        let stdout_thread = scope.spawn(move || read_captured(stdout));
        let stderr_thread = scope.spawn(move || read_captured(stderr));

        let (status, timed_out) = match child.wait_timeout(timeout).context("wait for command")? {
            Some(status) => (status, false),
            None => {
                terminate_command(&mut child).context("kill timed out command")?;
                (child.wait().context("reap timed out command")?, true)
            }
        };
        if !timed_out {
            // A successful parent can leave descendants holding our pipes open forever.
            // run_capture owns the whole command tree, so no descendant should outlive it.
            let _ = terminate_process_group(child.id());
        }
        if let Some(handle) = input_thread {
            join_io_thread(handle, "write stdin")?;
        }
        let stdout = join_io_thread(stdout_thread, "read stdout")?;
        let stderr = join_io_thread(stderr_thread, "read stderr")?;
        drop(active);
        Ok(RunOutput {
            code: status.code(),
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
            timed_out,
        })
    })
}

pub(crate) fn terminate_active_commands() {
    let pids = active_commands()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .begin_shutdown();
    for pid in pids {
        let _ = terminate_process_group(pid);
    }
}

#[cfg(test)]
fn active_command_is_registered(pid: u32) -> bool {
    active_commands()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .pids
        .contains(&pid)
}

fn read_captured(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = MAX_CAPTURE_BYTES.saturating_sub(output.len());
        output.extend_from_slice(&buffer[..read.min(remaining)]);
        truncated |= read > remaining;
    }
    if truncated {
        output.extend_from_slice(OUTPUT_TRUNCATED_MARKER);
    }
    Ok(output)
}

#[cfg(unix)]
fn terminate_process_group(pid: u32) -> io::Result<()> {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }

    // SAFETY: the child starts a fresh process group whose id is its positive process id.
    if unsafe { kill(-(pid as i32), 9) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
fn terminate_command(child: &mut Child) -> io::Result<()> {
    terminate_process_group(child.id()).or_else(|_| match child.kill() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::InvalidInput => Ok(()),
        Err(error) => Err(error),
    })
}

#[cfg(windows)]
fn terminate_process_group(pid: u32) -> io::Result<()> {
    let pid = pid.to_string();
    let status = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid])
        .status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(io::Error::other("taskkill failed")),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn terminate_command(child: &mut Child) -> io::Result<()> {
    terminate_process_group(child.id()).or_else(|_| child.kill())
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_group(_pid: u32) -> io::Result<()> {
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "process tree termination is unsupported",
    ))
}

#[cfg(not(any(unix, windows)))]
fn terminate_command(child: &mut Child) -> io::Result<()> {
    child.kill()
}

fn join_io_thread<T>(handle: ScopedJoinHandle<'_, std::io::Result<T>>, action: &str) -> Result<T> {
    handle
        .join()
        .map_err(|_| anyhow!("{action} thread panicked"))?
        .with_context(|| action.to_string())
}

pub fn which(name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    let pathext = env::var_os("PATHEXT");
    env::split_paths(&paths).find_map(|dir| find_in_dir(&dir, name, pathext.as_deref()))
}

fn find_in_dir(dir: &Path, name: &str, pathext: Option<&OsStr>) -> Option<PathBuf> {
    let path = dir.join(name);
    if is_executable(&path) {
        return Some(path);
    }
    if Path::new(name).extension().is_some() {
        return None;
    }
    pathext?
        .to_string_lossy()
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| {
            let ext = if ext.starts_with('.') {
                ext.to_string()
            } else {
                format!(".{ext}")
            };
            dir.join(format!("{name}{ext}"))
        })
        .find(|path| is_executable(path))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

pub fn shell_process(command: &str) -> Command {
    if cfg!(windows) {
        let mut process = Command::new("cmd");
        process.args(["/C", command]);
        process
    } else {
        let mut process = Command::new("sh");
        process.args(["-c", command]);
        process
    }
}

pub fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn unique_temp_path(prefix: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{nanos}.{ext}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn run_capture_tolerates_child_that_exits_before_reading_stdin() {
        let mut command = shell_process("exit 0");
        let output = run_capture(
            &mut command,
            &"x".repeat(1024 * 1024),
            Duration::from_secs(5),
        )
        .unwrap();

        assert_eq!(output.code, Some(0));
        assert!(!output.timed_out);
    }

    #[cfg(unix)]
    #[test]
    fn run_capture_drains_output_while_child_is_running() {
        let mut command = shell_process(
            "i=0; while [ \"$i\" -lt 20000 ]; do printf 0123456789abcdef; i=$((i + 1)); done",
        );
        let output = run_capture(&mut command, "", Duration::from_secs(2)).unwrap();

        assert!(!output.timed_out);
        assert_eq!(output.stdout.len(), 320_000);
    }

    #[cfg(unix)]
    #[test]
    fn run_capture_timeout_terminates_descendants() {
        let started = std::time::Instant::now();
        let mut command = shell_process("sleep 3 & wait");

        let output = run_capture(&mut command, "", Duration::from_millis(100)).unwrap();

        assert!(output.timed_out);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn run_capture_does_not_wait_for_descendants_after_parent_exits() {
        let started = std::time::Instant::now();
        let mut command = shell_process("sleep 3 &");

        let output = run_capture(&mut command, "", Duration::from_secs(5)).unwrap();

        assert_eq!(output.code, Some(0));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn run_capture_bounds_captured_output() {
        let mut command = shell_process("dd if=/dev/zero bs=1048576 count=6 2>/dev/null");

        let output = run_capture(&mut command, "", Duration::from_secs(5)).unwrap();

        assert!(!output.timed_out);
        assert!(output.stdout.len() < 5 * 1024 * 1024);
        assert!(output.stdout.contains("[output truncated]"));
    }

    #[test]
    fn active_command_registration_is_removed_on_drop() {
        let pid = u32::MAX;
        {
            let _active = ActiveCommand::register(pid);
            assert!(active_command_is_registered(pid));
        }
        assert!(!active_command_is_registered(pid));
    }

    #[test]
    fn command_registry_rejects_registration_after_shutdown_starts() {
        let mut registry = CommandRegistry::default();
        assert!(registry.register(10));
        assert_eq!(registry.begin_shutdown(), vec![10]);
        assert!(!registry.register(11));
    }

    #[test]
    fn which_honors_pathext_suffixes() {
        let root = unique_temp_path("practicode-which", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let exe = root.join("tool.CMD");
        std::fs::write(&exe, "").unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(
            find_in_dir(&root, "tool", Some(OsStr::new(".EXE;.CMD"))),
            Some(exe)
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn which_rejects_non_executable_files() {
        let root = unique_temp_path("practicode-which-permissions", "dir");
        std::fs::create_dir_all(&root).unwrap();
        let tool = root.join("tool");
        std::fs::write(&tool, "").unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o644)).unwrap();

        assert_eq!(find_in_dir(&root, "tool", None), None);

        let _ = std::fs::remove_dir_all(root);
    }
}
