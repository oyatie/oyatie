//! The refusal taxonomy is the spine's outward contract: gate order and
//! labels are frozen here, so any widening is an explicit reviewed diff.

use foundry_spine::{RefusalGate, Refused};

#[test]
fn gate_labels_are_frozen() {
    assert_eq!(RefusalGate::Authorization.label(), "authorization");
    assert_eq!(RefusalGate::Parameters.label(), "parameters");
    assert_eq!(RefusalGate::Admission.label(), "admission");
}

#[test]
fn gates_order_by_check_sequence() {
    assert!(RefusalGate::Authorization < RefusalGate::Parameters);
    assert!(RefusalGate::Parameters < RefusalGate::Admission);
}

#[test]
fn refusals_carry_gate_and_static_cause() {
    let refused = Refused {
        gate: RefusalGate::Admission,
        cause: "reserved edit kind",
    };
    assert_eq!(refused.gate.label(), "admission");
    assert_eq!(refused.cause, "reserved edit kind");
}
