use crate::ResourceState;

#[test]
fn resource_state_as_str_and_parse_round_trip() {
    let variants = [
        ResourceState::Pending,
        ResourceState::Running,
        ResourceState::Stopped,
        ResourceState::Terminated,
        ResourceState::Error,
    ];
    for state in variants {
        assert_eq!(
            ResourceState::parse(state.as_str()),
            Some(state),
            "parse(as_str({state:?})) must round-trip"
        );
    }
}

#[test]
fn resource_state_parse_rejects_unknown_inputs() {
    assert_eq!(ResourceState::parse("bogus"), None);
    assert_eq!(ResourceState::parse(""), None);
    assert_eq!(ResourceState::parse("RUNNING"), None);
    assert_eq!(ResourceState::parse("Pending"), None);
}

#[test]
fn resource_state_classifiers_match_only_correct_variant() {
    assert!(ResourceState::Running.is_active());
    assert!(!ResourceState::Pending.is_active());
    assert!(!ResourceState::Stopped.is_active());
    assert!(!ResourceState::Terminated.is_active());
    assert!(!ResourceState::Error.is_active());

    assert!(ResourceState::Stopped.is_quiescent());
    assert!(!ResourceState::Pending.is_quiescent());
    assert!(!ResourceState::Running.is_quiescent());
    assert!(!ResourceState::Terminated.is_quiescent());
    assert!(!ResourceState::Error.is_quiescent());
}

#[test]
fn terminated_allowed_next_contains_only_self_loop() {
    let nexts = ResourceState::Terminated.allowed_next();
    assert_eq!(nexts, &[ResourceState::Terminated]);
}

#[test]
fn transition_graph_allowed_next_agrees_with_can_transition_to_for_all_pairs() {
    let all_states = [
        ResourceState::Pending,
        ResourceState::Running,
        ResourceState::Stopped,
        ResourceState::Terminated,
        ResourceState::Error,
    ];
    for &from in &all_states {
        let nexts = from.allowed_next();
        for &to in &all_states {
            let via_predicate = from.can_transition_to(to);
            let via_graph = nexts.contains(&to);
            assert_eq!(
                via_predicate, via_graph,
                "allowed_next({from:?}) and can_transition_to({from:?}, {to:?}) disagree: \
                     predicate={via_predicate}, graph={via_graph}"
            );
        }
    }
}
