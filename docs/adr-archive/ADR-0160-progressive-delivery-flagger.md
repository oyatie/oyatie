---
id: ADR-0160
status: Superseded
deciders: council-architecture, axis-cloud-k8s, axis-observability, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0515]
supersession_note: "Flagger superseded by Argo Rollouts per D10 ruling; ADR-0515 is the canonical CI/CD + progressive-delivery ADR. D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-12/P1."
related: [ADR-0110, ADR-0114, ADR-0121, ADR-0124, ADR-0139, ADR-0148, ADR-0157, ADR-0158]
related_specs:
  - /specs/agentic-slo-gated-promotion.json
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0160 — Progressive Delivery via Flagger 1.x (canary + blue-green + A/B), ArgoCD-integrated, SLO-gated promotion

## Status

Accepted (2026-05-18). Promotes Flagger 1.x as the canonical progressive-delivery controller. Integrates with ArgoCD (per ADR-0121) and the SLO-gated promotion contract (ADR-0139).

## Context

ADR-0114 (canary observability + rollback) named *what* the progressive-delivery shape should be — automated 1% → 5% → 25% → 100% ramp with SLO-driven gate at each step + automatic rollback on regression. ADR-0139 fixed the agentic SLO-gated promotion contract — every promotion step requires observed SLO compliance over a defined evaluation window before advancing.

What has NOT been fixed is the **concrete controller** that implements this shape on a running Kubernetes cluster. Without an explicit choice, every µservice pack ends up implementing its own promotion script. That violates the cohesion thesis (ADR-0001), makes per-µservice canary semantics drift, and creates a forever debugging surface during rollouts.

The hyperscaler precedent has converged on two open-source controllers:

- **Flagger** (Flux project; CNCF) — declarative `Canary` CRD; supports canary / blue-green / A/B mirroring; integrates with Istio, Linkerd, NGINX, Contour, Gloo, App Mesh, Kuma; weighted-traffic shifts via service-mesh primitives; SLO-driven gating via Prometheus / Datadog / Dynatrace queries.
- **Argo Rollouts** (Argo project; CNCF) — declarative `Rollout` CRD; similar surface; tightly coupled with ArgoCD.

The choice matters because we already adopted:

- ArgoCD for GitOps deploy (ADR-0121).
- Istio for east-west mesh (ADR-0148).
- The agentic SLO-gated promotion contract (ADR-0139).

The integration surface — `Canary` / `Rollout` CRD <→ Istio `VirtualService` <→ Prometheus SLO queries — must compose cleanly with all three.

## Decision

Oyatie adopts **Flagger 1.x** as the canonical progressive-delivery controller for every workload µservice that promotes through the dev → staging → production lifecycle.

### Operational shape

1. **One `Canary` CRD per workload Deployment.** Each µservice's `iac/helm/<ms>/templates/canary.yaml` declares the canary spec.
2. **Istio integration.** Flagger writes Istio `VirtualService` weights directly (Istio is already in-cluster per ADR-0148). No re-implementation of L7 routing.
3. **ArgoCD integration.** ArgoCD applies the `Canary` CRD; Flagger then performs the actual ramp. ArgoCD does not see weight changes (Flagger owns them); ArgoCD sees only the Helm-rendered CRD.
4. **SLO-gated promotion (ADR-0139).** Each canary step's `metrics:` block references PromQL queries against the observability µservice's Prometheus / Mimir backend. Defaults:
   - `request-success-rate >= 99.5%` over 5-minute evaluation window.
   - `request-duration-p99 <= µservice-declared-budget` (per ADR-0145 cross-µservice latency budget).
   - `audit-chain-emission-success-rate >= 99.9%` (ADR-0003 invariant).
   - Per-µservice custom SLO queries declared in `slos/canary-gates.openslo.yaml`.
5. **Promotion ladder.**
   - **Canary mode (default for workload µservices)** — 5% → 10% → 25% → 50% → 100%; SLO gate at each step (5-minute window).
   - **Blue-green mode (for stateful µservices like workflow-engine)** — full replica side-by-side; cutover after SLO gate passes; old replica retained 1 hour for rollback.
   - **A/B mirroring (for shadow-test releases)** — production traffic mirrored to new version; new version's response is discarded; used for behavioral diff testing.
6. **Webhook gates.** Per-step webhooks call back into the api-gateway / governance µservice to verify (a) audit-chain seals are sealing, (b) Cedar policy load succeeded, (c) tenant-routing tables haven't drifted. Webhook failure halts promotion.
7. **Automatic rollback.** SLO breach during any step triggers immediate weight rollback to 0% + audit-chain seal `PromotionRolledBack` + slack-mcp incident notification (per ADR-0114 incident contract).
8. **Per-cell scope.** A canary is scoped to a single cell (per ADR-0009). Cross-cell promotion is sequential: cell-1 fully promoted → cell-2 starts canary → etc. Sovereign-pinned cells run their own canary loop.

### Why Flagger over Argo Rollouts

- **Istio integration depth.** Flagger has a longer history with Istio (Weaveworks → Flux); `VirtualService` weight management is its native primitive. Argo Rollouts supports Istio but treats it as one of many integrations.
- **SLO-driven gate is first-class.** Flagger's `metrics:` block is a first-class CRD field. Argo Rollouts has analysis templates but they require external `AnalysisRun` CRDs.
- **Multi-mesh support.** If the on-prem packs ever evaluate Linkerd or Cilium Service Mesh (per ADR-0148 deferred), Flagger supports both natively.
- **ArgoCD compatibility.** Flagger does not depend on Argo Rollouts; runs alongside ArgoCD; ArgoCD applies the `Canary` CRD, Flagger executes.

### Per-µservice declaration

Every workload µservice MUST ship:

- `iac/helm/<ms>/templates/canary.yaml` — the `Canary` CRD.
- `slos/canary-gates.openslo.yaml` — the SLO queries that gate promotion.
- `incident-response.md#progressive-delivery` — runbook for canary failure.

A CI lane `oya gate validate progressive-delivery-canary` validates (a) every workload µservice's `Canary` CRD references a real `Deployment`, (b) the `metrics:` block references SLO queries that exist in the observability µservice's catalog, (c) the rollback runbook section exists.

## Alternatives considered

### Alternative A — Argo Rollouts

- **Pros:** native ArgoCD integration; rich UI in ArgoCD dashboard; mature; same Argo project we already use.
- **Cons:** SLO-driven gate requires `AnalysisRun` CRD (extra layer); Istio integration is one-of-many rather than first-class; multi-mesh story weaker.
- **Rejected because:** Flagger's first-class Istio + first-class SLO-driven gate matches ADR-0139 + ADR-0148 better. The integration-depth gap is dispositive.

### Alternative B — Custom canary scripts (per-µservice shell / Rust)

- **Pros:** zero new operator; full control.
- **Cons:** 33 per-µservice scripts is 33 ways for promotion semantics to drift; SLO-gate logic re-implemented per µservice; rollback automation per µservice; this is the historical anti-pattern.
- **Rejected because:** cohesion thesis (ADR-0001) + uniform SLO-gate contract (ADR-0139) forbid per-µservice scripts.

### Alternative C — Spinnaker

- **Pros:** Netflix-class deploy tool; rich pipeline model; canary analysis (Kayenta) is a known good.
- **Cons:** Java + Groovy stack adds operational complexity; conflicts with the Rust-first toolchain (ADR-0120); Spinnaker's GitOps story is weaker than ArgoCD.
- **Rejected because:** ADR-0120 + ADR-0121 already chose ArgoCD as the deploy controller.

### Alternative D — Flagger 1.x (this ADR)

- **Pros:** first-class Istio (ADR-0148); first-class SLO-driven gate (ADR-0139); compatible with ArgoCD (ADR-0121); CNCF graduated; mature.
- **Cons:** another operator to install; per-cell installation footprint; learning curve for ops.
- **Accepted.**

### Alternative E — Manual promotion (human-driven 5% → 100%)

- **Pros:** zero automation cost.
- **Cons:** human operators slow promotion to days; SLO-gate evaluation is hand-eye; rollback latency is human-paged-out; defeats the agentic-SLO-gated contract (ADR-0139).
- **Rejected because:** ADR-0139 mandates agentic / automated promotion.

## Consequences

### Positive

1. **One canonical progressive-delivery controller across the fleet.** No per-µservice promotion-script drift.
2. **SLO-gated promotion enforced structurally.** Each canary step gated on observed SLO compliance; ADR-0139 contract closed end-to-end.
3. **Istio integration native.** Flagger writes `VirtualService` weights directly; no double-routing layer.
4. **Automatic rollback within minutes.** SLO breach detected within evaluation window (5 min default); rollback within 30 seconds; total worst-case impact < 6 minutes.
5. **ArgoCD GitOps clean.** ArgoCD applies the CRD; Flagger executes; no GitOps drift.
6. **Per-cell scope aligns with sovereign-tenant pin** (ADR-0158). Each cell's canary loop runs independently.

### Negative

1. **Flagger operator per cell.** Each cell installs Flagger; ops adds Flagger to the on-call rotation.
2. **`Canary` CRD overhead per µservice.** Every workload µservice authors + maintains the CRD + SLO gates.
3. **SLO query latency budget.** Flagger calls Prometheus at each evaluation; ~50 ms per query × N queries per step adds up. Mitigated by per-cell Prometheus.
4. **Stateful µservices need blue-green not canary.** Canary mode assumes stateless or backward-compatible state migrations. Blue-green is required for workflow-engine et al. PRD declares mode.

### Operational

1. Flagger 1.x Helm chart shipped at `microservices/cloud-iac/iac/helm/flagger/Chart.yaml` (Companion).
2. Per-cell Flagger installation handled by the cloud-k8s pack.
3. CI lane `oya gate validate progressive-delivery-canary` enforces per-µservice `Canary` CRD existence + SLO-gate wiring.
4. Per-µservice `slos/canary-gates.openslo.yaml` template stamped from `docs/standards/canary-slo-gates-canonical.md`.
5. Promotion telemetry surfaces to ops portal: per-µservice / per-cell canary state, last promotion, last rollback.

## References

- Flagger documentation — https://fluxcd.io/flagger/
- Flagger Canary CRD spec — https://fluxcd.io/flagger/usage/how-it-works/
- Argo Rollouts (alternative) — https://argoproj.github.io/argo-rollouts/
- Kayenta automated canary analysis — Netflix/Google joint project, https://github.com/spinnaker/kayenta
- Istio traffic management for canary — https://istio.io/latest/docs/concepts/traffic-management/
- OpenSLO specification — https://openslo.com/
- Stripe progressive delivery — Stripe engineering blog "Online migrations at scale" (2017).
- Netflix canary analysis — Netflix technology blog "Automated Canary Analysis" (2018).
- Google SRE Workbook — Chapter 16 (Canarying Releases).
- ADR-0110 — ChangeSet state machine.
- ADR-0114 — canary observability + rollback (this ADR is the controller for ADR-0114's shape).
- ADR-0121 — onprem K8s stack (ArgoCD already in-stack).
- ADR-0124 — own merge queue (interacts with ChangeSet promotion).
- ADR-0139 — agentic SLO-gated promotion (this ADR is the operationalization).
- ADR-0148 — Istio service mesh (Flagger writes `VirtualService`).
- ADR-0157 — api-gateway tier (webhook gates).
- ADR-0158 — multi-region disposition (per-cell canary scope).
