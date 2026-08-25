use cell_routing::{CellBinding, CellBindingCreate, CellRouter};
use network_residency::ResidencyClass;

use crate::model::{
    CloudAz, CloudAzCreate, CloudCell, CloudCellCreate, CloudCellState, CloudRegion,
    CloudRegionCatalog, CloudRegionCreate, CloudRegionError, TenantCellRouteRequest,
};
use crate::validation::{region_allows_residency, validate_tenant_id};
use crate::{AzCode, CellId, RegionCode};

impl CloudRegionCatalog {
    pub fn register_region(
        &mut self,
        input: CloudRegionCreate,
    ) -> Result<CloudRegion, CloudRegionError> {
        let region = CloudRegion::new(input)?;
        if self.regions.contains_key(&region.code.value) {
            return Err(CloudRegionError::DuplicateRegion);
        }
        self.regions
            .insert(region.code.value.clone(), region.clone());
        Ok(region)
    }

    pub fn register_az(&mut self, input: CloudAzCreate) -> Result<CloudAz, CloudRegionError> {
        let az = CloudAz::new(input)?;
        if self.azs.contains_key(&az.code.value) {
            return Err(CloudRegionError::DuplicateAz);
        }
        let region = self
            .regions
            .get_mut(&az.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        region.azs.value.push(az.code.value.clone());
        region.azs.value.sort();
        region.azs.value.dedup();
        self.azs.insert(az.code.value.clone(), az.clone());
        Ok(az)
    }

    pub fn register_cell(&mut self, input: CloudCellCreate) -> Result<CloudCell, CloudRegionError> {
        let cell = CloudCell::new(input)?;
        if self.cells.contains_key(&cell.id.value) {
            return Err(CloudRegionError::DuplicateCell);
        }
        let region = self
            .regions
            .get(&cell.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        let az = self
            .azs
            .get_mut(&cell.az_code.value)
            .ok_or(CloudRegionError::UnknownAz)?;
        if az.region_code.value != cell.region_code.value {
            return Err(CloudRegionError::CellAzMismatch);
        }
        for residency_class in &cell.allowed_residency.value {
            if !region_allows_residency(region, residency_class) {
                return Err(CloudRegionError::CellResidencyNotAllowedInRegion);
            }
        }
        az.cells.value.push(cell.id.value.clone());
        az.cells.value.sort();
        az.cells.value.dedup();
        self.cells.insert(cell.id.value.clone(), cell.clone());
        Ok(cell)
    }

    pub fn region(&self, code: &RegionCode) -> Option<&CloudRegion> {
        self.regions.get(code)
    }

    pub fn az(&self, code: &AzCode) -> Option<&CloudAz> {
        self.azs.get(code)
    }

    pub fn cell(&self, id: &CellId) -> Option<&CloudCell> {
        self.cells.get(id)
    }

    pub fn regions(&self) -> impl Iterator<Item = &CloudRegion> {
        self.regions.values()
    }

    pub fn azs_for_region<'a>(
        &'a self,
        region_code: &'a RegionCode,
    ) -> impl Iterator<Item = &'a CloudAz> + 'a {
        self.azs
            .values()
            .filter(move |az| &az.region_code.value == region_code)
    }

    pub fn cells_for_region<'a>(
        &'a self,
        region_code: &'a RegionCode,
    ) -> impl Iterator<Item = &'a CloudCell> + 'a {
        self.cells
            .values()
            .filter(move |cell| &cell.region_code.value == region_code)
    }

    pub fn route_for_tenant(
        &self,
        request: TenantCellRouteRequest,
    ) -> Result<CellBindingCreate, CloudRegionError> {
        validate_tenant_id(&request.tenant_id)?;
        let home_region = RegionCode::new(request.home_region_code)?;
        let region = self
            .regions
            .get(&home_region)
            .ok_or(CloudRegionError::UnknownRegion)?;
        if !region_allows_residency(region, &request.residency_class) {
            return Err(CloudRegionError::RegionResidencyMismatch);
        }
        let cell = self
            .cells
            .values()
            .find(|cell| {
                cell.region_code.value == home_region
                    && cell.state.value == CloudCellState::Active
                    && request
                        .required_density
                        .is_none_or(|density| density == cell.tenant_density.value)
                    && cell.allows_residency(&request.residency_class)
                    && cell.has_route_capacity()
            })
            .ok_or(CloudRegionError::NoCompatibleCell)?;
        self.binding_for_cell(request.tenant_id, request.residency_class, &cell.id.value)
    }

    pub fn binding_for_cell(
        &self,
        tenant_id: String,
        residency_class: ResidencyClass,
        cell_id: &CellId,
    ) -> Result<CellBindingCreate, CloudRegionError> {
        validate_tenant_id(&tenant_id)?;
        let cell = self
            .cells
            .get(cell_id)
            .ok_or(CloudRegionError::UnknownCell)?;
        let region = self
            .regions
            .get(&cell.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        if !region_allows_residency(region, &residency_class) {
            return Err(CloudRegionError::RegionResidencyMismatch);
        }
        if !cell.allows_residency(&residency_class) {
            return Err(CloudRegionError::CellResidencyDenied);
        }
        Ok(CellBindingCreate {
            tenant_id,
            region: region.region_ref.value.clone(),
            residency_class,
            az: cell.az_code.value.value.clone(),
            cell_id: cell.id.value.value.clone(),
            tier: cell.tenant_density.value.cell_tier(),
            hsm_partition_ref: cell.hsm_partition_ref.value.clone(),
        })
    }

    pub fn bind_route_for_tenant(
        &self,
        router: &mut CellRouter,
        request: TenantCellRouteRequest,
    ) -> Result<CellBinding, CloudRegionError> {
        let binding = self.route_for_tenant(request)?;
        router
            .bind(binding)
            .map_err(CloudRegionError::CellBindingRejected)
    }
}
