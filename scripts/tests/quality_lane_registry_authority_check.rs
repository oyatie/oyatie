#[allow(dead_code)]
#[path = "../ci/assert-quality-lane-registry-authority.rs"]
mod check;

fn minimal_registry(id: &str, status: &str, purpose: &str, command: Option<&str>) -> String {
    let command_line = command
        .map(|value| format!("\n    check_command: {value}"))
        .unwrap_or_default();
    format!(
        r#"# test fixture
command_authority_policy: active check_command rows must use Buck2/Prow native targets only; retired local CLI, raw Cargo, npm/pnpm, and script rows stay planned until native targets exist.
lanes:
  - id: {id}
    stage: per-pr
    status: {status}
    owner_team: axis-foundry
    purpose: {purpose}
    source: TEST
    runtime_budget_seconds: 60{command_line}
"#
    )
}

fn minimal_doc(id: &str, purpose: &str) -> String {
    format!(
        r#"# test

### 1.2 Per-PR gates

| Lane | Purpose |
|---|---|
| `{id}` | {purpose} |
"#
    )
}

#[test]
fn accepts_active_buck2_authority_with_doc_mirror() {
    let registry = minimal_registry(
        "buck2-build",
        "active",
        "Buck2 build graph remains the merge authority",
        Some("buck2 build //..."),
    );
    let doc = minimal_doc(
        "buck2-build",
        "Buck2 build graph remains the merge authority",
    );

    let report = check::evaluate_contents(&registry, &doc);

    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert_eq!(report.active_lanes, 1);
    assert_eq!(report.active_buck2_commands, 1);
}

#[test]
fn rejects_active_retired_cli_or_raw_tool_command() {
    let retired_command = format!(
        "{}{} -- {} validate workspace-hygiene",
        "oya-dev", "-cli", "gate"
    );
    let registry = minimal_registry(
        "workspace-hygiene",
        "active",
        "workspace hygiene stays native",
        Some(&retired_command),
    );
    let doc = minimal_doc("workspace-hygiene", "workspace hygiene stays native");

    let report = check::evaluate_contents(&registry, &doc);

    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("must use Buck2/Prow check authority")),
        "{:?}",
        report.failures
    );
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("oya-dev-cli")),
        "{:?}",
        report.failures
    );
}

#[test]
fn rejects_planned_lane_with_command_claim() {
    let registry = minimal_registry(
        "buck2-unused-deps",
        "planned",
        "unused dependency policy awaits a native Buck2 target",
        Some("buck2 build //:future-unused-deps-check"),
    );
    let doc = minimal_doc(
        "buck2-unused-deps",
        "unused dependency policy awaits a native Buck2 target",
    );

    let report = check::evaluate_contents(&registry, &doc);

    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("planned lane buck2-unused-deps must not carry")),
        "{:?}",
        report.failures
    );
}

#[test]
fn rejects_missing_policy_preamble() {
    let registry = minimal_registry(
        "buck2-test",
        "active",
        "Buck2 test graph remains the merge authority",
        Some("buck2 test //..."),
    )
    .replace("command_authority_policy:", "retired_policy:");
    let doc = minimal_doc("buck2-test", "Buck2 test graph remains the merge authority");

    let report = check::evaluate_contents(&registry, &doc);

    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("missing command_authority_policy")),
        "{:?}",
        report.failures
    );
}

#[test]
fn rejects_doc_purpose_drift() {
    let registry = minimal_registry(
        "quality-lanes",
        "active",
        "quality lane registry remains mirrored",
        Some("buck2 build //:quality-lane-registry-authority-check"),
    );
    let doc = minimal_doc("quality-lanes", "different words");

    let report = check::evaluate_contents(&registry, &doc);

    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("purpose drift")),
        "{:?}",
        report.failures
    );
}

#[test]
fn rejects_active_prose_that_mentions_retired_authority() {
    let registry = minimal_registry(
        "artifact-contract",
        "active",
        "artifact contract invoked from `oya gate run-all`",
        Some(
            "buck2 build //libs/oya-check-active-artifact-contract:oya-check-active-artifact-contract",
        ),
    );
    let doc = minimal_doc(
        "artifact-contract",
        "artifact contract invoked from `oya gate run-all`",
    );

    let report = check::evaluate_contents(&registry, &doc);

    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("invoked from `oya gate run-all`")),
        "{:?}",
        report.failures
    );
}
