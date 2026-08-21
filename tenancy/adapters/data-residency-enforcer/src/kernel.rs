//! Closed residency vocabulary: the parsed forms of everything a
//! [`crate::ResidencyContext`] carries as an untrusted string, plus the two
//! ports the decision engine reads — the region catalog and the legal-transfer
//! register.
//!
//! Every `parse_label` here returns `None` for an unrecognised label. That is
//! the load-bearing property of this module: the caller hands the adapter free
//! strings, and a residency control that guesses at a string it does not
//! recognise is not a control. Unknown parses to `None`, and `None` becomes a
//! denial in [`crate::usecase`] — never an allow.
//!
//! The same reasoning is why a transfer basis is not believed on the caller's
//! word. A [`TransferBasis`] is an ASSERTION; it turns a denial into an allow
//! only when [`ResidencyTransferRegister`] resolves it to a register row whose
//! jurisdictions, tenant and purpose match the route being decided. The
//! adequacy decision and the Schrems-II supplementary measures are facts about
//! the receiving jurisdiction and the deployment, so they are read from the
//! register row, never from the request.

use crate::ResidencyAdapterError;

/// Tenant residency class, mirroring the vocabulary already bound to tenants by
/// `tenancy/core/kernel/src/lib.rs::ResidencyClass`.
///
/// The labels are byte-for-byte the ones that kernel emits so a tenant record
/// can be projected onto this adapter without a translation table. The
/// duplication is deliberate — see the "Gaps" paragraph in `lib.rs`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyClass {
    /// Data never leaves its home region. `allows_failover_region` is false in
    /// the tenancy kernel, so not even an in-pack DR region is reachable.
    StrictHomeRegion,
    /// Data never leaves its federated home region; also no failover region.
    StrictFederatedRegion,
    /// Home region plus a recovery region, reachable only for recovery work.
    HomeWithRecoveryFailover,
    /// No residency-class restriction of its own. Pack and data-class rules
    /// still apply — `compliance.md` §"higher-restriction-wins".
    Global,
}

impl ResidencyClass {
    /// The wire label for this class.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StrictHomeRegion => "strict_home_region",
            Self::StrictFederatedRegion => "strict_federated_region",
            Self::HomeWithRecoveryFailover => "home_with_recovery_failover",
            Self::Global => "global",
        }
    }

    /// Parse a wire label. Unknown labels yield `None` (fail closed).
    #[must_use]
    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "strict_home_region" => Some(Self::StrictHomeRegion),
            "strict_federated_region" => Some(Self::StrictFederatedRegion),
            "home_with_recovery_failover" => Some(Self::HomeWithRecoveryFailover),
            "global" => Some(Self::Global),
            _ => None,
        }
    }

    /// Whether this class forbids every cross-region route outright.
    ///
    /// Derived from `tenancy/policy/data-residency.cedar` rule 1 (a strict
    /// tenant may not be processed outside its home jurisdiction) together with
    /// `tenancy/core/kernel::ResidencyClass::allows_failover_region`, which is
    /// `false` for both strict classes.
    #[must_use]
    pub const fn forbids_all_cross_region(self) -> bool {
        matches!(self, Self::StrictHomeRegion | Self::StrictFederatedRegion)
    }
}

/// A data class as written down in the tenancy corpus.
///
/// Sources: `tenancy/ARCHITECTURE.md` (`INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`),
/// `tenancy/dpia.md` §2.3 (`SENSITIVE_PIPA_ART23`, `BEHAVIORAL_TENANT_PRODUCT`),
/// `tenancy/policy/data-residency.md` §"Retention by Jurisdiction × Data Class"
/// (`PII_IDENTIFYING`, `SECRET`), and
/// `registry/catalog/tenancy-data-residency-enforcer.yaml` (`PUBLIC`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyDataClass {
    /// Publishable; carries no tenant-identifying content.
    Public,
    /// Default class for tenant-operational metadata.
    InternalOnly,
    /// Lifecycle / RLS-install audit rows.
    Audit,
    /// Tenant lifecycle history, plan tier, jurisdiction, cell assignment.
    BehavioralTenantProduct,
    /// Quasi-identifying tenant attributes.
    PiiQuasi,
    /// Directly identifying personal data (e.g. operator OIDC subject).
    PiiIdentifying,
    /// KR PIPA Art. 23 sensitive personal information.
    SensitivePipaArt23,
    /// Key material and credentials; per-pack and per-environment by PRD.
    Secret,
}

impl ResidencyDataClass {
    /// The canonical wire label for this class.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::InternalOnly => "INTERNAL_ONLY",
            Self::Audit => "AUDIT",
            Self::BehavioralTenantProduct => "BEHAVIORAL_TENANT_PRODUCT",
            Self::PiiQuasi => "PII_QUASI",
            Self::PiiIdentifying => "PII_IDENTIFYING",
            Self::SensitivePipaArt23 => "SENSITIVE_PIPA_ART23",
            Self::Secret => "SECRET",
        }
    }

    /// Parse a wire label. Matching is CASE-SENSITIVE: the corpus writes these
    /// labels in screaming snake case and a differently-cased string is an
    /// unrecognised label, not a near-miss to be repaired. `PII_QUASI_IDENTIFIER`
    /// is accepted as an alias because
    /// `registry/catalog/tenancy-data-residency-enforcer.yaml` spells it that
    /// way while `tenancy/ARCHITECTURE.md` spells it `PII_QUASI`.
    #[must_use]
    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "PUBLIC" => Some(Self::Public),
            "INTERNAL_ONLY" => Some(Self::InternalOnly),
            "AUDIT" => Some(Self::Audit),
            "BEHAVIORAL_TENANT_PRODUCT" => Some(Self::BehavioralTenantProduct),
            "PII_QUASI" | "PII_QUASI_IDENTIFIER" => Some(Self::PiiQuasi),
            "PII_IDENTIFYING" => Some(Self::PiiIdentifying),
            "SENSITIVE_PIPA_ART23" => Some(Self::SensitivePipaArt23),
            "SECRET" => Some(Self::Secret),
            _ => None,
        }
    }
}

/// The outbound operation being guarded.
///
/// `assign_dr_cell` and `migrate_tenant_cross_jurisdiction` are the two Cedar
/// actions that name residency explicitly — in
/// `tenancy/policy/data-residency.cedar` and `tenancy/cedar/policies.cedar`
/// respectively. The rest are the dispatch shapes IP-020 §B names: outbound
/// events, RPC calls, storage replication, DR promotion, and DSR receipt
/// aggregation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyOperation {
    /// Read of tenant metadata from another region.
    ReadTenantMetadata,
    /// Outbound Workflow event dispatch.
    EmitEvent,
    /// Outbound RPC call.
    RpcCall,
    /// Postgres / Citus / audit-chain replication.
    ReplicateStorage,
    /// Bind a DR cell to a tenant (`Action::"assign_dr_cell"`).
    AssignDrCell,
    /// Promote a DR region to primary.
    PromoteDr,
    /// Aggregate DSR erasure receipts.
    AggregateDsrReceipt,
    /// Move a tenant across jurisdictions
    /// (`Action::"MigrateTenantCrossJurisdiction"`).
    MigrateTenantCrossJurisdiction,
}

impl ResidencyOperation {
    /// The wire label for this operation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReadTenantMetadata => "read_tenant_metadata",
            Self::EmitEvent => "emit_event",
            Self::RpcCall => "rpc_call",
            Self::ReplicateStorage => "replicate_storage",
            Self::AssignDrCell => "assign_dr_cell",
            Self::PromoteDr => "promote_dr",
            Self::AggregateDsrReceipt => "aggregate_dsr_receipt",
            Self::MigrateTenantCrossJurisdiction => "migrate_tenant_cross_jurisdiction",
        }
    }

    /// Parse a wire label. Unknown labels yield `None` (fail closed).
    #[must_use]
    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "read_tenant_metadata" => Some(Self::ReadTenantMetadata),
            "emit_event" => Some(Self::EmitEvent),
            "rpc_call" => Some(Self::RpcCall),
            "replicate_storage" => Some(Self::ReplicateStorage),
            "assign_dr_cell" => Some(Self::AssignDrCell),
            "promote_dr" => Some(Self::PromoteDr),
            "aggregate_dsr_receipt" => Some(Self::AggregateDsrReceipt),
            "migrate_tenant_cross_jurisdiction" => Some(Self::MigrateTenantCrossJurisdiction),
            _ => None,
        }
    }

    /// Whether this operation is business-continuity work, which is the only
    /// non-DSR traffic `tenancy/multi-region.md` §"DR Failover" lets a
    /// `home_with_recovery_failover` tenant put on its DR-pair region.
    #[must_use]
    pub const fn is_recovery_operation(self) -> bool {
        matches!(
            self,
            Self::ReplicateStorage | Self::AssignDrCell | Self::PromoteDr
        )
    }

    /// Whether this operation is the DSR receipt fan-in that
    /// `tenancy/policy/data-residency.md` §"DSR (Data Subject Request) Cascade"
    /// steps 4-5 describe and IP-020 §D.4 names as a permitted route.
    ///
    /// It is permitted INTRA-PACK only. The same document keeps audit-chain
    /// seals within-pack ("each pack has its own audit-chain instance") and
    /// forbids cross-pack replication by default, so a cross-pack receipt
    /// fan-in has no authorising sentence anywhere in the corpus and stays
    /// denied — see "Gaps" in `lib.rs`.
    #[must_use]
    pub const fn is_dsr_aggregation(self) -> bool {
        matches!(self, Self::AggregateDsrReceipt)
    }
}

/// A residency overlay from `tenancy/policy/data-residency.cedar`.
///
/// This is NOT the tenant's full compliance-pack list. It is exactly the closed
/// set of overlays that Cedar fragment pins a processing region against; a
/// non-residency pack such as `SOC2-T2` has no residency rule and does not
/// belong here. See the "Gaps" paragraph in `lib.rs` for the projection the
/// caller owns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyOverlay {
    /// KR-CSAP: the processing REGION must be literally `kr`, as
    /// `data-residency.cedar` rule 2 spells it (`processing_region != "kr"`).
    /// No region in the documented roster is named `kr`; see "Gaps" in
    /// `lib.rs`.
    KrCsap,
    /// EU sovereign: processing region must be an `eu-sovereign-*` region.
    EuSovereign,
    /// CN PIPL: processing region must be `cn-onshore`.
    CnPipl,
}

impl ResidencyOverlay {
    /// The wire label for this overlay, as the Cedar fragment spells it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::KrCsap => "kr-csap",
            Self::EuSovereign => "eu-sovereign",
            Self::CnPipl => "cn-pipl",
        }
    }

    /// Parse a wire label. Unknown labels yield `None` (fail closed).
    #[must_use]
    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "kr-csap" => Some(Self::KrCsap),
            "eu-sovereign" => Some(Self::EuSovereign),
            "cn-pipl" => Some(Self::CnPipl),
            _ => None,
        }
    }
}

/// The role a region plays inside its pack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RegionRole {
    /// The pack's primary region.
    Primary,
    /// The pack's warm-standby DR-pair region.
    DrPair,
}

/// One row of the region catalog: which pack and jurisdiction a region belongs
/// to, and whether it is that pack's primary or its DR pair.
///
/// The identity of a row is the PAIR `(region_id, pack_id)`, not the region id
/// alone. `tenancy/policy/data-residency.md` §"Default: pack-pinning at
/// creation time" puts `pack-us` and `pack-us-healthcare` on the same OCI
/// regions while requiring the healthcare cluster be isolated, and
/// §"Per-Pack Jurisdiction Tagging" gives them distinct `jurisdiction_code`
/// values (`US` vs `US-HC`). A catalog keyed by region id alone cannot express
/// that topology, which is why the caller declares which pack a route's
/// endpoints belong to — see [`crate::ResidencyContext::with_pack`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegionRecord {
    /// Region identifier, e.g. `ap-seoul-1`.
    pub region_id: String, // data_class: INTERNAL_ONLY
    /// Owning pack, e.g. `pack-kr`.
    pub pack_id: String, // data_class: INTERNAL_ONLY
    /// Jurisdiction code, e.g. `KR`. Values follow the `jurisdiction_code`
    /// enumeration in `tenancy/policy/data-residency.md`.
    pub jurisdiction: String, // data_class: INTERNAL_ONLY
    /// Primary or DR pair within `pack_id`.
    pub role: RegionRole, // data_class: INTERNAL_ONLY
}

impl RegionRecord {
    /// Build a record. No validation here; the engine validates the whole
    /// catalog once at construction via [`RegionRecord::validate`].
    #[must_use]
    pub fn new(
        region_id: impl Into<String>,
        pack_id: impl Into<String>,
        jurisdiction: impl Into<String>,
        role: RegionRole,
    ) -> Self {
        Self {
            region_id: region_id.into(),
            pack_id: pack_id.into(),
            jurisdiction: jurisdiction.into(),
            role,
        }
    }

    /// Reject a structurally unusable row.
    ///
    /// A row with a blank jurisdiction would make every comparison against it
    /// vacuously equal, which is how a residency control degrades into an
    /// allow-everything. That is a malformed policy, not a denial.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::PolicyMalformed`] when any field is blank.
    pub fn validate(&self) -> Result<(), ResidencyAdapterError> {
        if self.region_id.trim().is_empty()
            || self.pack_id.trim().is_empty()
            || self.jurisdiction.trim().is_empty()
        {
            return Err(ResidencyAdapterError::PolicyMalformed);
        }
        Ok(())
    }

    /// Whether this row is the cell of `pack_id` inside its region.
    #[must_use]
    pub fn is_in_pack(&self, pack_id: &str) -> bool {
        self.pack_id == pack_id
    }
}

/// The µservice name `tenancy/cedar/policies.cedar` requires of a
/// `MigrateTenantCrossJurisdiction` resource.
pub const TENANCY_MICROSERVICE: &str = "tenancy";

/// The legal basis a caller ASSERTS for a route that is otherwise forbidden.
///
/// Both variants come from `tenancy/policy/data-residency.md` §"Cross-Pack
/// Replication Policy": the SCC exception (GDPR Arts. 44-46) and the explicit
/// cross-jurisdiction Cedar permit that `tenancy/cedar/policies.cedar` demands
/// of `Action::"MigrateTenantCrossJurisdiction"`.
///
/// An assertion is not an authorisation. The predicates below check that the
/// caller filled the assertion in COMPLETELY; whether it is TRUE is decided by
/// [`ResidencyTransferRegister`], and both must hold before the cascade in
/// [`crate::domain`] will turn a denial into an allow.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransferBasis {
    /// Tenant-executed Standard Contractual Clauses.
    StandardContractualClauses {
        /// Row reference in `legal/transfer-register.md` (requirement 1).
        register_ref: String,
        /// The specifically-named processing purpose (requirement 3);
        /// ad-hoc transfer is not authorised, so a blank purpose is no basis.
        transfer_purpose: String,
        /// Receiving jurisdiction has an adequacy decision or equivalent
        /// safeguard (requirement 2). The caller's claim only; the register
        /// row is what the engine believes.
        adequacy_or_safeguard: bool,
        /// Schrems-II supplementary technical measures are in place
        /// (requirement 5). The caller's claim only; the register row is what
        /// the engine believes.
        supplementary_measures: bool,
    },
    /// An explicit cross-jurisdiction permit, carrying all four conjuncts
    /// `tenancy/cedar/policies.cedar` requires of
    /// `Action::"MigrateTenantCrossJurisdiction"`.
    CrossJurisdictionCedarPermit {
        /// `context.cedar_permit_id`; must be non-empty.
        permit_id: String,
        /// `context.cross_jurisdiction_permit_id`; must be non-empty and is a
        /// DIFFERENT id from `permit_id` in the Cedar fragment.
        cross_jurisdiction_permit_id: String,
        /// `context.audit_chain_emit`; must be `true`.
        audit_chain_emit: bool,
        /// `resource.microservice`; must be `tenancy`.
        microservice: String,
    },
}

impl TransferBasis {
    /// Whether this is a completely-filled SCC assertion: all four recorded
    /// requirements of `data-residency.md` §"Exception: tenant-executed SCCs"
    /// claimed.
    ///
    /// A half-filled SCC is not a transfer basis. It is a claim. A completely
    /// filled one is a complete claim — still not an authorisation until
    /// [`ResidencyTransferRegister::scc_entry`] confirms it.
    #[must_use]
    pub fn is_complete_scc(&self) -> bool {
        match self {
            Self::StandardContractualClauses {
                register_ref,
                transfer_purpose,
                adequacy_or_safeguard,
                supplementary_measures,
            } => {
                !register_ref.trim().is_empty()
                    && !transfer_purpose.trim().is_empty()
                    && *adequacy_or_safeguard
                    && *supplementary_measures
            }
            Self::CrossJurisdictionCedarPermit { .. } => false,
        }
    }

    /// Whether this permit satisfies every conjunct of the Cedar `unless`
    /// clause in `tenancy/cedar/policies.cedar` lines 40-49: the resource is
    /// the `tenancy` µservice, both permit ids are non-empty, and audit-chain
    /// emission is on. Dropping any one of them would forbid the migration
    /// under the fragment this mirrors.
    #[must_use]
    pub fn is_valid_cross_jurisdiction_permit(&self) -> bool {
        match self {
            Self::CrossJurisdictionCedarPermit {
                permit_id,
                cross_jurisdiction_permit_id,
                audit_chain_emit,
                microservice,
            } => {
                !permit_id.trim().is_empty()
                    && !cross_jurisdiction_permit_id.trim().is_empty()
                    && *audit_chain_emit
                    && microservice.trim() == TENANCY_MICROSERVICE
            }
            Self::StandardContractualClauses { .. } => false,
        }
    }

    /// The register row this basis points at, if it is an SCC.
    #[must_use]
    pub fn scc_register_ref(&self) -> Option<&str> {
        match self {
            Self::StandardContractualClauses { register_ref, .. } => Some(register_ref.as_str()),
            Self::CrossJurisdictionCedarPermit { .. } => None,
        }
    }

    /// The purpose this basis names, if it is an SCC.
    #[must_use]
    pub fn scc_transfer_purpose(&self) -> Option<&str> {
        match self {
            Self::StandardContractualClauses {
                transfer_purpose, ..
            } => Some(transfer_purpose.as_str()),
            Self::CrossJurisdictionCedarPermit { .. } => None,
        }
    }

    /// The permit id this basis points at, if it is a Cedar permit.
    #[must_use]
    pub fn cedar_permit_id(&self) -> Option<&str> {
        match self {
            Self::CrossJurisdictionCedarPermit { permit_id, .. } => Some(permit_id.as_str()),
            Self::StandardContractualClauses { .. } => None,
        }
    }
}

/// One row of `microservices/tenancy/legal/transfer-register.md`: an SCC that
/// somebody actually executed, for a named tenant, route and purpose.
///
/// The two booleans are deliberately HERE rather than on [`TransferBasis`]:
/// whether the receiving jurisdiction has an adequacy decision, and whether the
/// Schrems-II supplementary measures are deployed, are facts about the world
/// that the register records — not something the request may assert about
/// itself.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SccRegisterEntry {
    /// The row reference callers cite (requirement 1).
    pub register_ref: String, // data_class: INTERNAL_ONLY
    /// The tenant that executed the SCC.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Jurisdiction the data leaves.
    pub source_jurisdiction: String, // data_class: INTERNAL_ONLY
    /// Jurisdiction the data may reach (requirement 2's receiving pack).
    pub destination_jurisdiction: String, // data_class: INTERNAL_ONLY
    /// The specifically-named processing purpose (requirement 3).
    pub transfer_purpose: String, // data_class: INTERNAL_ONLY
    /// Adequacy decision or equivalent safeguard on file (requirement 2).
    pub adequacy_or_safeguard: bool, // data_class: INTERNAL_ONLY
    /// Schrems-II supplementary technical measures deployed (requirement 5).
    pub supplementary_measures: bool, // data_class: INTERNAL_ONLY
}

/// One issued cross-jurisdiction migration permit, as
/// `tenancy/cedar/policies.cedar` expects the control plane to hold.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CrossJurisdictionPermitEntry {
    /// The permit id callers cite.
    pub permit_id: String, // data_class: INTERNAL_ONLY
    /// The tenant the permit was issued for.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Jurisdiction the tenant migrates from.
    pub source_jurisdiction: String, // data_class: INTERNAL_ONLY
    /// Jurisdiction the tenant migrates to.
    pub destination_jurisdiction: String, // data_class: INTERNAL_ONLY
    /// Whether the issuing flow committed to audit-chain emission.
    pub audit_chain_emit: bool, // data_class: INTERNAL_ONLY
}

/// The legal-transfer register port: resolve an ASSERTED basis to a recorded
/// one.
///
/// Sync for the same reason the catalog is: the decision has to stay a
/// reproducible pure function of resolved facts.
pub trait ResidencyTransferRegister {
    /// The SCC row with this reference, if the register holds one.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the register cannot
    /// answer. An error is never an allow.
    fn scc_entry(
        &self,
        register_ref: &str,
    ) -> Result<Option<SccRegisterEntry>, ResidencyAdapterError>;

    /// The cross-jurisdiction permit with this id, if the register holds one.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the register cannot
    /// answer. An error is never an allow.
    fn cross_jurisdiction_permit(
        &self,
        permit_id: &str,
    ) -> Result<Option<CrossJurisdictionPermitEntry>, ResidencyAdapterError>;
}

/// The register an engine has when no register was wired in.
///
/// It holds nothing, so no asserted basis ever resolves and every route that
/// depends on one denies. That is the correct default for a control whose only
/// allow-producing input is the one the caller supplies: absent a register to
/// check it against, the assertion buys nothing.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NoTransferRegister;

impl ResidencyTransferRegister for NoTransferRegister {
    fn scc_entry(
        &self,
        _register_ref: &str,
    ) -> Result<Option<SccRegisterEntry>, ResidencyAdapterError> {
        Ok(None)
    }

    fn cross_jurisdiction_permit(
        &self,
        _permit_id: &str,
    ) -> Result<Option<CrossJurisdictionPermitEntry>, ResidencyAdapterError> {
        Ok(None)
    }
}

/// The region/jurisdiction catalog the engine reads.
///
/// Sync by construction: the decision must be reproducible from its inputs, and
/// an async catalog would put an I/O boundary inside a control that has to be
/// testable as a pure matrix. See "Gaps" in `lib.rs`.
pub trait ResidencyRegionCatalog {
    /// Look one region up by id, where the id identifies exactly one cell.
    ///
    /// `Ok(None)` means "this catalog has no such region", which the engine
    /// treats as a denial. `Err` means the catalog could not answer, which is
    /// never an allow. A catalog whose region hosts more than one pack must
    /// return `Ok(None)` here and report the rows from [`Self::rows_for`]
    /// instead, so the ambiguity surfaces as a denial rather than as an
    /// arbitrary pick.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the catalog is
    /// unreachable or otherwise unable to answer.
    fn lookup(&self, region_id: &str) -> Result<Option<RegionRecord>, ResidencyAdapterError>;

    /// EVERY cell in this region — one row per pack present there.
    ///
    /// The default implementation answers from [`Self::lookup`], which is
    /// correct for a catalog whose regions host a single pack. A catalog that
    /// models the documented `pack-us` / `pack-us-healthcare` co-tenancy on
    /// `us-ashburn-1` overrides it.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the catalog is
    /// unreachable.
    fn rows_for(&self, region_id: &str) -> Result<Vec<RegionRecord>, ResidencyAdapterError> {
        Ok(self.lookup(region_id)?.into_iter().collect())
    }

    /// Every row this catalog holds, so the engine can validate the whole
    /// policy surface once at construction instead of discovering a blank
    /// jurisdiction on the request that needed it.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the catalog is
    /// unreachable.
    fn regions(&self) -> Result<Vec<RegionRecord>, ResidencyAdapterError>;
}
