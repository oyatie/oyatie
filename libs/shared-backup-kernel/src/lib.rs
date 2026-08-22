//! Backup substrate kernel — ADR-0197.
//!
//! # Why this crate exists
//!
//! ADR-0197 mandates a three-pronged backup substrate (Velero +
//! pgBackRest + Restic) where each prong owns exactly one concern. This
//! crate exposes the `BackupExecutor` trait — the inviolate seam between
//! ops-layer code and the prong implementations. Application code never
//! sees this trait directly; the µservice's Postgres-backup sidecar
//! controller, the cluster-state backup scheduler, and the
//! filesystem-state agent all call through the trait so prong swaps
//! (e.g. pgBackRest → pgxbackup, Velero → in-house orchestrator) are
//! single adapter changes.
//!
//! # In-house roadmap (per ADR-0197 §In-house roadmap)
//!
//! - **pgBackRest**: KEEP (community standard). pgxbackup is the named
//!   continuity-fork adapter swap.
//! - **Restic**: KEEP (community standard). Borg/Duplicacy are alternates.
//! - **Velero**: Phase 0 wrapped via this trait; Phase 2 in-house build
//!   `backup-orchestrator` is the named target.
//!
//! # Naming justification
//!
//! `shared-backup-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:backup>-<layer:kernel>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

// =====================================================================
// Types
// =====================================================================

/// Which prong is performing the backup (per ADR-0197 D-1).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BackupProng {
    /// Velero — Kubernetes state + persistent volumes via kopia.
    KubernetesState,
    /// pgBackRest — Postgres PITR.
    PostgresPitr,
    /// Restic — non-K8s host state.
    Filesystem,
}

impl BackupProng {
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::KubernetesState => "kubernetes-state",
            Self::PostgresPitr => "postgres-pitr",
            Self::Filesystem => "filesystem",
        }
    }
}

/// Workload class per ADR-0152 / ADR-0197 D-4.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkloadClass {
    App,
    Batch,
    Gpu,
    Regulatory,
}

impl WorkloadClass {
    /// Recovery point objective for the class (per ADR-0197 D-4).
    #[must_use]
    pub const fn rpo(self) -> Duration {
        match self {
            Self::App => Duration::from_secs(15 * 60),
            Self::Batch | Self::Gpu => Duration::from_secs(60 * 60),
            Self::Regulatory => Duration::from_secs(5 * 60),
        }
    }
    /// Recovery time objective for the class.
    #[must_use]
    pub const fn rto(self) -> Duration {
        match self {
            Self::App => Duration::from_secs(60 * 60),
            Self::Batch | Self::Gpu => Duration::from_secs(4 * 60 * 60),
            Self::Regulatory => Duration::from_secs(30 * 60),
        }
    }
}

/// Regulatory pack — controls retention floor (per ADR-0197 D-5).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RegulatoryPack {
    Generic,
    PackPrimary,
    PackSecondary,
    PackHealth,
    PackFinancial,
    PackPublicSector,
}

impl RegulatoryPack {
    /// Annual retention floor in days (per the standards doc backup-canonical.md).
    #[must_use]
    pub const fn retention_floor_days(self) -> u32 {
        match self {
            Self::Generic | Self::PackSecondary | Self::PackFinancial | Self::PackPublicSector => {
                2_555
            } // 7y
            Self::PackPrimary => 1_825, // 5y
            Self::PackHealth => 2_190,  // 6y
        }
    }
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::PackPrimary => "pack-primary",
            Self::PackSecondary => "pack-secondary",
            Self::PackHealth => "pack-health",
            Self::PackFinancial => "pack-financial",
            Self::PackPublicSector => "pack-public-sector",
        }
    }
}

/// Backup target — must be a SeaweedFS bucket per ADR-0197 D-2.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupTarget {
    pub bucket: String,                  // data_class: INTERNAL_ONLY
    pub prefix: String,                  // data_class: INTERNAL_ONLY
    pub regulatory_pack: RegulatoryPack, // data_class: INTERNAL_ONLY
    pub workload_class: WorkloadClass,   // data_class: INTERNAL_ONLY
    pub tenant_id: Option<String>,       // data_class: INTERNAL_ONLY
}

/// A backup-job request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupRequest {
    pub prong: BackupProng,   // data_class: INTERNAL_ONLY
    pub microservice: String, // data_class: INTERNAL_ONLY
    pub namespace: String,    // data_class: INTERNAL_ONLY
    pub target: BackupTarget, // data_class: INTERNAL_ONLY
    /// How many days the resulting backup must be retained.
    pub retention_days: u32, // data_class: INTERNAL_ONLY
    /// age recipient public key (ADR-0197 D-3 encryption).
    pub age_public_key: String, // data_class: INTERNAL_ONLY
    pub labels: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl BackupRequest {
    /// Validates the request against ADR-0197 invariants.
    ///
    /// # Errors
    /// - `RetentionBelowFloor` if `retention_days` is less than the pack
    ///   floor (ADR-0197 D-5).
    /// - `MissingAgeKey` if no age recipient key is set (ADR-0197 D-3
    ///   mandates encryption).
    /// - `BucketNamingViolation` if the target bucket is not in the
    ///   canonical `oya-` shape per ADR-0196 D-3.
    /// - `MissingCostLabels` if mandatory cost labels are missing
    ///   (ADR-0199 D-1).
    pub fn validate(&self) -> Result<(), BackupError> {
        let floor = self.target.regulatory_pack.retention_floor_days();
        if self.retention_days < floor {
            return Err(BackupError::RetentionBelowFloor {
                requested_days: self.retention_days,
                floor_days: floor,
                pack: self.target.regulatory_pack.wire_name(),
            });
        }
        if self.age_public_key.trim().is_empty() {
            return Err(BackupError::MissingAgeKey);
        }
        if !self.target.bucket.starts_with("oya-") {
            return Err(BackupError::BucketNamingViolation);
        }
        // ADR-0199 D-1: every backup MUST carry tenant cost labels
        for required in ["oya.io/cost-center", "oya.io/workload-class"] {
            if !self.labels.contains_key(required) {
                return Err(BackupError::MissingCostLabel {
                    label: required.to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Outcome of a backup attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupOutcome {
    pub backup_id: String,  // data_class: INTERNAL_ONLY
    pub prong: BackupProng, // data_class: INTERNAL_ONLY
    pub bytes_written: u64, // data_class: INTERNAL_ONLY
    pub duration: Duration, // data_class: INTERNAL_ONLY
    /// Observed RPO at completion.
    pub observed_rpo: Duration, // data_class: INTERNAL_ONLY
    pub sealed_audit_event_class: Option<String>, // data_class: INTERNAL_ONLY
}

/// Restore-drill outcome (per ADR-0197 D-6).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreDrillOutcome {
    pub drill_id: String,               // data_class: INTERNAL_ONLY
    pub microservice: String,           // data_class: INTERNAL_ONLY
    pub workload_class: WorkloadClass,  // data_class: INTERNAL_ONLY
    pub observed_rpo: Duration,         // data_class: INTERNAL_ONLY
    pub observed_rto: Duration,         // data_class: INTERNAL_ONLY
    pub passed: bool,                   // data_class: INTERNAL_ONLY
    pub failure_reason: Option<String>, // data_class: INTERNAL_ONLY
    /// `class: BackupRestoreDrill` audit event id when sealed.
    pub audit_event_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl RestoreDrillOutcome {
    /// Returns true iff the drill meets the workload-class RPO + RTO
    /// targets per ADR-0197 D-4.
    #[must_use]
    pub fn meets_targets(&self) -> bool {
        self.passed
            && self.observed_rpo <= self.workload_class.rpo()
            && self.observed_rto <= self.workload_class.rto()
    }
}

/// Errors emitted by the kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackupError {
    RetentionBelowFloor {
        requested_days: u32,
        floor_days: u32,
        pack: &'static str,
    },
    MissingAgeKey,
    BucketNamingViolation,
    MissingCostLabel {
        label: String,
    },
    AdapterFailure {
        detail: String,
    },
    DrillFailedTarget {
        workload_class: WorkloadClass,
    },
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetentionBelowFloor {
                requested_days,
                floor_days,
                pack,
            } => write!(
                f,
                "retention {requested_days}d below regulatory-pack floor {floor_days}d (pack={pack}, ADR-0197 D-5)"
            ),
            Self::MissingAgeKey => write!(
                f,
                "age public key missing (ADR-0197 D-3 mandates encryption)"
            ),
            Self::BucketNamingViolation => write!(
                f,
                "target bucket not in canonical `oya-` shape (ADR-0196 D-3)"
            ),
            Self::MissingCostLabel { label } => write!(
                f,
                "required tenant cost label missing: {label} (ADR-0199 D-1)"
            ),
            Self::AdapterFailure { detail } => write!(f, "backup adapter failure: {detail}"),
            Self::DrillFailedTarget { workload_class } => write!(
                f,
                "restore drill exceeded {workload_class:?} RPO/RTO target"
            ),
        }
    }
}

impl std::error::Error for BackupError {}

// =====================================================================
// Trait
// =====================================================================

/// Canonical backup-prong seam per ADR-0197.
pub trait BackupExecutor: Send + Sync {
    /// Which prong this executor implements.
    fn prong(&self) -> BackupProng;

    /// Backend identification (Velero / pgBackRest / Restic / in-house).
    fn backend_kind(&self) -> &'static str;

    /// Run a backup.
    ///
    /// # Errors
    /// Returns `BackupError` on any invariant violation (validation) or
    /// adapter-layer failure (`AdapterFailure`).
    fn run(&self, request: &BackupRequest) -> Result<BackupOutcome, BackupError>;

    /// Run a quarterly restore drill (per ADR-0197 D-6).
    ///
    /// # Errors
    /// Returns `DrillFailedTarget` when observed RPO/RTO exceed the
    /// workload-class targets.
    fn run_restore_drill(
        &self,
        microservice: &str,
        workload_class: WorkloadClass,
    ) -> Result<RestoreDrillOutcome, BackupError>;
}

// =====================================================================
// Reference in-memory executor
// =====================================================================

#[derive(Debug, Default)]
struct InMemoryLedger {
    next_backup_id: u64, // data_class: INTERNAL_ONLY
    next_drill_id: u64,  // data_class: INTERNAL_ONLY
    /// Track total bytes per (prong, microservice) for assertions.
    bytes_per_microservice: BTreeMap<(BackupProng, String), u64>, // data_class: INTERNAL_ONLY
}

/// Reference in-memory `BackupExecutor`. Use in tests.
#[derive(Debug)]
pub struct InMemoryBackupExecutor {
    prong: BackupProng,            // data_class: INTERNAL_ONLY
    ledger: Mutex<InMemoryLedger>, // data_class: INTERNAL_ONLY
    /// Simulated RPO at run-time (defaults to half the workload-class RPO).
    simulated_rpo_secs: u64, // data_class: INTERNAL_ONLY
    /// Simulated RTO during drills (defaults to half the workload-class RTO).
    simulated_rto_secs: u64, // data_class: INTERNAL_ONLY
}

impl InMemoryBackupExecutor {
    #[must_use]
    pub fn new(prong: BackupProng) -> Self {
        Self {
            prong,
            ledger: Mutex::new(InMemoryLedger::default()),
            simulated_rpo_secs: 60,
            simulated_rto_secs: 5 * 60,
        }
    }

    fn lock(&self) -> MutexGuard<'_, InMemoryLedger> {
        self.ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl BackupExecutor for InMemoryBackupExecutor {
    fn prong(&self) -> BackupProng {
        self.prong
    }

    fn backend_kind(&self) -> &'static str {
        match self.prong {
            BackupProng::KubernetesState => "velero-in-memory-reference",
            BackupProng::PostgresPitr => "pgbackrest-in-memory-reference",
            BackupProng::Filesystem => "restic-in-memory-reference",
        }
    }

    fn run(&self, request: &BackupRequest) -> Result<BackupOutcome, BackupError> {
        request.validate()?;
        if request.prong != self.prong {
            return Err(BackupError::AdapterFailure {
                detail: format!(
                    "executor prong {:?} mismatch request prong {:?}",
                    self.prong, request.prong
                ),
            });
        }
        let mut ledger = self.lock();
        ledger.next_backup_id += 1;
        let id = format!("{}-{:06}", request.prong.wire_name(), ledger.next_backup_id);
        let bytes = 1_024 * u64::from(request.retention_days);
        let entry = ledger
            .bytes_per_microservice
            .entry((self.prong, request.microservice.clone()))
            .or_insert(0);
        *entry = entry.saturating_add(bytes);
        Ok(BackupOutcome {
            backup_id: id,
            prong: self.prong,
            bytes_written: bytes,
            duration: Duration::from_secs(30),
            observed_rpo: Duration::from_secs(self.simulated_rpo_secs),
            sealed_audit_event_class: Some("BackupRunSealed".to_string()),
        })
    }

    fn run_restore_drill(
        &self,
        microservice: &str,
        workload_class: WorkloadClass,
    ) -> Result<RestoreDrillOutcome, BackupError> {
        let mut ledger = self.lock();
        ledger.next_drill_id += 1;
        let id = format!("drill-{}-{:06}", microservice, ledger.next_drill_id);
        let observed_rpo = Duration::from_secs(self.simulated_rpo_secs);
        let observed_rto = Duration::from_secs(self.simulated_rto_secs);
        let passed = observed_rpo <= workload_class.rpo() && observed_rto <= workload_class.rto();
        Ok(RestoreDrillOutcome {
            drill_id: id.clone(),
            microservice: microservice.to_string(),
            workload_class,
            observed_rpo,
            observed_rto,
            passed,
            failure_reason: if passed {
                None
            } else {
                Some("simulated RPO/RTO above target".to_string())
            },
            audit_event_id: Some(format!("audit-{id}")),
        })
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert("oya.io/cost-center".into(), "axis-foundry".into());
        m.insert("oya.io/workload-class".into(), "app".into());
        m
    }

    fn request(prong: BackupProng, pack: RegulatoryPack, retention_days: u32) -> BackupRequest {
        BackupRequest {
            prong,
            microservice: "foundry".into(),
            namespace: "foundry-app-prod".into(),
            target: BackupTarget {
                bucket: "velero-backup-shared-prod".into(),
                prefix: "foundry/".into(),
                regulatory_pack: pack,
                workload_class: WorkloadClass::App,
                tenant_id: None,
            },
            retention_days,
            age_public_key: "age1q0pretendkey".into(),
            labels: labels(),
        }
    }

    // ---- Retention floor enforcement (ADR-0197 D-5) ----

    #[test]
    fn retention_meeting_pack_floor_accepted() {
        let req = request(BackupProng::KubernetesState, RegulatoryPack::Generic, 2_555);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn retention_below_generic_floor_rejected() {
        let req = request(BackupProng::KubernetesState, RegulatoryPack::Generic, 100);
        let err = req.validate().unwrap_err();
        assert!(matches!(err, BackupError::RetentionBelowFloor { .. }));
    }

    #[test]
    fn retention_pack_primary_floor_is_5y() {
        assert_eq!(RegulatoryPack::PackPrimary.retention_floor_days(), 1_825);
        let req = request(
            BackupProng::KubernetesState,
            RegulatoryPack::PackPrimary,
            1_825,
        );
        assert!(req.validate().is_ok());
    }

    #[test]
    fn retention_pack_health_floor_is_6y() {
        assert_eq!(RegulatoryPack::PackHealth.retention_floor_days(), 2_190);
    }

    // ---- age-key enforcement (ADR-0197 D-3) ----

    #[test]
    fn missing_age_key_rejected() {
        let mut req = request(BackupProng::PostgresPitr, RegulatoryPack::Generic, 2_555);
        req.age_public_key = String::new();
        assert_eq!(req.validate().unwrap_err(), BackupError::MissingAgeKey);
    }

    #[test]
    fn whitespace_age_key_rejected() {
        let mut req = request(BackupProng::PostgresPitr, RegulatoryPack::Generic, 2_555);
        req.age_public_key = "   ".into();
        assert_eq!(req.validate().unwrap_err(), BackupError::MissingAgeKey);
    }

    // ---- Bucket-naming enforcement (ADR-0196 D-3) ----

    #[test]
    fn bucket_without_prefix_rejected() {
        let mut req = request(BackupProng::Filesystem, RegulatoryPack::Generic, 2_555);
        req.target.bucket = "not-bucket".into();
        assert_eq!(
            req.validate().unwrap_err(),
            BackupError::BucketNamingViolation
        );
    }

    // ---- Cost-label enforcement (ADR-0199 D-1) ----

    #[test]
    fn missing_cost_center_label_rejected() {
        let mut req = request(BackupProng::KubernetesState, RegulatoryPack::Generic, 2_555);
        req.labels.remove("oya.io/cost-center");
        let err = req.validate().unwrap_err();
        assert_eq!(
            err,
            BackupError::MissingCostLabel {
                label: "oya.io/cost-center".into(),
            }
        );
    }

    // ---- Executor end-to-end ----

    #[test]
    fn velero_executor_runs_a_backup() {
        let exec = InMemoryBackupExecutor::new(BackupProng::KubernetesState);
        let outcome = exec
            .run(&request(
                BackupProng::KubernetesState,
                RegulatoryPack::Generic,
                2_555,
            ))
            .unwrap();
        assert!(outcome.backup_id.starts_with("kubernetes-state-"));
        assert!(outcome.bytes_written > 0);
        assert_eq!(outcome.prong, BackupProng::KubernetesState);
        assert_eq!(
            outcome.sealed_audit_event_class.as_deref(),
            Some("BackupRunSealed")
        );
    }

    #[test]
    fn executor_rejects_request_with_wrong_prong() {
        let exec = InMemoryBackupExecutor::new(BackupProng::KubernetesState);
        let err = exec
            .run(&request(
                BackupProng::PostgresPitr,
                RegulatoryPack::Generic,
                2_555,
            ))
            .unwrap_err();
        assert!(matches!(err, BackupError::AdapterFailure { .. }));
    }

    #[test]
    fn restore_drill_passes_for_app_class() {
        let exec = InMemoryBackupExecutor::new(BackupProng::KubernetesState);
        let outcome = exec
            .run_restore_drill("foundry", WorkloadClass::App)
            .unwrap();
        assert!(outcome.passed);
        assert!(outcome.meets_targets());
        assert!(outcome.audit_event_id.is_some());
    }

    #[test]
    fn workload_rpo_rto_match_adr_table() {
        assert_eq!(WorkloadClass::App.rpo(), Duration::from_secs(15 * 60));
        assert_eq!(WorkloadClass::App.rto(), Duration::from_secs(60 * 60));
        assert_eq!(WorkloadClass::Regulatory.rpo(), Duration::from_secs(5 * 60));
        assert_eq!(
            WorkloadClass::Regulatory.rto(),
            Duration::from_secs(30 * 60)
        );
    }

    #[test]
    fn prong_wire_names_are_stable() {
        assert_eq!(BackupProng::KubernetesState.wire_name(), "kubernetes-state");
        assert_eq!(BackupProng::PostgresPitr.wire_name(), "postgres-pitr");
        assert_eq!(BackupProng::Filesystem.wire_name(), "filesystem");
    }
}
