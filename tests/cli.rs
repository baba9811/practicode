use std::process::Command;

#[test]
fn version_does_not_require_a_data_home() {
    let output = Command::new(env!("CARGO_BIN_EXE_practicode"))
        .arg("--version")
        .env_remove("PRACTICODE_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("practicode "));
}

#[test]
fn unknown_argument_is_rejected_before_resolving_the_data_home() {
    let output = Command::new(env!("CARGO_BIN_EXE_practicode"))
        .arg("--typo")
        .env_remove("PRACTICODE_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown argument: --typo"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
