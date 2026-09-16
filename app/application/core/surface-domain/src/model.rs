use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudSurfaceCreate {
    pub id: String,                                // data_class: PUBLIC
    pub compute: ComputeSurfaceCreate,             // data_class: PUBLIC
    pub storage: StorageSurfaceCreate,             // data_class: PUBLIC
    pub network: NetworkSurfaceCreate,             // data_class: PUBLIC
    pub iam: IamSurfaceCreate,                     // data_class: PUBLIC
    pub regions: RegionsSurfaceCreate,             // data_class: PUBLIC
    pub billing: BillingSurfaceCreate,             // data_class: PUBLIC
    pub observability: ObservabilitySurfaceCreate, // data_class: PUBLIC
    pub finops: FinOpsSurfaceCreate,               // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudSurface {
    pub id: Classified<CloudSurfaceId>,      // data_class: PUBLIC
    pub compute: Classified<ComputeSurface>, // data_class: PUBLIC
    pub storage: Classified<StorageSurface>, // data_class: PUBLIC
    pub network: Classified<NetworkSurface>, // data_class: PUBLIC
    pub iam: Classified<IamSurface>,         // data_class: PUBLIC
    pub regions: Classified<RegionsSurface>, // data_class: PUBLIC
    pub billing: Classified<BillingSurface>, // data_class: PUBLIC
    pub observability: Classified<ObservabilitySurface>, // data_class: PUBLIC
    pub finops: Classified<FinOpsSurface>,   // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudSurfaceError {
    InvalidSurfaceId,
    InvalidSkuId,
    InvalidProviderRef,
    InvalidDataClass,
    InvalidFulfillment,
    MissingComputeSkuKind,
    DuplicateComputeSku,
    MissingStorageSurface,
    MissingNetworkSurface,
    InvalidIamSurface,
    InvalidRegionsSurface,
    InvalidBillingSurface,
    InvalidObservabilitySurface,
    InvalidFinOpsSurface,
}
