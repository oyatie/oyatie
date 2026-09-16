use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSurfaceId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSkuId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FulfillmentPhase {
    PublicCloudConsumption,
    HybridColo,
    OwnedMegaDc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeSkuKind {
    ManagedKubernetes,
    Functions,
    VirtualMachine,
    BareMetalLease,
    Gpu,
    EdgeCompute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KubeTier {
    Standard,
    HighAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NodeClass {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
    Gpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FunctionRuntime {
    Rust,
    TypeScript,
    Python,
    Wasm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ColdStartClass {
    Interactive,
    Batch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmShape {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IsolationLevel {
    SharedCell,
    DedicatedCell,
    SovereignCell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RackClass {
    GeneralPurpose,
    StorageOptimized,
    GpuDense,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaseTerm {
    Monthly,
    OneYear,
    ThreeYear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AcceleratorClass {
    Inference,
    Training,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InterconnectClass {
    Pcie,
    Infiniband,
    EthernetRoce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PopClass {
    Regional,
    Metro,
    SovereignEdge,
}
