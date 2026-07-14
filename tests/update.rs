use practicode::update::is_newer;

#[test]
fn version_compare_detects_patch_updates() {
    assert!(is_newer("0.1.2", "0.1.1"));
    assert!(is_newer("0.2.0", "0.1.9"));
    assert!(!is_newer("0.1.1", "0.1.1"));
    assert!(!is_newer("0.1.0", "0.1.1"));
}

#[test]
fn malformed_or_prerelease_versions_do_not_trigger_an_update() {
    assert!(!is_newer("999.invalid", "0.2.0"));
    assert!(!is_newer("1.0", "0.2.0"));
    assert!(!is_newer("1.0.0-beta.1", "0.2.0"));
}
