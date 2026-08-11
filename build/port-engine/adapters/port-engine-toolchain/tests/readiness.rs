use port_engine_toolchain::w0_ready;

#[test]
fn slice9_claims_toolchain_adapter_readiness() {
    assert!(w0_ready());
}
