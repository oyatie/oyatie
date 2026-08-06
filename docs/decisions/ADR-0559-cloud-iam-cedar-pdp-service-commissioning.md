---
id: ADR-0559
title: "Commission the cloud-iam Cedar PDP service (G004 slice 1): a runnable authorization-decision service over the shared embedded engine"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-12
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0536, ADR-0550]
amends: []
related: [ADR-0243, ADR-0476, ADR-0510, ADR-0512, ADR-0515, ADR-0541, ADR-0547, ADR-0553, ADR-0555]
related_specs: []
milestone: W0
---

# ADR-0559: Commission the cloud-iam Cedar PDP service (G004 slice 1)

## Status

**Proposed - 2026-06-12 (G004 vertical opener, sanctioned by the ultragoal story
G004-g04-cedar-pdp-policy-store and the ADR-0536 D-2 authorization decision; door: two-way —
no consumer points at the service yet, deleting the three crates restores the prior state).**

## Context

Three live consumers already authorize through Cedar PEP **adapters** against a decision
substrate that does not exist as a service:

1. **oya/identity** — `CedarWorkloadAuthorizer` behind the `WorkloadAuthorizer` port
   (ADR-0553/ADR-0476): a per-service policy file, no central bundle, no policy versioning.
2. **cloud/cloud-intelligence** — `CedarAuthzGate` behind the kernel `AuthzGate` trait
   (ADR-0384 D7): a compiled-in policy file with a hard-coded action→role mapping.
3. **cloud/cloud-kms** — the crypto API consumes *authorization receipts*
   (`decision_id` + tenant + principal match, `AuthorizationDenied` fail-closed paths) minted
   by an upstream decision point that nothing currently mints.

G006 (tenancy + RBAC) is blocked on G004. The shared shapes already exist and are
conformance-tested: `libs/oya-shared-pdp-kernel` (the embedded-PDP port, bundle/cache/audit
value types) and `iam/adapters/pdp-cedar` / `iam-pdp-cedar` (the upstream formally-verified
`cedar-policy` engine behind that port — the single decision algorithm per ADR-0243), over the
locked PDP contract family in `libs/oya-shared-platform-contracts-kernel::pdp` (PARC request,
attributable response, zookie policy-version semantics).

Per the three-plane identity doctrine, **cloud-iam IS the IdP substrate, including "Cedar PDP +
policy-bundle distribution"** — the decision point and the policy store live in
`cloud/cloud-iam`. ADR-0536 D-2 fixes the runtime posture: embedded in-process PDPs everywhere,
a central policy store that compiles/signs/pushes content-addressed bundles, RBAC + ABAC + PBAC
as the full suite, and a structural tenant-isolation forbid.

## Decision

Commission `cloud/cloud-iam`'s policy-decision-point service as G004 slice 1: a **runnable
authorization-decision service** in the ADR-0550 kernel/adapter/app shape, reusing the shared
PDP kernel + Cedar adapter wholesale (zero forked evaluation logic).

### D1 — Service shape (ADR-0550 seams)

- `oya-cloud-iam-pdp-kernel` — pure ports: `PolicyBundleStore` (policy-store backend seam),
  `DecisionAuditSink` (decision-audit emission, the oya-identity `AuditSink` shape), and the
  twelve-factor `PdpConfig` parser. Kernel-pure per ADR-0547 (no transient tech; gate-enforced).
- `oya-cloud-iam-pdp-bundle-file-adapter` — the slice-1 policy-store backend: one declarative
  policy-bundle JSON document on a mounted path (ConfigMap transport), closed-schema parsed and
  invariant-re-checked, every error fail-closed. Deliberately throwaway (ADR-0510/ADR-0550).
- `oya-cloud-iam-pdp-app` — the composition root and delivery surface: gRPC
  (`oya.cloud.iam.pdp.v1.CloudIamPdp`) + REST (`POST /v1/authorize`, `/healthz`, `/readyz`)
  decision endpoints over ONE shared decision core (`PdpState::decide` — the two protocols
  cannot drift), structured decision-audit emission, production ULID decision-id minting
  (ADR-0506 blessed entropy), graceful-drain serving, and the binary entrypoint.

### D2 — API surface (designed for the three consumers; no trait changes at adoption)

`authorize(principal, action, resource, context, entity-slice, min_policy_version?) →
decision_id + Allow/Deny + policy_version + determining_policy_ids + obligations`:

- the PEP assembles and ships the entity slice; the PDP never dials a PIP at decision time
  (ADR-0536 D-2 — no network hop *inside* the decision);
- a **Deny is a decision response**, never a protocol error; every refusal (invalid request,
  unknown action, stale zookie pin) is a non-success status that PEPs MUST treat as deny;
- `decision_id` is minted fresh per decision (cached replays included) — the audit-chain
  correlation key the cloud-kms receipt validation already expects;
- the response echoes `policy_version` (zookie) and accepts `min_policy_version` pins
  (equality-only, refusal on mismatch — read-your-writes against the policy store);
- RBAC + ABAC + PBAC are all expressible: the surface is PARC + attributes + context +
  template-linked bundles, never an RBAC-only shape (full-spectrum authz doctrine).

### D3 — Policy-store port destination

Slice 1 transports bundles as a file/ConfigMap document behind `PolicyBundleStore`. The
destination (follow-up slices) is **declarative policy-bundle distribution owned by cloud-iam**:
a policy-bundle CRD + operator reconciliation pushing content-addressed, signed bundles
(compile/sign/push per ADR-0536 D-2), with signature verification at the store boundary and
`swap_bundle` live-reload (the shared adapter already supports atomic swap + structural cache
invalidation). Content addressing is NOT fabricated locally in slice 1 — minting content
addresses without the store control plane would be fake provenance; the version token stays
store-owned and opaque (ADR-0541 content-addressed identity is the durable destination).
Cutover litmus: `load() → PolicyBundle` survives unchanged; distribution transports come and go
behind it.

### D4 — Doctrine bindings

- **Default-deny everywhere**: Cedar deny-by-default + forbid-overrides-permit; unknown routes
  404; unknown actions refuse; the structural tenant-isolation forbid ships in the seed bundle.
- **Fail-closed boot**: bundle unavailable/malformed/strict-validation-failure ⇒ BOOT REFUSAL
  (non-zero exit), never a degraded serve (the oya-identity precedent).
- **PDP, not PEP**: decisions are deterministic + side-effect-free; the only emission is the
  audit record, which never alters the decision.
- **API-only service** (cli_surface_policy): no CLI surface; declarative bundles are the
  management surface; K8s-native env config (twelve-factor) + SIGTERM drain.
- **Audit per decision** (G004 acceptance): one attributable record per decision — allow or
  deny, cached or evaluated — behind the `DecisionAuditSink` port; the audit-chain bridge lands
  behind the same port.

### D5 — Testing ladder (founder standard; unit alone inadequate)

- kernel/adapter unit suites (config, ports, fail-closed RED cases: missing file, malformed
  JSON, unknown fields, invalid version token);
- contract suites on BOTH delivery surfaces (REST via in-process router, gRPC via direct
  service impl): RBAC/ABAC/PBAC exemplars, deny-by-default, structural cross-tenant forbid,
  refusal status mapping, closed body schema, audit-per-decision, cache-replay id freshness;
- live-socket E2E through the production boot path (`server::start` — the tested wiring IS the
  production wiring): REST + gRPC over real sockets, readiness echoing the loaded bundle;
- RED fixtures: unauthorized → deny; policy-load failure (missing/garbage/invalid-Cedar
  bundle) → boot refusal;
- seed-parity guard pinning the crate-local Cedar seeds byte-identical to the canonical FD-001
  seeds (the established conformance-suite pattern).

### D6 — Adoption path (follow-up slices, not this one)

0. **PDP-API caller authentication** (workload-identity mTLS/SPIFFE via oya/identity) lands
   BEFORE any consumer pointing: in slice 1 the decision API trusts its network boundary
   (cluster-internal, no consumer pointed at it yet), and the PEP-supplied entity slice is
   only as trustworthy as the caller — the standard embedded-PDP/AVP trust model, which is
   exactly why callers must be authenticated workloads before adoption.
1. Policy-bundle CRD + operator distribution (D3 destination) + bundle signing/verification.
2. Point oya/identity's `WorkloadAuthorizer` adapter at the shared PDP port/bundle fabric;
   retire its per-service policy file.
3. Point cloud-intelligence's `AuthzGate` adapter at the same fabric; retire the hard-coded
   action→role mapping (its documented v2).
4. Mint cloud-kms authorization receipts from `AuthorizeResponse.decision_id`.
5. Retire the hand-rolled `oya/policy/crates/oya-policy-cedar-*` evaluator (ADR-0243: two
   decision algorithms must never coexist) — tracked by the G004 story.

### D7 — Ownership + justification manifest (ADR-0555 D2)

Owner: `cloud/cloud-iam/OWNERS` = `axis-cloud-platform` (the existing cloud-kms /
cloud-intelligence owner). Files commissioned by this decision:

`cloud/cloud-iam/OWNERS`,
`iam/core/cloud-pdp-kernel/BUCK`,
`iam/core/cloud-pdp-kernel/Cargo.toml`,
`iam/core/cloud-pdp-kernel/src/lib.rs`,
`iam/adapters/cloud-pdp-bundle-file/BUCK`,
`iam/adapters/cloud-pdp-bundle-file/Cargo.toml`,
`iam/adapters/cloud-pdp-bundle-file/src/lib.rs`,
`iam/facade/cloud-pdp-app/BUCK`,
`iam/facade/cloud-pdp-app/Cargo.toml`,
`iam/facade/cloud-pdp-app/build.rs`,
`iam/facade/cloud-pdp-app/proto/cloud-iam-pdp.proto`,
`iam/facade/cloud-pdp-app/cedar/platform.cedarschema`,
`iam/facade/cloud-pdp-app/cedar/platform-policies.cedar`,
`iam/facade/cloud-pdp-app/cedar/platform-templates.cedar`,
`iam/facade/cloud-pdp-app/src/lib.rs`,
`iam/facade/cloud-pdp-app/src/audit.rs`,
`iam/facade/cloud-pdp-app/src/grpc.rs`,
`iam/facade/cloud-pdp-app/src/idgen.rs`,
`iam/facade/cloud-pdp-app/src/observability.rs`,
`iam/facade/cloud-pdp-app/src/rest.rs`,
`iam/facade/cloud-pdp-app/src/server.rs`,
`iam/facade/cloud-pdp-app/src/main.rs`,
`iam/facade/cloud-pdp-app/tests/common/mod.rs`,
`iam/facade/cloud-pdp-app/tests/rest_contract.rs`,
`iam/facade/cloud-pdp-app/tests/grpc_contract.rs`,
`iam/facade/cloud-pdp-app/tests/e2e_live_socket.rs`,
`iam/facade/cloud-pdp-app/tests/seed_parity.rs`.

Wave A Cloud IAM API-first parity addendum: the REST/OpenAPI contract and its
Buck2 parity gate are commissioned under the same Cloud IAM authorization
decision surface, reusing `iam/ports/cloud-api` surface constants rather than
minting a second source of truth:

`cloud/cloud-iam/contracts/openapi/cloud/cloud-iam-v1.yaml`,
`cloud/cloud-iam/contracts/BUCK`,
`cloud/cloud-iam/contracts/tests/openapi_parity.rs`,
`evidence/multispectrum/waveA-cloud-iam-openapi-parity-20260625-1782426426.json`.

The gRPC proto stays crate-local in slice 1 (structurally accounted via cargo-members
reachability); it promotes to `cloud/cloud-iam/contracts/proto/` (export_file PUBLIC + its own
reachability registration) with the first external consumer slice.

## Precedent

- **Cedar / Amazon Verified Permissions** (the architecture this service IS): a formally
  verified evaluator (arXiv 2403.04651) embeddable in-process AND exposed as a managed
  decision API; policy templates as policy-as-data (our PBAC `TemplateLink`); policy stores
  versioned centrally. AVP's `IsAuthorized` returns decision + determining policies — the
  exact `AuthorizeResponse` shape.
- **Google Zanzibar** (the relationship-authz CONTRAST and the consistency model): zookies —
  every decision carries the policy version it was evaluated against and callers pin freshness
  floors — are adopted (`policy_version` echo + `min_policy_version`); Zanzibar's global
  relation-tuple store is NOT — Oyatie's authorization model is policy-over-attributes
  (Cedar), not a relationship graph, and a Zanzibar-class tuple store is unjustified
  complexity at this stage (revisit only if relationship-heavy products demand it; the
  quorum-not-etcd-class doctrine counsels against a single global consistency domain anyway).
- **Google IAM / AWS IAM decision-point patterns**: central policy administration with
  decisions evaluated close to the resource (embedded evaluators distributed via policy
  push), default-deny with explicit-deny dominance — the same compile/push/evaluate split as
  ADR-0536 D-2.
- **Owned-stack posture**: Cedar is Rust (Apache-2.0), embedded as a crate — no transient
  vendor service to absorb behind an adapter; the engine itself is terminal (ADR-0536 D-2),
  only the bundle TRANSPORT is transitional.

## Rejected

- **A remote-only PDP on every request path** (ADR-0536 D-2 already rejected: latency +
  availability coupling). This service does not contradict D-2: consumers keep their embedded
  PDPs; the service is the decision point for consumers without embedded bundles yet, the
  decision API for the policy store itself, and the substrate the bundle-distribution fabric
  grows from.
- **Forking evaluation logic into cloud-iam** (ADR-0243: two decision algorithms must never
  coexist; the shared adapter is the single engine).
- **Hand-rolled relationship-tuple store (Zanzibar-class)** — see Precedent.
- **Locally fabricated content addresses for bundles in slice 1** — fake provenance; the
  store control plane mints addresses when it exists (D3).
- **A CLI management surface** — retirement-marked class (cli_surface_policy); bundles are
  declarative data.

## Consequences

- The G004 vertical is open with a runnable, fully-tested decision service; later slices are
  consumer pointings + distribution fabric, each independently shippable (FRIC-1781440000
  tracks the stub-PDP friction this closes).
- `cloud/cloud-iam` gains its OWNERS marker, retiring it from the unowned residue.
- The legacy `oya-cloud-iam-app` Cedar *bind* surface (over the hand-rolled
  `oya-policy-cedar-domain`) remains untouched in this slice; it is absorbed when the
  ADR-0243 retirement slice lands (D6.5).
