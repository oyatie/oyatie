---
id: ADR-0199
status: Superseded
deciders: council-architecture, ops-finops, axis-cloud-iac, axis-observability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0701]
related: [ADR-0064, ADR-0131, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0174, ADR-0240-sovereign-cloud-per-regional-pack, ADR-0184, ADR-0186, ADR-0196, ADR-0197, ADR-0198]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0199 — Per-tenant cost attribution + FinOps substrate (OpenCost + FOCUS 1.3)

## Status

Accepted (2026-05-18). Mandates per-tenant cost attribution via a fixed
label block on every workload + cloud resource, aggregation through
OpenCost (CNCF incubating), normalization to FOCUS 1.3, and chargeback
emission to the tenant billing portal per ADR-0174.

## Context

ADR-0174 (FinOps cost-attribution + chargeback) named the cost-tag block,
chargeback formula, and quarterly regulator-evidence cadence, but deferred
the runtime substrate. ADR-0184 (storage tier layering) and ADR-0186
(observability backplane) wire the metrics path. ADR-0198 (Karpenter
autoscaling) emits per-workload-class node metrics. This ADR ties them
together into a runnable per-tenant cost-attribution substrate.

The hyperscaler reference shape for FinOps at multi-tenant SaaS scale:

- **Stripe** — per-tenant cost-of-goods-sold tracking via internal
  cost-allocation pipeline; the same shape that became FOCUS.
- **Spotify** — public engineering blog on OpenCost adoption for
  per-team / per-product cost attribution at K8s scale.
- **Adobe** — OpenCost in production for per-tenant cost reporting in
  Adobe Experience Platform.
- **FOCUS** — FinOps Open Cost & Usage Specification 1.3 (ratified
  2025-12-05); the cross-cloud schema cloud bills + cluster costs
  normalize to.

Anti-patterns this ADR forecloses:

1. CSV cost exports — no real-time signal, no anomaly detection, no
   chargeback automation.
2. Per-tool-specific tagging — each cloud (AWS, GCP, Azure) and each
   K8s tool (Kubecost, OpenCost, Karpenter) consuming different label
   conventions; cost lineage breaks at boundaries.
3. Per-µservice ad-hoc cost dashboards — no aggregated tenant signal;
   no anomaly detection; no chargeback automation.

## Decision

### D-1. Canonical tenant label block (CI-enforced)

Every Kubernetes workload + cloud resource MUST carry:

| Label                       | Cardinality | Required | Source                          |
|-----------------------------|-------------|----------|---------------------------------|
| `oya.io/tenant-id`          | per pod / resource | yes | tenancy µservice (ULID)    |
| `oya.io/cost-center`        | per µservice | yes | µservice manifest             |
| `oya.io/workload-class`     | per pod / resource | yes | manifest (app / batch / gpu / regulatory) |
| `oya.io/regulatory-pack`    | per pod / resource | yes | µservice manifest (`generic` / `kr` / `eu` / `us-healthcare` / `us-financial`) |

**Privacy + non-PII discipline**: `tenant-id` is a ULID and carries no
PII. Cost-center, workload-class, and regulatory-pack are closed enums.
Cardinality is bounded per ADR-0186 metric-cardinality discipline.

### D-2. Helm helper `oya.tenantCostLabels`

A new helper in `microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`
emits the canonical label block:

```yaml
{{- define "oya.tenantCostLabels" -}}
oya.io/tenant-id: {{ .Values.costAttribution.tenantId | default "shared" | quote }}
oya.io/cost-center: {{ required "costAttribution.costCenter is required" .Values.costAttribution.costCenter | quote }}
oya.io/workload-class: {{ required "costAttribution.workloadClass is required" .Values.costAttribution.workloadClass | quote }}
oya.io/regulatory-pack: {{ .Values.costAttribution.regulatoryPack | default "generic" | quote }}
{{- end }}
```

The helper is `required`-guarded — Helm rendering FAILS if `cost-center`
or `workload-class` are absent. The check
`oya-check-tenant-cost-labels-coverage` (this batch) scans rendered Helm
output and reports per-µservice coverage in advisory mode (sweep-up
strict promotion follows when all µservices migrate).

### D-3. OpenCost as the aggregation engine

- **OpenCost 1.110.0** (CNCF incubating; Apache 2.0). Chosen over Kubecost
  for OSS preference per ADR-0173. Kubecost's commercial-only features
  (federated multi-cluster cost view, scheduled reports) are not
  required at oyatie's current scale; if they become required, OpenCost
  is the OSS substrate Kubecost's commercial features layer on top of.
- Federated to Mimir per ADR-0186 Stage 2: OpenCost exporter scrapes
  cluster metrics, joins with cloud-pricing data, emits cost metrics
  back into Mimir with the tenant label block intact.
- Custom-pricing config provided via ConfigMap for on-prem (per-pack
  overlay supplies per-region rates).

### D-4. FOCUS 1.3 normalization

- All cost data (OpenCost cluster metrics + cloud-provider bills + on-prem
  rate cards) are normalized to **FOCUS 1.3** schema before chargeback
  computation.
- FOCUS 1.3 enhancements adopted:
  - **Contract Commitment dataset** — separates contract terms (start/
    end, remaining units) from cost/usage rows. Useful for committed-use
    discounts on autoscaler node pools.
  - **Allocation columns** — declare how costs split across workloads.
- The FOCUS export pipeline lands in SeaweedFS bucket
  `oya-finops-focus-export-shared-<env>` per ADR-0196; consumers
  (tenant billing portal, ops-finops dashboards, regulator quarterly
  emit) read from there.

### D-5. Cost anomaly alerts

Three alert classes (per the Prometheus rules in the OpenCost Helm chart):

- **TenantCostAnomalySpike** — per-tenant cost > 1.5× rolling-7d average
  over a 1 h window. Severity warning; routes to ops-finops.
- **TenantBudgetHeadroomLow** — per-tenant budget remaining < 10 %.
  Severity warning; routes to ops-finops + tenant success.
- **TenantBudgetExhausted** — per-tenant budget remaining = 0. Severity
  critical; pages ops-finops.

### D-6. Chargeback emission (per ADR-0174)

- Quarterly per-tenant chargeback report is emitted to the audit chain
  with `class: FinOpsQuarterlyReport`, signed by ops-finops key.
- Customer-facing rollup at `cost_center` level via the billing API;
  internal rollup at `microservice` level via the internal API.

### D-7. Tagging discipline for cloud resources

- All cloud resources tagged `oya:tenant-id`, `oya:cost-center`,
  `oya:workload-class`, `oya:regulatory-pack` via OpenTofu module
  conventions (per ADR-0240-sovereign).
- A CI gate (`oya-check-tenant-cost-labels-coverage`) renders Helm
  templates and reports any workload without the full label block.

### D-8. Backup retention is FinOps-aware

- `crates/oya-check-backup-retention-discipline/` cross-validates
  declared retention against the µservice's `regulatory-pack` label.
  Backup cost is attributable per tenant via the same label block.

## Alternatives considered

### (a) Kubecost commercial — REJECTED

- **Pros:** richer UI; federated multi-cluster view; mature dashboards.
- **Cons:** commercial licensing (per ADR-0173 vendor-lock-in); the OSS
  scope is exactly OpenCost (donated to CNCF by Kubecost themselves).
  No need to pay for what is OSS-available.
- **Rejected:** vendor lock-in; OSS preference.

### (b) Cast.ai FinOps — REJECTED

- **Pros:** integrated autoscaler + FinOps surface.
- **Cons:** vendor lock-in; commercial-only; ties cost data to a single
  vendor's autoscaler decisioning.
- **Rejected:** vendor lock-in.

### (c) Apptio Cloudability — REJECTED

- **Pros:** enterprise BI for cloud cost.
- **Cons:** vendor lock-in; punitively-priced for SaaS scale; FOCUS-export
  is post-hoc rather than native.
- **Rejected:** vendor lock-in + cost.

### (d) CSV cost exports + homegrown joiner — REJECTED

- **Pros:** zero ops; one engineer can hack it together.
- **Cons:** no real-time signal; no anomaly detection; bus-factor risk;
  no FOCUS conformance.
- **Rejected:** not hyperscaler-grade.

### (e) **CHOSEN:** OpenCost + FOCUS 1.3 + canonical tenant label block +
mandatory Helm helper + CI-enforced advisory check.

## Consequences

### Positive

- Per-tenant cost is queryable in real time via the canonical label
  block.
- One schema (FOCUS 1.3) across cloud bills + on-prem + K8s metrics; no
  ETL drift.
- OSS substrate aligns with ADR-0173 (vendor lock-in avoidance).
- Backup-retention enforcement uses the same label block.

### Negative

- Every µservice must declare `cost-center` + `workload-class` in its
  Helm values + manifest. Mitigation: advisory CI gate this batch;
  strict promotion when fleet migrates.
- OpenCost's federated multi-cluster view requires a future ADR (or a
  commercial Kubecost upgrade); oyatie's per-cell cost view today is
  per-cluster aggregated via Mimir federation.

### Neutral

- The label block is the same across Helm + cloud-IaC + audit-chain;
  one vocabulary across the substrate.

## In-house roadmap

Per the user directive "wherever possible, support in-house tech stack —
like AWS, Google, Microsoft, Oracle" (2026-05-18), the FinOps substrate
splits cleanly: the open standard (FOCUS) is permanent; the cost
aggregator (OpenCost) is adapter-wrapped today, with a Phase 2 in-house
build for the tenant-billing UX layer.

### FOCUS 1.3 — KEEP (open standard, in-house rebuild never)

- FOCUS is the FinOps Foundation's open specification — equivalent in
  status to OpenAPI for cost data. Every hyperscaler (AWS, Google,
  Azure, Oracle, Alibaba) is implementing native FOCUS exports.
- **No in-house rebuild.** Oyatie consumes FOCUS as a schema standard;
  oyatie's own cost-export pipeline emits to FOCUS 1.3 so downstream
  consumers (tenant billing portal, regulator-evidence emit) speak
  the standard.
- This matches how AWS, Google, Microsoft, Oracle all consume FOCUS:
  as a publishing standard, not a vendor product.

### OpenCost — Phase 0 / Phase 2 ladder

#### Phase 0 (TODAY)

- OpenCost 1.110 via Helm at `microservices/observability/iac/helm/
  opencost/`.
- Federated to Mimir per ADR-0186 Stage 2.
- Custom-pricing config per-pack overlay.
- Aggregation keys: `tenant-id,cost-center,workload-class,regulatory-pack`.

#### Phase 1 — adapter hardening (M02-M03 horizon)

- Conformance set for `oya-cloud-finops-kernel`'s cost-aggregator
  trait.
- FOCUS-export pipeline graduated to strict mode.
- Anomaly detector tuning per workload class.

#### Phase 2 — in-house `oya-finops-portal` (~Q2 2027 target)

- Build `oya-finops-portal` as the in-house tenant-billing UX layer.
- **Scope:** the differentiated UX layer — tenant-facing invoice
  presentation, drill-down dashboards, anomaly explanation,
  cost-allocation policy editor, FOCUS-export downloads, regulator-
  evidence on-demand.
- **NOT scope:** the underlying cost-aggregation logic (cluster cost
  joining with cloud bills) — that remains OpenCost-backed. The
  portal sits on top of the OpenCost data plane.
- **Build trigger** (one of):
  - Tenant-billing UX requirements exceed what OpenCost UI can
    deliver (per ADR-0185 Workflow Studio client stack quality bar).
  - Differentiated tenant-tier features needed (credit ledgers,
    committed-use discounts UX, customer-success negotiation
    workflow) that don't belong in the OpenCost upstream.
  - Customer-facing branding + workflow integration with Workflow
    Studio requires UX coherence with oyatie's design system.
- **Parallel to:** AWS Cost Explorer + Billing Console (in-house),
  Google Cloud Billing (in-house), Azure Cost Management (in-house),
  Oracle Cost Analysis (in-house). Hyperscalers all ladder from
  "expose the raw numbers" to "build the tenant-billing UX layer
  in-house"; oyatie pre-stages the seam now.

### What stays "in-house" today (Phase 0)

- **The tenant label block** (`oya.io/tenant-id`, etc.) is oyatie-
  authored, CI-enforced, mandatory. No upstream tool dictates this
  shape.
- **The chargeback formula** (ADR-0174) is Oya-authored.
- **The anomaly thresholds** (1.5× rolling 7d, < 10 % headroom) are
  Oya-authored and tuned per workload class.
- **The regulatory-pack-aware retention discipline** for backups is
  Oya-authored (this batch).

The Phase 2 portal builds on top of these in-house surfaces.

### Cross-substrate dependency

- Phase 2 `oya-finops-portal` consumes:
  - `storage-object-store-kernel` (FOCUS exports land in object
    store).
  - `oya-shared-backup-kernel` retention metadata (backup cost
    component).
  - `oya-cloud-finops-kernel` cost-aggregator trait (OpenCost data
    plane today, in-house aggregator possible if/when needed).

The portal is the latest item on the in-house ladder; the underlying
substrate (labels + traits + standards) is already in-house today.

## Industry sources

- **OpenCost project** — <https://opencost.io/>; CNCF incubating since
  2024-10-25.
- **OpenCost 2026 roadmap** — <https://www.cncf.io/blog/2026/01/12/opencost-reflecting-on-2025-and-looking-ahead-to-2026/>.
- **Spotify** — public engineering blog on OpenCost adoption for
  per-team / per-product cost attribution at K8s scale.
- **Adobe** — OpenCost in production for Adobe Experience Platform.
- **FOCUS 1.3** — <https://focus.finops.org/focus-specification/>;
  ratified 2025-12-05; <https://focus.finops.org/>.
- **FinOps Foundation framework 1.1** — adopted by FOCUS as the
  vocabulary anchor.
- **AWS cost-allocation tagging best practices** — *Tagging best
  practices for cost allocation*, AWS Builders Library.
- **Microsoft Cloud Adoption Framework** — cost management discipline
  module.

## Verification

- Helm chart at `microservices/observability/iac/helm/opencost/` renders.
- The `oya.tenantCostLabels` helper will be added by parent wiring (this
  batch declares it in the ADR + emits the parent-wiring work item).
- `crates/oya-check-tenant-cost-labels-coverage/` scans rendered Helm
  output and reports per-µservice coverage; advisory mode this batch.
- `crates/oya-check-backup-retention-discipline/` validates declared
  retention against regulatory-pack floors.
- FOCUS 1.3 export lands in SeaweedFS `oya-finops-focus-export-shared-
  <env>` and is consumable by ops-finops dashboards.

## Footnotes (versions verified 2026-05-18)

- OpenCost 1.110.0: <https://opencost.io/blog/>.
- FOCUS 1.3 ratified 2025-12-05: <https://focus.finops.org/focus-specification/>.
- Kubecost vs OpenCost relationship: <https://opencost.io/blog/cncf-incubation/>.
