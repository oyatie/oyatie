use port_engine_identity::w0_ready;

#[test]
fn slice9_claims_identity_adapter_readiness() {
    assert!(w0_ready());
}
