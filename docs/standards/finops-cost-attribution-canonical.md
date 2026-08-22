---
contract: finops-cost-attribution-canonical
authored: 2026-05-18
canonical_authority: ADR-0199
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
related_adrs:
  - ADR-0174
  - ADR-0184
  - ADR-0186
  - ADR-0196
  - ADR-0197
  - ADR-0198
  - ADR-0199
status: canonical-base
authorities_cited:
  - FOCUS 1.3 — FinOps Open Cost & Usage Specification ratified 2025-12-05 (https://focus.finops.org/focus-specification/)
  - FinOps Foundation framework 1.1 (https://www.finops.org/)
  - OpenCost (CNCF Incubating; https://opencost.io/)
  - Spotify Engineering — OpenCost for per-team / per-product cost attribution at K8s scale
  - Adobe — OpenCost in production for Adobe Experience Platform
  - AWS Builders Library — Tagging best practices for cost allocation
---

# FinOps cost attribution canonical (per-tenant labels + OpenCost + FOCUS 1.3)

## Why this policy exists

ADR-0174 establishes the chargeback formula. ADR-0199 names the runtime
substrate. This standards doc consolidates the operational contract every
µservice + every cloud resource must honor for per-tenant cost
attribution.

## The canonical tenant label block (mandatory)

Every Kubernetes workload **and** every cloud resource MUST carry the
following labels:

| Label                       | Required | Cardinality target | Source                                |
|-----------------------------|----------|--------------------|---------------------------------------|
| `oya.io/tenant-id`          | yes      | bounded; ULID      | tenancy µservice                      |
| `oya.io/cost-center`        | yes      | closed enum        | µservice manifest                     |
| `oya.io/workload-class`     | yes      | closed enum: `app` / `batch` / `gpu` / `regulatory` | µservice manifest |
| `oya.io/regulatory-pack`    | yes      | closed enum: `generic` / `kr` / `eu` / `us-healthcare` / `us-financial` / `us-public-sector` | µservice manifest |

**Privacy + cardinality discipline (per ADR-0186):**

- `tenant-id` is a ULID (26 chars, lexicographically sortable); contains
  no PII.
- `cost-center`, `workload-class`, `regulatory-pack` are closed enums;
  cardinality is bounded.
- Adding a new `cost-center` value requires an ADR amendment to ADR-0174's
  vocabulary registry (`registry/finops/cost-tag-vocabulary.yaml`).

## Cloud-resource tagging (OpenTofu modules)

Every cloud resource provisioned via `microservices/cloud-iac/iac/` MUST
carry:

| Tag                       | Value source                          |
|---------------------------|---------------------------------------|
| `oya:tenant-id`           | OpenTofu input variable               |
| `oya:cost-center`         | OpenTofu input variable               |
| `oya:workload-class`      | OpenTofu input variable               |
| `oya:regulatory-pack`     | OpenTofu input variable               |

Note: cloud-resource tag keys use `:` separator (cloud convention);
Kubernetes label keys use `/` separator (K8s convention). The semantic
fields are 1:1 across the two surfaces.

## Helm helper (canonical)

The library helper `oya.tenantCostLabels` emits the K8s label block:

```yaml
{{- define "oya.tenantCostLabels" -}}
oya.io/tenant-id: {{ .Values.costAttribution.tenantId | default "shared" | quote }}
oya.io/cost-center: {{ required "costAttribution.costCenter is required" .Values.costAttribution.costCenter | quote }}
oya.io/workload-class: {{ required "costAttribution.workloadClass is required" .Values.costAttribution.workloadClass | quote }}
oya.io/regulatory-pack: {{ .Values.costAttribution.regulatoryPack | default "generic" | quote }}
{{- end }}
```

The helper is `required`-guarded — Helm rendering FAILS if `cost-center`
or `workload-class` are absent. (Parent wiring task: extend
`_oya-helpers/templates/_helpers.tpl`.)

Per-µservice `Chart.yaml` + `values.yaml`:

```yaml
# Chart.yaml
dependencies:
  - name: helpers
    version: 0.1.0
    repository: "file://../../../../governance/iac/helm/_oya-helpers"

# values.yaml
costAttribution:
  costCenter: "axis-foundry"
  workloadClass: "app"
  regulatoryPack: "generic"  # overlay supplies for kr / eu / etc.
```

Every Deployment / StatefulSet / DaemonSet pod spec MUST include:

```yaml
metadata:
  labels:
    {{ include "oya.labels" $ | nindent 4 }}
    {{ include "oya.tenantCostLabels" $ | nindent 4 }}
```

## OpenCost aggregation

- **OpenCost 1.110.0** (CNCF incubating; Apache 2.0).
- Federated to Mimir per ADR-0186 Stage 2.
- Aggregation keys configured via the `OPENCOST_AGGREGATION_KEYS`
  environment variable:
  `tenant-id,cost-center,workload-class,regulatory-pack`.
- Custom-pricing config provided via ConfigMap (per-pack overlay supplies
  per-region rates).

## FOCUS 1.3 normalization

- All cost data normalizes to **FOCUS 1.3** schema before chargeback.
- FOCUS 1.3 adoption notes:
  - **Contract Commitment dataset** — separates contract terms from
    cost/usage rows; used for committed-use discounts on Karpenter
    NodePools.
  - **Allocation columns** — declare how costs split across workloads.
- FOCUS export lands in SeaweedFS bucket
  `finops-focus-export-shared-<env>` per ADR-0196.

## Cost anomaly alerts (canonical)

Three classes (Prometheus rules shipped via the OpenCost Helm chart):

| Alert                       | Trigger                                                   | Severity | Routes to                |
|-----------------------------|-----------------------------------------------------------|----------|--------------------------|
| TenantCostAnomalySpike      | per-tenant > 1.5× rolling-7d average over 1 h window      | warning  | ops-finops               |
| TenantBudgetHeadroomLow     | per-tenant budget remaining < 10 %                        | warning  | ops-finops + tenant-cs   |
| TenantBudgetExhausted       | per-tenant budget remaining = 0                           | critical | ops-finops (page)        |

Runbooks live at `docs/runbooks/finops-<alert-slug>.md`.

## CI gates this policy is enforced by

| Gate                                          | Lane mode | Behavior                                              |
|-----------------------------------------------|-----------|-------------------------------------------------------|
| `check-tenant-cost-labels-coverage`       | advisory  | renders Helm + reports workloads missing the label block |

Strict promotion follows when the per-µservice coverage backlog reaches
zero.

## Chargeback emission (per ADR-0174)

Quarterly:

- Compute per-tenant chargeback via the ADR-0174 formula.
- Emit `class: FinOpsQuarterlyReport` to the audit chain, signed by
  ops-finops key.
- Customer-facing rollup at the `cost-center` level via the billing API.
- Internal rollup at the `microservice` level via the internal API.

## Worked example

Tenant `t-1234`, Pro tier, period 2026-Q2:

- Cluster cost (OpenCost) — `oya.io/tenant-id=01HX...` aggregated:
  $4,200.
- Foundry capability invocations — 1.2M × $0.00015: $180.
- Audit-chain emit — 18M rows × $0.000002: $36.
- Storage (`class-tenant-pii`) — 240 GB × $0.20: $48.
- Customer-success credit: −$200.
- **Total**: $4,264.

## References

- ADR-0199 — per-tenant cost attribution + FinOps substrate (this doc's
  canonical authority).
- ADR-0174 — FinOps cost-attribution + chargeback formula.
- ADR-0186 — observability backplane (metrics path).
- ADR-0184 — storage tier layering (where cost data lives at rest).
- FOCUS 1.3 — https://focus.finops.org/focus-specification/.
- OpenCost docs — https://opencost.io/docs/.
