---
id: ADR-0582
title: "DTO-authz-trust gate (caller-supplied authorization decision backstop)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amends: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0540, ADR-0544, ADR-0547, ADR-0548, ADR-0566, ADR-0572]
related: [ADR-0506, ADR-0561, ADR-0564, ADR-0566, ADR-0570]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0582: DTO-authz-trust gate (caller-supplied authorization decision backstop)

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

Ship `ci/facade/caller-supplied-authorization`, a born-blocking cloud-ci gate that
makes a NEW instance of the caller-supplied-authz-trust antipattern IMPOSSIBLE to ship while
frozen-baselining the existing debt (shrink-only). It is a SIBLING of the authz-coverage gate
(ADR-0566) and registers in the same `oya-ci-required` matrix gate family. It mirrors the
kernel-purity (ADR-0547) / port-placement (ADR-0570) / authz-coverage (ADR-0566) born-accounting
shape: a neutral pure-Rust engine + policy-as-data, hermetic over the tracked source tree, ratcheted
against a frozen baseline.

### D1 — The antipattern, mechanically

A function is a DTO-AUTHZ-TRUST instance iff BOTH hold over its CODE-ONLY body (comments and
string/char literal CONTENT elided via a length-preserving mask, so a doc-comment mention never
triggers). This is the **v2 two-signal heuristic** (v1 had a third self-compare precondition that
was evadable by `Vec::contains` / `binary_search` / `is_superset`; v2 inverts it — FN-06):

- **(a) it reads a CALLER-SUPPLIED authorization-decision field** — it takes a parameter whose type
  name tail ends with the policy `authorization_dto_type_suffixes` (default `Authorization` — the
  forged blob), OR its body reads a `trigger_decision_field_idents` member (`allowed_surfaces`,
  `permitted_scopes`, `caller_roles`, `granted`, `allowed_actions` — authz allow-lists/grants with
  no benign business meaning) off a binding, OR it reads an `authorization_header_idents`
  (`x-authorization-decision-id`, `x-authorization-surfaces`, `x-authorization-principal-id`,
  `x-authorization-tenant-id`) — detected in a comment-stripped string-preserving body view so
  header names in string literals are found but header names only in comments are not (FP-01);
- **(b) the body makes NO whole-token CALL to a `pdp_decision_idents` port** (`.decide(`,
  `ensure_authorized`, `check_authz`, `ensure_authz`, `authorize_decision`, `pdp_decide`).

**`verify_principal` is intentionally ABSENT from `pdp_decision_idents`** — it is an
AUTHENTICATION step (verifies identity from an unforgeable credential), not an authorization
decision. A function that calls `verify_principal` but self-compares the authz DTO is still flagged.
The authorization decision requires a server-side PDP call (`ensure_authorized` / `.decide()`).

Self-compare tokens (`==`, `!=`, `.iter().any(`, `.contains(`) are retained in policy as
description-enrichment signals only — they annotate the finding narrative but are not a gate
precondition (FN-06 inversion).

Header detection runs on a comment-stripped but string-preserving view of the body (not the
fully-masked body and not the raw original text), so header names in string literals trigger while
header names only in comments do not (FP-01 fix).

`#[cfg(not(test))]`-gated items are PRODUCTION code and are scanned. Only `#[cfg(test)]` positive
predicates mark test-fixture blocks that are excluded (FN-05 fix).

### D2 — Conservative in the SAFE direction + honest limits

For (b), a PDP-port call recognized as a whole-token CALL in the code-only body marks the function
GREEN — the gate never invents a false finding for a function that genuinely delegates to a PDP. A
function that BOTH reads an authorization DTO AND calls a PDP port is GREEN: the PDP call is the
real decision, the DTO read is a redundant precondition.

**v2.2 split-decision allowlist (ADR-0591 split-decision) — the EXPLICIT, non-launderable FP
mechanism that REPLACES the removed name-reachability heuristic:** the AUTH-005 remediations landed a
benign mirror of the opaque-helper limit — a use-case splits its authorization into (i) the
authoritative server-side PDP `ensure_authorized`/`decide` and (ii) a residual NON-AUTHORITATIVE
correlation-consistency cross-check (a `*_correlation` function that reads the authorization DTO but
"NEVER grants"), plus a third FP shape: a no-verdict `*_fingerprint_for` SERIALIZER that folds the
authorization fields into an idempotency-dedup fingerprint string (returns a struct, not a `Result`;
makes no decision at all). The per-function detector flags all three because the authoritative PDP
call lives elsewhere / there is no verdict.

An earlier attempt (`recognize_sibling_pdp_delegation`, a policy-gated heuristic that marked a flagged
function GREEN iff it was transitively reachable via the same-file call graph from a PDP-calling
function) was **REMOVED as UNSOUND.** A hostile review proved empirically (via buck2 probes) that it
suppressed by function-NAME reachability rather than by proving a PDP decision dominates the guarded
operation, so three distinct bypasses laundered a genuinely-forgeable check to GREEN: (A) a dead-code
`if false { x.ensure_authorized(); }` "PDP root" with a call edge to the forgeable fn; (C) a
same-named overload in a DIFFERENT impl block where name-set membership laundered the unrelated
forgeable overload; and (D) a single-file bypass where a public `entry` made ITSELF a PDP root via a
dead `ensure_authorized` and called a forgeable `decide_access` — yielding `flagged=[] verdict=Green`
while a forgeable check shipped. All three are now RED regression tests.

The replacement is an **EXPLICIT, CURATED `split_decision_allowlist`** in policy DATA: a tiny
hand-audited list, NOT a heuristic. Each entry is the SAME exact-key shape as the baseline
(`<file>#<fn>:<body_hash>`), so any body change re-keys and re-flags it (`DAT-STALE-SPLIT-DECISION-ALLOWLIST`
self-clean). It suppresses ONLY the exact audited body — it cannot suppress a different function that
merely shares a name, nor a body edited after the audit. Each entry was hand-verified by tracing the
call graph that an authoritative server-side PDP dominates every path to the function and that the
function itself never grants. The five cleared entries:
`audit/core/usecase` `validate_authorization` (PDP at `emit_audit_event_authorized`),
`billing/ports/finops-api` `validate_authorization_correlation` (PDP at
`generate_cloud_finops_report_from_api`), `observability/core/api`
`cross_check_authorization_correlation` (PDP at `read_cloud_observability_audit_from_api`), and the
two no-verdict fingerprint serializers `audit/core/usecase` `audit_event_emit_fingerprint_for` +
`billing/ports/finops-api` `finops_report_fingerprint_for`. This cleared the FPs the rebase onto the
AUTH-005 tip surfaced WITHOUT growing the baseline (no `--allow-new`) and WITHOUT any name-reachability
suppression.

Honest LIMITS (documented, not hidden):

- **Dead-code evasion**: `if false { .decide() }` suppresses signal (b). The gate does not perform
  reachability analysis — a PDP ident in the code-only body marks the function GREEN regardless of
  control flow. Code reviewers must catch intentional `if false { decide() }` suppressions. (This is
  why the name-reachability heuristic was unsound: it COMPOUNDED this limit into a launderable
  call-graph suppression. The explicit exact-key allowlist does not.)
- **Wrong-receiver evasion**: `other_svc.ensure_authorized()` on a different receiver also suppresses
  signal (b). Policy-keyed idents should be scoped to the owned PDP port shape.
- (a) is the load-bearing precision lever: requiring an *authorization-DTO* / *authz-specific
  allow-list* signal (NOT just any `tenant_id` comparison) is what keeps benign tenant/path-binding
  validators GREEN. An overloaded field (`decision_id`, which also names retention/policy business
  keys) is a description-only corroborator, never a standalone trigger — so the gate does not
  false-positive on retention/policy `decision_id` integrity checks.
- Opaque-helper evasion: a handler that hides the same authz-decision behind a helper giving NO
  authorization-DTO signal is outside the envelope (human review + the authz-coverage gate own that).
- Split-decision allowlist scope: the allowlist suppresses ONLY exact audited bodies. It cannot be
  used to launder a forgeable check (a body edit re-keys it); a curator who allowlists a body that
  DOES decide authorization is making an auditable, reviewable claim in policy DATA, not hiding behind
  a heuristic.

### D3 — Born-blocking with a FROZEN, SHRINK-ONLY baseline (v2: body-hash keying)

The pre-existing instances are enumerated and FROZEN as known debt
(`frozen_dto_authz_trust_instances`, keyed by stable `<file>#<fn>:<body_hash>` — a 32-hex-char
FNV-1a hash of the code-only masked function body is appended to the file+fn components). The
baseline is **73 keys** on the rebased dev tip: it was re-anchored by a SHRINK-ONLY `--write` (no
`--allow-new`, no firewall-door sign-off) after the rebase onto the AUTH-005 W2/W2b integration tip
dropped the ~18 instances those PRs remediated; the 5 genuine false positives (3 split-decision
correlation checks + 2 no-verdict fingerprint serializers) are cleared by the EXPLICIT exact-key
`split_decision_allowlist` (NOT baselined, NOT laundered by a heuristic). This
prevents a NEW function with the SAME name in a different `mod` (or a refactored body) from
auto-laundering as a baselined instance (FN-02). A line shift with no body change does NOT change
the key. An instance whose key is in the baseline is ACCEPTED (no block) — each owner's remediation
over time. A NEW instance whose key is NOT in the baseline → RED. The baseline is shrink-only by
construction (a fixed/removed instance drops its key); a stale key self-cleans via
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
`ci/facade/caller-supplied-authorization/src/lib.rs` (the I/O collector
`collect_instances(root, policy)` + the pure `evaluate_keyed(policy, observed)`), the
`--write`-capable binary in `ci/facade/caller-supplied-authorization/src/main.rs`, the
live-corpus + RED/GREEN self-test in
`ci/facade/caller-supplied-authorization/tests/dto_authz_trust.rs`, its buck2 wiring in
`ci/facade/caller-supplied-authorization/BUCK`, its manifest in
`ci/facade/caller-supplied-authorization/Cargo.toml`, its ownership in
`ci/facade/caller-supplied-authorization/OWNERS`, and ALL repo-specifics as DATA in
`ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json`.

## Consequences

- A NEW use-case / handler that trusts a forged authorization DTO / header in place of a server-side
  PDP decision is blocked at presubmit (`DAT-CALLER-SUPPLIED-AUTHZ-TRUST`), with a remediation
  pointer to the fixed `iam/ports/policy-cedar-api/src/authz.rs` doctrine.
- The ~52 existing instances become tracked, shrink-only debt; each capability team remediates by
  deriving the principal from a verified mTLS/SVID/bearer credential and calling the cloud-iam Cedar
  PDP server-side to `decide(principal, action, resource)`, failing closed.
- The gate runs in the single `oya-ci-required` fan-in (matrix `gate` lane), so it gates merge.

## Deferred — baseline merge-base anchoring (INTEGRITY-01)

The `frozen_dto_authz_trust_instances` baseline carried inline in
`dto-authz-trust-policy.json` is, like every other gate's frozen baseline, an in-tree
reference rather than a value materialized from `git merge-base <base_ref> HEAD`. The
gate-baseline PR/push asymmetry (a frozen-in-tree reference can be edited within the same
PR that adds a new instance) is the INTEGRITY-01 residual. It is **intentionally deferred
here**: anchoring this gate's baseline to the merge-base is a single instance of a
**fleet-wide class** that must be solved once, uniformly, for ALL gate baselines by the
merge-base-anchor meta-enforcement program (the same program that owns the
`gate-baseline.generated.json` merge-candidate materialization via the scm-facts emitter).
This PR does not block on it; the `dto-authz-trust-policy.json` baseline anchoring is
**tracked there** and will be migrated in lockstep with the other gates rather than as a
bespoke one-off here. Until then the binary-level shrink-only `--write` (growing requires
the explicit `--allow-new` reviewed grandfather) is the enforcement floor.

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
