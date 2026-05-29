//! Managed data-service catalogue (M03-P03-IP-001 `DataService` / `DatabaseEngine`).
//!
//! Provides a provider-tagged `DatabaseEngine` enum and a `DataService`
//! aggregate that pairs a `DataServicePlan` with a concrete engine.
//! The `provision_data_service` admission fn validates that the engine
//! family matches the plan's `DataServiceKind` before any adapter is
//! permitted to proceed with provisioning.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::{DataServiceKind, DataServicePlan};

/// Concrete managed-database engine offered by an adapter.
///
/// Each variant is tagged with its canonical `DataServiceKind` family so
/// the kernel can enforce kind-matching without any adapter-specific logic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DatabaseEngine {
    // OLTP-relational family
    Postgres,
    Citus,
    // OLTP key-value family
    Valkey,
    // OLAP-columnar family
    ClickHouse,
    // OLTP-relational family (Postgres extension; see `kind()` doc)
    Pgvector,
    // Stream family
    Kafka,
}

impl DatabaseEngine {
    /// Canonical short name used in telemetry and audit logs.
    pub fn name(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Citus => "citus",
            Self::Valkey => "valkey",
            Self::ClickHouse => "clickhouse",
            Self::Pgvector => "pgvector",
            Self::Kafka => "kafka",
        }
    }

    /// The `DataServiceKind` family this engine belongs to.
    ///
    /// `Pgvector` is classified as `OltpRelational` because it is a
    /// Postgres-extension engine; the cloud data domain's `ManagedDataEngine::tier()`
    /// maps it to `ManagedDataTier::Oltp`, keeping the two kernels consistent.
    pub fn kind(self) -> DataServiceKind {
        match self {
            Self::Postgres | Self::Citus | Self::Pgvector => DataServiceKind::OltpRelational,
            Self::Valkey => DataServiceKind::OltpKeyValue,
            Self::ClickHouse => DataServiceKind::OlapColumnar,
            Self::Kafka => DataServiceKind::Stream,
        }
    }
}

/// A provisioned data service — plan + concrete engine pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataService {
    // data_class: INTERNAL_ONLY
    pub service_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub plan: DataServicePlan, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub engine: DatabaseEngine, // data_class: INTERNAL_ONLY
}

/// Errors that can be returned by `provision_data_service`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataServiceError {
    EmptyServiceId,
    EmptyPlanId,
    EngineFamilyMismatch {
        plan_kind: DataServiceKind,
        engine_kind: DataServiceKind,
    },
}

impl DataServiceError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyServiceId => "service id is empty".to_owned(),
            Self::EmptyPlanId => "plan id is empty".to_owned(),
            Self::EngineFamilyMismatch {
                plan_kind,
                engine_kind,
            } => format!(
                "engine family mismatch: plan={} engine={}",
                plan_kind.name(),
                engine_kind.name()
            ),
        }
    }
}

/// Validate that a `DataService` can be provisioned: non-empty IDs and
/// engine family must match plan kind.
pub fn provision_data_service(svc: &DataService) -> Result<(), DataServiceError> {
    if svc.service_id.is_empty() {
        return Err(DataServiceError::EmptyServiceId);
    }
    if svc.plan.plan_id.is_empty() {
        return Err(DataServiceError::EmptyPlanId);
    }
    let engine_kind = svc.engine.kind();
    if svc.plan.kind != engine_kind {
        return Err(DataServiceError::EngineFamilyMismatch {
            plan_kind: svc.plan.kind,
            engine_kind,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EncryptionRequirement, ResidencyClass};

    fn plan(kind: DataServiceKind) -> DataServicePlan {
        DataServicePlan {
            plan_id: "plan-1".into(),
            kind,
            residency: ResidencyClass::Global,
            encryption: EncryptionRequirement::AtRest,
        }
    }

    fn svc(id: &str, engine: DatabaseEngine) -> DataService {
        DataService {
            service_id: id.into(),
            plan: plan(engine.kind()),
            engine,
        }
    }

    #[test]
    fn postgres_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-1", DatabaseEngine::Postgres)).is_ok());
    }

    #[test]
    fn citus_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-2", DatabaseEngine::Citus)).is_ok());
    }

    #[test]
    fn valkey_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-3", DatabaseEngine::Valkey)).is_ok());
    }

    #[test]
    fn clickhouse_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-4", DatabaseEngine::ClickHouse)).is_ok());
    }

    #[test]
    fn pgvector_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-5", DatabaseEngine::Pgvector)).is_ok());
    }

    /// Regression guard: before the fix, Pgvector was classified as SearchIndex,
    /// so an OltpRelational plan + Pgvector engine would have been rejected with
    /// `EngineFamilyMismatch`. Verify the admission gate now passes.
    #[test]
    fn pgvector_oltp_relational_plan_admitted() {
        let s = DataService {
            service_id: "svc-pgv".into(),
            plan: plan(DataServiceKind::OltpRelational),
            engine: DatabaseEngine::Pgvector,
        };
        assert!(
            provision_data_service(&s).is_ok(),
            "pgvector + OltpRelational plan must be admitted (was blocked before reclassification)"
        );
    }

    #[test]
    fn kafka_provisions_successfully() {
        assert!(provision_data_service(&svc("svc-6", DatabaseEngine::Kafka)).is_ok());
    }

    #[test]
    fn empty_service_id_rejected() {
        let s = DataService {
            service_id: "".into(),
            plan: plan(DataServiceKind::OltpRelational),
            engine: DatabaseEngine::Postgres,
        };
        assert!(matches!(
            provision_data_service(&s),
            Err(DataServiceError::EmptyServiceId)
        ));
    }

    #[test]
    fn empty_plan_id_rejected() {
        let s = DataService {
            service_id: "svc-1".into(),
            plan: DataServicePlan {
                plan_id: "".into(),
                kind: DataServiceKind::OltpRelational,
                residency: ResidencyClass::Global,
                encryption: EncryptionRequirement::AtRest,
            },
            engine: DatabaseEngine::Postgres,
        };
        assert!(matches!(
            provision_data_service(&s),
            Err(DataServiceError::EmptyPlanId)
        ));
    }

    #[test]
    fn engine_family_mismatch_rejected() {
        // Plan says OlapColumnar but engine is Postgres (OltpRelational)
        let s = DataService {
            service_id: "svc-1".into(),
            plan: plan(DataServiceKind::OlapColumnar),
            engine: DatabaseEngine::Postgres,
        };
        assert!(matches!(
            provision_data_service(&s),
            Err(DataServiceError::EngineFamilyMismatch { .. })
        ));
    }

    #[test]
    fn engine_names_distinct() {
        use std::collections::HashSet;
        let names: HashSet<_> = [
            DatabaseEngine::Postgres,
            DatabaseEngine::Citus,
            DatabaseEngine::Valkey,
            DatabaseEngine::ClickHouse,
            DatabaseEngine::Pgvector,
            DatabaseEngine::Kafka,
        ]
        .iter()
        .map(|e| e.name())
        .collect();
        assert_eq!(names.len(), 6);
    }

    #[test]
    fn engine_kind_mappings_correct() {
        assert_eq!(
            DatabaseEngine::Postgres.kind(),
            DataServiceKind::OltpRelational
        );
        assert_eq!(
            DatabaseEngine::Citus.kind(),
            DataServiceKind::OltpRelational
        );
        assert_eq!(DatabaseEngine::Valkey.kind(), DataServiceKind::OltpKeyValue);
        assert_eq!(
            DatabaseEngine::ClickHouse.kind(),
            DataServiceKind::OlapColumnar
        );
        assert_eq!(
            DatabaseEngine::Pgvector.kind(),
            DataServiceKind::OltpRelational
        );
        assert_eq!(DatabaseEngine::Kafka.kind(), DataServiceKind::Stream);
    }

    #[test]
    fn error_messages_non_empty() {
        let errors = [
            DataServiceError::EmptyServiceId,
            DataServiceError::EmptyPlanId,
            DataServiceError::EngineFamilyMismatch {
                plan_kind: DataServiceKind::OlapColumnar,
                engine_kind: DataServiceKind::OltpRelational,
            },
        ];
        for e in &errors {
            assert!(!e.message().is_empty(), "message empty for {:?}", e);
        }
    }
}
