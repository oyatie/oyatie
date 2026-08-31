use crate::{CloudResourceError, ResourceRegistry, ResourceRepo, ResourceState};

use super::fixture::compute_resource_create;

#[test]
fn registry_rejects_duplicate_resource_id_and_applies_valid_state_transitions() {
    let mut registry = ResourceRegistry::default();
    let resource = registry
        .create(compute_resource_create())
        .expect("first create succeeds");
    assert_eq!(
        registry
            .create(compute_resource_create())
            .expect_err("duplicate resource id denied"),
        CloudResourceError::DuplicateResource
    );

    let running = registry
        .transition_state(&resource.id.value, ResourceState::Running, 1_700_000_010)
        .expect("pending can become running");
    assert_eq!(running.state.value, ResourceState::Running);

    let terminated = registry
        .transition_state(&resource.id.value, ResourceState::Terminated, 1_700_000_020)
        .expect("running can terminate");
    assert_eq!(terminated.state.value, ResourceState::Terminated);

    let error = registry
        .transition_state(&resource.id.value, ResourceState::Running, 1_700_000_030)
        .expect_err("terminated resources are terminal");
    assert_eq!(error, CloudResourceError::InvalidStateTransition);
}
