use port_engine_frontend_go::w0_ready;

#[test]
fn slice4_claims_readiness() {
    assert!(w0_ready());
}
