use port_engine_analysis::w0_ready;

#[test]
fn claims_analysis_readiness() {
    assert!(w0_ready());
}
