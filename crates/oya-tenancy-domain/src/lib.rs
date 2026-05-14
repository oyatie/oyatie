//! Tenant kernel: tenant identity, residency, and regional-pack binding.

use oya_data_boundary_kernel::{Classified, DataClass};
use oya_residency_domain::{ResidencyClass, residency_class_allows_home_region_label};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tenant {
    pub id: String, // data_class: INTERNAL_ONLY
    pub legal_name: Classified<String>,
    pub home_region: Classified<String>,
    pub residency_class: Classified<ResidencyClass>,
    pub regulatory_packs: Classified<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantError {
    InvalidTenantId,
    EmptyLegalName,
    EmptyHomeRegion,
    HomeRegionNotAllowedForResidency,
    MissingRegionalPack,
}

impl Tenant {
    pub fn new(
        id: String,
        legal_name: String,
        home_region: String,
        residency_class: ResidencyClass,
        regulatory_packs: Vec<String>,
    ) -> Result<Self, TenantError> {
        if !id.starts_with("ten_") || id.len() <= 4 {
            return Err(TenantError::InvalidTenantId);
        }
        if legal_name.trim().is_empty() {
            return Err(TenantError::EmptyLegalName);
        }
        if home_region.trim().is_empty() {
            return Err(TenantError::EmptyHomeRegion);
        }
        if !residency_class_allows_home_region_label(&residency_class, &home_region) {
            return Err(TenantError::HomeRegionNotAllowedForResidency);
        }
        if regulatory_packs.is_empty() {
            return Err(TenantError::MissingRegionalPack);
        }

        Ok(Self {
            id,
            legal_name: Classified::new(legal_name, DataClass::InternalOnly),
            home_region: Classified::new(home_region, DataClass::InternalOnly),
            residency_class: Classified::new(residency_class, DataClass::InternalOnly),
            regulatory_packs: Classified::new(regulatory_packs, DataClass::InternalOnly),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_identity_includes_residency_class() {
        let tenant = Tenant::new(
            "ten_kr".to_string(),
            "KR Tenant".to_string(),
            "kr-seoul".to_string(),
            ResidencyClass::StrictKr,
            vec!["oya-pack-kr".to_string()],
        )
        .expect("KR tenant residency is valid");

        assert_eq!(tenant.residency_class.value.label(), Some("strict_kr"));
    }

    #[test]
    fn tenant_rejects_residency_home_region_mismatch() {
        let error = Tenant::new(
            "ten_us".to_string(),
            "US Tenant".to_string(),
            "us-east".to_string(),
            ResidencyClass::StrictKr,
            vec!["oya-pack-kr".to_string()],
        )
        .expect_err("strict KR tenants require KR home region");

        assert_eq!(error, TenantError::HomeRegionNotAllowedForResidency);
    }
}
