//! Reserved-namespace guard — implementation of
//! `tenancy/IP-017-reserved-namespace-enforcer.md`.
//!
//! Tenant creation, tenant rename and sub-scope creation all mint a
//! human-visible name. This crate is the gate they call before persisting
//! one: it refuses names that ARE a platform-owned namespace, names that
//! merely look like one, names that are not legal labels at all, and names
//! the calling principal has no grant to claim.
//!
//! Per ADR-0242 (`feedback_oyatie_is_a_tenant_doctrine`) the platform owner
//! is a tenant like any other, so the owner's own name is reserved and
//! there is NO carve-out anywhere in this crate that lets it through. Per
//! ADR-0284 the owner name is never written down here: it arrives through
//! [`ReservedNamespaceSource`], which real deployments back with
//! `/specs/platform-owner-binding.json`.
//!
//! # Layout
//!
//! IP-017 specifies a `-usecase` crate with `normalization`,
//! `reserved_set` and `enforce` modules. The tenancy capability is capped
//! at twelve crates and this lane's `Cargo.lock` is frozen, so the plan's
//! shape is collapsed into one crate as a module tree with the same
//! layering discipline:
//!
//! - [`kernel`] — entities ([`NamespaceCandidate`], [`NamespaceAction`],
//!   [`NamespaceDecision`]), the two ports
//!   ([`ReservedNamespaceSource`], [`NamespaceActionAuthorizer`]) and the
//!   single error type [`NamespaceUsecaseError`]. No logic.
//! - [`domain`] — pure decisions: [`validate_syntax`], [`normalize`],
//!   [`skeleton`], [`boundary_prefixes`] and [`ReservedSet`] matching. No
//!   I/O, no clock, no randomness.
//! - [`usecase`] — [`evaluate`] and [`evaluate_detailed`], which sequence
//!   the ports around those decisions.
//! - [`inmemory`] — reference port implementations and the test fixtures.
//!
//! # The decision ladder
//!
//! Stages run in a fixed order — request well-formedness, syntax,
//! reservation, confusability, authorization — and each produces a distinct
//! [`NamespaceDecision`]. Syntax runs before the reservation list is read,
//! so a malformed name costs no port call and its refusal does not depend
//! on policy state. Authorization runs last, so a name nobody may claim
//! reads as refused for the NAME rather than for the caller.
//!
//! # What "reserved" means
//!
//! A candidate hits the list when ANY of its [`boundary_prefixes`] equals a
//! reserved token. Separators are stripped before the prefix is taken and a
//! prefix only counts where a separator stood, so `oyatie-support`,
//! `o-yatie-support` and `oyati-e-support` all collide with a reservation on
//! `oyatie`, while `oyatier-customer` — a different root that merely starts
//! alike — does not. Matching only the first separator-delimited segment is
//! not enough: one separator placed inside the owner token defeats it.
//!
//! # Tenant scoping
//!
//! [`NamespaceCandidate`] carries IP-017 §D.4's tenant context and audit
//! correlation id, and the whole candidate is what
//! [`NamespaceActionAuthorizer`] receives, so a policy adapter can express a
//! rule about the RESOURCE and not only about the principal. For a rename or
//! a sub-scope alias the tenant context is mandatory
//! ([`NamespaceUsecaseError::TenantContextMissing`]); otherwise a principal
//! holding a grant in one tenant could mint a name in another simply by
//! omitting it.
//!
//! # An outage is not a denial
//!
//! Every port failure becomes a [`NamespaceUsecaseError`] carrying the
//! adapter's own cause, and never a `Deny`. Failing closed is the caller's job and the typed error says to;
//! what the guard will not do is let an operator read "your name is
//! reserved" when the truth is "the reservation source was unreachable",
//! because those two have opposite remediations. For the same reason an
//! empty reservation list is [`NamespaceUsecaseError::EmptyReservationList`]
//! rather than "nothing is reserved": a correctly resolved owner binding
//! always yields at least the owner token, so an empty list means the
//! binding did not resolve.
//!
//! # Determinism
//!
//! Nothing below the port boundary reads a clock or draws randomness. Two
//! processes holding the same reservation list and the same candidate
//! produce the identical decision, skeleton and digest, so a refusal can be
//! replayed from its audit record.
//!
//! # Gaps
//!
//! Deliberately deferred, and named here rather than hidden:
//!
//! - **No Unicode confusable handling at all.** IP-017 §D.3 calls for
//!   Unicode confusable skeletons — Cyrillic `а` vs Latin `a`, Greek `ο`,
//!   full-width forms, and the rest of UTS #39. That needs a confusables
//!   table this lane cannot take a dependency on, and none is vendored
//!   here. [`skeleton`] is an ASCII-only fold over a fixed, published
//!   substitution table (`0`→`o`, `1`/`i`→`l`, `5`→`s`, `rn`→`m`, `vv`→`w`,
//!   `cl`→`d`, and the rest documented on the function). What that means
//!   concretely: a Cyrillic homograph is refused today ONLY because
//!   [`validate_syntax`] rejects every non-ASCII byte outright, and there
//!   is nothing behind that rule. Any caller that relaxes the charset — an
//!   IDN-accepting alias, a display-name check, a future non-ASCII slug
//!   policy — gets ZERO homograph protection from this crate. Do not read
//!   [`NamespaceDecision::DenyConfusable`] as "Unicode-safe".
//! - **The skeleton is lossy in both directions.** It collapses names that
//!   are not attacks (`oyatle` folds onto `oyatie`) and misses attacks it
//!   has no rule for (repeated characters are not collapsed, so `ooyatie`
//!   passes). The bias is deliberate: over-refusing a slug costs a tenant
//!   one rename, and the alternative costs a phishing incident.
//! - **blake3 → FNV-1a.** IP-017 §D.5 emits a candidate hash;
//!   [`fnv1a_64`] stands in for a cryptographic digest. FNV is not
//!   preimage-resistant and slugs are short, so
//!   [`NamespaceEvaluation::candidate_digest`] is a correlation key that
//!   hides the candidate from a casual reader of a log and from nobody
//!   else. It must not be treated as anonymization.
//! - **`/specs/platform-owner-binding.json` is not read here.** That
//!   adapter needs a JSON parser. [`ReservedNamespaceSource`] is the seam
//!   it plugs into and
//!   [`inmemory::InMemoryReservedNamespaceSource::for_owner`] builds the
//!   list §D.2 specifies from an owner name, so the adapter's only job is
//!   to parse the file and hand over the strings.
//! - **Cedar is not evaluated here.** `tenancy/policy/action-authorization.cedar`
//!   needs the `cedar-policy` crate. [`NamespaceActionAuthorizer`] is the
//!   seam; [`inmemory::InMemoryNamespaceActionAuthorizer`] is a grant-set
//!   stand-in and is NOT a policy engine.
//! - **No event is emitted.** §D.5's
//!   `oya.tenancy.reserved-namespace-create-refused` needs a facade
//!   dependency. [`NamespaceEvaluation`] carries the whole payload and
//!   [`NamespaceEvaluation::refusal_event`] names the topic; publishing it
//!   belongs to the caller.
//! - **Ports are synchronous.** Async would require an executor
//!   dependency. The trait shapes are chosen so an async adapter can wrap
//!   them without changing any decision in [`domain`].
//! - **Nothing wires this guard into tenant creation.** IP-017 §D.7 makes
//!   it a pre-persistence check in IP-004 and IP-016; those crates are
//!   other lanes' files and are untouched here. Until that wiring lands,
//!   this crate can decide and cannot enforce.
//! - **Tenant creation may omit its tenant context.** A rename and a
//!   sub-scope alias both name a tenant that already exists, so both refuse
//!   to evaluate without one. Tenant creation has no such resource yet, so
//!   [`NamespaceCandidate::tenant`] stays optional there and a Cedar
//!   adapter gating [`NamespaceAction::CreateTenant`] sees `None` unless the
//!   caller supplies the requesting organization. Nothing in this crate
//!   forces it to.
//! - **The correlation id is echoed, never minted.** Generating one needs
//!   randomness, which the determinism rule forbids below the port
//!   boundary, so a caller that supplies none gets an evaluation with
//!   `correlation_id: None` and an event that cannot be joined to its
//!   request.
//! - **The reserved list is not versioned.** Adding an entry can retro-
//!   actively make an existing tenant's slug reserved; this crate has no
//!   notion of grandfathering and every caller re-evaluates against the
//!   current list.
//!
//! ADR-0083 Tier-3: production code in this crate carries no
//! `unwrap`/`expect`/`panic`, and no `as` cast that can truncate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{
    FNV_OFFSET_BASIS_64, FNV_PRIME_64, MalformedReason, ReservedSet, boundary_prefixes,
    candidate_root, fnv1a_64, fold_character, fold_digraph, is_boundary, is_permitted,
    is_separator, normalize, reserved_root, skeleton, validate_syntax,
};
pub use inmemory::{InMemoryNamespaceActionAuthorizer, InMemoryReservedNamespaceSource};
pub use kernel::{
    MAX_LABEL_LEN, NamespaceAction, NamespaceActionAuthorizer, NamespaceCandidate,
    NamespaceDecision, NamespaceUsecaseError, ReservedNamespaceSource,
};
pub use usecase::{NamespaceEvaluation, evaluate, evaluate_detailed};
