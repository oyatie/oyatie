use port_engine_emit::w0_ready;

#[test]
fn slice13_claims_emit_adapter_readiness() {
    assert!(w0_ready());
}
