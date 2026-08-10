use port_engine_rulepack::w0_ready;

#[test]
fn slice7_claims_rulepack_adapter_readiness() {
    assert!(w0_ready());
}
