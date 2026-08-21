//! The residency decision cascade: a pure function of already-parsed inputs.
//!
//! Every arm cites the artifact it mirrors. This engine is NOT the authority —
//! `tenancy/policy/data-residency.cedar` plus its prose companion
//! `tenancy/policy/data-residency.md` are. This is a hand-written mirror of
//! them, written because Cedar is a third-party dependency this crate may not
//! take (see "Gaps" in `lib.rs`).
//!
//! Ordering is part of the contract. Cedar `forbid` beats `permit`
//! unconditionally, so the overlay forbids are evaluated BEFORE the same-region
//! allow — an `eu-sovereign` tenant is forbidden by region shape, not by
//! whether the route happens to be a cross-region one. Within the cross-region
//! branch the data-class rules run before the residency-class rules so the
//! audit record names the most specific reason the route was refused.
//!
//! Inside the cross-jurisdiction branch the requirements are CONJUNCTIVE, not
//! a chain of short-circuiting alternatives. An EU-sourced tenant migration has
//! to satisfy the GDPR transfer rule AND the Cedar migration rule; satisfying
//! one does not excuse the other. Reading them as alternatives is how
//! EU-resident data leaves the EU on a bare permit id.

use crate::ResidencyDecision;
use crate::kernel::{
    RegionRecord, RegionRole, ResidencyClass, ResidencyDataClass, ResidencyOperation,
    ResidencyOverlay, TransferBasis,
};

/// The named rule that produced a decision.
///
/// Carried alongside the decision so a denial audit record can say WHICH rule
/// refused, which the four-variant [`ResidencyDecision`] cannot express on its
/// own. A denial whose reason is not recorded is an incident with no evidence.
/// [`crate::ResidencyDenialAuditSink::emit_denial_detailed`] is the path that
/// actually carries it onto the record.
///
/// Allow-producing rules are DISTINCT variants from the deny-producing rule
/// that guards the same route, so grouping audit rows by [`Self::code`]
/// separates authorised transfers from refused ones.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyRule {
    /// `data_class` was not a recognised label.
    UnknownDataClass,
    /// `residency_class` was not a recognised label.
    UnknownResidencyClass,
    /// `operation` was not a recognised label.
    UnknownOperation,
    /// A residency overlay label was not recognised.
    UnknownResidencyOverlay,
    /// The source or destination region is not in the catalog.
    UnknownRegion,
    /// The region hosts more than one pack and the caller did not say which
    /// cell the route touches.
    AmbiguousRegionRequiresPack,
    /// KR-CSAP tenant, processing region other than `kr`.
    OverlayKrCsapOffshore,
    /// EU-sovereign tenant, destination outside an `eu-sovereign-*` region.
    OverlayEuSovereignNonSovereignRegion,
    /// CN-PIPL tenant, destination other than `cn-onshore`.
    OverlayCnPiplOffshore,
    /// Source and destination are the same cell: no border is crossed.
    SameRegion,
    /// KR PIPA Art. 23 sensitive data leaving its jurisdiction.
    SensitiveDataCrossJurisdiction,
    /// Key material leaving its pack.
    SecretCrossPack,
    /// DSR receipt aggregation reaching outside the tenant's own pack.
    DsrAggregationRequiresIntraPack,
    /// A strict residency class forbids every cross-region route.
    StrictResidencyForbidsCrossRegion,
    /// `home_with_recovery_failover` reached something other than its own
    /// pack's DR pair, or used it for traffic that is neither recovery work
    /// nor DSR receipt aggregation.
    RecoveryFailoverRequiresIntraPackDrPair,
    /// Cross-jurisdiction migration without a registered Cedar permit.
    CrossJurisdictionMigrationRequiresPermit,
    /// Cross-jurisdiction migration WITH one: the allow.
    CrossJurisdictionMigrationAuthorised,
    /// EU-sourced cross-jurisdiction transfer without a registered SCC basis.
    EuTransferRequiresScc,
    /// EU-sourced cross-jurisdiction transfer WITH one: the allow.
    EuTransferAuthorisedByScc,
    /// Healthcare-pack data leaving its jurisdiction at all.
    HealthcarePackCrossJurisdictionUnauthorised,
    /// Cross-jurisdiction with no recorded basis: the default posture.
    CrossJurisdictionForbiddenByDefault,
    /// Same pack, same jurisdiction, DR-pair endpoint, recovery work: the
    /// in-pack DR path.
    IntraPackDrPairTransfer,
    /// Same pack, same jurisdiction: ordinary in-pack traffic that is not DR
    /// work and must not be counted as if it were.
    IntraPackTransfer,
    /// Same pack, same jurisdiction, DSR receipt fan-in.
    IntraPackDsrReceiptAggregation,
    /// Two packs inside one jurisdiction are still isolated from each other.
    CrossPackWithinJurisdictionForbidden,
    /// The evaluator reported a decision without a rule.
    ///
    /// Only reachable through the default
    /// [`crate::ResidencyPolicyEvaluator::evaluate_outcome`], i.e. an evaluator
    /// that implements the decision-only method and nothing else. It is a
    /// visible hole in the evidence, not a rule.
    RuleNotReported,
}

impl ResidencyRule {
    /// A stable slug for logs, metrics, and audit rows.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnknownDataClass => "unknown-data-class",
            Self::UnknownResidencyClass => "unknown-residency-class",
            Self::UnknownOperation => "unknown-operation",
            Self::UnknownResidencyOverlay => "unknown-residency-overlay",
            Self::UnknownRegion => "unknown-region",
            Self::AmbiguousRegionRequiresPack => "ambiguous-region-requires-pack",
            Self::OverlayKrCsapOffshore => "overlay-kr-csap-offshore",
            Self::OverlayEuSovereignNonSovereignRegion => "overlay-eu-sovereign-non-sovereign",
            Self::OverlayCnPiplOffshore => "overlay-cn-pipl-offshore",
            Self::SameRegion => "same-region",
            Self::SensitiveDataCrossJurisdiction => "sensitive-data-cross-jurisdiction",
            Self::SecretCrossPack => "secret-cross-pack",
            Self::DsrAggregationRequiresIntraPack => "dsr-aggregation-requires-intra-pack",
            Self::StrictResidencyForbidsCrossRegion => "strict-residency-forbids-cross-region",
            Self::RecoveryFailoverRequiresIntraPackDrPair => "recovery-failover-intra-pack-dr-only",
            Self::CrossJurisdictionMigrationRequiresPermit => "migration-requires-cedar-permit",
            Self::CrossJurisdictionMigrationAuthorised => "migration-authorised-by-cedar-permit",
            Self::EuTransferRequiresScc => "eu-transfer-requires-scc",
            Self::EuTransferAuthorisedByScc => "eu-transfer-authorised-by-scc",
            Self::HealthcarePackCrossJurisdictionUnauthorised => "healthcare-cross-jurisdiction",
            Self::CrossJurisdictionForbiddenByDefault => "cross-jurisdiction-default-forbid",
            Self::IntraPackDrPairTransfer => "intra-pack-dr-pair-transfer",
            Self::IntraPackTransfer => "intra-pack-transfer",
            Self::IntraPackDsrReceiptAggregation => "intra-pack-dsr-receipt-aggregation",
            Self::CrossPackWithinJurisdictionForbidden => "cross-pack-within-jurisdiction",
            Self::RuleNotReported => "rule-not-reported",
        }
    }

    /// The repository artifact this rule mirrors, so a reviewer can check the
    /// mirror against the authority by hand.
    #[must_use]
    pub const fn citation(self) -> &'static str {
        match self {
            Self::UnknownDataClass
            | Self::UnknownResidencyClass
            | Self::UnknownOperation
            | Self::UnknownResidencyOverlay
            | Self::UnknownRegion
            | Self::AmbiguousRegionRequiresPack => {
                "fail-closed default; no artifact authorises an unknown or ambiguous input"
            }
            Self::OverlayKrCsapOffshore
            | Self::OverlayEuSovereignNonSovereignRegion
            | Self::OverlayCnPiplOffshore => {
                "tenancy/policy/data-residency.cedar (compliance-pack forbids)"
            }
            Self::SameRegion => "tenancy/policy/data-residency.md §Default: pack-pinning",
            Self::SensitiveDataCrossJurisdiction => {
                "tenancy/policy/data-residency.md §pack-kr (PIPA Art. 23-2)"
            }
            Self::SecretCrossPack => "tenancy/PRD.md §Security posture (per-pack signing keys)",
            Self::DsrAggregationRequiresIntraPack => {
                "tenancy/policy/data-residency.md §DSR Cascade + §Cross-Pack Replication Policy"
            }
            Self::StrictResidencyForbidsCrossRegion => {
                "tenancy/policy/data-residency.cedar rule 1 + tenancy/core/kernel ResidencyClass"
            }
            Self::RecoveryFailoverRequiresIntraPackDrPair => {
                "tenancy/multi-region.md §Replication topology (intra-pack only)"
            }
            Self::CrossJurisdictionMigrationRequiresPermit
            | Self::CrossJurisdictionMigrationAuthorised => {
                "tenancy/cedar/policies.cedar (MigrateTenantCrossJurisdiction)"
            }
            Self::EuTransferRequiresScc | Self::EuTransferAuthorisedByScc => {
                "tenancy/policy/data-residency.md §Exception: tenant-executed SCCs"
            }
            Self::HealthcarePackCrossJurisdictionUnauthorised => {
                "tenancy/policy/data-residency.md §Exception: HIPAA BAA + DR failover"
            }
            Self::CrossJurisdictionForbiddenByDefault => {
                "tenancy/policy/data-residency.md §Cross-Pack Replication Policy"
            }
            Self::IntraPackDrPairTransfer => "tenancy/multi-region.md §DR Failover + §Failback",
            Self::IntraPackTransfer => {
                "tenancy/multi-region.md §Replication (intra-pack only, no DR claim)"
            }
            Self::IntraPackDsrReceiptAggregation => {
                "tenancy/policy/data-residency.md §DSR Cascade steps 4-5"
            }
            Self::CrossPackWithinJurisdictionForbidden => {
                "tenancy/policy/data-residency.md §pack-us-healthcare (isolated from pack-us)"
            }
            Self::RuleNotReported => "no rule; the evaluator reported a decision without one",
        }
    }
}

/// A decision plus the rule that produced it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResidencyOutcome {
    /// What the engine decided.
    pub decision: ResidencyDecision, // data_class: INTERNAL_ONLY
    /// Which rule decided it.
    pub rule: ResidencyRule, // data_class: INTERNAL_ONLY
}

impl ResidencyOutcome {
    /// Whether the route may be dispatched.
    #[must_use]
    pub fn is_allow(&self) -> bool {
        self.decision == ResidencyDecision::Allow
    }
}

/// The parsed request the cascade decides on. Constructed by
/// [`crate::usecase::ResidencyPolicyEngine`] after every string in the
/// [`crate::ResidencyContext`] has survived its `parse_label`, both regions
/// have resolved to a single cell, and every asserted transfer basis has been
/// looked up in the register.
#[derive(Clone, Copy, Debug)]
pub struct EvaluationInputs<'a> {
    /// The tenant's residency class.
    pub residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    /// The operation being guarded.
    pub operation: ResidencyOperation, // data_class: INTERNAL_ONLY
    /// The class of the data being moved.
    pub data_class: ResidencyDataClass, // data_class: TENANT_SCOPED
    /// The tenant's residency overlays.
    pub overlays: &'a [ResidencyOverlay], // data_class: TENANT_SCOPED
    /// Catalog row for the source cell.
    pub source: &'a RegionRecord, // data_class: INTERNAL_ONLY
    /// Catalog row for the destination cell.
    pub destination: &'a RegionRecord, // data_class: INTERNAL_ONLY
    /// The primary legal basis the caller asserts, if any.
    pub transfer_basis: Option<&'a TransferBasis>, // data_class: TENANT_SCOPED
    /// Further asserted bases. A route can need two at once — an EU-sourced
    /// tenant migration owes both an SCC and a migration permit — and one
    /// `Option` cannot carry both.
    pub additional_bases: &'a [TransferBasis], // data_class: TENANT_SCOPED
    /// Whether the legal-transfer register CONFIRMED an SCC covering exactly
    /// this tenant, route and purpose. Resolved by
    /// [`crate::usecase::ResidencyPolicyEngine`]; `false` when no register is
    /// wired, which is why an unregistered engine authorises nothing.
    pub scc_register_verified: bool, // data_class: INTERNAL_ONLY
    /// Whether the register CONFIRMED a cross-jurisdiction migration permit for
    /// exactly this tenant and route.
    pub permit_register_verified: bool, // data_class: INTERNAL_ONLY
}

impl EvaluationInputs<'_> {
    /// Every basis the caller asserted, primary first.
    fn asserted_bases(&self) -> impl Iterator<Item = &TransferBasis> {
        self.transfer_basis
            .into_iter()
            .chain(self.additional_bases.iter())
    }

    /// A complete SCC assertion that the register also confirmed. Both halves
    /// are required: a filled-in form nobody registered is a claim, and a
    /// register hit the caller never cited is not this request's basis.
    fn scc_basis_established(&self) -> bool {
        self.scc_register_verified && self.asserted_bases().any(TransferBasis::is_complete_scc)
    }

    /// A four-conjunct Cedar permit assertion that the register also confirmed.
    fn migration_permit_established(&self) -> bool {
        self.permit_register_verified
            && self
                .asserted_bases()
                .any(TransferBasis::is_valid_cross_jurisdiction_permit)
    }
}

/// The `eu-sovereign` region prefix the Cedar fragment tests for.
///
/// No region in the roster of `tenancy/multi-region.md` carries this prefix.
/// That contradiction is real and is left as a denial rather than papered over;
/// see "Gaps" in `lib.rs`.
pub const EU_SOVEREIGN_REGION_PREFIX: &str = "eu-sovereign-";

/// The `cn-pipl` onshore region the Cedar fragment names.
pub const CN_ONSHORE_REGION: &str = "cn-onshore";

/// The `kr-csap` processing region the Cedar fragment names.
///
/// `data-residency.cedar` rule 2 compares `resource.processing_region` against
/// the literal `"kr"`, exactly as rule 4 compares against `"cn-onshore"`. No
/// region in the documented roster is named `kr`, so this is the same
/// artifact-level contradiction as the `eu-sovereign` prefix and is resolved
/// the same way: literally, as a denial. See "Gaps" in `lib.rs`.
pub const KR_ONSHORE_REGION: &str = "kr";

/// The jurisdiction code of the healthcare pack, per
/// `tenancy/policy/data-residency.md` §"Per-Pack Jurisdiction Tagging".
pub const HEALTHCARE_JURISDICTION: &str = "US-HC";

/// The jurisdiction code of the KR pack.
pub const KR_JURISDICTION: &str = "KR";

/// The jurisdiction code of the EU pack.
pub const EU_JURISDICTION: &str = "EU";

fn overlay_denial(
    overlays: &[ResidencyOverlay],
    destination: &RegionRecord,
) -> Option<ResidencyRule> {
    for overlay in overlays {
        // Each arm compares the PROCESSING REGION against the literal the Cedar
        // fragment names, because that is what the fragment compares. Reading
        // one of them as a jurisdiction code instead is the permissive
        // invention `lib.rs` refuses to make.
        let violated = match overlay {
            ResidencyOverlay::KrCsap => {
                if destination.region_id == KR_ONSHORE_REGION {
                    None
                } else {
                    Some(ResidencyRule::OverlayKrCsapOffshore)
                }
            }
            ResidencyOverlay::EuSovereign => {
                if destination
                    .region_id
                    .starts_with(EU_SOVEREIGN_REGION_PREFIX)
                {
                    None
                } else {
                    Some(ResidencyRule::OverlayEuSovereignNonSovereignRegion)
                }
            }
            ResidencyOverlay::CnPipl => {
                if destination.region_id == CN_ONSHORE_REGION {
                    None
                } else {
                    Some(ResidencyRule::OverlayCnPiplOffshore)
                }
            }
        };
        if let Some(rule) = violated {
            return Some(rule);
        }
    }
    None
}

const fn deny(decision: ResidencyDecision, rule: ResidencyRule) -> ResidencyOutcome {
    ResidencyOutcome { decision, rule }
}

const fn allow(rule: ResidencyRule) -> ResidencyOutcome {
    ResidencyOutcome {
        decision: ResidencyDecision::Allow,
        rule,
    }
}

/// Decide one route.
///
/// Pure: no clock, no randomness, no I/O. The same inputs always produce the
/// same outcome, which is what makes the matrix in `tests/` a proof rather than
/// a sample.
#[must_use]
pub fn decide(inputs: &EvaluationInputs<'_>) -> ResidencyOutcome {
    // Cedar `forbid` is unconditional and beats every `permit`, so the overlay
    // forbids run first — before, and independently of, whether this route even
    // crosses a border.
    if let Some(rule) = overlay_denial(inputs.overlays, inputs.destination) {
        return deny(ResidencyDecision::DenyJurisdictionPack, rule);
    }

    let cross_jurisdiction = inputs.source.jurisdiction != inputs.destination.jurisdiction;
    let cross_pack = inputs.source.pack_id != inputs.destination.pack_id;

    // Same CELL — same region and same pack — crosses no boundary. Region
    // equality alone is not enough: `pack-us` and `pack-us-healthcare` share
    // OCI regions and are required to be isolated clusters, so a route between
    // them inside one region is a cross-pack move, not a no-op. This is reached
    // only after both endpoints resolved in the catalog.
    if !cross_pack && inputs.source.region_id == inputs.destination.region_id {
        return allow(ResidencyRule::SameRegion);
    }

    // Data-class rules first: these classes are refused by what they ARE, and
    // the audit record should say so rather than reporting the coarser
    // residency-class reason that would also have refused them.
    if cross_jurisdiction && inputs.data_class == ResidencyDataClass::SensitivePipaArt23 {
        return deny(
            ResidencyDecision::DenyDataClass,
            ResidencyRule::SensitiveDataCrossJurisdiction,
        );
    }
    if cross_pack && inputs.data_class == ResidencyDataClass::Secret {
        return deny(
            ResidencyDecision::DenyDataClass,
            ResidencyRule::SecretCrossPack,
        );
    }

    // DSR receipt aggregation is a named permitted route (IP-020 §D.4), but
    // only inside the pack: `data-residency.md` keeps each pack's audit chain
    // in that pack and forbids cross-pack replication by default. Naming the
    // refusal here keeps a blocked DSR fan-in distinguishable from ordinary
    // cross-pack traffic in the audit trail.
    if cross_pack && inputs.operation.is_dsr_aggregation() {
        return deny(
            ResidencyDecision::DenyJurisdictionPack,
            ResidencyRule::DsrAggregationRequiresIntraPack,
        );
    }

    let dr_pair_endpoint =
        inputs.source.role == RegionRole::DrPair || inputs.destination.role == RegionRole::DrPair;

    // Residency class.
    match inputs.residency_class {
        ResidencyClass::StrictHomeRegion | ResidencyClass::StrictFederatedRegion => {
            return deny(
                ResidencyDecision::DenyResidency,
                ResidencyRule::StrictResidencyForbidsCrossRegion,
            );
        }
        ResidencyClass::HomeWithRecoveryFailover => {
            // `multi-region.md` §Failback documents failback as a real,
            // scheduled procedure that "mirrors DR Failover steps in reverse",
            // and the Replication table constrains replication to "intra-pack
            // only" with NO direction constraint. So the test is that the pair
            // is intra-pack and one endpoint is the DR region — not that the
            // DR region is the destination, which would make the class that
            // exists to support DR the only one unable to complete a DR cycle.
            let recovery_route = !cross_pack
                && dr_pair_endpoint
                && (inputs.operation.is_recovery_operation()
                    || inputs.operation.is_dsr_aggregation());
            if !recovery_route {
                return deny(
                    ResidencyDecision::DenyResidency,
                    ResidencyRule::RecoveryFailoverRequiresIntraPackDrPair,
                );
            }
        }
        ResidencyClass::Global => {}
    }

    // Cross-jurisdiction needs a recorded legal basis. "Forbidden by default"
    // is the documented posture, so the fallthrough here is a denial, and the
    // requirements below are conjunctive: each one that applies must be met.
    if cross_jurisdiction {
        if inputs.source.jurisdiction == HEALTHCARE_JURISDICTION
            || inputs.destination.jurisdiction == HEALTHCARE_JURISDICTION
        {
            return deny(
                ResidencyDecision::DenyJurisdictionPack,
                ResidencyRule::HealthcarePackCrossJurisdictionUnauthorised,
            );
        }

        // GDPR first, and for EVERY operation including migration.
        // `multi-region.md` §"Cross-pack replication: FORBIDDEN by default"
        // states it unconditionally: EU-resident tenant metadata never reaches
        // a non-EU region without a Schrems-II-compatible SCC + supplementary
        // measures on file. A migration permit is not an SCC and does not
        // stand in for one.
        let eu_source = inputs.source.jurisdiction == EU_JURISDICTION;
        if eu_source && !inputs.scc_basis_established() {
            return deny(
                ResidencyDecision::DenyJurisdictionPack,
                ResidencyRule::EuTransferRequiresScc,
            );
        }

        if inputs.operation == ResidencyOperation::MigrateTenantCrossJurisdiction {
            if !inputs.migration_permit_established() {
                return deny(
                    ResidencyDecision::DenyJurisdictionPack,
                    ResidencyRule::CrossJurisdictionMigrationRequiresPermit,
                );
            }
            return allow(ResidencyRule::CrossJurisdictionMigrationAuthorised);
        }

        if eu_source {
            return allow(ResidencyRule::EuTransferAuthorisedByScc);
        }

        return deny(
            ResidencyDecision::DenyJurisdictionPack,
            ResidencyRule::CrossJurisdictionForbiddenByDefault,
        );
    }

    // Same jurisdiction, different pack. Two packs may share a jurisdiction and
    // still be isolated clusters, so pack identity — not jurisdiction — decides
    // here.
    if cross_pack {
        return deny(
            ResidencyDecision::DenyJurisdictionPack,
            ResidencyRule::CrossPackWithinJurisdictionForbidden,
        );
    }

    // Intra-pack. Label the allow for what it actually is, so an operator
    // auditing "which allows were granted under DR authority" does not get a
    // count that includes every ordinary in-pack event.
    if inputs.operation.is_dsr_aggregation() {
        return allow(ResidencyRule::IntraPackDsrReceiptAggregation);
    }
    if dr_pair_endpoint && inputs.operation.is_recovery_operation() {
        return allow(ResidencyRule::IntraPackDrPairTransfer);
    }
    allow(ResidencyRule::IntraPackTransfer)
}
