//! Foundry catalog kernel.
//!
//! Pure value objects for validating the `registry/catalog/<crate>.yaml`
//! records that drive Foundry engineering-platform gates.

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::{
    Classified, DataClass, OperationalDataClass, PrivacyDataClass,
    data_classes_from_privacy_data_classes, parse_data_class_label,
    parse_operational_data_class_label,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CatalogRole {
    Kernel,
    Domain,
    Usecase,
    App,
    Api,
    Worker,
    Adapter,
    Runtime,
    Cli,
    Test,
    Infrastructure,
    Bindings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CatalogPlane {
    Control,
    Data,
    Analytics,
    Audit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApiStability {
    Preview,
    Stable,
    Ga,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecurityReview {
    Unreviewed,
    SelfReviewed,
    Independent,
    ExternalAudit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SupplyChainAttestation {
    SourceOnly,
    LicenseChecked,
    Sbom,
    SignedProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogError {
    InvalidCrateId,
    EmptyContext,
    InvalidRole,
    EmptyCapability,
    InvalidPlane,
    InvalidDataClass,
    InvalidApiStability,
    InvalidSecurityReview,
    InvalidSupplyChain,
    MissingDataClasses,
    DuplicateCrateRecord,
    MissingCrateRecord { crate_id: String },
    PlaneChanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogRecordInput {
    pub crate_id: String,
    pub context: String,
    pub role: String,
    pub capability: String,
    pub plane: String,
    pub data_classes_owned: Vec<String>,
    pub operational_classes_owned: Vec<String>, // data_class: PUBLIC
    pub api_stability: String,
    pub security_review: String,
    pub supply_chain: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogRecord {
    pub crate_id: Classified<String>,
    pub context: Classified<String>,
    pub role: Classified<CatalogRole>,
    pub capability: Classified<String>,
    pub plane: Classified<CatalogPlane>,
    pub privacy_data_classes_owned: Classified<Vec<PrivacyDataClass>>, // data_class: PUBLIC
    pub operational_classes_owned: Classified<Vec<OperationalDataClass>>,
    pub api_stability: Classified<ApiStability>,
    pub security_review: Classified<SecurityReview>,
    pub supply_chain: Classified<SupplyChainAttestation>,
    pub data_class: Classified<DataClass>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogIndex {
    records: Classified<BTreeMap<String, CatalogRecord>>,
}

impl CatalogRecordInput {
    pub fn build(self) -> Result<CatalogRecord, CatalogError> {
        validate_crate_id(&self.crate_id)?;
        if self.context.trim().is_empty() {
            return Err(CatalogError::EmptyContext);
        }
        if self.capability.trim().is_empty() {
            return Err(CatalogError::EmptyCapability);
        }
        if self.data_classes_owned.is_empty() {
            return Err(CatalogError::MissingDataClasses);
        }
        let data_classes_owned = self
            .data_classes_owned
            .iter()
            .map(|data_class| parse_privacy_data_class(data_class))
            .collect::<Result<Vec<_>, _>>()?;
        let operational_classes_owned = self
            .operational_classes_owned
            .iter()
            .map(|operational_class| parse_operational_class(operational_class))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CatalogRecord {
            crate_id: Classified::new(self.crate_id, DataClass::Public),
            context: Classified::new(self.context, DataClass::Public),
            role: Classified::new(parse_role(&self.role)?, DataClass::Public),
            capability: Classified::new(self.capability, DataClass::Public),
            plane: Classified::new(parse_plane(&self.plane)?, DataClass::Public),
            privacy_data_classes_owned: Classified::new(data_classes_owned, DataClass::Public),
            operational_classes_owned: Classified::new(
                operational_classes_owned,
                DataClass::Public,
            ),
            api_stability: Classified::new(
                parse_api_stability(&self.api_stability)?,
                DataClass::Public,
            ),
            security_review: Classified::new(
                parse_security_review(&self.security_review)?,
                DataClass::Public,
            ),
            supply_chain: Classified::new(
                parse_supply_chain(&self.supply_chain)?,
                DataClass::Public,
            ),
            data_class: Classified::new(DataClass::Public, DataClass::InternalOnly),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        })
    }
}

impl CatalogRecord {
    pub fn privacy_data_classes_owned(&self) -> &Classified<Vec<PrivacyDataClass>> {
        &self.privacy_data_classes_owned
    }

    /// Legacy catalog-record projection for consumers that still compare the
    /// historical `data_classes_owned` labels. The stored contract is the
    /// typed [`PrivacyDataClass`] vector above; this method is a lossless
    /// projection from that typed state and cannot admit operational or
    /// subject markers.
    pub fn legacy_data_classes_owned(&self) -> Classified<Vec<DataClass>> {
        Classified::new(
            data_classes_from_privacy_data_classes(&self.privacy_data_classes_owned.value),
            self.privacy_data_classes_owned.data_class,
        )
    }

    #[deprecated(
        note = "use privacy_data_classes_owned for canonical typed access or legacy_data_classes_owned for the compatibility projection"
    )]
    pub fn data_classes_owned(&self) -> Classified<Vec<DataClass>> {
        self.legacy_data_classes_owned()
    }
}

impl CatalogIndex {
    pub fn from_records(records: Vec<CatalogRecord>) -> Result<Self, CatalogError> {
        let mut by_crate_id = BTreeMap::new();
        for record in records {
            if by_crate_id
                .insert(record.crate_id.value.clone(), record)
                .is_some()
            {
                return Err(CatalogError::DuplicateCrateRecord);
            }
        }
        Ok(Self {
            records: Classified::new(by_crate_id, DataClass::Public),
        })
    }

    pub fn lookup(&self, crate_id: &str) -> Option<&CatalogRecord> {
        self.records.value.get(crate_id)
    }

    pub fn len(&self) -> usize {
        self.records.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.value.is_empty()
    }

    pub fn records(&self) -> impl Iterator<Item = &CatalogRecord> {
        self.records.value.values()
    }

    pub fn validate_required_crates<I, S>(&self, crate_ids: I) -> Result<(), CatalogError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for crate_id in crate_ids {
            let s = crate_id.as_ref();
            if !self.records.value.contains_key(s) {
                return Err(CatalogError::MissingCrateRecord {
                    crate_id: s.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn validate_plane_stability<I, S>(
        &self,
        baseline: &CatalogIndex,
        reviewed_changes: I,
    ) -> Result<(), CatalogError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let reviewed_changes = reviewed_changes
            .into_iter()
            .map(|crate_id| crate_id.as_ref().to_string())
            .collect::<BTreeSet<_>>();
        for crate_id in &reviewed_changes {
            validate_crate_id(crate_id)?;
            if !self.records.value.contains_key(crate_id) {
                return Err(CatalogError::MissingCrateRecord {
                    crate_id: crate_id.clone(),
                });
            }
        }
        for (crate_id, baseline_record) in &baseline.records.value {
            let Some(current_record) = self.records.value.get(crate_id) else {
                continue;
            };
            if current_record.plane.value != baseline_record.plane.value
                && !reviewed_changes.contains(crate_id)
            {
                return Err(CatalogError::PlaneChanged);
            }
        }
        Ok(())
    }
}

fn validate_crate_id(crate_id: &str) -> Result<(), CatalogError> {
    // De-brand transition (supersedes ADR-0017 oya- prefix enforcement; naming
    // grammar de-brand mandates updating naming-enforcing gates in lockstep):
    // catalog crate-ids must match the live workspace crate name, which is now
    // de-branded (e.g. `marketplace-plugin-kernel`). Accept any valid cargo
    // crate name — non-empty, first char an ascii lowercase letter, remaining
    // chars ascii-lowercase / ascii-digit / '-'. This accepts both de-branded
    // ids and still-`oya-` ids that have not yet been disposed. No liveness
    // check here — that is a separate gate.
    let mut characters = crate_id.chars();
    let starts_with_lowercase = characters
        .next()
        .is_some_and(|character| character.is_ascii_lowercase());
    if starts_with_lowercase
        && crate_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        Ok(())
    } else {
        Err(CatalogError::InvalidCrateId)
    }
}

fn parse_role(role: &str) -> Result<CatalogRole, CatalogError> {
    match role {
        "kernel" => Ok(CatalogRole::Kernel),
        "domain" => Ok(CatalogRole::Domain),
        "usecase" => Ok(CatalogRole::Usecase),
        "app" | "application" => Ok(CatalogRole::App),
        "api" | "rest" | "grpc" => Ok(CatalogRole::Api),
        "worker" => Ok(CatalogRole::Worker),
        "adapter" => Ok(CatalogRole::Adapter),
        "runtime" => Ok(CatalogRole::Runtime),
        "cli" => Ok(CatalogRole::Cli),
        "test" => Ok(CatalogRole::Test),
        "infrastructure" => Ok(CatalogRole::Infrastructure),
        "bindings" => Ok(CatalogRole::Bindings),
        _ => Err(CatalogError::InvalidRole),
    }
}

fn parse_plane(plane: &str) -> Result<CatalogPlane, CatalogError> {
    match plane {
        "control" => Ok(CatalogPlane::Control),
        "data" => Ok(CatalogPlane::Data),
        "analytics" => Ok(CatalogPlane::Analytics),
        "audit" => Ok(CatalogPlane::Audit),
        _ => Err(CatalogError::InvalidPlane),
    }
}

fn parse_api_stability(api_stability: &str) -> Result<ApiStability, CatalogError> {
    match api_stability {
        "preview" => Ok(ApiStability::Preview),
        "stable" => Ok(ApiStability::Stable),
        "GA" | "ga" => Ok(ApiStability::Ga),
        _ => Err(CatalogError::InvalidApiStability),
    }
}

fn parse_security_review(security_review: &str) -> Result<SecurityReview, CatalogError> {
    match security_review {
        "unreviewed" => Ok(SecurityReview::Unreviewed),
        "self-reviewed" => Ok(SecurityReview::SelfReviewed),
        "independent" => Ok(SecurityReview::Independent),
        "external-audit" => Ok(SecurityReview::ExternalAudit),
        _ => Err(CatalogError::InvalidSecurityReview),
    }
}

fn parse_supply_chain(supply_chain: &str) -> Result<SupplyChainAttestation, CatalogError> {
    match supply_chain {
        "source-only" => Ok(SupplyChainAttestation::SourceOnly),
        "license-checked" => Ok(SupplyChainAttestation::LicenseChecked),
        "sbom" => Ok(SupplyChainAttestation::Sbom),
        "signed-provenance" => Ok(SupplyChainAttestation::SignedProvenance),
        _ => Err(CatalogError::InvalidSupplyChain),
    }
}

fn parse_privacy_data_class(data_class: &str) -> Result<PrivacyDataClass, CatalogError> {
    let data_class = parse_data_class_label(data_class).ok_or(CatalogError::InvalidDataClass)?;
    PrivacyDataClass::try_from(data_class).map_err(|_| CatalogError::InvalidDataClass)
}

fn parse_operational_class(operational_class: &str) -> Result<OperationalDataClass, CatalogError> {
    parse_operational_data_class_label(operational_class).ok_or(CatalogError::InvalidDataClass)
}
