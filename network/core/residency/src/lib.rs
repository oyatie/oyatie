//! Platform residency kernel.
//!
//! Canonical ADR-0049 value contracts for per-pack residency defaults, immutable
//! tenant residency binding, cross-region transfer permits, and recreate-based
//! residency change planning. This crate owns typed invariants only; tenant,
//! cloud, Workspace, and trust-portal apps own persistence and orchestration.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const REGION_REF_SCHEMA_VERSION: u32 = 1;
const REGULATOR_OVERLAY_SCHEMA_VERSION: u32 = 1;
const PER_PACK_RESIDENCY_SCHEMA_VERSION: u32 = 1;
const REGIONAL_PACK_RESIDENCY_SCHEMA_VERSION: u32 = 1;
const TENANT_RESIDENCY_BINDING_SCHEMA_VERSION: u32 = 1;
const RESIDENCY_CHANGE_PLAN_SCHEMA_VERSION: u32 = 1;
const CROSS_REGION_TRANSFER_PERMIT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResidencyError {
    InvalidTenantId,
    InvalidRegionId,
    InvalidCellGroupRef,
    InvalidPackId,
    InvalidEvidenceRef,
    InvalidRegulatorRef,
    EmptyRegulatorSet,
    DuplicateRegulatorRef,
    EmptyResidencyClassSet,
    DuplicateResidencyClass,
    DefaultResidencyNotAllowed,
    EmptyRegionSet,
    DuplicateRegion,
    ForbiddenRegionOverlap,
    ResidencyAlreadyBound,
    ResidencyChangeRequiresNewTenant,
    ResidencyChangeRequiresDifferentTarget,
    InvalidMigrationPlanRef,
    InvalidDsrId,
    InvalidDeletionCertificateRef,
    InvalidLegalBasisRef,
    InvalidConsentReceiptRef,
    MissingConsentReceipt,
    InvalidCedarPolicyRef,
    InvalidMtlsPolicyRef,
    InvalidHsmPartitionRef,
    InvalidAuditEventRef,
    InvalidTrustPortalEntryRef,
    InvalidRegionPair,
    SourceRegionNotAllowed,
    DestinationRegionNotAllowed,
    PurposeNotAllowed,
    DataClassDeniedForResidency,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RegionJurisdiction {
    Home,
    Federated,
    Recovery,
    Expansion,
    MarketAlpha,
    MarketBeta,
    MarketGamma,
    MarketDelta,
    MarketEpsilon,
    MarketZeta,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrossRegionTransferPurpose {
    DisasterRecovery,
    Backup,
    TenantRequestedReplication,
    AnalyticsReplica,
    DsrCascade,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RegulatorOverlayCreate {
    pub regulator_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RegulatorOverlay {
    pub regulator_refs: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>,        // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PerPackResidencyCreate {
    pub allowed_primary_regions: Vec<String>, // data_class: INTERNAL_ONLY
    pub allowed_replica_regions: Vec<String>, // data_class: INTERNAL_ONLY
    pub forbidden_regions: Vec<String>,       // data_class: INTERNAL_ONLY
    pub regulator_overlay: RegulatorOverlay,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PerPackResidency {
    pub allowed_primary_regions: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub allowed_replica_regions: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub forbidden_regions: Classified<Vec<String>>,       // data_class: INTERNAL_ONLY
    pub regulator_overlay: Classified<RegulatorOverlay>,  // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResidencyClass {
    StrictHomeRegion,
    HomeWithRecoveryFailover,
    Global,
    PerPack(Box<PerPackResidency>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionRefCreate {
    pub region_id: String,                // data_class: INTERNAL_ONLY
    pub jurisdiction: RegionJurisdiction, // data_class: INTERNAL_ONLY
    pub cell_group_ref: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RegionRef {
    pub region_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<RegionJurisdiction>, // data_class: INTERNAL_ONLY
    pub cell_group_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackResidencyDefaultCreate {
    pub pack_id: String,                                // data_class: INTERNAL_ONLY
    pub home_region: RegionRef,                         // data_class: INTERNAL_ONLY
    pub default_residency_class: ResidencyClass,        // data_class: INTERNAL_ONLY
    pub allowed_residency_classes: Vec<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub regulator_overlay: RegulatorOverlay,            // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackResidencyDefault {
    pub pack_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub home_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub default_residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub allowed_residency_classes: Classified<Vec<ResidencyClass>>, // data_class: INTERNAL_ONLY
    pub regulator_overlay: Classified<RegulatorOverlay>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>,   // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantResidencyBindingCreate {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub primary_region: RegionRef,       // data_class: INTERNAL_ONLY
    pub residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    pub regional_pack_id: String,        // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub bound_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantResidencyBinding {
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub primary_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub regional_pack_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub bound_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyChangePlanCreate {
    pub old_binding: TenantResidencyBinding, // data_class: INTERNAL_ONLY
    pub new_tenant_id: String,               // data_class: INTERNAL_ONLY
    pub target_primary_region: RegionRef,    // data_class: INTERNAL_ONLY
    pub target_residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    pub migration_plan_ref: String,          // data_class: INTERNAL_ONLY
    pub dsr_id: String,                      // data_class: INTERNAL_ONLY
    pub deletion_certificate_ref: String,    // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyChangePlan {
    pub old_tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub new_tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub old_primary_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub target_primary_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub old_residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub target_residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub migration_plan_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub deletion_certificate_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossRegionTransferPermitCreate {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub residency_class: ResidencyClass,       // data_class: INTERNAL_ONLY
    pub source_region: RegionRef,              // data_class: INTERNAL_ONLY
    pub destination_region: RegionRef,         // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,          // data_class: INTERNAL_ONLY
    pub purpose: CrossRegionTransferPurpose,   // data_class: INTERNAL_ONLY
    pub legal_basis_ref: String,               // data_class: INTERNAL_ONLY
    pub consent_receipt_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub cedar_policy_ref: String,              // data_class: INTERNAL_ONLY
    pub mtls_policy_ref: String,               // data_class: INTERNAL_ONLY
    pub destination_hsm_partition_ref: String, // data_class: INTERNAL_ONLY
    pub audit_event_ref: String,               // data_class: INTERNAL_ONLY
    pub trust_portal_entry_ref: String,        // data_class: INTERNAL_ONLY
    pub permitted_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossRegionTransferPermit {
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub source_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub destination_region: Classified<RegionRef>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub purpose: Classified<CrossRegionTransferPurpose>, // data_class: INTERNAL_ONLY
    pub legal_basis_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub consent_receipt_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub cedar_policy_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub mtls_policy_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub destination_hsm_partition_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub audit_event_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub trust_portal_entry_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub permitted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantResidencyRegistry {
    bindings: BTreeMap<String, TenantResidencyBinding>,
}

impl RegulatorOverlay {
    pub fn new(input: RegulatorOverlayCreate) -> Result<Self, ResidencyError> {
        validate_non_empty(&input.evidence_ref, ResidencyError::InvalidEvidenceRef)?;
        validate_non_empty_set(
            &input.regulator_refs,
            ResidencyError::EmptyRegulatorSet,
            ResidencyError::InvalidRegulatorRef,
            ResidencyError::DuplicateRegulatorRef,
        )?;
        Ok(Self {
            regulator_refs: internal(input.regulator_refs),
            evidence_ref: internal(input.evidence_ref),
            schema_version: internal(REGULATOR_OVERLAY_SCHEMA_VERSION),
        })
    }
}

impl PerPackResidency {
    pub fn new(input: PerPackResidencyCreate) -> Result<Self, ResidencyError> {
        validate_region_set(&input.allowed_primary_regions)?;
        validate_region_set(&input.allowed_replica_regions)?;
        validate_region_set_allow_empty(&input.forbidden_regions)?;
        ensure_no_forbidden_overlap(&input)?;
        Ok(Self {
            allowed_primary_regions: internal(input.allowed_primary_regions),
            allowed_replica_regions: internal(input.allowed_replica_regions),
            forbidden_regions: internal(input.forbidden_regions),
            regulator_overlay: internal(input.regulator_overlay),
            schema_version: internal(PER_PACK_RESIDENCY_SCHEMA_VERSION),
        })
    }

    pub fn allows_primary_region(&self, region: &RegionRef) -> bool {
        self.allowed_primary_regions
            .value
            .contains(&region.region_id.value)
            && !self
                .forbidden_regions
                .value
                .contains(&region.region_id.value)
    }

    pub fn allows_replica_region(&self, region: &RegionRef) -> bool {
        self.allowed_replica_regions
            .value
            .contains(&region.region_id.value)
            && !self
                .forbidden_regions
                .value
                .contains(&region.region_id.value)
    }
}

impl RegionRef {
    pub fn new(input: RegionRefCreate) -> Result<Self, ResidencyError> {
        validate_non_empty(&input.region_id, ResidencyError::InvalidRegionId)?;
        validate_non_empty(&input.cell_group_ref, ResidencyError::InvalidCellGroupRef)?;
        Ok(Self {
            region_id: internal(input.region_id),
            jurisdiction: internal(input.jurisdiction),
            cell_group_ref: internal(input.cell_group_ref),
            schema_version: internal(REGION_REF_SCHEMA_VERSION),
        })
    }
}

impl RegionalPackResidencyDefault {
    pub fn new(input: RegionalPackResidencyDefaultCreate) -> Result<Self, ResidencyError> {
        validate_pack_id(&input.pack_id)?;
        validate_non_empty(&input.evidence_ref, ResidencyError::InvalidEvidenceRef)?;
        validate_residency_classes(&input.allowed_residency_classes)?;
        if !input
            .allowed_residency_classes
            .contains(&input.default_residency_class)
        {
            return Err(ResidencyError::DefaultResidencyNotAllowed);
        }
        Ok(Self {
            pack_id: internal(input.pack_id),
            home_region: internal(input.home_region),
            default_residency_class: internal(input.default_residency_class),
            allowed_residency_classes: internal(input.allowed_residency_classes),
            regulator_overlay: internal(input.regulator_overlay),
            evidence_ref: internal(input.evidence_ref),
            schema_version: internal(REGIONAL_PACK_RESIDENCY_SCHEMA_VERSION),
        })
    }
}

impl TenantResidencyBinding {
    pub fn new(input: TenantResidencyBindingCreate) -> Result<Self, ResidencyError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_pack_id(&input.regional_pack_id)?;
        validate_non_empty(&input.evidence_ref, ResidencyError::InvalidEvidenceRef)?;
        validate_primary_region_for_class(&input.residency_class, &input.primary_region)?;
        Ok(Self {
            tenant_id: internal(input.tenant_id),
            primary_region: internal(input.primary_region),
            residency_class: internal(input.residency_class),
            regional_pack_id: internal(input.regional_pack_id),
            evidence_ref: internal(input.evidence_ref),
            bound_at_epoch_seconds: internal(input.bound_at_epoch_seconds),
            schema_version: internal(TENANT_RESIDENCY_BINDING_SCHEMA_VERSION),
        })
    }
}

impl ResidencyChangePlan {
    pub fn new(input: ResidencyChangePlanCreate) -> Result<Self, ResidencyError> {
        validate_tenant_id(&input.new_tenant_id)?;
        validate_non_empty(
            &input.migration_plan_ref,
            ResidencyError::InvalidMigrationPlanRef,
        )?;
        validate_non_empty(&input.dsr_id, ResidencyError::InvalidDsrId)?;
        validate_non_empty(
            &input.deletion_certificate_ref,
            ResidencyError::InvalidDeletionCertificateRef,
        )?;
        validate_time_order(
            input.old_binding.bound_at_epoch_seconds.value,
            input.requested_at_epoch_seconds,
        )?;
        if input.new_tenant_id == input.old_binding.tenant_id.value {
            return Err(ResidencyError::ResidencyChangeRequiresNewTenant);
        }
        if input.target_primary_region == input.old_binding.primary_region.value
            && input.target_residency_class == input.old_binding.residency_class.value
        {
            return Err(ResidencyError::ResidencyChangeRequiresDifferentTarget);
        }
        validate_primary_region_for_class(
            &input.target_residency_class,
            &input.target_primary_region,
        )?;
        Ok(Self {
            old_tenant_id: internal(input.old_binding.tenant_id.value),
            new_tenant_id: internal(input.new_tenant_id),
            old_primary_region: internal(input.old_binding.primary_region.value),
            target_primary_region: internal(input.target_primary_region),
            old_residency_class: internal(input.old_binding.residency_class.value),
            target_residency_class: internal(input.target_residency_class),
            migration_plan_ref: internal(input.migration_plan_ref),
            dsr_id: internal(input.dsr_id),
            deletion_certificate_ref: internal(input.deletion_certificate_ref),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: internal(RESIDENCY_CHANGE_PLAN_SCHEMA_VERSION),
        })
    }
}

impl CrossRegionTransferPermit {
    pub fn new(input: CrossRegionTransferPermitCreate) -> Result<Self, ResidencyError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_non_empty(&input.legal_basis_ref, ResidencyError::InvalidLegalBasisRef)?;
        validate_optional_non_empty(
            input.consent_receipt_ref.as_deref(),
            ResidencyError::InvalidConsentReceiptRef,
        )?;
        validate_non_empty(
            &input.cedar_policy_ref,
            ResidencyError::InvalidCedarPolicyRef,
        )?;
        validate_non_empty(&input.mtls_policy_ref, ResidencyError::InvalidMtlsPolicyRef)?;
        validate_non_empty(
            &input.destination_hsm_partition_ref,
            ResidencyError::InvalidHsmPartitionRef,
        )?;
        validate_non_empty(&input.audit_event_ref, ResidencyError::InvalidAuditEventRef)?;
        validate_non_empty(
            &input.trust_portal_entry_ref,
            ResidencyError::InvalidTrustPortalEntryRef,
        )?;
        if input.source_region.region_id.value == input.destination_region.region_id.value {
            return Err(ResidencyError::InvalidRegionPair);
        }
        validate_transfer_for_residency(&input)?;
        Ok(Self {
            tenant_id: internal(input.tenant_id),
            residency_class: internal(input.residency_class),
            source_region: internal(input.source_region),
            destination_region: internal(input.destination_region),
            data_class: internal(input.data_class),
            purpose: internal(input.purpose),
            legal_basis_ref: internal(input.legal_basis_ref),
            consent_receipt_ref: internal(input.consent_receipt_ref),
            cedar_policy_ref: internal(input.cedar_policy_ref),
            mtls_policy_ref: internal(input.mtls_policy_ref),
            destination_hsm_partition_ref: internal(input.destination_hsm_partition_ref),
            audit_event_ref: internal(input.audit_event_ref),
            trust_portal_entry_ref: internal(input.trust_portal_entry_ref),
            permitted_at_epoch_seconds: internal(input.permitted_at_epoch_seconds),
            schema_version: internal(CROSS_REGION_TRANSFER_PERMIT_SCHEMA_VERSION),
        })
    }
}

impl TenantResidencyRegistry {
    pub fn bind(
        &mut self,
        input: TenantResidencyBindingCreate,
    ) -> Result<TenantResidencyBinding, ResidencyError> {
        if self.bindings.contains_key(&input.tenant_id) {
            return Err(ResidencyError::ResidencyAlreadyBound);
        }
        let binding = TenantResidencyBinding::new(input)?;
        self.bindings
            .insert(binding.tenant_id.value.clone(), binding.clone());
        Ok(binding)
    }

    pub fn get(&self, tenant_id: &str) -> Option<&TenantResidencyBinding> {
        self.bindings.get(tenant_id)
    }
}

impl ResidencyClass {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            Self::StrictHomeRegion => Some("strict_home_region"),
            Self::HomeWithRecoveryFailover => Some("home_with_recovery_failover"),
            Self::Global => Some("global"),
            Self::PerPack(_) => None,
        }
    }
}

pub fn parse_residency_class_label(label: &str) -> Option<ResidencyClass> {
    match label.trim() {
        "strict_home_region" => Some(ResidencyClass::StrictHomeRegion),
        "home_with_recovery_failover" => Some(ResidencyClass::HomeWithRecoveryFailover),
        "global" => Some(ResidencyClass::Global),
        _ => None,
    }
}

pub fn residency_class_allows_home_region_label(
    residency_class: &ResidencyClass,
    home_region: &str,
) -> bool {
    let normalized = home_region.trim().to_ascii_lowercase();
    match residency_class {
        ResidencyClass::StrictHomeRegion | ResidencyClass::HomeWithRecoveryFailover => {
            matches_region_family(&normalized, "region-home")
        }
        ResidencyClass::Global => !normalized.is_empty(),
        ResidencyClass::PerPack(per_pack) => per_pack
            .allowed_primary_regions
            .value
            .iter()
            .any(|region| region.eq_ignore_ascii_case(home_region.trim())),
    }
}

pub fn infer_region_jurisdiction_label(region_id: &str) -> RegionJurisdiction {
    let normalized = region_id.trim().to_ascii_lowercase();
    if matches_region_family(&normalized, "region-home") {
        RegionJurisdiction::Home
    } else if matches_region_family(&normalized, "region-federated") {
        RegionJurisdiction::Federated
    } else if matches_region_family(&normalized, "region-recovery") {
        RegionJurisdiction::Recovery
    } else if matches_region_family(&normalized, "region-expansion") {
        RegionJurisdiction::Expansion
    } else if matches_region_family(&normalized, "region-market-alpha") {
        RegionJurisdiction::MarketAlpha
    } else if matches_region_family(&normalized, "region-market-beta") {
        RegionJurisdiction::MarketBeta
    } else if matches_region_family(&normalized, "region-market-gamma") {
        RegionJurisdiction::MarketGamma
    } else if matches_region_family(&normalized, "region-market-delta") {
        RegionJurisdiction::MarketDelta
    } else if matches_region_family(&normalized, "region-market-epsilon") {
        RegionJurisdiction::MarketEpsilon
    } else if matches_region_family(&normalized, "region-market-zeta") {
        RegionJurisdiction::MarketZeta
    } else {
        RegionJurisdiction::Other
    }
}

fn matches_region_family(normalized_region: &str, family: &str) -> bool {
    normalized_region == family
        || normalized_region
            .strip_prefix(family)
            .is_some_and(|suffix| suffix.starts_with('-'))
}

pub fn residency_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, ResidencyError> {
    PrivacyDataClass::new(data_class).map_err(|_| ResidencyError::InvalidDataClass)
}

fn validate_transfer_for_residency(
    input: &CrossRegionTransferPermitCreate,
) -> Result<(), ResidencyError> {
    match &input.residency_class {
        ResidencyClass::StrictHomeRegion => validate_strict_home_region_transfer(input),
        ResidencyClass::HomeWithRecoveryFailover => validate_home_with_recovery_transfer(input),
        ResidencyClass::Global => validate_global_transfer(input),
        ResidencyClass::PerPack(per_pack) => validate_per_pack_transfer(input, per_pack),
    }
}

fn validate_strict_home_region_transfer(
    input: &CrossRegionTransferPermitCreate,
) -> Result<(), ResidencyError> {
    if input.source_region.jurisdiction.value != RegionJurisdiction::Home {
        return Err(ResidencyError::SourceRegionNotAllowed);
    }
    if input.destination_region.jurisdiction.value != RegionJurisdiction::Home {
        return Err(ResidencyError::DestinationRegionNotAllowed);
    }
    if !matches!(
        input.purpose,
        CrossRegionTransferPurpose::DisasterRecovery | CrossRegionTransferPurpose::Backup
    ) {
        return Err(ResidencyError::PurposeNotAllowed);
    }
    Ok(())
}

fn validate_home_with_recovery_transfer(
    input: &CrossRegionTransferPermitCreate,
) -> Result<(), ResidencyError> {
    if input.source_region.jurisdiction.value != RegionJurisdiction::Home {
        return Err(ResidencyError::SourceRegionNotAllowed);
    }
    if input.destination_region.jurisdiction.value != RegionJurisdiction::Recovery {
        return Err(ResidencyError::DestinationRegionNotAllowed);
    }
    if !matches!(
        input.purpose,
        CrossRegionTransferPurpose::DisasterRecovery | CrossRegionTransferPurpose::Backup
    ) {
        return Err(ResidencyError::PurposeNotAllowed);
    }
    if is_always_denied_for_home_recovery(input.data_class.data_class()) {
        return Err(ResidencyError::DataClassDeniedForResidency);
    }
    require_consent(input)
}

fn validate_global_transfer(input: &CrossRegionTransferPermitCreate) -> Result<(), ResidencyError> {
    require_consent(input)
}

fn validate_per_pack_transfer(
    input: &CrossRegionTransferPermitCreate,
    per_pack: &PerPackResidency,
) -> Result<(), ResidencyError> {
    if !per_pack.allows_primary_region(&input.source_region) {
        return Err(ResidencyError::SourceRegionNotAllowed);
    }
    if !per_pack.allows_replica_region(&input.destination_region) {
        return Err(ResidencyError::DestinationRegionNotAllowed);
    }
    require_consent(input)
}

fn validate_primary_region_for_class(
    residency_class: &ResidencyClass,
    primary_region: &RegionRef,
) -> Result<(), ResidencyError> {
    match residency_class {
        ResidencyClass::StrictHomeRegion | ResidencyClass::HomeWithRecoveryFailover => {
            if primary_region.jurisdiction.value == RegionJurisdiction::Home {
                Ok(())
            } else {
                Err(ResidencyError::SourceRegionNotAllowed)
            }
        }
        ResidencyClass::Global => Ok(()),
        ResidencyClass::PerPack(per_pack) => {
            if per_pack.allows_primary_region(primary_region) {
                Ok(())
            } else {
                Err(ResidencyError::SourceRegionNotAllowed)
            }
        }
    }
}

fn require_consent(input: &CrossRegionTransferPermitCreate) -> Result<(), ResidencyError> {
    if input.consent_receipt_ref.is_none() {
        Err(ResidencyError::MissingConsentReceipt)
    } else {
        Ok(())
    }
}

fn is_always_denied_for_home_recovery(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
    )
}

fn validate_residency_classes(classes: &[ResidencyClass]) -> Result<(), ResidencyError> {
    if classes.is_empty() {
        return Err(ResidencyError::EmptyResidencyClassSet);
    }
    let mut seen = BTreeSet::new();
    for class in classes {
        if !seen.insert(class) {
            return Err(ResidencyError::DuplicateResidencyClass);
        }
    }
    Ok(())
}

fn validate_region_set(regions: &[String]) -> Result<(), ResidencyError> {
    if regions.is_empty() {
        return Err(ResidencyError::EmptyRegionSet);
    }
    validate_region_set_allow_empty(regions)
}

fn validate_region_set_allow_empty(regions: &[String]) -> Result<(), ResidencyError> {
    let mut seen = BTreeSet::new();
    for region in regions {
        validate_non_empty(region, ResidencyError::InvalidRegionId)?;
        if !seen.insert(region.as_str()) {
            return Err(ResidencyError::DuplicateRegion);
        }
    }
    Ok(())
}

fn ensure_no_forbidden_overlap(input: &PerPackResidencyCreate) -> Result<(), ResidencyError> {
    let forbidden = input
        .forbidden_regions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for region in input
        .allowed_primary_regions
        .iter()
        .chain(input.allowed_replica_regions.iter())
    {
        if forbidden.contains(region.as_str()) {
            return Err(ResidencyError::ForbiddenRegionOverlap);
        }
    }
    Ok(())
}

fn validate_non_empty_set(
    values: &[String],
    empty_error: ResidencyError,
    invalid_error: ResidencyError,
    duplicate_error: ResidencyError,
) -> Result<(), ResidencyError> {
    if values.is_empty() {
        return Err(empty_error);
    }
    let mut seen = BTreeSet::new();
    for value in values {
        validate_non_empty(value, invalid_error.clone())?;
        if !seen.insert(value.as_str()) {
            return Err(duplicate_error);
        }
    }
    Ok(())
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), ResidencyError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > 4 {
        Ok(())
    } else {
        Err(ResidencyError::InvalidTenantId)
    }
}

fn validate_pack_id(pack_id: &str) -> Result<(), ResidencyError> {
    if pack_id.starts_with("pack-") && pack_id.len() > "pack-".len() {
        Ok(())
    } else {
        Err(ResidencyError::InvalidPackId)
    }
}

fn validate_non_empty(value: &str, error: ResidencyError) -> Result<(), ResidencyError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_optional_non_empty(
    value: Option<&str>,
    error: ResidencyError,
) -> Result<(), ResidencyError> {
    match value {
        Some(value) => validate_non_empty(value, error),
        None => Ok(()),
    }
}

fn validate_time_order(first: u64, second: u64) -> Result<(), ResidencyError> {
    if first <= second {
        Ok(())
    } else {
        Err(ResidencyError::InvalidTimeOrder)
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, internal_data_class())
}

fn internal_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use the infallible kernel constructor; the previous
    // `.expect()` proved a statically known invariant that the kernel now
    // encodes at the type level.
    PrivacyDataClass::internal_only()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).expect("test fixture uses privacy class")
    }

    fn region(region_id: &str, jurisdiction: RegionJurisdiction) -> RegionRef {
        RegionRef::new(RegionRefCreate {
            region_id: region_id.to_string(),
            jurisdiction,
            cell_group_ref: format!("cells/{region_id}"),
        })
        .expect("region fixture is valid")
    }

    fn kr_primary() -> RegionRef {
        region("region-home-1", RegionJurisdiction::Home)
    }

    fn kr_secondary() -> RegionRef {
        region("region-home-2", RegionJurisdiction::Home)
    }

    fn us_warm() -> RegionRef {
        region("region-recovery-1", RegionJurisdiction::Recovery)
    }

    fn regulator_overlay() -> RegulatorOverlay {
        RegulatorOverlay::new(RegulatorOverlayCreate {
            regulator_refs: vec!["regulator-alpha".to_string(), "regulator-beta".to_string()],
            evidence_ref: "regulator-overlay/alpha".to_string(),
        })
        .expect("regulator overlay fixture is valid")
    }

    fn binding_create() -> TenantResidencyBindingCreate {
        TenantResidencyBindingCreate {
            tenant_id: "ten_1".to_string(),
            primary_region: kr_primary(),
            residency_class: ResidencyClass::HomeWithRecoveryFailover,
            regional_pack_id: "pack-alpha".to_string(),
            evidence_ref: "residency/binding/ten_1".to_string(),
            bound_at_epoch_seconds: 100,
        }
    }

    fn permit_create(residency_class: ResidencyClass) -> CrossRegionTransferPermitCreate {
        CrossRegionTransferPermitCreate {
            tenant_id: "ten_1".to_string(),
            residency_class,
            source_region: kr_primary(),
            destination_region: us_warm(),
            data_class: privacy(DataClass::PiiIdentifying),
            purpose: CrossRegionTransferPurpose::DisasterRecovery,
            legal_basis_ref: "legal/pack-primary-transfer".to_string(),
            consent_receipt_ref: Some("consent/receipt-1".to_string()),
            cedar_policy_ref: "cedar/residency/home-recovery".to_string(),
            mtls_policy_ref: "mesh/mtls/cross-cell".to_string(),
            destination_hsm_partition_ref: "hsm/region-recovery-1/tenant-1".to_string(),
            audit_event_ref: "audit/cross-region/1".to_string(),
            trust_portal_entry_ref: "trust-portal/residency/1".to_string(),
            permitted_at_epoch_seconds: 120,
        }
    }

    #[test]
    fn regional_pack_default_must_be_in_allowed_residency_classes() {
        let default = RegionalPackResidencyDefault::new(RegionalPackResidencyDefaultCreate {
            pack_id: "pack-alpha".to_string(),
            home_region: kr_primary(),
            default_residency_class: ResidencyClass::StrictHomeRegion,
            allowed_residency_classes: vec![
                ResidencyClass::StrictHomeRegion,
                ResidencyClass::HomeWithRecoveryFailover,
            ],
            regulator_overlay: regulator_overlay(),
            evidence_ref: "regional-pack/alpha/residency".to_string(),
        })
        .expect("default class is allowed");
        assert_eq!(default.pack_id.value, "pack-alpha");

        let error = RegionalPackResidencyDefault::new(RegionalPackResidencyDefaultCreate {
            pack_id: "pack-alpha".to_string(),
            home_region: kr_primary(),
            default_residency_class: ResidencyClass::Global,
            allowed_residency_classes: vec![ResidencyClass::StrictHomeRegion],
            regulator_overlay: regulator_overlay(),
            evidence_ref: "regional-pack/alpha/residency".to_string(),
        })
        .expect_err("default class must be allowed");
        assert_eq!(error, ResidencyError::DefaultResidencyNotAllowed);
    }

    #[test]
    fn canonical_residency_labels_are_pack_neutral() {
        assert_eq!(
            ResidencyClass::StrictHomeRegion.label(),
            Some("strict_home_region")
        );
        assert_eq!(
            ResidencyClass::HomeWithRecoveryFailover.label(),
            Some("home_with_recovery_failover")
        );
    }

    #[test]
    fn tenant_residency_registry_rejects_rebinding() {
        let mut registry = TenantResidencyRegistry::default();
        let first = registry
            .bind(binding_create())
            .expect("first residency binding should succeed");
        assert_eq!(first.tenant_id.value, "ten_1");

        let error = registry
            .bind(binding_create())
            .expect_err("tenant residency is immutable post-create");
        assert_eq!(error, ResidencyError::ResidencyAlreadyBound);
    }

    #[test]
    fn residency_change_requires_new_tenant_dsr_and_deletion_certificate() {
        let old_binding =
            TenantResidencyBinding::new(binding_create()).expect("binding fixture is valid");
        let plan = ResidencyChangePlan::new(ResidencyChangePlanCreate {
            old_binding: old_binding.clone(),
            new_tenant_id: "ten_2".to_string(),
            target_primary_region: kr_primary(),
            target_residency_class: ResidencyClass::StrictHomeRegion,
            migration_plan_ref: "migration/ten-1-to-ten-2".to_string(),
            dsr_id: "dsr-residency-change-1".to_string(),
            deletion_certificate_ref: "certificate/deletion/ten_1".to_string(),
            requested_at_epoch_seconds: 200,
        })
        .expect("recreate residency change plan should build");
        assert_eq!(plan.old_tenant_id.value, "ten_1");
        assert_eq!(plan.new_tenant_id.value, "ten_2");

        let error = ResidencyChangePlan::new(ResidencyChangePlanCreate {
            old_binding,
            new_tenant_id: "ten_1".to_string(),
            target_primary_region: kr_primary(),
            target_residency_class: ResidencyClass::StrictHomeRegion,
            migration_plan_ref: "migration/ten-1-to-ten-1".to_string(),
            dsr_id: "dsr-residency-change-1".to_string(),
            deletion_certificate_ref: "certificate/deletion/ten_1".to_string(),
            requested_at_epoch_seconds: 200,
        })
        .expect_err("residency changes require a new tenant identity");
        assert_eq!(error, ResidencyError::ResidencyChangeRequiresNewTenant);
    }

    #[test]
    fn strict_home_region_allows_only_intra_kr_dr_or_backup() {
        let permit = CrossRegionTransferPermit::new(CrossRegionTransferPermitCreate {
            residency_class: ResidencyClass::StrictHomeRegion,
            destination_region: kr_secondary(),
            consent_receipt_ref: None,
            ..permit_create(ResidencyClass::StrictHomeRegion)
        })
        .expect("strict home-region allows intra-home-region DR transfer");
        assert_eq!(
            permit.destination_region.value.jurisdiction.value,
            RegionJurisdiction::Home
        );

        let error = CrossRegionTransferPermit::new(permit_create(ResidencyClass::StrictHomeRegion))
            .expect_err("strict home-region cannot write recovery replicas");
        assert_eq!(error, ResidencyError::DestinationRegionNotAllowed);
    }

    #[test]
    fn home_with_recovery_failover_requires_consent_and_denies_high_risk_classes() {
        let permit =
            CrossRegionTransferPermit::new(permit_create(ResidencyClass::HomeWithRecoveryFailover))
                .expect("home-with-recovery failover allows consented DR replica");
        assert_eq!(
            permit.consent_receipt_ref.value.as_deref(),
            Some("consent/receipt-1")
        );

        let missing_consent = CrossRegionTransferPermit::new(CrossRegionTransferPermitCreate {
            consent_receipt_ref: None,
            ..permit_create(ResidencyClass::HomeWithRecoveryFailover)
        })
        .expect_err("home to recovery failover needs per-class consent");
        assert_eq!(missing_consent, ResidencyError::MissingConsentReceipt);

        let denied_class = CrossRegionTransferPermit::new(CrossRegionTransferPermitCreate {
            data_class: privacy(DataClass::SensitivePipaArticle23),
            ..permit_create(ResidencyClass::HomeWithRecoveryFailover)
        })
        .expect_err("Sensitive PIPA Art 23 cannot enter recovery warm replicas");
        assert_eq!(denied_class, ResidencyError::DataClassDeniedForResidency);
    }

    #[test]
    fn per_pack_residency_enforces_allowed_regions() {
        let per_pack = PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-federated-1".to_string()],
            allowed_replica_regions: vec!["region-federated-2".to_string()],
            forbidden_regions: vec!["region-recovery-1".to_string()],
            regulator_overlay: regulator_overlay(),
        })
        .expect("per-pack residency fixture is valid");
        let permit = CrossRegionTransferPermit::new(CrossRegionTransferPermitCreate {
            residency_class: ResidencyClass::PerPack(Box::new(per_pack.clone())),
            source_region: region("region-federated-1", RegionJurisdiction::Federated),
            destination_region: region("region-federated-2", RegionJurisdiction::Federated),
            purpose: CrossRegionTransferPurpose::Backup,
            ..permit_create(ResidencyClass::PerPack(Box::new(per_pack.clone())))
        })
        .expect("per-pack residency allows declared replica region");
        assert_eq!(
            permit.destination_region.value.region_id.value,
            "region-federated-2"
        );

        let error = CrossRegionTransferPermit::new(CrossRegionTransferPermitCreate {
            residency_class: ResidencyClass::PerPack(Box::new(per_pack)),
            source_region: region("region-federated-1", RegionJurisdiction::Federated),
            destination_region: us_warm(),
            purpose: CrossRegionTransferPurpose::Backup,
            ..permit_create(ResidencyClass::Global)
        })
        .expect_err("per-pack residency rejects forbidden destination");
        assert_eq!(error, ResidencyError::DestinationRegionNotAllowed);
    }

    #[test]
    fn legacy_operational_data_class_is_rejected() {
        let error = residency_data_class_from_legacy(DataClass::Audit)
            .expect_err("operational labels cannot enter privacy residency scope");
        assert_eq!(error, ResidencyError::InvalidDataClass);
    }
}
