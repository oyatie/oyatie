//! Integration scaffold: RP verifier never claims hardware PASS.

use os_attestation_relying_party::{
    confidential_platform_extension_note, AttestationVerdict, CollateralStatus, GuestEvidence,
    RelyingPartyVerifier, StubRelyingPartyVerifier, TeeType, CEDAR_CONTEXT_KEYS,
};

#[test]
fn scaffold_covers_all_tees_as_unknown() {
    let v = StubRelyingPartyVerifier::default();
    for tee in [TeeType::SevSnp, TeeType::Tdx, TeeType::ArmCca] {
        let evidence = GuestEvidence::scaffold_collector(tee, 64);
        let r = v.verify(&evidence, CollateralStatus::Fresh).unwrap();
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
        assert!(!r.hardware_verified);
        assert_eq!(r.tee_type, tee.as_str());
    }
    assert!(!confidential_platform_extension_note().is_empty());
    assert_eq!(CEDAR_CONTEXT_KEYS.len(), 7);
}
