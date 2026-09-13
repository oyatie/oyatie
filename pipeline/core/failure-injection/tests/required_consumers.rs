//! The consumer set is pinned by name: a dropped or renamed consumer fails
//! here instead of shifting every index the other tests read.

use pipeline_failure_injection::REQUIRED_CONSUMERS;

#[test]
fn the_protocol_requires_exactly_these_nine_consumers() {
    #[rustfmt::skip]
    let nine = [
        "compiler", "test", "runtime", "pdp", "slo-controller",
        "reconciler", "cargo", "buck", "ownership",
    ];
    assert_eq!(REQUIRED_CONSUMERS, nine);
}
