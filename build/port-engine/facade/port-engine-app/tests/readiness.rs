use port_engine_app::w0_ready;

#[test]
fn slice11_claims_driver_readiness() {
    assert!(w0_ready());
}
