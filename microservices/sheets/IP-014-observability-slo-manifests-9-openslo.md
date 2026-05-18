---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-014-observability-slo-manifests-9-openslo
status: pending
owner: axis-sheets + axis-observability
acceptance_lanes: [openslo-validate, oya-vcs-promotion-readiness]
depends_on: [IP-013]
---

# IP-014: 9 OpenSLO v1.0 manifests — sheets self-SLOs

## Intent

Author 9 OpenSLO v1.0 manifests for sheets self-SLOs at `microservices/sheets/slos/`. Per ADR-0130 + ADR-0131 §"SLO authoring at microservices/<ms>/slos/*.openslo.yaml mandatory before any µservice promotes past dev". Consumed by the observability µservice's SLO engine which gates Sheets's promotion.

## ChangeSet boundary

Nine OpenSLO manifests:
- `sheet-open-latency.openslo.yaml`
- `cell-edit-render-latency.openslo.yaml`
- `recalc-100k-cells-latency.openslo.yaml`
- `recalc-1m-cells-latency.openslo.yaml`
- `collab-cursor-sync-latency.openslo.yaml`
- `export-xlsx-latency.openslo.yaml`
- `chart-render-latency.openslo.yaml`
- `crdt-merge-no-silent-loss.openslo.yaml` (correctness; 100% target)
- `formula-engine-correctness.openslo.yaml` (correctness; 100% target)

All authored in `microservices/sheets/slos/` per IP-014.

## Acceptance Gates

```bash
oya gate validate openslo-validate --manifests 'microservices/sheets/slos/*.openslo.yaml'
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| openslo schema validate | every file conforms to OpenSLO v1.0 schema |
| recording-rule generation | observability engine generates Mimir recording rules |
| budget calc | error budget targets resolve to expected window |
| burn-rate alert wires | alert rules emit to AlertManager + Grafana OnCall |
| correctness-SLO Sev-1 page | crdt-merge-no-silent-loss + formula-engine-correctness fire Sev-1 on any breach |

## Halt Conditions

- Any manifest fails OpenSLO v1.0 schema validation — STOP.
- Recording rule fails to load in Mimir — STOP.

## Next IP

[`IP-015-hg-sheets-registration-and-branch-protection.md`](IP-015-hg-sheets-registration-and-branch-protection.md)

## References

- ADR-0130 Agentic SLO-gated promotion.
- ADR-0131 §"SLO authoring mandatory".
- OpenSLO v1.0 spec — `github.com/OpenSLO/OpenSLO`.
- microservices/observability/PRD.md.
- Google SRE Workbook ch. 4.
