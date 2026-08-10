use port_engine_app::w0_ready;

#[test]
fn slice3_claims_driver_readiness() {
    assert!(w0_ready());
}
