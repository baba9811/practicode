use anyhow::{Context, Result, bail};
use crossterm::{cursor::SetCursorStyle, event::DisableMouseCapture, execute};
use std::{
    env,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, stdout},
    path::{Path, PathBuf},
};

pub mod ai;
pub mod core;
pub mod i18n;
pub mod process;
pub mod text;
pub mod tui;
pub mod update;

const LEGACY_MIGRATION_MARKER: &str = ".legacy-migration-in-progress";

pub fn cli_help_text() -> &'static str {
    "practicode - local-first coding-test practice in your terminal

Usage:
  practicode            Open the TUI
  practicode --smoke    Print the current problem title and exit
  practicode --version  Print the installed version
  practicode --help     Show this help

Inside the TUI:
  Esc then /            Open the command palette
  ?                     Open help
  Ctrl+C                Quit"
}

pub fn run_cli() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!("{}", cli_help_text());
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-V" | "--version"))
    {
        println!("practicode {}", update::CURRENT_VERSION);
        return Ok(());
    }

    let launch_dir = env::current_dir().context("read current directory")?;
    let root = absolute_data_root(
        &launch_dir,
        resolve_data_root(
            env::var_os("PRACTICODE_HOME"),
            env::var_os("HOME"),
            env::var_os("USERPROFILE"),
        )?,
    );
    migrate_legacy_data(&launch_dir, &root)?;
    if args.iter().any(|arg| arg == "--smoke") {
        let bank = core::load_bank(&root)?;
        let state = core::load_state(&root, &bank)?;
        let problem = core::problem_by_id(&bank, &state.current_problem).unwrap_or(&bank[0]);
        println!(
            "{}",
            core::localized(&problem.title, &state.settings.ui_language)
        );
        return Ok(());
    }

    let mut app = tui::PracticodeApp::new(root)?;
    let mut terminal = ratatui::init();
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    let result = app.run(&mut terminal);
    ratatui::restore();
    let _ = execute!(
        stdout(),
        SetCursorStyle::DefaultUserShape,
        DisableMouseCapture
    );
    result
}

fn non_empty_path(value: Option<OsString>) -> Option<PathBuf> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from)
}

fn resolve_data_root(
    practicode_home: Option<OsString>,
    home: Option<OsString>,
    user_profile: Option<OsString>,
) -> Result<PathBuf> {
    if let Some(path) = non_empty_path(practicode_home) {
        return Ok(path);
    }
    if let Some(path) = non_empty_path(home) {
        return Ok(path.join(".practicode"));
    }
    if let Some(path) = non_empty_path(user_profile) {
        return Ok(path.join(".practicode"));
    }
    bail!("cannot find a user home directory; set PRACTICODE_HOME")
}

fn absolute_data_root(launch_dir: &Path, root: PathBuf) -> PathBuf {
    if root.is_absolute() {
        root
    } else {
        launch_dir.join(root)
    }
}

fn migrate_legacy_data(launch_dir: &Path, root: &Path) -> Result<()> {
    let legacy_metadata = launch_dir.join(".practicode");
    if !path_exists(
        &legacy_metadata.join(core::STATE_PATH),
        "inspect legacy marker",
    )? && !path_exists(
        &legacy_metadata.join(core::BANK_PATH),
        "inspect legacy marker",
    )? {
        return Ok(());
    }

    let same_metadata = same_existing_path(&legacy_metadata, root)?;
    if same_metadata {
        return Ok(());
    }
    let marker = root.join(LEGACY_MIGRATION_MARKER);
    let migration_in_progress = match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.file_type().is_file() => true,
        Ok(_) => bail!(
            "migration marker {} is not a regular file",
            marker.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect migration marker {}", marker.display()));
        }
    };
    if !migration_in_progress
        && (path_exists(&root.join(core::STATE_PATH), "inspect destination")?
            || path_exists(&root.join(core::BANK_PATH), "inspect destination")?)
    {
        return Ok(());
    }

    fs::create_dir_all(root).with_context(|| format!("create data root {}", root.display()))?;
    let canonical_root = validate_migration_root(launch_dir, root)?;
    if !migration_in_progress {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&marker)
            .with_context(|| format!("create migration marker {}", marker.display()))?;
    }
    for name in [core::STATE_PATH, core::BANK_PATH, core::PROBLEM_NOTES_PATH] {
        copy_file_if_missing(
            &legacy_metadata.join(name),
            &root.join(name),
            &canonical_root,
        )?;
    }
    for name in ["problems", "submissions"] {
        copy_tree_missing(&launch_dir.join(name), &root.join(name), &canonical_root)?;
    }
    fs::remove_file(&marker)
        .with_context(|| format!("remove migration marker {}", marker.display()))?;
    Ok(())
}

fn path_exists(path: &Path, action: &str) -> Result<bool> {
    path.try_exists()
        .with_context(|| format!("{action} {}", path.display()))
}

fn validate_migration_root(launch_dir: &Path, root: &Path) -> Result<PathBuf> {
    let root =
        fs::canonicalize(root).with_context(|| format!("resolve data root {}", root.display()))?;
    for name in ["problems", "submissions"] {
        let source = launch_dir.join(name);
        let metadata = match fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", source.display()));
            }
        };
        if !metadata.file_type().is_dir() {
            continue;
        }
        let source = fs::canonicalize(&source)
            .with_context(|| format!("resolve legacy directory {}", source.display()))?;
        if root.starts_with(&source) {
            bail!(
                "data root {} cannot be inside legacy directory {}",
                root.display(),
                source.display()
            );
        }
    }
    Ok(root)
}

fn same_existing_path(left: &Path, right: &Path) -> Result<bool> {
    let left = fs::canonicalize(left)
        .with_context(|| format!("resolve legacy metadata {}", left.display()))?;
    match fs::canonicalize(right) {
        Ok(right) => Ok(left == right),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("resolve data root {}", right.display())),
    }
}

fn copy_file_if_missing(source: &Path, destination: &Path, root: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {}", source.display()));
        }
    };
    if !metadata.file_type().is_file() {
        return Ok(());
    }
    match fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.file_type().is_symlink() => bail!(
            "destination file symlink {} is not allowed",
            destination.display()
        ),
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {}", destination.display()));
        }
    }
    let parent = destination
        .parent()
        .with_context(|| format!("find parent for {}", destination.display()))?;
    let parent = fs::canonicalize(parent)
        .with_context(|| format!("resolve destination directory {}", parent.display()))?;
    if !parent.starts_with(root) {
        bail!(
            "destination directory {} is outside data root {}",
            parent.display(),
            root.display()
        );
    }

    let mut input = File::open(source).with_context(|| format!("open {}", source.display()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .with_context(|| format!("create {} without overwriting", destination.display()))?;
    if let Err(error) = io::copy(&mut input, &mut output) {
        let _ = fs::remove_file(destination);
        return Err(error).with_context(|| {
            format!(
                "copy legacy data {} to {}",
                source.display(),
                destination.display()
            )
        });
    }
    Ok(())
}

fn copy_tree_missing(source: &Path, destination: &Path, root: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {}", source.display()));
        }
    };
    if !metadata.file_type().is_dir() {
        return Ok(());
    }

    match fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            let destination = fs::canonicalize(destination).with_context(|| {
                format!("resolve destination directory {}", destination.display())
            })?;
            if !destination.starts_with(root) {
                bail!(
                    "destination directory {} is outside data root {}",
                    destination.display(),
                    root.display()
                );
            }
        }
        Ok(_) => bail!(
            "destination directory {} is not a regular directory",
            destination.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = destination
                .parent()
                .with_context(|| format!("find parent for {}", destination.display()))?;
            let parent = fs::canonicalize(parent)
                .with_context(|| format!("resolve destination directory {}", parent.display()))?;
            if !parent.starts_with(root) {
                bail!(
                    "destination directory {} is outside data root {}",
                    parent.display(),
                    root.display()
                );
            }
            fs::create_dir(destination)
                .with_context(|| format!("create directory {}", destination.display()))?;
        }
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {}", destination.display()));
        }
    }
    for entry in fs::read_dir(source)
        .with_context(|| format!("read legacy directory {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", source.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("inspect {}", entry.path().display()))?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree_missing(&entry.path(), &target, root)?;
        } else if file_type.is_file() {
            copy_file_if_missing(&entry.path(), &target, root)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::OsString,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!(
            "practicode-lib-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn cli_help_lists_non_interactive_flags_and_tui_exit() {
        let help = cli_help_text();
        assert!(help.contains("--smoke"));
        assert!(help.contains("--version"));
        assert!(help.contains("Ctrl+C"));
    }

    #[test]
    fn data_root_prefers_practicode_home() {
        let root = resolve_data_root(
            Some(OsString::from("/custom")),
            Some(OsString::from("/home/user")),
            Some(OsString::from("/users/user")),
        )
        .unwrap();
        assert_eq!(root, PathBuf::from("/custom"));
    }

    #[test]
    fn data_root_uses_home() {
        let root = resolve_data_root(None, Some(OsString::from("/home/user")), None).unwrap();
        assert_eq!(root, PathBuf::from("/home/user/.practicode"));
    }

    #[test]
    fn data_root_uses_user_profile_when_home_is_empty() {
        let root = resolve_data_root(
            None,
            Some(OsString::new()),
            Some(OsString::from(r"C:\Users\user")),
        )
        .unwrap();
        assert_eq!(root, PathBuf::from(r"C:\Users\user").join(".practicode"));
    }

    #[test]
    fn data_root_requires_a_home_directory() {
        let error = resolve_data_root(None, None, None).unwrap_err().to_string();
        assert!(error.contains("PRACTICODE_HOME"));
    }

    #[test]
    fn relative_data_root_is_resolved_from_the_launch_directory() {
        assert_eq!(
            absolute_data_root(Path::new("/launch"), PathBuf::from("data")),
            PathBuf::from("/launch/data")
        );
    }

    #[test]
    fn legacy_migration_copies_user_data_without_cache_or_overwrite() {
        let launch = temp_root("legacy-copy-launch");
        let root = temp_root("legacy-copy-root");
        let metadata = launch.join(".practicode");
        fs::create_dir_all(metadata.join("build")).unwrap();
        fs::create_dir_all(launch.join("problems/002-echo")).unwrap();
        fs::create_dir_all(launch.join("submissions/002-echo")).unwrap();
        fs::write(metadata.join("problem-state.json"), "old state").unwrap();
        fs::write(metadata.join("problem_bank.json"), "old bank").unwrap();
        fs::write(metadata.join("problem_notes.md"), "old notes").unwrap();
        fs::write(metadata.join("build/cache"), "cache").unwrap();
        fs::write(launch.join("problems/INDEX.md"), "index").unwrap();
        fs::write(launch.join("problems/002-echo/README.md"), "problem").unwrap();
        fs::write(
            launch.join("submissions/002-echo/solution.rs"),
            "fn main() {}",
        )
        .unwrap();
        fs::write(root.join("problem_notes.md"), "keep notes").unwrap();

        migrate_legacy_data(&launch, &root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join("problem-state.json")).unwrap(),
            "old state"
        );
        assert_eq!(
            fs::read_to_string(root.join("problem_bank.json")).unwrap(),
            "old bank"
        );
        assert_eq!(
            fs::read_to_string(root.join("problem_notes.md")).unwrap(),
            "keep notes"
        );
        assert_eq!(
            fs::read_to_string(root.join("problems/002-echo/README.md")).unwrap(),
            "problem"
        );
        assert_eq!(
            fs::read_to_string(root.join("submissions/002-echo/solution.rs")).unwrap(),
            "fn main() {}"
        );
        assert!(!root.join("build").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_migration_skips_a_populated_destination() {
        let launch = temp_root("legacy-skip-launch");
        let root = temp_root("legacy-skip-root");
        fs::create_dir_all(launch.join(".practicode")).unwrap();
        fs::create_dir_all(launch.join("problems")).unwrap();
        fs::write(launch.join(".practicode/problem-state.json"), "old state").unwrap();
        fs::write(launch.join("problems/INDEX.md"), "old index").unwrap();
        fs::write(root.join("problem-state.json"), "new state").unwrap();

        migrate_legacy_data(&launch, &root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join("problem-state.json")).unwrap(),
            "new state"
        );
        assert!(!root.join("problems").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_migration_resumes_an_interrupted_copy() {
        let launch = temp_root("legacy-resume-launch");
        let root = temp_root("legacy-resume-root");
        fs::create_dir_all(launch.join(".practicode")).unwrap();
        fs::create_dir_all(launch.join("problems")).unwrap();
        fs::write(launch.join(".practicode/problem-state.json"), "state").unwrap();
        fs::write(launch.join(".practicode/problem_bank.json"), "bank").unwrap();
        fs::write(launch.join("problems/INDEX.md"), "index").unwrap();
        fs::write(root.join("problem-state.json"), "state").unwrap();
        fs::write(root.join(".legacy-migration-in-progress"), "").unwrap();

        migrate_legacy_data(&launch, &root).unwrap();

        assert_eq!(
            fs::read_to_string(root.join("problem_bank.json")).unwrap(),
            "bank"
        );
        assert_eq!(
            fs::read_to_string(root.join("problems/INDEX.md")).unwrap(),
            "index"
        );
        assert!(!root.join(".legacy-migration-in-progress").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_root_rejects_a_legacy_source_descendant() {
        let launch = temp_root("legacy-descendant");
        let root = launch.join("problems/data");
        fs::create_dir_all(&root).unwrap();

        let error = validate_migration_root(&launch, &root)
            .unwrap_err()
            .to_string();

        assert!(error.contains("inside legacy"));
        fs::remove_dir_all(launch).unwrap();
    }

    #[test]
    fn legacy_migration_reports_marker_lookup_errors() {
        let launch = temp_root("legacy-marker-error");
        let root = temp_root("legacy-marker-error-root");
        fs::write(launch.join(".practicode"), "not a directory").unwrap();

        let error = migrate_legacy_data(&launch, &root).unwrap_err().to_string();

        assert!(error.contains("inspect legacy marker"));
        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn legacy_migration_does_not_follow_destination_symlinks() {
        use std::os::unix::fs::symlink;

        let launch = temp_root("legacy-symlink-launch");
        let root = temp_root("legacy-symlink-root");
        let outside = temp_root("legacy-symlink-outside");
        fs::create_dir_all(launch.join(".practicode")).unwrap();
        fs::create_dir_all(launch.join("problems")).unwrap();
        fs::write(launch.join(".practicode/problem-state.json"), "state").unwrap();
        fs::write(launch.join(".practicode/problem_bank.json"), "bank").unwrap();
        fs::write(launch.join(".practicode/problem_notes.md"), "notes").unwrap();
        fs::write(launch.join("problems/INDEX.md"), "index").unwrap();
        symlink(outside.join("notes.md"), root.join("problem_notes.md")).unwrap();
        symlink(&outside, root.join("problems")).unwrap();

        let error = migrate_legacy_data(&launch, &root).unwrap_err().to_string();

        assert!(error.contains("destination"));
        assert!(!outside.join("notes.md").exists());
        assert!(!outside.join("INDEX.md").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn legacy_migration_rejects_a_destination_file_symlink() {
        use std::os::unix::fs::symlink;

        let launch = temp_root("legacy-file-symlink-launch");
        let root = temp_root("legacy-file-symlink-root");
        let outside = temp_root("legacy-file-symlink-outside");
        fs::create_dir_all(launch.join(".practicode")).unwrap();
        fs::write(launch.join(".practicode/problem-state.json"), "state").unwrap();
        fs::write(launch.join(".practicode/problem_notes.md"), "notes").unwrap();
        symlink(outside.join("notes.md"), root.join("problem_notes.md")).unwrap();

        let error = migrate_legacy_data(&launch, &root).unwrap_err().to_string();

        assert!(error.contains("destination file symlink"));
        assert!(!outside.join("notes.md").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn legacy_migration_rejects_a_marker_symlink() {
        use std::os::unix::fs::symlink;

        let launch = temp_root("legacy-marker-symlink-launch");
        let root = temp_root("legacy-marker-symlink-root");
        let outside = temp_root("legacy-marker-symlink-outside");
        fs::create_dir_all(launch.join(".practicode")).unwrap();
        fs::write(launch.join(".practicode/problem-state.json"), "state").unwrap();
        symlink(outside.join("marker"), root.join(LEGACY_MIGRATION_MARKER)).unwrap();

        let error = migrate_legacy_data(&launch, &root).unwrap_err().to_string();

        assert!(error.contains("migration marker"));
        assert!(!outside.join("marker").exists());

        fs::remove_dir_all(launch).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn legacy_migration_does_not_copy_siblings_when_metadata_is_already_global() {
        let launch = temp_root("legacy-same-root");
        let root = launch.join(".practicode");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(launch.join("problems")).unwrap();
        fs::create_dir_all(launch.join("submissions/002-echo")).unwrap();
        fs::write(root.join("problem-state.json"), "state").unwrap();
        fs::write(launch.join("problems/INDEX.md"), "index").unwrap();
        fs::write(
            launch.join("submissions/002-echo/solution.py"),
            "print('echo')",
        )
        .unwrap();

        migrate_legacy_data(&launch, &root).unwrap();

        assert!(!root.join("problems").exists());
        assert!(!root.join("submissions").exists());

        fs::remove_dir_all(launch).unwrap();
    }
}
