//! Foundry claim-ceiling kernel.

use intelligence_catalog_domain::{
    ApiStability, CatalogIndex, CatalogRecord, SecurityReview, SupplyChainAttestation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimCeilingError {
    ApiStabilityAboveFoundation,
    SecurityReviewAboveFoundation,
    SupplyChainAboveFoundation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FoundationClaimCeiling {
    max_api_stability: ApiStability,
    max_security_review: SecurityReview,
    max_supply_chain: SupplyChainAttestation,
}

impl FoundationClaimCeiling {
    pub fn preview_foundation() -> Self {
        Self {
            max_api_stability: ApiStability::Preview,
            max_security_review: SecurityReview::Unreviewed,
            max_supply_chain: SupplyChainAttestation::SourceOnly,
        }
    }

    pub fn validate_catalog(&self, catalog: &CatalogIndex) -> Result<(), ClaimCeilingError> {
        for record in catalog.records() {
            self.validate_record(record)?;
        }
        Ok(())
    }

    pub fn validate_record(&self, record: &CatalogRecord) -> Result<(), ClaimCeilingError> {
        if record.api_stability.value > self.max_api_stability {
            return Err(ClaimCeilingError::ApiStabilityAboveFoundation);
        }
        if record.security_review.value > self.max_security_review {
            return Err(ClaimCeilingError::SecurityReviewAboveFoundation);
        }
        if record.supply_chain.value > self.max_supply_chain {
            return Err(ClaimCeilingError::SupplyChainAboveFoundation);
        }
        Ok(())
    }
}
