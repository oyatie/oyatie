---
doc_class: StateMachine
microservice: contract-lifecycle-management
dimension_id: Q-014
related_packs: [sox-404]
date: 2026-05-21
---

# Obligation State Machine

Obligations are extracted from signed contracts per IP-027 obligation-extraction-confidence-review. Each obligation transitions through a defined state machine.

## States

```
   Pending ---> Acknowledged ---> Active ---> Satisfied
                                     |
                                     +---> Overdue ---> Cured ---> Satisfied
                                     |                  |
                                     |                  +---> Disputed ---> Resolved
                                     |
                                     +---> Waived
                                     |
                                     +---> Suspended (force-majeure)
                                     |
                                     +---> Terminated
```

### Pending

Obligation extracted by IP-027 but not yet acknowledged. Confidence band determines next action:

- Confidence ≥ 0.95: auto-acknowledge → Active.
- 0.85 ≤ Confidence < 0.95: queue for human review.
- Confidence < 0.85: advisory-only; does not transition to Active.

### Acknowledged

Obligation reviewed and accepted by the owning party. Becomes Active when the trigger condition is met.

### Active

Obligation is currently due. Owner has the obligation to perform.

Allowed transitions: → Satisfied (performed in time), → Overdue (past due date), → Waived (waived by other party), → Suspended (force-majeure), → Terminated (contract terminated).

### Satisfied

Obligation performed. Evidence of performance attached.

### Overdue

Past due date without performance. Notice-and-cure (if applicable) may grant additional time.

Allowed transitions: → Cured (performed during cure period), → Disputed (dispute over performance), → Breach (no cure within period).

### Cured

Performed during cure period. Transitions back to Satisfied.

### Disputed

Performance disputed. May go to mediation, arbitration, or litigation.

Allowed transitions: → Resolved (resolution reached).

### Resolved

Dispute resolved. May be performed, waived, or settled.

### Waived

Obligation waived by the obligor's counterparty. Permanent.

### Suspended

Suspended due to force-majeure (per `legal-dimensions/force-majeure-obligation-suspension.md`). Clock paused.

Allowed transitions: → Active (force-majeure resolved), → Terminated (force-majeure duration cap exceeded → termination right invoked).

### Terminated

Contract terminated; obligation no longer applicable (unless it is a surviving obligation per contract terms).

## Obligation schema

```
Obligation {
  obligation_id: UUIDv7,
  contract_id: ContractId,
  source_clause_id: ClauseId,
  source_span: ClauseSourceSpan,
  obligation_type: ObligationType,           // payment | performance | reporting | covenant | warranty
  owner_role: OwnerRole,                      // who owes (tenant or counterparty)
  due_basis: DueBasis,                        // expression for computing due date
  due_date: Date,                              // computed
  amount: MoneyAmount?,                        // for payment obligations
  state: ObligationState,
  confidence_band: ConfidenceBand,            // from IP-027 extraction
  human_reviewed: bool,
  human_reviewer_principal_id: PrincipalId?,
  surviving: bool,                             // survives contract termination?
  notice_and_cure: NoticeAndCureLink?,
  audit_event_id: AuditEventId,
}

enum ObligationType {
  Payment,                                     // pay X by Y
  Performance,                                 // deliver X by Y
  Reporting,                                   // produce report by Y
  Covenant,                                    // maintain X covenant
  Warranty,                                    // X warranted true
  AntiCorruption,                              // FCPA / UKBA certification
  ServiceLevel,                                // SLA threshold
  ConfidentialityMaintenance,                  // ongoing confidentiality
  ChangeOfControlNotification,
  AssignmentRestriction,
  Other { description: String },
}

enum OwnerRole {
  Tenant,
  Counterparty,
  Both,
}

struct DueBasis {
  expression: String,                          // per legal-dimensions/obligation-due-basis-grammar.md
  resolved_date: Date,
}

enum ConfidenceBand {
  AutoPropose,                                 // ≥ 0.95
  HumanReview,                                 // 0.85 - 0.95
  Advisory,                                    // 0.70 - 0.85
  LowConfidence,                               // < 0.70 (filtered out)
}
```

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ObligationDelete",
  resource is Obligation
) when {
  resource.state in ["Active", "Overdue", "Disputed", "Suspended"]
};

permit (
  principal,
  action == Action::"ObligationWaive",
  resource is Obligation
) when {
  resource.owner_role == "Counterparty" &&
  principal.role in ["contracts_manager", "general_counsel"] &&
  principal.waive_authority_satisfied == true
};
```

## Audit events

- `oya.contract.lifecycle.management.obligation.pending`
- `oya.contract.lifecycle.management.obligation.acknowledged`
- `oya.contract.lifecycle.management.obligation.active`
- `oya.contract.lifecycle.management.obligation.satisfied`
- `oya.contract.lifecycle.management.obligation.overdue`
- `oya.contract.lifecycle.management.obligation.cured`
- `oya.contract.lifecycle.management.obligation.disputed`
- `oya.contract.lifecycle.management.obligation.resolved`
- `oya.contract.lifecycle.management.obligation.waived`
- `oya.contract.lifecycle.management.obligation.suspended`
- `oya.contract.lifecycle.management.obligation.terminated`
