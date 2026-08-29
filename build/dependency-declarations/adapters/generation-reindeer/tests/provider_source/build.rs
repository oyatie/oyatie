use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::support::{SourceFixture, materialized_fixture, pinned_source_root};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ClippyDiagnosticV1 {
    file: String,
    code: String,
    message: String,
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn adapted_source_builds_as_the_pinned_provider() {
    let source_root = pinned_source_root();
    let pristine = SourceFixture::copy_from(&source_root);
    let (_, fixture) = materialized_fixture(&source_root);
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let pristine_diagnostics = clippy_diagnostics(&cargo, pristine.path());
    let adapted_diagnostics = clippy_diagnostics(&cargo, fixture.path());
    assert!(
        !pristine_diagnostics.is_empty(),
        "pinned upstream unexpectedly emitted no Clippy baseline"
    );
    for (diagnostic, adapted_count) in adapted_diagnostics {
        let pristine_count = pristine_diagnostics
            .get(&diagnostic)
            .copied()
            .unwrap_or_default();
        assert!(
            adapted_count <= pristine_count,
            "adaptation introduced Clippy diagnostic: {diagnostic:?}"
        );
    }

    let fault_tests = Command::new(cargo)
        .args(["test", "--locked", "--offline", "artifact_"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", fixture.path().join("target"))
        .output()
        .expect("adapted provider fault tests must run");
    assert_command_succeeded(&fault_tests, "fault tests");
}

fn clippy_diagnostics(cargo: &OsStr, root: &Path) -> BTreeMap<ClippyDiagnosticV1, usize> {
    let output = Command::new(cargo)
        .args([
            "clippy",
            "--locked",
            "--offline",
            "--all-targets",
            "--no-deps",
            "--message-format=json",
        ])
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .expect("provider Clippy differential must run");
    assert_command_succeeded(&output, "Clippy differential");

    let mut diagnostics = BTreeMap::new();
    for line in output.stdout.split(|byte| *byte == b'\n') {
        let Ok(event) = serde_json::from_slice::<serde_json::Value>(line) else {
            continue;
        };
        if event["reason"] != "compiler-message" || event["message"]["level"] != "warning" {
            continue;
        }
        let message = &event["message"];
        let Some(primary) = message["spans"]
            .as_array()
            .and_then(|spans| spans.iter().find(|span| span["is_primary"] == true))
        else {
            continue;
        };
        let Some(file) = primary["file_name"].as_str() else {
            continue;
        };
        if !file.starts_with("src/") {
            continue;
        }
        let diagnostic = ClippyDiagnosticV1 {
            file: file.to_owned(),
            code: message["code"]["code"]
                .as_str()
                .unwrap_or("uncoded-warning")
                .to_owned(),
            message: message["message"].as_str().unwrap_or_default().to_owned(),
        };
        *diagnostics.entry(diagnostic).or_default() += 1;
    }
    diagnostics
}

fn assert_command_succeeded(output: &std::process::Output, operation: &str) {
    assert!(
        output.status.success(),
        "adapted provider {operation} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
