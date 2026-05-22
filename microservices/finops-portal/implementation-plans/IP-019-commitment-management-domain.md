---
ip_id: IP-019
microservice: finops-portal
bounded_context: commitment-management
layer: domain
related_adrs: [ADR-0199, ADR-0252, ADR-0276]
---

# IP-019 — commitment-management domain

## Goal

Track tenant's committed-use discounts (CUDs) / reserved instances / savings plans. Immutable
ledger (append-only) per ADR-finops-portal-004. TrueTime-ordered commit per ADR-0252 for
financial finalisation.

## Crate

`oya-finops-portal-commitment-management-domain`.

## Acceptance

- Append-only ledger; cryptographic seal per entry.
- Per-commitment lifecycle (PROPOSED → APPROVED → ACTIVE → EXPIRED).
- Audit event `CommitmentDiscountApplied`.
