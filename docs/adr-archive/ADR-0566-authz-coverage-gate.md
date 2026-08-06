---
id: ADR-0566
title: "Authz-coverage gate (unauthenticated HTTP control-plane backstop)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-21
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0540, ADR-0544, ADR-0547, ADR-0548, ADR-0559]
amends: []
related: [ADR-0547, ADR-0506, ADR-0540, ADR-0544, ADR-0559, ADR-0561, ADR-0564]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


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
(`ci/facade/endpoint-authorization-coverage`), mirroring the kernel-purity (ADR-0547)
registration footprint: own crate, own policy JSON, one appended matrix line in
`.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding.

The gate's neutral Rust engine lives in
`ci/facade/endpoint-authorization-coverage/src/lib.rs` (the I/O collector
`collect_surfaces(root, policy)` + the pure `evaluate_keyed(policy, observed)`), the binary in
`ci/facade/endpoint-authorization-coverage/src/main.rs`, the live-corpus + RED/GREEN
self-test in `ci/facade/endpoint-authorization-coverage/tests/authz_coverage.rs`, its
buck2 wiring in `ci/facade/endpoint-authorization-coverage/BUCK`, its manifest in
`ci/facade/endpoint-authorization-coverage/Cargo.toml`, and ALL repo-specifics as DATA
in `ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json`.

### D1 — What is a control-plane surface

Surface DISCOVERY is anchored on the **route-introduction call set** — `.route(` / `.route_service(`
— **not** on a `Router::new()` constructor: a route cannot exist without one of those calls. Every
such call in a file is found and attributed to its enclosing function scope, so a route is discovered
regardless of how (or whether) a `Router::new()` / `Router::default()` / `Router::<S>::new()` / a
`Router` **parameter** / an aliased binding / a builder-returned `Router` produced it, and regardless
of whether it is declared before or **after** `.with_state(...)` (PR #780 second-pass BLOCKER-1/2/3;
the first pass anchored on the literal `Router::new()` token with a chain truncated at `.with_state(`,
which an adversarial re-review bypassed five ways). Two route grammars are classified: axum
`.route(path, METHOD(handler))` and the owned `oya-http-router-kernel`
`.route(HttpMethod::X, path, handler)`. A surface is a CONTROL PLANE when any route is a **mutating**
method (`post`/`put`/`patch`/`delete`/`any` or `HttpMethod::POST/PUT/PATCH/DELETE`) on a non-exempt
path, or a per-resource path param (`{id}`/`{tenant_id}`/...) on a mutating method, or any route the
engine cannot fully classify (fail-closed, see D2a). `/healthz`-style unauthenticated reads are exempt
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
its failure mode is **fail-closed** — including surface DISCOVERY itself (anchored on the
route-introduction call set, so it does not miss a router shape it never imagined). The precise
recognized envelope:

- **Route grammars**: axum `.route(path, METHOD(handler))` and the owned `oya-http-router-kernel`
  `.route(HttpMethod::X, path, handler)`. An `HttpMethod` method may be a literal `HttpMethod::X`
  or a `const NAME: HttpMethod = HttpMethod::X;` resolved file-locally (a `const … = HttpMethod::Post`
  must resolve, else a mutating route would be invisible — fail-closed `AC-UNCLASSIFIED-METHOD`).
- **Route paths**: a `"..."`/raw-string (`r"..."`/`r#"..."#`) literal, or a `const`/`static
  NAME: &str = "..."` ident resolved by file-local substitution (first-pass bypass B1). Any path the
  engine cannot resolve to a concrete string (a non-literal expression such as `&format!(...)`, a
  const with no string-literal initializer, a loop variable) yields `AC-UNRESOLVED-ROUTE-PATH`.
- **Method routers**: inline `get/head/options/trace` (non-mutating), `post/put/patch/delete`
  (mutating), `any` (mutating), `on(MethodFilter::X, h)` / `on_service(MethodFilter::X, h)`
  classified by the filter set (first-pass bypass B2), and a `MethodRouter` bound by
  `let m = METHOD(h);` resolved file-locally. Any method-router shape the engine cannot classify
  yields `AC-UNCLASSIFIED-METHOD` (potentially mutating).
- **Whole call shape**: a `.route(`/`.route_service(` matching NEITHER grammar (a macro-shaped or
  truncated/unbounded call) yields `AC-UNCLASSIFIED-SURFACE`. A `.merge(X)`/`.nest(p, X)`/
  `.nest_service(...)` whose sub-router X the engine did not scan-and-clear yields
  `AC-UNRESOLVED-SUBROUTER` — merged/nested content is NOT assumed covered.
- **Auth-layer / guard matching is whole-token** (PR #780 second-pass BLOCKER-4/5): an auth-layer
  ident must appear as a complete identifier token (so `RequireAuthMetricsRecorder` does NOT satisfy
  `RequireAuth`), and a plain-ident guard must be a complete token followed by `(` (so
  `unauthorized_response()` does NOT satisfy `authorize`).

**The guarantee (precise, NOT overstated):** any route-introduction the engine cannot FULLY classify
— method, path, AND authz-coverage — produces a finding (RED); it is never silently skipped. A
router-level auth layer does not excuse an unresolved-path / unclassified-method / unclassified-surface
/ unresolved-subrouter route, because those are recognition failures, not coverage facts. The one
deliberate non-finding boundary: a `.route(` whose arg-shape is indistinguishable from a same-named
**domain/dispatch** method (`usecase.route(input)`, an enum `intent.route()`, a zero-arg `.route()`)
is treated as a NON-route (dropped), which cannot hide an HTTP route because an HTTP route always
carries a string-ish path or an `HttpMethod::` verb.

### D3 — Conservative in the SAFE direction

Handler-body authz detection is a token over-approximation: a recognized guard ident invoked as a
whole-token CALL in the **code-only** handler body (comments + string/char literals elided) counts as
covered. It never invents a false UNAUTHENTICATED finding for a handler that genuinely calls a guard.
The whole-token + call-shape match (second-pass BLOCKER-5) closes the substring false-cover
(`unauthorized_response()` no longer satisfies `authorize`); the code-only view (first-pass B3)
closes the comment/string false-cover. The residual risk it trades away — a guard *function* CALLED
in code but on a never-taken branch — is acceptable; the deeper call-graph proof is the audit-coverage
gate's and human review's job. The honest claim is therefore **not** that an unauthenticated control
plane is *impossible* to ship — a guard ident CALLED but unreached can still slip, and the deliberate
domain-method drop boundary (D2a) is a textual heuristic — but that surface DISCOVERY is fail-closed
(anchored on the route-introduction call set) and any route-introduction the engine cannot positively
classify as non-mutating-or-covered **fails the gate**, so the zero-authz / const-path /
variable-method / on(MethodFilter) / comment-or-substring-guard / Router::default()-or-param /
post-`.with_state()` / owned-kernel-POST / unresolved-composition classes are mechanically blocked.

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

Baseline keys are **stable scope+signatures** —
`<file>#<scope>::router[<sorted (method, route-path) tuples>]`, independent of line numbers and
route-declaration order (review M2), with `<scope>` the enclosing-fn name so two route scopes in one
file get distinct keys. Re-baselining is **AUTOMATED but SHRINK-ONLY** (PR #780 second-pass MAJOR-1):
the gate binary's `--write` (alias `--update-baseline`) drops keys for fixed/removed surfaces but
**refuses to absorb any key absent from the prior committed baseline** (a NEW unauthenticated control
plane), exiting 2 and printing each new key; growing the baseline requires the explicit `--allow-new`
flag (a reviewed grandfather). The CI matrix runs the gate's `rust_test` legs (a verdict assertion),
NOT the binary with `--write`, so no automation can silently regrow the baseline. When the
second-pass broadened discovery surfaced pre-existing control planes that were previously invisible
(the owned-kernel `*_runtime_router` POST surfaces across billing/iam-tenant-rbac/hr/payroll, the
intelligence compat APIs, the provider-pool reload, k8s facades, ci-controller), each was either
recognized as authz-covered (no baseline entry — e.g. intelligence/adapters/rest, tenancy, and the
webhook gateway's `verify_any` signature check) or grandfathered into the frozen baseline with a
per-surface justification in the policy `_frozen_baseline_note`. The **ESCALATE list** (genuine
pre-existing unauthenticated mutating control planes, baselined so the gate stays green per the
blocking-NEW contract, tracked for their owners): billing/adapters/accounting-http,
iam/facade/tenant-rbac-app, oya/hr employment-infrastructure, oya/payroll run-infrastructure,
oya/intelligence anthropic-compat-api + openai-compat-api, oya/intelligence provider-pool-app
`POST /internal/seats/reload`, iam/facade/identity-workload-rest `POST /tokens/validate` +
`POST /principals/{id_and_verb}`, k8s control-plane-host / cluster-lifecycle / tenant-quota,
oya/ci-controller `POST /gate-run`. A separate **covered-but-unprovable** set (handlers carry authz
but a path/method the engine cannot resolve fails it closed — remediation is to make the path/method
a literal/const, not to add authz): the identity-service SCIM `&format!`-path router, the
workspace-shell const-path router, and the iac/provider-pool loop-variable registration calls.

### D5 — Hermetic, fail-closed

`collect_surfaces` is read-only: it walks the policy scan roots over the candidate tree, never
shells out, never touches the network or git (it does NOT use `cargo metadata`). Malformed policy,
a gate-id mismatch, and a below-floor surface census all fail closed
(`AC-POLICY-MALFORMED` / `AC-POLICY-GATE-ID-MISMATCH` / `AC-EMPTY-SCAN`). The blocking per-surface
codes are `AC-UNAUTHENTICATED-CONTROL-PLANE` (a recognized mutating route, no detectable authz),
`AC-UNRESOLVED-ROUTE-PATH` (a path the engine could not resolve to a concrete string — fail-closed),
`AC-UNCLASSIFIED-METHOD` (a method-router the engine could not classify — fail-closed, treated as
potentially mutating), `AC-UNCLASSIFIED-SURFACE` (a `.route(`/`.route_service(` whose whole call shape
matched neither route grammar — fail-closed), and `AC-UNRESOLVED-SUBROUTER` (a `.merge`/`.nest`
sub-router the engine could not resolve to a scanned-and-cleared router — fail-closed composition);
`AC-STALE-BASELINE` self-cleans a baseline key with no live finding. The structure scan runs against
a length-preserving code-only mask of each file (raw-string aware, so `r#"..."#` content cannot
desync the mask), so comment/string mentions of `.route(` (including the gate scanning its own source)
never register as surfaces. `#![forbid(unsafe_code)]`; deterministic sorted output.

## Consequences

- A NEW control plane (axum or owned-kernel) with a mutating route and no detectable authz fails the
  `cloud-ci-authz-coverage` lane, with a remediation pointer to the
  `intelligence/adapters/rest` + `tenancy/facade/tenant-lifecycle-app` doctrine. Within the
  detection envelope (D2a) — route-introduction-anchored discovery, both route grammars, const-path
  and raw-string-path resolution, `on(MethodFilter)` shapes, variable/const-bound methods,
  whole-token auth-layer/guard matching, and fail-closed unresolved-path / unclassified-method /
  unclassified-surface / unresolved-subrouter handling — any route-introduction the gate cannot prove
  non-mutating-or-covered fails the lane, so the AUTH-005 zero-authz class plus the second-pass
  bypasses (Router::default()/param, post-`.with_state()` route, substring auth-layer/guard) are
  mechanically blocked. (This is a fail-closed textual backstop, not a call-graph proof; see D3 for
  the precise, non-overstated guarantee, including the deliberate domain-method drop boundary.)
- The two reference surfaces are GREEN by authz detection, not by exemption.
- The broadened discovery surfaced a substantial ESCALATE list of genuine pre-existing unauthenticated
  mutating control planes (D4) — grandfathered into the frozen baseline so the gate lands green, each
  owned by its capability team for remediation.
- A surface that grows the frozen baseline (a new unauthenticated control plane) requires real authz
  before merge; the `--write` re-baseline is shrink-only and refuses new keys without `--allow-new`,
  so a careless re-baseline cannot silently absorb a new gap.
- The HR remediation of the grandfathered `oya/hr employment-infrastructure` surface is accounted
  by the crate-scoped `oya/hr/crates/oya-hr-employment-infrastructure/OWNERS` boundary and the
  fail-closed source module `oya/hr/crates/oya-hr-employment-infrastructure/src/authz.rs`. The
  module supplies the verified-principal/PDP seam for the HR mutating routes named in D4 without
  widening the authz-coverage gate policy.
- The payroll auth-failure API-contract addendum is accounted by
  `evidence/multispectrum/hr-payroll-payroll-auth-contract-20260625-1782429919.json`, which records
  the HR/payroll capability matrix and Buck2 contract verification for the documented 401/403
  payroll money-movement responses.

## Alternatives considered

- **Full call-graph authz reachability** (prove every mutating handler reaches a PDP decision):
  rejected for v1 — it needs a Rust-source semantic graph the repo does not yet have; the
  over-approximation in D3 catches the zero-authz class without it. Reserved for a v2 once a
  Rust-source kernel exists.
- **A producer-face binding** (emit findings into the accounting registry): rejected — the
  accounting-registry producer is oyatie-specific; binding a face would kill R0 portability
  (same rationale as ADR-0547 D1).
