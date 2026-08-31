//! The audit port contract: fail-closed construction, deterministic
//! identity, idempotent re-emission, loud divergence.

use foundry_audit_draft::{
    AuditDisposition, AuditPortError, AuditSink, FoundryAuditEvent, MemoryAuditSink,
};

fn applied(tenant: &str, ordinal: u64) -> FoundryAuditEvent {
    FoundryAuditEvent::new(
        tenant,
        "reading.calibrated",
        "prn_alice",
        "dec_1",
        "ent_r1",
        AuditDisposition::Applied { ordinal },
        1_700_000_000_000,
    )
    .unwrap()
}

#[test]
fn identity_fields_fail_closed() {
    for (bad, field) in [
        (("", "e", "p", "d", "o"), "tenant_id"),
        (("t", " ", "p", "d", "o"), "audit_event_type"),
        (("t", "e", "", "d", "o"), "principal_id"),
        (("t", "e", "p", "", "o"), "decision_id"),
        (("t", "e", "p", "d", ""), "object_ref"),
    ] {
        let (t, e, p, d, o) = bad;
        assert_eq!(
            FoundryAuditEvent::new(t, e, p, d, o, AuditDisposition::Applied { ordinal: 1 }, 1),
            Err(AuditPortError::Empty { field }),
        );
    }
    assert_eq!(
        FoundryAuditEvent::new(
            "t",
            "e",
            "p",
            "d",
            "o",
            AuditDisposition::Denied { gate: " ".into() },
            1,
        ),
        Err(AuditPortError::Empty { field: "gate" }),
    );
}

#[test]
fn event_identity_is_deterministic_and_discriminated() {
    let first = applied("ten_test", 4);
    assert_eq!(first.event_id(), applied("ten_test", 4).event_id());
    assert_ne!(first.event_id(), applied("ten_test", 5).event_id());
    let denied = FoundryAuditEvent::new(
        "ten_test",
        "reading.calibrated",
        "prn_mallory",
        "dec_1",
        "ent_r1",
        AuditDisposition::Denied {
            gate: "authorization".into(),
        },
        1_700_000_000_000,
    )
    .unwrap();
    assert_ne!(first.event_id(), denied.event_id());
}

#[test]
fn re_emission_is_idempotent_and_divergence_is_loud() {
    let mut sink = MemoryAuditSink::default();
    sink.emit(applied("ten_test", 1)).unwrap();
    sink.emit(applied("ten_test", 1)).unwrap();
    assert_eq!(sink.events().len(), 1, "identical re-emission dedups");

    let mut divergent = applied("ten_test", 1);
    divergent.principal_id = "prn_other".into();
    assert!(matches!(
        sink.emit(divergent),
        Err(AuditPortError::Sink { .. })
    ));
    assert_eq!(sink.events().len(), 1);
}
