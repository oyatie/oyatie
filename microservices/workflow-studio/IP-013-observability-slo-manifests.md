---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-013-observability-slo-manifests
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + axis-observability
acceptance_lanes: [openslo-validate, oya-vcs-promotion-readiness]
depends_on: [IP-012]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: workflow-studio OpenSLO manifests

## Intent

Author OpenSLO v1.0 manifests for workflow-studio's self-SLOs at `microservices/workflow-studio/slos/`. Per ADR-0139 + ADR-0131 §"SLO authoring at microservices/<ms>/slos/*.openslo.yaml mandatory before any µservice promotes past dev". Consumed by the observability µservice's SLO engine which gates Studio's promotion.

## ChangeSet boundary

Six OpenSLO manifests (one per primary BC + cluster aggregate):
- `editor-rest-availability.openslo.yaml`
- `editor-rest-latency.openslo.yaml` (save round-trip)
- `collab-crdt-merge-latency.openslo.yaml`
- `collab-crdt-no-silent-loss.openslo.yaml` (correctness SLI)
- `license-gate-cedar-availability.openslo.yaml`
- `editor-tti-cdn-availability.openslo.yaml`

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml` | create |
| `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml` | create |
| `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml` | create |
| `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml` | create |
| `microservices/workflow-studio/slos/license-gate-cedar-availability.openslo.yaml` | create |
| `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml` | create |

## Code Shape

`slos/editor-rest-availability.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-workflow-studio-editor-rest-availability
  displayName: Workflow Studio Editor REST — Availability
  labels:
    microservice: workflow-studio
    bounded_context: visual-canvas
    tier: external-facing
spec:
  service: oya-workflow-studio-visual-canvas-rest
  indicator:
    metadata:
      name: editor-rest-availability-sli
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            type: prometheus
            spec:
              query: 'sum(rate(http_request_total{service="oya-workflow-studio-visual-canvas-rest",status!~"5.."}[5m]))'
        total:
          metricSource:
            type: prometheus
            spec:
              query: 'sum(rate(http_request_total{service="oya-workflow-studio-visual-canvas-rest"}[5m]))'
  objectives:
    - target: 0.9995  # 99.95% GA per PRD §"Availability + SLO"
      displayName: 99.95% successful editor REST requests
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
oya gate validate openslo-validate \
  --manifests 'microservices/workflow-studio/slos/*.openslo.yaml'
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness \
  --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| openslo schema validate | every file conforms to OpenSLO v1.0 schema |
| recording-rule generation | observability engine generates Mimir recording rules from these manifests |
| budget calc | error budget targets resolve to expected window |
| burn-rate alert wires | alert rules emit to AlertManager + Grafana OnCall |

## Halt Conditions

- Any manifest fails OpenSLO v1.0 schema validation — STOP.
- Recording rule fails to load in Mimir — STOP.

## Next IP

[`IP-014-branch-protection-and-hyperscaler-gates.md`](IP-014-branch-protection-and-hyperscaler-gates.md)

## References

- ADR-0139 Agentic SLO-gated promotion.
- ADR-0131 §"SLO authoring mandatory".
- OpenSLO v1.0 spec — `github.com/OpenSLO/OpenSLO`.
- microservices/observability/PRD.md.
- Google SRE Workbook ch. 4 (Service Level Objectives).

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-013-observability-slo-manifests.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-013-observability-slo-manifests.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-013-observability-slo-manifests.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
