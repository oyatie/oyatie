use super::*;

#[test]
fn lifecycle_is_closed_nullable_and_never_calls_an_unobserved_generation_current() {
    let mut lifecycle = ModelLifecycle {
        desired_generation: number(2),
        observed_generation: None,
        phase: ModelLifecyclePhase::Unobserved,
        ready: false,
        scope: ModelObservationScope::RespondingInstance,
    };
    assert_closed(&lifecycle);
    assert_eq!(
        ModelLifecycle::decode(&lifecycle.encode().unwrap()).unwrap(),
        lifecycle
    );
    lifecycle.phase = ModelLifecyclePhase::Current;
    assert!(lifecycle.encode().is_err());
    lifecycle.observed_generation = Some(number(1));
    assert!(lifecycle.encode().is_err());
    lifecycle.phase = ModelLifecyclePhase::Pending;
    assert!(lifecycle.encode().is_ok());
    lifecycle.ready = true;
    assert!(lifecycle.encode().is_err());
    lifecycle.phase = ModelLifecyclePhase::CommitUnknown;
    assert!(lifecycle.encode().is_err());
    lifecycle.ready = false;
    assert!(lifecycle.encode().is_ok());
    lifecycle.phase = ModelLifecyclePhase::Current;
    lifecycle.observed_generation = Some(number(2));
    lifecycle.ready = true;
    let published = ModelPublished {
        receipt: ModelReceiptWire {
            generation: number(2),
            prior_generation: number(1),
            draft_id: number(1),
            draft_version: number(1),
            owner: "usr_author".into(),
        },
        lifecycle,
    };
    assert_closed(&published);
    assert_eq!(
        ModelPublished::decode(&published.encode().unwrap()).unwrap(),
        published
    );
    let limits = model_wire_limits().unwrap();
    assert!(ModelPublished::decode(&vec![b' '; limits.published + 1]).is_err());
    assert!(ModelLifecycle::decode(&vec![b' '; limits.lifecycle + 1]).is_err());
}
