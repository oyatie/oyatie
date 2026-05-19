---
doc_kind: implementation-plan
id: IP-001
title: Connect retirement design readiness bundle
status: Accepted
owner_team: council-architecture
related_adrs: [ADR-0134, ADR-0135]
---

# IP-001: Connect Retirement Design Readiness Bundle

## Intent

Make the retiring `connect` umbrella auditable as a design/spec surface while preserving the rule that no new product runtime scope lands here.

## Scope

- Add machine-readable manifest coverage for retirement status, ADRs, contracts, SLO, policy, and audit events.
- Define read-only retirement status contracts.
- Document tenant, residency, cost, threat, failure, and operational boundaries for the temporary umbrella.

## Acceptance

- The gate can verify all required design/spec surfaces under `microservices/connect`.
- Contracts expose only retirement status and readiness evidence.
- Policy forbids new runtime product ownership under `connect`.
- The artifacts do not claim operational maturity, product completeness, or deployed scale.
