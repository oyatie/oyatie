#![forbid(unsafe_code)]
//! DOCX/XLSX/PPTX import, export, render, preview, and poison-file worker contracts.
//!
//! The worker is intentionally stateless at the contract layer: a Drive-bound
//! format job maps to a parallel lane plus an ordered execution sequence. Runtime
//! queueing, sandbox implementation, storage adapters, and metrics exporters land
//! in later vertical-slice stories.

use oya_office_format_domain::{FormatJobContract, FormatJobDirection, FormatWorkerIsolationTier};

/// Stable application identifier used by workspace and Buck2 scaffold verification.
pub const APP_NAME: &str = "oya-office-format-worker";

/// Product vertical slice owned by this deployable.
pub const VERTICAL_SLICE: &str = "format";

/// Source-shaped deployable layer represented by this scaffold.
pub const DEPLOYABLE_LAYER: &str = "worker";

/// Parallel worker lane selected from job direction and fixture risk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatWorkerLane {
    /// Import queue for benign Office uploads.
    Import,
    /// Export queue for benign Drive-to-Office exports.
    Export,
    /// Round-trip queue for fixture/benchmark compatibility checks.
    RoundTrip,
    /// Quarantine-capable queue for macros, external links, or container edges.
    Quarantine,
}

/// Ordered steps every format worker plan must follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatWorkerSequenceStep {
    /// Authorize the tenant-scoped Drive object binding before content access.
    AuthorizeDriveBinding,
    /// Run high-risk package checks before conversion when quarantine is required.
    QuarantineRiskScan,
    /// Fetch Drive/object-store content under bounded range and timeout policy.
    FetchDriveObject,
    /// Convert or parse the Office format in a bounded sandbox.
    ConvertOfficeFormat,
    /// Persist imported/exported/roundtrip output through Drive/storage ports.
    PersistDriveResult,
    /// Emit audit and product metrics after state is persisted.
    EmitAuditAndMetrics,
}

const IMPORT_SEQUENCE: [FormatWorkerSequenceStep; 5] = [
    FormatWorkerSequenceStep::AuthorizeDriveBinding,
    FormatWorkerSequenceStep::FetchDriveObject,
    FormatWorkerSequenceStep::ConvertOfficeFormat,
    FormatWorkerSequenceStep::PersistDriveResult,
    FormatWorkerSequenceStep::EmitAuditAndMetrics,
];

const EXPORT_SEQUENCE: [FormatWorkerSequenceStep; 5] = [
    FormatWorkerSequenceStep::AuthorizeDriveBinding,
    FormatWorkerSequenceStep::FetchDriveObject,
    FormatWorkerSequenceStep::ConvertOfficeFormat,
    FormatWorkerSequenceStep::PersistDriveResult,
    FormatWorkerSequenceStep::EmitAuditAndMetrics,
];

const ROUNDTRIP_SEQUENCE: [FormatWorkerSequenceStep; 5] = [
    FormatWorkerSequenceStep::AuthorizeDriveBinding,
    FormatWorkerSequenceStep::FetchDriveObject,
    FormatWorkerSequenceStep::ConvertOfficeFormat,
    FormatWorkerSequenceStep::PersistDriveResult,
    FormatWorkerSequenceStep::EmitAuditAndMetrics,
];

const QUARANTINE_SEQUENCE: [FormatWorkerSequenceStep; 6] = [
    FormatWorkerSequenceStep::AuthorizeDriveBinding,
    FormatWorkerSequenceStep::QuarantineRiskScan,
    FormatWorkerSequenceStep::FetchDriveObject,
    FormatWorkerSequenceStep::ConvertOfficeFormat,
    FormatWorkerSequenceStep::PersistDriveResult,
    FormatWorkerSequenceStep::EmitAuditAndMetrics,
];

/// Stateless worker execution plan derived from a Drive-bound format job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatWorkerPlan {
    contract: FormatJobContract,
    lane: FormatWorkerLane,
    sequence: &'static [FormatWorkerSequenceStep],
}

impl FormatWorkerPlan {
    /// Creates a worker plan with parallel lane and ordered step semantics.
    #[must_use]
    pub fn from_contract(contract: FormatJobContract) -> Self {
        let (lane, sequence) =
            if contract.required_isolation_tier() == FormatWorkerIsolationTier::Quarantine {
                (FormatWorkerLane::Quarantine, QUARANTINE_SEQUENCE.as_slice())
            } else {
                match contract.direction() {
                    FormatJobDirection::Import => {
                        (FormatWorkerLane::Import, IMPORT_SEQUENCE.as_slice())
                    }
                    FormatJobDirection::Export => {
                        (FormatWorkerLane::Export, EXPORT_SEQUENCE.as_slice())
                    }
                    FormatJobDirection::RoundTrip => {
                        (FormatWorkerLane::RoundTrip, ROUNDTRIP_SEQUENCE.as_slice())
                    }
                }
            };

        Self {
            contract,
            lane,
            sequence,
        }
    }

    /// Returns the selected parallel lane.
    #[must_use]
    pub const fn lane(&self) -> FormatWorkerLane {
        self.lane
    }

    /// Returns the ordered sequence of worker steps.
    #[must_use]
    pub const fn sequence(&self) -> &'static [FormatWorkerSequenceStep] {
        self.sequence
    }

    /// Returns the underlying job contract.
    #[must_use]
    pub const fn contract(&self) -> &FormatJobContract {
        &self.contract
    }
}

/// Returns every lane the horizontally scalable format worker fleet must expose.
#[must_use]
pub const fn format_worker_lanes() -> [FormatWorkerLane; 4] {
    [
        FormatWorkerLane::Import,
        FormatWorkerLane::Export,
        FormatWorkerLane::RoundTrip,
        FormatWorkerLane::Quarantine,
    ]
}

/// Starts the scaffolded application entrypoint.
///
/// Later stories replace this no-op with real Rust runtime wiring while keeping
/// the app horizontally scalable and free of global mutable singleton state.
pub fn run() {}

#[cfg(test)]
mod tests {
    use super::{
        APP_NAME, DEPLOYABLE_LAYER, FormatWorkerLane, FormatWorkerPlan, FormatWorkerSequenceStep,
        VERTICAL_SLICE,
    };
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_format_domain::{
        FixtureComplexity, FixtureCorpusId, FixtureFeature, FormatFixtureBinding, FormatFixtureId,
        FormatFixtureSpec, FormatJobContract, FormatJobDirection, OfficeFormatKind,
    };
    use oya_office_kernel::{DataClass, ObjectId, RequestId, TenantId};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!APP_NAME.is_empty());
        assert!(!DEPLOYABLE_LAYER.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn worker_plan_assigns_parallel_lane_and_ordered_steps() {
        let job = FormatJobContract::from_fixture(
            RequestId::new("format-job-export-1").expect("valid request id"),
            FormatJobDirection::Export,
            &fixture(
                FixtureComplexity::Representative,
                vec![FixtureFeature::Formulas],
            ),
        )
        .expect("job contract");

        let plan = FormatWorkerPlan::from_contract(job);

        assert_eq!(plan.lane(), FormatWorkerLane::Export);
        assert_eq!(
            plan.sequence(),
            &[
                FormatWorkerSequenceStep::AuthorizeDriveBinding,
                FormatWorkerSequenceStep::FetchDriveObject,
                FormatWorkerSequenceStep::ConvertOfficeFormat,
                FormatWorkerSequenceStep::PersistDriveResult,
                FormatWorkerSequenceStep::EmitAuditAndMetrics,
            ]
        );
    }

    #[test]
    fn worker_plan_routes_high_risk_fixture_to_quarantine_lane() {
        let job = FormatJobContract::from_fixture(
            RequestId::new("format-job-import-quarantine").expect("valid request id"),
            FormatJobDirection::Import,
            &fixture(
                FixtureComplexity::Adversarial,
                vec![FixtureFeature::Formulas, FixtureFeature::ExternalLinks],
            ),
        )
        .expect("job contract");

        let plan = FormatWorkerPlan::from_contract(job);

        assert_eq!(plan.lane(), FormatWorkerLane::Quarantine);
        assert!(
            plan.sequence()
                .contains(&FormatWorkerSequenceStep::QuarantineRiskScan)
        );
    }

    fn fixture(complexity: FixtureComplexity, features: Vec<FixtureFeature>) -> FormatFixtureSpec {
        FormatFixtureSpec::new(
            FormatFixtureId::new("fixture-xlsx-platform").expect("valid fixture id"),
            FixtureCorpusId::new("corpus-ooxml-sheets").expect("valid corpus id"),
            FormatFixtureBinding::new(
                DriveObjectBinding::new(
                    TenantId::new("tenant-alpha").expect("valid tenant id"),
                    ObjectId::new("sheet-1").expect("valid object id"),
                    DriveObjectKind::Spreadsheet,
                    DataClass::Confidential,
                ),
                OfficeFormatKind::Xlsx,
            )
            .expect("fixture binding"),
            complexity,
            features,
        )
        .expect("fixture spec")
    }
}
