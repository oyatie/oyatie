use secrets_kms_operator_kernel::{
    Action, Clock, DataClassLabel, DesiredState, HsmValidation, KeyOrigin, KeyRing, KeyUsage,
    KeyVersionRotationPolicy, KeyVersionState, ObservedHealth, ObservedKeyRing, ObservedKeyVersion,
    ObservedSealingRoot, ObservedState, ReadConsistency, ResidencyMode, SealingRoot, reconcile,
};

#[derive(Clone, Copy)]
struct FixedClock {
    now: u64,
}

impl Clock for FixedClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

#[test]
fn creates_missing_key_ring_and_sealing_root() {
    let desired = desired_state();
    let actions = reconcile(
        &ObservedState::default(),
        &desired,
        &FixedClock { now: 1_800 },
    );

    assert_eq!(
        actions,
        vec![
            Action::CreateSealingRoot {
                sealing_root: desired.sealing_roots[0].clone(),
            },
            Action::CreateKeyRing {
                key_ring: desired.key_rings[0].clone(),
                requested_at_epoch_seconds: 1_800,
            },
        ]
    );
}

#[test]
fn creates_key_ring_when_crd_status_has_no_versions_yet() {
    let desired = desired_state();
    let observed = observed_with_versions(Vec::new());

    let actions = reconcile(&observed, &desired, &FixedClock { now: 500 });

    assert_eq!(
        actions,
        vec![Action::CreateKeyRing {
            key_ring: desired.key_rings[0].clone(),
            requested_at_epoch_seconds: 500,
        }]
    );
}

#[test]
fn creates_sealing_root_when_observed_version_lags_spec() {
    let desired = desired_state();
    let mut observed = observed_with_versions(vec![ObservedKeyVersion {
        version: 1,
        state: KeyVersionState::Active,
        created_at_epoch_seconds: 100,
        activated_at_epoch_seconds: 100,
        decrypt_only_since_epoch_seconds: None,
    }]);
    observed.sealing_roots[0].observed_version = 0;

    let actions = reconcile(&observed, &desired, &FixedClock { now: 500 });

    assert_eq!(
        actions,
        vec![Action::CreateSealingRoot {
            sealing_root: desired.sealing_roots[0].clone(),
        }]
    );
}

#[test]
fn rotates_active_key_version_after_policy_age() {
    let desired = desired_state();
    let observed = observed_with_versions(vec![ObservedKeyVersion {
        version: 1,
        state: KeyVersionState::Active,
        created_at_epoch_seconds: 100,
        activated_at_epoch_seconds: 100,
        decrypt_only_since_epoch_seconds: None,
    }]);

    let actions = reconcile(&observed, &desired, &FixedClock { now: 1_200 });

    assert_eq!(
        actions,
        vec![Action::RotateKeyVersion {
            key_ring: desired.key_rings[0].clone(),
            observed_active_version: 1,
            reason: "active key version age 1100s exceeds policy 900s".to_owned(),
            requested_at_epoch_seconds: 1_200,
        }]
    );
}

#[test]
fn demotes_older_active_version_to_decrypt_only() {
    let desired = desired_state();
    let observed = observed_with_versions(vec![
        ObservedKeyVersion {
            version: 1,
            state: KeyVersionState::Active,
            created_at_epoch_seconds: 100,
            activated_at_epoch_seconds: 100,
            decrypt_only_since_epoch_seconds: None,
        },
        ObservedKeyVersion {
            version: 2,
            state: KeyVersionState::Active,
            created_at_epoch_seconds: 1_000,
            activated_at_epoch_seconds: 1_000,
            decrypt_only_since_epoch_seconds: None,
        },
    ]);

    let actions = reconcile(&observed, &desired, &FixedClock { now: 1_020 });

    assert_eq!(
        actions,
        vec![Action::DemoteKeyVersionToDecryptOnly {
            key_ring_name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            version: 1,
            reason: "newer active key version 2 is present".to_owned(),
            effective_at_epoch_seconds: 1_020,
        }]
    );
}

#[test]
fn quarantines_compromised_key_ring() {
    let desired = desired_state();
    let mut observed = observed_with_versions(vec![ObservedKeyVersion {
        version: 1,
        state: KeyVersionState::Active,
        created_at_epoch_seconds: 100,
        activated_at_epoch_seconds: 100,
        decrypt_only_since_epoch_seconds: None,
    }]);
    observed.key_rings[0].health =
        ObservedHealth::Compromised("provider evidence digest mismatch".to_owned());

    let actions = reconcile(&observed, &desired, &FixedClock { now: 1_020 });

    assert_eq!(
        actions,
        vec![Action::QuarantineKeyRing {
            key_ring_name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            reason: "provider evidence digest mismatch".to_owned(),
            effective_at_epoch_seconds: 1_020,
        }]
    );
}

#[test]
fn partial_or_ambiguous_observation_only_emits_fail_closed_quarantine() {
    let desired = desired_state();
    let observed = ObservedState {
        read_consistency: ReadConsistency::Partial,
        key_rings: vec![],
        sealing_roots: vec![],
    };

    let actions = reconcile(&observed, &desired, &FixedClock { now: 7 });

    assert_eq!(
        actions,
        vec![Action::QuarantineObservedState {
            reason: "observed state was not complete".to_owned(),
            effective_at_epoch_seconds: 7,
        }]
    );
}

#[test]
fn applying_create_rotate_demote_and_quarantine_actions_is_idempotent() {
    let desired = desired_state();

    let mut observed = ObservedState::default();
    let create_actions = reconcile(&observed, &desired, &FixedClock { now: 1_000 });
    apply_actions(&mut observed, &desired, &create_actions);
    assert_eq!(
        reconcile(&observed, &desired, &FixedClock { now: 1_000 }),
        Vec::<Action>::new()
    );

    let rotate_actions = reconcile(&observed, &desired, &FixedClock { now: 2_000 });
    apply_actions(&mut observed, &desired, &rotate_actions);
    assert_eq!(
        reconcile(&observed, &desired, &FixedClock { now: 2_000 }),
        Vec::<Action>::new()
    );

    observed.key_rings[0].versions.push(ObservedKeyVersion {
        version: 3,
        state: KeyVersionState::Active,
        created_at_epoch_seconds: 2_100,
        activated_at_epoch_seconds: 2_100,
        decrypt_only_since_epoch_seconds: None,
    });
    let demote_actions = reconcile(&observed, &desired, &FixedClock { now: 2_110 });
    apply_actions(&mut observed, &desired, &demote_actions);
    assert_eq!(
        reconcile(&observed, &desired, &FixedClock { now: 2_110 }),
        Vec::<Action>::new()
    );

    observed.key_rings[0].health = ObservedHealth::Compromised("tamper".to_owned());
    let quarantine_actions = reconcile(&observed, &desired, &FixedClock { now: 2_120 });
    apply_actions(&mut observed, &desired, &quarantine_actions);
    assert_eq!(
        reconcile(&observed, &desired, &FixedClock { now: 2_120 }),
        Vec::<Action>::new()
    );
}

fn desired_state() -> DesiredState {
    DesiredState {
        sealing_roots: vec![SealingRoot {
            name: "tenant-a-root".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            region: "us-east-1".to_owned(),
            cell_id: "cell-us-east-1a".to_owned(),
            root_ref: "sealing-root/tenant-a".to_owned(),
            active_version: 1,
            rotate_after_seconds: 86_400,
        }],
        key_rings: vec![KeyRing {
            name: "tenant-a-ring".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            region: "us-east-1".to_owned(),
            cell_id: "cell-us-east-1a".to_owned(),
            hsm_partition_ref: "hsm/us-east-1/cell-us-east-1a".to_owned(),
            origin: KeyOrigin::OyatieManaged,
            usage: KeyUsage::EncryptDecrypt,
            hsm_validation: HsmValidation::Fips1403Level3,
            residency: ResidencyMode::StrictHomeRegion,
            data_class: DataClassLabel::InternalOnly,
            rotation_policy: KeyVersionRotationPolicy {
                rotate_after_seconds: 900,
                decrypt_only_grace_seconds: 300,
            },
        }],
    }
}

fn observed_with_versions(versions: Vec<ObservedKeyVersion>) -> ObservedState {
    let desired = desired_state();
    ObservedState {
        read_consistency: ReadConsistency::Complete,
        sealing_roots: vec![ObservedSealingRoot {
            desired: desired.sealing_roots[0].clone(),
            observed_version: 1,
            health: ObservedHealth::Healthy,
        }],
        key_rings: vec![ObservedKeyRing {
            desired: desired.key_rings[0].clone(),
            versions,
            health: ObservedHealth::Healthy,
        }],
    }
}

fn apply_actions(observed: &mut ObservedState, desired: &DesiredState, actions: &[Action]) {
    for action in actions {
        match action {
            Action::CreateSealingRoot { sealing_root } => {
                observed.sealing_roots.push(ObservedSealingRoot {
                    desired: sealing_root.clone(),
                    observed_version: sealing_root.active_version,
                    health: ObservedHealth::Healthy,
                });
            }
            Action::CreateKeyRing {
                key_ring,
                requested_at_epoch_seconds,
            } => {
                observed.key_rings.push(ObservedKeyRing {
                    desired: key_ring.clone(),
                    versions: vec![ObservedKeyVersion {
                        version: 1,
                        state: KeyVersionState::Active,
                        created_at_epoch_seconds: *requested_at_epoch_seconds,
                        activated_at_epoch_seconds: *requested_at_epoch_seconds,
                        decrypt_only_since_epoch_seconds: None,
                    }],
                    health: ObservedHealth::Healthy,
                });
                observed.read_consistency = ReadConsistency::Complete;
            }
            Action::RotateKeyVersion {
                requested_at_epoch_seconds,
                ..
            } => {
                if let Some(key_ring) = observed.key_rings.first_mut() {
                    let next_version = key_ring
                        .versions
                        .iter()
                        .map(|version| version.version)
                        .max()
                        .unwrap_or(0)
                        + 1;
                    for version in &mut key_ring.versions {
                        if version.state == KeyVersionState::Active {
                            version.state = KeyVersionState::DecryptOnly;
                            version.decrypt_only_since_epoch_seconds =
                                Some(*requested_at_epoch_seconds);
                        }
                    }
                    key_ring.versions.push(ObservedKeyVersion {
                        version: next_version,
                        state: KeyVersionState::Active,
                        created_at_epoch_seconds: *requested_at_epoch_seconds,
                        activated_at_epoch_seconds: *requested_at_epoch_seconds,
                        decrypt_only_since_epoch_seconds: None,
                    });
                }
            }
            Action::DemoteKeyVersionToDecryptOnly {
                version,
                effective_at_epoch_seconds,
                ..
            } => {
                if let Some(key_ring) = observed.key_rings.first_mut() {
                    for key_version in &mut key_ring.versions {
                        if key_version.version == *version {
                            key_version.state = KeyVersionState::DecryptOnly;
                            key_version.decrypt_only_since_epoch_seconds =
                                Some(*effective_at_epoch_seconds);
                        }
                    }
                }
            }
            Action::QuarantineKeyRing { .. } => {
                if let Some(key_ring) = observed.key_rings.first_mut() {
                    key_ring.health = ObservedHealth::Healthy;
                    for version in &mut key_ring.versions {
                        version.state = KeyVersionState::Quarantined;
                    }
                }
            }
            Action::QuarantineObservedState { .. } => {
                observed.read_consistency = ReadConsistency::Complete;
                observed.key_rings = desired
                    .key_rings
                    .iter()
                    .cloned()
                    .map(|key_ring| ObservedKeyRing {
                        desired: key_ring,
                        versions: vec![],
                        health: ObservedHealth::Healthy,
                    })
                    .collect();
            }
        }
    }
}
