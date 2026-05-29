---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: S-010
authoritative_source: Common-law freedom of contract + corporate playbook practice
related_packs: [sox-404]
date: 2026-05-21
---

# Clause Library Inheritance Model

Standard clause libraries inherit from a corporate playbook with per-deal override. CLM supports a three-tier inheritance model:

- **Tenant playbook**: tenant-wide clause defaults set by the tenant's general counsel.
- **Deal-type playbook**: per-contract-type overrides (e.g. SaaS contracts may have different default liability caps than reseller contracts).
- **Per-deal override**: deal-specific authoring with provenance.

## Inheritance model

```
ClauseLibrary {
  library_id: UUIDv7,
  tenant_id: TenantId,
  scope: LibraryScope,
  clauses: [ClauseTemplate],
  inheritance_parents: [LibraryRef],
  effective_from: Date,
  effective_to: Date?,
  approved_by: PrincipalId,
  approval_evidence: ArtefactId,
}

enum LibraryScope {
  TenantWide,
  ContractType { type: ContractType },
  Jurisdiction { jurisdiction: CountryCode },
  Counterparty { counterparty_id: CounterpartyId },  // negotiated standards
  DealSpecific { deal_id: DealId },
}

struct ClauseTemplate {
  template_id: ClauseTemplateId,
  clause_family: ClauseFamily,
  template_text: String,
  variable_bindings: [VariableBinding],         // e.g. {{cap_amount}}, {{notice_days}}
  standard_clause: bool,
  fallback_clauses: [ClauseTemplateId],         // approved fallback positions
  prohibited_modifications: [ProhibitedModification],
  approval_authority: ApprovalAuthority,
  authored_by: PrincipalId,
  authored_at: Timestamp<RFC3339>,
  version: u32,
}

enum ApprovalAuthority {
  AutoApproved,                                  // boilerplate; no review
  ContractsManagerReview,
  LegalReview,
  GeneralCounselApproval,
  BoardApproval,
}

enum ProhibitedModification {
  TextEditForbidden,                             // clause must be verbatim
  VariableEditOnly,                              // only declared variables may change
  ScopeReductionForbidden,                       // text cannot reduce tenant's rights
  AnyEditAllowed,
}
```

## Resolution order

When drafting a contract, the resolution order for a clause is:

1. Per-deal override (if any).
2. Counterparty-negotiated standard (if any).
3. Jurisdiction-specific clause (if jurisdiction-pack applies).
4. Contract-type playbook.
5. Tenant playbook.
6. Oyatie default template library (cross-tenant; readable only).

## Override evidence

Every override carries:

- The base clause it overrides.
- The authoring principal.
- The authoring rationale.
- The approval authority that approved the override.
- The audit-chain event.

This satisfies SOX-404 segregation-of-duties (author ≠ approver) and produces evidence of negotiation history.

## Fallback positions

Standard playbooks declare fallback positions: "Cap at $1M default; fallback positions $2M, $5M, $10M with VP-Legal approval; >$10M requires General Counsel approval". The µservice surfaces fallbacks to the drafter and routes approval per `legal-dimensions/approval-routing-matrix.md`.

## Prohibited modifications

Some clauses cannot be edited:

- Anti-corruption certification (per `legal-dimensions/fcpa-ukba-detection.md`).
- GDPR Article 28 DPA flow-down.
- HIPAA BAA flow-down.
- IP indemnity in certain contract types.

Attempted edits to prohibited-modification clauses are blocked by Cedar gate.

## Cross-counterparty playbook

For repeat counterparties, the µservice maintains a per-counterparty negotiated-standard playbook. After several negotiations, the counterparty's standard position is recorded; future contracts pre-populate with the prior negotiated outcome.

## Library versioning

Each `ClauseLibrary` is versioned with a content-addressed Merkle root. Library updates produce new library versions; contracts pin the library version they were authored against.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ClauseEdit",
  resource is Clause
) when {
  resource.template.prohibited_modifications.contains("TextEditForbidden")
};

forbid (
  principal,
  action == Action::"ClauseApprove",
  resource is Clause
) when {
  resource.template.approval_authority == "GeneralCounselApproval" &&
  principal.role != "general_counsel"
};

forbid (
  principal,
  action == Action::"ClauseApprove",
  resource is Clause
) when {
  resource.modified_by == principal.principal_id
  // segregation of duties: author ≠ approver
};
```

## Audit events

- `oya.contract.lifecycle.management.clause_library.template_added`
- `oya.contract.lifecycle.management.clause_library.template_versioned`
- `oya.contract.lifecycle.management.clause_library.template_used`
- `oya.contract.lifecycle.management.clause_library.fallback_invoked`
- `oya.contract.lifecycle.management.clause_library.prohibited_modification_blocked`
- `oya.contract.lifecycle.management.clause_library.counterparty_standard_recorded`

## Composition with packs

- `sox-404`: clause library + override evidence preserved 7 years.
- `gdpr`: tenant's customers' personal data must not be in library templates.
- `hipaa-baa`: BAA template prohibited from modification on flow-down provisions.

## Standards references

- Restatement (Second) of Contracts.
- World Commerce & Contracting (WorldCC) Standard Forms.
- IACCM Operational Guide to Contract Management.
- ABA Section of Business Law Model Stock Purchase Agreement.
