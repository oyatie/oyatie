//! Data-residency enforcer adapter — the guard that decides whether tenant data
//! may cross a border.
//!
//! Implements IP-020 (`tenancy/IP-020-data-residency-enforcer-adapter.md`),
//! collapsed from that plan's four-crate `-{kernel,domain,usecase,adapter}`
//! layout into one crate as a module tree: [`kernel`] (closed vocabulary +
//! catalog and register ports), [`domain`] (the pure decision cascade),
//! [`usecase`] (the evaluator that parses and resolves), [`inmemory`] (region
//! catalog, transfer register, audit sink). The capability is capped at 12
//! crates and the lockfile is frozen, so the collapse is a decided design
//! point.
//!
//! # What the policy actually says
//!
//! The rule set mirrors what this repository really writes down, not what the
//! plan assumed:
//!
//! - `tenancy/policy/data-residency.cedar` — five `forbid` rules: strict-tenant
//!   processing outside its home jurisdiction; KR-CSAP off the `kr` processing
//!   region; EU-sovereign off an `eu-sovereign-*` region; CN-PIPL off
//!   `cn-onshore`; DR cell outside the home jurisdiction for a strict tenant.
//! - `tenancy/policy/data-residency.md` — cross-pack replication forbidden by
//!   default, with three named exceptions (tenant-executed SCCs, HIPAA DR
//!   failover inside the healthcare pack, scheduled intra-pack BCDR drills),
//!   the `jurisdiction_code` / pack roster, and the DSR receipt cascade.
//! - `tenancy/cedar/policies.cedar` — `MigrateTenantCrossJurisdiction` is
//!   forbidden unless the resource is the `tenancy` µservice, BOTH permit ids
//!   are non-empty, and audit-chain emission is on. All four conjuncts are
//!   enforced; see [`kernel::TransferBasis::is_valid_cross_jurisdiction_permit`].
//! - `tenancy/multi-region.md` — every replication mode is `intra-pack only`
//!   with no direction constraint (§Failback is a real, scheduled procedure);
//!   the 11-pack region roster; and the unconditional sentence that EU-resident
//!   tenant metadata never reaches a non-EU region without a Schrems-II
//!   -compatible SCC + supplementary measures on file.
//! - `tenancy/compliance.md` §Domain 14 — a transfer check uses
//!   `jurisdiction_code`, home cell, pack roster and data class TOGETHER; no
//!   single field decides alone. Higher-restriction-wins on pack conflict.
//! - `tenancy/dpia.md` §2.2 — cross-border transfer forbidden by default;
//!   SCC-only for GDPR-scope tenants.
//!
//! Every arm of the cascade names its source; see
//! [`domain::ResidencyRule::citation`].
//!
//! # Posture
//!
//! Fail closed, everywhere. An unrecognised data class, residency class,
//! operation, residency overlay, or region is a DENIAL, never an allow — a
//! residency control that permits on input it does not understand is not a
//! control. A region that hosts more than one pack is likewise a denial until
//! the caller says which cell it means. A catalog or register that cannot
//! answer, or whose rows are structurally unusable, is an error rather than a
//! decision; use [`dispatch_permitted`] so that error can never be read as
//! permission.
//!
//! Cross-border requirements are CONJUNCTIVE. An EU-sourced tenant migration
//! owes both the GDPR transfer basis and the Cedar migration permit; meeting
//! one does not excuse the other, and neither is believed on the caller's word
//! — [`kernel::ResidencyTransferRegister`] has to confirm it for this tenant
//! and this route.
//!
//! Every decision that leaves this crate produces exactly one audit record: a
//! denial via [`ResidencyDenialAuditSink::emit_denial_detailed`], an authorised
//! boundary crossing via
//! [`ResidencyDenialAuditSink::emit_transfer_seal_detailed`]. Both carry the
//! [`domain::ResidencyRule`] that decided, so a caller defect and a compliance
//! block are distinguishable rows. If the audit path is down, [`enforce`]
//! propagates [`ResidencyAdapterError::AuditSinkUnavailable`] — an unevidenced
//! decision surfaces as an error rather than as a quiet allow.
//!
//! # Gaps
//!
//! Deliberately deferred, and why:
//!
//! - **Cedar is not linked.** `cedar-policy` is a third-party dependency and
//!   this lane may not touch `Cargo.lock`, so [`domain::decide`] is a
//!   HAND-WRITTEN MIRROR of `tenancy/policy/data-residency.cedar`, not a
//!   replacement for it. The `.cedar` fragment remains the authority. Nothing
//!   binds the two: there is no automated check that this cascade still agrees
//!   with the policy file, and such a check would have to live outside this
//!   crate's envelope (it needs the Cedar evaluator and the policy corpus).
//!   Until it exists, keeping the mirror in step is a human obligation, and a
//!   silent divergence between them is the most likely way this control fails.
//! - **Two Cedar overlay rules cannot be satisfied by the documented roster.**
//!   The fragment requires an `eu-sovereign` tenant's processing region to
//!   start with `eu-sovereign-` and a `kr-csap` tenant's to be literally `kr`.
//!   No region in `tenancy/multi-region.md` is either — the EU pack runs on
//!   `eu-frankfurt-1`/`eu-amsterdam-1` and the KR pack on `ap-seoul-1`. Under
//!   the literal rules those tenants may be processed NOWHERE in the documented
//!   roster, including their own home region. This crate implements both
//!   literally and denies, in the same shape as the `cn-onshore` rule, because
//!   a contradiction between two policy artifacts is not a licence to invent
//!   the permissive reading for whichever one is convenient. Resolving it needs
//!   a policy change, not a code change.
//! - **The tenant's pack is the caller's assertion.** `pack-us` and
//!   `pack-us-healthcare` are documented on the SAME OCI regions
//!   (`us-ashburn-1`, `us-phoenix-1`) with different `jurisdiction_code`s, so a
//!   region id does not identify a cell. [`ResidencyContext::with_pack`] lets
//!   the caller name the cell, and a route that leaves an ambiguous region
//!   unqualified is refused rather than resolved arbitrarily. What this crate
//!   cannot do is verify the claim: there is no tenant-registry port here, so a
//!   caller that labels a `pack-us-healthcare` tenant as `pack-us` gets the
//!   `pack-us` rules. Binding the pack to the tenant record needs the registry
//!   adapter that lands with the database dependency.
//! - **Nothing loads the transfer register.**
//!   [`kernel::ResidencyTransferRegister`] is the port that turns an asserted
//!   basis into a checked one, and [`inmemory::InMemoryTransferRegister`] is a
//!   real implementation of it — but the rows have to be handed in by the
//!   caller. `data-residency.md` cites the register as
//!   `microservices/tenancy/legal/transfer-register.md`, a path that does not
//!   exist in this tree, and this crate parses no document and reaches no
//!   database. An engine built with [`usecase::ResidencyPolicyEngine::try_new`]
//!   therefore has an EMPTY register and authorises no SCC transfer and no
//!   cross-jurisdiction migration at all. That is the fail-closed default, not
//!   a working integration.
//! - **No consent field.** `data-residency.md` §pack-kr permits sensitive
//!   cross-border data with explicit PIPA Art. 23-2 consent. This context
//!   carries no consent evidence, so `SENSITIVE_PIPA_ART23` crossing a
//!   jurisdiction always denies. That is the fail-closed reading of a rule
//!   whose permitting half has no input.
//! - **Cross-pack DSR receipt aggregation stays denied.** IP-020 §D.4 asks for
//!   DSR aggregation as a permitted route "where policy allows it". Policy
//!   allows it inside the pack: `data-residency.md` gives every pack its own
//!   audit-chain instance and forbids cross-pack replication by default, so no
//!   sentence in the corpus authorises a cross-pack receipt fan-in. It is
//!   refused under its own rule
//!   ([`domain::ResidencyRule::DsrAggregationRequiresIntraPack`]) rather than
//!   folded into the generic cross-pack denial, so the audit trail shows a
//!   blocked DSR fan-in as what it is. If the platform really does aggregate
//!   receipts across packs, the corpus — not this cascade — is what has to say
//!   so first.
//! - **`home_jurisdiction` is approximated by the source cell.**
//!   `ResidencyContext` has no home-jurisdiction field, so the Cedar rule
//!   "processing region != home jurisdiction" is evaluated as "destination
//!   jurisdiction != source-cell jurisdiction". For an outbound guard the
//!   source IS home; for a route whose source is already a DR region it is not,
//!   and that case is not distinguished.
//! - **The residency-overlay projection is the caller's.**
//!   `ResidencyContext::residency_overlays` is the closed three-value set of
//!   overlays the Cedar fragment knows (`kr-csap`, `eu-sovereign`, `cn-pipl`),
//!   not the tenant's full compliance-pack list. Projecting a tenant's packs
//!   onto it is the caller's job, and nothing here verifies the projection is
//!   complete — a residency-bearing pack the caller forgets to project simply
//!   is not enforced.
//! - **The audit record has no clock and no retention.**
//!   [`inmemory::ResidencyDenialRecord`] carries the rule, the route and a
//!   monotonic `sequence`, plus whatever
//!   [`ResidencyContext::audit_correlation_id`] the caller set (IP-020 §D.1) —
//!   but no timestamp, because the domain reads no clock and this crate takes
//!   no clock port. It also applies no retention window and no redaction, so
//!   records hold tenant identifiers until drained. The in-memory sink is
//!   BOUNDED and refuses at capacity rather than growing without limit, which
//!   is a memory-safety property, not a retention policy.
//! - **No async, no I/O, no crypto, no external adapters.** No Postgres
//!   tenant-registry, no audit-chain client, no event/RPC transport wrapper
//!   (IP-020 §D.2 `guard_outbound_event` / `guard_rpc_call`), no
//!   `oya.tenancy.residency-violation-blocked` emission, no correlation-id
//!   minting, and no `contracts/asyncapi/tenant-events.yaml` residency labels.
//!   Those need `tokio`, `sqlx`, and a signing stack; all three are dependencies
//!   and the lockfile is frozen. The ports are sync so the decision stays a
//!   reproducible pure function; wrapping them in an async transport is an
//!   adapter concern that lands with the dependency.
//! - **`ResidencyContext` duplicates vocabulary that already exists.**
//!   `tenancy/core/kernel::ResidencyClass` and `network/core/residency` define
//!   the same labels. [`kernel::ResidencyClass`] copies them byte-for-byte
//!   rather than depending on either crate, because a path dependency also
//!   rewrites `Cargo.lock`. It should collapse onto the kernel type when the
//!   lockfile unfreezes.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{EvaluationInputs, ResidencyOutcome, ResidencyRule, decide};
pub use kernel::{
    CrossJurisdictionPermitEntry, NoTransferRegister, RegionRecord, RegionRole, ResidencyClass,
    ResidencyDataClass, ResidencyOperation, ResidencyOverlay, ResidencyRegionCatalog,
    ResidencyTransferRegister, SccRegisterEntry, TransferBasis,
};
pub use usecase::ResidencyPolicyEngine;

use core::fmt;

/// One route to decide: who, from where, to where, carrying what, under which
/// residency class, doing which operation, with what legal basis.
///
/// The strings are the adapter boundary and are UNTRUSTED. Nothing here is
/// pre-validated; [`usecase::ResidencyPolicyEngine`] parses every one of them
/// against a closed vocabulary, resolves both endpoints to exactly one cell,
/// and checks every asserted basis against the legal-transfer register. It
/// denies anything it cannot recognise, cannot disambiguate, or cannot confirm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyContext {
    /// The tenant whose data is moving.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Region the data is leaving.
    pub source_region: String, // data_class: INTERNAL_ONLY
    /// Region the data would arrive in.
    pub destination_region: String, // data_class: INTERNAL_ONLY
    /// Class of the data being moved; see [`kernel::ResidencyDataClass`].
    pub data_class: String, // data_class: TENANT_SCOPED
    /// The tenant's residency class; see [`kernel::ResidencyClass`].
    pub residency_class: String, // data_class: TENANT_SCOPED
    /// The operation being guarded; see [`kernel::ResidencyOperation`].
    pub operation: String, // data_class: INTERNAL_ONLY
    /// The tenant's residency overlays; see [`kernel::ResidencyOverlay`]. This
    /// is NOT the tenant's whole compliance-pack list.
    pub residency_overlays: Vec<String>, // data_class: TENANT_SCOPED
    /// The legal basis asserted for an otherwise-forbidden route, if any.
    pub transfer_basis: Option<TransferBasis>, // data_class: TENANT_SCOPED
    /// Further asserted bases, for routes that owe more than one — an
    /// EU-sourced tenant migration owes both an SCC and a migration permit.
    pub additional_transfer_bases: Vec<TransferBasis>, // data_class: TENANT_SCOPED
    /// Which pack's cell in `source_region` this route leaves from.
    ///
    /// Required when the region hosts more than one pack, which the documented
    /// `pack-us` / `pack-us-healthcare` co-tenancy on `us-ashburn-1` does.
    /// `None` resolves only when the region holds exactly one cell.
    pub tenant_pack: Option<String>, // data_class: TENANT_SCOPED
    /// Which pack's cell in `destination_region` this route arrives at.
    /// Defaults to nothing, NOT to `tenant_pack`: a cross-pack route has to say
    /// so out loud.
    pub destination_pack: Option<String>, // data_class: INTERNAL_ONLY
    /// The caller's correlation id for this dispatch (IP-020 §D.1), copied onto
    /// the audit record so a denial can be joined to the request that caused
    /// it. Nothing here mints one.
    pub audit_correlation_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl ResidencyContext {
    /// A route with no overlays, no declared packs and no asserted transfer
    /// basis.
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        source_region: impl Into<String>,
        destination_region: impl Into<String>,
        data_class: impl Into<String>,
        residency_class: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            source_region: source_region.into(),
            destination_region: destination_region.into(),
            data_class: data_class.into(),
            residency_class: residency_class.into(),
            operation: operation.into(),
            residency_overlays: Vec::new(),
            transfer_basis: None,
            additional_transfer_bases: Vec::new(),
            tenant_pack: None,
            destination_pack: None,
            audit_correlation_id: None,
        }
    }

    /// Attach residency overlays.
    #[must_use]
    pub fn with_overlays<S: Into<String>>(mut self, overlays: impl IntoIterator<Item = S>) -> Self {
        self.residency_overlays = overlays.into_iter().map(Into::into).collect();
        self
    }

    /// Attach an asserted legal basis for the route.
    #[must_use]
    pub fn with_transfer_basis(mut self, basis: TransferBasis) -> Self {
        self.transfer_basis = Some(basis);
        self
    }

    /// Attach further asserted bases, for a route that owes more than one.
    #[must_use]
    pub fn with_additional_transfer_bases(
        mut self,
        bases: impl IntoIterator<Item = TransferBasis>,
    ) -> Self {
        self.additional_transfer_bases = bases.into_iter().collect();
        self
    }

    /// Declare that both endpoints are cells of the same pack — the ordinary
    /// intra-pack case.
    #[must_use]
    pub fn with_pack(mut self, pack_id: impl Into<String>) -> Self {
        let pack_id = pack_id.into();
        self.tenant_pack = Some(pack_id.clone());
        self.destination_pack = Some(pack_id);
        self
    }

    /// Declare which pack's cell the route leaves from.
    #[must_use]
    pub fn with_tenant_pack(mut self, pack_id: impl Into<String>) -> Self {
        self.tenant_pack = Some(pack_id.into());
        self
    }

    /// Declare which pack's cell the route arrives at.
    #[must_use]
    pub fn with_destination_pack(mut self, pack_id: impl Into<String>) -> Self {
        self.destination_pack = Some(pack_id.into());
        self
    }

    /// Attach the caller's audit correlation id.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.audit_correlation_id = Some(correlation_id.into());
        self
    }

    /// Whether this route names two different regions at all.
    #[must_use]
    pub fn is_cross_region(&self) -> bool {
        self.source_region != self.destination_region
    }

    /// Whether this route names two different packs.
    ///
    /// Only observable when the caller declared both; an undeclared pack is not
    /// evidence of sameness.
    #[must_use]
    pub fn is_cross_pack(&self) -> bool {
        match (
            self.tenant_pack.as_deref(),
            self.destination_pack.as_deref(),
        ) {
            (Some(source), Some(destination)) => source != destination,
            _ => false,
        }
    }

    /// Whether this route leaves the cell it started in — a different region,
    /// or a different pack inside one region. This is what has to be sealed
    /// when it is authorised.
    #[must_use]
    pub fn crosses_cell_boundary(&self) -> bool {
        self.is_cross_region() || self.is_cross_pack()
    }

    /// Every basis the caller asserted, primary first.
    pub fn transfer_bases(&self) -> impl Iterator<Item = &TransferBasis> {
        self.transfer_basis
            .iter()
            .chain(self.additional_transfer_bases.iter())
    }
}

/// What the enforcer concluded about a route.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyDecision {
    /// The route may be dispatched.
    Allow,
    /// Refused by the tenant's residency class or an unresolvable region.
    DenyResidency,
    /// Refused by what the data IS, independent of the route.
    DenyDataClass,
    /// Refused by a jurisdiction/pack boundary or a residency overlay.
    DenyJurisdictionPack,
}

impl ResidencyDecision {
    /// A stable slug for logs and metrics.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::DenyResidency => "deny-residency",
            Self::DenyDataClass => "deny-data-class",
            Self::DenyJurisdictionPack => "deny-jurisdiction-pack",
        }
    }

    /// Whether this decision permits dispatch.
    #[must_use]
    pub const fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }
}

impl fmt::Display for ResidencyDecision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

/// The policy port: decide one route.
pub trait ResidencyPolicyEvaluator {
    /// Decide whether `ctx` may be dispatched.
    ///
    /// # Errors
    ///
    /// Any [`ResidencyAdapterError`] the evaluator cannot turn into a decision.
    /// An error is NOT an allow.
    fn evaluate(&self, ctx: &ResidencyContext) -> Result<ResidencyDecision, ResidencyAdapterError>;

    /// Decide, and say WHICH rule decided.
    ///
    /// The default implementation reports [`ResidencyRule::RuleNotReported`],
    /// which is honest about an evaluator that cannot name its reason rather
    /// than inventing one. [`usecase::ResidencyPolicyEngine`] overrides it.
    ///
    /// # Errors
    ///
    /// As [`Self::evaluate`].
    fn evaluate_outcome(
        &self,
        ctx: &ResidencyContext,
    ) -> Result<ResidencyOutcome, ResidencyAdapterError> {
        Ok(ResidencyOutcome {
            decision: self.evaluate(ctx)?,
            rule: ResidencyRule::RuleNotReported,
        })
    }
}

/// The evidence port: record what was decided about a route.
pub trait ResidencyDenialAuditSink {
    /// Record exactly one denial.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::AuditSinkUnavailable`] when the evidence cannot
    /// be written. The caller must not proceed: a denial nobody recorded is an
    /// incident with no evidence.
    fn emit_denial(
        &self,
        ctx: &ResidencyContext,
        decision: ResidencyDecision,
    ) -> Result<(), ResidencyAdapterError>;

    /// Record one denial WITH the rule that produced it.
    ///
    /// This is the path [`enforce`] uses. The default implementation drops the
    /// rule and delegates to [`Self::emit_denial`], so an existing sink keeps
    /// working — but a sink that only implements `emit_denial` cannot tell a
    /// caller defect from a compliance block afterwards, which is the whole
    /// reason [`ResidencyRule`] exists.
    ///
    /// # Errors
    ///
    /// As [`Self::emit_denial`].
    fn emit_denial_detailed(
        &self,
        ctx: &ResidencyContext,
        outcome: ResidencyOutcome,
    ) -> Result<(), ResidencyAdapterError> {
        self.emit_denial(ctx, outcome.decision)
    }

    /// Record an AUTHORISED transfer across a cell boundary, as
    /// `tenancy/policy/data-residency.md` §"Exception: tenant-executed SCCs"
    /// requirement 4 demands of every transfer event.
    ///
    /// The default implementation REFUSES rather than silently doing nothing: a
    /// sink that has not implemented sealing cannot be used with [`enforce`],
    /// and finding that out as an error beats finding it out as a missing audit
    /// trail.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::AuditSinkUnavailable`] when the seal cannot be
    /// written, and by default always.
    fn emit_transfer_seal(&self, ctx: &ResidencyContext) -> Result<(), ResidencyAdapterError> {
        let _ = ctx;
        Err(ResidencyAdapterError::AuditSinkUnavailable)
    }

    /// Record an authorised transfer WITH the rule that authorised it.
    ///
    /// # Errors
    ///
    /// As [`Self::emit_transfer_seal`].
    fn emit_transfer_seal_detailed(
        &self,
        ctx: &ResidencyContext,
        rule: ResidencyRule,
    ) -> Result<(), ResidencyAdapterError> {
        let _ = rule;
        self.emit_transfer_seal(ctx)
    }
}

/// Everything that stops this adapter from producing a usable decision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyAdapterError {
    /// The policy surface itself is unusable — an empty catalog, a row with a
    /// blank jurisdiction, or a contradictory duplicate row.
    PolicyMalformed,
    /// The evaluator could not reach something it needed.
    EvaluationFailed,
    /// The decision could not be recorded.
    AuditSinkUnavailable,
}

impl ResidencyAdapterError {
    /// A stable slug for logs and metrics.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolicyMalformed => "policy-malformed",
            Self::EvaluationFailed => "evaluation-failed",
            Self::AuditSinkUnavailable => "audit-sink-unavailable",
        }
    }
}

impl fmt::Display for ResidencyAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PolicyMalformed => "residency policy is malformed; no route may be dispatched",
            Self::EvaluationFailed => "residency evaluation failed; no route may be dispatched",
            Self::AuditSinkUnavailable => {
                "residency decision could not be recorded; no route may be dispatched"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ResidencyAdapterError {}

/// Evaluate a route and record what was decided.
///
/// A non-`Allow` decision is written to the audit sink with the rule that
/// produced it. An `Allow` that crosses a cell boundary is SEALED, because
/// `data-residency.md` requirement 4 and the `audit_chain_emit` conjunct of the
/// Cedar migration rule both demand evidence of an authorised transfer, not
/// only of a refused one; an authorised cross-border move that left no record
/// is the failure this closes. Same-cell traffic crosses no boundary and is not
/// sealed.
///
/// If the sink refuses either write, so does this — the caller gets an error,
/// not a decision it could mistake for permission.
///
/// # Errors
///
/// Whatever the evaluator or the sink reports, including a seal that could not
/// be written: an unsealed authorised transfer is refused rather than
/// dispatched unrecorded.
pub fn enforce<E: ResidencyPolicyEvaluator, S: ResidencyDenialAuditSink>(
    evaluator: &E,
    sink: &S,
    ctx: &ResidencyContext,
) -> Result<ResidencyDecision, ResidencyAdapterError> {
    Ok(enforce_detailed(evaluator, sink, ctx)?.decision)
}

/// [`enforce`], returning the rule as well as the decision.
///
/// The rule the caller gets back here is the one that was written to the audit
/// record, so a caller can log the two consistently without re-running the
/// engine.
///
/// # Errors
///
/// As [`enforce`].
pub fn enforce_detailed<E: ResidencyPolicyEvaluator, S: ResidencyDenialAuditSink>(
    evaluator: &E,
    sink: &S,
    ctx: &ResidencyContext,
) -> Result<ResidencyOutcome, ResidencyAdapterError> {
    let outcome = evaluator.evaluate_outcome(ctx)?;
    if outcome.is_allow() {
        if ctx.crosses_cell_boundary() {
            sink.emit_transfer_seal_detailed(ctx, outcome.rule)?;
        }
        return Ok(outcome);
    }
    sink.emit_denial_detailed(ctx, outcome)?;
    Ok(outcome)
}

/// [`enforce`] under its older name, kept because it is published.
///
/// The two are now the same function: sealing an authorised boundary crossing
/// is not an opt-in variant of enforcement, it is part of it.
///
/// # Errors
///
/// As [`enforce`].
pub fn enforce_sealed<E: ResidencyPolicyEvaluator, S: ResidencyDenialAuditSink>(
    evaluator: &E,
    sink: &S,
    ctx: &ResidencyContext,
) -> Result<ResidencyDecision, ResidencyAdapterError> {
    enforce(evaluator, sink, ctx)
}

/// Whether an enforcement result permits dispatch.
///
/// The single place that answers "may this go out?", so no caller has to
/// remember that an `Err` is a refusal. Only `Ok(Allow)` is permission;
/// every error and every denial is not.
#[must_use]
pub fn dispatch_permitted(result: &Result<ResidencyDecision, ResidencyAdapterError>) -> bool {
    matches!(result, Ok(ResidencyDecision::Allow))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inmemory::{InMemoryDenialAuditSink, InMemoryRegionCatalog, default_engine};

    fn ctx(source: &str, destination: &str) -> ResidencyContext {
        ResidencyContext::new(
            "tenant:abc",
            source,
            destination,
            "INTERNAL_ONLY",
            "global",
            "emit_event",
        )
    }

    fn dr_route() -> ResidencyContext {
        ResidencyContext::new(
            "tenant:abc",
            "us-ashburn-1",
            "us-phoenix-1",
            "INTERNAL_ONLY",
            "home_with_recovery_failover",
            "replicate_storage",
        )
        .with_pack("pack-us")
    }

    #[test]
    fn dispatch_permitted_only_for_ok_allow() {
        assert!(dispatch_permitted(&Ok(ResidencyDecision::Allow)));
        assert!(!dispatch_permitted(&Ok(ResidencyDecision::DenyResidency)));
        assert!(!dispatch_permitted(&Ok(ResidencyDecision::DenyDataClass)));
        assert!(!dispatch_permitted(&Ok(
            ResidencyDecision::DenyJurisdictionPack
        )));
        for error in [
            ResidencyAdapterError::PolicyMalformed,
            ResidencyAdapterError::EvaluationFailed,
            ResidencyAdapterError::AuditSinkUnavailable,
        ] {
            assert!(
                !dispatch_permitted(&Err(error)),
                "{} must not read as permission",
                error.code()
            );
        }
    }

    #[test]
    fn empty_catalog_is_malformed_policy_not_a_permissive_one() {
        let engine = ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(Vec::new()));
        assert_eq!(engine.err(), Some(ResidencyAdapterError::PolicyMalformed));
    }

    #[test]
    fn blank_jurisdiction_row_is_malformed_policy() {
        let engine =
            ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(vec![RegionRecord::new(
                "ap-seoul-1",
                "pack-kr",
                "  ",
                RegionRole::Primary,
            )]));
        assert_eq!(engine.err(), Some(ResidencyAdapterError::PolicyMalformed));
    }

    #[test]
    fn unreachable_catalog_fails_evaluation_rather_than_allowing() {
        let mut catalog = InMemoryRegionCatalog::oyatie_pack_roster();
        catalog.set_unavailable(true);
        let engine = ResidencyPolicyEngine::try_new(catalog);
        assert_eq!(engine.err(), Some(ResidencyAdapterError::EvaluationFailed));
    }

    #[test]
    fn enforce_returns_allow_and_writes_nothing_for_same_region() {
        let engine = default_engine().unwrap();
        let sink = InMemoryDenialAuditSink::new();
        let decision = enforce(&engine, &sink, &ctx("ap-seoul-1", "ap-seoul-1")).unwrap();
        assert_eq!(decision, ResidencyDecision::Allow);
        assert_eq!(sink.denial_count(), 0);
        assert_eq!(sink.seal_count(), 0);
    }

    #[test]
    fn every_denial_writes_exactly_one_audit_record_naming_its_rule() {
        let engine = default_engine().unwrap();
        let sink = InMemoryDenialAuditSink::new();
        let route = ctx("ap-seoul-1", "eu-frankfurt-1").with_correlation_id("corr-1");
        let decision = enforce(&engine, &sink, &route).unwrap();
        assert_eq!(decision, ResidencyDecision::DenyJurisdictionPack);
        assert_eq!(sink.denial_count(), 1);
        let record = sink.denials().first().cloned().unwrap();
        assert_eq!(record.decision, ResidencyDecision::DenyJurisdictionPack);
        assert_eq!(
            record.rule,
            ResidencyRule::CrossJurisdictionForbiddenByDefault
        );
        assert_eq!(
            record.context.audit_correlation_id.as_deref(),
            Some("corr-1")
        );
        assert_eq!(record.context, route);
    }

    #[test]
    fn a_denial_with_a_broken_audit_path_is_an_error_not_a_quiet_allow() {
        let engine = default_engine().unwrap();
        let sink = InMemoryDenialAuditSink::new();
        sink.set_unavailable(true);
        let result = enforce(&engine, &sink, &ctx("ap-seoul-1", "eu-frankfurt-1"));
        assert_eq!(result, Err(ResidencyAdapterError::AuditSinkUnavailable));
        assert!(!dispatch_permitted(&result));
        assert_eq!(sink.denial_count(), 0);
    }

    #[test]
    fn enforce_seals_an_authorised_cross_region_transfer() {
        let engine = default_engine().unwrap();
        let sink = InMemoryDenialAuditSink::new();
        let route = dr_route();
        let decision = enforce(&engine, &sink, &route).unwrap();
        assert_eq!(decision, ResidencyDecision::Allow);
        assert_eq!(sink.seal_count(), 1);
        assert_eq!(sink.denial_count(), 0);
        assert_eq!(
            sink.seals()[0].rule,
            ResidencyRule::IntraPackDrPairTransfer,
            "the seal must name the rule that authorised the crossing"
        );

        // The older published name is the same function, seal included.
        let alias_sink = InMemoryDenialAuditSink::new();
        enforce_sealed(&engine, &alias_sink, &route).unwrap();
        assert_eq!(alias_sink.seal_count(), 1);
    }

    #[test]
    fn enforce_does_not_seal_same_region_traffic() {
        let engine = default_engine().unwrap();
        let sink = InMemoryDenialAuditSink::new();
        enforce(&engine, &sink, &ctx("ap-tokyo-1", "ap-tokyo-1")).unwrap();
        assert_eq!(sink.seal_count(), 0);
        assert_eq!(sink.denial_count(), 0);
    }

    #[test]
    fn an_unsealable_authorised_transfer_is_refused() {
        struct SealBlindSink;
        impl ResidencyDenialAuditSink for SealBlindSink {
            fn emit_denial(
                &self,
                _ctx: &ResidencyContext,
                _decision: ResidencyDecision,
            ) -> Result<(), ResidencyAdapterError> {
                Ok(())
            }
        }
        let engine = default_engine().unwrap();
        let result = enforce(&engine, &SealBlindSink, &dr_route());
        assert_eq!(result, Err(ResidencyAdapterError::AuditSinkUnavailable));
        assert!(!dispatch_permitted(&result));
    }

    #[test]
    fn a_sink_that_only_implements_emit_denial_still_records_the_denial() {
        // The default `emit_denial_detailed` must not silently skip the write.
        struct MinimalSink {
            written: std::sync::atomic::AtomicUsize,
        }
        impl ResidencyDenialAuditSink for MinimalSink {
            fn emit_denial(
                &self,
                _ctx: &ResidencyContext,
                _decision: ResidencyDecision,
            ) -> Result<(), ResidencyAdapterError> {
                self.written
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
        let engine = default_engine().unwrap();
        let sink = MinimalSink {
            written: std::sync::atomic::AtomicUsize::new(0),
        };
        enforce(&engine, &sink, &ctx("ap-seoul-1", "eu-frankfurt-1")).unwrap();
        assert_eq!(sink.written.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn decision_and_error_codes_are_stable_and_display() {
        assert_eq!(ResidencyDecision::Allow.to_string(), "allow");
        assert_eq!(
            ResidencyDecision::DenyJurisdictionPack.to_string(),
            "deny-jurisdiction-pack"
        );
        assert!(ResidencyDecision::Allow.is_allow());
        assert!(!ResidencyDecision::DenyDataClass.is_allow());
        assert_eq!(
            ResidencyAdapterError::PolicyMalformed.code(),
            "policy-malformed"
        );
        assert!(
            ResidencyAdapterError::AuditSinkUnavailable
                .to_string()
                .contains("could not be recorded")
        );
    }

    #[test]
    fn context_builders_do_not_lose_fields() {
        let route = ctx("ap-seoul-1", "ap-tokyo-1")
            .with_overlays(["kr-csap"])
            .with_transfer_basis(TransferBasis::CrossJurisdictionCedarPermit {
                permit_id: "permit-1".to_owned(),
                cross_jurisdiction_permit_id: "xj-1".to_owned(),
                audit_chain_emit: true,
                microservice: "tenancy".to_owned(),
            })
            .with_pack("pack-kr")
            .with_correlation_id("corr-9");
        assert_eq!(route.residency_overlays, vec!["kr-csap".to_owned()]);
        assert!(route.transfer_basis.is_some());
        assert_eq!(route.transfer_bases().count(), 1);
        assert_eq!(route.tenant_pack.as_deref(), Some("pack-kr"));
        assert_eq!(route.destination_pack.as_deref(), Some("pack-kr"));
        assert_eq!(route.audit_correlation_id.as_deref(), Some("corr-9"));
        assert!(route.is_cross_region());
        assert!(!route.is_cross_pack());
        assert!(route.crosses_cell_boundary());
        assert!(!ctx("ap-seoul-1", "ap-seoul-1").is_cross_region());
    }

    #[test]
    fn a_route_between_two_packs_in_one_region_crosses_a_cell_boundary() {
        let route = ctx("us-ashburn-1", "us-ashburn-1")
            .with_tenant_pack("pack-us-healthcare")
            .with_destination_pack("pack-us");
        assert!(!route.is_cross_region());
        assert!(route.is_cross_pack());
        assert!(
            route.crosses_cell_boundary(),
            "same region, different isolated cluster, is still a crossing"
        );
    }
}
