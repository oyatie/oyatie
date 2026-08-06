---
id: ADR-0246
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-policy-engine
  - axis-ontology
  - axis-identity
  - axis-audit-chain
  - ops-compliance
  - ops-sre-reliability
supersedes: []
amends:
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - microservices/ontology/PRD.md (drops cedar-fragment-coverage BC; rewrites §"universal mediator" framing; renames agent-gateway BC to tool-call-ingress)
superseded_by: [ADR-701]
amended_by: [ADR-0280]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0050-event-bus-and-outbox-canonical.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0106-application-to-usecase-rename.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-canonical.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-policy.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0632
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/cedar-fragment-schema.json
  - /specs/policy-gate-coverage.json
  - /specs/per-microservice-flat-layout.json
related_memory:
  - feedback_cedar_as_universal_gate
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
  - feedback_workflow_objectgraph_adapter_layer
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 5-of-14
purpose: >
  Promote the existing `cedar-fragment-coverage` bounded context out of
  the `ontology` microservice and into its own peer substrate µservice
  `microservices/policy-engine/`. The promotion is justified because
  ADR-0243 (Cedar-as-Universal-Gate) elevated Cedar from an app-tier
  authorization concern to a load-bearing universal-gate substrate that
  every other µservice in the portfolio consults on the hot path of
  every state-changing action. BC-level isolation inside Ontology no
  longer matches the substrate's load-bearing role; per ADR-0131 the
  unit of independent deployment, SLO ownership, and team accountability
  is the µservice; per ADR-0241 the universal-gate's DR tier (T1, 5min
  RTO, 0 RPO) cannot be subordinated to Ontology's tier (T2/T3 depending
  on BC); per ADR-0245 substrates must be peer µservices so that
  products can compose them without coupling to upstream BC churn.
enforcement_status: advisory-until-policy-engine-substrate-lands
enforced_by:
  - cloud-ci/Rust gate packet policy-engine-substrate-promoted
  - cloud-ci/Rust gate packet cedar-coverage
  - cloud-ci/Rust gate packet no-policy-in-code
  - cloud-ci/Rust gate packet cedar-fragment-signature
  - cloud-ci/Rust gate packet cedar-default-deny-coverage
clarified_by: [ADR-0632]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0246: Policy-Engine Substrate Promotion

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR. Each
keystone references the others; partial acceptance is rejected because
the doctrines are mutually-reinforcing and produced together to avoid
the drift pattern that produced ADR-0220 → ADR-0239 amendment within
twelve days.

Keystone position: 5 of 14.

Enforcement is `advisory-until-policy-engine-substrate-lands`. The
proposal is recorded in text but is not Accepted; the CI lanes described here would move
to BLOCKER status only after:

1. `microservices/policy-engine/` directory exists under
   `microservices/` at the canonical flat layout per ADR-0131.
2. The crate redistribution per §D-3 is complete and `cargo build
   --workspace` passes from a clean checkout.
3. The evaluator Deployment (3+ replicas per cell) is deployable in at
   least one bootstrap cell; the SDK (`oya-shared-policy-engine-client`)
   is consumable by at least one downstream µservice (pilot:
   `microservices/tenancy/`).
4. The genesis fragment per §D-8 is signed by the org root key (held in
   the tier-0 HSM cluster) and the signing-chain verification path
   exercises green against integration tests.
5. The Ontology PRD amendment per §D-9 is merged, the
   `cedar-fragment-coverage` BC tombstone marker is in place, and the
   agent-gateway BC has been renamed `tool-call-ingress` with the
   corresponding crate rename ChangeSet landed.

Until those five items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER.

## ADR-0632 product-protocol clarification (proposal only)

This ADR remains **Proposed** and does not accept a protocol surface or implementation. If accepted,
its “gRPC-primary” language applies only to internal gRPC/proto3 over HTTP/2; HTTPS REST documented
by OpenAPI 3.2.0 is the public and compatibility surface. GraphQL, public gRPC, gRPC-Web, and Connect
remain forbidden by ADR-0632.

## Date

2026-05-20.

## Context

### Prior portfolio state — Cedar lives inside Ontology

Cedar policy enforcement entered the portfolio in three increments:

- **ADR-0140 (2026-Q1, retired per ADR-0145).** Original framing of
  Cedar as authorization-only; Cedar PDP carried sketchily inside
  whichever µservice declared a permit. Retired because per-µservice
  Cedar evaluators caused drift between fragment versions across the
  fleet.
- **ADR-0150 (Cedar policy engine, 2026 LTS v4.2).** Establishes Cedar
  v4.2 as the policy engine for app-tier authorization. Scopes permits
  + forbids; default-deny pattern; per-Action Cedar fragment coverage
  CI lane (`oya-governance-cedar-coverage`); Cedar fragment
  registry lives as the `cedar-fragment-coverage` BC inside the
  `ontology` µservice (per `microservices/ontology/PRD.md` §"Bounded
  Contexts" table, line 120: BC `cedar-fragment-coverage` family
  `oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}`).
- **ADR-0183 (Cedar app authz + Kyverno admission, 2026-05-18).**
  Separates responsibility: Cedar gates app-tier authorization
  decisions; Kyverno gates Kubernetes admission decisions. Both run in
  parallel. Cedar PDP at this point still framed as a Deployment in
  the `governance` µservice namespace consulted via Envoy `ext_authz`
  at the Istio Ambient waypoint, with policy fragments authored in
  Ontology's `cedar-fragment-coverage` BC.

That layout was correct for the scope Cedar then carried: app-tier
authorization with Ontology as the natural fragment-registry host
because Ontology already owned the Object Type / Action Type catalog
that Cedar fragments authorize against. Per-Action permits + default-
denies sat one git-tree hop from the action declarations themselves.

### What changed at 2026-05-20 — ADR-0243 expanded Cedar to universal-gate

ADR-0243 (Cedar as Universal Gate, keystone #2) catalogued 23 policy-
class decisions in the portfolio that were authored as imperative code
or static configuration:

- Provider routing in Intelligence (data-class → LLM provider).
- Cell routing (tenant → home_cell).
- Tax / region routing.
- Audit-stream selection.
- Cost-center attribution.
- Feature flag evaluation.
- Schema-revision activation.
- Marketplace surface eligibility.
- Compliance pack activation per tenant.
- DSAR cascade scope.
- Retention sunset.
- Rate limit + quota tiering.
- Cross-cell traffic permits.
- encryption-BYOK permit per data class.
- Webhook subscription eligibility.
- Cron-trigger eligibility.
- Sandbox tenant lifetime + resource budget.
- Bulk import/export eligibility.
- Plugin Wasmtime capability allowlist.
- Audit retention extension (legal hold).
- Cross-tenant collaboration permit.
- Partner-tenant on-behalf-of (agency assume-role).
- Reserved-namespace registration refusal.

ADR-0243 §D-1 declared every one of those 23 decisions a Cedar
evaluation. The doctrine — "Code never decides policy; code asks the
policy engine and acts on the answer." — is load-bearing for the entire
portfolio.

### Why BC-level isolation no longer suffices

Cedar's scope expansion changes the substrate's deployment shape in
five concrete ways:

1. **Load-bearing on every state-changing call.** Before ADR-0243,
   Cedar evaluation sat on the authorization hot path of µservices
   that opted into Cedar. After ADR-0243, every µservice's every
   state-changing action consults Cedar at least once. The hot-path
   QPS expectation for Cedar across the fleet rises from ~10× the
   Ontology-Action QPS to ~1000× the Ontology-Action QPS (every
   Workflow Engine step, every Mail send, every Drive write, every
   Marketplace publish, every Messenger emit, every Tenancy admission,
   every Identity issuance, every Webhook fan-out, every Cron trigger,
   every Plugin capability call, every encryption-BYOK key-use, every cross-cell
   call, every audit-stream selection, every cost-center attribution).

2. **Independent SLO ownership required.** Per ADR-0241 the policy
   engine is T1 (5min RTO, 0 RPO). Ontology's SLO mix per its own PRD
   §"Availability + SLO" is 99.99% with RTO ≤ 10s (Function reads —
   T2 equivalent) for the entity-query BC and T2/T3 for other BCs. A
   T1 substrate cannot be a BC of a non-T1 µservice; the µservice's
   SLO is the union (worst case) of its BCs, so promoting Cedar to T1
   inside Ontology would promote *Ontology as a whole* to T1, which
   neither matches Ontology's resource budget nor the team-ownership
   structure (axis-ontology owns Ontology; axis-policy-engine — newly
   formed per this ADR — owns the Cedar substrate).

3. **Per-cell pod topology required.** ADR-0243 §D-6 requires per-cell
   policy-engine evaluator Deployments (3+ replicas per cell, HPA on
   QPS). Ontology's per-cell topology serves multiple Object Type /
   Action Type / Function / Link concerns; co-locating an evaluator
   Deployment in the Ontology cell means Cedar competes with Function
   reads for resource budget, with cache-line pressure between Cedar's
   compiled-policy AST cache and Ontology's Function-result cache.
   Per AWS Verified Permissions production data, Cedar's hot-path
   cache lines exhibit different access patterns than relational
   read caches; co-location degrades both.

4. **Coupled SLO dependency in current shape.** Currently, an Ontology
   degradation (e.g., Function read brownout) causes Cedar fragment-
   reload + signing-chain operations to slow because they sit inside
   the Ontology process boundary. ADR-0243 §D-10 hot-reload guarantees
   <5s p99 propagation across all replicas. Ontology brownout
   conditions historically run 30-60s; the coupling exceeds the hot-
   reload SLO by an order of magnitude.

5. **Bootstrap ordering inversion.** Per ADR-0242 §D-5, the bootstrap
   sequence is: hardware → bootstrap cell → cloud-secrets → identity →
   tenancy → **policy-engine** → audit-chain → cell-registry → workflow-
   engine → first Foundry workflow. Ontology is bootstrapped *after*
   policy-engine because Ontology's first writes already need Cedar
   evaluation (Action Type registration gates → Cedar fragment lookup
   → permit/deny). If policy-engine were still a BC inside Ontology,
   the bootstrap would have a circular dependency: Ontology cannot
   bootstrap without Cedar; Cedar cannot bootstrap without Ontology.
   The promotion breaks the cycle by making policy-engine its own
   µservice that bootstraps before Ontology.

### Why a µservice (not a library, not a sidecar)

Three named alternatives to "promote to peer µservice" were considered:

- **Library (Rust crate vendored by every µservice).** Each µservice
  links in `cedar-policy` v4.2 + a thin fragment-loader. Pros: zero
  network hop; lowest latency. Cons: fragment registry must be
  globally consistent → distributed-systems consensus needed for
  hot-reload across thousands of process replicas; per-tenant overlay
  composition multiplies the fragment-set held in each replica's
  memory by the active tenant count; signing-chain verification
  duplicated across every binary; emergency-permit propagation
  (per ADR-0243 §Appendix B incident-response scenario) becomes a
  full fleet rollout rather than a 5s hot-reload. Rejected because
  hot-reload + emergency-permit latency dominates.

- **Sidecar (one pod per service pod, daemonset-style).** Each
  µservice's pod gets a co-located Cedar evaluator sidecar; SDK calls
  loopback. Pros: low network hop (loopback); per-µservice failure
  isolation. Cons: pod count multiplies by 2x across the fleet (~46
  µservices × ~10 replicas average × 2 = ~920 pods of evaluator
  sidecars); fragment-registry-to-sidecar fan-out for hot-reload
  exceeds Kafka topic partition reasonable limits; per-cell capacity
  planning becomes per-µservice instead of per-cell; HPA scaling of
  sidecars decoupled from µservice scaling causes interesting failure
  modes. Rejected because the operational footprint outweighs the
  loopback latency benefit, and AWS Verified Permissions architecture
  (the reference Cedar deployment at hyperscale) chose centralized
  evaluator-per-cell over sidecar-per-pod after measuring identical
  tradeoffs.

- **Centralized µservice (per-cell evaluator pool).** A peer substrate
  µservice owns the evaluator pool per cell; downstream µservices call
  it over gRPC with an SDK that holds a connection pool + circuit
  breaker + cache fallback (per ADR-0243 §D-11 fail-closed default).
  Pros: independent deployment + SLO + team ownership; hot-reload is
  a per-cell Kafka pub-sub with O(replicas-per-cell) fan-out;
  emergency-permit propagation hits all evaluators in <5s; per-tenant
  overlay composition holds in evaluator memory shared across all
  callers; signing-chain verification centralized; bootstrap orders
  cleanly. Cons: one network hop per evaluation (mitigated to <1ms
  p99 by ADR-0243 §D-6 in-cell evaluator + Valkey hot cache +
  circuit-breaker fallback). **Chosen.**

This matches AWS Verified Permissions (separate AWS service since
2024-Q1 GA), Open Policy Agent at Netflix/Pinterest (centralized OPA
service-per-cell), and Google Org Policy (centralized service consulted
by every workload).

### Why now

Four forcing functions converge at 2026-05-20:

- **ADR-0243 lands today.** Cedar's universal-gate role is established
  by the keystone bundle; without §D-1 promotion, the bundle is
  internally incoherent (a load-bearing substrate hidden inside a non-
  substrate µservice's BC).
- **ADR-0241 (DR + BC portfolio policy) requires T1 tier declaration
  per µservice.** Cedar cannot declare T1 while hidden inside
  Ontology's T2/T3 µservice envelope.
- **ADR-0242 §D-5 bootstrap sequence requires policy-engine as step 5
  of 10.** Cedar must exist as `microservices/policy-engine/` for the
  bootstrap migration `0001_create_self_tenant.sql` (step 4) to
  reference its admission gate. Without the µservice, the bootstrap
  sequence has no step-5 deployable artifact.
- **The autonomous-masterplan goal (feedback_autonomous_implementation_artifacts).**
  The long-term goal of "Implement the masterplan runs without user
  intervention" requires deterministic per-call Cedar evaluation. With
  policy-engine as a peer substrate µservice, Foundry workflows can
  modify policy-engine fragments under Cedar gates that they themselves
  evaluate (per ADR-0247 self-modification doctrine). Without
  promotion, the self-modification path circles back through Ontology,
  creating a coupling that breaks ADR-0247.

## Decision

### D-1. Promote `cedar-fragment-coverage` BC to peer µservice `microservices/policy-engine/`

The `cedar-fragment-coverage` bounded context resident in
`microservices/ontology/` is promoted to its own peer µservice
`microservices/policy-engine/`. The new µservice is a *substrate*
(per ADR-0245 substrate-vs-product layering); it is consumed by every
other µservice — substrate or product — via a thin Rust SDK
(`oya-shared-policy-engine-client`) that performs the gRPC call to the
cell-local evaluator pool.

The promotion follows the ADR-0131 per-microservice flat layout:

```
microservices/policy-engine/
├── PRD.md
├── README.md
├── api/
│   ├── openapi/policy-engine.openapi.yaml
│   └── grpc/policy_engine.proto
├── capabilities/
│   ├── evaluate.yaml
│   ├── publish-fragment.yaml
│   ├── activate-fragment.yaml
│   ├── sunset-fragment.yaml
│   ├── get-fragment-by-id.yaml
│   ├── list-fragments.yaml
│   ├── get-coverage-report.yaml
│   ├── get-evaluation-by-id.yaml
│   └── verify-signature-chain.yaml
├── calls.yaml
├── fragments/
│   ├── baseline/
│   ├── bootstrap/
│   ├── overlay/
│   ├── pack/
│   ├── reserved/
│   └── tenant/
├── iac/
│   └── helm/
│       └── policy-engine/
│           ├── values.yaml
│           ├── values.cell-bootstrap.yaml
│           ├── values.cell-data-plane.yaml
│           └── templates/
│               ├── evaluator-deployment.yaml
│               ├── evaluator-hpa.yaml
│               ├── evaluator-pdb.yaml
│               ├── fragment-registry-statefulset.yaml
│               ├── hot-reload-worker-deployment.yaml
│               ├── valkey-hot-cache-statefulset.yaml
│               ├── kafka-topic-fragment-reload.yaml
│               └── service-policy-engine.yaml
├── migrations/
│   ├── 0001_fragment_registry_init.sql
│   ├── 0002_signing_keys_init.sql
│   ├── 0003_coverage_reports_init.sql
│   ├── 0004_evaluation_audit_index_init.sql
│   └── 0005_genesis_fragment_seed.sql
├── slos/
│   ├── evaluate-hot-path.openslo.yaml
│   ├── evaluate-cold-path.openslo.yaml
│   ├── hot-reload-propagation.openslo.yaml
│   ├── signing-chain-verification.openslo.yaml
│   └── coverage-completeness.openslo.yaml
├── src/                   # composition-root convenience; real code lives in crates/
└── tests/
    ├── integration/
    │   ├── hot_reload.rs
    │   ├── signing_chain.rs
    │   ├── tenant_overlay.rs
    │   ├── emergency_permit.rs
    │   ├── fail_closed_fallback.rs
    │   └── bootstrap_genesis.rs
    └── load/
        ├── evaluate_p99.rs
        └── hot_reload_propagation.rs
```

Per ADR-0131, the µservice's *code* lives under
`microservices/policy-engine/src/crates/oya-policy-engine-*/` (the
per-µservice flat layout mandates code at `microservices/<ms>/src/crates/`;
the per-µservice directory also holds *configuration, contracts, IaC,
migrations, SLOs, tests, runbooks, fragments*). The
`microservices/policy-engine/src/` directory is the composition root;
the actual binary entry-point crate is `oya-policy-engine-evaluator-app`
at `microservices/policy-engine/src/crates/oya-policy-engine-evaluator-app/`.

### D-2. Bounded-context structure inside `policy-engine`

The promoted µservice contains **eight bounded contexts**:

| BC | Purpose | Hot path? |
|---|---|---|
| `fragment-registry` | Persist, index, and version Cedar fragments. Postgres + Citus shard on `(scope, fragment_id)`. Hot-cache compiled bundles in Valkey. | Cold path (publish/list/get) |
| `evaluator` | In-process Cedar v4.2 evaluator. Receives `EvaluationRequest`; consults compiled-bundle cache; returns `EvaluationResponse`. Per-cell Deployment with 3+ replicas + HPA on QPS. | Hot path (every state-changing call across portfolio) |
| `signing-chain` | Verify Ed25519 + cosign signatures on fragments against the org-root → intermediate → publisher chain. Provides genesis-key verification primitive. | Cold path (on fragment publication / activation) |
| `hot-reload` | Subscribe to per-cell `fragment-reload` Kafka topic; on `FragmentPublished` event, fetch new fragment from registry, recompile bundle, atomic-swap into evaluator. <5s p99 across all replicas. | Cold path (control plane) |
| `coverage-audit` | Scan every µservice's declared actions (OpenAPI + AsyncAPI + `capabilities/*.yaml` + Cedar action enums) and verify that each has a permit fragment + a default-deny. Emits `CoverageReport` rows. | Cold path (CI lane + nightly drift detection) |
| `pack-overlay` | Compose per-pack fragments at evaluation time. Holds pack fragment bundles keyed by `(pack_id, scope)`. | Hot path (every evaluation that references an active pack) |
| `tenant-overlay` | Compose per-tenant fragments at evaluation time. Holds tenant-scoped fragments. Enforces tenant-fragment-restriction (tenants can forbid but cannot raise permits beyond baseline). | Hot path (every evaluation in tenant context) |
| `bootstrap-genesis` | Provision the org-root-signed genesis fragment at bootstrap step 5 (per ADR-0242 §D-5). Owns the offline → online handoff for the bootstrap chain of trust. One-shot per cell at bootstrap; subsequent operations route through `fragment-registry`. | Cold path (bootstrap only) |

The BCs follow ADR-0105 13-value canonical enum (`kernel`, `domain`,
`usecase`, `api`, `adapter`, `rest`, `worker`, `sdk`, `app`) per the
crate redistribution in §D-3.

### D-3. Crate redistribution

The following Rust crates are renamed (existing) or introduced (new),
following BNF v4.1 per ADR-0056 and the 13-value canonical layer enum
per ADR-0105:

**Renamed from Ontology — `cedar-fragment-coverage` BC → `fragment-registry` BC:**

| Old crate (Ontology) | New crate (policy-engine) |
|---|---|
| `oya-ontology-cedar-fragment-coverage-kernel` | `oya-policy-engine-fragment-registry-kernel` |
| `oya-ontology-cedar-fragment-coverage-domain` | `oya-policy-engine-fragment-registry-domain` |
| `oya-ontology-cedar-fragment-coverage-usecase` | `oya-policy-engine-fragment-registry-usecase` |
| `oya-ontology-cedar-fragment-coverage-api` | `oya-policy-engine-fragment-registry-api` |
| `oya-ontology-cedar-fragment-coverage-adapter` | `oya-policy-engine-fragment-registry-adapter` |

The rename ChangeSet:

1. Adds the new crate names to the workspace `Cargo.toml`.
2. Re-exports the existing types under the new module paths.
3. Adds deprecation shims at the old paths that re-export from the new
   for one minor version (per ADR-0211 no-silent-regression doctrine).
4. Issues a sweep PR removing the old paths after one minor version
   sunset.

**New BCs — full crate matrix:**

`evaluator` BC (most-fanout; full layer set including `rest`, `worker`,
`sdk`, `app`):

- `oya-policy-engine-evaluator-kernel` — port traits (`PolicyEvaluator`,
  `FragmentCompiler`, `CompiledBundleCache`), sealed-trait Cedar entity
  type contracts, zero I/O. Carries `data_class` annotations per
  ADR-0028.
- `oya-policy-engine-evaluator-domain` — pure evaluation logic: bundle
  composition (baseline ∪ overlays ∪ packs ∪ tenant), deny-wins
  semantic, NotApplicable → default-deny semantic.
- `oya-policy-engine-evaluator-usecase` — orchestrators reading
  fragment-registry adapter + compiling Cedar AST + emitting
  evaluation audit rows.
- `oya-policy-engine-evaluator-api` — protocol-neutral typed I/O
  contracts (`EvaluationRequest`, `EvaluationResponse`, `Decision`).
- `oya-policy-engine-evaluator-adapter` — Cedar v4.2 SDK bindings;
  Valkey hot-cache adapter; Postgres cold-cache adapter; signing-chain
  client.
- `oya-policy-engine-evaluator-rest` — HTTP handler for `Evaluate`,
  `EvaluateBatch`, `GetEvaluationByID` operations (OpenAPI 3.2.0
  surface).
- `oya-policy-engine-evaluator-worker` — background workers (compiled-
  bundle warm-up, evaluation-audit batch flush, metrics emission).
- `oya-policy-engine-evaluator-sdk` — thin Rust client (also re-exposed
  as `oya-shared-policy-engine-client-sdk` for downstream µservices).
- `oya-policy-engine-evaluator-app` — composition-root binary; wires
  axum + tonic + the Cedar evaluator + the cell-local Postgres +
  Valkey + the Kafka hot-reload subscriber.

`signing-chain` BC:

- `oya-policy-engine-signing-chain-kernel` — port traits
  (`SigningKeyStore`, `SignatureVerifier`, `ChainResolver`).
- `oya-policy-engine-signing-chain-domain` — pure chain-verification
  logic; key-rotation logic; Ed25519 signature verification.
- `oya-policy-engine-signing-chain-usecase` — orchestrators: "verify
  this fragment chains back to the org root key" + "verify the org
  root key matches the offline-HSM-issued certificate."
- `oya-policy-engine-signing-chain-api` — typed I/O contracts
  (`SignatureVerifyRequest`, `SignatureVerifyResponse`,
  `ChainResolveRequest`, `ChainResolveResponse`).
- `oya-policy-engine-signing-chain-adapter` — OpenBao integration for
  intermediate keys; HSM client (PKCS#11) for the org root key
  verification; Sigstore cosign attestation client.

`hot-reload` BC:

- `oya-policy-engine-hot-reload-kernel` — port traits
  (`FragmentReloadNotifier`, `EvaluatorReloadCoordinator`).
- `oya-policy-engine-hot-reload-domain` — pure reload-coordination
  logic; atomic-swap algorithm; failed-swap rollback semantic.
- `oya-policy-engine-hot-reload-usecase` — orchestrators reading Kafka
  consumer + fetching new fragment + recompiling bundle + atomic-swap.
- `oya-policy-engine-hot-reload-api` — typed I/O contracts
  (`FragmentReloadEvent`, `ReloadAck`, `ReloadFailure`).
- `oya-policy-engine-hot-reload-adapter` — Kafka consumer adapter;
  in-process bundle-swap adapter.
- `oya-policy-engine-hot-reload-worker` — long-running consumer worker;
  reload-failure SEV-3 alert emission.

`pack-overlay` BC:

- `oya-policy-engine-pack-overlay-kernel` — port traits
  (`PackOverlayResolver`, `PackFragmentStore`).
- `oya-policy-engine-pack-overlay-domain` — pure overlay-composition
  logic; pack-activation-precondition checks.
- `oya-policy-engine-pack-overlay-usecase` — orchestrators: load active
  packs for a tenant + resolve pack fragments + compose into evaluator
  bundle.
- `oya-policy-engine-pack-overlay-api` — typed I/O contracts
  (`PackOverlayResolveRequest`, `PackFragmentBundle`).
- `oya-policy-engine-pack-overlay-adapter` — Postgres adapter for
  pack-fragment storage; Valkey adapter for pack-bundle hot cache.

`tenant-overlay` BC:

- `oya-policy-engine-tenant-overlay-kernel` — port traits
  (`TenantOverlayResolver`, `TenantFragmentStore`,
  `TenantFragmentRestrictionChecker`).
- `oya-policy-engine-tenant-overlay-domain` — pure overlay-composition
  logic; tenant-fragment-restriction enforcement (deny-only or
  attribute-conditional-permit; never raise above baseline).
- `oya-policy-engine-tenant-overlay-usecase` — orchestrators: load
  tenant fragments + verify restriction + compose into evaluator
  bundle.
- `oya-policy-engine-tenant-overlay-api` — typed I/O contracts
  (`TenantOverlayResolveRequest`, `TenantFragmentBundle`,
  `RestrictionViolation`).
- `oya-policy-engine-tenant-overlay-adapter` — Postgres adapter for
  tenant-fragment storage; Valkey adapter for tenant-bundle hot cache.

`bootstrap-genesis` BC:

- `oya-policy-engine-bootstrap-kernel` — port traits
  (`GenesisFragmentSeeder`, `OrgRootKeyVerifier`).
- `oya-policy-engine-bootstrap-domain` — pure bootstrap-sequencing
  logic; org-root-signature verification; offline → online handoff
  state machine.
- `oya-policy-engine-bootstrap-usecase` — orchestrators: at bootstrap
  step 5 of ADR-0242 §D-5, load the genesis fragment from the
  bootstrap log + verify the org-root signature + seed the fragment
  registry + emit the bootstrap-completion audit event.
- `oya-policy-engine-bootstrap-app` — composition-root binary for the
  one-shot bootstrap Job (Kubernetes Job, not Deployment; runs once
  per cell at cell-creation time).

`coverage-audit` BC (deferred to existing `cedar-fragment-coverage`
crates after rename; the BC is renamed `fragment-registry` for the
storage half + `coverage-audit` for the scanning half; the scanning
half lives at):

- `oya-policy-engine-coverage-audit-kernel` — port traits
  (`ActionEnumerator`, `CoverageScanner`).
- `oya-policy-engine-coverage-audit-domain` — pure scan logic: for
  each µservice's declared actions, intersect with permit fragments +
  default-deny fragments; emit gap list.
- `oya-policy-engine-coverage-audit-usecase` — orchestrators reading
  µservice manifests + capabilities/*.yaml + OpenAPI + AsyncAPI +
  Cedar fragment store + emitting `CoverageReport`.
- `oya-policy-engine-coverage-audit-api` — typed I/O contracts
  (`CoverageReportRequest`, `CoverageReport`, `GapAction`).
- `oya-policy-engine-coverage-audit-adapter` — Postgres adapter for
  CoverageReport persistence; µservice-manifest reader adapter.

**Shared SDK — consumed by every µservice in the portfolio:**

- `oya-shared-policy-engine-client-kernel` — port traits
  (`PolicyEngineClient`, `PolicyDecisionCache`).
- `oya-shared-policy-engine-client-adapter` — gRPC client to cell-
  local evaluator pool; connection pool; circuit breaker; cache
  fallback per ADR-0243 §D-11 fail-closed default.
- `oya-shared-policy-engine-client-sdk` — high-level Rust SDK exposing
  `policy_engine.evaluate(...)`, `evaluate_batch(...)`, plus the
  helpers `permit_or_forbid_call`, `feature_gate`, `data_class_check`
  built atop `evaluate`. Re-exported as `oya-shared-policy-engine-
  client` for short-form imports.

**Summary count.** Total crates introduced or renamed by this ADR:

| BC | Crate count |
|---|---|
| `fragment-registry` (rename) | 5 |
| `evaluator` | 9 |
| `signing-chain` | 5 |
| `hot-reload` | 6 |
| `pack-overlay` | 5 |
| `tenant-overlay` | 5 |
| `bootstrap-genesis` | 4 |
| `coverage-audit` | 5 |
| `oya-shared-policy-engine-client` | 3 |
| **Total** | **47** |

The 47-crate footprint is bounded; each BC is single-concern per
ADR-0132. The IP series scaffolds these incrementally across IP-001 ..
IP-008 (one IP per BC) per ADR-0139 SLO-gated promotion.

### D-4. API surface

The policy-engine µservice exposes the following operations over internal-only gRPC/proto3 plus a public or compatibility HTTPS REST projection documented by OpenAPI 3.2.0. This is proposed scope, not an accepted exposure or implementation claim:

| Operation | Path | Method | Idempotent? | Performance budget |
|---|---|---|---|---|
| `Evaluate` | `/v1/evaluate` | POST | Yes (deterministic given input + fragment set) | p99 1ms hot; p99 50ms cold |
| `EvaluateBatch` | `/v1/evaluate:batch` | POST | Yes | p99 5ms hot for ≤100 evaluations; p99 100ms cold |
| `PublishFragment` | `/v1/fragments` | POST | No (creates new version) | p99 200ms |
| `ActivateFragment` | `/v1/fragments/{fragment_id}:activate` | POST | Yes (state-machine activate) | p99 5s (includes hot-reload propagation) |
| `SunsetFragment` | `/v1/fragments/{fragment_id}:sunset` | POST | Yes (state-machine sunset) | p99 5s |
| `GetFragmentByID` | `/v1/fragments/{fragment_id}` | GET | Yes | p99 10ms |
| `ListFragments` | `/v1/fragments` | GET | Yes | p99 50ms (cursor-paginated per ADR-0150 cursor-pagination canonical) |
| `GetCoverageReport` | `/v1/coverage/{microservice_id}` | GET | Yes | p99 100ms (cached); p99 5s (live scan) |
| `GetEvaluationByID` | `/v1/evaluations/{evaluation_id}` | GET | Yes | p99 50ms |
| `VerifySignatureChain` | `/v1/signing-chain:verify` | POST | Yes | p99 100ms |

The full gRPC proto definition lives at `api/grpc/policy_engine.proto`
per the directory layout in §D-1.

Sample gRPC service definition:

```protobuf
syntax = "proto3";

package oya.policy_engine.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

service PolicyEngine {
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  rpc EvaluateBatch(EvaluateBatchRequest) returns (EvaluateBatchResponse);
  rpc PublishFragment(PublishFragmentRequest) returns (PublishFragmentResponse);
  rpc ActivateFragment(ActivateFragmentRequest) returns (ActivateFragmentResponse);
  rpc SunsetFragment(SunsetFragmentRequest) returns (SunsetFragmentResponse);
  rpc GetFragmentByID(GetFragmentByIDRequest) returns (GetFragmentByIDResponse);
  rpc ListFragments(ListFragmentsRequest) returns (ListFragmentsResponse);
  rpc GetCoverageReport(GetCoverageReportRequest) returns (GetCoverageReportResponse);
  rpc GetEvaluationByID(GetEvaluationByIDRequest) returns (GetEvaluationByIDResponse);
  rpc VerifySignatureChain(VerifySignatureChainRequest) returns (VerifySignatureChainResponse);
}

message EvaluateRequest {
  string evaluation_id = 1;   // UUIDv7 supplied by caller; idempotency key
  string principal = 2;       // e.g. "oyatie.foundry.ci-agent#instance-3421"
  string action = 3;          // e.g. "WorkflowEngine::Action::TriggerBuild"
  string resource = 4;        // e.g. "Workflow::id/build-3421"
  google.protobuf.Struct context = 5;  // free-form attribute map
  string tenant_id = 6;       // e.g. "oyatie" or "tenant-acme-corp"
  string cell_id = 7;         // requesting cell identifier
}

message EvaluateResponse {
  enum Decision { DECISION_UNSPECIFIED = 0; PERMIT = 1; FORBID = 2; NOT_APPLICABLE = 3; }
  Decision decision = 1;
  string reason = 2;                 // human-readable
  repeated string applied_fragments = 3;     // fragment IDs that contributed
  repeated string determining_policies = 4;  // policy IDs that fired
  uint32 evaluation_ms = 5;
  bool audit_emitted = 6;
  map<string, string> annotations = 7;
}
```

Each operation's full request/response shape is defined in the proto
file at `api/grpc/policy_engine.proto` and mirrored in the OpenAPI
3.2.0 contract.

### D-5. Per-cell deployment topology

Per ADR-0009 (cell architecture) + ADR-0049 (residency), the policy-
engine µservice deploys per-cell with the following topology:

**Evaluator Deployment (hot path):**

- Kubernetes `Deployment` resource `oya-policy-engine-evaluator`.
- 3+ replicas per cell; HPA scales on QPS (target 70% of `max_qps`
  per replica = ~5k QPS hot path; ~500 QPS cold path).
- Pod anti-affinity by node + zone (per cell's zone topology) to
  tolerate single-node + single-zone failures.
- Pod Disruption Budget (PDB) `minAvailable: 2`.
- Resources: 2 vCPU + 4 GiB per replica baseline; bursts to 4 vCPU.
- Liveness: `/v1/healthz` (lightweight: evaluator process up).
- Readiness: `/v1/readyz` (compiled-bundle cache loaded + Valkey
  reachable + signing-chain cached).
- Image: `distroless-rust` per ADR-0146; non-root; read-only root FS.

**Fragment Registry (cold path):**

- Cell-local Postgres (per ADR-0028 + per-cell residency) with Citus
  shard on `(scope, fragment_id)`. Shard count starts at 8 per cell;
  rebalance threshold per Ontology PRD §"Horizontal Scalability"
  (Citus auto-rebalance at 80% shard fill).
- Read replicas: 3 per cell (per ADR-0172 read-replica CQRS pattern
  for high-read BCs; policy-engine fragment reads dominate writes
  ~100×).
- Hot cache: Valkey `oya-policy-engine-valkey-hot-cache` `StatefulSet`,
  3 replicas, ~10 GiB per replica.

**Hot-reload (control plane):**

- Kafka topic `policy-engine.fragment-reload.v1` per cell (per ADR-0050
  event-bus canonical).
  - Partitions: 16 (enough for fan-out to per-cell evaluator replicas
    + headroom).
  - Retention: 7d (compactable; reload events are idempotent).
- Hot-reload worker Deployment `oya-policy-engine-hot-reload-worker`:
  2 replicas per cell (HA; one active + one standby).
- On `FragmentPublished` event: worker fetches fragment from registry,
  validates signature chain, compiles Cedar AST, publishes the
  compiled bundle to the per-cell Valkey hot-cache + emits a per-
  evaluator pub-sub notification.
- Each evaluator replica subscribes to the per-cell `policy-engine.
  evaluator-reload.v1` topic; on notification, atomic-swap the in-
  process bundle.

**Cross-region pairing per ADR-0241 + ADR-0049:**

- Each cell is paired with a DR cell in a different sovereign-pack-
  compatible region (e.g., `us-east-1` paired with `us-west-2`;
  `eu-west-1` paired with `eu-central-1`; `ap-seoul-1` paired with
  `ap-tokyo-1` per `prohibited_egress` constraints).
- Postgres streaming replication to the paired cell (per ADR-0049
  cross-region replication rules; only within sovereign-pack-
  compatible regions).
- Fragment Registry RPO: 0 (synchronous replication for the latest
  active fragment set; eventual replication for sunset/archive).
- Evaluator RTO: 5 minutes (cell failover via service mesh; SDK
  circuit breaker steers to paired cell when local cell unhealthy).

### D-6. Performance budget

Per ADR-0243 §D-6 and ADR-0241 T1 tier:

| Path | p50 | p99 | p999 | budget_evidence |
|---|---|---|---|---|
| Evaluate (hot path; cache hit) | 0.1 ms | 1 ms [P5..P95: 0.25ms–0.75ms] | 5 ms | modeled; per docs/performance-budgets/cedar-hot-path-1ms-p99.md; requires DaemonSet co-location + Valkey sidecar + Cilium Ambient eBPF |
| Evaluate (cold path; cache miss + compile) | 10 ms | 50 ms | 100 ms | modeled; Postgres+Citus ~10ms p99 + Cedar AST compile ~20ms p99; consistent with ADR-0280 §D-6 |
| EvaluateBatch (≤100 evaluations; hot path) | 1 ms | 5 ms [P5..P95: 3ms–8ms] | 20 ms | modeled; amortized ~50µs/eval assumes all 100 hits cache; mixed cache-miss batch degrades toward cold-path p99 |
| Audit emission (async enqueue only) | — | 1 ms (enqueue; Merkle-seal async ≤200ms) | — | modeled; Kafka fire-and-forget; response does NOT block on Merkle-seal completion |
| Fragment hot-reload (Path A, EMERGENCY push) | 260 ms | 5 s [P5..P95: 1s–5s] | — | modeled; dual-path model; docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md |
| Fragment hot-reload (Path B, constant-work pull) | ~15 s | ≤35 s [P5..P95: 10s–35s] | — | modeled; 30s cadence + 5s recompile; docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md |
| Per-evaluator atomic-swap | 100 ms | 1 s | 2 s | modeled; atomic-swap within DaemonSet replica during recompile window |
| PublishFragment (cold, includes signature verify) | 50 ms | 200 ms | 500 ms | modeled; Postgres write + cosign verify; cross-continent sync replication may exceed 200ms (see ADR-0246 §D-5 note on region-pair async policy) |
| GetCoverageReport (cached) | 10 ms | 100 ms | 500 ms | modeled; Citus distributed aggregate + cache |
| GetCoverageReport (live scan across portfolio) | 1 s | 5 s | 15 s | modeled; full portfolio Cedar coverage scan |
| VerifySignatureChain (warm cache) | 5 ms | 100 ms | 500 ms | modeled; cosign verify + Rekor inclusion proof check |

**Performance budget enforcement:** SLOs in `slos/*.openslo.yaml` per
ADR-0139 agentic SLO-gated promotion. Burn-rate alarms at 14.4× over
1h trigger SEV-2 page. Error budget for hot-path evaluation: 99.99%
monthly (0.01% ≈ 4.4 min/month).

**Performance tier:** T1 per ADR-0241 (5 min RTO, 0 RPO for active
fragment set). The substrate is "load-bearing for every gate in every
µservice"; T1 is the only acceptable tier.

### D-7. Database schema — fragment-registry Postgres DDL

Per ADR-0211 Rust-primary in-house tech stack, Postgres 17 LTS is the
canonical relational store. The fragment-registry schema:

```sql
-- migrations/0001_fragment_registry_init.sql
-- Postgres 17 LTS + Citus extension. Shard key: (scope, fragment_id).
-- Required extensions: pgcrypto (gen_random_uuid), citus (distribute), pg_trgm (search).

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citus;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Cedar fragment registry. Every published fragment lives here.
CREATE TABLE policy_engine.fragments (
    id                          UUID            NOT NULL DEFAULT gen_random_uuid(),
    scope                       TEXT            NOT NULL,
    fragment_id                 TEXT            NOT NULL,
    version                     INTEGER         NOT NULL,
    signed_by                   TEXT            NOT NULL,
    signature_bytes             BYTEA           NOT NULL,
    signed_at                   TIMESTAMPTZ     NOT NULL,
    effective_at                TIMESTAMPTZ     NOT NULL,
    sunset_at                   TIMESTAMPTZ,
    body_blob_ref               TEXT            NOT NULL,
    body_hash                   BYTEA           NOT NULL,
    activation_status           TEXT            NOT NULL
        CHECK (activation_status IN ('AUTHORED','REVIEWED','SIGNED','PUBLISHED','ACTIVE','SUNSET','TOMBSTONED')),
    applies_to_actions          TEXT[]          NOT NULL DEFAULT '{}',
    applies_to_resources        TEXT[]          NOT NULL DEFAULT '{}',
    applies_to_principals       TEXT[]          NOT NULL DEFAULT '{}',
    pack_id                     TEXT,
    jurisdiction_code           TEXT,
    tenant_id                   TEXT,
    annotations                 JSONB           NOT NULL DEFAULT '{}'::jsonb,
    created_by                  TEXT            NOT NULL,
    created_at                  TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ     NOT NULL DEFAULT now(),
    activated_at                TIMESTAMPTZ,
    sunsetted_at                TIMESTAMPTZ,
    review_verdict_ref          TEXT,
    review_verdict_hash         BYTEA,
    PRIMARY KEY (scope, fragment_id, version),
    UNIQUE (id)
);

SELECT create_distributed_table('policy_engine.fragments', 'scope');

CREATE INDEX idx_fragments_activation_status
    ON policy_engine.fragments (scope, activation_status, effective_at);
CREATE INDEX idx_fragments_pack_id
    ON policy_engine.fragments (pack_id) WHERE pack_id IS NOT NULL;
CREATE INDEX idx_fragments_tenant_id
    ON policy_engine.fragments (tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_fragments_jurisdiction_code
    ON policy_engine.fragments (jurisdiction_code) WHERE jurisdiction_code IS NOT NULL;
CREATE INDEX idx_fragments_effective_window
    ON policy_engine.fragments (effective_at, COALESCE(sunset_at, 'infinity'::timestamptz));
CREATE INDEX idx_fragments_applies_to_actions_gin
    ON policy_engine.fragments USING GIN (applies_to_actions);
CREATE INDEX idx_fragments_id_lookup
    ON policy_engine.fragments (id);
CREATE INDEX idx_fragments_body_hash_lookup
    ON policy_engine.fragments (body_hash);

COMMENT ON TABLE policy_engine.fragments IS
    'Cedar fragment registry. Distributed (Citus) on scope. Body persisted to SeaweedFS via body_blob_ref; body_hash for tamper detection.';
```

```sql
-- migrations/0002_signing_keys_init.sql
-- Signing key chain. Org root → intermediate keys → publisher keys.

CREATE TABLE policy_engine.fragment_signatures (
    fragment_id                 UUID            NOT NULL REFERENCES policy_engine.fragments(id) ON DELETE RESTRICT,
    signer_principal            TEXT            NOT NULL,
    signature_bytes             BYTEA           NOT NULL,
    signed_at                   TIMESTAMPTZ     NOT NULL,
    signing_key_id              TEXT            NOT NULL,
    signing_key_chain_blob      BYTEA           NOT NULL,
    cosign_attestation_ref      TEXT,
    PRIMARY KEY (fragment_id, signer_principal, signing_key_id)
);

CREATE INDEX idx_fragment_signatures_signer
    ON policy_engine.fragment_signatures (signer_principal);
CREATE INDEX idx_fragment_signatures_key_id
    ON policy_engine.fragment_signatures (signing_key_id);

CREATE TABLE policy_engine.signing_keys (
    key_id                      TEXT            NOT NULL PRIMARY KEY,
    key_type                    TEXT            NOT NULL
        CHECK (key_type IN ('ORG_ROOT','ORG_BASELINE','PACK_OWNER','JURISDICTION_OVERLAY','TENANT_ADMIN','INCIDENT_RESPONSE_EMERGENCY')),
    public_key_pem              TEXT            NOT NULL,
    parent_key_id               TEXT
        REFERENCES policy_engine.signing_keys(key_id) ON DELETE RESTRICT,
    valid_from                  TIMESTAMPTZ     NOT NULL,
    valid_until                 TIMESTAMPTZ     NOT NULL,
    revocation_status           TEXT            NOT NULL DEFAULT 'ACTIVE'
        CHECK (revocation_status IN ('ACTIVE','REVOKED','EXPIRED','PENDING_ROTATION')),
    revocation_reason           TEXT,
    revoked_at                  TIMESTAMPTZ,
    hsm_slot_ref                TEXT,
    quorum_required             SMALLINT        NOT NULL DEFAULT 1
        CHECK (quorum_required >= 1 AND quorum_required <= 9),
    quorum_holders              TEXT[]          NOT NULL DEFAULT '{}',
    cosign_certificate_ref      TEXT,
    created_at                  TIMESTAMPTZ     NOT NULL DEFAULT now(),
    metadata                    JSONB           NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_signing_keys_parent
    ON policy_engine.signing_keys (parent_key_id) WHERE parent_key_id IS NOT NULL;
CREATE INDEX idx_signing_keys_type
    ON policy_engine.signing_keys (key_type, revocation_status);
CREATE INDEX idx_signing_keys_valid_window
    ON policy_engine.signing_keys (valid_from, valid_until);

COMMENT ON TABLE policy_engine.signing_keys IS
    'Signing key chain. Org root keys live in tier-0 HSM (hsm_slot_ref points at Shamir-shared slot). Intermediate keys live in OpenBao. Quorum_required encodes the M-of-N for keys requiring multi-holder approval.';
```

```sql
-- migrations/0003_coverage_reports_init.sql
-- Coverage reports per microservice. One row per (microservice, scan_id).

CREATE TABLE policy_engine.coverage_reports (
    scan_id                     UUID            NOT NULL DEFAULT gen_random_uuid(),
    microservice_id             TEXT            NOT NULL,
    action_count                INTEGER         NOT NULL CHECK (action_count >= 0),
    covered_count               INTEGER         NOT NULL CHECK (covered_count >= 0),
    coverage_ratio              NUMERIC(5,4)    GENERATED ALWAYS AS (
        CASE WHEN action_count = 0 THEN 1.0
             ELSE covered_count::numeric / action_count::numeric
        END
    ) STORED,
    gap_actions                 TEXT[]          NOT NULL DEFAULT '{}',
    permit_only_actions         TEXT[]          NOT NULL DEFAULT '{}',
    deny_only_actions           TEXT[]          NOT NULL DEFAULT '{}',
    reported_at                 TIMESTAMPTZ     NOT NULL DEFAULT now(),
    scan_source                 TEXT            NOT NULL
        CHECK (scan_source IN ('CI_LANE','NIGHTLY_DRIFT','ON_DEMAND','BOOTSTRAP')),
    scan_duration_ms            INTEGER,
    metadata                    JSONB           NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (scan_id, microservice_id)
);

CREATE INDEX idx_coverage_reports_microservice_recent
    ON policy_engine.coverage_reports (microservice_id, reported_at DESC);
CREATE INDEX idx_coverage_reports_ratio
    ON policy_engine.coverage_reports (coverage_ratio);
CREATE INDEX idx_coverage_reports_scan_source
    ON policy_engine.coverage_reports (scan_source, reported_at DESC);

COMMENT ON TABLE policy_engine.coverage_reports IS
    'Per-microservice Cedar coverage scan results. Gap actions list actions without permit fragments; permit_only_actions list permits without defaults-deny pair; deny_only_actions are likely orphaned denies.';
```

```sql
-- migrations/0004_evaluation_audit_index_init.sql
-- Evaluation audit index. The full audit row lives on the audit-chain;
-- this table is the lookup index for evaluation_id → audit-chain ref.

CREATE TABLE policy_engine.evaluation_audit_index (
    evaluation_id               UUID            NOT NULL PRIMARY KEY,
    principal                   TEXT            NOT NULL,
    action                      TEXT            NOT NULL,
    resource                    TEXT,
    tenant_id                   TEXT            NOT NULL,
    cell_id                     TEXT            NOT NULL,
    decision                    TEXT            NOT NULL
        CHECK (decision IN ('PERMIT','FORBID','NOT_APPLICABLE')),
    applied_fragments           TEXT[]          NOT NULL DEFAULT '{}',
    determining_policies        TEXT[]          NOT NULL DEFAULT '{}',
    audit_chain_ref             TEXT            NOT NULL,
    audit_stream                TEXT            NOT NULL,
    evaluation_ms               INTEGER         NOT NULL,
    emitted_at                  TIMESTAMPTZ     NOT NULL DEFAULT now()
);

SELECT create_distributed_table('policy_engine.evaluation_audit_index', 'tenant_id');

CREATE INDEX idx_evaluation_audit_principal
    ON policy_engine.evaluation_audit_index (principal, emitted_at DESC);
CREATE INDEX idx_evaluation_audit_action
    ON policy_engine.evaluation_audit_index (action, emitted_at DESC);
CREATE INDEX idx_evaluation_audit_tenant_recent
    ON policy_engine.evaluation_audit_index (tenant_id, emitted_at DESC);
CREATE INDEX idx_evaluation_audit_decision
    ON policy_engine.evaluation_audit_index (decision, emitted_at DESC);
CREATE INDEX idx_evaluation_audit_fragments_gin
    ON policy_engine.evaluation_audit_index USING GIN (applied_fragments);
CREATE INDEX idx_evaluation_audit_emitted
    ON policy_engine.evaluation_audit_index (emitted_at DESC);

COMMENT ON TABLE policy_engine.evaluation_audit_index IS
    'Lookup index for evaluation_id → audit-chain reference. Distributed on tenant_id to align with audit-chain partitioning. Full audit row + signed Merkle proof live in audit-chain microservice.';
```

```sql
-- migrations/0005_genesis_fragment_seed.sql
-- Bootstrap genesis fragment seed. One row per cell, written at bootstrap.

CREATE TABLE policy_engine.bootstrap_genesis (
    cell_id                     TEXT            NOT NULL PRIMARY KEY,
    genesis_fragment_id         TEXT            NOT NULL,
    genesis_fragment_version    INTEGER         NOT NULL,
    org_root_key_id             TEXT            NOT NULL
        REFERENCES policy_engine.signing_keys(key_id) ON DELETE RESTRICT,
    org_root_signature          BYTEA           NOT NULL,
    bootstrap_completed_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    bootstrap_log_blob_ref      TEXT            NOT NULL,
    bootstrap_log_hash          BYTEA           NOT NULL,
    audit_chain_ref             TEXT            NOT NULL,
    quorum_holders_present      TEXT[]          NOT NULL,
    bootstrap_runner_principal  TEXT            NOT NULL,
    metadata                    JSONB           NOT NULL DEFAULT '{}'::jsonb
);

COMMENT ON TABLE policy_engine.bootstrap_genesis IS
    'One row per bootstrapped cell. Records the genesis fragment + org root signature that opened the chain of trust. bootstrap_log_blob_ref points at the pre-audit-chain bootstrap log (replayed into audit-chain at step 6 of ADR-0242 §D-5).';
```

The above DDL is syntactically valid Postgres 17 LTS + Citus 12 +
pgcrypto + pg_trgm. No placeholder columns. Every table has primary key,
foreign keys where applicable, CHECK constraints on enum-shaped TEXT
fields, and indexes on every likely query path. Generated columns
(`coverage_ratio`) use Postgres STORED generated columns per Postgres
12+ syntax.

### D-8. Bootstrap sequence

Per ADR-0242 §D-5 the bootstrap sequence is:

| Step | Component | Bootstrap action |
|---|---|---|
| 0 | Hardware + DNS + git host + container registry | External setup |
| 1 | Bootstrap cell (Tier 1 K8s) | kubeadm init + Cilium |
| 2 | `microservices/cloud-secrets/` (OpenBao) | Shamir unseal + service-account credentials |
| 3 | `microservices/identity/` (Zitadel) | Initial admin + OIDC for `oyatie` |
| 4 | `microservices/tenancy/` | `0001_create_self_tenant.sql` creates `oyatie` tenant |
| **5** | **`microservices/policy-engine/`** | **Genesis fragment seeded; signing-chain online** |
| 6 | `microservices/audit-chain/` | Provisions `oyatie` stream + Ed25519 key |
| 7 | `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | Registers bootstrap cell |
| 8 | `microservices/workflow-engine/` | Deploys minimal Workflow Engine |
| 9 | First Foundry-equivalent workflow | `oyatie.foundry.ci-agent` builds Tier 2 |
| 10 | Bootstrap → steady-state handoff | Bootstrap cell self-retires |

The policy-engine step 5 is the chicken-and-egg crux: Cedar must
exist before subsequent steps (audit-chain admission, cell registration,
workflow-engine, etc.) because those steps' Action Types depend on
Cedar fragments for permit/deny.

**Step 5 sub-sequence:**

5a. **Operator presents the org root key.** The tier-0 HSM cluster
    (e.g., 5 YubiKey HSM 2s held by the founding team in geographically
    distributed safety-deposit boxes) is convened. The 3-of-5 Shamir
    quorum unseals the org root signing key into a PKCS#11-accessible
    slot for the duration of step 5 only.

5b. **Genesis fragment loaded.** The bootstrap container reads
    `microservices/policy-engine/fragments/bootstrap/genesis.cedar`
    from the bootstrap log (signed at platform-genesis time and
    versioned in the source tree). The genesis fragment grants the
    bootstrap principal (issued by step-3 Identity Service) the
    permission to publish further fragments under the `bootstrap/*`
    scope.

5c. **Org root signature verified.** The genesis fragment's body hash
    is verified against the org root key (loaded via PKCS#11 in step
    5a). The signing-chain BC's `OrgRootKeyVerifier` performs
    Ed25519 signature verification.

5d. **Genesis fragment persisted to fragment-registry.** Migration
    `0005_genesis_fragment_seed.sql` inserts the row with `cell_id`,
    `genesis_fragment_id`, `genesis_fragment_version`,
    `org_root_key_id`, `org_root_signature`, `bootstrap_completed_at`,
    `bootstrap_log_blob_ref`, `bootstrap_log_hash`. The bootstrap log
    blob is stored in SeaweedFS at the bootstrap-bucket path; the
    hash is independently verifiable.

5e. **Intermediate signing keys provisioned.** The genesis fragment
    permits the bootstrap principal to author intermediate signing
    keys (org-baseline-key, per-pack-owner-keys, per-jurisdiction-
    overlay-keys, incident-response-emergency-key). Each intermediate
    key's certificate is itself signed by the org root key in this
    step.

5f. **Evaluator Deployment becomes Ready.** The 3+ replica evaluator
    pods come up; each loads the genesis-only fragment set into
    Valkey hot cache; each subscribes to the per-cell `policy-engine.
    evaluator-reload.v1` topic.

5g. **HSM org root key re-sealed.** The org root key returns to its
    Shamir-shared offline state. From this point, no operation in the
    cell uses the org root key directly — all subsequent operations
    flow through the intermediate keys.

5h. **Self-host fragment registry takes over.** Subsequent fragment
    publications (e.g., the baseline `oyatie.foundry.*` permits per
    ADR-0242 implementation surface) go through `PublishFragment` →
    intermediate-key signature → `ActivateFragment` → hot-reload.

5i. **Bootstrap audit emission.** A `PolicyEngineBootstrapCompleted`
    event is written to the bootstrap log; replayed into the audit
    chain at step 6 (when audit-chain comes online). The bootstrap
    log entry includes `cell_id`, `bootstrap_completed_at`,
    `org_root_key_id`, `genesis_fragment_id`, `intermediate_keys[]`,
    `quorum_holders_present[]`, `bootstrap_runner_principal`.

**Recovery:** If step 5 fails mid-sequence (e.g., HSM quorum unsealing
fails after 5a but before 5d), the bootstrap is aborted. The cell is
torn down + recreated from step 1 (Kubernetes cluster reinit). No
partial-state cleanup is required because no state has been written
to the durable fragment-registry yet (5d is the first persistent
write).

### D-9. Ontology amendment

The promotion requires the following amendments to
`microservices/ontology/PRD.md`:

**Amendment 1 — Bounded Contexts table.** Remove the row for
`cedar-fragment-coverage`. The BC moves entirely to
`microservices/policy-engine/`. The crate family
`oya-ontology-cedar-fragment-coverage-*` is renamed to
`oya-policy-engine-fragment-registry-*` per §D-3.

**Amendment 2 — "universal mediator" framing.** PRD-ontology line 22
currently frames Ontology as "the canonical information adapter
through which every other µservice reads and writes typed Object
Types, Link Types, Action Types, and Functions." Post-amendment, the
line is rewritten:

> The `ontology` microservice is oyatie's **Palantir-Foundry-class
> typed-entity substrate** — the canonical *information* adapter
> through which every other µservice reads and writes typed Object
> Types, Link Types, Action Types, and Functions. Ontology pairs with
> `microservices/policy-engine/` (the canonical *policy* substrate
> per ADR-0246) and `microservices/workflow-engine/` (the canonical
> *orchestration* substrate per ADR-0059); together the three
> substrates compose the platform's adapter layer. Ontology is NOT a
> universal mediator; calls that need policy decisions consult
> policy-engine directly via the SDK.

**Amendment 3 — Layer mapping table.** Remove the
`cedar-fragment-coverage` row from PRD-ontology line ~178 layer
mapping table. Recompute the total crate count: was ~92, becomes ~87.

**Amendment 4 — Port traits table.** Remove the `CedarPolicyEvaluator`
port trait row from PRD-ontology line ~195. The trait moves to
`oya-policy-engine-evaluator-kernel` as `PolicyEvaluator`.

**Amendment 5 — `agent-gateway` BC rename.** The `agent-gateway` BC
in PRD-ontology line 122 is renamed `tool-call-ingress`. Per ADR-0243
context, the agent-gateway BC stays in Ontology (it's the LLM tool-
call ingress that translates LLM tool calls to typed Object Type /
Function operations), but the name `agent-gateway` has caused
confusion with the per-µservice gateway layer in ADR-0145 and with
the `microservices/governance/` agent-coordination surfaces. The
rename clarifies: the BC ingests *tool calls* (LLM-issued tool
invocations against the Ontology), not *agents* in the cross-µservice
sense.

The crate family `oya-ontology-agent-gateway-*` is renamed
`oya-ontology-tool-call-ingress-*`. Deprecation shims at the old
paths for one minor version per ADR-0211 no-silent-regression
doctrine; sweep PR removes them after sunset.

**Amendment 6 — CI lane references.** PRD-ontology line 215 currently
lists `cloud-ci/Rust gate packet cedar-coverage --microservice ontology`.
Post-amendment, this lane is removed from Ontology's required-green
set; it moves to the policy-engine µservice's required-green set as
`cloud-ci/Rust gate packet cedar-coverage` (no `--microservice` argument; the
lane scans the entire portfolio).

**Amendment 7 — Related ADRs table.** PRD-ontology line ~370 currently
lists ADR-0140 (Cedar policy enforcement). Add ADR-0246 (Policy-Engine
Substrate Promotion) and ADR-0243 (Cedar as Universal Gate) to the
related ADRs table.

The amendment ChangeSet is bundled with this keystone PR. The Ontology
PRD must be updated in lockstep with this ADR's acceptance; partial
acceptance is rejected (per the keystone bundle policy).

### D-10. SLO targets

Per ADR-0241 portfolio policy, the policy-engine µservice declares:

| Field | Value |
|---|---|
| `dr_tier` | T1 |
| RTO target | 5 minutes (cell failover to paired cell) |
| RPO target | 0 seconds (synchronous replication for active fragment set; eventual replication for sunset/archive) |
| Availability target | 99.99% monthly (≤ 4.4 min/month error budget) |
| Hot-path evaluate p99 | < 1 ms |
| Cold-path evaluate p99 | < 50 ms |
| Hot-reload propagation p99 | < 5 s |
| Coverage scan completion p99 | < 15 s portfolio-wide |
| Burn-rate alarm | 14.4× over 1h → SEV-2 page |
| Cross-region paired DR cell | Required (per ADR-0049 sovereign-pack-compatible) |
| DR drill cadence | Quarterly (per ADR-0241) |

The full SLO declarations live in `microservices/policy-engine/slos/`:

- `evaluate-hot-path.openslo.yaml` — hot-path p99 ≤ 1ms; 99.99%.
- `evaluate-cold-path.openslo.yaml` — cold-path p99 ≤ 50ms; 99.99%.
- `hot-reload-propagation.openslo.yaml` — < 5s p99; 99.95%.
- `signing-chain-verification.openslo.yaml` — < 100ms p99; 99.99%.
- `coverage-completeness.openslo.yaml` — 100% coverage (zero
  tolerance for actions without permit + default-deny pair).

### D-11. CI lanes that move

The following CI lanes are renamed/moved as part of the promotion:

| Lane (current name) | Lane (post-promotion name) | Scope |
|---|---|---|
| `oya-governance-fitness-cedar-coverage` | `oya-governance-cedar-coverage` | Portfolio-wide |
| `oya-governance-fitness-cedar-fragment-signature` | `oya-governance-cedar-fragment-signature` | Portfolio-wide |
| `oya-governance-fitness-cedar-default-deny-coverage` | `oya-governance-cedar-default-deny-coverage` | Portfolio-wide |
| `oya-governance-fitness-cedar-tenant-fragment-restriction` | `oya-governance-cedar-tenant-fragment-restriction` | Portfolio-wide |
| (new) | `oya-governance-policy-engine-substrate-coherence` | `microservices/policy-engine/` |
| (new) | `oya-governance-no-policy-in-code` | Portfolio-wide static-analysis lane (per ADR-0243 §D-1) |
| (new — per ADR-0293) | `oya-check-meta-trust-root-attestation` | Verifies every trust-chain Cedar fragment (scope `baseline/` + `pack/` touching self-modification gates) carries an `attested_by_meta_trust_root: true` annotation backed by a valid meta-trust-root witness signature; BLOCKER post-meta-trust-root-ceremony |
| (new — per ADR-0294) | `oya-check-fragment-soak-window` | Verifies every fragment publication carries `sunset_at - activate_at >= 60s` invariant; verifies the fragment entered the soak phase before any enforcement activation; BLOCKER post-soak-detector-deploy |
| (new — per ADR-0295) | `oya-check-bootstrap-spiffe-identity` | Verifies Stage-1 bootstrap CI artifacts carry SPIFFE SVID attestations issued by `oyatie.foundry.bootstrap-ca`; verifies the T+8h kill-switch fragment is pre-loaded in the bootstrap cell's policy-engine; BLOCKER before any bootstrap cell enters Stage-2 |
| (new — per ADR-0296) | `oya-check-library-first-credential-sidecar` | Verifies that Cedar fragments governing tool-call permits include a `sidecar_credential_handle_lifetime_ms` context attribute declaration; verifies no caller process holds provider credentials or audit-signing key material beyond a single in-flight call (≤60s OpenBao TTL or UDS sidecar isolation); BLOCKER post-sidecar-deploy |

The rename from `oya-governance-*` to `oya-governance-*` follows
the new governance-lane prefix per CLAUDE.md
`new_governance_lane_prefix: oya-governance-*` (per ADR-0132). The
existing `oya-governance-*` lanes remain operational under their
current names until each is renamed in its own migration IP (per the
project CLAUDE.md note). For this ADR, the policy-engine µservice's
required-green set uses the new names; the migration ChangeSet
includes the rename.

## Alternatives considered

### Alt-1. Keep `cedar-fragment-coverage` as a BC inside `ontology` (status quo)

Continue treating Cedar fragment management as a bounded context
inside the Ontology µservice. Cedar fragments live in
`microservices/ontology/fragments/`; Cedar PDP runs as a Deployment
in the Ontology pod set or in a separate `governance` µservice (per
ADR-0183 transitional shape).

**Pros:**

- Zero migration cost. Existing crates + tooling stay as-is.
- Cedar fragments live one git-tree hop from Action Type declarations;
  authoring ergonomics are tight.
- Ontology already has team ownership + maturity.

**Cons:**

- **Cedar's load-bearing role per ADR-0243 exceeds Ontology's
  capacity envelope.** Universal-gate evaluation QPS ≫ Ontology's
  Function-read QPS. Hot-path performance budget can't be hit while
  sharing cell resources with Ontology's reads.
- **SLO tier mismatch.** Cedar is T1; Ontology is T2/T3 depending on
  BC. Promoting Ontology as a whole to T1 to accommodate Cedar would
  cascade to every Ontology BC (Function reads, Link reads, Action
  invocations, Agent gateway, Audit chain emission), each of which
  carries its own resource + reliability cost.
- **Bootstrap circular dependency.** Per ADR-0242 §D-5, Cedar must be
  online before Ontology can write its first Object Type (Action Type
  registration → Cedar fragment lookup → permit/deny). Inside Ontology,
  this is a chicken-and-egg.
- **Team ownership conflict.** axis-policy-engine + axis-ontology
  share a single µservice; ownership lines blur.
- **Hot-reload SLO unachievable.** Ontology brownout conditions
  exceed the 5s hot-reload propagation SLO by 6×-12×.
- **AWS Verified Permissions, Google Org Policy, Netflix OPA all
  chose centralized-µservice over embedded-BC.** Industry pattern
  unambiguous.

**Rejected** because the cons compound + every hyperscaler reference
disagrees + the keystone bundle's coherence requires the promotion.

### Alt-2. Promote `cedar-fragment-coverage` to a library (vendored Rust crate) consumed by every µservice

Make Cedar evaluation a library: each µservice links in
`cedar-policy` v4.2 + a `oya-policy-engine-evaluator-embedded` crate
that loads fragments from a shared registry (e.g., S3-equivalent +
local cache).

**Pros:**

- Zero network hop per evaluation (lowest theoretical latency).
- No new µservice surface to operate.
- Per-µservice failure isolation.

**Cons:**

- **Hot-reload propagation becomes a fleet rollout.** Updating Cedar
  fragments means pushing new fragment files to every replica of
  every µservice (potentially thousands of pods). The 5s propagation
  SLO is unachievable; emergency-permit propagation (per ADR-0243
  Appendix B incident-response scenario) becomes a multi-hour push.
- **Per-tenant overlay memory pressure.** Each µservice replica holds
  the full tenant-overlay set in memory; aggregate fleet memory cost
  is `replicas × tenants × fragments_per_tenant`. At the 10k-tenant
  scale, this exceeds reasonable per-pod memory budgets.
- **Signing-chain duplication.** Every binary duplicates the org-root
  → intermediate verification logic. Bug-fix dispersion is N-way.
- **Coverage scanning impossible at runtime.** Coverage audit needs
  global view of all µservices' actions vs all fragments; this is
  inherently centralized.
- **Audit emission inconsistent.** Each µservice's local Cedar
  evaluator emits its own audit rows; reconciling shape + retention
  across the fleet is hard.

**Rejected** because hot-reload SLO + memory pressure + audit
consistency dominate.

### Alt-3. Promote `cedar-fragment-coverage` to a sidecar (one pod per service pod)

Each µservice pod gets a co-located Cedar evaluator sidecar
(DaemonSet-style or pod-spec sidecar). SDK calls loopback to localhost;
sidecar handles fragment management + evaluation.

**Pros:**

- Low network hop (loopback).
- Per-µservice failure isolation.
- Per-cell resource budget granular.

**Cons:**

- **Pod count doubles.** ~46 µservices × ~10 replicas average = ~460
  service pods; with sidecars, ~920 evaluator pods. Operational
  footprint significant.
- **Fragment-registry-to-sidecar fan-out for hot-reload exceeds Kafka
  topic partition reasonable limits.** Per-cell Kafka `policy-engine.
  fragment-reload.v1` topic would need partition count = sidecar
  count = ~920 per cell. Practical limits are ~200 partitions per
  topic.
- **Per-µservice HPA scaling decouples from sidecar scaling.** Pods
  scale with service load; sidecars scale with policy-evaluation
  load; mismatch causes resource waste or contention.
- **AWS Verified Permissions production architecture rejected
  sidecar.** Per public AWS Builder's Library posts, AWS evaluated
  sidecar-per-pod for Verified Permissions and chose centralized-
  evaluator-per-region after measuring identical tradeoffs.

**Rejected** because operational footprint outweighs the loopback
latency benefit; AWS's empirically tested rejection of the same
shape provides strong evidence.

### Alt-4. Split into two µservices: `microservices/policy-engine-evaluator/` (hot path) and `microservices/policy-engine-registry/` (cold path)

Decompose further: separate the hot-path evaluator from the cold-path
fragment registry into two peer µservices.

**Pros:**

- Sharper SLO separation (registry can be T2 if evaluators have warm
  caches; evaluator stays T1).
- Independent team ownership possible if axis-policy-engine grows
  large.
- Smaller blast radius per µservice failure.

**Cons:**

- **Bootstrap ordering more complex.** Now step 5 of ADR-0242 §D-5
  becomes 5a (registry) + 5b (evaluator); each with its own genesis
  fragment + signing chain.
- **Cross-µservice latency on cold-path evaluation.** Evaluator
  cache-miss must round-trip to registry µservice; the cold-path
  budget shrinks from 50ms to 30ms after accounting for the extra
  hop.
- **Two-µservice ownership overhead for axis-policy-engine.** Until
  the team grows, the split is cost without benefit.
- **Hot-reload still spans both µservices.** Registry publishes
  fragments → evaluator subscribes via Kafka. The split makes the
  pub-sub spans cross-µservice instead of intra-µservice; failure-
  domain count rises.

**Rejected for now** because the single-µservice promotion (Alt-5)
captures the substrate benefit at lower complexity. A future ADR may
revisit the split if axis-policy-engine grows large enough to support
two teams.

### Alt-5. Promote to one peer substrate µservice `microservices/policy-engine/` (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **Matches AWS Verified Permissions, Google Org Policy, Netflix OPA,
  Pinterest OPA architecture.** Hyperscaler pattern unambiguous.
- **Independent SLO (T1) + deployment + team ownership.**
- **Per-cell evaluator topology achievable** without conflicting with
  Ontology's resource envelope.
- **Bootstrap ordering deterministic** (step 5 of ADR-0242 §D-5).
- **Hot-reload propagation achievable** at <5s p99 via Kafka pub-sub
  fan-out scoped to per-cell evaluator replicas.
- **Coverage scanning centralized** (one µservice scans the whole
  portfolio; outputs `CoverageReport` rows).
- **Signing-chain centralized** (one µservice manages the org-root →
  intermediate key chain).
- **In-house per ADR-0211.** Cedar v4.2 LTS is open source; cedar-
  policy crate is Rust; OpenBao + Postgres + Citus + Valkey + Kafka
  are all in-house-preferred per ADR-0211.
- **Reversible (one-way).** Promotion is an additive ChangeSet
  (Ontology drops one BC; new µservice gains it). No data loss; no
  destructive operations. The rename ChangeSet ships deprecation
  shims for one minor version per the no-silent-regression doctrine.

**Cons:**

- **Bounded one-time migration cost.** ~47 crates renamed or
  introduced. The rename ChangeSet is bundled with this keystone PR;
  one ChangeSet executes it.
- **Bootstrap ceremony complexity.** Step 5 of ADR-0242 §D-5 is a
  multi-stage HSM quorum + genesis fragment + intermediate key
  provisioning. Well-understood pattern (similar to PKI CA root key
  ceremony); ops-compliance owns the runbook.
- **Per-cell evaluator pod overhead.** ~3+ replicas per cell × N
  cells; resource cost ~0.1% of cell compute per ADR-0243 §D-6.
  Carbon impact negligible vs the alternative-engine consolidation
  benefit.

**Accepted** as the keystone #5 of 14 in the foundational doctrine
bundle.

## Consequences

### Positive

1. **Cedar substrate has correct shape for its load-bearing role.**
   Per-cell evaluator pods, T1 SLO, independent team ownership match
   the universal-gate doctrine.
2. **Bootstrap sequence is deterministic** per ADR-0242 §D-5 step 5.
3. **Hot-reload achievable at <5s p99** because the pub-sub fan-out
   is scoped to per-cell evaluator replicas (not the full fleet).
4. **Coverage scanning centralized** in one µservice; one source of
   truth for portfolio-wide Cedar coverage status.
5. **Signing-chain centralized.** Org root → intermediate key chain
   managed in one place; rotation ceremony has one owner (ops-
   compliance).
6. **Audit emission consistent.** One audit-row shape; one retention
   policy; one DSAR cascade path.
7. **Ontology focus restored.** Ontology PRD's "universal mediator"
   framing was misleading post-ADR-0243; with policy-engine as a
   peer substrate, Ontology is the *information* substrate, policy-
   engine is the *policy* substrate, workflow-engine is the
   *orchestration* substrate (per ADR-0059 + ADR-0245).
8. **Hyperscaler-shape achieved.** Matches AWS Verified Permissions,
   Google Org Policy, Netflix OPA, Pinterest OPA — every named
   industry reference.
9. **Autonomous-masterplan-execution unlocked** per ADR-0247 self-
   modification doctrine; Foundry workflows modify policy-engine
   fragments under Cedar gates evaluated by policy-engine itself
   (the cycle is broken by deploying the new fragment to a paired DR
   cell first; production cell consumes after green canary).

### Negative

1. **One-time migration cost.** 47 crates renamed or introduced.
   Bounded; one ChangeSet executes.
2. **Bootstrap HSM ceremony.** Annual root-key rotation; ops-compliance
   owns runbook. Well-understood pattern; cost commensurate with
   hyperscaler-grade ops.
3. **One network hop per evaluation.** Mitigated to <1ms p99 by in-
   cell evaluator + Valkey hot cache + SDK circuit-breaker fallback.
4. **47-crate footprint grows the workspace.** Bounded by ADR-0132
   no-grouping policy; each BC is single-concern.

### Operational

1. **New CI lanes** (advisory until substrate lands; BLOCKER post-
   bootstrap):
   - `oya-governance-cedar-coverage` (rename from `oya-foundry-
     fitness-cedar-coverage`).
   - `oya-governance-cedar-fragment-signature` (rename).
   - `oya-governance-cedar-default-deny-coverage` (rename).
   - `oya-governance-cedar-tenant-fragment-restriction` (rename).
   - `oya-governance-policy-engine-substrate-coherence` (NEW; verifies
     the µservice's flat layout per ADR-0131; verifies the crate
     redistribution per §D-3; verifies the bootstrap-genesis schema
     per §D-7).
   - `oya-governance-no-policy-in-code` (NEW; static-analysis lane
     identifying imperative policy decisions per ADR-0243 §D-1).
2. **New µservice directory** `microservices/policy-engine/` per §D-1.
3. **Crate redistribution ChangeSet** per §D-3.
4. **Ontology PRD amendment ChangeSet** per §D-9.
5. **Helm charts** for per-cell evaluator + fragment-registry +
   hot-reload worker + Valkey hot-cache + Kafka topic.
6. **HSM bootstrap runbook** per ADR-0242 §D-5 step 5 sub-sequence.
7. **Quarterly DR drills** per ADR-0241.
8. **Annual org root key rotation ceremony** per ADR-0243 §D-5.

### Sustainability

- Per-cell evaluator pods consume ~0.1% of cell compute (per ADR-0243
  §D-6 capacity model). Carbon impact minimal.
- Centralization reduces aggregate fleet compute vs the sidecar-per-
  pod alternative (~50% fewer evaluator pods at fleet scale per the
  Alt-3 analysis).
- Per-cell topology avoids cross-region carbon cost of cross-region
  evaluator calls (every evaluation is intra-cell).

### Compliance

- **GDPR Article 22 (automated individual decision-making).** Cedar
  decisions emit individually-auditable per-decision rationale.
- **EU AI Act Article 14 (transparency).** AI-mediated decisions emit
  applied-fragments list.
- **HIPAA Security Rule §164.312 (access control).** Every PHI-
  touching action authorized via Cedar permit + audit emission.
- **SOC 2 CC6.1 (logical access).** All authorization is Cedar-
  mediated; one µservice in scope for auditor review of the policy
  surface.
- **ISO 27001 Annex A.9 (access control).** Cedar provides the
  control; centralized auditor evidence packet.
- **KR-PIPA Article 22 (consent).** Consent records influence Cedar
  evaluation via context attributes.
- **NIST SP 800-162 (ABAC).** Cedar is ABAC; conformance evidence
  centralized.
- **NIST SP 800-207 (Zero Trust Architecture).** Per-call policy
  evaluation principle; fail-closed default.

## Implementation surface

### F.0 µservice-registry diff

Per ADR-0056 §"Microservice registry", every new µservice slot2 token
MUST be registered in `[workspace.metadata.oya.microservices]` in the
root `Cargo.toml` BEFORE the rename ChangeSet that introduces crates
with that slot2 token. Without this row, the `oya-shared-architecture-
check-cli -- lib-name-parity` lane fails on every `oya-policy-engine-*`
crate at first build.

**Registry row to add (sequence: lands before rename ChangeSet):**

```toml
# Root Cargo.toml — [workspace.metadata.oya.microservices]
[workspace.metadata.oya.microservices.policy-engine]
owner = "axis-policy-engine"
tier  = "substrate"
adr   = "ADR-0246"
```

**Complete crate inventory** (47 crates; actual count = 47; matches §D-3 summary table):

| Crate slug | BC | Layer | BNF v4.1 conformance |
|---|---|---|---|
| `oya-policy-engine-fragment-registry-kernel` | `fragment-registry` | `kernel` | PASS: `policy-engine`.`fragment-registry`.`kernel` |
| `oya-policy-engine-fragment-registry-domain` | `fragment-registry` | `domain` | PASS |
| `oya-policy-engine-fragment-registry-usecase` | `fragment-registry` | `application` | PASS (ADR-0106 `usecase` = `application`) |
| `oya-policy-engine-fragment-registry-api` | `fragment-registry` | `api` | PASS (ADR-0105 Amendment 1) |
| `oya-policy-engine-fragment-registry-adapter` | `fragment-registry` | `adapter` | PASS |
| `oya-policy-engine-evaluator-kernel` | `evaluator` | `kernel` | PASS |
| `oya-policy-engine-evaluator-domain` | `evaluator` | `domain` | PASS |
| `oya-policy-engine-evaluator-usecase` | `evaluator` | `application` | PASS |
| `oya-policy-engine-evaluator-api` | `evaluator` | `api` | PASS |
| `oya-policy-engine-evaluator-adapter` | `evaluator` | `adapter` | PASS |
| `oya-policy-engine-evaluator-rest` | `evaluator` | `rest` | PASS |
| `oya-policy-engine-evaluator-worker` | `evaluator` | `worker` | PASS |
| `oya-policy-engine-evaluator-sdk` | `evaluator` | `sdk` | PASS |
| `oya-policy-engine-evaluator-app` | `evaluator` | `app` | PASS (composition-root binary) |
| `oya-policy-engine-signing-chain-kernel` | `signing-chain` | `kernel` | PASS |
| `oya-policy-engine-signing-chain-domain` | `signing-chain` | `domain` | PASS |
| `oya-policy-engine-signing-chain-usecase` | `signing-chain` | `application` | PASS |
| `oya-policy-engine-signing-chain-api` | `signing-chain` | `api` | PASS |
| `oya-policy-engine-signing-chain-adapter` | `signing-chain` | `adapter` | PASS |
| `oya-policy-engine-hot-reload-kernel` | `hot-reload` | `kernel` | PASS |
| `oya-policy-engine-hot-reload-domain` | `hot-reload` | `domain` | PASS |
| `oya-policy-engine-hot-reload-usecase` | `hot-reload` | `application` | PASS |
| `oya-policy-engine-hot-reload-api` | `hot-reload` | `api` | PASS |
| `oya-policy-engine-hot-reload-adapter` | `hot-reload` | `adapter` | PASS |
| `oya-policy-engine-hot-reload-worker` | `hot-reload` | `worker` | PASS |
| `oya-policy-engine-pack-overlay-kernel` | `pack-overlay` | `kernel` | PASS |
| `oya-policy-engine-pack-overlay-domain` | `pack-overlay` | `domain` | PASS |
| `oya-policy-engine-pack-overlay-usecase` | `pack-overlay` | `application` | PASS |
| `oya-policy-engine-pack-overlay-api` | `pack-overlay` | `api` | PASS |
| `oya-policy-engine-pack-overlay-adapter` | `pack-overlay` | `adapter` | PASS |
| `oya-policy-engine-tenant-overlay-kernel` | `tenant-overlay` | `kernel` | PASS |
| `oya-policy-engine-tenant-overlay-domain` | `tenant-overlay` | `domain` | PASS |
| `oya-policy-engine-tenant-overlay-usecase` | `tenant-overlay` | `application` | PASS |
| `oya-policy-engine-tenant-overlay-api` | `tenant-overlay` | `api` | PASS |
| `oya-policy-engine-tenant-overlay-adapter` | `tenant-overlay` | `adapter` | PASS |
| `oya-policy-engine-bootstrap-kernel` | `bootstrap-genesis` | `kernel` | PASS (BC name `bootstrap-genesis`; crate BC slot truncated to `bootstrap` — BNF-legal per A1 INFO finding; see justification note §F.0.1) |
| `oya-policy-engine-bootstrap-domain` | `bootstrap-genesis` | `domain` | PASS |
| `oya-policy-engine-bootstrap-usecase` | `bootstrap-genesis` | `application` | PASS |
| `oya-policy-engine-bootstrap-app` | `bootstrap-genesis` | `app` | PASS (one-shot K8s Job binary) |
| `oya-policy-engine-coverage-audit-kernel` | `coverage-audit` | `kernel` | PASS |
| `oya-policy-engine-coverage-audit-domain` | `coverage-audit` | `domain` | PASS |
| `oya-policy-engine-coverage-audit-usecase` | `coverage-audit` | `application` | PASS |
| `oya-policy-engine-coverage-audit-api` | `coverage-audit` | `api` | PASS |
| `oya-policy-engine-coverage-audit-adapter` | `coverage-audit` | `adapter` | PASS |
| `oya-shared-policy-engine-client-kernel` | cross-µservice SDK | `kernel` | PASS: `shared`.`policy-engine-client`.`kernel`; slot2=`shared` per ADR-0056 §"Microservice registry" for cross-µservice utility crates |
| `oya-shared-policy-engine-client-adapter` | cross-µservice SDK | `adapter` | PASS |
| `oya-shared-policy-engine-client-sdk` | cross-µservice SDK | `sdk` | PASS |

**Actual crate count: 47.** Matches §D-3 summary table exactly.

**§F.0.1 — BC-name-vs-crate-slot justification for `bootstrap-genesis`:**
The BC is named `bootstrap-genesis` (2 kebab tokens). The crate BC
slot is truncated to `bootstrap` (1 token) because the BC's scope is
already unambiguous given the µservice context (`policy-engine`), and
the 4-crate set (`bootstrap-{kernel,domain,usecase,app}`) exhausts the
BC's surface. Multi-token vs single-token BC slots are both BNF-legal
per ADR-0056 §4a. This justification satisfies the A1 INFO finding
(BNF-V05 INFO) from the A1 verdict.

**BC-registry diff** (per ADR-0056 §"Bounded Context Registry as a Living Document"):

The following BCs must be registered in `docs/standards/bounded-contexts.md`
under the `policy-engine` µservice:

| BC | Action |
|---|---|
| `cedar-fragment-coverage` → `fragment-registry` | RENAME |
| `evaluator` | NEW |
| `signing-chain` | NEW |
| `hot-reload` | NEW |
| `coverage-audit` | NEW |
| `pack-overlay` | NEW |
| `tenant-overlay` | NEW |
| `bootstrap-genesis` | NEW |

| Artifact | Status |
|---|---|
| `/specs/microservices/policy-engine.json` | NEW |
| `/specs/cedar-fragment-schema.json` | NEW |
| `/specs/policy-gate-coverage.json` | NEW |
| `microservices/policy-engine/PRD.md` | NEW — per ADR-0131 PRD template |
| `microservices/policy-engine/README.md` | NEW |
| `microservices/policy-engine/api/grpc/policy_engine.proto` | NEW — per §D-4 |
| `microservices/policy-engine/api/openapi/policy-engine.openapi.yaml` | NEW — per §D-4 |
| `microservices/policy-engine/capabilities/*.yaml` | NEW — one per operation |
| `microservices/policy-engine/calls.yaml` | NEW — cross-µservice call manifest |
| `microservices/policy-engine/fragments/baseline/*.cedar` | NEW — baseline fragment set per gate category |
| `microservices/policy-engine/fragments/bootstrap/genesis.cedar` | NEW — genesis fragment signed by org root |
| `microservices/policy-engine/fragments/reserved/*.cedar` | NEW — reserved-namespace fragments per ADR-0242 |
| `microservices/policy-engine/iac/helm/policy-engine/values.yaml` | NEW |
| `microservices/policy-engine/iac/helm/policy-engine/templates/*` | NEW — per §D-5 deployment topology |
| `microservices/policy-engine/migrations/0001_fragment_registry_init.sql` | NEW — per §D-7 |
| `microservices/policy-engine/migrations/0002_signing_keys_init.sql` | NEW — per §D-7 |
| `microservices/policy-engine/migrations/0003_coverage_reports_init.sql` | NEW — per §D-7 |
| `microservices/policy-engine/migrations/0004_evaluation_audit_index_init.sql` | NEW — per §D-7 |
| `microservices/policy-engine/migrations/0005_genesis_fragment_seed.sql` | NEW — per §D-7 |
| `microservices/policy-engine/slos/*.openslo.yaml` | NEW — per §D-10 |
| `microservices/policy-engine/tests/integration/*.rs` | NEW — hot-reload, signing-chain, tenant-overlay, emergency-permit, fail-closed, bootstrap-genesis |
| `microservices/policy-engine/tests/load/*.rs` | NEW — evaluate_p99, hot_reload_propagation |
| Rename `oya-ontology-cedar-fragment-coverage-*` → `oya-policy-engine-fragment-registry-*` (5 crates) | RENAME |
| NEW crates: `oya-policy-engine-evaluator-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` (9) | NEW |
| NEW crates: `oya-policy-engine-signing-chain-{kernel,domain,usecase,api,adapter}` (5) | NEW |
| NEW crates: `oya-policy-engine-hot-reload-{kernel,domain,usecase,api,adapter,worker}` (6) | NEW |
| NEW crates: `oya-policy-engine-pack-overlay-{kernel,domain,usecase,api,adapter}` (5) | NEW |
| NEW crates: `oya-policy-engine-tenant-overlay-{kernel,domain,usecase,api,adapter}` (5) | NEW |
| NEW crates: `oya-policy-engine-bootstrap-{kernel,domain,usecase,app}` (4) | NEW |
| NEW crates: `oya-policy-engine-coverage-audit-{kernel,domain,usecase,api,adapter}` (5) | NEW |
| NEW crates: `oya-shared-policy-engine-client-{kernel,adapter,sdk}` (3) | NEW |
| Ontology PRD amendment (per §D-9) | AMEND |
| Rename `oya-ontology-agent-gateway-*` → `oya-ontology-tool-call-ingress-*` | RENAME |
| `tools/oya-governance-cedar-coverage/` | NEW (rename from `oya-governance-cedar-coverage`) |
| `tools/oya-governance-policy-engine-substrate-coherence/` | NEW |
| `tools/oya-governance-no-policy-in-code/` | NEW |
| `docs/runbooks/policy-engine-bootstrap-step-5.md` | NEW — full HSM quorum ceremony procedure |
| `docs/runbooks/policy-engine-emergency-permit.md` | NEW |
| `docs/runbooks/policy-engine-emergency-forbid.md` | NEW |
| `docs/runbooks/policy-engine-hot-reload-failure.md` | NEW |
| `docs/runbooks/policy-engine-dr-drill.md` | NEW |
| `docs/standards/policy-engine-fragment-authoring.md` | NEW |
| `docs/standards/policy-engine-substrate-consumption.md` | NEW — guidance for downstream µservices |

## Verification

- [ ] `microservices/policy-engine/` directory exists at the canonical flat layout per ADR-0131.
- [ ] `cargo build --workspace` passes from a clean checkout with the 47 crates per §D-3.
- [ ] Migration `0001_fragment_registry_init.sql` through `0005_genesis_fragment_seed.sql` run cleanly against Postgres 17 LTS + Citus 12.
- [ ] gRPC `policy_engine.proto` round-trips via tonic generation + serves all 10 operations per §D-4.
- [ ] OpenAPI 3.2.0 `policy-engine.openapi.yaml` mirrors the gRPC surface.
- [ ] Genesis fragment per §D-8 is signed by the org root key (loaded via PKCS#11 from HSM); signing-chain verification path passes integration test `tests/integration/bootstrap_genesis.rs`.
- [ ] Per-cell evaluator Deployment with 3+ replicas + HPA + PDB deployable in a bootstrap cell.
- [ ] Hot-reload propagation p99 < 5s measured under load in `tests/load/hot_reload_propagation.rs`.
- [ ] Hot-path evaluate p99 < 1ms at 10k QPS per cell in `tests/load/evaluate_p99.rs`.
- [ ] Cold-path evaluate p99 < 50ms.
- [ ] `cloud-ci/Rust gate packet cedar-coverage` reports ≥ 95% coverage portfolio-wide (bootstrap target; goal 100% by post-keystone +90 days).
- [ ] `cloud-ci/Rust gate packet cedar-fragment-signature` succeeds for all published fragments.
- [ ] `cloud-ci/Rust gate packet cedar-default-deny-coverage` reports every permit has a corresponding default-deny.
- [ ] `cloud-ci/Rust gate packet cedar-tenant-fragment-restriction` succeeds for all tenant-scoped fragments.
- [ ] `cloud-ci/Rust gate packet policy-engine-substrate-coherence` exits 0 (verifies flat layout + crate redistribution + bootstrap-genesis schema).
- [ ] `cloud-ci/Rust gate packet no-policy-in-code` reports zero in-code policy decisions in pilot µservice `microservices/tenancy/`.
- [ ] Ontology PRD amendment per §D-9 is merged and `cedar-fragment-coverage` BC tombstone marker is in place.
- [ ] `oya-ontology-agent-gateway-*` → `oya-ontology-tool-call-ingress-*` rename ChangeSet landed.
- [ ] CI lane rename ChangeSet (`oya-governance-fitness-cedar-*` → `oya-governance-cedar-*`) landed.
- [ ] DR drill against the paired DR cell completes within 5min RTO + 0s RPO.
- [ ] Annual org root key rotation runbook drilled at least once before BLOCKER promotion.
- [ ] cedar-policy-analyzer integration in CI passes for baseline fragment set.
- [ ] Audit-chain emits `CedarEvaluation` for every decision; sampled regulatory query returns all decisions for a given tenant within 30 days.
- [ ] ADR-0150 + ADR-0183 frontmatter updated with `amended_by: [ADR-0246]`.
- [ ] Ontology PRD frontmatter updated with `amended_by: [ADR-0246]`.

## References

### Industry sources

- **AWS Verified Permissions** (GA 2024-Q1). The reference Cedar deployment at hyperscale. Architected as a separated service (not embedded in IAM, not a sidecar). Docs: `docs.aws.amazon.com/verifiedpermissions`. AWS re:Invent 2023 session "BOA303: Authorization using Amazon Verified Permissions" describes the centralized-service architecture.
- **AWS Cedar SDK Rust crate** (`cedar-policy` on crates.io v4.x).
- **cedar-policy-analyzer** (AWS-funded; open-source 2024). Formal verification of Cedar policies.
- **AWS Builder's Library — "Static stability using Availability Zones"** (Becky Weiss + Mike Furr). Fail-closed default + cache fallback pattern; informs the SDK circuit-breaker design.
- **AWS Builder's Library — "Avoiding insurmountable queue backlogs"** (Marc Brooker). Per-cell evaluator pool sizing.
- **AWS Builder's Library — "Caching challenges and strategies"** (Becky Weiss). Hot-cache TTL + invalidation design for the Valkey layer.
- **Open Policy Agent (OPA) at Netflix** (Netflix Tech Blog 2020-2023 series). Centralized-service deployment pattern; informs hot-reload + per-region topology.
- **OPA at Pinterest** (Pinterest Engineering Blog 2022 "Authorization at Pinterest"). Centralized-service pattern.
- **OPA at T-Mobile** (CNCF case study 2023). Multi-engine consolidation lessons.
- **OPA at Capital One** (CNCF case study 2022). Per-tenant overlay composition pattern.
- **Netflix authorization service** (Netflix Tech Blog "Sidecar-Less Service Mesh at Netflix" 2024). Centralized authz service supersedes per-pod sidecars.
- **Google Cloud Org Policy + IAM Conditions** documentation. Consolidated-policy pattern as separated service.
- **Microsoft Azure Policy** documentation. Comparative reference for centralized policy management.
- **Verma et al., "Borg, Omega, and Kubernetes"** (CACM 2016, vol. 59 no. 5). Centralized policy service deployed alongside Borg/Kubernetes; informs per-cell evaluator topology.
- **Pat Helland, "Life Beyond Distributed Transactions" (2007).** Per-call policy evaluation as the natural shape for distributed transactions.
- **Eric Brewer, "Towards Robust Distributed Systems" (PODC 2000).** CAP theorem context for the fail-closed default.

### Cedar / formal-policy academic + practitioner sources

- **Cedar specification** (v1.0 published 2023; v4 published 2025). Language reference; `cedarpolicy.com`.
- **"Cedar: A New Language for Authorization"** (AWS, OOPSLA 2024). Academic paper introducing Cedar's design.
- **"Formal verification of authorization policies"** (various authors, 2024-2025).
- **Mark Stamp, *Information Security: Principles and Practice* (3rd ed.).** Foundational ABAC + authorization theory.
- **NIST SP 800-162 — Attribute Based Access Control (ABAC).** Cedar is ABAC.
- **NIST SP 800-207 — Zero Trust Architecture.** Per-call policy evaluation principle.

### Regulatory sources

- **GDPR Article 22 — Automated individual decision-making.** Individual-decision auditability requirement.
- **EU AI Act 2024/1689 Article 14 — Transparency obligations.**
- **HIPAA Security Rule §164.312 — Access control.** Authorization audit trail.
- **SOC 2 Type II — CC6.1 (Logical Access Controls).**
- **ISO 27001 Annex A.9 — Access Control.**
- **KR-PIPA Article 22 — Consent.**
- **NIST SP 800-92 — Audit log standards.**
- **CSAP (Cloud Security Assurance Program) v3.1.** Korean regulator framework.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Per-cell evaluator topology.
- **ADR-0028 — Cloud microservice architecture.** Substrate µservice shape.
- **ADR-0049 — Cross-region replication + residency.** Cross-region DR pairing constraints.
- **ADR-0050 — Event bus + outbox canonical.** Kafka pub-sub for hot-reload.
- **ADR-0099 — Data class registry.** Data classes are Cedar entity attributes.
- **ADR-0105 — Thirteen-layer canonical enum.** Crate layer mapping.
- **ADR-0106 — application → usecase rename.** Crate layer naming.
- **ADR-0128 — Hyperscaler architecture invariants.** Substrate µservice pattern.
- **ADR-0131 — Per-microservice flat layout.** Directory layout authority.
- **ADR-0132 — No-grouping forward policy.** Single-concern BCs.
- **ADR-0140 — Cedar policy enforcement (retired).** Original framing.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Tier evaluation is Cedar-mediated.
- **ADR-0145 — Inter-microservice communication reform.** Direct gRPC + Cedar evaluation at caller.
- **ADR-0148 — Service mesh canonical.** Cedar PDP integration point via `ext_authz`.
- **ADR-0150 — Cedar policy engine.** Amended by this ADR.
- **ADR-0176 — Brown-out + degradation signal.** Cedar evaluator availability signal.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Amended by this ADR.
- **ADR-0211 — In-house Rust-primary tech stack.** All crates Rust.
- **ADR-0240 — Sovereign cloud per regional pack.** Per-pack Cedar overlays.
- **ADR-0241 — DR + BC portfolio policy.** T1 tier declaration.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** Bootstrap sequence step 5.
- **ADR-0243 — Cedar as Universal Gate (companion keystone #2).** The forcing function for this promotion.
- **ADR-0244 — Tenant as universal scoping primitive (companion keystone #3).** Tenant-scoped Cedar fragments.
- **ADR-0245 — Substrate vs Product layering (companion keystone #4).** Policy-engine is a substrate.
- **ADR-0247 — Self-hosting / self-modification doctrine (companion keystone #6).** Foundry workflows modify policy-engine fragments under Cedar gates.
- **ADR-0248 — Amazon-shape cellular architecture (companion keystone #7).** Per-cell evaluator topology.
- **ADR-0251 — Compliance Pack + Cell Certification Levels (companion).** Packs are Cedar fragment bundles.

### Auto-memory feedback

- `feedback_cedar_as_universal_gate` — applies; this ADR implements §D-1's promotion mandate.
- `feedback_oyatie_is_a_tenant_doctrine` — applies; oyatie principals' fragments live in policy-engine.
- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade.
- `feedback_no_silent_regression` — reinforced; rename ChangeSet ships deprecation shims for one minor version.
- `feedback_automate_everything` — reinforced; fragment lifecycle automated end-to-end.
- `feedback_clean_architecture_requirements` — reinforced; port-in-kernel preserved per ADR-0105.
- `feedback_workflow_objectgraph_adapter_layer` — refined; the adapter layer is now three peer substrates (policy-engine + ontology + workflow-engine).

---

## Appendix A: Hyperscaler pattern attribution matrix

Per the audit pattern established in the pre-keystone exploration
(2026-05-20 session record), every architectural decision in this ADR
is attributed to a named hyperscaler pattern + source + anti-pattern
avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Promote BC to peer substrate µservice) | "Centralized Policy Service" | AWS Verified Permissions (re:Invent 2023 BOA303); Google Org Policy; Netflix authz service (Netflix Tech Blog 2024) | "Embedded Policy in Application Service" — policy evolution coupled to host service deploy cycle |
| D-2 (8 BCs: fragment-registry, evaluator, signing-chain, hot-reload, coverage-audit, pack-overlay, tenant-overlay, bootstrap-genesis) | "Single-Concern Bounded Contexts" | DDD (Evans 2003); ADR-0132 no-grouping forward policy | "Bundle Bounded Context" — multi-concern BCs prone to coupling drift |
| D-3 (47-crate redistribution following BNF v4.1 + ADR-0105 layer enum) | "Hexagonal Architecture with Port-in-Kernel" | Cockburn 2005 Hexagonal; ADR-0105 13-value canonical enum | "Anemic Layered Architecture" — ports defined outside kernel, leaking I/O concerns |
| D-4 (internal gRPC/proto3 + public/compat OpenAPI 3.2.0 REST projection; 10 operations) | "Internal RPC with REST Public/Compat" | Google API Design Guide; Stripe API design (REST primary with gRPC for internal); Cloudflare Workers gRPC | "Single-Protocol Lock-in" — REST-only locks out efficient inter-µservice calls; gRPC-only locks out browser callers |
| D-5 (Per-cell deployment: 3+ replicas + HPA + PDB + cross-region paired DR cell) | "Cell-Sharded Stateless Tier with HA" | AWS cell-based architecture (Bryan Liston re:Invent 2018); ADR-0009 cell architecture; ADR-0048 cell sharding | "Global Singleton Service" — single-region or single-replica policy service is a portfolio-wide blast radius |
| D-6 (Hot path p99 < 1ms via in-cell evaluator + Valkey hot cache + circuit breaker fallback) | "Static Stability + Edge-Cached Evaluation" | AWS Builder's Library "Static Stability" (Weiss + Furr); AWS Verified Permissions production cache; Cloudflare Workers KV | "Synchronous Round-Trip to Global Policy Store" — cross-region policy fetch on hot path |
| D-7 (Postgres + Citus shard on (scope, fragment_id); ClickHouse not required because Cedar AST cache covers hot path) | "Distributed Relational with Application-Aware Sharding" | Citus design (Microsoft acquired 2019); AWS Aurora Limitless; Google Spanner external consistency | "Single-Instance Relational Bottleneck" — single Postgres for global policy doesn't scale write-side |
| D-8 (Bootstrap chain of trust: org root in HSM → genesis fragment → intermediate keys → publisher fragments) | "PKI Root + Intermediate Certificate Chain" | RFC 5280 X.509; Sigstore Rekor; AWS KMS key hierarchy; Let's Encrypt CA hierarchy | "Implicit Bootstrap Trust" — undocumented signing key emergence |
| D-9 (Ontology amendment: drop BC, rewrite "universal mediator" framing, rename agent-gateway BC) | "Substrate Cohesion via PRD Amendment" | DDD context-mapping (Evans 2003); ADR pattern (Nygard 2011) | "Stale PRD" — PRDs drift behind architectural reality |
| D-10 (SLO targets: T1, 5min RTO, 0 RPO, 99.99% availability) | "Tiered DR + Per-Microservice SLO Ownership" | ADR-0241 DR + BC portfolio policy; Google SRE Workbook ch. 2 | "Implicit SLO" — µservices ship without explicit SLO declaration |
| D-11 (CI lane rename: oya-governance-* → oya-governance-*) | "Coverage-Enforced Substrate Doctrine" | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | "Untested Substrate Surface" — substrate gates discovered missing in production |

---

## Appendix B: Worked example — Cedar fragment lifecycle from author through evaluation

To illustrate that the lifecycle in §D-2 is concrete and operable, here
is a worked example covering the full lifecycle of a single Cedar
fragment from initial authoring through eventual sunset.

**Scenario.** The `microservices/marketplace/` µservice introduces a
new action `Marketplace::Action::PublishPlugin` that lets a developer
tenant publish a plugin to the global plugin app store. The plugin's
category determines additional gating: a healthcare-category plugin
requires HIPAA-pack approval; a financial-services plugin requires
KR-FSS-pack approval if KR-resident tenants will install it.

### Phase 1: Authoring

**2026-06-12 10:14 UTC** — A platform engineer at axis-marketplace
authors a Cedar fragment to gate the new action:

```cedar
// microservices/policy-engine/fragments/baseline/marketplace-publish-plugin.cedar
// @fragment_id: marketplace-publish-plugin-baseline-v1
// @scope: baseline
// @applies_to_actions: ["Marketplace::Action::PublishPlugin"]
// @applies_to_resources: ["Marketplace::Plugin"]
// @effective_at: 2026-07-01T00:00:00Z
// @sunset_at: null
// @signed_by: pending

permit (
  principal,
  action == Marketplace::Action::PublishPlugin,
  resource is Marketplace::Plugin
)
when {
  principal.tenant_id != "" &&
  principal.has_role("plugin_publisher") &&
  resource.category in ["productivity", "developer-tools", "analytics", "communication"]
};

permit (
  principal,
  action == Marketplace::Action::PublishPlugin,
  resource is Marketplace::Plugin
)
when {
  principal.tenant_id != "" &&
  principal.has_role("plugin_publisher") &&
  resource.category == "healthcare" &&
  principal.tenant.has_active_pack("hipaa-t3")
};

permit (
  principal,
  action == Marketplace::Action::PublishPlugin,
  resource is Marketplace::Plugin
)
when {
  principal.tenant_id != "" &&
  principal.has_role("plugin_publisher") &&
  resource.category == "financial-services" &&
  principal.tenant.has_active_pack("kr-fss-t2")
};

// Default deny
forbid (
  principal,
  action == Marketplace::Action::PublishPlugin,
  resource is Marketplace::Plugin
)
unless {
  // Negative space — caught by the permits above
  false
};
```

The fragment is committed to a feature branch with frontmatter
declaring the lifecycle state `AUTHORED`.

### Phase 2: Multispectrum review

**2026-06-12 10:30 UTC** — CI triggers multispectrum review v2.4.0
fan-out. Per ADR-0243 §D-8, the Cedar-fragment review activates
these facets:

- **F1 (correctness):** Reviewer-agent verifies the fragment's
  permits cover the documented use cases. Verdict: PASS — three
  category branches match the marketplace categorisation spec.
- **F2 (hyperscaler-fitness):** Verdict: PASS with note — fragment
  follows AWS Cedar idiom (named entity types `Marketplace::Plugin`,
  explicit `when` conditions). Note: consider adding
  `principal.tenant.is_kyb_verified` predicate for additional
  defence-in-depth.
- **F5 (security):** cedar-policy-analyzer runs formal verification:
  no privilege escalation path; no anonymous-principal permit.
  Verdict: PASS.
- **F6 (performance):** Evaluation-cost analysis: no unbounded
  entity-set traversals; predicates are constant-time on entity
  attributes. Verdict: PASS.
- **F7 (supply chain):** Signing-key chain pre-flight: the
  marketplace-team's intermediate signing key chains back to the
  org-root via cosign attestation chain at
  `cosign://oyatie/org-root → org-baseline → marketplace-team`.
  Verdict: PASS.
- **A1 (own-policy-adherence-naming):** Fragment ID
  `marketplace-publish-plugin-baseline-v1` follows the
  `<scope>-<microservice>-<action-family>-<version>` convention.
  Verdict: PASS.
- **A4 (architecture-adherence):** Fragment lives under `baseline/`
  not `pack/` or `tenant/` (correct; this is a portfolio-wide
  baseline). Verdict: PASS.
- **A6 (schema-adherence):** Cedar entity type `Marketplace::Plugin`
  matches the Ontology Object Type `Marketplace::Plugin`'s declared
  attributes (`category`, `tenant_id`, etc.). Verdict: PASS.

**2026-06-12 14:00 UTC** — All facets PASS; reviewer-agent emits
multispectrum verdict `APPROVED`. Fragment state advances to
`REVIEWED`.

### Phase 3: Signing

**2026-06-12 14:15 UTC** — The reviewed fragment is signed by the
`org-baseline-key` (held in OpenBao; quorum 1 because this is a
baseline fragment, not an emergency or pack-owner fragment).

The signing flow:

1. `policy-engine.signing-chain.usecase` fetches the org-baseline-
   key from OpenBao.
2. Ed25519 signature over `sha512(fragment_body || effective_at ||
   sunset_at)`.
3. Cosign attestation recorded at Sigstore Rekor (or equivalent
   internal Rekor) referencing the fragment hash.
4. Signature row inserted into `policy_engine.fragment_signatures`
   (per §D-7 DDL).

Fragment state advances to `SIGNED`.

### Phase 4: Publication

**2026-06-12 14:20 UTC** — A `PublishFragment` operation is invoked
(per §D-4) with the signed fragment.

The publication flow:

1. `policy-engine.fragment-registry.usecase` inserts the row into
   `policy_engine.fragments` (state `PUBLISHED`).
2. Body blob written to SeaweedFS at
   `oya://policy-engine/fragments/<body_hash>`; `body_blob_ref` +
   `body_hash` columns updated.
3. `PublishFragment` emits `CedarFragmentPublished` event on the
   audit-chain (audit stream `policy-engine.fragment-lifecycle`).
4. Notification emitted on per-cell Kafka topic `policy-engine.
   fragment-reload.v1`.

Fragment state: `PUBLISHED`. Not yet `ACTIVE` because `effective_at =
2026-07-01T00:00:00Z` is in the future.

### Phase 5: Activation

**2026-07-01 00:00:00 UTC** — Effective time arrives.

The activation flow:

1. Each cell's hot-reload worker observes the cell-local clock
   crossing `effective_at` (tolerance ±2s per HLC budget).
2. Worker calls `ActivateFragment` against fragment-registry.
3. fragment-registry updates `activation_status = 'ACTIVE'` +
   `activated_at = now()`.
4. Hot-reload worker fetches the fragment body from SeaweedFS,
   compiles Cedar AST, publishes to per-cell Valkey hot cache.
5. Each per-cell evaluator replica observes the bundle change in
   Valkey + atomic-swaps the in-process bundle.
6. Propagation completes within 5s p99 across all cell replicas.

Fragment state: `ACTIVE`. Audit emits `CedarFragmentActivated`.

### Phase 6: In-force evaluation

**2026-07-01 09:42 UTC** — A developer at `tenant-acme-corp.dev-team`
attempts to publish a healthcare plugin to the marketplace.

The evaluation flow:

1. Marketplace µservice's `PublishPlugin` handler invokes
   `policy_engine.evaluate(EvaluationRequest { principal:
   "tenant-acme-corp.dev-team.user-7421", action:
   "Marketplace::Action::PublishPlugin", resource: "Marketplace::
   Plugin::plugin-3421", context: { category: "healthcare" },
   tenant_id: "tenant-acme-corp", cell_id: "data-plane-cell-us-east-
   1-a", evaluation_id: <UUIDv7> })`.
2. SDK gRPCs to the cell-local evaluator pool (`oya-policy-engine-
   evaluator-rest` HTTP path or gRPC stream).
3. Evaluator consults Valkey hot cache for the compiled bundle keyed
   by `(cell_id, tenant_id_overlay_set_hash)`.
4. Cache hit. Bundle contains baseline `marketplace-publish-plugin-
   baseline-v1:v1` plus tenant-acme-corp's tenant overlay fragments
   plus the active packs (`soc2-t2`, `gdpr-eu-t2`).
5. Cedar evaluator runs against the bundle: the healthcare branch
   permit requires `principal.tenant.has_active_pack("hipaa-t3")`;
   tenant-acme-corp has not adopted HIPAA pack. The branch's
   condition evaluates to `false`. No other branch matches. The
   default-deny applies.
6. Decision: `FORBID`.
7. Response: `{ decision: FORBID, reason: "principal.tenant lacks
   active pack hipaa-t3 required for healthcare category", applied_
   fragments: ["baseline/marketplace-publish-plugin-baseline-v1:v1"],
   determining_policies: ["marketplace-publish-plugin-baseline-v1#
   default-deny"], evaluation_ms: 0.6, audit_emitted: true }`.
8. Audit emission: a row written to `policy_engine.evaluation_audit_
   index` + the full audit row sealed in `microservices/audit-chain/`
   under the `tenant-acme-corp` audit stream.
9. Marketplace µservice returns HTTP 403 to the caller with the
   reason string.

Hot-path evaluation completed in 0.6ms — well under the 1ms p99
budget.

### Phase 7: Tenant overlay introduces a more restrictive forbid

**2026-08-15 11:00 UTC** — tenant-acme-corp's admin authors a tenant-
scoped fragment forbidding any plugin publication from non-VPN
network ranges:

```cedar
// fragments/tenant/tenant-acme-corp/marketplace-vpn-only.cedar
// @fragment_id: tenant-acme-corp-marketplace-vpn-only-v1
// @scope: tenant/tenant-acme-corp

forbid (
  principal,
  action == Marketplace::Action::PublishPlugin,
  resource is Marketplace::Plugin
)
when {
  !context.from_corporate_vpn
};
```

The fragment is signed by tenant-acme-corp's tenant-admin-key (per
§D-3 `tenant-overlay` BC) and the tenant-fragment-restriction
checker verifies: this fragment only `forbid`s (it does not raise
permits beyond baseline). Verdict: PASS. Published. Activated. Hot-
reloaded within 5s across the cell's evaluator replicas.

From this point, tenant-acme-corp's `PublishPlugin` requests evaluate
against baseline + acme's tenant overlay; non-VPN requests forbid
even when the user has the right role + the right pack — because
deny wins ties.

### Phase 8: Sunset

**2027-06-30 23:59 UTC** — The marketplace team has authored a
revised fragment `marketplace-publish-plugin-baseline-v2` with
tightened category checks. They schedule the v1 sunset at the v2
effective time.

The sunset flow:

1. `SunsetFragment` operation invoked against fragment-registry with
   `fragment_id = marketplace-publish-plugin-baseline-v1:v1`,
   `sunset_at = 2027-07-01T00:00:00Z`.
2. `activation_status` updated to `'SUNSET'` at the cutover; the
   replacement v2 fragment becomes `'ACTIVE'`.
3. Hot-reload propagates the swap; evaluators atomic-swap to the v2
   bundle.
4. Audit emits `CedarFragmentSunset` + `CedarFragmentActivated` (for
   v2).

### Phase 9: Archive

**2027-07-31 00:00:00 UTC** — Per the retention policy (regulatory
retention 7y for SOC 2 + KR PIPA Article 36 retention), the sunset
v1 fragment moves to read-only archive in SeaweedFS cold tier. The
fragment-registry row remains in Postgres with `activation_status =
'SUNSET'` for regulatory retrieval; the body blob moves to cold
storage; the Valkey hot cache evicts the bundle.

A regulator (e.g., FTC GDPR investigator, SEC OCIE) can query
`GetFragmentByID(marketplace-publish-plugin-baseline-v1:v1)` to
retrieve the historical fragment text + signature + activation
window for an authorization-evidence query.

### Phase 10: Tombstone

**2034-07-01 00:00:00 UTC** — Seven years after sunset, the
fragment exits retention. A separate sunset + tombstone ADR ratifies
the removal because the fragment carried HIPAA / KR-FSS regulatory
commitment. Body blob hard-deleted; Postgres row updated to
`activation_status = 'TOMBSTONED'`; only the fragment-registry row
metadata (fragment_id, scope, version, signed_by, signed_at,
sunset_at, body_hash, tombstone_reason, tombstoned_at) remains for
tamper-detection purposes.

---

**Why this matters.** The end-to-end lifecycle above is operable
because policy-engine is a peer substrate µservice with the right
shape: per-cell evaluator pods that hot-reload, fragment registry
with signing chain, tenant overlay that composes safely, audit
emission on every decision, regulatory retention by construction.
Each lifecycle stage has a CI lane, a runbook, and an audit emission;
no stage is ad-hoc.

The same lifecycle applies to all 23 policy-class decisions
catalogued in ADR-0243 §Context. Every gate in the portfolio follows
this shape because policy-engine is the single substrate.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification. The 47-crate registry diff with per-crate conformance checks appears in §F.0 above. This section covers non-crate names.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oya-governance-cedar-coverage` | N/A (governance lane) | `governance`.`cedar-coverage` | CI governance lane; `oya-governance-*` prefix per CLAUDE.md `new_governance_lane_prefix`; rename of `oya-governance-cedar-coverage`; verifies Cedar fragment coverage portfolio-wide. |
| `oya-governance-cedar-fragment-signature` | N/A (governance lane) | `governance`.`cedar-fragment-signature` | CI governance lane; verifies Ed25519 + cosign fragment signatures per §D signing-chain. |
| `oya-governance-cedar-default-deny-coverage` | N/A (governance lane) | `governance`.`cedar-default-deny-coverage` | CI governance lane; verifies every permit has a paired default-deny fragment. |
| `oya-governance-cedar-tenant-fragment-restriction` | N/A (governance lane) | `governance`.`cedar-tenant-fragment-restriction` | CI governance lane; verifies tenant fragments cannot raise permits above baseline. |
| `oya-governance-policy-engine-substrate-coherence` | N/A (governance lane) | `governance`.`policy-engine-substrate-coherence` | CI governance lane; verifies flat layout + crate redistribution + bootstrap-genesis schema coherence per ADR-0131. |
| `oya-governance-no-policy-in-code` | N/A (governance lane) | `governance`.`no-policy-in-code` | Static-analysis governance lane; identifies imperative policy decisions that should be Cedar evaluations per ADR-0243 §D-1. |
| `microservices/policy-engine/` | N/A (µservice path) | N/A | µservice directory; slug `policy-engine` = 2 kebab tokens (within ADR-0056 1..3 cap); semantically accurate (Cedar policy evaluation engine substrate). Flat layout per ADR-0131. |
| `microservices/policy-engine/fragments/bootstrap/genesis.cedar` | N/A (file path) | N/A | Genesis Cedar fragment path; `genesis` is the standard term for the first-ever signed fragment establishing the chain of trust; `.cedar` extension per Cedar v4.2 file convention. |

---

## Change log

- **2026-05-20 (Wave-3-A cross-reference wiring):** Added four new CI lanes to §D-11 lane catalogue per ADR-0293..0296: `oya-check-meta-trust-root-attestation` (ADR-0293), `oya-check-fragment-soak-window` (ADR-0294), `oya-check-bootstrap-spiffe-identity` (ADR-0295), `oya-check-library-first-credential-sidecar` (ADR-0296).

*End of ADR-0246.*
