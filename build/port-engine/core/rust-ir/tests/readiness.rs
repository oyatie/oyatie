use port_engine_rust_ir::w0_ready;

#[test]
fn slice5_claims_rust_ir_syn_quote_readiness() {
    assert!(w0_ready());
}
