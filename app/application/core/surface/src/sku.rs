//! Compute SKU taxonomy and the fulfillment phase it is served from.

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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeSku {
    ManagedKubernetes {
        tier: KubeTier,
        node_class: NodeClass,
    }, // data_class: PUBLIC
    Functions {
        runtime: FunctionRuntime,
        cold_start_class: ColdStartClass,
    }, // data_class: PUBLIC
    VirtualMachine {
        shape: VmShape,
        isolation: IsolationLevel,
    }, // data_class: PUBLIC
    BareMetalLease {
        rack_class: RackClass,
        term: LeaseTerm,
    }, // data_class: PUBLIC
    Gpu {
        accelerator: AcceleratorClass,
        interconnect: InterconnectClass,
    }, // data_class: PUBLIC
    EdgeCompute {
        pop_class: PopClass,
        latency_budget_ms: u16,
    }, // data_class: PUBLIC
}

impl ComputeSku {
    pub const fn kind(&self) -> ComputeSkuKind {
        match self {
            Self::ManagedKubernetes { .. } => ComputeSkuKind::ManagedKubernetes,
            Self::Functions { .. } => ComputeSkuKind::Functions,
            Self::VirtualMachine { .. } => ComputeSkuKind::VirtualMachine,
            Self::BareMetalLease { .. } => ComputeSkuKind::BareMetalLease,
            Self::Gpu { .. } => ComputeSkuKind::Gpu,
            Self::EdgeCompute { .. } => ComputeSkuKind::EdgeCompute,
        }
    }
}
