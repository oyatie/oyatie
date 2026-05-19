---
doc_class: Product-Requirements
owner: ops-sre-reliability
status: accepted-design-anchor
surface: ops-dashboard-control-center
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

## Exit claim boundary

This PRD is a design/spec control surface. Runtime exit remains blocked until implementation crates, policy tests, deployment manifests, SLO windows, restore evidence, and signed evidence-pack verification land.
