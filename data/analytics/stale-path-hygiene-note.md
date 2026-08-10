---
doc_class: JudgmentNote
title: Stale microservices/analytics contract+SLO path hygiene (wave-1/2 prep)
status: Accepted
owner_team: council-analytics
date: 2026-08-10
related_artifacts:
  - data/analytics/catalog/contracts.json
  - data/analytics/catalog/oya-analytics-api.json
  - data/analytics/catalog/slos.json
  - data/analytics/catalog/runbooks.json
  - data/analytics/catalog/capabilities.json
  - data/analytics/catalog/dashboards.json
  - data/analytics/catalog/scorecards.json
  - data/analytics/catalog/cedar-policies.json
  - data/analytics/catalog/pack-overlays.json
  - data/analytics/manifest.json
  - data/analytics/contracts/
  - data/analytics/capabilities/
  - data/core/analytics-usecase/
  - data/ports/analytics-api/
ssot_todo: free-capability-data-analytics-contract
---

# Stale path hygiene note (`data/analytics/**` contracts + SLO anchors)

## Chesterton challenge

`microservices/analytics/...` citations were left as strangler compatibility after the capability tree landed under `data/analytics/` (and SLOs under `data/observability/slos/analytics/`). On this tip the legacy directory is **gone** (`test ! -d microservices/analytics`). Keeping dead absolute citations fails day-2 contract/SLO operability; inventing missing trees to satisfy greps would violate YAGNI and the wave envelope.

## Wave-1 scope (retargeted)

| Surface | Change |
|---|---|
| `data/analytics/catalog/contracts.json` | Contract paths → `data/analytics/contracts/{openapi-v1.yaml,asyncapi-v1.yaml,analytics.proto}` |
| `data/analytics/catalog/oya-analytics-api.json` | Same three contract paths |
| `data/analytics/contracts/openapi-v1.yaml` | Dashboard SLO prose → `data/observability/slos/analytics/dashboard-api-latency.openslo.yaml` |
| `data/analytics/capabilities/{dashboard-query,audit-log-query}.json` | `slo_anchor` → `data/observability/slos/analytics/...` |
| `data/analytics/catalog/slos.json` | `directory` → `data/observability/slos/analytics/` |
| `data/core/analytics-usecase` | Add cross-tenant refusal unit test (caller≠query tenant → `CrossTenantAccessDenied`) |
| `data/ports/analytics-api` | Contract path constants + integration test; retarget `Cargo.toml` metadata off `oya/analytics/contracts/` |

Verified destinations for every retargeted cite:

- `data/analytics/contracts/openapi-v1.yaml`
- `data/analytics/contracts/asyncapi-v1.yaml`
- `data/analytics/contracts/analytics.proto`
- `data/observability/slos/analytics/dashboard-api-latency.openslo.yaml`
- `data/observability/slos/analytics/audit-log-query-latency.openslo.yaml`

## Wave-2 scope (retargeted)

Analytics has **no** workflow-studio `openslo_ref` / `explainer_path` fields; the analogous surfaces are catalog/manifest SLO+artifact paths. Retarget only where destination files/dirs exist under `data/**`:

| Surface | Change |
|---|---|
| `catalog/{capabilities,runbooks,dashboards,scorecards,cedar-policies,pack-overlays}.json` | Paths → `data/analytics/...` |
| `catalog/clickhouse-analytics-helm.json` | Helm chart path → `data/analytics/iac/helm/clickhouse-analytics/` |
| `manifest.json` | Residency packs, openslo_manifests, dashboards, scorecards, contract public surfaces → verified `data/...` |
| `iac/.../prometheus-rule.yaml` | Alert runbook labels → `data/analytics/runbooks/...` |
| `runbooks/{keeper-quorum-recovery,mv-lag-triage}.md` | Companion cites → verified homes |
| `data/core/analytics-usecase` | Extend tenancy refusal tests to audit-log + billing rollup |
| `data/ports/analytics-api` | Contract test asserts files exist under `data/` root |

## openslo_ref / explainer_path (Wave-2 deferral note)

Filled: N/A (schema fields not present on analytics catalog/templates).  
Deferred: inventing template-style `openslo_ref`/`explainer_path` keys — out of envelope and YAGNI. Service-level OpenSLO homes already live under `data/observability/slos/analytics/` (wave-1 + manifest openslo_manifests retarget).

## Left as intentional legacy (destination missing — do not invent)

| Cite | Why deferred |
|---|---|
| `microservices/analytics/PRD.md` | No `data/analytics/PRD.md` |
| `microservices/analytics/ARCHITECTURE.md` | No `data/analytics/ARCHITECTURE.md` |
| `microservices/analytics/specs/IP-007-tenant-dashboard-api.md` | No specs/IP-007 under data/analytics |
| `microservices/analytics/decisions/ADR-AN-001..005` | No `data/analytics/decisions/` tree |
| `microservices/analytics/iac/helm/analytics-app/` | Chart not present (only clickhouse-analytics) |
| `microservices/analytics/{backfill-replay,capacity-model}.md` + `values-cluster-2.yaml` + IP-001 specs cite | Destination files absent |
| Interior IP dossier SCOPE/VERIFY prose under `data/analytics/IPs/` | Out of priority; leave until judged land |

## Explicit non-goals

- No hubs, `Cargo.lock`, `specs/reachability*`, product-protocol-policy, restack onto `dev`.
- No mass rewrite of `IP-*.md` / inventing PRD/ARCHITECTURE/decision trees.
- Envelope `data/**` only.
