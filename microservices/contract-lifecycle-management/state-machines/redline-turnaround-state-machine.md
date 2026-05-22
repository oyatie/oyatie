---
doc_class: StateMachine
microservice: contract-lifecycle-management
dimension_id: S-011
related_packs: [sox-404]
date: 2026-05-21
---

# Redline Turnaround State Machine

Negotiation cycle: tenant-author draft → counterparty redline → internal review → counter-redline → counterparty re-review.

## States

```
   InternalDraft ---> SentForRedline ---> CounterpartyEdited ---> InternalReview
                                                                      |
                                                                      v
                                                                CounterRedline ---> ResubmittedToCounterparty
                                                                      |
                                                                      +--(loop until)--> ConvergedToFinal ---> ApprovalRouting ---> Signed
```

### InternalDraft

Tenant author drafting. Multiple authors via Loro CRDT collaboration.

### SentForRedline

Draft sent to counterparty. Sent via secure link + email + ESIGN-equivalent intent capture (if applicable). Counterparty may edit; tenant view is read-only of counterparty edits.

### CounterpartyEdited

Counterparty has made edits. Edits ingested as redline events per IP-029 (counterparty-redline-provenance).

### InternalReview

Tenant reviews counterparty's redlines. Each redline classified:

- Accept (counterparty's position is acceptable).
- Reject (counterparty's position is rejected; counter-position drafted).
- Negotiate (alternative position proposed).

### CounterRedline

Tenant produces counter-redlines. Counter-redlines + accepts produce a new version.

### ResubmittedToCounterparty

New version sent to counterparty.

### ConvergedToFinal

Both parties have agreed on a final text. No outstanding redlines.

### ApprovalRouting

Final text routed for approval per `legal-dimensions/approval-routing-matrix.md`. SOX-404 segregation-of-duties applied.

### Signed

Approval complete; contract signed.

## Turnaround time metric

Per CLM SLO + capacity-model.md, the µservice tracks redline turnaround time:

- Time from counterparty redline received → internal counter-redline sent.
- Time from internal counter-redline sent → counterparty re-edit received.

Per IP-026 clause-deviation-negotiation-ledger, the negotiation history is preserved as the deal-record for analytics.

## Redline event schema

```
RedlineEvent {
  event_id: UUIDv7,
  contract_id: ContractId,
  contract_version: ContractVersion,
  author_principal_id: PrincipalId,
  author_party: AuthorParty,                  // Tenant | Counterparty
  source_span: ClauseSourceSpan,
  redline_text_before: String,
  redline_text_after: String,
  redline_classification: RedlineClassification,
  rationale: String?,
  fallback_position_invoked: ClauseTemplateId?,
  approval_required: ApprovalAuthority?,
  audit_event_id: AuditEventId,
}

enum RedlineClassification {
  TextChange,
  Insertion,
  Deletion,
  Substitution,
  Annotation,                                  // comment without text change
  AccessibilityOnly,                           // formatting only
}
```

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"RedlineAdd",
  resource is Contract
) when {
  resource.state !in ["InternalDraft", "InternalReview", "CounterRedline",
                         "CounterpartyEdited", "SentForRedline", "ResubmittedToCounterparty"]
};

forbid (
  principal,
  action == Action::"RedlineForceConvergence",
  resource is Contract
) when {
  principal.role != "deal_desk_lead"
};
```

## Audit events

- `oya.contract.lifecycle.management.redline.added`
- `oya.contract.lifecycle.management.redline.classified`
- `oya.contract.lifecycle.management.redline.fallback_invoked`
- `oya.contract.lifecycle.management.redline.counterparty_edited`
- `oya.contract.lifecycle.management.redline.converged`

## Composition with packs

- `sox-404`: redline history preserved 7 years; approval segregation enforced.
- `eu-ai-act`: AI-suggested redlines marked per IP-030.
- `gdpr`: redlines containing PII subject to GDPR retention.
