use std::process::Command;

use dependency_declarations_generation::RenderedDeclarationProjectionPort;
use dependency_declarations_generation_reindeer::StarlarkSyntaxProjectionV1;
use dependency_declarations_reconcile::DigestV1;

use crate::qualification::assert_provider_parser_reconciliation;
use crate::support::{
    QualifiedProvider, parse_artifact, run_artifact, write_qualification_workspace,
};

pub(super) fn one_adapted_binary_produces_distinct_equivalent_whole_graph_runs(
    provider: &QualifiedProvider,
) {
    let binary = provider.binary();

    let run_a = provider.source_root().join("qualification-a");
    let run_b = provider.source_root().join("qualification-b");
    write_qualification_workspace(&run_a);
    write_qualification_workspace(&run_b);
    let first_bytes = run_artifact(binary, &run_a, "run-a");
    let second_bytes = run_artifact(binary, &run_b, "run-b");
    let help = Command::new(binary)
        .arg("-c")
        .arg(run_a.join("reindeer.toml"))
        .args(["buckify", "--help"])
        .output()
        .expect("provider help must run");
    let first = parse_artifact(&first_bytes);
    let second = parse_artifact(&second_bytes);
    let projection = StarlarkSyntaxProjectionV1::new(DigestV1::of(b"qualification-profile"))
        .project(first.rendered_buck)
        .expect("maintained parser must project exact provider output");
    let recipe_identity = provider.adaptation().profile().recipe_identity().as_bytes();
    let semantic_schema = provider
        .adaptation()
        .schema()
        .semantic_schema_sha256()
        .bytes();

    assert!(help.status.success());
    assert!(!String::from_utf8_lossy(&help.stdout).contains("--artifact-v1"));
    for option in ["--stdout", "--fast", "--vendor-cleanup"] {
        assert_artifact_option_refused(binary, &run_a, option);
    }
    assert!(!run_a.join("third-party/BUCK").exists());
    assert!(!run_b.join("third-party/BUCK").exists());
    assert_eq!(first.invocation_id, b"run-a");
    assert_eq!(second.invocation_id, b"run-b");
    assert_eq!(first.graph, second.graph);
    assert_eq!(first.rendered_buck, second.rendered_buck);
    assert_ne!(first.receipt_sha256, second.receipt_sha256);
    assert!(!projection.graph().rules().is_empty());
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
    assert_provider_parser_reconciliation(binary, provider.adaptation(), &run_a, &run_b);
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
