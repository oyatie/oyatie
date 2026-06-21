---
id: ADR-0566
title: "Authz-coverage gate (unauthenticated HTTP control-plane backstop)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-21
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0540, ADR-0544, ADR-0547, ADR-0548, ADR-0559]
amends: []
related: [ADR-0547, ADR-0506, ADR-0540, ADR-0544, ADR-0559, ADR-0561, ADR-0564]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0566: Authz-coverage gate (unauthenticated HTTP control-plane backstop)

## Status

**Proposed - 2026-06-21 (authored for founder sign-off; door: one-way).**

## Context

PR #768 (ADR-0564 tenant-lifecycle commissioning) shipped a multi-tenant REST control plane
(`tenancy/facade/tenant-lifecycle-app`) with **zero authn/authz**: any network caller could
`DELETE /v1/tenants/{id}` and irreversibly retire any tenant. It passed **all 34 cloud-ci gates
green**. The independent reviewer caught it; the pipeline did not.

Per founder doctrine — friction is a process failure; productize the engine so the anti-pattern is
*impossible* to ship (ADR-0548 pipeline-as-product) — the pipeline itself must catch this class.
The repo already carries the correct fail-closed doctrine: `intelligence/adapters/rest/src/lib.rs`
(`admin_tenant_allowed` + a PDP `gate.decide` + `constant_time_eq`) and, post-fix,
`tenancy/facade/tenant-lifecycle-app/src/lib.rs` (`authenticate_caller` + `authorize()` per route).
Nothing mechanically enforced that **new** HTTP control-plane surfaces follow it.

This complements the audit-coverage requirement (AC-W-13: every authorize decision emits one audit
record) — that checks a decision is *audited*; this gate checks a decision is *made at all*. It
dogfoods the cloud-iam Cedar PDP as the canonical authz substrate (ADR-0559).

## Decision

Ship a **self-contained cloud-ci gate**, `cloud-ci-authz-coverage`
(`cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app`), mirroring the kernel-purity (ADR-0547)
registration footprint: own crate, own policy JSON, one appended matrix line in
`.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding.

The gate's neutral Rust engine lives in
`cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/src/lib.rs` (the I/O collector
`collect_surfaces(root, policy)` + the pure `evaluate_keyed(policy, observed)`), the binary in
`cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/src/main.rs`, the live-corpus + RED/GREEN
self-test in `cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/tests/authz_coverage.rs`, its
buck2 wiring in `cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/BUCK`, its manifest in
`cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/Cargo.toml`, and ALL repo-specifics as DATA
in `cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/authz-coverage-policy.json`.

### D1 — What is a control-plane surface

A Rust file constructing an axum `Router` (`Router::new()....route(path, METHOD(handler))`) is a
surface. A surface is a CONTROL PLANE when any route is a **mutating** method
(`post`/`put`/`patch`/`delete`) on a non-exempt path, or a per-resource path param
(`{id}`/`{tenant_id}`/...) on a mutating method. `/healthz`-style unauthenticated reads are exempt
via an explicit DATA allowlist (`exempt_path_substrings`), never code.

### D2 — Required authz coverage

A control-plane surface is COVERED iff its builder chain carries a recognized router-level auth
`.layer(...)` (a verified-principal extractor in `auth_layer_idents`) **and/or** every mutating
handler invokes a recognized authz decision in its body — an `admin_tenant_allowed`-style guard,
the tenancy `authorize(...)` pattern, a PDP `decide(...)` port call, a bearer/peer auth guard, or a
webhook signature verification, all named in `authz_guard_idents`. A mutating handler that derives
no caller identity → UNAUTHENTICATED. Handler-body detection follows up to one thin-wrapper
delegation (the real `intelligence/adapters/rest` data-plane wrappers delegate to a proxy that
calls `require_data_plane_bearer`), and recognizes the `guard!`-macro authz the iam SCIM surface
uses. The guard probe runs on a **code-only view** of the handler body — line/block comments and
string/char literals are elided — so a guard ident mentioned only in a `// TODO: authorize()`
comment or a string literal can **never** false-cover a handler (review bypass B3).
A generic `.layer(DefaultBodyLimit::max(...))` is **not** an auth layer.

### D2a — Detection envelope (recognized shapes) + fail-closed backstop

The detector is a textual matcher (no Rust-source AST kernel yet); it need not be perfect because
its failure mode is **fail-closed**. The precise recognized envelope:

- **Route paths**: a `"..."` string literal, or a `const`/`static NAME: &str = "..."` ident
  resolved by file-local substitution (review bypass B1 — const-path routers were previously
  invisible). Any path the engine cannot resolve to a concrete string (a non-literal expression
  such as `&format!(...)`, or a const with no string-literal initializer) yields
  `AC-UNRESOLVED-ROUTE-PATH`.
- **Method routers**: inline `get/head/options/trace` (non-mutating), `post/put/patch/delete`
  (mutating), `any` (mutating), `on(MethodFilter::X, h)` / `on_service(MethodFilter::X, h)`
  classified by the filter set (mutating if it names any POST/PUT/PATCH/DELETE or an unknown
  filter; review bypass B2 — `on(MethodFilter::DELETE, h)` was previously invisible), and a
  `MethodRouter` bound by `let m = METHOD(h);` resolved file-locally. Any method-router shape the
  engine cannot classify (an unresolved variable, an unrecognized call) yields
  `AC-UNCLASSIFIED-METHOD` and is treated as **potentially mutating**.

**The guarantee (precise):** any `.route(...)` the engine cannot prove is (a) non-mutating on a
resolved path OR (b) authorization-covered produces a finding (RED) — it is never silently skipped.
A router-level auth layer does not excuse an unresolved-path / unclassified-method route, because
those are recognition failures, not coverage facts.

### D3 — Conservative in the SAFE direction

Like the kernel-purity src-ident liveness probe, handler-body authz detection is a token
over-approximation: a guard ident appearing in the **code-only** handler body counts as covered. It
never invents a false UNAUTHENTICATED finding for a handler that genuinely calls a guard. The risk
it trades away (a guard *function* named in code but on a never-taken branch) is acceptable — this
gate stops the ZERO-authz class (the AUTH-005 exhibit had no guard token anywhere) plus the four
idiomatic-axum bypasses the PR #780 review reproduced; the deeper call-graph proof is the
audit-coverage gate's and human review's job. The honest claim is therefore **not** that an
unauthenticated control plane is *impossible* to ship — a sufficiently exotic router shape outside
the envelope, or a guard ident named-but-unreached, can still slip — but that any surface the
matcher cannot positively classify as non-mutating-or-covered **fails the gate**, so the
zero-authz / const-path / variable-method / comment-guard classes are mechanically blocked.

### D4 — Ratchet vs a frozen baseline of currently-known surfaces

Several pre-existing surfaces are unauthenticated today (`k8s/facade/control-plane-host-app`,
`k8s/facade/tenant-quota-app`, `k8s/facade/cluster-lifecycle-app`, the ci-controller `/gate-run`
route). Blocking them now is out of scope; that is each owner's remediation. The gate ships with a
FROZEN baseline of today's known-unauthenticated surface keys (`frozen_unauthenticated_surfaces` in
policy DATA): a finding whose key is in the baseline is ACCEPTED; a finding whose key is NOT in the
baseline is a NEW unauthenticated control plane → RED. This mirrors the
capability-membership / tier-acyclicity posture (born-green, enforce-no-regression). The baseline is
shrink-only — a fixed/removed surface drops its key, and a stale key self-cleans via
`AC-STALE-BASELINE`.

Baseline keys are **stable signatures** — `<file>::router[<sorted (method, route-path) tuples>]`,
independent of line numbers and route-declaration order — so an unrelated edit that shifts a
router's line no longer spuriously re-REDs a baselined surface (review M2). Re-baselining is
**AUTOMATED**: the gate binary's `--write` (alias `--update-baseline`) regenerates
`frozen_unauthenticated_surfaces` from the live tree (mirroring the kernel-purity `--fix` /
arch-graph `--write` pattern); the baseline is never hand-edited. When the broadened detector
surfaced pre-existing control planes that were previously invisible (the const-path
identity-workload-rest router, the `&format!`-path SCIM router), each was either recognized as
authz-covered (no baseline entry — e.g. the webhook gateway's `verify_any` signature check) or
grandfathered into the frozen baseline with a per-surface justification in the policy
`_frozen_baseline_note`. One genuine pre-existing gap is recorded there for escalation
(identity-workload-rest `POST /principals/{id}:suspend|retire` performs no authz) — baselined so the
gate stays green per the blocking-NEW contract, tracked for its owner to remediate.

### D5 — Hermetic, fail-closed

`collect_surfaces` is read-only: it walks the policy scan roots over the candidate tree, never
shells out, never touches the network or git (it does NOT use `cargo metadata`). Malformed policy,
a gate-id mismatch, and a below-floor surface census all fail closed
(`AC-POLICY-MALFORMED` / `AC-POLICY-GATE-ID-MISMATCH` / `AC-EMPTY-SCAN`). The blocking per-surface
codes are `AC-UNAUTHENTICATED-CONTROL-PLANE` (a recognized mutating route, no detectable authz),
`AC-UNRESOLVED-ROUTE-PATH` (a path the engine could not resolve to a concrete string — fail-closed),
and `AC-UNCLASSIFIED-METHOD` (a method-router the engine could not classify — fail-closed, treated as
potentially mutating); `AC-STALE-BASELINE` self-cleans a baseline key with no live finding. The
structure scan runs against a length-preserving code-only mask of each file, so comment/string
mentions of `Router::new()` / `.route(` (including the gate scanning its own source) never register
as surfaces. `#![forbid(unsafe_code)]`; deterministic sorted output.

## Consequences

- A NEW axum control plane with a mutating route and no detectable authz fails the
  `cloud-ci-authz-coverage` lane, with a remediation pointer to the
  `intelligence/adapters/rest` + `tenancy/facade/tenant-lifecycle-app` doctrine. Within the
  detection envelope (D2a) — including const-path routers, `on(MethodFilter)` shapes,
  variable-bound method routers, and comment/string-stripped coverage — any surface the gate cannot
  prove non-mutating-or-covered fails the lane, so the AUTH-005 zero-authz class and the four
  reviewed idiomatic-axum bypasses are mechanically blocked. (This is a fail-closed textual
  backstop, not a call-graph proof; see D3 for the precise, non-overstated guarantee.)
- The two reference surfaces are GREEN by authz detection, not by exemption.
- A surface that grows the frozen baseline (a new unauthenticated control plane) requires either
  real authz before merge or — for an intentional, founder-signed exception — a baseline edit.

## Alternatives considered

- **Full call-graph authz reachability** (prove every mutating handler reaches a PDP decision):
  rejected for v1 — it needs a Rust-source semantic graph the repo does not yet have; the
  over-approximation in D3 catches the zero-authz class without it. Reserved for a v2 once a
  Rust-source kernel exists.
- **A producer-face binding** (emit findings into the accounting registry): rejected — the
  accounting-registry producer is oyatie-specific; binding a face would kill R0 portability
  (same rationale as ADR-0547 D1).
