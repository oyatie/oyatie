use port_engine_source_pin::w0_ready;

#[test]
fn slice3_claims_pin_loader_readiness() {
    assert!(w0_ready());
}
