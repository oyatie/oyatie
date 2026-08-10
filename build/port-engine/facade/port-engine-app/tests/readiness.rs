use port_engine_app::w0_ready;

#[test]
fn slice12_claims_driver_readiness() {
    assert!(w0_ready());
}
