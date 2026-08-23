//! Cross-axis contract types for M03-P08 (DESIGN §10 machine-readable surface).
//!
//! Adds three net-new types to the architecture-map kernel:
//!   `AxisKind`         — closed enum of the 7 product axes
//!   `CrossAxisContract` — a DESIGN §10 row binding two axes via a contract id
//!   `AxisBinding`      — directional owner → consumer-set pair with a stability tag
//!
//! All types are pure `std`-only; no serde, no workspace-dep additions.
//! Merge-variant execution: added to the existing kernel crate rather than
//! scaffolding a new crate, per `execution_variant=merge-into-existing-crates`
//! (decided 2026-05-17, user-directive-option-2).
//!
//! Naming justification:
//! - `AxisKind` — `Kind` suffix matches `NodeKind` / `EdgeKind` in `lib.rs`;
//!   `Axis` is the noun used in DESIGN §10 and `contracts.json`.
//! - `CrossAxisContract` — noun phrase directly mirrors the `cross_axis_contracts`
//!   key in `docs/machine-readable/contracts.json`; `Cross` + `Axis` + `Contract`
//!   each carries one semantic layer without abbreviation.
//! - `AxisBinding` — `Binding` is the directed owner→consumer relationship;
//!   `Axis` scopes it to this module, parallel to `Edge` in the graph model.
//! - `ConsumerSpec` — `Spec` matches the "spec" framing in DESIGN §10; the enum
//!   encodes both typed axis consumers and free-form labels (`"all"`,
//!   `"all-regulated"`, domain-scoped strings) present in `contracts.json`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

// ─── AxisKind ────────────────────────────────────────────────────────────────

/// Closed enum of the 7 product axes defined in DESIGN §10.
///
/// `non_exhaustive` is intentionally NOT applied: the 7-axis set is closed by
/// the master plan.  Any new axis requires a new ADR + version bump.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AxisKind {
    /// SaaS platform axis — tenancy, identity, policy, billing.
    Saas,
    /// Cloud infrastructure axis — compute, data, FinOps, capacity.
    Cloud,
    /// Search & discovery axis — indexing, ranking, retrieval.
    Search,
    /// Ads monetisation axis — targeting, auction, delivery.
    Ads,
    /// Vertical industry axis — Korea pack + future locale packs.
    Vertical,
    /// Workspace productivity axis — 14 surfaces.
    Workspace,
    /// Foundry / agent-runtime axis — capability invocation, subagent runtime.
    Foundry,
}

impl AxisKind {
    /// All axis variants in declaration order.
    pub const ALL: [AxisKind; 7] = [
        AxisKind::Saas,
        AxisKind::Cloud,
        AxisKind::Search,
        AxisKind::Ads,
        AxisKind::Vertical,
        AxisKind::Workspace,
        AxisKind::Foundry,
    ];

    /// Stable lowercase identifier used in contract ids and filenames.
    pub fn id(self) -> &'static str {
        match self {
            AxisKind::Saas => "saas",
            AxisKind::Cloud => "cloud",
            AxisKind::Search => "search",
            AxisKind::Ads => "ads",
            AxisKind::Vertical => "vertical",
            AxisKind::Workspace => "workspace",
            AxisKind::Foundry => "foundry",
        }
    }

    /// Human-readable display label.
    pub fn label(self) -> &'static str {
        match self {
            AxisKind::Saas => "SaaS Platform",
            AxisKind::Cloud => "Cloud Infrastructure",
            AxisKind::Search => "Search & Discovery",
            AxisKind::Ads => "Ads Monetisation",
            AxisKind::Vertical => "Vertical (Regional)",
            AxisKind::Workspace => "Workspace (14 surfaces)",
            AxisKind::Foundry => "Foundry / Agent-runtime",
        }
    }

    /// Parse from a stable id string.  Returns `Err(UnknownAxis)` on mismatch.
    pub fn parse(s: &str) -> Result<Self, UnknownAxis> {
        match s {
            "saas" => Ok(AxisKind::Saas),
            "cloud" => Ok(AxisKind::Cloud),
            "search" => Ok(AxisKind::Search),
            "ads" => Ok(AxisKind::Ads),
            "vertical" => Ok(AxisKind::Vertical),
            "workspace" => Ok(AxisKind::Workspace),
            "foundry" => Ok(AxisKind::Foundry),
            _ => Err(UnknownAxis {
                input: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for AxisKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Error returned by `AxisKind::parse` when the input does not match any
/// known axis id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownAxis {
    pub input: String, // data_class: INTERNAL_ONLY
}

impl fmt::Display for UnknownAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown axis id: {:?}", self.input)
    }
}

// ─── ConsumerSpec ─────────────────────────────────────────────────────────────

/// One entry in a contract's consumer set.
///
/// `contracts.json` `consumer_axes` values are heterogeneous: some are typed
/// axis identifiers (`"saas"`, `"foundry"`) and others are free-form labels
/// (`"all"`, `"all-regulated"`, `"RAG"`, `"ISVs"`, `"external"`,
/// `"agent-mediated cloud ops"`, etc.).  This enum preserves the full fidelity
/// of the source data without lossy coercion into `AxisKind`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConsumerSpec {
    /// A typed, well-known product axis.
    Axis(AxisKind),
    /// A free-form label used in `contracts.json` (e.g. `"all"`,
    /// `"all-regulated"`, `"ISVs"`, `"external"`).
    Label(&'static str),
}

impl ConsumerSpec {
    /// Parse a consumer string: try `AxisKind` first; fall back to `Label`.
    pub fn parse(s: &'static str) -> Self {
        match AxisKind::parse(s) {
            Ok(axis) => ConsumerSpec::Axis(axis),
            Err(_) => ConsumerSpec::Label(s),
        }
    }
}

impl fmt::Display for ConsumerSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsumerSpec::Axis(a) => f.write_str(a.id()),
            ConsumerSpec::Label(l) => f.write_str(l),
        }
    }
}

// ─── AxisBinding ─────────────────────────────────────────────────────────────

/// Stability tier of a cross-axis contract surface, mirroring the tiers in
/// `docs/machine-readable/contracts.json` → `_metadata.stability_tiers`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StabilityTier {
    Preview,
    Stable,
    Ga,
}

impl StabilityTier {
    /// Canonical stability tier identifier.
    ///
    /// Matches `docs/machine-readable/contracts.json` `_metadata.stability_tiers`
    /// exactly — note uppercase `"GA"` for the GA tier.
    pub fn id(self) -> &'static str {
        match self {
            StabilityTier::Preview => "preview",
            StabilityTier::Stable => "stable",
            StabilityTier::Ga => "GA",
        }
    }
}

/// A directional axis-to-axis dependency with a stability tier.
///
/// `owner` is the raw `owner_axis` string from `contracts.json`.  It is kept
/// as `&'static str` rather than `AxisKind` because the canonical registry
/// contains composite and domain-scoped owner identifiers that are not single
/// axis enum variants — for example `"foundry+governance"`,
/// `"platform-audit-evidence"`, `"cloud+saas"`, `"saas+cloud+ads+marketplace"`.
/// Coercing these into one `AxisKind` would either fail parsing or silently
/// drop owner semantics.
///
/// `consumers` is the full consumer set from `consumer_axes`.  It is a
/// `Box<[ConsumerSpec]>` so that contracts with multi-element consumer lists
/// (`["billing", "tax", "marketplace"]`) are represented without loss.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AxisBinding {
    /// Raw `owner_axis` value from `contracts.json` (may be composite).
    pub owner: &'static str, // data_class: INTERNAL_ONLY
    /// Full `consumer_axes` set from `contracts.json`.
    pub consumers: Box<[ConsumerSpec]>, // data_class: INTERNAL_ONLY
    pub stability: StabilityTier, // data_class: INTERNAL_ONLY
}

impl AxisBinding {
    pub fn new(
        owner: &'static str,
        consumers: impl Into<Box<[ConsumerSpec]>>,
        stability: StabilityTier,
    ) -> Self {
        Self {
            owner,
            consumers: consumers.into(),
            stability,
        }
    }
}

// ─── CrossAxisContract ───────────────────────────────────────────────────────

/// A DESIGN §10 cross-axis contract row, binding a contract id to a directed
/// `AxisBinding` and an optional artifact location string.
///
/// The `id` field matches the `id` key in `contracts.json#cross_axis_contracts`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossAxisContract {
    pub id: &'static str,                        // data_class: INTERNAL_ONLY
    pub binding: AxisBinding,                    // data_class: INTERNAL_ONLY
    pub artifact_location: Option<&'static str>, // data_class: INTERNAL_ONLY
}

impl CrossAxisContract {
    pub fn new(
        id: &'static str,
        binding: AxisBinding,
        artifact_location: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            binding,
            artifact_location,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_kind_all_has_seven_variants() {
        assert_eq!(AxisKind::ALL.len(), 7);
    }

    #[test]
    fn axis_kind_ids_are_unique() {
        let ids: std::collections::BTreeSet<_> = AxisKind::ALL.iter().map(|a| a.id()).collect();
        assert_eq!(ids.len(), 7);
    }

    #[test]
    fn axis_kind_parse_round_trips() {
        for axis in AxisKind::ALL {
            assert_eq!(AxisKind::parse(axis.id()).unwrap(), axis);
        }
    }

    #[test]
    fn axis_kind_parse_unknown_errors() {
        let err = AxisKind::parse("galaxy").unwrap_err();
        assert_eq!(err.input, "galaxy");
    }

    #[test]
    fn axis_kind_display_uses_label() {
        assert_eq!(format!("{}", AxisKind::Saas), AxisKind::Saas.label());
    }

    #[test]
    fn stability_tier_ids_distinct() {
        let tiers = [
            StabilityTier::Preview,
            StabilityTier::Stable,
            StabilityTier::Ga,
        ];
        let ids: std::collections::BTreeSet<_> = tiers.iter().map(|t| t.id()).collect();
        assert_eq!(ids.len(), 3);
    }

    /// P2: `StabilityTier::Ga` must emit `"GA"` (uppercase) to match the
    /// canonical `_metadata.stability_tiers` list in `contracts.json`.
    #[test]
    fn stability_tier_ga_id_is_uppercase() {
        assert_eq!(StabilityTier::Ga.id(), "GA");
    }

    #[test]
    fn axis_binding_fields_accessible() {
        let consumers = vec![ConsumerSpec::Axis(AxisKind::Cloud)];
        let b = AxisBinding::new("saas", consumers, StabilityTier::Preview);
        assert_eq!(b.owner, "saas");
        assert_eq!(b.consumers.len(), 1);
        assert_eq!(b.consumers[0], ConsumerSpec::Axis(AxisKind::Cloud));
        assert_eq!(b.stability, StabilityTier::Preview);
    }

    #[test]
    fn cross_axis_contract_construction() {
        let consumers = vec![ConsumerSpec::Axis(AxisKind::Cloud)];
        let b = AxisBinding::new("saas", consumers, StabilityTier::Stable);
        let c = CrossAxisContract::new("TENANT_KERNEL", b, Some("crates/tenancy-kernel"));
        assert_eq!(c.id, "TENANT_KERNEL");
        assert_eq!(c.binding.owner, "saas");
        assert_eq!(c.artifact_location, Some("crates/tenancy-kernel"));
    }

    #[test]
    fn cross_axis_contract_no_artifact() {
        let consumers = vec![ConsumerSpec::Label("RAG")];
        let b = AxisBinding::new("foundry+search", consumers, StabilityTier::Preview);
        let c = CrossAxisContract::new("FOUNDRY_SEARCH_RETRIEVAL_BOUNDARY", b, None);
        assert!(c.artifact_location.is_none());
    }

    #[test]
    fn axis_binding_ordering() {
        // "foundry" < "saas" lexicographically, so b1 sorts before b2.
        let b1 = AxisBinding::new(
            "foundry",
            vec![ConsumerSpec::Axis(AxisKind::Cloud)],
            StabilityTier::Preview,
        );
        let b2 = AxisBinding::new(
            "saas",
            vec![ConsumerSpec::Axis(AxisKind::Cloud)],
            StabilityTier::Ga,
        );
        assert!(b1 < b2);
    }

    // ── Synthetic P1 violation tests ─────────────────────────────────────────

    /// P1a: composite owner ids from `contracts.json` must round-trip through
    /// `AxisBinding.owner` without loss.  Any attempt to coerce these into a
    /// single `AxisKind` would either fail or drop semantics.
    #[test]
    fn axis_binding_preserves_composite_owner_ids() {
        let composite_owners = [
            "foundry+governance",          // AUTONOMY_CEILING_POLICY
            "platform-audit-evidence",     // AUDIT_CHAIN_EVENT
            "cloud+saas",                  // IAM_SSO_SAML_OIDC
            "cloud+saas-metering",         // BILLING_EVENT
            "saas+cloud",                  // WEBHOOK_SIGNING
            "saas+cloud+search+ads",       // PUBLIC_REST_STABILITY_TIER
            "foundry+saas",                // MARKETPLACE_LISTING
            "foundation-contracts",        // EVENTING_BACKBONE
            "cloud+search",                // CLOUD_SEARCH_CAPACITY_AND_RESIDENCY
            "search+ads",                  // SEARCH_ADS_SERP_AND_QUERY_PRIVACY
            "foundry+cloud",               // FOUNDRY_CLOUD_MUTATION_CONTROL
            "foundry+search",              // FOUNDRY_SEARCH_RETRIEVAL_BOUNDARY
            "saas+vertical+ads+analytics", // TENANT_ADS_ANALYTICS_ELIGIBILITY
            "saas+cloud+ads+marketplace",  // REVENUE_METERING_TAX_INVOICE
        ];
        for owner in composite_owners {
            let b = AxisBinding::new(
                owner,
                vec![ConsumerSpec::Label("all")],
                StabilityTier::Stable,
            );
            assert_eq!(
                b.owner, owner,
                "owner id '{owner}' was corrupted through AxisBinding"
            );
        }
    }

    /// P1b: consumer sets from `contracts.json` must be preserved in full,
    /// including multi-element sets and free-form labels that are not `AxisKind`
    /// variants.
    #[test]
    fn axis_binding_preserves_full_consumer_set() {
        // CLOUD_RESOURCE_TYPE: consumer_axes = ["cloud-customers", "tenant-resource-lifecycle", "billing"]
        let consumers = vec![
            ConsumerSpec::Label("cloud-customers"),
            ConsumerSpec::Label("tenant-resource-lifecycle"),
            ConsumerSpec::Label("billing"),
        ];
        let b = AxisBinding::new("cloud", consumers, StabilityTier::Stable);
        assert_eq!(b.consumers.len(), 3);

        // AUTONOMY_CEILING_POLICY: consumer_axes = ["all-regulated"]
        let consumers2 = vec![ConsumerSpec::Label("all-regulated")];
        let b2 = AxisBinding::new("foundry+governance", consumers2, StabilityTier::Stable);
        assert_eq!(b2.consumers[0], ConsumerSpec::Label("all-regulated"));

        // OBJECT_GRAPH_PROPERTY_TIER: consumer_axes = ["search", "vertical", "ads"]
        let consumers3 = vec![
            ConsumerSpec::Axis(AxisKind::Search),
            ConsumerSpec::Axis(AxisKind::Vertical),
            ConsumerSpec::Axis(AxisKind::Ads),
        ];
        let b3 = AxisBinding::new("saas", consumers3, StabilityTier::Stable);
        assert_eq!(b3.consumers.len(), 3);
        assert_eq!(b3.consumers[1], ConsumerSpec::Axis(AxisKind::Vertical));
    }

    /// `ConsumerSpec::parse` must resolve typed axis ids to `Axis(_)` and
    /// unrecognised strings to `Label(_)`.
    #[test]
    fn consumer_spec_parse_typed_and_freeform() {
        assert_eq!(
            ConsumerSpec::parse("saas"),
            ConsumerSpec::Axis(AxisKind::Saas)
        );
        assert_eq!(
            ConsumerSpec::parse("foundry"),
            ConsumerSpec::Axis(AxisKind::Foundry)
        );
        assert_eq!(ConsumerSpec::parse("all"), ConsumerSpec::Label("all"));
        assert_eq!(
            ConsumerSpec::parse("all-regulated"),
            ConsumerSpec::Label("all-regulated")
        );
        assert_eq!(ConsumerSpec::parse("ISVs"), ConsumerSpec::Label("ISVs"));
        assert_eq!(
            ConsumerSpec::parse("agent-mediated cloud ops"),
            ConsumerSpec::Label("agent-mediated cloud ops")
        );
    }

    /// `ConsumerSpec::Display` must emit the underlying id or label unchanged.
    #[test]
    fn consumer_spec_display() {
        assert_eq!(format!("{}", ConsumerSpec::Axis(AxisKind::Cloud)), "cloud");
        assert_eq!(
            format!("{}", ConsumerSpec::Label("all-regulated")),
            "all-regulated"
        );
    }
}
