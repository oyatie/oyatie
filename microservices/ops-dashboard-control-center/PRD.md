---
doc_class: Product-Requirements
owner: ops-sre-reliability
status: accepted-design-anchor
surface: ops-dashboard-control-center
related_adrs:
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
---
# Ops Dashboard / Control Center PRD

## Purpose

FD-001 includes an operator surface, not a side dashboard. Ops Dashboard / Control Center gives SRE, release, support, and compliance operators one governed control surface for incident response, deployment approvals, rollback decisions, cluster health, tenant isolation posture, policy/audit decisions, signed evidence packs, bootstrap/recovery workflows, and localization-aware escalation.

## Scope

In scope:
- incident command with severity, communications, remediation decisions, and post-incident evidence;
- deployment approval, hold, freeze-window, progressive rollout, and rollback decisions;
- cluster/node/cell/service-mesh/bootstrap/restore health views;
- tenant lifecycle, quota, isolation, residency, policy, and SLO posture views;
- evidence-pack export and audit-chain replay receipts;
- regional-pack operational runbook hooks, including KR escalation linkage;
- safe operator actions with idempotency, approval, Cedar authorization, and audit-chain seals.

Out of scope for this design anchor:
- direct mutable SSH host management;
- bypassing GitOps, Oya VCS, Cedar, OpenBao, or audit-chain controls;
- claiming runtime maturity before implementation evidence, SLO windows, and restore evidence are green.

## Users

- On-call SRE: declares incidents, reads cluster health, records mitigation, and requests recovery workflows.
- Release manager: approves deployment progression, records holds, and authorizes rollback decisions.
- Tenant support operator: views tenant isolation posture and evidence refs without cross-tenant data leakage.
- Compliance operator: exports signed evidence packs and reviews policy/audit decisions.
- Localization operator: follows regional escalation runbooks without changing canonical-base behavior.

## Acceptance criteria

- AC-01: every mutating API requires an idempotency key, authenticated operator identity, Cedar authorization, and audit-chain seal reference.
- AC-02: deployment approval and rollback decisions are separate command types with explicit rationale and actor identity.
- AC-03: tenant posture reads include only authorized tenant scope and cite evidence refs instead of raw cross-tenant data.
- AC-04: cluster health reads distinguish observed signals from operator decisions.
- AC-05: evidence-pack export returns a ticket plus audit seal, not an opaque side effect.
- AC-06: KR localization runbook hooks exist as operational attachments while canonical base remains jurisdiction-neutral.
- AC-07: design/spec maturity gate finds PRD, manifest, IPs, ADR refs, OpenAPI, AsyncAPI, Proto3, capabilities, Cedar policy, SLOs, runbooks, threat model, failure modes, residency, cost/FinOps, audit evidence emission, tenant isolation, operational boundaries, and implementation-ready acceptance criteria.

## Non-functional requirements

### DR posture per ADR-0343

- Target: RTO 1800 seconds and RPO 300 seconds per `manifest.json#dr`; the hot operator `manifest.json#rpo_rto` surface tightens rollback and recovery actions to RTO 300 seconds and RPO 60 seconds.
- Compliance floors: HIPAA-2024 requires 3600/300 with multi-region, SOC2-T2 requires 14400/900, ISO27001-2022 requires 14400/3600, and public-sector FedRAMP/IL5 overlays do not add numeric floors in `specs/compliance-pack-floors.json`. The manifest DR target is stricter than those listed floors, and the hot operator target is stricter still.
- Failover runbook reference: `microservices/ops-dashboard-control-center/multi-region.md`, `runbooks/deployment-rollback.md`, `runbooks/incident-command.md`, `runbooks/admin-action-rollback.md`, and `runbooks/kr-localization-escalation.md`.
- Multi-region active-active posture: enabled for incident command, rollback decisions, evidence-pack export requests, tenant-isolation posture reads, and recovery workflow commands.
- Why: operators use this surface during outages, so the control center must stay available while the failing cell or deployment is the object being investigated.

### Capacity model per ADR-0340

- Per-tenant baseline: 0.18 vCPU, 384 MiB RAM, 2 GiB operator/evidence metadata storage, 6 Postgres connections, 6 Valkey connections, and 40 outbound HTTP sockets.
- Scaling dimension: `per_request`, with separate burst budgets for health polling, evidence export, rollback approvals, and recovery workflow starts.
- Cell placement class: Tier-1 for operator command and recovery surfaces; read-only fleet health panels can run Tier-2 replicas.
- Autoscaling boundaries: minimum 2 active replicas per control cell, maximum 12 replicas per tenant slice, and evidence export workers capped at 4 per tenant to protect audit-chain throughput.
- Why: steady operator traffic is small, but incidents create sharp read and command bursts across cluster health, tenant posture, rollout, rollback, and evidence views.

### Sustainability and cost attribution per ADR-0344

- Every audit-chain row emitted by incident declaration, severity change, deployment approval, rollback, cluster-health observation, tenant isolation posture view, policy review, evidence export, and recovery workflow carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Carbon-aware provider routing: no for incident command, rollback, recovery workflow, and deployment approval commands; yes for evidence-pack export, cost-read panels, and background fleet report generation when SLO burn allows.
- Tenant cost surface: FinOps Portal owns the billable transparency view, while this control center displays bounded operator cost signals from cloud-finops for incident and recovery decisions.
- Why: CSRD, SB-253, and SEC climate-disclosure reporting require operator-plane cost visibility, but live rollback and recovery controls cannot be delayed by carbon-preferred placement.

### API versioning posture per ADR-0342

- Public API model: YYYY-MM-DD carrier triplet across `Oyatie-Version`, `/v/<YYYY-MM-DD>/ops-dashboard-control-center/...`, and proto3 `oyatie_version`.
- SDK model: generated SRE, release-manager, and compliance-operator SDKs use semantic `major.minor.patch` versions.
- Support window: the last 3 public API versions remain supported for at least 180 days.
- Per-tenant pinning: yes, for tenant support tooling, regulated operator workstations, and external GRC integrations.
- Internal mesh exemption: yes, preserving ADR-0145 direct gRPC for service-control, audit-chain, cloud-finops, policy, and recovery workflow calls.

## Exit claim boundary

This PRD is a design/spec control surface. Runtime exit remains blocked until implementation crates, policy tests, deployment manifests, SLO windows, restore evidence, and signed evidence-pack verification land.
