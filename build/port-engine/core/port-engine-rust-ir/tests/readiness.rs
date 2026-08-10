use port_engine_rust_ir::w0_ready;

#[test]
fn slice3_claims_rust_ir_stub_readiness() {
    assert!(w0_ready());
}
