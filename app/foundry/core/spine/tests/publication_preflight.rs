#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[allow(dead_code)]
mod catchup_support;
#[allow(dead_code)]
mod write_through_support;
use catchup_support::{TENANT, mixed_log, registry};
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore, ProjectionStoreError};
use foundry_spine::{
    ProjectionPreflightError as Error, ProjectionState, preflight_projection, project_through,
};

#[test]
fn complete_mixed_prefix_is_read_only_and_matches_the_original_fold() {
    let registry = registry();
    let log = mixed_log();
    let mut store = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &registry);
    project_through(&mut state, &mut store, &log).unwrap();
    let before = format!("{store:?}");
    assert_eq!(
        preflight_projection(TENANT, &registry, &store, &log, 5).unwrap(),
        state
    );
    assert_eq!(before, format!("{store:?}"));
}
#[test]
fn prefix_attribution_divergence_refuses_even_when_the_complete_final_fold_agrees() {
    let registry = registry();
    let mut log = catchup_support::log();
    for (index, entry) in log.iter_mut().enumerate() {
        entry.envelope.object_ref = "ent_overwritten".into();
        entry.receipt.object_sequence = index as u64 + 1;
    }
    let mut held = log.clone();
    let mut record = foundry_edits::decode_action_record(&held[0].envelope.payload).unwrap();
    record.principal_id = "prn_another_first_actor".into();
    held[0].envelope.payload = foundry_edits::encode_action_record(&record);
    let mut store = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &registry);
    project_through(&mut state, &mut store, &held).unwrap();
    assert_eq!(
        state,
        foundry_spine::fold_from_scratch(TENANT, &registry, log.iter()),
        "even every final in-memory field agrees after later entries overwrite the first actor"
    );
    let before = format!("{store:?}");
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &log, 3),
        Err(Error::Entry {
            ordinal: 1,
            error: ProjectionStoreError::DivergentReplay { ordinal: 1 }
        })
    ));
    assert_eq!(before, format!("{store:?}"));
}

#[test]
fn incomplete_foreign_and_non_dense_logs_and_unequal_heads_refuse() {
    let registry = registry();
    let log = mixed_log();
    let store = MemoryProjectionStore::default();
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &log, 5),
        Err(Error::ProjectionHead {
            expected: 5,
            found: 0
        })
    ));
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &log, 6),
        Err(Error::LogHead { .. })
    ));
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &log[1..], 5),
        Err(Error::InvalidLog { .. })
    ));
    let mut foreign = log.clone();
    foreign[0].envelope.tenant_id = "ten_foreign".into();
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &foreign, 5),
        Err(Error::InvalidLog { ordinal: 1 })
    ));
    let mut gap = log.clone();
    gap[1].receipt.ordinal = 3;
    assert!(matches!(
        preflight_projection(TENANT, &registry, &store, &gap, 5),
        Err(Error::InvalidLog { ordinal: 3 })
    ));
    assert!(preflight_projection(TENANT, &registry, &store, &[], 0).is_ok());
}

#[test]
fn unreadable_and_unsupported_store_observation_refuses_without_writes() {
    let registry = registry();
    let log = mixed_log();
    for unreadable in [true, false] {
        let mut inner = MemoryProjectionStore::default();
        let mut state = ProjectionState::new(TENANT, &registry);
        project_through(&mut state, &mut inner, &log).unwrap();
        let store = write_through_support::FailsAt {
            inner,
            fail_on_ordinal: 1,
            fail_head: unreadable,
            fail_poisoned: false,
        };
        let before = format!("{:?}", store.inner);
        let result = preflight_projection(TENANT, &registry, &store, &log, 5);
        if unreadable {
            assert!(matches!(
                result,
                Err(Error::Read(ProjectionStoreError::Storage { .. }))
            ));
        } else {
            assert!(matches!(
                result,
                Err(Error::Entry {
                    ordinal: 1,
                    error: ProjectionStoreError::VerificationUnsupported
                })
            ));
        }
        assert_eq!(before, format!("{:?}", store.inner));
    }
}
#[test]
fn scratch_key_law_refuses_without_mutating_the_live_store() {
    use data_ontology_kernel::{ActionTypeId, OntologyEngine};
    let candidate = registry();
    let mut prior = OntologyEngine::default();
    for entity in candidate.entity_types(TENANT) {
        let mut entity = entity.clone();
        entity.primary_key_property = None;
        prior.register_entity_type(entity).unwrap();
    }
    for link in candidate.link_types(TENANT) {
        prior.register_link_type(link.clone()).unwrap();
    }
    prior
        .register_action_type(
            candidate
                .action_type(TENANT, &ActionTypeId::new("aty_calibrate").unwrap())
                .unwrap()
                .clone(),
        )
        .unwrap();
    let log = vec![
        catchup_support::sealed(1, "same-key"),
        catchup_support::sealed(2, "same-key"),
    ];
    let mut store = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &prior);
    project_through(&mut state, &mut store, &log).unwrap();
    let before = format!("{store:?}");
    assert!(matches!(
        preflight_projection(TENANT, &candidate, &store, &log, 2),
        Err(Error::Scratch {
            ordinal: 2,
            error: ProjectionStoreError::DuplicatePrimaryKey { .. }
        })
    ));
    assert_eq!(before, format!("{store:?}"));
}
#[test]
fn divergence_at_the_last_poison_or_link_also_refuses() {
    let registry = registry();
    let log = mixed_log();
    let mut store = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &registry);
    project_through(&mut state, &mut store, &log).unwrap();
    let before = format!("{store:?}");
    for ordinal in [3, 4, 5] {
        let mut offered = log.clone();
        offered[ordinal - 1] = catchup_support::sealed_at_unknown_revision(ordinal as u64);
        assert!(
            matches!(preflight_projection(TENANT, &registry, &store, &offered, 5), Err(Error::Entry { ordinal: found, error: ProjectionStoreError::DivergentReplay { .. } }) if found == ordinal as u64)
        );
    }
    assert_eq!(before, format!("{store:?}"));
}
