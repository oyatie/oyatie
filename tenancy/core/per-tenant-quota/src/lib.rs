//! Per-tenant quota usecase — IP-022 (`tenancy/IP-022-per-tenant-quota-usecase.md`).
//!
//! Tenancy owns the canonical tenant record, so tenancy owns the canonical
//! quota: downstream services enforce locally, but the number and — just as
//! importantly — the *reason* for the number come from here. An audit event
//! that cannot say which policy layer refused a request is not an audit
//! event.
//!
//! What this crate implements:
//! - The closed resource vocabulary of IP-022 §D.1 ([`QuotaResource`]).
//! - Class (plan-tier) defaults for `trial` / `production` / `sandbox` /
//!   `internal` (§D.2) and pack ceilings for regulated packs (§D.3), in
//!   [`QuotaPolicyCatalog`].
//! - The precedence chain (§D.4): class default -> pack override -> tenant
//!   override, with hard caps clamping the winner. [`QuotaDecision::source`]
//!   names the layer that produced the enforced number, so a clamped override
//!   reports [`QuotaSource::HardCap`] and not the layer whose number lost.
//! - Packs tighten, they never entitle. A pack override displaces the class
//!   default only when it is *stricter*, so binding a compliance pack to a
//!   sandbox tenant cannot raise that tenant's ceiling to the pack's; and a
//!   regulated pack backs its ceiling with a pack hard cap, which a tenant
//!   override cannot buy past.
//!   [`QuotaPolicyCatalog::undefended_pack_overrides`] names any pack ceiling
//!   that forgot to.
//! - Pack identifiers fail closed. They are normalised (`US-HC` and `us_hc`
//!   both resolve to `us-hc`), and a pack the catalog does not declare is
//!   [`QuotaUsecaseError::UnknownPack`] rather than a silent no-op — the same
//!   rule [`QuotaResource::parse`] applies to resource names. A decision
//!   therefore never reports `pack: Some(..)` for a pack that was not
//!   actually consulted.
//! - `limit` vs `effective`: `limit` is what the winning layer *declared*;
//!   `effective` is what is *enforced* after clamping. `effective <= limit`.
//! - Reserve / commit / release accounting with soft-warn and hard-refuse
//!   thresholds and a per-window reset ([`QuotaLedger`]).
//! - In-memory adapters for both read ports ([`inmemory`]).
//!
//! Determinism: nothing here reads a clock or draws randomness. The observed
//! instant is an argument (`window_start`, `advance_to`), the policy is data
//! the caller supplies, and every arithmetic step is checked or saturating —
//! an unsigned underflow in quota accounting presents as an unlimited quota.
//!
//! # Gaps
//!
//! Deliberately deferred, and why:
//! - **Persistence.** IP-022 names a Postgres-backed `QuotaStore`. The
//!   lockfile is frozen for this wave, so no `sqlx`/`tokio` dependency may be
//!   added; [`inmemory`] holds the reference semantics the real adapter must
//!   reproduce, behind the same two ports.
//! - **Async.** The ports are synchronous. A network-backed adapter will need
//!   async, which is a port-signature change plus a runtime dependency —
//!   again lockfile-frozen. The pure logic is unaffected by that switch.
//! - **Cedar authorization (§D.5).** `update_quota` with its Cedar gate,
//!   actor, reason code and idempotency key is not implemented: it needs the
//!   policy-evaluation port and a real store. What lands here is the read /
//!   resolve / enforce half; the mutation half is a follow-up.
//! - **Event emission (§D.6).** `oya.tenancy.quota-updated`,
//!   `quota-breach` and `quota-soft-threshold-crossed` are not published. The
//!   *decisions* those events carry are modelled — [`QuotaOutcome`] is exactly
//!   the breach/soft-threshold distinction — but the emit port needs the
//!   messaging adapter.
//! - **Crate layout.** IP-022 sketches `resolve_quota.rs` / `update_quota.rs`
//!   / `decision.rs` / `ports.rs` across a multi-crate slice; the capability
//!   is capped at 12 crates, so this collapses into one crate's module tree
//!   (`kernel` / `domain` / `usecase` / `inmemory`).
//! - **Soft thresholds are a percentage of the effective ceiling**, not an
//!   independently declared absolute value. Absolute per-resource thresholds
//!   can be added to [`QuotaAllowance`] without changing the chain.
//! - **Port-failure diagnostics.** [`QuotaUsecaseError::UnknownTenant`] and
//!   [`QuotaUsecaseError::PersistenceUnavailable`] are the scaffold's
//!   published unit variants and carry no tenant id or port identity, so an
//!   operator cannot tell the tenant read model from the override table when
//!   one is down. Enriching them is a breaking change to that contract and is
//!   deliberately held for the wave that introduces the real (async,
//!   network-backed) adapters, where the port identities actually exist.
//! - **Quota classes are strings, not a closed enum.** `plan_tier` is closed
//!   in the OpenAPI contract but open here, so an undeclared class is
//!   [`QuotaUsecaseError::NoPolicyForClass`] at resolve time rather than
//!   unrepresentable at compile time. Packs are open by the same choice, but
//!   are validated against the catalog's declared set.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{
    QuotaLedger, QuotaOutcome, QuotaPolicyCatalog, US_HC_PACK, resolve_from_policy,
    soft_threshold_of,
};
pub use kernel::{
    DEFAULT_SOFT_THRESHOLD_PERCENT, QuotaAllowance, QuotaDecision, QuotaKey,
    QuotaOverrideRepository, QuotaResource, QuotaSource, QuotaUsageError, QuotaUsecaseError,
    ResetWindow, TenantClassReader,
};
pub use usecase::{open_ledger, resolve, resolve_effective_quota, resolve_quota_sheet};
