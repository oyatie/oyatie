use std::collections::BTreeMap;
use std::path::Path;

use dependency_declarations_generation_reindeer::ReindeerProviderSourceSnapshotV1;

use crate::cargo_build::{CargoMessageStreamV1, provider_cargo, qualification_target_dir};
use crate::support::{QualifiedProvider, SourceFixture};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ClippyDiagnosticV1 {
    file: String,
    code: String,
    message: String,
}

pub(super) fn adapted_source_builds_as_the_pinned_provider(
    snapshot: &ReindeerProviderSourceSnapshotV1,
    provider: &QualifiedProvider,
) {
    let pristine = SourceFixture::from_snapshot(snapshot);
    let pristine_diagnostics = clippy_diagnostics(pristine.path());
    let adapted_diagnostics = clippy_diagnostics(provider.source_root());
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

    let fault_tests = provider_cargo(provider.source_root())
        .args(["test", "--locked", "--offline", "artifact_"])
        .env(
            "CARGO_TARGET_DIR",
            qualification_target_dir(provider.source_root()),
        )
        .output()
        .expect("adapted provider fault tests must run");
    assert_command_succeeded(&fault_tests, "fault tests");
}

fn clippy_diagnostics(root: &Path) -> BTreeMap<ClippyDiagnosticV1, usize> {
    let output = provider_cargo(root)
        .args([
            "clippy",
            "--locked",
            "--offline",
            "--all-targets",
            "--no-deps",
            "--message-format=json",
        ])
        .env("CARGO_TARGET_DIR", qualification_target_dir(root))
        .output()
        .expect("provider Clippy differential must run");
    assert_command_succeeded(&output, "Clippy differential");

    let mut diagnostics = BTreeMap::new();
    let mut saw_reindeer_artifact = false;
    let messages = CargoMessageStreamV1::try_new(&output.stdout)
        .expect("Clippy must emit one complete Cargo message stream");
    for event in messages.events() {
        if event["reason"] == "compiler-artifact" && event["target"]["name"] == "reindeer" {
            saw_reindeer_artifact = true;
        }
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
    assert!(saw_reindeer_artifact, "Clippy emitted no Reindeer artifact");
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
