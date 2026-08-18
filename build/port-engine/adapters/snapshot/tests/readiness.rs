use port_engine_snapshot::w0_ready;

#[test]
fn slice8_claims_snapshot_adapter_readiness() {
    assert!(w0_ready());
}
