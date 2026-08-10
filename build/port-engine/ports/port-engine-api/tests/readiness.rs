use port_engine_api::w0_ready;

#[test]
fn slice1_does_not_claim_readiness() {
    assert!(!w0_ready());
}
