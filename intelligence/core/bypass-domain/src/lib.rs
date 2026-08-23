//! Foundry foundation-bypass kernel.
//!
//! Pure value objects for tracked, expirable foundation-gate bypasses.

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass};
pub use intelligence_capability_domain::AutonomyTier;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BypassGate {
    Architecture,
    Contracts,
    License,
    Supply,
    Migration,
    Bench,
    SearchDub,
    AdsClass,
    AdsSourceSingleton,
    ClaimCeiling,
    PlaneClass,
    Catalog,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BypassError {
    InvalidBypassId,
    EmptyPrRef,
    InvalidCrateRef,
    InvalidGate,
    InvalidPrincipalId,
    EmptyRationale,
    InvalidRegressionWindow,
    InvalidRemediationTime,
    DuplicateBypass,
    ExpiredBypass,
    InvalidBreakGlassId,
    EmptyTenantId,
    InvalidCapabilityId,
    InvalidBreakGlassQuorum,
    InvalidBreakGlassTier,
    InsufficientBreakGlassApprovals,
    DuplicateBreakGlassApprover,
    BreakGlassSelfApproval,
    InvalidBreakGlassExpiry,
    InvalidBreakGlassRevocationTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundationBypassInput {
    pub id: String,
    pub pr_ref: String,
    pub crate_ref: String,
    pub gate_bypassed: String,
    pub bypassing_actor: String,
    pub rationale: String,
    pub regression_window_days: u32,
    pub created_at_epoch_days: u64,
    pub remediated_at_epoch_days: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundationBypass {
    pub id: Classified<String>,
    pub pr_ref: Classified<String>,
    pub crate_ref: Classified<String>,
    pub gate_bypassed: Classified<BypassGate>,
    pub bypassing_actor: Classified<String>,
    pub rationale: Classified<String>,
    pub regression_window_days: Classified<u32>,
    pub data_class: Classified<DataClass>,
    pub created_at_epoch_days: Classified<u64>,
    pub remediated_at_epoch_days: Classified<Option<u64>>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BreakGlassQuorum {
    TwoOfThree,
    ThreeOfFive,
}

impl BreakGlassQuorum {
    pub const fn required_approvals(self) -> usize {
        match self {
            Self::TwoOfThree => 2,
            Self::ThreeOfFive => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomyBreakGlassInput {
    pub id: String,                         // data_class: PUBLIC
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub capability_id: String,              // data_class: PUBLIC
    pub requested_tier: AutonomyTier,       // data_class: PUBLIC
    pub permitted_tier: AutonomyTier,       // data_class: PUBLIC
    pub requesting_actor: String,           // data_class: INTERNAL_ONLY
    pub approving_actors: Vec<String>,      // data_class: INTERNAL_ONLY
    pub approval_quorum: String,            // data_class: PUBLIC
    pub rationale: String,                  // data_class: PUBLIC
    pub created_at_epoch_days: u64,         // data_class: PUBLIC
    pub expires_at_epoch_days: u64,         // data_class: PUBLIC
    pub revoked_at_epoch_days: Option<u64>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomyBreakGlass {
    pub id: Classified<String>,                         // data_class: PUBLIC
    pub tenant_id: Classified<String>,                  // data_class: INTERNAL_ONLY
    pub capability_id: Classified<String>,              // data_class: PUBLIC
    pub requested_tier: Classified<AutonomyTier>,       // data_class: PUBLIC
    pub permitted_tier: Classified<AutonomyTier>,       // data_class: PUBLIC
    pub requesting_actor: Classified<String>,           // data_class: INTERNAL_ONLY
    pub approving_actors: Classified<Vec<String>>,      // data_class: INTERNAL_ONLY
    pub approval_quorum: Classified<BreakGlassQuorum>,  // data_class: PUBLIC
    pub rationale: Classified<String>,                  // data_class: PUBLIC
    pub data_class: Classified<DataClass>,              // data_class: INTERNAL_ONLY
    pub created_at_epoch_days: Classified<u64>,         // data_class: PUBLIC
    pub expires_at_epoch_days: Classified<u64>,         // data_class: PUBLIC
    pub revoked_at_epoch_days: Classified<Option<u64>>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BypassLedgerRecord {
    FoundationBypass(FoundationBypass),
    AutonomyBreakGlass(AutonomyBreakGlass),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BypassLedger {
    records: Classified<BTreeMap<String, BypassLedgerRecord>>,
}

impl Default for BypassLedger {
    fn default() -> Self {
        Self {
            records: Classified::new(BTreeMap::new(), DataClass::Public),
        }
    }
}

impl FoundationBypassInput {
    pub fn build(self) -> Result<FoundationBypass, BypassError> {
        validate_bypass_id(&self.id)?;
        if self.pr_ref.trim().is_empty() {
            return Err(BypassError::EmptyPrRef);
        }
        validate_crate_ref(&self.crate_ref)?;
        validate_principal_id(&self.bypassing_actor)?;
        if self.rationale.trim().is_empty() {
            return Err(BypassError::EmptyRationale);
        }
        if self.regression_window_days == 0 {
            return Err(BypassError::InvalidRegressionWindow);
        }
        if self
            .remediated_at_epoch_days
            .is_some_and(|remediated_at| remediated_at < self.created_at_epoch_days)
        {
            return Err(BypassError::InvalidRemediationTime);
        }
        Ok(FoundationBypass {
            id: Classified::new(self.id, DataClass::Public),
            pr_ref: Classified::new(self.pr_ref, DataClass::Public),
            crate_ref: Classified::new(self.crate_ref, DataClass::Public),
            gate_bypassed: Classified::new(parse_gate(&self.gate_bypassed)?, DataClass::Public),
            bypassing_actor: Classified::new(self.bypassing_actor, DataClass::InternalOnly),
            rationale: Classified::new(self.rationale, DataClass::Public),
            regression_window_days: Classified::new(self.regression_window_days, DataClass::Public),
            data_class: Classified::new(DataClass::Public, DataClass::InternalOnly),
            created_at_epoch_days: Classified::new(self.created_at_epoch_days, DataClass::Public),
            remediated_at_epoch_days: Classified::new(
                self.remediated_at_epoch_days,
                DataClass::Public,
            ),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        })
    }
}

impl AutonomyBreakGlassInput {
    pub fn build(self) -> Result<AutonomyBreakGlass, BypassError> {
        validate_break_glass_id(&self.id)?;
        validate_non_empty(&self.tenant_id, BypassError::EmptyTenantId)?;
        validate_capability_id(&self.capability_id)?;
        validate_principal_id(&self.requesting_actor)?;
        if self.rationale.trim().is_empty() {
            return Err(BypassError::EmptyRationale);
        }
        if self.expires_at_epoch_days <= self.created_at_epoch_days {
            return Err(BypassError::InvalidBreakGlassExpiry);
        }
        if self.permitted_tier > self.requested_tier {
            return Err(BypassError::InvalidBreakGlassTier);
        }
        if self
            .revoked_at_epoch_days
            .is_some_and(|revoked_at| revoked_at < self.created_at_epoch_days)
        {
            return Err(BypassError::InvalidBreakGlassRevocationTime);
        }
        let approval_quorum = parse_break_glass_quorum(&self.approval_quorum)?;
        validate_approvers(
            &self.requesting_actor,
            &self.approving_actors,
            approval_quorum,
        )?;

        Ok(AutonomyBreakGlass {
            id: Classified::new(self.id, DataClass::Public),
            tenant_id: Classified::new(self.tenant_id, DataClass::InternalOnly),
            capability_id: Classified::new(self.capability_id, DataClass::Public),
            requested_tier: Classified::new(self.requested_tier, DataClass::Public),
            permitted_tier: Classified::new(self.permitted_tier, DataClass::Public),
            requesting_actor: Classified::new(self.requesting_actor, DataClass::InternalOnly),
            approving_actors: Classified::new(self.approving_actors, DataClass::InternalOnly),
            approval_quorum: Classified::new(approval_quorum, DataClass::Public),
            rationale: Classified::new(self.rationale, DataClass::Public),
            data_class: Classified::new(DataClass::Public, DataClass::InternalOnly),
            created_at_epoch_days: Classified::new(self.created_at_epoch_days, DataClass::Public),
            expires_at_epoch_days: Classified::new(self.expires_at_epoch_days, DataClass::Public),
            revoked_at_epoch_days: Classified::new(self.revoked_at_epoch_days, DataClass::Public),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        })
    }
}

impl From<FoundationBypass> for BypassLedgerRecord {
    fn from(record: FoundationBypass) -> Self {
        Self::FoundationBypass(record)
    }
}

impl From<AutonomyBreakGlass> for BypassLedgerRecord {
    fn from(record: AutonomyBreakGlass) -> Self {
        Self::AutonomyBreakGlass(record)
    }
}

impl BypassLedger {
    pub fn from_records(records: Vec<FoundationBypass>) -> Result<Self, BypassError> {
        Self::from_ledger_records(records.into_iter().map(BypassLedgerRecord::from).collect())
    }

    pub fn from_ledger_records(records: Vec<BypassLedgerRecord>) -> Result<Self, BypassError> {
        let mut by_id = BTreeMap::new();
        for record in records {
            if by_id.insert(record.id().to_string(), record).is_some() {
                return Err(BypassError::DuplicateBypass);
            }
        }
        Ok(Self {
            records: Classified::new(by_id, DataClass::Public),
        })
    }

    pub fn insert_record(&mut self, record: BypassLedgerRecord) -> Result<(), BypassError> {
        if self.records.value.contains_key(record.id()) {
            return Err(BypassError::DuplicateBypass);
        }
        self.records.value.insert(record.id().to_string(), record);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.records.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.value.is_empty()
    }

    pub fn open_count(&self) -> usize {
        self.records
            .value
            .values()
            .filter(|record| record.is_open())
            .count()
    }

    pub fn validate_windows(&self, now_epoch_days: u64) -> Result<(), BypassError> {
        for record in self.records.value.values() {
            record.validate_window(now_epoch_days)?;
        }
        Ok(())
    }

    pub fn active_autonomy_break_glass_for(
        &self,
        tenant_id: &str,
        capability_id: &str,
        required_tier: AutonomyTier,
        now_epoch_days: u64,
    ) -> Option<&AutonomyBreakGlass> {
        self.records.value.values().find_map(|record| match record {
            BypassLedgerRecord::AutonomyBreakGlass(break_glass)
                if break_glass.tenant_id.value == tenant_id
                    && break_glass.capability_id.value == capability_id
                    && break_glass.revoked_at_epoch_days.value.is_none()
                    && break_glass.created_at_epoch_days.value <= now_epoch_days
                    && now_epoch_days <= break_glass.expires_at_epoch_days.value
                    && break_glass.permitted_tier.value >= required_tier =>
            {
                Some(break_glass)
            }
            _ => None,
        })
    }
}

impl BypassLedgerRecord {
    fn id(&self) -> &str {
        match self {
            Self::FoundationBypass(record) => &record.id.value,
            Self::AutonomyBreakGlass(record) => &record.id.value,
        }
    }

    fn is_open(&self) -> bool {
        match self {
            Self::FoundationBypass(record) => record.remediated_at_epoch_days.value.is_none(),
            Self::AutonomyBreakGlass(record) => record.revoked_at_epoch_days.value.is_none(),
        }
    }

    fn validate_window(&self, now_epoch_days: u64) -> Result<(), BypassError> {
        match self {
            Self::FoundationBypass(record) => {
                validate_foundation_bypass_window(record, now_epoch_days)
            }
            Self::AutonomyBreakGlass(record) => validate_break_glass_window(record, now_epoch_days),
        }
    }
}

fn validate_foundation_bypass_window(
    record: &FoundationBypass,
    now_epoch_days: u64,
) -> Result<(), BypassError> {
    let expires_at = record
        .created_at_epoch_days
        .value
        .checked_add(u64::from(record.regression_window_days.value))
        .ok_or(BypassError::InvalidRegressionWindow)?;
    match record.remediated_at_epoch_days.value {
        Some(remediated_at) if remediated_at > expires_at => Err(BypassError::ExpiredBypass),
        None if now_epoch_days > expires_at => Err(BypassError::ExpiredBypass),
        _ => Ok(()),
    }
}

fn validate_break_glass_window(
    record: &AutonomyBreakGlass,
    now_epoch_days: u64,
) -> Result<(), BypassError> {
    match record.revoked_at_epoch_days.value {
        Some(revoked_at) if revoked_at > record.expires_at_epoch_days.value => {
            Err(BypassError::ExpiredBypass)
        }
        None if now_epoch_days > record.expires_at_epoch_days.value => {
            Err(BypassError::ExpiredBypass)
        }
        _ => Ok(()),
    }
}

fn validate_bypass_id(id: &str) -> Result<(), BypassError> {
    if id.starts_with("byp_")
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        Ok(())
    } else {
        Err(BypassError::InvalidBypassId)
    }
}

fn validate_crate_ref(crate_ref: &str) -> Result<(), BypassError> {
    let mut characters = crate_ref.chars();
    let starts_with_lowercase = characters
        .next()
        .is_some_and(|character| character.is_ascii_lowercase());
    if starts_with_lowercase
        && crate_ref.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        Ok(())
    } else {
        Err(BypassError::InvalidCrateRef)
    }
}

fn validate_principal_id(principal_id: &str) -> Result<(), BypassError> {
    if principal_id.starts_with("usr_") || principal_id.starts_with("svc_") {
        Ok(())
    } else {
        Err(BypassError::InvalidPrincipalId)
    }
}

fn validate_break_glass_id(id: &str) -> Result<(), BypassError> {
    if id.starts_with("abg_")
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        Ok(())
    } else {
        Err(BypassError::InvalidBreakGlassId)
    }
}

fn validate_capability_id(capability_id: &str) -> Result<(), BypassError> {
    if capability_id.starts_with("cap.") && !capability_id.trim().is_empty() {
        Ok(())
    } else {
        Err(BypassError::InvalidCapabilityId)
    }
}

fn validate_non_empty(value: &str, error: BypassError) -> Result<(), BypassError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_approvers(
    requesting_actor: &str,
    approving_actors: &[String],
    approval_quorum: BreakGlassQuorum,
) -> Result<(), BypassError> {
    let mut unique_approvers = BTreeSet::new();
    for actor in approving_actors {
        validate_principal_id(actor)?;
        if actor == requesting_actor {
            return Err(BypassError::BreakGlassSelfApproval);
        }
        if !unique_approvers.insert(actor) {
            return Err(BypassError::DuplicateBreakGlassApprover);
        }
    }
    if unique_approvers.len() < approval_quorum.required_approvals() {
        return Err(BypassError::InsufficientBreakGlassApprovals);
    }
    Ok(())
}

fn parse_break_glass_quorum(quorum: &str) -> Result<BreakGlassQuorum, BypassError> {
    match quorum {
        "two-of-three" => Ok(BreakGlassQuorum::TwoOfThree),
        "three-of-five" => Ok(BreakGlassQuorum::ThreeOfFive),
        _ => Err(BypassError::InvalidBreakGlassQuorum),
    }
}

fn parse_gate(gate: &str) -> Result<BypassGate, BypassError> {
    match gate {
        "architecture" => Ok(BypassGate::Architecture),
        "contracts" => Ok(BypassGate::Contracts),
        "license" => Ok(BypassGate::License),
        "supply" => Ok(BypassGate::Supply),
        "migration" => Ok(BypassGate::Migration),
        "bench" => Ok(BypassGate::Bench),
        "search-dub" => Ok(BypassGate::SearchDub),
        "ads-class" => Ok(BypassGate::AdsClass),
        "ads-source-singleton" => Ok(BypassGate::AdsSourceSingleton),
        "claim-ceiling" => Ok(BypassGate::ClaimCeiling),
        "plane-class" => Ok(BypassGate::PlaneClass),
        "catalog" => Ok(BypassGate::Catalog),
        _ => Err(BypassError::InvalidGate),
    }
}
