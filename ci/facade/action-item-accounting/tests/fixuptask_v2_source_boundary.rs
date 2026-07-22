//! Buck-level source-composition regression for the durable v2 target.

const BUCK: &str = include_str!("../BUCK");

#[test]
fn durable_target_compiles_only_durable_sources() {
    let start = BUCK
        .find("name = \"fixuptask-v2-admission\"")
        .expect("durable target");
    let end = BUCK[start..]
        .find("\n)")
        .map(|offset| start + offset)
        .expect("target end");
    let target = &BUCK[start..end];
    assert!(target.contains("\"src/fixuptask_v2.rs\""));
    assert!(target.contains("\"fixuptask-v2-schema.json\""));
    assert!(!target.contains("glob("));
    assert!(!target.contains("ci-action-item-accounting"));
    assert!(!target.contains("legacy_friction_adapter"));
}
