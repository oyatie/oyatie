use secrets_kms_domain::{CloudKmsDirectory, KmsKeyState, KmsKeyVersionLifecycleState};
use secrets_kms_operator_k8s::{
    AdapterError, DomainKmsOperatorActuator, ExponentialBackoff, InMemoryObservedStateProvider,
    KMS_KEY_RING_PLURAL, KMS_SEALING_ROOT_PLURAL, KmsOperatorActuator, KmsStatusPatchKind,
    ProjectedKmsObject, crd_manifests, desired_state_from_observed, key_ring_api_resource,
    project_observed_state, reconcile_observation_failure, run_reconcile_cycle,
    sealing_root_api_resource, status_patches_for_actions,
};
use secrets_kms_operator_kernel::{
    Action, Clock, DataClassLabel, DesiredState, HsmValidation, KeyOrigin, KeyRing, KeyUsage,
    KeyVersionRotationPolicy, ObservedState, ResidencyMode, SealingRoot,
};
use serde_json::json;

#[derive(Clone, Copy)]
struct FixedClock {
    now: u64,
}

impl Clock for FixedClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

#[derive(Default)]
struct RecordingActuator {
    actions: Vec<Action>,
}

impl KmsOperatorActuator for RecordingActuator {
    fn execute(&mut self, action: &Action) -> Result<(), AdapterError> {
        self.actions.push(action.clone());
        Ok(())
    }
}

#[test]
fn exposes_key_ring_and_sealing_root_crd_manifests() {
    let manifests = crd_manifests();

    assert_eq!(manifests.len(), 2);
    assert!(manifests[0].contains("kind: CustomResourceDefinition"));
    assert!(manifests[0].contains("kmskeyrings.kms.oyatie.com"));
    assert!(manifests[1].contains("kmssealingroots.kms.oyatie.com"));
}

#[test]
fn dynamic_api_resources_use_explicit_crd_plurals() {
    let key_ring = key_ring_api_resource();
    let sealing_root = sealing_root_api_resource();

    assert_eq!(key_ring.plural, KMS_KEY_RING_PLURAL);
    assert_eq!(key_ring.kind, "KmsKeyRing");
    assert_eq!(sealing_root.plural, KMS_SEALING_ROOT_PLURAL);
    assert_eq!(sealing_root.kind, "KmsSealingRoot");
}

#[test]
fn projects_crd_json_into_kernel_observed_state() {
    let objects = vec![
        ProjectedKmsObject::from_json(json!({
            "kind": "KmsSealingRoot",
            "metadata": {"name": "tenant-a-root"},
            "spec": {
                "tenantId": "ten_alpha",
                "region": "us-east-1",
                "cellId": "cell-us-east-1-a",
                "rootRef": "sealing-root/tenant-a",
                "activeVersion": 1,
                "rotateAfterSeconds": 86400
            },
            "status": {
                "observedVersion": 1,
                "health": {"state": "Healthy"}
            }
        })),
        ProjectedKmsObject::from_json(json!({
            "kind": "KmsKeyRing",
            "metadata": {"name": "tenant-a-ring"},
            "spec": {
                "tenantId": "ten_alpha",
                "region": "region-home",
                "cellId": "cell-region-home-a-001",
                "hsmPartitionRef": "hsm/region-home/cell-region-home-a-001",
                "origin": "OyatieManaged",
                "usage": "EncryptDecrypt",
                "hsmValidation": "PackEnhancedFips1403Level3",
                "residency": "StrictHomeRegion",
                "dataClass": "InternalOnly",
                "rotationPolicy": {
                    "rotateAfterSeconds": 900,
                    "decryptOnlyGraceSeconds": 300
                }
            },
            "status": {
                "health": {"state": "Healthy"},
                "versions": [{
                    "version": 1,
                    "state": "Active",
                    "createdAtEpochSeconds": 100,
                    "activatedAtEpochSeconds": 100
                }]
            }
        })),
    ];

    let observed = project_observed_state(&objects).expect("projection should parse");

    assert_eq!(observed.sealing_roots.len(), 1);
    assert_eq!(observed.key_rings.len(), 1);
    assert_eq!(observed.key_rings[0].desired.name, "tenant-a-ring");
    assert_eq!(observed.key_rings[0].versions[0].version, 1);
}

#[test]
fn key_ring_missing_status_reconciles_as_not_yet_created() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json()
    }))];

    let observed = project_observed_state(&objects).expect("fresh CR without status is desired");

    assert_eq!(observed.key_rings.len(), 1);
    assert_eq!(observed.key_rings[0].versions, Vec::new());
}

#[test]
fn key_ring_status_missing_versions_fails_closed() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": {
            "health": {"state": "Healthy"}
        }
    }))];

    let error = project_observed_state(&objects).expect_err("partial status must fail closed");

    assert_eq!(
        error,
        AdapterError::PartialObservedState(
            "KmsKeyRing tenant-a-ring status.versions is missing".to_owned()
        )
    );
}

#[test]
fn key_ring_status_missing_health_fails_closed() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": {
            "versions": []
        }
    }))];

    let error = project_observed_state(&objects).expect_err("partial status must fail closed");

    assert_eq!(
        error,
        AdapterError::PartialObservedState(
            "KmsKeyRing tenant-a-ring status.health is missing".to_owned()
        )
    );
}

#[test]
fn sealing_root_status_missing_observed_version_fails_closed() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsSealingRoot",
        "metadata": {"name": "tenant-a-root"},
        "spec": sealing_root_spec_json(),
        "status": {
            "health": {"state": "Healthy"}
        }
    }))];

    let error = project_observed_state(&objects).expect_err("partial status must fail closed");

    assert_eq!(
        error,
        AdapterError::PartialObservedState(
            "KmsSealingRoot tenant-a-root status.observedVersion is missing".to_owned()
        )
    );
}

#[test]
fn unknown_health_state_is_invalid_crd_object() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": {
            "health": {"state": "MaybeHealthy"},
            "versions": []
        }
    }))];

    let error = project_observed_state(&objects).expect_err("unknown health is invalid");

    assert!(matches!(error, AdapterError::InvalidCrdObject(_)));
}

#[test]
fn partial_observed_state_fails_closed_without_side_effects() {
    let provider = InMemoryObservedStateProvider::partial("watch list timed out");
    let desired = desired_state();
    let mut actuator = RecordingActuator::default();

    let result = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 1_000 },
        &mut actuator,
    );

    let failure = result.expect_err("partial observed state must fail closed");
    assert_eq!(
        failure.error,
        AdapterError::PartialObservedState("watch list timed out".to_owned())
    );
    assert_eq!(failure.wide_event.status, "failed");
    assert_eq!(
        failure.wide_event.error_class,
        Some("partial_observed_state".to_owned())
    );
    assert_eq!(actuator.actions, Vec::<Action>::new());
}

#[test]
fn observation_failure_builds_one_failed_wide_event() {
    let failure = reconcile_observation_failure(AdapterError::PartialObservedState(
        "watch list timed out".to_owned(),
    ));

    assert_eq!(failure.planned_actions, 0);
    assert_eq!(failure.executed_actions, 0);
    assert_eq!(failure.wide_event.status, "failed");
    assert_eq!(failure.wide_event.action_count, 0);
    assert_eq!(
        failure.wide_event.error_class,
        Some("partial_observed_state".to_owned())
    );
}

#[test]
fn desired_state_is_derived_from_crd_specs() {
    let objects = vec![
        ProjectedKmsObject::from_json(json!({
            "kind": "KmsSealingRoot",
            "metadata": {"name": "tenant-a-root"},
            "spec": sealing_root_spec_json()
        })),
        ProjectedKmsObject::from_json(json!({
            "kind": "KmsKeyRing",
            "metadata": {"name": "tenant-a-ring"},
            "spec": key_ring_spec_json()
        })),
    ];
    let observed = project_observed_state(&objects).expect("fresh CR specs are desired state");

    let desired = desired_state_from_observed(&observed);

    assert_eq!(
        desired.key_rings,
        vec![desired_state().key_rings[0].clone()]
    );
    assert_eq!(
        desired.sealing_roots,
        vec![desired_state().sealing_roots[0].clone()]
    );
}

#[test]
fn domain_actuator_uses_non_default_crd_spec_for_supported_key_ring_actions() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-b-ring"},
        "spec": {
            "tenantId": "ten_beta",
            "region": "region-home",
            "cellId": "cell-region-home-b-001",
            "hsmPartitionRef": "hsm/region-home/cell-region-home-b-001",
            "origin": "OyatieManaged",
            "usage": "EncryptDecrypt",
            "hsmValidation": "PackEnhancedFips1403Level3",
            "residency": "StrictHomeRegion",
            "dataClass": "PiiIdentifying",
            "rotationPolicy": {
                "rotateAfterSeconds": 7776000,
                "decryptOnlyGraceSeconds": 300
            }
        }
    }))];
    let observed = project_observed_state(&objects).expect("fresh non-default CR should parse");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let desired = actuator.desired_state_for_observed(&observed);

    assert_eq!(desired.sealing_roots, Vec::new());
    assert_eq!(desired.key_rings[0].tenant_id, "ten_beta");
    assert_eq!(desired.key_rings[0].cell_id, "cell-region-home-b-001");

    let report = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 1_000 },
        &mut actuator,
    )
    .expect("supported key-ring action should use the domain repo");

    assert_eq!(report.executed_actions, 1);
    let repo = actuator.into_inner();
    let key = repo.keys().next().expect("domain repo has created key");
    assert_eq!(key.created_at_epoch_seconds.value, 1_000);
    assert_eq!(key.updated_at_epoch_seconds.value, 1_000);
}

#[test]
fn domain_actuator_includes_sealing_root_crds_and_actuates_through_domain() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsSealingRoot",
        "metadata": {"name": "tenant-a-root"},
        "spec": sealing_root_spec_json()
    }))];
    let observed = project_observed_state(&objects).expect("fresh sealing-root CR should parse");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let desired = actuator.desired_state_for_observed(&observed);

    assert_eq!(desired.key_rings, Vec::new());
    assert_eq!(
        desired.sealing_roots,
        vec![desired_state().sealing_roots[0].clone()]
    );

    let report = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 1_000 },
        &mut actuator,
    )
    .expect("sealing-root creation should actuate through the domain");

    assert_eq!(report.planned_actions, 1);
    assert_eq!(report.executed_actions, 1);
    let repo = actuator.into_inner();
    assert_eq!(repo.sealing_roots().count(), 1);
}

#[test]
fn domain_actuator_repairs_stale_sealing_root_status_without_duplicate_root() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsSealingRoot",
        "metadata": {"name": "tenant-a-root"},
        "spec": sealing_root_spec_json()
    }))];
    let observed = project_observed_state(&objects).expect("fresh sealing-root CR should parse");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let desired = actuator.desired_state_for_observed(&observed);

    let first = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 2_000 },
        &mut actuator,
    )
    .expect("fresh sealing root should be created in the domain");
    let first_patches = actuator
        .status_patches_for_actions(&observed, &first.actions)
        .expect("created sealing root should project to a status patch");
    actuator.remember_status_patches(&first.actions, &first_patches);

    let second = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 2_001 },
        &mut actuator,
    )
    .expect("stale retry should repair status without a duplicate sealing root");
    let second_patches = actuator
        .status_patches_for_actions(&observed, &second.actions)
        .expect("remembered sealing-root status should be reusable on retry");

    assert_eq!(second_patches, first_patches);
    let repo = actuator.into_inner();
    assert_eq!(repo.sealing_roots().count(), 1);
}

#[test]
fn fresh_key_ring_status_patch_makes_next_cycle_converge() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json()
    }))];
    let observed = project_observed_state(&objects).expect("fresh CR without status is desired");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let desired = desired_state_from_observed(&observed);
    let mut actuator = RecordingActuator::default();

    let report = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 2_000 },
        &mut actuator,
    )
    .expect("fresh key ring should be created");
    let patches = status_patches_for_actions(&observed, &report.actions)
        .expect("create action should build status patch");

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].kind, KmsStatusPatchKind::KeyRing);
    assert_eq!(
        patches[0].status["versions"][0]["createdAtEpochSeconds"],
        2_000
    );

    let patched_observed = project_observed_state(&[ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": patches[0].status.clone()
    }))])
    .expect("patched status should be complete");
    let patched_provider = InMemoryObservedStateProvider::complete(patched_observed.clone());
    let patched_desired = desired_state_from_observed(&patched_observed);
    let mut second_actuator = RecordingActuator::default();
    let second = run_reconcile_cycle(
        &patched_provider,
        &patched_desired,
        &FixedClock { now: 2_001 },
        &mut second_actuator,
    )
    .expect("second cycle should converge");

    assert_eq!(second.planned_actions, 0);
    assert_eq!(second_actuator.actions, Vec::<Action>::new());
}

#[test]
fn domain_actuator_repairs_stale_create_status_without_duplicate_key() {
    let objects = vec![ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-b-ring"},
        "spec": domain_key_ring_spec_json()
    }))];
    let observed = project_observed_state(&objects).expect("fresh CR without status is desired");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let desired = actuator.desired_state_for_observed(&observed);

    let first = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 2_000 },
        &mut actuator,
    )
    .expect("fresh key ring should be created in the domain");
    let first_patches = actuator
        .status_patches_for_actions(&observed, &first.actions)
        .expect("created domain key should project to a status patch");
    actuator.remember_status_patches(&first.actions, &first_patches);

    let second = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 2_001 },
        &mut actuator,
    )
    .expect("stale retry should repair status without a duplicate create");
    let second_patches = actuator
        .status_patches_for_actions(&observed, &second.actions)
        .expect("remembered create status should be reusable on retry");

    assert_eq!(second_patches, first_patches);
    let repo = actuator.into_inner();
    let keys: Vec<_> = repo.keys().collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].current_version.value, 1);
    assert_eq!(keys[0].created_at_epoch_seconds.value, 2_000);
}

#[test]
fn rotation_status_patch_demotes_previous_active_version() {
    let observed = project_observed_state(&[ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": {
            "health": {"state": "Healthy"},
            "versions": [{
                "version": 1,
                "state": "Active",
                "createdAtEpochSeconds": 100,
                "activatedAtEpochSeconds": 100
            }]
        }
    }))])
    .expect("complete active version should parse");
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let desired = desired_state_from_observed(&observed);
    let mut actuator = RecordingActuator::default();

    let report = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 1_200 },
        &mut actuator,
    )
    .expect("expired active version should rotate");
    let patches = status_patches_for_actions(&observed, &report.actions)
        .expect("rotate action should build status patch");

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].status["versions"][0]["state"], "DecryptOnly");
    assert_eq!(
        patches[0].status["versions"][0]["decryptOnlySinceEpochSeconds"],
        1_200
    );
    assert_eq!(patches[0].status["versions"][1]["version"], 2);
    assert_eq!(patches[0].status["versions"][1]["state"], "Active");
}

#[test]
fn domain_actuator_repairs_stale_rotation_status_without_duplicate_rotation() {
    let fresh_observed = project_observed_state(&[ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-b-ring"},
        "spec": domain_key_ring_spec_json()
    }))])
    .expect("fresh CR without status should parse");
    let fresh_provider = InMemoryObservedStateProvider::complete(fresh_observed.clone());
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let fresh_desired = actuator.desired_state_for_observed(&fresh_observed);
    let create_report = run_reconcile_cycle(
        &fresh_provider,
        &fresh_desired,
        &FixedClock { now: 100 },
        &mut actuator,
    )
    .expect("fresh key ring should be created before rotation");
    let create_patches = actuator
        .status_patches_for_actions(&fresh_observed, &create_report.actions)
        .expect("create should project a status patch");
    actuator.remember_status_patches(&create_report.actions, &create_patches);

    let stale_observed = project_observed_state(&[ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-b-ring"},
        "spec": domain_key_ring_spec_json(),
        "status": {
            "health": {"state": "Healthy"},
            "versions": [{
                "version": 1,
                "state": "Active",
                "createdAtEpochSeconds": 100,
                "activatedAtEpochSeconds": 100
            }]
        }
    }))])
    .expect("stale active version status should parse");
    let stale_provider = InMemoryObservedStateProvider::complete(stale_observed.clone());
    let stale_desired = actuator.desired_state_for_observed(&stale_observed);
    let first_rotation = run_reconcile_cycle(
        &stale_provider,
        &stale_desired,
        &FixedClock { now: 1_200 },
        &mut actuator,
    )
    .expect("expired active key should rotate once");
    let first_rotation_patches = actuator
        .status_patches_for_actions(&stale_observed, &first_rotation.actions)
        .expect("rotation should project a status patch");
    actuator.remember_status_patches(&first_rotation.actions, &first_rotation_patches);

    let retry_rotation = run_reconcile_cycle(
        &stale_provider,
        &stale_desired,
        &FixedClock { now: 1_300 },
        &mut actuator,
    )
    .expect("stale retry should repair status without a second rotation");
    let retry_rotation_patches = actuator
        .status_patches_for_actions(&stale_observed, &retry_rotation.actions)
        .expect("remembered rotation status should be reusable on retry");

    assert_eq!(retry_rotation_patches, first_rotation_patches);
    assert_eq!(
        retry_rotation_patches[0].status["versions"][1]["createdAtEpochSeconds"],
        1_200
    );
    let repo = actuator.into_inner();
    let keys: Vec<_> = repo.keys().collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].current_version.value, 2);
    assert_eq!(keys[0].updated_at_epoch_seconds.value, 1_200);
}

#[test]
fn domain_actuator_executes_sealing_root_demote_and_quarantine_actions() {
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    actuator
        .execute(&Action::CreateSealingRoot {
            sealing_root: desired_state()
                .sealing_roots
                .first()
                .expect("fixture has a sealing root")
                .clone(),
        })
        .expect("sealing-root mutation should use the domain lifecycle port");
    actuator
        .execute(&Action::CreateKeyRing {
            key_ring: desired_state()
                .key_rings
                .first()
                .expect("fixture has a key ring")
                .clone(),
            requested_at_epoch_seconds: 100,
        })
        .expect("key-ring creation should seed version lifecycle state");
    actuator
        .execute(&Action::RotateKeyVersion {
            key_ring: desired_state()
                .key_rings
                .first()
                .expect("fixture has a key ring")
                .clone(),
            observed_active_version: 1,
            reason: "test rotation".to_owned(),
            requested_at_epoch_seconds: 1_200,
        })
        .expect("rotation should use the domain port");
    actuator
        .execute(&Action::DemoteKeyVersionToDecryptOnly {
            key_ring_name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            version: 1,
            reason: "newer active key version 2 is present".to_owned(),
            effective_at_epoch_seconds: 1_210,
        })
        .expect("demotion should use the domain lifecycle port");
    actuator
        .execute(&Action::QuarantineKeyRing {
            key_ring_name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            reason: "compromised observation".to_owned(),
            effective_at_epoch_seconds: 1_220,
        })
        .expect("quarantine should use the domain lifecycle port");

    let repo = actuator.into_inner();
    assert_eq!(repo.sealing_roots().count(), 1);
    let key = repo.keys().next().expect("key remains present");
    assert_eq!(key.state.value, KmsKeyState::Disabled);
    assert!(
        repo.key_version_lifecycle()
            .all(|version| version.state.value == KmsKeyVersionLifecycleState::Quarantined)
    );
}

#[test]
fn status_patches_project_demote_and_quarantine_actions() {
    let observed = observed_dual_active_key_ring();
    let demote = Action::DemoteKeyVersionToDecryptOnly {
        key_ring_name: "tenant-a-ring".to_owned(),
        tenant_id: "ten_alpha".to_owned(),
        version: 1,
        reason: "newer active key version 2 is present".to_owned(),
        effective_at_epoch_seconds: 1_210,
    };
    let quarantine = Action::QuarantineKeyRing {
        key_ring_name: "tenant-a-ring".to_owned(),
        tenant_id: "ten_alpha".to_owned(),
        reason: "compromised observation".to_owned(),
        effective_at_epoch_seconds: 1_220,
    };

    let demote_patch = status_patches_for_actions(&observed, &[demote])
        .expect("demote action should patch key-ring status");
    let quarantine_patch = status_patches_for_actions(&observed, &[quarantine])
        .expect("quarantine action should patch key-ring status");

    assert_eq!(demote_patch.len(), 1);
    assert_eq!(
        demote_patch[0].status["versions"][0]["state"],
        "DecryptOnly"
    );
    assert_eq!(
        demote_patch[0].status["versions"][0]["decryptOnlySinceEpochSeconds"],
        1_210
    );
    assert_eq!(quarantine_patch.len(), 1);
    assert_eq!(quarantine_patch[0].status["health"]["state"], "Compromised");
    assert_eq!(
        quarantine_patch[0].status["versions"][0]["state"],
        "Quarantined"
    );
    assert_eq!(
        quarantine_patch[0].status["versions"][1]["state"],
        "Quarantined"
    );
}

#[test]
fn persistent_domain_repo_reloads_operator_mutations_from_state_path() {
    let path = std::env::temp_dir().join(format!("kms-operator-state-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let mut actuator = DomainKmsOperatorActuator::new(
            secrets_kms_operator_k8s::PersistentCloudKmsDirectory::open(&path)
                .expect("state file should open"),
        );
        actuator
            .execute(&Action::CreateSealingRoot {
                sealing_root: desired_state()
                    .sealing_roots
                    .first()
                    .expect("fixture has a sealing root")
                    .clone(),
            })
            .expect("sealing-root mutation should persist");
        actuator
            .execute(&Action::CreateKeyRing {
                key_ring: desired_state()
                    .key_rings
                    .first()
                    .expect("fixture has a key ring")
                    .clone(),
                requested_at_epoch_seconds: 100,
            })
            .expect("key-ring mutation should persist");
    }

    let reloaded = secrets_kms_operator_k8s::PersistentCloudKmsDirectory::open(&path)
        .expect("state should reload");

    assert_eq!(reloaded.sealing_roots().count(), 1);
    assert_eq!(reloaded.keys().count(), 1);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn domain_actuator_fails_closed_for_quarantined_observed_state() {
    let mut actuator = DomainKmsOperatorActuator::new(CloudKmsDirectory::default());
    let action = Action::QuarantineObservedState {
        reason: "ambiguous relist".to_owned(),
        effective_at_epoch_seconds: 1_000,
    };

    let error = actuator
        .execute(&action)
        .expect_err("ambiguous observed state must not mutate the domain");

    assert_eq!(
        error,
        AdapterError::PartialObservedState(
            "refusing to act on quarantined observed state".to_owned()
        )
    );
}

#[test]
fn sealing_root_status_patch_is_still_available_after_domain_actuation() {
    let action = Action::CreateSealingRoot {
        sealing_root: desired_state()
            .sealing_roots
            .first()
            .expect("fixture has a sealing root")
            .clone(),
    };

    let patches = status_patches_for_actions(&ObservedState::default(), &[action])
        .expect("sealing-root action should build status patch");

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].kind, KmsStatusPatchKind::SealingRoot);
    assert_eq!(patches[0].status["observedVersion"], 1);
}

#[test]
fn reconcile_cycle_executes_actions_and_emits_one_wide_event() {
    let provider = InMemoryObservedStateProvider::complete(Default::default());
    let desired = desired_state();
    let mut actuator = RecordingActuator::default();

    let report = run_reconcile_cycle(
        &provider,
        &desired,
        &FixedClock { now: 1_000 },
        &mut actuator,
    )
    .expect("complete observed state should reconcile");

    assert_eq!(actuator.actions.len(), 2);
    assert_eq!(report.planned_actions, 2);
    assert_eq!(report.executed_actions, 2);
    assert_eq!(
        report.wide_event.event_name,
        "secrets_kms_operator_reconcile"
    );
    assert_eq!(report.wide_event.action_count, 2);
}

#[test]
fn exponential_backoff_is_capped() {
    let backoff = ExponentialBackoff {
        base_seconds: 5,
        max_seconds: 60,
    };

    assert_eq!(backoff.delay_seconds(0), 5);
    assert_eq!(backoff.delay_seconds(1), 10);
    assert_eq!(backoff.delay_seconds(10), 60);
}

fn key_ring_spec_json() -> serde_json::Value {
    json!({
        "tenantId": "ten_alpha",
        "region": "region-home",
        "cellId": "cell-region-home-a-001",
        "hsmPartitionRef": "hsm/region-home/cell-region-home-a-001",
        "origin": "OyatieManaged",
        "usage": "EncryptDecrypt",
        "hsmValidation": "PackEnhancedFips1403Level3",
        "residency": "StrictHomeRegion",
        "dataClass": "InternalOnly",
        "rotationPolicy": {
            "rotateAfterSeconds": 900,
            "decryptOnlyGraceSeconds": 300
        }
    })
}

fn sealing_root_spec_json() -> serde_json::Value {
    json!({
        "tenantId": "ten_alpha",
        "region": "us-east-1",
        "cellId": "cell-us-east-1-a",
        "rootRef": "sealing-root/tenant-a",
        "activeVersion": 1,
        "rotateAfterSeconds": 86400
    })
}

fn domain_key_ring_spec_json() -> serde_json::Value {
    json!({
        "tenantId": "ten_beta",
        "region": "region-home",
        "cellId": "cell-region-home-b-001",
        "hsmPartitionRef": "hsm/region-home/cell-region-home-b-001",
        "origin": "OyatieManaged",
        "usage": "EncryptDecrypt",
        "hsmValidation": "PackEnhancedFips1403Level3",
        "residency": "StrictHomeRegion",
        "dataClass": "PiiIdentifying",
        "rotationPolicy": {
            "rotateAfterSeconds": 900,
            "decryptOnlyGraceSeconds": 300
        }
    })
}

fn observed_dual_active_key_ring() -> ObservedState {
    project_observed_state(&[ProjectedKmsObject::from_json(json!({
        "kind": "KmsKeyRing",
        "metadata": {"name": "tenant-a-ring"},
        "spec": key_ring_spec_json(),
        "status": {
            "health": {"state": "Healthy"},
            "versions": [
                {
                    "version": 1,
                    "state": "Active",
                    "createdAtEpochSeconds": 100,
                    "activatedAtEpochSeconds": 100
                },
                {
                    "version": 2,
                    "state": "Active",
                    "createdAtEpochSeconds": 1200,
                    "activatedAtEpochSeconds": 1200
                }
            ]
        }
    }))])
    .expect("dual-active status should parse")
}

fn desired_state() -> DesiredState {
    DesiredState {
        sealing_roots: vec![SealingRoot {
            name: "tenant-a-root".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            region: "us-east-1".to_owned(),
            cell_id: "cell-us-east-1-a".to_owned(),
            root_ref: "sealing-root/tenant-a".to_owned(),
            active_version: 1,
            rotate_after_seconds: 86_400,
        }],
        key_rings: vec![KeyRing {
            name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            region: "region-home".to_owned(),
            cell_id: "cell-region-home-a-001".to_owned(),
            hsm_partition_ref: "hsm/region-home/cell-region-home-a-001".to_owned(),
            origin: KeyOrigin::OyatieManaged,
            usage: KeyUsage::EncryptDecrypt,
            hsm_validation: HsmValidation::PackEnhancedFips1403Level3,
            residency: ResidencyMode::StrictHomeRegion,
            data_class: DataClassLabel::InternalOnly,
            rotation_policy: KeyVersionRotationPolicy {
                rotate_after_seconds: 900,
                decrypt_only_grace_seconds: 300,
            },
        }],
    }
}
