//! Pure Cloud KMS operator reconcile kernel.
//!
//! This crate owns CRD-shaped desired/observed state and the pure reconcile
//! decision function. It intentionally has no kube-rs, k8s-openapi, async
//! runtime, or system-clock dependency.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub trait Clock {
    fn now_epoch_seconds(&self) -> u64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyOrigin {
    OyatieManaged,
    Byok,
    Hyok,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyUsage {
    EncryptDecrypt,
    SignVerify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HsmValidation {
    PackEnhancedFips1403Level3,
    Fips1403Level3,
    Cryptrec,
    CommonCriteriaEal4,
    PciHsm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidencyMode {
    StrictHomeRegion,
    HomeWithRecoveryFailover,
    Global,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassLabel {
    Public,
    InternalOnly,
    PiiIdentifying,
    Phi,
    Pci,
    Secret,
    Audit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyVersionRotationPolicy {
    pub rotate_after_seconds: u64,       // data_class: INTERNAL_ONLY
    pub decrypt_only_grace_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyRing {
    pub name: String,                              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub cell_id: String,                           // data_class: PUBLIC
    pub hsm_partition_ref: String,                 // data_class: INTERNAL_ONLY
    pub origin: KeyOrigin,                         // data_class: PUBLIC
    pub usage: KeyUsage,                           // data_class: PUBLIC
    pub hsm_validation: HsmValidation,             // data_class: PUBLIC
    pub residency: ResidencyMode,                  // data_class: INTERNAL_ONLY
    pub data_class: DataClassLabel,                // data_class: INTERNAL_ONLY
    pub rotation_policy: KeyVersionRotationPolicy, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SealingRoot {
    pub name: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub region: String,            // data_class: PUBLIC
    pub cell_id: String,           // data_class: PUBLIC
    pub root_ref: String,          // data_class: INTERNAL_ONLY
    pub active_version: u32,       // data_class: INTERNAL_ONLY
    pub rotate_after_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesiredState {
    pub key_rings: Vec<KeyRing>,         // data_class: INTERNAL_ONLY
    pub sealing_roots: Vec<SealingRoot>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadConsistency {
    #[default]
    Complete,
    Partial,
    Ambiguous,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "reason", rename_all = "snake_case")]
pub enum ObservedHealth {
    #[default]
    Healthy,
    Ambiguous(String),
    Compromised(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyVersionState {
    Pending,
    Active,
    DecryptOnly,
    Quarantined,
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedKeyVersion {
    pub version: u32,                                  // data_class: INTERNAL_ONLY
    pub state: KeyVersionState,                        // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,                 // data_class: INTERNAL_ONLY
    pub activated_at_epoch_seconds: u64,               // data_class: INTERNAL_ONLY
    pub decrypt_only_since_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedKeyRing {
    pub desired: KeyRing,                  // data_class: INTERNAL_ONLY
    pub versions: Vec<ObservedKeyVersion>, // data_class: INTERNAL_ONLY
    pub health: ObservedHealth,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedSealingRoot {
    pub desired: SealingRoot,   // data_class: INTERNAL_ONLY
    pub observed_version: u32,  // data_class: INTERNAL_ONLY
    pub health: ObservedHealth, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedState {
    pub read_consistency: ReadConsistency, // data_class: INTERNAL_ONLY
    pub key_rings: Vec<ObservedKeyRing>,   // data_class: INTERNAL_ONLY
    pub sealing_roots: Vec<ObservedSealingRoot>, // data_class: INTERNAL_ONLY
}

impl Default for ObservedState {
    fn default() -> Self {
        Self {
            read_consistency: ReadConsistency::Complete,
            key_rings: Vec::new(),
            sealing_roots: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    CreateSealingRoot {
        sealing_root: SealingRoot, // data_class: INTERNAL_ONLY
    },
    CreateKeyRing {
        key_ring: KeyRing,               // data_class: INTERNAL_ONLY
        requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    },
    RotateKeyVersion {
        key_ring: KeyRing,               // data_class: INTERNAL_ONLY
        observed_active_version: u32,    // data_class: INTERNAL_ONLY
        reason: String,                  // data_class: INTERNAL_ONLY
        requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    },
    DemoteKeyVersionToDecryptOnly {
        key_ring_name: String,           // data_class: INTERNAL_ONLY
        tenant_id: String,               // data_class: INTERNAL_ONLY
        version: u32,                    // data_class: INTERNAL_ONLY
        reason: String,                  // data_class: INTERNAL_ONLY
        effective_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    },
    QuarantineKeyRing {
        key_ring_name: String,           // data_class: INTERNAL_ONLY
        tenant_id: String,               // data_class: INTERNAL_ONLY
        reason: String,                  // data_class: INTERNAL_ONLY
        effective_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    },
    QuarantineObservedState {
        reason: String,                  // data_class: INTERNAL_ONLY
        effective_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    },
}

pub fn reconcile<C: Clock>(
    observed: &ObservedState,
    desired: &DesiredState,
    clock: &C,
) -> Vec<Action> {
    let now = clock.now_epoch_seconds();
    if observed.read_consistency != ReadConsistency::Complete {
        return vec![Action::QuarantineObservedState {
            reason: "observed state was not complete".to_owned(),
            effective_at_epoch_seconds: now,
        }];
    }

    let mut actions = Vec::new();

    for sealing_root in &desired.sealing_roots {
        match observed
            .sealing_roots
            .iter()
            .find(|observed_root| same_sealing_root(&observed_root.desired, sealing_root))
        {
            Some(observed_root) => match &observed_root.health {
                ObservedHealth::Healthy => {
                    if observed_root.observed_version != sealing_root.active_version {
                        actions.push(Action::CreateSealingRoot {
                            sealing_root: sealing_root.clone(),
                        });
                    }
                }
                ObservedHealth::Ambiguous(reason) | ObservedHealth::Compromised(reason) => {
                    actions.push(Action::QuarantineObservedState {
                        reason: format!("sealing root {} unhealthy: {reason}", sealing_root.name),
                        effective_at_epoch_seconds: now,
                    });
                }
            },
            None => actions.push(Action::CreateSealingRoot {
                sealing_root: sealing_root.clone(),
            }),
        }
    }

    for key_ring in &desired.key_rings {
        match observed
            .key_rings
            .iter()
            .find(|observed_ring| same_key_ring(&observed_ring.desired, key_ring))
        {
            Some(observed_ring) => {
                reconcile_key_ring(observed_ring, key_ring, now, &mut actions);
            }
            None => actions.push(Action::CreateKeyRing {
                key_ring: key_ring.clone(),
                requested_at_epoch_seconds: now,
            }),
        }
    }

    actions
}

fn reconcile_key_ring(
    observed: &ObservedKeyRing,
    desired: &KeyRing,
    now_epoch_seconds: u64,
    actions: &mut Vec<Action>,
) {
    match &observed.health {
        ObservedHealth::Healthy => {}
        ObservedHealth::Ambiguous(reason) | ObservedHealth::Compromised(reason) => {
            if !all_versions_quarantined(&observed.versions) {
                actions.push(Action::QuarantineKeyRing {
                    key_ring_name: desired.name.clone(),
                    tenant_id: desired.tenant_id.clone(),
                    reason: reason.clone(),
                    effective_at_epoch_seconds: now_epoch_seconds,
                });
            }
            return;
        }
    }

    let active_versions: Vec<&ObservedKeyVersion> = observed
        .versions
        .iter()
        .filter(|version| version.state == KeyVersionState::Active)
        .collect();

    if active_versions.is_empty() {
        if observed.versions.is_empty() {
            actions.push(Action::CreateKeyRing {
                key_ring: desired.clone(),
                requested_at_epoch_seconds: now_epoch_seconds,
            });
        } else if !all_versions_quarantined(&observed.versions) {
            actions.push(Action::QuarantineKeyRing {
                key_ring_name: desired.name.clone(),
                tenant_id: desired.tenant_id.clone(),
                reason: "no active key version observed".to_owned(),
                effective_at_epoch_seconds: now_epoch_seconds,
            });
        }
        return;
    }

    if active_versions.len() > 1 {
        if let Some(newest_active) = newest_active_version(&active_versions) {
            for active in &active_versions {
                if active.version != newest_active.version {
                    actions.push(Action::DemoteKeyVersionToDecryptOnly {
                        key_ring_name: desired.name.clone(),
                        tenant_id: desired.tenant_id.clone(),
                        version: active.version,
                        reason: format!(
                            "newer active key version {} is present",
                            newest_active.version
                        ),
                        effective_at_epoch_seconds: now_epoch_seconds,
                    });
                }
            }
        }
        return;
    }

    if let Some(active) = active_versions.first() {
        let age_seconds = now_epoch_seconds.saturating_sub(active.activated_at_epoch_seconds);
        if age_seconds >= desired.rotation_policy.rotate_after_seconds
            && !has_newer_non_destroyed_version(&observed.versions, active.version)
        {
            actions.push(Action::RotateKeyVersion {
                key_ring: desired.clone(),
                observed_active_version: active.version,
                reason: format!(
                    "active key version age {age_seconds}s exceeds policy {}s",
                    desired.rotation_policy.rotate_after_seconds
                ),
                requested_at_epoch_seconds: now_epoch_seconds,
            });
        }
    }
}

fn same_key_ring(observed: &KeyRing, desired: &KeyRing) -> bool {
    observed.name == desired.name && observed.tenant_id == desired.tenant_id
}

fn same_sealing_root(observed: &SealingRoot, desired: &SealingRoot) -> bool {
    observed.name == desired.name && observed.tenant_id == desired.tenant_id
}

fn all_versions_quarantined(versions: &[ObservedKeyVersion]) -> bool {
    !versions.is_empty()
        && versions
            .iter()
            .all(|version| version.state == KeyVersionState::Quarantined)
}

fn newest_active_version<'a>(
    versions: &'a [&'a ObservedKeyVersion],
) -> Option<&'a ObservedKeyVersion> {
    versions
        .iter()
        .copied()
        .max_by_key(|version| version.version)
}

fn has_newer_non_destroyed_version(versions: &[ObservedKeyVersion], active_version: u32) -> bool {
    versions.iter().any(|version| {
        version.version > active_version && version.state != KeyVersionState::Destroyed
    })
}
