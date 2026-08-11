use port_engine_hash::w0_ready;

#[test]
fn slice7_claims_hash_adapter_readiness() {
    assert!(w0_ready());
}
