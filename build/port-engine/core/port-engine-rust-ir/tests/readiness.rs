use port_engine_rust_ir::w0_ready;

#[test]
fn slice1_does_not_claim_readiness() {
    assert!(!w0_ready());
}
