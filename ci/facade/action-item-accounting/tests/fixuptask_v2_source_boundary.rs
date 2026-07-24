//! Buck-level source-composition regression for the durable v2 target.

const BUCK: &str = include_str!("../BUCK");
const CARGO: &str = include_str!("../Cargo.toml");

fn target(name: &str) -> &'static str {
    let start = BUCK.find(&format!("name = \"{name}\"")).expect(name);
    let end = BUCK[start..]
        .find("\n)")
        .map(|offset| start + offset)
        .expect("target end");
    &BUCK[start..end]
}

fn repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

#[test]
fn durable_target_compiles_only_durable_sources() {
    let target = target("fixuptask-v2-admission");
    assert!(target.contains("\"src/fixuptask_v2.rs\""));
    assert!(target.contains("\"fixuptask-v2-schema.json\""));
    assert!(!target.contains("glob("));
    assert!(!target.contains("ci-action-item-accounting"));
    assert!(!target.contains("legacy_friction_adapter"));
}

#[test]
fn materialized_gate_runs_the_filesystem_adapter() {
    let target = target("fixuptask-v2-materialized-gate");
    assert!(target.contains("\"tests/fixuptask_v2_admission.rs\""));
    assert!(target.contains("\":ci-action-item-accounting\""));
}

#[test]
fn proposed_v2_targets_are_not_dispatched_by_required_ci() {
    let workflow =
        std::fs::read_to_string(repo_root().join(".github/workflows/oya-ci-required.yml"))
            .expect("required workflow");
    assert!(
        workflow.contains(
            "gate · friction-accounting (ADR-0544, closed-loop friction-ledger accounting)"
        )
    );
    for target in [
        "//ci/facade/action-item-accounting:fixuptask-v2-admission",
        "//ci/facade/action-item-accounting:fixuptask-v2-materialized-gate",
        "//ci/facade/action-item-accounting:fixuptask-v2-source-boundary",
    ] {
        assert!(
            !workflow.contains(target),
            "ADR-0622 remains Proposed, so required CI must not dispatch {target}",
        );
    }
}

#[test]
fn catalog_marks_v2_as_nonbinding_prototype() {
    let catalog = std::fs::read_to_string(repo_root().join("docs/oya-ci/gate-catalog.md"))
        .expect("gate catalog");
    assert!(catalog.contains("`prototype-fixuptask-v2-admission`"));
    assert!(catalog.contains("ADR-0622 Proposed; nonbinding; not dispatched by `oya-ci-required`"));
    assert!(!catalog.contains("`cloud-ci-fixuptask-v2-admission` |"));
}

#[test]
fn durable_cargo_authority_comment_cites_the_successor_adr() {
    assert!(CARGO.contains("# ADR-0622:"));
    assert!(!CARGO.contains("# ADR-0621:"));
}
