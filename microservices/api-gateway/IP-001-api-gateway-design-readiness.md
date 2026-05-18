---
doc_kind: implementation-plan
id: IP-001
title: API gateway design readiness bundle
status: Accepted
owner_team: axis-network
related_adrs: [ADR-0157, ADR-0182, ADR-0183]
---

# IP-001: API Gateway Design Readiness Bundle

## Intent

Close the design/spec surface for the dedicated north-south edge tier without claiming runtime readiness. The implementation path remains separate from this evidence bundle.

## Scope

- Bind API gateway contracts across OpenAPI, AsyncAPI, and proto3.
- Define the edge admission capability, tenant policy, OpenSLO target, runbook, threat model, failure modes, FinOps model, and operational boundaries.
- Keep coarse edge authorization at the gateway while fine-grained authorization stays with workload services.

## Acceptance

- `manifest.json` references ADR authority, contracts, capability, SLO, IP, residency packs, and audit-chain events.
- `contracts/` contains OpenAPI, AsyncAPI, and proto3 surfaces for edge admission and denial events.
- `policy/tenant-scope.cedar` denies cross-tenant and cross-cell admission at the edge.
- `runbooks/edge-admission-regression.md`, `threat-model.md`, `failure-modes.md`, `cost-budget.md`, and `operational-boundaries.md` explain operator-facing boundaries without asserting production evidence.
