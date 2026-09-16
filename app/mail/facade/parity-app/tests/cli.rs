use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const CRATES: &[&str] = &[
    "main",
    "types",
    "email",
    "smtp",
    "imap",
    "imap-proto",
    "jmap",
    "jmap-proto",
    "pop3",
    "managesieve",
    "dav",
    "dav-proto",
    "groupware",
    "http",
    "http-proto",
    "scim",
    "scim-proto",
    "directory",
    "store",
    "coordinator",
    "registry",
    "services",
    "spam-filter",
    "nlp",
    "common",
    "trc",
    "utils",
    "migration",
    "trc/event-macro",
    "utils/proc-macros",
];
fn git(root: &Path, args: &[&str]) -> String {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.name=Mail Test",
            "-c",
            "user.email=mail-test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_owned()
}
fn put(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn snapshot(root: &Path) -> String {
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "Test snapshot"]);
    git(root, &["rev-parse", "HEAD"])
}
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init"]);
    for name in CRATES {
        put(
            dir.path(),
            &format!("crates/{name}/Cargo.toml"),
            &format!(
                "[package]\nname = {:?}\nlicense = \"Apache-2.0\"\n[features]\nenterprise = []\n",
                name.replace('/', "-")
            ),
        );
    }
    put(
        dir.path(),
        "tests/Cargo.toml",
        "[package]\nname = \"tests\"\n",
    );
    put(
        dir.path(),
        "tests/src/protocol.rs",
        "pub fn protocol() {}\n",
    );
    put(
        dir.path(),
        "crates/imap/src/lib.rs",
        "#[cfg(feature = \"enterprise\")]\npub fn enterprise() {}\n",
    );
    put(
        dir.path(),
        "tests/resources/email.txt",
        "Subject: baseline\r\n\r\nbody\r\n",
    );
    snapshot(dir.path());
    dir
}
fn run(root: &Path, verb: &str, revisions: &[&str]) -> Output {
    Command::new(
        option_env!("MAIL_PARITY_BINARY")
            .or(option_env!("CARGO_BIN_EXE_mail-parity-app"))
            .unwrap(),
    )
    .arg(verb)
    .arg(root)
    .args(revisions)
    .output()
    .unwrap()
}
fn json(result: &Output) -> Value {
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn mapped_upstream_snapshot_is_bound_to_git_bytes_and_never_asserts_full_parity() {
    let dir = fixture();
    let root = dir.path();
    let revision = git(root, &["rev-parse", "HEAD"]);
    let tree = git(root, &["rev-parse", "HEAD^{tree}"]);
    let inspected = run(root, "inspect", &[&revision]);
    assert!(
        inspected.status.success(),
        "{}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    let report = json(&inspected);
    assert_eq!(report["commit"], revision);
    assert_eq!(report["tree"], tree);
    assert_eq!(report["mapping_complete"], true);
    assert_eq!(report["crates"].as_array().unwrap().len(), 31);
    assert_eq!(report["enterprise_surfaces"].as_array().unwrap().len(), 1);
    assert_eq!(report["test_files"].as_array().unwrap().len(), 1);
    assert_eq!(report["scope"]["full_parity"], false);
    let checked = run(root, "check", &[&revision]);
    assert!(!checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stderr).contains("full parity not qualified"));
    let mapped = run(root, "map", &[&revision]);
    assert!(mapped.status.success());
    let view = String::from_utf8(mapped.stdout).unwrap();
    assert!(view.contains(&revision) && view.contains(&tree));
    assert!(view.contains("imap-server") && view.contains("app/mail/ports"));
}

#[test]
fn drift_reports_added_modified_removed_inputs_and_refuses_unknown_crates() {
    let dir = fixture();
    let root = dir.path();
    let before = git(root, &["rev-parse", "HEAD"]);
    fs::remove_dir_all(root.join("crates/pop3")).unwrap();
    put(
        root,
        "crates/unclassified/Cargo.toml",
        "[package]\nname = \"unknown\"\n",
    );
    put(root, "crates/imap/src/lib.rs", "pub fn revised() {}\n");
    put(
        root,
        "tests/resources/email.txt",
        "Subject: changed\r\n\r\nbody\r\n",
    );
    put(root, "tests/src/added.rs", "pub fn added() {}\n");
    let after = snapshot(root);
    let diff = run(root, "diff", &[&before, &after]);
    assert!(diff.status.success());
    let report = json(&diff);
    assert_eq!(report["base"], before);
    assert_eq!(report["target"], after);
    assert_eq!(report["target_mapping_complete"], false);
    assert_eq!(
        report["target_unmapped"],
        serde_json::json!(["crates/unclassified"])
    );
    let changes = report["changes"].as_array().unwrap();
    for (category, path, change) in [
        ("crates", "crates/imap", "modified"),
        ("crates", "crates/pop3", "removed"),
        ("test_files", "tests/src/added.rs", "added"),
        ("enterprise_surfaces", "crates/imap/src/lib.rs", "removed"),
        (
            "compatibility_inputs",
            "tests/resources/email.txt",
            "modified",
        ),
    ] {
        assert!(
            changes
                .iter()
                .any(|v| v["category"] == category && v["path"] == path && v["change"] == change),
            "{report}"
        );
    }
    let refused = run(root, "inspect", &[&after]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("mapping changed"));
    assert!(!run(root, "inspect", &["missing-revision"]).status.success());
}
