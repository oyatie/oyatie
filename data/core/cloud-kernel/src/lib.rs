//! Cloud data-plane kernel (M03-P03-IP-001).
//!
//! Provider-neutral abstractions for OLTP/OLAP/object-store/queue
//! data-services. Defines DataServiceKind, ResidencyClass, and a
//! single `DataServicePlan` admission rule that residency + encryption
//! requirements are satisfied before an adapter is allowed to provision.
//! The `data_service` module adds the managed `DatabaseEngine` catalogue
//! and `DataService` aggregate (M03-P03-IP-001 `DataService`/`DatabaseEngine`).
//! The `streaming_partition` module adds provider-neutral shard admission
//! rules for stream data services (M06-P04-IP-002).

pub mod data_service;
pub mod streaming_partition;
pub use data_service::{DataService, DataServiceError, DatabaseEngine, provision_data_service};
pub use streaming_partition::{
    StreamingPartitionError, StreamingPartitionStrategy, admit_streaming_partition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DataServiceKind {
    OltpRelational,
    OltpKeyValue,
    OlapColumnar,
    Stream,
    ObjectStore,
    SearchIndex,
    Queue,
}

impl DataServiceKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::OltpRelational => "oltp-relational",
            Self::OltpKeyValue => "oltp-keyvalue",
            Self::OlapColumnar => "olap-columnar",
            Self::Stream => "stream",
            Self::ObjectStore => "object-store",
            Self::SearchIndex => "search-index",
            Self::Queue => "queue",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResidencyClass {
    Global,
    RegionBound,
    SovereignPack,
    FederatedPack,
    DedicatedPack,
}

impl ResidencyClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::RegionBound => "region-bound",
            Self::SovereignPack => "sovereign-pack",
            Self::FederatedPack => "federated-pack",
            Self::DedicatedPack => "dedicated-pack",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncryptionRequirement {
    NotRequired,
    AtRest,
    AtRestAndInTransit,
    AtRestAndInTransitAndAtUse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataServicePlan {
    // data_class: INTERNAL_ONLY
    pub plan_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: DataServiceKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub encryption: EncryptionRequirement, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterCapabilities {
    // data_class: INTERNAL_ONLY
    pub adapter_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: DataServiceKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub supports_residency: Vec<ResidencyClass>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub max_encryption: EncryptionRequirement, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKernelError {
    EmptyPlanId,
    EmptyAdapterId,
    KindMismatch {
        plan: DataServiceKind,
        adapter: DataServiceKind,
    },
    ResidencyUnsupported,
    EncryptionInsufficient {
        required: EncryptionRequirement,
        max_supported: EncryptionRequirement,
    },
}

impl DataKernelError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPlanId => "plan id is empty".to_owned(),
            Self::EmptyAdapterId => "adapter id is empty".to_owned(),
            Self::KindMismatch { plan, adapter } => format!(
                "service-kind mismatch: plan={} adapter={}",
                plan.name(),
                adapter.name()
            ),
            Self::ResidencyUnsupported => "adapter does not support requested residency".to_owned(),
            Self::EncryptionInsufficient {
                required,
                max_supported,
            } => format!(
                "encryption insufficient: required={:?} max_supported={:?}",
                required, max_supported
            ),
        }
    }
}

fn encryption_rank(e: EncryptionRequirement) -> u8 {
    match e {
        EncryptionRequirement::NotRequired => 0,
        EncryptionRequirement::AtRest => 1,
        EncryptionRequirement::AtRestAndInTransit => 2,
        EncryptionRequirement::AtRestAndInTransitAndAtUse => 3,
    }
}

pub fn admit_plan(
    plan: &DataServicePlan,
    adapter: &AdapterCapabilities,
) -> Result<(), DataKernelError> {
    if plan.plan_id.is_empty() {
        return Err(DataKernelError::EmptyPlanId);
    }
    if adapter.adapter_id.is_empty() {
        return Err(DataKernelError::EmptyAdapterId);
    }
    if plan.kind != adapter.kind {
        return Err(DataKernelError::KindMismatch {
            plan: plan.kind,
            adapter: adapter.kind,
        });
    }
    if !adapter.supports_residency.contains(&plan.residency) {
        return Err(DataKernelError::ResidencyUnsupported);
    }
    if encryption_rank(plan.encryption) > encryption_rank(adapter.max_encryption) {
        return Err(DataKernelError::EncryptionInsufficient {
            required: plan.encryption,
            max_supported: adapter.max_encryption,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        kind: DataServiceKind,
        res: ResidencyClass,
        enc: EncryptionRequirement,
    ) -> DataServicePlan {
        DataServicePlan {
            plan_id: "plan-1".into(),
            kind,
            residency: res,
            encryption: enc,
        }
    }

    fn adapter(
        kind: DataServiceKind,
        res: Vec<ResidencyClass>,
        enc: EncryptionRequirement,
    ) -> AdapterCapabilities {
        AdapterCapabilities {
            adapter_id: "aws-rds".into(),
            kind,
            supports_residency: res,
            max_encryption: enc,
        }
    }

    #[test]
    fn matching_plan_and_adapter_passes() {
        assert!(
            admit_plan(
                &plan(
                    DataServiceKind::OltpRelational,
                    ResidencyClass::SovereignPack,
                    EncryptionRequirement::AtRest
                ),
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::SovereignPack],
                    EncryptionRequirement::AtRestAndInTransit
                ),
            )
            .is_ok()
        );
    }

    #[test]
    fn kind_mismatch_rejected() {
        assert!(matches!(
            admit_plan(
                &plan(
                    DataServiceKind::OlapColumnar,
                    ResidencyClass::Global,
                    EncryptionRequirement::NotRequired
                ),
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::Global],
                    EncryptionRequirement::AtRest
                ),
            ),
            Err(DataKernelError::KindMismatch { .. })
        ));
    }

    #[test]
    fn residency_unsupported_rejected() {
        assert!(matches!(
            admit_plan(
                &plan(
                    DataServiceKind::OltpRelational,
                    ResidencyClass::SovereignPack,
                    EncryptionRequirement::AtRest
                ),
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::DedicatedPack],
                    EncryptionRequirement::AtRest
                ),
            ),
            Err(DataKernelError::ResidencyUnsupported)
        ));
    }

    #[test]
    fn encryption_insufficient_rejected() {
        assert!(matches!(
            admit_plan(
                &plan(
                    DataServiceKind::OltpRelational,
                    ResidencyClass::Global,
                    EncryptionRequirement::AtRestAndInTransitAndAtUse
                ),
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::Global],
                    EncryptionRequirement::AtRest
                ),
            ),
            Err(DataKernelError::EncryptionInsufficient { .. })
        ));
    }

    #[test]
    fn at_rest_satisfies_not_required() {
        assert!(
            admit_plan(
                &plan(
                    DataServiceKind::OltpRelational,
                    ResidencyClass::Global,
                    EncryptionRequirement::NotRequired
                ),
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::Global],
                    EncryptionRequirement::AtRest
                ),
            )
            .is_ok()
        );
    }

    #[test]
    fn empty_plan_id_rejected() {
        let mut p = plan(
            DataServiceKind::OltpRelational,
            ResidencyClass::Global,
            EncryptionRequirement::NotRequired,
        );
        p.plan_id = "".into();
        assert!(matches!(
            admit_plan(
                &p,
                &adapter(
                    DataServiceKind::OltpRelational,
                    vec![ResidencyClass::Global],
                    EncryptionRequirement::AtRest
                )
            ),
            Err(DataKernelError::EmptyPlanId)
        ));
    }

    #[test]
    fn kind_names_distinct() {
        use std::collections::HashSet;
        let names: HashSet<_> = [
            DataServiceKind::OltpRelational,
            DataServiceKind::OltpKeyValue,
            DataServiceKind::OlapColumnar,
            DataServiceKind::Stream,
            DataServiceKind::ObjectStore,
            DataServiceKind::SearchIndex,
            DataServiceKind::Queue,
        ]
        .iter()
        .map(|k| k.name())
        .collect();
        assert_eq!(names.len(), 7);
    }

    #[test]
    fn residency_names_distinct() {
        use std::collections::HashSet;
        let names: HashSet<_> = [
            ResidencyClass::Global,
            ResidencyClass::RegionBound,
            ResidencyClass::SovereignPack,
            ResidencyClass::FederatedPack,
            ResidencyClass::DedicatedPack,
        ]
        .iter()
        .map(|r| r.name())
        .collect();
        assert_eq!(names.len(), 5);
    }
}
