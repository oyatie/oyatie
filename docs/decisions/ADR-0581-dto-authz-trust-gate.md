---
id: ADR-0581
title: "DTO-authz-trust gate (caller-supplied authorization decision backstop)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0540, ADR-0544, ADR-0547, ADR-0548, ADR-0566, ADR-0572]
related: [ADR-0506, ADR-0561, ADR-0564, ADR-0566, ADR-0570]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0581: DTO-authz-trust gate (caller-supplied authorization decision backstop)

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; door: one-way).**

## Context

A whole-repo security review found a #1 systemic antipattern at 30+ trust boundaries:
**caller-supplied authorization trusted as the authz decision.** A request handler / use-case reads
an *authorization decision* FROM the request itself — a `{decision_id, tenant_id, principal_id,
allowed_surfaces}` blob in the request DTO (an `*Authorization` struct), or `x-authorization-*`
headers — and "validates" it by string-comparing those fields against the SAME request (presence /
equality / surface-membership checks), **without ever calling the cloud-iam Cedar PDP server-side.**
This is **default-ALLOW-on-forged-input**: any caller forges the blob and authorizes itself.

Representative confirmed instances: `secrets/ports/kms-api` `validate_authorization`,
`tenancy/ports/api`, `network/ports/{lb,vpc,dns}`, `audit/core/usecase`,
`compliance/ports/dsr-usecase`, `observability/core/api`, `billing/ports/finops-api`, and ~45 more
across `compute/facade/*`, `storage/ports/*`, `data/*`, `cell/ports/*`, `iam/*`,
`oya/application/*`, `oya/intelligence/*`, and `workflow/ports/*`.

This PASSES the existing unauthenticated-surface gate (ADR-0566 / issue #780,
`oya-cloud-ci-authz-coverage-app`) because it HAS guard-looking code — a `validate_authorization`
function exists and is called. The authz-coverage gate verifies that an HTTP control-plane surface
*invokes a guard*; it does not verify that the guard *consults a server-side PDP* rather than
trusting the caller's own claim.

Per founder doctrine — friction is a process failure; productize the engine so the antipattern is
*impossible* to ship (ADR-0548 pipeline-as-product) — the pipeline itself must catch this CLASS in
the gate, not the instance. The repo already carries the correct fixed pattern: the IAM keystone
instances at `iam/ports/policy-cedar-api/src/authz.rs` (task #124 / ADR-0572, PRs #815/#816) verify a
principal from an **unforgeable credential** (`PrincipalVerifier::verify_principal(credential) ->
VerifiedPrincipal`) and call a **server-side PDP decision port** (`PublishAuthorizer::ensure_authorized`
/ a PDP `decide(...)`), failing closed (any refusal = deny). Nothing mechanically enforced that
**new** use-cases follow it.

## Decision

Ship `cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app`, a born-blocking cloud-ci gate that
makes a NEW instance of the caller-supplied-authz-trust antipattern IMPOSSIBLE to ship while
frozen-baselining the existing debt (shrink-only). It is a SIBLING of the authz-coverage gate
(ADR-0566) and registers in the same `oya-ci-required` matrix gate family. It mirrors the
kernel-purity (ADR-0547) / port-placement (ADR-0570) / authz-coverage (ADR-0566) born-accounting
shape: a neutral pure-Rust engine + policy-as-data, hermetic over the tracked source tree, ratcheted
against a frozen baseline.

### D1 — The antipattern, mechanically

A function is a DTO-AUTHZ-TRUST instance iff ALL THREE hold over its CODE-ONLY body (comments and
string/char literal CONTENT elided via a length-preserving mask, so a doc-comment mention never
triggers):

- **(a) it reads a CALLER-SUPPLIED authorization-decision field** — it takes a parameter whose type
  name tail ends with the policy `authorization_dto_type_suffixes` (default `Authorization` — the
  forged blob), OR its body reads a `trigger_decision_field_idents` member (default `allowed_surfaces`
  — an authz allow-list with no benign business meaning) off a binding, OR it reads an
  `authorization_header_idents` (`x-authorization-*`);
- **(b) the only "check" is self-comparison / equality / membership against the request** — the body
  contains a `self_compare_tokens` operator (`==` / `!=` / `.iter().any(` / `.contains(`);
- **(c) the body makes NO whole-token CALL to a `pdp_decision_idents` port** (`.decide(`,
  `ensure_authorized`, `verify_principal`, `check_authz`, `ensure_authz`).

### D2 — Conservative in the SAFE direction + honest limits

For (c), a PDP-port call recognized as a whole-token CALL in the code-only body marks the function
GREEN — the gate never invents a false finding for a function that genuinely delegates to a PDP. A
function that BOTH self-compares an authorization DTO AND calls a PDP port is GREEN: the PDP call is
the real decision, the self-comparison is a redundant precondition.

Honest LIMITS (documented, not hidden):

- The gate recognizes the dominant corpus shape: a synchronous `fn` with an `*Authorization` DTO
  param (or an `allowed_surfaces` read) that self-compares. A handler that hides the same
  self-comparison behind an opaque helper giving NO authorization-DTO signal is outside the envelope
  (human review + the authz-coverage gate own that). This deliberate boundary keeps the baseline
  meaningful and the false-positive rate near zero.
- (a) is the load-bearing precision lever: requiring an *authorization-DTO* / *authz-specific
  allow-list* signal (NOT just any `tenant_id` comparison) is what keeps benign tenant/path-binding
  validators GREEN. An overloaded field (`decision_id`, which also names retention/policy business
  keys) is a description-only corroborator, never a standalone trigger — so the gate does not
  false-positive on retention/policy `decision_id` integrity checks.

### D3 — Born-blocking with a FROZEN, SHRINK-ONLY baseline

The ~52 pre-existing instances are enumerated and FROZEN as known debt
(`frozen_dto_authz_trust_instances`, keyed by stable `<file>#<fn>` so a line shift never re-REDs a
baselined instance). An instance whose key is in the baseline is ACCEPTED (no block) — each owner's
remediation over time. A NEW instance whose key is NOT in the baseline → RED. The baseline is
shrink-only by construction (a fixed/removed instance drops its key); a stale key self-cleans via
`DAT-STALE-BASELINE`. AUTOMATED: re-baselining is mechanical via the gate binary `--write`
(shrink-only; growing requires the explicit `--allow-new` flag, a reviewed grandfather).

### D4 — Hermetic, fail-closed, pack-shaped

The crate is a NEUTRAL engine: every repo-specific (the DTO type suffix, the trigger / description
decision fields, the authorization-header idents, the PDP decision-port idents, the scan
roots/excludes, the frozen baseline, the liveness floor) is DATA in `dto-authz-trust-policy.json`. A
different repo adopts the gate by repointing the policy. The only I/O is a read-only walk of the
candidate tree's `.rs` source (NO shell / network / VCS / clock / rand). Malformed policy
(`DAT-POLICY-MALFORMED`), gate-id mismatch (`DAT-POLICY-GATE-ID-MISMATCH`), and a below-floor
function census (`DAT-EMPTY-SCAN`) all fail closed against a silent false-green. ADR-0083 Tier-3:
production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.

The gate's neutral Rust engine + I/O collector lives in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/src/lib.rs` (the I/O collector
`collect_instances(root, policy)` + the pure `evaluate_keyed(policy, observed)`), the
`--write`-capable binary in `cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/src/main.rs`, the
live-corpus + RED/GREEN self-test in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/tests/dto_authz_trust.rs`, its buck2 wiring in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/BUCK`, its manifest in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/Cargo.toml`, its ownership in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/OWNERS`, and ALL repo-specifics as DATA in
`cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/dto-authz-trust-policy.json`.

## Consequences

- A NEW use-case / handler that trusts a forged authorization DTO / header in place of a server-side
  PDP decision is blocked at presubmit (`DAT-CALLER-SUPPLIED-AUTHZ-TRUST`), with a remediation
  pointer to the fixed `iam/ports/policy-cedar-api/src/authz.rs` doctrine.
- The ~52 existing instances become tracked, shrink-only debt; each capability team remediates by
  deriving the principal from a verified mTLS/SVID/bearer credential and calling the cloud-iam Cedar
  PDP server-side to `decide(principal, action, resource)`, failing closed.
- The gate runs in the single `oya-ci-required` fan-in (matrix `gate` lane), so it gates merge.

## Alternatives considered

- **Extend the authz-coverage gate (ADR-0566).** Rejected: that gate anchors on HTTP route-surface
  discovery (router builder chains); this antipattern is function-level use-case authz validation
  with no router. A sibling gate keeps each engine's anchoring precise and its baseline meaningful.
- **Flag any `tenant_id` equality check.** Rejected: floods false positives on benign tenant/path
  binding validators that assert no authorization verdict. Requiring an authorization-DTO /
  authz-specific allow-list signal is the precision lever.
- **Block the existing instances immediately (no baseline).** Rejected: remediating ~52 surfaces is
  each owner's work; born-green + enforce-no-regression mirrors the established gate posture
  (capability-membership, tier-acyclicity, authz-coverage, port-placement) and ships the backstop now.
