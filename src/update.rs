use crate::process::{run_capture, which};
use std::{env, process::Command, time::Duration};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateCheck {
    Disabled,
    Current,
    Available(String),
    Failed,
}

pub fn check_latest_version() -> UpdateCheck {
    if env::var("PRACTICODE_NO_UPDATE_CHECK").ok().as_deref() == Some("1") {
        return UpdateCheck::Disabled;
    }
    let Some(npm) = which("npm") else {
        return UpdateCheck::Failed;
    };
    let mut command = Command::new(npm);
    command.args(["view", "practicode", "version", "--silent"]);
    match run_capture(&mut command, "", Duration::from_secs(5)) {
        Ok(output) if output.code == Some(0) => {
            let latest = output.stdout.trim();
            if is_newer(latest, CURRENT_VERSION) {
                UpdateCheck::Available(latest.to_string())
            } else {
                UpdateCheck::Current
            }
        }
        _ => UpdateCheck::Failed,
    }
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!(
        (version_parts(latest), version_parts(current)),
        (Some(latest), Some(current)) if latest > current
    )
}

fn version_parts(version: &str) -> Option<[u64; 3]> {
    let mut parts = version.split('.');
    let version = [
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ];
    parts.next().is_none().then_some(version)
}
