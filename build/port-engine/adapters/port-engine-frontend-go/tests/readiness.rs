use port_engine_frontend_go::w0_ready;

#[test]
fn slice1_does_not_claim_readiness() {
    assert!(!w0_ready());
}
