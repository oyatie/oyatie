use crate::{
    BoxCellFuture, CapacityVectorV1, Digest32, ImmutableEvidenceRefV1, PlacementContractError,
    PlacementReadAuthorityV1, ProofConstructionError,
};

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
                Err(ProofConstructionError::NotImplemented)
            }
        }
    };
}

opaque_id!(RealmId);
opaque_id!(JurisdictionId);
opaque_id!(RegionId);
opaque_id!(PoolId);
opaque_id!(TopologyAuthority);
opaque_id!(TopologyDomainKind);
opaque_id!(TopologyDomainId);
opaque_id!(CorrelationSetId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrelationSetRefV1 {
    pub authority: TopologyAuthority,
    pub identifier: CorrelationSetId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyDomainKeyV1 {
    pub authority: TopologyAuthority,
    pub kind: TopologyDomainKind,
    pub identifier: TopologyDomainId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyDomainRefV1 {
    pub key: TopologyDomainKeyV1,
    pub parent: Option<TopologyDomainKeyV1>,
    pub correlation_sets: Vec<CorrelationSetRefV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyInventoryV1 {
    pub source: ImmutableEvidenceRefV1,
    pub ordered_domain_root_digest: Digest32,
    pub domain_count: u64,
    pub ordered_correlation_set_root_digest: Digest32,
    pub correlation_set_count: u64,
    pub inventory_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyDomainMemberV1 {
    pub ordinal: u64,
    pub domain: TopologyDomainRefV1,
    pub member_digest: Digest32,
    pub inclusion_path: Vec<Digest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyPageTokenV1(Vec<u8>);

impl CellTopologyPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyPageRequestV1 {
    pub inventory: CellTopologyInventoryV1,
    pub page_size: u32,
    pub page_token: Option<CellTopologyPageTokenV1>,
    pub maximum_inclusion_path_depth: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyPageV1 {
    pub domains: Vec<CellTopologyDomainMemberV1>,
    pub next_page_token: Option<CellTopologyPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureScenarioV1 {
    pub unavailable_correlation_sets: Vec<CorrelationSetRefV1>,
    pub required_surviving_capacity: CapacityVectorV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResilienceObjectiveV1 {
    pub scope: PlacementLocationV1,
    pub declared_correlation_set_root_digest: Digest32,
    pub declared_correlation_set_count: u64,
    pub ordered_scenario_root_digest: Digest32,
    pub scenario_count: u64,
    pub coverage_proof_digest: Digest32,
    pub scenarios: Vec<FailureScenarioV1>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlacementPartitionV1 {
    pub realm: RealmId,
    pub jurisdiction: JurisdictionId,
    pub region: RegionId,
    pub pool: PoolId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementLocationV1 {
    pub realm: RealmId,
    pub jurisdiction: JurisdictionId,
    pub region: RegionId,
}

pub trait CellTopologyInventoryReader: Send + Sync {
    fn read_page<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        request: &'a CellTopologyPageRequestV1,
    ) -> BoxCellFuture<'a, Result<CellTopologyPageV1, PlacementContractError>>;
}
