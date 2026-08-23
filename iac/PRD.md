---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-iac-app
microservice: iac-app
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0117
  - ADR-0120
  - ADR-0121
  - ADR-0123
  - ADR-0139
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-0171
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
owner_team: axis-iac-app
doc_status: published
---

# PRD-iac-app: Meta-IaC Pipeline Substrate (Helm + Terraform/OpenTofu + Kustomize + GitOps)

## Purpose

The `iac-app` microservice is oyatie's **meta-IaC** substrate: the µservice that authors, validates, applies, and rolls back every other µservice's Infrastructure-as-Code (Helm charts, Terraform/OpenTofu modules, Kustomize overlays). It runs the IaC pipeline itself. Per ADR-0131 (Cloud-product split: iac-app + cloud-k8s + cloud-secrets), this µservice owns the substrate that turns git-tracked IaC into deployed cluster state across every active oyatie pack (pack-kr / pack-eu / pack-us / pack-us-healthcare / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa).

This µservice is **shared substrate**, not a hero product. Every oyatie µservice that ships IaC depends on iac-app to render, validate, plan-preview, and apply that IaC; iac-app is the precondition for ADR-0117's cloud-native posture and the operational counterpart of ADR-0120's Rust-first on-prem tooling. Its existence eliminates the "ten ways to apply Helm" anti-pattern by canonicalising one apply pipeline + one validator catalog + one drift-detection cycle across the whole estate.

This µservice has no Bominal equivalent and originates in oyatie under ADR-0131.

## Tenant Value

- **Tenant Outcome 1 — Vendor-neutral GitOps without vendor lock.** Tenants' workloads land via the same self-hosted ArgoCD + Flux + OpenTofu + Helm-controller + Kustomize-controller stack used by oyatie's own substrate; no Spacelift / Env0 / Terraform Cloud / Pulumi Cloud subscription required.
- **Tenant Outcome 2 — Per-µservice render + apply traceability.** Every apply emits a signed `ApplyExecuted` event consumed by `cloud-governance-evidence` + `audit-chain`; tenants and auditors get cryptographic proof of what changed, when, and by whom.
- **Tenant Outcome 3 — Drift-free production posture.** Continuous drift detection per cluster ≤1h cycle; reconciler converges on the git-declared state automatically; rollback to last-green state is a first-class primitive (not a "restore from backup" escape hatch).
- **Internal Outcome 4 — One IaC pipeline across the whole estate.** Eliminates per-team divergence in chart structure, plan-preview gates, secret-reference patterns, and registry conventions. The iac-app registry is the single source of truth for "what chart is deployed where at what version."

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | µservice author | to declare my chart/module at `microservices/<ms>/iac/{helm,terraform,kustomize}/` | iac-app discovers + renders it without per-µservice registration | iac-registry | Must |
| FR-02 | iac-renderer | to render Helm + Kustomize + Terraform/OpenTofu manifests deterministically (same input → same output) | every PR shows a reproducible preview | iac-renderer | Must |
| FR-03 | iac-validator | to plan-preview against a live cluster and surface drift before apply | reviewers see the actual state delta in the PR | iac-validator | Must |
| FR-04 | iac-applier | to apply a rendered manifest set in dependency-correct order with apply quorum + retry | applies are not racey; partial failure is recoverable | iac-applier | Must |
| FR-05 | iac-rollback | to revert a µservice's IaC state to the prior known-good apply | regressions auto-revert without human escalation when paired with the SLO gate | iac-rollback | Must |
| FR-06 | drift-detector | to continuously diff live cluster state against git per ≤1h cycle | unauthorised mutation is detected within 1h and reconciled or alerted | iac-validator | Must |
| FR-07 | iac-registry | to maintain a versioned catalog of charts + modules + per-pack overlays | every deployed artifact is provenance-traceable to its git commit + SLSA L3 attestation | iac-registry | Must |
| FR-08 | GitOps reconciler (ArgoCD or Flux) | to reconcile git → cluster automatically | tenants never edit cluster state directly; git is the only source of truth | iac-applier | Must |
| FR-09 | tenant operator | to view their µservice's current apply state + drift posture in a per-tenant dashboard | tenants self-detect drift before incidents | iac-registry, iac-validator | Must |
| FR-10 | governed capability consumer (cloud-governance-evidence, audit-chain) | to consume `ApplyStarted / ApplyCompleted / ApplyRolledBack / DriftDetected / RenderRequested / RenderCompleted` events | downstream pipelines have a complete audit trail | (cross-cutting) | Must |
| FR-11 | aggregation index | to regenerate per-pack apply manifests + chart-version matrix from per-µservice sources | central indices are never hand-edited; per-µservice folders are source of truth | iac-registry | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Render latency (Helm/Kustomize/OpenTofu-plan; per µservice) | ≤1s | ≤5s | ≤10s | end-to-end from PR webhook to rendered diff |
| Apply latency p99 (per µservice; not waiting on workload health) | ≤90s | ≤5min | ≤15min | dependency-ordered apply across resources; bounded by k8s reconcile + OpenTofu refresh |
| Drift-detection cycle per cluster | — | ≤1h | — | continuous; one full cluster diff per hour minimum |
| Validator plan-preview p99 (PR-time) | ≤10s | ≤30s | ≤60s | per-µservice; runs in CI lane before merge |
| Rollback execution p99 | ≤30s | ≤2min | ≤5min | fast-revert when paired with SLO gate auto-rollback |
| Registry lookup p99 | ≤100ms | ≤300ms | ≤1s | catalog read for "what is currently deployed at pack-X" |

### Security

- All applies are SLSA L3 build-provenance attested per OpenSSF SLSA framework; the attestation is verified pre-apply by the iac-applier.
- Helm chart signing required via Sigstore Cosign (per docs/standards/observability-slo.md §"Supply-chain conformance"); unsigned charts refused.
- Terraform/OpenTofu state encrypted at rest with per-pack KMS keys (no cross-pack key usage); state stored in pack-pinned Postgres (the iac-state-index) + S3-compatible object storage.
- Per-µservice apply scope enforced: iac-app will refuse to apply a manifest that mutates resources outside the µservice's declared scope (Cedar policy `iac-isolation.md`).
- Secrets (cluster API credentials, Terraform-state encryption keys, ArgoCD admin tokens) follow the local-OpenBao SecretReference pattern; raw secrets never enter the repo, chat, or checkpoints.
- Apply-time RBAC: only the iac-applier ServiceAccount may mutate cluster state; humans use the read-only IaC plan-preview API (read-only) routinely; `iac apply` from human console requires JIT elevation through OpenBao + 2-person rule.

### Audit + Compliance

- Every `RenderRequested`, `RenderCompleted`, `ApplyStarted`, `ApplyCompleted`, `ApplyRolledBack`, and `DriftDetected` event emits an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028) consumed by the `audit-chain` µservice.
- Apply audit retention ≥ 6 years for pack-us-healthcare (HIPAA §164.316(b)(2)); ≥ 3 years for pack-kr (KR-FSS guidance); ≥ 2 years universally.
- Audit-chain seal latency ≤1s per apply event.
- Per-µservice apply ledger lives in iac-state-index Postgres with append-only constraint enforced at the schema level; backed up to S3 per pack.

### Availability + SLO

- Availability target: 99.95% monthly for the iac-applier's apply-event path; 99.9% for the iac-validator's plan-preview path.
- GitOps reconciler (ArgoCD or Flux) availability: 99.95% monthly per their respective published SLO postures, validated against oyatie's `observability` µservice SLO substrate (ADR-0139) — iac-app is itself gated by the SLO promotion gate.
- Drift-detection completeness: ≥99.5% of clusters polled per 1h cycle.
- RTO: ≤15min. RPO: ≤5min (last successful iac-state-index commit).

### Data residency

- IaC manifests are themselves `INTERNAL_ONLY` data; per-pack overlays containing tenant-bound configuration are tagged with the tenant's pack jurisdiction. Per-pack Terraform/OpenTofu state remains pack-pinned (pack-kr state stays in KR; pack-eu state stays in EU; etc.) per ADR-0117 + observability `data-residency.md`. Cross-pack state movement forbidden by default; the only exceptions are tenant-executed SCC paths inherited from observability's residency contract.

### DR posture (ADR-0343)

- Declared target: RTO <= 900 seconds and RPO <= 300 seconds, matching the existing Availability + SLO section. `manifest.json` currently lacks a `dr` block, so this PRD records the value that D-2 backfill must mirror.
- Applicable floors: HIPAA-2024 (3600/300, multi-region), SOC2-T2 (14400/900), ISO27001-2022 (14400/3600), KR-CSAP-v3.1 (3600/900, multi-region), and KR-PIPA-2023-amendment (14400/900) are represented by the declared pack set. Effective strictness remains RTO 900 seconds and RPO 300 seconds; multi-region is required for HIPAA/KR-CSAP-like packs even when apply execution stays single-writer.
- Failover runbook reference: `runbooks/registry-restore.md` for iac-state-index recovery, `runbooks/seaweedfs-volume-failover.md` for artifact buckets, and `runbooks/restore-drill-quarterly.md` for proof cadence.
- multi_region_active_active posture: false for `iac-applier` writes and state-lock ownership; render, validation, provenance read, and signed-attestation reads may run active-active because they are deterministic or read-only.
- WHY: infrastructure mutation must survive regional loss without creating two writers for the same cluster state; tenants get bounded recovery of deployability while avoiding split-brain applies.

### Capacity model (ADR-0340)

- Per-tenant baseline: D-2 has not populated `capacity_model`; until then the PRD-level baseline is service-unit based rather than tenant-compute based. `iac-renderer` and `iac-validator` allocate per-request CPU/RAM, while `iac-applier` and `iac-registry` allocate shared substrate capacity for all µservices in a cell.
- Scaling dimension: `per_capability` for render, plan-preview, apply, rollback, registry, and drift-detection workers; artifact and backup paths add `per_storage_gb` pressure through SeaweedFS and pgBackRest buckets.
- Cell placement class: Tier-1 because iac-app is shared deployment substrate and touches tenant-bound cluster state, SecretReference projections, iac-state-index rows, and pack-pinned OpenTofu state.
- Autoscaling boundaries: renderer/validator workers may scale horizontally with PR and drift volume; applier/rollback workers are bounded by the single-writer lock per target cluster; SeaweedFS bootstrap declares 3 masters, 6 volume servers, 3 filers, and 4 S3 gateways as the current M-tier substrate shape.
- WHY: the dominant tenant load is deployment/change volume rather than end-user traffic; the capacity model protects plan-preview responsiveness while keeping state mutation serialized where correctness requires it.

### Sustainability + cost attribution (ADR-0344)

- Every `RenderRequested`, `RenderCompleted`, `ApplyStarted`, `ApplyCompleted`, `ApplyRolledBack`, and `DriftDetected` audit-chain row also emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the tenant/product/capability/provider/cell/compliance_pack axes.
- Provider routing is carbon-aware for render, validation, artifact-provenance lookup, backup verification, and scheduled drift work. It is not carbon-routed for emergency rollback, stuck-apply recovery, or DR restore because those paths are incident-control actions.
- Tenant transparency surface: finops-portal infrastructure-cost view, per-tenant apply ledger, and provenance export for signed artifact storage and backup usage.
- WHY: CSRD, SB-253, SEC climate-disclosure, and customer FinOps require IaC changes to show the operational cost of deployment decisions, not only runtime workload cost.

### API versioning posture (ADR-0342)

- Public API version model: plan-preview, apply-state, provenance, and chart-signature validation contracts use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and proto3 version field.
- SDK semver model: iac-app SDKs use major.minor.patch, with major bumps only when a supported date-version carrier or generated type contract breaks.
- Support window: last N=3 public contract versions are supported for at least 180 days.
- Per-tenant pinning: supported for paid and regulated tenants whose deployment pipeline must remain frozen during audit windows; non-production sample_trial follows the platform default.
- Internal-mesh exemption: yes; ArgoCD/Flux/OpenTofu worker mesh traffic keeps ADR-0145 direct gRPC semantics and records the date-version only at external or replay boundaries.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-argocd`, `adapter-opentofu`, `adapter-helm`, `adapter-kustomize`, `adapter-postgres`, `rest`, `worker`, `sdk`, `app`. Backend-qualified adapters follow ADR-0105 Amendment 3 (`*-adapter-<backend>` pattern).

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `iac-renderer` | `iac-app-iac-renderer-{kernel,domain,usecase,api,adapter,adapter-helm,adapter-kustomize,adapter-opentofu,rest,worker,sdk,app}` | Render Helm/Kustomize/OpenTofu-plan deterministically from `microservices/<ms>/iac/` sources; emit `RenderCompleted` event with content-addressable digest | `ChartSource`, `ModuleSource`, `OverlaySource`, `RenderedManifest`, `ContentDigest` |
| `iac-validator` | `iac-app-iac-validator-{kernel,domain,usecase,api,adapter,rest,worker,app}` | Schema + policy + plan-preview + drift-diff; refuses applies that would mutate out-of-scope resources or violate Cedar policy | `PlanPreview`, `DriftReport`, `ValidationVerdict`, `PolicyViolation` |
| `iac-applier` | `iac-app-iac-applier-{kernel,domain,usecase,api,adapter,adapter-argocd,rest,worker,app}` | Apply orchestration: dependency-correct apply, retry, idempotency, per-µservice scope enforcement; mediates ArgoCD/Flux reconciler | `ApplyJob`, `ApplyResult`, `ApplyOrder`, `RetryBudget` |
| `iac-rollback` | `iac-app-iac-rollback-{kernel,domain,usecase,api,adapter,rest,worker,app}` | State-revert engine: revert to last-green apply; coordinate with observability SLO gate rollback primitive | `RollbackTarget`, `StateRevertPlan`, `RollbackVerdict` |
| `iac-registry` | `iac-app-iac-registry-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Versioned chart + module + overlay catalog; per-pack apply-state index; provenance store | `ChartRecord`, `ModuleRecord`, `OverlayRecord`, `ApplyStateIndex`, `Provenance` |

Naming justification — `iac-renderer`:

```
NAME: iac-app-iac-renderer-<layer>
JUSTIFICATION:
- microservice = iac-app: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = iac-renderer: primary BC for deterministic rendering (Helm/Kustomize/
  Terraform/OpenTofu plan). ADR-0056 v4.1 BC-optionality rule honoured: sibling BCs
  (iac-validator, iac-applier, iac-rollback, iac-registry) exist, justifying explicit
  BC token.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (ChartSource, ModuleSource,
    OverlaySource, RenderedManifest, ContentDigest). Zero I/O. Carries data_class
    annotations on every field per Bominal ADR-0028 + check-data-class lane.
  - domain: pure render math, dependency ordering, content-digest computation.
  - usecase (ADR-0106; replaces 'application'): orchestrators reading IaC sources
    + invoking adapter renderers + writing RenderCompleted events via ports.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-helm: backend-qualified adapter for Helm CLI / SDK; renders Charts.
  - adapter-kustomize: backend-qualified adapter for kustomize binary; resolves
    overlays.
  - adapter-opentofu: backend-qualified adapter for OpenTofu CLI; produces tfplan.
  - rest: HTTP handler layer; consumes -api.
  - worker: long-lived service handling render requests + emitting events.
  - sdk: client library; tenant + µservice integration.
  - app: composition root binary.
- exemptions claimed: none. Three -adapter-<backend> uses follow the canonical
  *-adapter-<backend> pattern per ADR-0105 Amendment 3.
```

Naming justification — `iac-validator`:

```
NAME: iac-app-iac-validator-<layer>
JUSTIFICATION:
- microservice = iac-app.
- bc-tokens = iac-validator: sibling BC for plan-preview + drift-diff + policy
  validation. Sibling BCs justify explicit BC token.
- layer = <layer>: per ADR-0105.
  - kernel: PlanPreview + DriftReport + ValidationVerdict entities + port traits
    (PolicyEvaluator, PlanComputer, DriftDiffer). Zero I/O.
  - domain: pure plan-diffing math; drift comparison.
  - usecase: orchestrate plan-preview + Cedar policy + emit verdicts.
  - api: typed I/O.
  - adapter: protocol-neutral impls.
  - rest: HTTP surface for plan-preview + drift queries.
  - worker: continuous drift-detection loop (≤1h cycle per cluster).
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `iac-applier`:

```
NAME: iac-app-iac-applier-<layer>
JUSTIFICATION:
- microservice = iac-app.
- bc-tokens = iac-applier: sibling BC for apply orchestration.
- layer = <layer>: per ADR-0105.
  - kernel: ApplyJob + ApplyOrder + RetryBudget entities + port traits
    (ClusterMutator, ReconcilerClient, ApplyEventEmitter).
  - domain: dependency-ordering algorithm; retry policy.
  - usecase: apply orchestrator.
  - api: typed I/O.
  - adapter: protocol-neutral impls.
  - adapter-argocd: backend-qualified adapter for ArgoCD reconciler API.
  - rest: HTTP surface.
  - worker: apply-worker (consumes ApplyRequested events; emits ApplyStarted /
    ApplyCompleted / ApplyRolledBack).
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `iac-rollback`:

```
NAME: iac-app-iac-rollback-<layer>
JUSTIFICATION:
- microservice = iac-app.
- bc-tokens = iac-rollback: sibling BC for state-revert engine.
- layer = <layer>: per ADR-0105.
  - kernel: RollbackTarget + StateRevertPlan + RollbackVerdict entities + ports.
  - domain: revert-plan computation; coordinate-with-SLO-gate logic.
  - usecase: rollback orchestrator.
  - api / adapter / rest / worker / app: per layer enum.
- exemptions claimed: none.
```

Naming justification — `iac-registry`:

```
NAME: iac-app-iac-registry-<layer>
JUSTIFICATION:
- microservice = iac-app.
- bc-tokens = iac-registry: sibling BC for versioned chart/module/overlay catalog +
  apply-state index + provenance store.
- layer = <layer>: per ADR-0105.
  - kernel: ChartRecord + ModuleRecord + OverlayRecord + ApplyStateIndex +
    Provenance entities + ports.
  - domain: catalog versioning logic; provenance link-validation.
  - usecase: catalog orchestrator.
  - api / adapter: per layer enum.
  - adapter-postgres: backend-qualified adapter for Postgres iac-state-index
    backend.
  - rest / worker / sdk / app: per layer enum.
- exemptions claimed: none.
```

Layer mapping per BC (canonical 13-layer enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-helm | adapter-kustomize | adapter-opentofu | adapter-argocd | adapter-postgres | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `iac-renderer` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | ✓ |
| `iac-validator` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `iac-applier` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | — | ✓ | ✓ | — | ✓ |
| `iac-rollback` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `iac-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |

Total crates introduced by this µservice: **40** (12 iac-renderer + 8 iac-validator + 9 iac-applier + 8 iac-rollback + 10 iac-registry; backend-qualified adapters and `sdk` only on BCs needing them per BNF justification).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `ChartSourceReader` | `iac-app-iac-renderer-kernel` | `-adapter-helm` (Helm CLI / SDK) | `INTERNAL_ONLY` (chart text) |
| `KustomizeOverlayReader` | `iac-app-iac-renderer-kernel` | `-adapter-kustomize` | `INTERNAL_ONLY` |
| `TerraformPlanComputer` | `iac-app-iac-renderer-kernel` | `-adapter-opentofu` (OpenTofu CLI) | `INTERNAL_ONLY` + `AUDIT` (plan output is an audit artifact) |
| `RenderEventEmitter` | `iac-app-iac-renderer-kernel` | `-adapter` (event bus) | `AUDIT` |
| `PolicyEvaluator` | `iac-app-iac-validator-kernel` | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `PlanComputer` | `iac-app-iac-validator-kernel` | `-adapter` | `INTERNAL_ONLY` + `AUDIT` |
| `DriftDiffer` | `iac-app-iac-validator-kernel` | `-adapter` (live-cluster API client) | `BEHAVIORAL_TENANT_PRODUCT` (cluster state per tenant) |
| `ClusterMutator` | `iac-app-iac-applier-kernel` | `-adapter` (Kubernetes API client) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` |
| `ReconcilerClient` | `iac-app-iac-applier-kernel` | `-adapter-argocd` (ArgoCD REST/gRPC client) | `BEHAVIORAL_TENANT_PRODUCT` |
| `ApplyEventEmitter` | `iac-app-iac-applier-kernel` | `-adapter` | `AUDIT` |
| `RollbackEventEmitter` | `iac-app-iac-rollback-kernel` | `-adapter` | `AUDIT` |
| `StateRevertPlanComputer` | `iac-app-iac-rollback-kernel` | `-adapter` | `AUDIT` |
| `ChartCatalogStore` | `iac-app-iac-registry-kernel` | `-adapter-postgres` (Postgres) | `INTERNAL_ONLY` |
| `ApplyStateIndexStore` | `iac-app-iac-registry-kernel` | `-adapter-postgres` | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` |
| `ProvenanceVerifier` | `iac-app-iac-registry-kernel` | `-adapter` (Sigstore Cosign + SLSA verifier) | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `check-data-class` LEAN lane refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `iac-app` MUST NOT import any other product µservice crate at any layer. All cross-product flows go through Workflow events (`ApplyStarted/Completed/RolledBack`, `RenderRequested/Completed`, `DriftDetected`) or Ontology reads/writes (`ChartRecord`, `ApplyStateIndex`, `Provenance`). LEAN-A2 CI lane enforces.

CI lanes that must green:

- ci governance gate `lean-a1` for --microservice iac-app is green in the branch-protected `presubmit` context — dependency-direction
- ci governance gate `lean-a2` for --microservice iac-app is green in the branch-protected `presubmit` context — cross-product-refusal
- ci governance gate `port-location` for --microservice iac-app is green in the branch-protected `presubmit` context — ports in kernel
- ci governance gate `layer-correctness` for --microservice iac-app is green in the branch-protected `presubmit` context — layer enum match
- ci governance gate `per-microservice-layout` for --microservice iac-app is green in the branch-protected `presubmit` context — ADR-0131 conformance
- ci governance gate `statelessness` for --microservice iac-app is green in the branch-protected `presubmit` context — renderer + validator are stateless; applier + rollback delegate to ArgoCD; registry is the only stateful component
- ci governance gate `shardability` for --microservice iac-app is green in the branch-protected `presubmit` context

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `RenderRequested` | PR with IaC change opens | `iac-renderer-worker` | render-state-machine |
| `RenderCompleted` | renderer emits rendered manifest digest | `iac-validator-worker`, `cloud-governance-evidence`, `audit-chain` | render-state-machine |
| `ApplyStarted` | applier picks up an apply job | `audit-chain`, `observability` (SLO gate metadata) | apply-state-machine |
| `ApplyCompleted` | applier confirms cluster state matches manifest | `cloud-governance-evidence`, `audit-chain`, `observability` (release-pointer-advance signal) | apply-state-machine |
| `ApplyRolledBack` | rollback engine reverts to prior apply | `audit-chain`, `observability` (rollback signal), `grafana-oncall` | apply-state-machine |
| `DriftDetected` | drift-detector finds live ≠ git | `audit-chain`, `grafana-oncall`, `observability` (alerting signal) | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `MicroserviceRegistered` | `tenancy` | `iac-registry` | discover the new µservice; ensure it has IaC scaffolding under `microservices/<ms>/iac/` |
| `EligibilityChanged` (verdict=eligible) | `observability` (per ADR-0139) | `iac-applier` | a µservice's SHA is eligible for promotion → apply that SHA's IaC to the target environment |
| `RollbackExecuted` (production-tier) | `observability` | `iac-rollback` | a release pointer rolled back → iac-app reverts IaC state to the prior apply |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `ChartRecord{microservice, chart_name, version, digest, signed_by}` | `chart_for→Microservice` | `iac-registry` | Ed25519 |
| `ApplyStateIndex{microservice, pack, environment, current_sha, applied_at}` | `applied_at→Pack` | `iac-registry` | Ed25519 |
| `Provenance{artifact_digest, slsa_attestation, sigstore_signature, builder_id}` | `attests→ChartRecord` | `iac-registry` | Ed25519 |
| `DriftReport{microservice, pack, drift_score, detected_at}` | `drift_for→ApplyStateIndex` | `iac-validator` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `iac-registry` | `filter(active=true)` to enumerate µservices requiring IaC coverage |
| `ReleasePointer` | `iac-applier` | `where(microservice=X, environment=Y)` to know which SHA to apply |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| ArgoCD | OSS GitOps reconciler | Cluster reconciliation, app-of-apps pattern, drift detection | `argo-cd.readthedocs.io` |
| Flux | OSS GitOps controller | Same; CNCF graduate | `fluxcd.io/docs` |
| Terraform Cloud | HashiCorp SaaS IaC platform | State storage, plan/apply orchestration, policy-as-code (Sentinel) | `developer.hashicorp.com/terraform/cloud-docs` |
| OpenTofu | Apache-2.0 fork of Terraform | Self-hosted state, OSS license | `opentofu.org/docs/` |
| Spacelift | Commercial IaC orchestration | Multi-IaC (Terraform + Pulumi + CloudFormation + Kubernetes) + policy-as-code | `docs.spacelift.io` |
| Atlantis | OSS Terraform PR automation | PR-time plan/apply | `runatlantis.io` |
| Env0 | Commercial IaC orchestration | Similar to Spacelift; deeper tenant cost-management | `docs.env0.com` |
| Pulumi Service | Commercial IaC SaaS | Stack management, real-language IaC | `pulumi.com/docs/intro/console/` |
| Crossplane | OSS Kubernetes-native IaC (CRD-based) | Cloud-resource management via k8s API | `crossplane.io/docs/` |

Key parity gaps to close (ordered by priority):

1. **Meta-IaC pipeline integration** — Spacelift / Env0 / Terraform Cloud cover Terraform but do NOT canonicalize Helm + Kustomize + Terraform under one apply pipeline; oyatie's differentiator is one pipeline across all three.
2. **Cryptographic provenance per apply** — none of the commercial offerings ship SLSA L3 + Sigstore attestation as a default invariant; oyatie's audit-chain integration makes this default.
3. **SLO-gate integration** — Spacelift / Env0 don't refuse applies based on a downstream burn-rate signal; iac-app × observability does (per ADR-0139).
4. **Self-hosted with no vendor coupling** — Spacelift / Env0 / Pulumi Cloud / Terraform Cloud are SaaS; oyatie hosts everything on the same Grafana / ArgoCD / OpenTofu stack.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Render latency | ≤1s | ≤5s | ≤10s | per µservice; deterministic |
| Apply latency | ≤90s | ≤5min | ≤15min | per µservice; bounded by k8s reconcile + OpenTofu refresh |
| Validator plan-preview at PR-time | ≤10s | ≤30s | ≤60s | feeds into CI lane decision |
| Drift-detection cycle per cluster | — | ≤1h | — | continuous; one full diff per hour minimum |
| Rollback execution | ≤30s | ≤2min | ≤5min | when paired with SLO gate |
| Registry lookup | ≤100ms | ≤300ms | ≤1s | catalog read |
| ArgoCD reconciler throughput | — | 1000+ apps/cluster | — | per published ArgoCD benchmarks |
| OpenTofu plan-throughput | — | 50+ concurrent plans/cluster | — | per OpenTofu CI footprint guidance |

Error budget:
- Monthly error budget for iac-applier: 0.05% (≈22min/month).
- Burn-rate alarm on iac-app's own SLOs (per ADR-0139 self-observability): 14.4× burn over 1h triggers page.
- Error budget policy: `microservices/iac-app/runbooks/error-budget-policy.md` (extends observability template).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale: iac-renderer + iac-validator workers are stateless (re-derivable from git + cluster state); iac-applier workers are stateless beyond in-flight job IDs; iac-rollback is stateless. iac-registry uses Postgres for the catalog + apply-state index (one Postgres cluster per pack region); Terraform/OpenTofu state stored in pack-pinned object storage.

**Active-active compatibility**: `stateless-compatible` for renderer / validator / applier / rollback workers. Postgres iac-state-index is primary + read-replica per pack region (no cross-pack replication; pack-pinned per ADR-0117).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Concurrent renders | 50 | 500 | renderer queue depth > 30s |
| Concurrent applies | 20 | 200 | applier queue depth > 60s |
| Tracked µservices in registry | 100 | 10,000 | registry read latency p99 > 300ms |
| Drift-detection clusters | 10 | 100 | drift-cycle cadence > 1h |
| iac-state-index Postgres rows | 10M | 1B | Postgres CPU > 70% |

Scale-out policy:
- Kubernetes HPA: renderer + validator + applier workers scale on CPU > 70%; min 2, max 50 replicas.
- Postgres: vertical scale up to OCI VM.Standard.E4 16-core; horizontal scale via per-pack-region instance.
- Pre-warmed pool: 2 standby renderer pods + 2 standby applier pods; cold-start ≤500ms.

Cross-region story:
- M01 launch: pack-kr only (single OCI ap-seoul-1).
- Post-M01 expansion: per-pack iac-registry Postgres + per-pack Terraform/OpenTofu state buckets; cross-pack replication forbidden per residency contract.

Sharding:
- Apply jobs partition by `microservice`; applier shards by µservice without coordination.
- `check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A new µservice's `iac/helm/<chart>/Chart.yaml` + `values.yaml` lands in the registry within 5 minutes of PR merge | end-to-end test under `microservices/iac-app/tests/e2e/registry-onboarding.rs` |
| AC-02 | Render is deterministic: re-running on the same input produces an identical content digest | integration test `microservices/iac-app/tests/integration/render-determinism.rs` |
| AC-03 | Plan-preview at PR-time surfaces a structured drift report | integration test |
| AC-04 | Apply refuses to mutate resources outside the µservice's declared scope | Cedar policy unit test + integration |
| AC-05 | Rollback reverts an apply within ≤2min when invoked | timed e2e drill |
| AC-06 | Drift detector finds a manually-mutated cluster resource within 1h | e2e injection drill |
| AC-07 | SLSA L3 attestation verified pre-apply | integration test against signed + unsigned chart |
| AC-08 | All Layer-A IaC components (ArgoCD + OpenTofu + Helm-controller + Kustomize-controller) deploy clean against a kind cluster | CI lane `iac-app-iac-smoke` |
| AC-09 | ci governance gate `per-microservice-layout` for --microservice iac-app is green in the branch-protected `presubmit` context | ADR-0131 lane |
| AC-10 | ci governance gate `authority-cohesion` is green in the branch-protected `presubmit` context | ADR-0123 lane; HG-CLOUD-IAC registered |
| AC-11 | Apply latency p99 ≤ 5min per µservice (excluding workload-health waits) | load test under `microservices/iac-app/tests/load/apply-latency.rs` |
| AC-12 | Drift-detection cycle per cluster ≤ 1h validated under nominal load | observability self-SLO |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | ArgoCD vs Flux as the canonical GitOps reconciler at M01 | axis-iac-app + ops-sre-reliability | resolved in IP-001 (ArgoCD chosen; Flux supported for tenant choice via adapter pattern) |
| 2 | OpenTofu version-pinning cadence (LTS vs trailing-stable) | axis-iac-app | resolved in IP-003 (LTS pin per docs/standards/observability-slo.md) |
| 3 | iac-state-index Postgres: single cluster per pack vs single global cluster | axis-iac-app + cloud-secrets | resolved in IP-008 (per-pack pinned per ADR-0117) |
| 4 | Cross-µservice apply: forbidden by default, or allowed under explicit DAG declaration? | architecture-governance | resolved in policy/iac-isolation.md — forbidden by default; explicit cross-µservice DAGs require architecture-governance approval + Cedar policy entitlement |
| 5 | Should iac-app apply its own IaC (bootstrap paradox)? | axis-iac-app | resolved in IP-015 — bootstrap via cloud-k8s + cloud-secrets minimum-viable; then iac-app applies itself thereafter (parallel to observability self-observability) |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0117 | Cloud-native infrastructure | residency authority |
| ADR-0120 | Rust-first on-prem tooling | tooling authority |
| ADR-0121 | On-prem k8s stack | substrate authority |
| ADR-0123 | Hyperscaler maturity claim gate | HG-CLOUD-IAC registers here |
| ADR-0139 | Agentic SLO-gated promotion | downstream consumer of ApplyExecuted events |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 | No-grouping policy | iac-app stands alone, not a platform member |
| ADR-0133 | Industry-best-practice conformance | competitor parity authority |
| ADR-0171 | Multi-cluster federation (ArgoCD ApplicationSets + Cluster API) | this PRD's canonical multi-cluster surface |

## Multi-Cluster Federation Addendum (per ADR-0171)

Per ADR-0171 (2026-05-18), iac-app adopts a three-component multi-cluster federation substrate as the canonical scaling shape from ≥12 clusters at M02:

### Component 1 — ArgoCD ApplicationSets (application deployment across N clusters)

- Every µservice's `iac/helm/<ms>/` chart references a single `ApplicationSet` declaration.
- Cluster-list / cluster-decision-resource generators fan out to each target cluster with per-pack value overrides.
- Per-pack overlays live as `values-<pack>.yaml` under `microservices/<ms>/iac/helm/<ms>/`.
- ArgoCD federation control plane lives in a dedicated meta-pack ("federation") — NOT a tenant data residency boundary; carries only ApplicationSets, CAPI controllers, and routing config.

### Component 2 — Cluster API (CAPI) (cluster lifecycle)

- Cluster create / upgrade / delete declarative via Cluster API CRDs.
- Per-environment providers:
  - On-prem sovereign packs → `cluster-api-provider-metal3` (bare-metal) per ADR-0117.
  - EU pack → `cluster-api-provider-openstack` or `cluster-api-provider-azure` per ADR-0049.
  - KR pack → sovereign on-prem; metal3.
  - Cloud governance GPU pools → cell-isolated CAPI clusters per ADR-0009.
- CAPI provider versions tracked in `registry/cluster-api-providers.json` (new registry entry; M02 deliverable).

### Component 3 — Federation control plane (cross-region routing)

- GeoDNS + multi-cluster Ingress pattern adapted from GKE Multi-Cluster Ingress to on-prem.
- Tenant DNS resolves to per-pack GeoDNS pool; per-tenant residency (per ADR-0010) constrains which pack receives the request.
- Failover within a pack: warm-standby DR cluster receives traffic via DNS-failover (TTL ≤60s).
- Failover across packs: ONLY for non-residency-bound tenants; otherwise fail-closed per ADR-0008 data-use boundary.

### Migration trigger

This addendum applies from M02 graduation (fleet ≥12 clusters). At M01-foundation scale (≤6 clusters) the kustomize+kubectl-context per-pack model continues; the federation tier is not deployed until M02 prep.

### Federation SLO

- Federation control plane availability: 99.95% (one nine below the platform — federation outage degrades new-deploy velocity, not tenant-facing traffic).
- ApplicationSet sync latency p99: ≤2min from PR-merge to first cluster acknowledged apply.
- CAPI cluster-provision time: ≤30min from `kubectl apply` to ready-cluster.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 is superseded for this surface: branch-protected `presubmit` is the canonical blocking CI authority; retired local CLI verifier output is not production or merge authority.
- ADR-0347 — every `governance-*` CI lane prefix in the Oyatie corpus RENAMES to `governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `governance-no-cloud-governance-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- ADR-0349 — GitHub Actions `presubmit` is the live CI authority until owned ci runner cutover; ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `governance-github-actions-presubmit-continuity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-github-actions-ci-jcasc-only`, and `governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `iac-app` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/iac-app/modules/<context>/<primitive>/`; `iac-app` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 6 module pin(s) across 2 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
