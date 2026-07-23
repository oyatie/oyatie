//! Buck-level source-composition regression for the durable v2 target.

const BUCK: &str = include_str!("../BUCK");

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
fn required_ci_dispatches_materialized_and_source_boundary_gates() {
    let workflow =
        std::fs::read_to_string(repo_root().join(".github/workflows/oya-ci-required.yml"))
            .expect("required workflow");
    assert!(workflow.contains("//ci/facade/action-item-accounting:fixuptask-v2-materialized-gate"));
    assert!(workflow.contains("//ci/facade/action-item-accounting:fixuptask-v2-source-boundary"));
}
