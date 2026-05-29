---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: S-015
related_packs: [sox-404]
date: 2026-05-21
---

# Approval Routing Matrix

CLM contracts route through approval per a tenant-configured matrix that combines:

- Contract type
- Financial value
- Risk classification (per IP-026 ClauseDeviation classification)
- Counterparty risk
- Materiality (per SOX-404)
- Jurisdiction
- Pack overlays active

## Default matrix

The µservice ships with a default matrix; tenants override per their organisation.

| Contract financial value | Default approvers required (N-of-M) |
|---|---|
| < $10k | Author + Contracts Manager (1-of-1) |
| $10k - $100k | Contracts Manager + Procurement (1-of-1 each) |
| $100k - $1M | Contracts Manager + Procurement + Legal Review (1-of-1 each) |
| $1M - $10M | Above + VP Procurement + VP Legal (1-of-1 each) |
| $10M - $100M | Above + General Counsel + CFO (1-of-1 each) |
| > $100M | Above + CEO + Board Sub-Committee (N-of-M with M = full sub-committee) |

Per `taxonomies/contract-type-taxonomy.md`, contract type also drives routing:

- NDA: Legal Review (1-of-1).
- BAA (HIPAA): Compliance Officer + Legal (1-of-1 each).
- M&A SPA: General Counsel + CFO + CEO + Board (full).
- IP Assignment: Legal + R&D (1-of-1 each).
- Real Estate: Real Estate Committee + General Counsel.
- Employment: HR Director + Legal (1-of-1 each).
- Government Contract: Government Affairs + Compliance Officer + Legal.

## Deviation classification override

Per IP-026, ClauseDeviation classification triggers additional approval:

- Fallback: original matrix applies.
- Non-standard: + General Counsel.
- High-risk: + General Counsel + Risk Officer.
- Prohibited: + CEO + Board (or escalation to refuse).
- Approved-exception: as-defined-by-approval-of-exception.

## Counterparty risk override

If counterparty has elevated risk (sanctions adjacency, prior litigation, PEP):

- + Compliance Officer.
- + General Counsel.

## Materiality override (SOX-404)

Material contracts (per tenant materiality threshold or rev-rec rules):

- Author ≠ approver enforced (segregation of duties).
- + CFO (for SOX-relevant contracts).
- Quarter-end material contracts surface on the CEO/CFO certification dashboard.

## Jurisdiction override

- KR-sovereign contracts: + KR Compliance Lead.
- EU contracts under `gdpr`: + DPO.
- HIPAA contracts under `hipaa-baa`: + HIPAA Privacy Officer.

## Approval flow

```
Author submits ---> Route resolved ---> Sequential or parallel approvals ---> All satisfied ---> Approved
                       |                       |
                       |                       +-- any rejection ---> Returned to Author
                       |
                       +-- escalation if SLA exceeded ---> Next-up approver
```

## SLA targets

| Approval level | SLA target |
|---|---|
| Contracts Manager | 4 business hours |
| Procurement | 1 business day |
| Legal Review | 2 business days |
| VP-level | 3 business days |
| General Counsel | 5 business days |
| CEO | 7 business days |
| Board sub-committee | 15 business days |

SLA breach triggers escalation to the next-up approver with a notification.

## Approval evidence

Each approval generates:

```
ApprovalEvidence {
  approval_id: UUIDv7,
  contract_id: ContractId,
  approver_principal_id: PrincipalId,
  approver_role: ApproverRole,
  approval_authority: ApprovalAuthority,
  approval_decision: ApprovalDecision,         // approve | reject | request_changes
  approval_rationale: String,
  approval_evidence_artefact_id: ArtefactId,    // signed approval doc
  approval_timestamp: Timestamp<RFC3339>,
  signature_envelope: SignatureEnvelopeRef?,    // for high-materiality approvals
  audit_event_id: AuditEventId,
}
```

## SOX-404 segregation enforcement

```cedar
forbid (
  principal,
  action == Action::"ContractApprove",
  resource is Contract
) when {
  resource.author_principal_id == principal.principal_id
};

forbid (
  principal,
  action == Action::"ContractApprove",
  resource is Contract
) when {
  resource.active_packs.contains("sox-404") &&
  resource.materiality_class == "material_revenue" &&
  principal.role !in ["cfo", "general_counsel", "ceo", "board_member"]
};
```

## Audit events

- `oya.contract.lifecycle.management.approval.required`
- `oya.contract.lifecycle.management.approval.granted`
- `oya.contract.lifecycle.management.approval.rejected`
- `oya.contract.lifecycle.management.approval.escalated`
- `oya.contract.lifecycle.management.approval.sla_breached`
- `oya.contract.lifecycle.management.approval.segregation_violation_blocked`

## Composition with packs

- `sox-404`: full segregation enforced + CFO approval for material.
- `hipaa-baa`: privacy officer approval for BAAs.
- `gdpr`: DPO approval for DPAs.
- `kr-pipa`: KR compliance lead approval for KR cross-border transfers.
- `fcpa-ukba`: + compliance officer for anti-corruption certifications.

## Standards references

- PCAOB AS 2201 (ICFR audit).
- COSO Internal Control - Integrated Framework.
- SOX § 404.
