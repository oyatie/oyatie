//! The aggregate cloud surface.

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::CLOUD_SURFACE_SCHEMA_VERSION;
use crate::error::CloudSurfaceError;
use crate::ids::CloudSurfaceId;
use crate::surfaces_infra::{
    ComputeSurface, ComputeSurfaceCreate, NetworkSurface, NetworkSurfaceCreate, StorageSurface,
    StorageSurfaceCreate,
};
use crate::surfaces_platform::{
    BillingSurface, BillingSurfaceCreate, FinOpsSurface, FinOpsSurfaceCreate, IamSurface,
    IamSurfaceCreate, ObservabilitySurface, ObservabilitySurfaceCreate, RegionsSurface,
    RegionsSurfaceCreate,
};
use crate::validate::{public, public_class};

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

impl CloudSurface {
    pub fn new(input: CloudSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        Ok(Self {
            id: public(CloudSurfaceId::new(input.id)?),
            compute: public(ComputeSurface::new(input.compute)?),
            storage: public(StorageSurface::new(input.storage)?),
            network: public(NetworkSurface::new(input.network)?),
            iam: public(IamSurface::new(input.iam)?),
            regions: public(RegionsSurface::new(input.regions)?),
            billing: public(BillingSurface::new(input.billing)?),
            observability: public(ObservabilitySurface::new(input.observability)?),
            finops: public(FinOpsSurface::new(input.finops)?),
            data_class: public_class(input.data_class)?,
            schema_version: public(CLOUD_SURFACE_SCHEMA_VERSION),
        })
    }
}
