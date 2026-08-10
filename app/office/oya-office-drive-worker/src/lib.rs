#![forbid(unsafe_code)]
//! Drive preview, index, lifecycle, version, trash, and retention worker scaffold.
//!
//! Workers are modeled as idempotent Drive jobs before queue/runtime dependencies
//! are adopted.

use oya_office_drive_api::DriveEventKind;
use oya_office_drive_domain::DriveObjectBinding;
use oya_office_kernel::{ObjectId, TenantId};

/// Stable application identifier used by workspace and Buck2 scaffold verification.
pub const APP_NAME: &str = "oya-office-drive-worker";

/// Product vertical slice owned by this deployable.
pub const VERTICAL_SLICE: &str = "drive";

/// Source-shaped deployable layer represented by this scaffold.
pub const DEPLOYABLE_LAYER: &str = "worker";

/// Drive worker job type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveWorkerJobKind {
    /// Generate object preview.
    Preview,
    /// Index metadata/content for search.
    Index,
    /// Apply lifecycle/retention state.
    Lifecycle,
    /// Trash or restore object.
    Trash,
}

impl DriveWorkerJobKind {
    /// Returns custom metric name used by HPA and queue dashboards.
    #[must_use]
    pub const fn queue_age_metric(self) -> &'static str {
        match self {
            Self::Preview => "oya_office_drive_preview_queue_age_seconds",
            Self::Index => "oya_office_drive_index_queue_age_seconds",
            Self::Lifecycle => "oya_office_drive_lifecycle_queue_age_seconds",
            Self::Trash => "oya_office_drive_trash_queue_age_seconds",
        }
    }
}

/// Drive worker autoscaling signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveWorkerScaleSignal {
    /// Preview queue age.
    PreviewQueueAgeSeconds,
    /// Search/index queue age.
    IndexQueueAgeSeconds,
    /// Lifecycle/retention queue age.
    LifecycleQueueAgeSeconds,
    /// Lag between object mutation and search index visibility.
    IndexLagSeconds,
    /// Poison-file rate for invalid or hostile OOXML/content jobs.
    PoisonFileRatePerMinute,
}

impl DriveWorkerScaleSignal {
    /// Returns product metric name.
    #[must_use]
    pub const fn metric_name(self) -> &'static str {
        match self {
            Self::PreviewQueueAgeSeconds => "oya_office_drive_preview_queue_age_seconds",
            Self::IndexQueueAgeSeconds => "oya_office_drive_index_queue_age_seconds",
            Self::LifecycleQueueAgeSeconds => "oya_office_drive_lifecycle_queue_age_seconds",
            Self::IndexLagSeconds => "oya_office_drive_index_lag_seconds",
            Self::PoisonFileRatePerMinute => "oya_office_drive_poison_file_rate_per_minute",
        }
    }

    /// Returns initial HPA target average value.
    #[must_use]
    pub const fn target_average_value(self) -> &'static str {
        match self {
            Self::PreviewQueueAgeSeconds => "30",
            Self::IndexQueueAgeSeconds => "30",
            Self::LifecycleQueueAgeSeconds => "60",
            Self::IndexLagSeconds => "45",
            Self::PoisonFileRatePerMinute => "1",
        }
    }
}

/// Returns Drive worker scale signals in the sequence expected by K8s contracts and dashboards.
#[must_use]
pub const fn drive_worker_scale_signals() -> [DriveWorkerScaleSignal; 5] {
    [
        DriveWorkerScaleSignal::PreviewQueueAgeSeconds,
        DriveWorkerScaleSignal::IndexQueueAgeSeconds,
        DriveWorkerScaleSignal::LifecycleQueueAgeSeconds,
        DriveWorkerScaleSignal::IndexLagSeconds,
        DriveWorkerScaleSignal::PoisonFileRatePerMinute,
    ]
}

/// Drive worker horizontal scaling contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveWorkerScalingContract {
    min_replicas: u16,
    max_replicas: u16,
    signals: [DriveWorkerScaleSignal; 5],
}

impl DriveWorkerScalingContract {
    /// Creates the default Drive worker scaling contract.
    #[must_use]
    pub const fn production_baseline() -> Self {
        Self {
            min_replicas: 2,
            max_replicas: 50,
            signals: drive_worker_scale_signals(),
        }
    }

    /// Returns minimum replicas.
    #[must_use]
    pub const fn min_replicas(&self) -> u16 {
        self.min_replicas
    }

    /// Returns maximum replicas.
    #[must_use]
    pub const fn max_replicas(&self) -> u16 {
        self.max_replicas
    }

    /// Returns scaling signals.
    #[must_use]
    pub const fn signals(&self) -> &[DriveWorkerScaleSignal; 5] {
        &self.signals
    }
}

/// Idempotent Drive worker job contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveWorkerJob {
    tenant_id: TenantId,
    object_id: ObjectId,
    kind: DriveWorkerJobKind,
    binding: DriveObjectBinding,
}

impl DriveWorkerJob {
    /// Creates a worker job from a Drive binding.
    #[must_use]
    pub fn new(binding: DriveObjectBinding, kind: DriveWorkerJobKind) -> Self {
        Self {
            tenant_id: binding.tenant_id().clone(),
            object_id: binding.object_id().clone(),
            kind,
            binding,
        }
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns job kind.
    #[must_use]
    pub const fn kind(&self) -> DriveWorkerJobKind {
        self.kind
    }

    /// Returns Drive binding.
    #[must_use]
    pub const fn binding(&self) -> &DriveObjectBinding {
        &self.binding
    }

    /// Returns event emitted after successful job completion.
    #[must_use]
    pub const fn success_event(&self) -> DriveEventKind {
        match self.kind {
            DriveWorkerJobKind::Preview | DriveWorkerJobKind::Index => {
                DriveEventKind::ObjectUpdated
            }
            DriveWorkerJobKind::Lifecycle | DriveWorkerJobKind::Trash => {
                DriveEventKind::LifecycleChanged
            }
        }
    }
}

/// Starts the scaffolded application entrypoint.
pub fn run() {}

#[cfg(test)]
mod tests {
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_kernel::{DataClass, ObjectId, TenantId};

    use super::{APP_NAME, DEPLOYABLE_LAYER, DriveWorkerJob, DriveWorkerJobKind, VERTICAL_SLICE};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!APP_NAME.is_empty());
        assert!(!DEPLOYABLE_LAYER.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn drive_worker_jobs_preserve_tenant_object_binding() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("drive-object-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Internal,
        );
        let job = DriveWorkerJob::new(binding, DriveWorkerJobKind::Index);
        assert_eq!(job.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(
            job.kind().queue_age_metric(),
            "oya_office_drive_index_queue_age_seconds"
        );
    }

    #[test]
    fn drive_worker_scaling_signals_use_queue_lag_and_poison_rate() {
        let signals = super::drive_worker_scale_signals();
        assert!(
            signals
                .iter()
                .any(|signal| signal.metric_name() == "oya_office_drive_preview_queue_age_seconds")
        );
        assert!(
            signals
                .iter()
                .any(|signal| signal.metric_name() == "oya_office_drive_index_lag_seconds")
        );
        assert!(signals
            .iter()
            .any(|signal| signal.metric_name() == "oya_office_drive_poison_file_rate_per_minute"));
        assert_eq!(
            super::DriveWorkerScalingContract::production_baseline().max_replicas(),
            50
        );
    }
}
