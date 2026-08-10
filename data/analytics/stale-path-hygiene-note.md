---
doc_class: JudgmentNote
title: Stale microservices/analytics contract+SLO path hygiene (wave-1 prep)
status: Accepted
owner_team: council-analytics
date: 2026-08-10
related_artifacts:
  - data/analytics/catalog/contracts.json
  - data/analytics/catalog/oya-analytics-api.json
  - data/analytics/catalog/slos.json
  - data/analytics/contracts/
  - data/analytics/capabilities/
  - data/core/analytics-usecase/
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

Verified destinations for every retargeted cite:

- `data/analytics/contracts/openapi-v1.yaml`
- `data/analytics/contracts/asyncapi-v1.yaml`
- `data/analytics/contracts/analytics.proto`
- `data/observability/slos/analytics/dashboard-api-latency.openslo.yaml`
- `data/observability/slos/analytics/audit-log-query-latency.openslo.yaml`

## Left as intentional legacy (destination missing or out of wave-1 priority — do not invent)

Interior docs outside wave-1 priority still cite `microservices/analytics/...` with no judged retarget this wave (IPs, runbooks, broad catalog pack/decision/dashboard rows, root `manifest.json` scorecard/PRD cites). Leave until a later judged land creates the artifact or deletes the cite.

## Explicit non-goals

- No hubs, `Cargo.lock`, `specs/reachability*`, product-protocol-policy, restack onto `dev`.
- No mass rewrite of `IP-*.md` / full runbook corpus.
- Envelope `data/**` only.
