---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-012
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + axis-observability
related_adrs: [ADR-0105, ADR-0135, ADR-0130, ADR-0131]
doc_status: published
---

# IP-012 — OpenSLO manifests + Grafana dashboards

## Intent

Author per-BC OpenSLO manifests at `slos/*.openslo.yaml` and Grafana dashboards at `dashboards/*.json`. Wire to observability burn-rate evaluator.

## Scope

- OpenSLO: feed-render-p99, post-create-p99, vote-cast-p99, search-p99, moderation-p99, kb-publish-p99.
- Dashboards: post-throughput, vote-rate, moderation-queue-depth (this IP).
- Burn-rate alerts via Alertmanager + Grafana OnCall.

## Deliverables

- 6 OpenSLO manifests.
- 3 Grafana dashboards JSON.
- Alertmanager routing config.

## Acceptance

- Manifests validate against OpenSLO schema.
- Dashboards render in Grafana.
- Burn-rate evaluator reads community SLOs.
- Alert routing tested end-to-end.

## Owner

axis-community + axis-observability.
