use port_engine_rulepack::w0_ready;

#[test]
fn slice10_claims_fixture_gated_rulepack_readiness() {
    assert!(w0_ready());
}
