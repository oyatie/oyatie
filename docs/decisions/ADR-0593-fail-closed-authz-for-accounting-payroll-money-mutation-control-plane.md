---
id: ADR-0593
title: "Fail-closed authz for the Accounting + Payroll money-mutation control planes (AUTH-005 / Wave-2b money-CRIT remediation)"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0090, ADR-0092, ADR-0094, ADR-0131, ADR-0559, ADR-0561, ADR-0566, ADR-0572, ADR-0573]
amends: []
related: [ADR-0559, ADR-0561, ADR-0566, ADR-0572, ADR-0573, ADR-0581]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0593: Fail-closed authz for the Accounting + Payroll money-mutation control planes (AUTH-005 / Wave-2b money-CRIT remediation)

## Status

**Proposed - 2026-06-23 (door: one-way).**

## Context

Two HTTP runtime adapters bind the most money-sensitive surfaces in the ERP product
verticals to the repo-native Hyper router/middleware foundation:

- `billing/adapters/accounting-http` (`billing-accounting-http-adapter`) — the accounting
  journal control plane: `POST /accounting/v1/journals`, `/payroll-postings`,
  `/vat-workflow-plans`. (Migrated from `oya/accounting/crates/oya-accounting-journal-infrastructure`
  by the capability-first reorg; the cited path is its current home.)
- `oya/payroll/crates/oya-payroll-run-infrastructure` (`oya-payroll-run-infrastructure`) — the
  payroll run control plane: `POST /payroll/v1/trial-closes`, `/accounting-journal-drafts`,
  `/hr-leave-impact-intakes`.

Both crates dispatched every request through an **empty** middleware chain:

```rust
pub fn accounting_runtime_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}
pub fn payroll_runtime_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}
```

`dispatch_{accounting,payroll}_request` therefore ran the money-mutating handlers with **no
verified identity and no policy decision**. The handlers then trusted the caller-supplied
`tenant_id` carried in the request **body** (`JournalPostRequest.tenant_id`,
`PayrollTrialCloseRequest.tenant_id`, …) to scope the mutation. Any caller who reaches the
socket posts a journal, payroll trial-close, or VAT workflow against **any tenant they name**.
This is the AUTH-005 unauthenticated / caller-supplied-authz class (the whole-repo review
"forgeable caller-supplied authz at the trust boundary" finding) realized on **money surfaces**
— the Wave-2b money CRITICAL for `oya-accounting-payroll`.

## Decision

Wire a **fail-closed, verified-principal + cloud-iam-PDP** authz seam onto the money-mutation
routes of both crates, mirroring the proven doctrine that landed for the Cloud KMS crypto
control plane (ADR-0573), the Cedar policy publish control plane (ADR-0572),
`intelligence/adapters/rest` (`constant_time_eq` bearer compare + a PDP `decide` port), the
cloud-iam PDP caller-authn precedent (ADR-0561 / #38), and the authz-coverage doctrine
(ADR-0566). Each crate gains an in-crate `src/authz.rs` PORTS module (clean architecture per
ADR-0131 — the concrete cloud-iam Cedar PDP client + bearer/SVID credential store are the
owned-W5 ADAPTERS, kept out of these boundary crates):

1. **Unforgeable verified principal.** `VerifiedPrincipal` has **private** fields, a
   `pub(crate)` constructor, public accessors, and a `#[cfg(test)]` test constructor. External
   crates cannot build one by struct literal; they MUST run a real `PrincipalVerifier`. The
   caller-supplied body `tenant_id` is NEVER the source of truth for identity.

2. **Constant-time, boot-refusing reference verifier.** `ConfiguredBearerPrincipalVerifier`
   verifies a bearer token via `constant_time_eq` (never `==`), binds `(principal_id, tenant_id)`
   from configuration (never the caller's claimed fields), and REFUSES construction on an empty
   secret or empty bound identity (boot-refusal). It is documented BREAK-GLASS-ONLY; the
   production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561). `Authorization`
   is a `SECRET` field with a custom redacting `Debug`.

3. **AUTHN before BODY.** A `MiddlewareChain`-installed `*AuthzMiddleware` runs on every
   money-mutation route BEFORE the terminal handler deserializes the body. It reads the
   `Authorization` header from the request `Parts`, verifies the principal (missing/invalid ->
   401), and short-circuits. Body size is already capped at 64 KiB by `ServerConfig`
   (`with_max_body_bytes`, the DefaultBodyLimit equivalent) at the hyper boundary, so no
   unbounded body is parsed before authn. Non-mutation routes (health) pass through
   unauthenticated.

4. **PDP decision bound to the TRUSTED tenant (true blast radius).** After authn, the
   middleware runs a `{Accounting,MoneyMutation}Authorizer` PDP `ensure_authorized` decision
   whose `resource.tenant_id` is bound to the **verified principal's** tenant — never flattened
   to the caller's body input. It injects the verified tenant into `path_captures`; the handler
   then REJECTS (403) any body whose `tenant_id` does not equal the verified tenant
   (constant-time compared). A cross-tenant body substitution (verified tenant A, body claims
   tenant B) is denied at BOTH the PDP layer and the handler cross-check.

5. **PDP-fault-denies.** The authorizer port contract maps every fault (timeout, network,
   unavailability) to `Err(Refused)` -> 403. There is no default-allow fallback and no `Default`
   impl on the provider; the chain builder and `dispatch_*` REQUIRE a provider, so a money
   mutation can never be served without both ports running. The provider's `catch_unwind` is a
   documented test/debug-only backstop (release is `panic = "abort"`); the real guarantee is the
   adapter contract.

## Consequences

- `accounting_runtime_chain` / `payroll_runtime_chain` now REQUIRE an authz provider argument;
  `dispatch_{accounting,payroll}_request` take a `{Accounting,Payroll}AuthzProvider`. The
  composition root must supply the verifier + PDP authorizer (no zero-arg default).
- Unauthenticated money mutation -> 401; cross-tenant / PDP-deny / PDP-fault -> 403; health stays
  open. These are enforced structurally by the required-provider signatures and proven by RED
  tests.
- No new workspace dependencies: the ports/middleware are self-contained on the existing
  `oya-http-middleware-kernel`, so `cargo metadata --locked` carries no `Cargo.lock` delta.
- Born-accounting: each crate gains a `src/authz.rs` source and an `*-unittest` `rust_test`
  target (the lib previously had no unit-test runner, so the in-lib `#[cfg(test)]` authz tests
  would otherwise not execute). These rows are justified: a new security module + the test target
  that runs its boot-refusal / constant-time / panic->Refused / config-bound-identity unit tests.
  The two new source modules this decision introduces and justifies are
  `billing/adapters/accounting-http/src/authz.rs` and
  `oya/payroll/crates/oya-payroll-run-infrastructure/src/authz.rs`; both are the fail-closed
  money-mutation authz ports mandated above and are owned by their nearest-ancestor `OWNERS`
  boundary (`billing/OWNERS` and the crate-scoped
  `oya/payroll/crates/oya-payroll-run-infrastructure/OWNERS` respectively).

## RED / GREEN tests

`tests/runtime.rs` (both crates):

- unauthenticated money mutation -> **401** (RED: empty chain returned 202/403)
- wrong/absent bearer -> **401**
- cross-tenant body substitution (verified ten_acme, body ten_victim) -> **403** (blast-radius)
- PDP deny -> **403**
- PDP fault (panicking authorizer) -> **403**, never 500, never allow
- authenticated + authorized + same-tenant -> **202** (GREEN)

`src/authz.rs` unit tests (both crates, via the new `*-unittest` target): boot-refusal on empty
secret/identity, config-bound identity (never caller-claimed), constant-time compare,
missing/wrong/non-Bearer credential rejection, panic->Refused wrapper, canonical action surfaces,
and route-template->action mapping.

## Alternatives considered

- **A shared `oya-http-authz-middleware` crate.** Deferred: the verifier/PDP port shapes are
  still stabilizing per-surface (KMS, Cedar-publish, payroll/accounting). Extracting now would
  freeze a contract before the W5 cloud-iam adapter lands. The in-crate ports mirror ADR-0573;
  consolidation is a follow-up once ≥3 surfaces share the identical port.
- **Parsing the body tenant in the middleware.** Rejected: that would deserialize the body
  before authn completes (wrong order) and couple the middleware to every DTO. Binding the PDP
  resource to the verified tenant + a handler cross-check keeps authn strictly before body
  deserialization.
