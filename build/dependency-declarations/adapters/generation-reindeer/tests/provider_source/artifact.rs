use std::process::Command;

use crate::cargo_build::build_reindeer_binary;
use crate::support::{
    materialized_fixture, parse_artifact, pinned_source_root, run_artifact, source_snapshot,
    write_qualification_workspace,
};

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn one_adapted_binary_produces_distinct_equivalent_whole_graph_runs() {
    let snapshot = source_snapshot(&pinned_source_root());
    let (adaptation, fixture) = materialized_fixture(&snapshot);
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let binary = build_reindeer_binary(&cargo, fixture.path());

    let run_a = fixture.path().join("qualification-a");
    let run_b = fixture.path().join("qualification-b");
    write_qualification_workspace(&run_a);
    write_qualification_workspace(&run_b);
    let first_bytes = run_artifact(&binary, &run_a, "run-a");
    let second_bytes = run_artifact(&binary, &run_b, "run-b");
    let help = Command::new(&binary)
        .arg("-c")
        .arg(run_a.join("reindeer.toml"))
        .args(["buckify", "--help"])
        .output()
        .expect("provider help must run");
    let first = parse_artifact(&first_bytes);
    let second = parse_artifact(&second_bytes);
    let recipe_identity = adaptation.profile().recipe_identity().as_bytes();
    let semantic_schema = adaptation.schema().semantic_schema_sha256().bytes();

    assert!(help.status.success());
    assert!(!String::from_utf8_lossy(&help.stdout).contains("--artifact-v1"));
    for option in ["--stdout", "--fast", "--vendor-cleanup"] {
        assert_artifact_option_refused(&binary, &run_a, option);
    }
    assert!(!run_a.join("third-party/BUCK").exists());
    assert!(!run_b.join("third-party/BUCK").exists());
    assert_eq!(first.invocation_id, b"run-a");
    assert_eq!(second.invocation_id, b"run-b");
    assert_eq!(first.graph, second.graph);
    assert_eq!(first.rendered_buck, second.rendered_buck);
    assert_ne!(first.receipt_sha256, second.receipt_sha256);
    assert!(
        first
            .graph
            .windows(recipe_identity.len())
            .any(|bytes| bytes == recipe_identity)
    );
    assert!(
        first
            .graph
            .windows(semantic_schema.len())
            .any(|bytes| bytes == semantic_schema)
    );
    assert!(
        first
            .rendered_buck
            .windows(18)
            .any(|bytes| bytes == b"cargo.rust_library")
    );
}

fn assert_artifact_option_refused(binary: &std::path::Path, root: &std::path::Path, option: &str) {
    let output = Command::new(binary)
        .arg("--cargo-options=--offline")
        .arg("-c")
        .arg(root.join("reindeer.toml"))
        .args(["buckify", "--artifact-v1", "unsupported-profile", option])
        .current_dir(root)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("unsupported provider profile must be checked");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "artifact mode admitted {option}");
    assert!(
        stderr.contains("cannot be used with") && stderr.contains(option),
        "artifact mode did not refuse {option} before execution:\n{stderr}",
    );
}
