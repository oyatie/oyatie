use shared_platform_contracts_kernel::ContractViolation;

use super::slice;

#[test]
fn entity_slice_rejects_duplicate_uids() {
    let mut s = slice();
    let dup = s.entities[0].clone();
    s.entities.push(dup);
    let violations = s.validate().unwrap_err();
    assert!(matches!(
        violations.as_slice(),
        [ContractViolation::BrokenReference { .. }]
    ));
}
