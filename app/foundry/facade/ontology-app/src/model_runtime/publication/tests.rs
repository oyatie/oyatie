#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "faults.rs"]
mod faults;
#[path = "preflight_tests.rs"]
mod preflight_tests;
#[path = "served_mutation_recovery.rs"]
mod served_mutation_recovery;
#[path = "served_recovery.rs"]
mod served_recovery;
#[path = "support.rs"]
mod support;
use super::*;
use faults::Fault;
use support::*;

#[test]
fn real_store_commit_failures_resolve_from_exact_receipt_and_head() {
    for (fault, committed) in [
        (Fault::RejectCommit, false),
        (Fault::CommittedError, true),
        (Fault::WrongAcknowledgement, true),
    ] {
        let files = Files::new();
        let (mut app, _) = files.compose(fault);
        let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
        let saved = draft(tenant);
        let staged =
            StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).unwrap();
        let result = staged.commit(tenant, &caller());
        assert_eq!(result.is_ok(), committed);
        assert_eq!(
            tenant.observed_model_generation(),
            Some(if committed { 2 } else { 1 })
        );
        assert!(tenant.publication_unknown.is_none());
        assert!(tenant.sync_status(TENANT).is_ok());
        assert_eq!(
            tenant
                .model_store
                .as_ref()
                .unwrap()
                .store
                .head(&caller())
                .unwrap()
                .generation,
            if committed { 2 } else { 1 }
        );
    }
}

#[test]
fn ambiguous_outcome_retains_truthful_observation_and_freezes_native_writes_until_recovery() {
    for (fault, committed) in [
        (Fault::CommitAndLoseReadback, true),
        (Fault::RejectAndLoseReadback, false),
    ] {
        let files = Files::new();
        let (mut app, fault) = files.compose(fault);
        let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
        let saved = draft(tenant);
        let before = tenant.projection.clone();
        let staged =
            StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).unwrap();
        assert!(staged.commit(tenant, &caller()).is_err());
        assert_eq!(tenant.projection, before);
        assert_eq!(tenant.observed_model_generation(), Some(1));
        assert!(tenant.publication_unknown.is_some());
        assert!(matches!(
            tenant.write_handles(),
            Err(crate::composition::WriteHandlesRefused::PublicationUnknown)
        ));
        assert!(tenant.sync_status(TENANT).is_err());
        assert!(StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).is_err());
        assert!(crate::model_runtime::lifecycle(tenant, &caller()).is_err());
        *fault.lock().unwrap() = Fault::None;
        let lifecycle = crate::model_runtime::lifecycle(tenant, &caller()).unwrap();
        assert_eq!(lifecycle.phase, ModelLifecyclePhase::CommitUnknown);
        assert_eq!(lifecycle.observed_generation.unwrap().get(), 1);
        assert!(!lifecycle.ready);
        assert_eq!(
            lifecycle.desired_generation.get(),
            if committed { 2 } else { 1 }
        );
        let receipt = tenant
            .model_store
            .as_ref()
            .unwrap()
            .store
            .receipt(&caller(), saved.id, saved.version, saved.base_generation)
            .unwrap();
        let recovered = StagedRecovery::prepare(tenant, &caller(), receipt.as_ref()).unwrap();
        assert_eq!(recovered.install(tenant).is_some(), committed);
        assert_eq!(
            tenant.observed_model_generation(),
            Some(if committed { 2 } else { 1 })
        );
        assert!(tenant.publication_unknown.is_none());
        assert!(tenant.write_handles().is_ok());
    }
}

#[test]
fn contradictory_receipt_and_head_never_install_the_prepared_model() {
    let files = Files::new();
    let (mut app, fault) = files.compose(Fault::ContradictoryHead);
    let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
    let saved = draft(tenant);
    let staged = StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).unwrap();
    assert!(staged.commit(tenant, &caller()).is_err());
    assert!(tenant.publication_unknown.is_some());
    assert_eq!(tenant.observed_model_generation(), Some(1));
    assert!(tenant.write_handles().is_err());
    *fault.lock().unwrap() = Fault::None;
    let receipt = tenant
        .model_store
        .as_ref()
        .unwrap()
        .store
        .receipt(&caller(), saved.id, saved.version, saved.base_generation)
        .unwrap();
    StagedRecovery::prepare(tenant, &caller(), receipt.as_ref())
        .unwrap()
        .install(tenant);
    assert_eq!(tenant.observed_model_generation(), Some(2));
}

const CRASH_DIRECTORY: &str = "OYATIE_PUBLICATION_CRASH_TEST_DIRECTORY";
#[test]
fn desired_head_can_be_pending_or_unreadable_without_inventing_installed_observation_or_readiness()
{
    for fault in [Fault::None, Fault::CommitAndLoseReadback] {
        let files = Files::new();
        let (mut app, _) = files.compose(fault);
        let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
        let saved = draft(tenant);
        let before = tenant.projection.clone();
        let model = tenant.model_store.as_mut().unwrap();
        let prepared = model
            .store
            .prepare(&caller(), saved.id, saved.version, saved.base_generation)
            .unwrap();
        let result = model.store.publish(&caller(), &prepared);
        assert_eq!(result.is_ok(), fault == Fault::None);
        assert_eq!(tenant.observed_model_generation(), Some(1));
        assert!(tenant.sync_status(TENANT).is_err());
        let lifecycle = crate::model_runtime::lifecycle(tenant, &caller());
        if fault == Fault::None {
            let lifecycle = lifecycle.unwrap();
            assert_eq!(lifecycle.phase, ModelLifecyclePhase::Pending);
            assert_eq!(lifecycle.observed_generation.unwrap().get(), 1);
            assert_eq!(lifecycle.desired_generation.get(), 2);
            assert!(!lifecycle.ready);
        } else {
            assert!(lifecycle.is_err());
        }
        assert_eq!(tenant.projection, before);
        assert!(!crate::observation::observe(&app).is_caught_up());
    }
}

#[test]
fn crash_child() {
    let Some(directory) = std::env::var_os(CRASH_DIRECTORY) else {
        return;
    };
    let files = Files::existing(directory.into());
    let (mut app, _) = files.compose(Fault::CrashAfterCommit);
    let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
    let saved = draft(tenant);
    let staged = StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).unwrap();
    let _ = staged.commit(tenant, &caller());
    panic!("the child must exit from the real commit before installation");
}

#[test]
fn process_death_after_real_commit_recovers_current_model_before_observation() {
    let files = Files::new();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "model_runtime::publication::tests::crash_child",
            "--nocapture",
        ])
        .env(CRASH_DIRECTORY, &files.directory)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(72));
    let (mut app, _) = files.compose(Fault::None);
    let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
    assert_eq!(tenant.observed_model_generation(), Some(2));
    assert!(tenant.sync_status(TENANT).is_ok());
    assert!(
        tenant
            .projection
            .registry_input
            .action_type(
                TENANT,
                &data_ontology_kernel::ActionTypeId::new("aty_record_write").unwrap()
            )
            .is_some()
    );
    assert_eq!(
        tenant
            .projection
            .registry_input
            .entity_types(TENANT)
            .next()
            .unwrap()
            .revision,
        2
    );
    assert_eq!(
        crate::model_runtime::lifecycle(tenant, &caller())
            .unwrap()
            .phase,
        ModelLifecyclePhase::Current
    );
}
