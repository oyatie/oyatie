use shared_platform_contracts_kernel::pdp::PolicyVersion;

use crate::PdpError;

#[test]
fn pdp_error_messages_are_legible() {
    let e = PdpError::StalePolicyVersion {
        required: PolicyVersion::new("psv-2").unwrap(),
        loaded: PolicyVersion::new("psv-1").unwrap(),
    };
    assert_eq!(
        e.to_string(),
        "policy bundle too stale: caller pinned psv-2 but loaded version is psv-1"
    );
}
