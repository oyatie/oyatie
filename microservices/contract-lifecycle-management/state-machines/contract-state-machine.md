---
doc_class: StateMachine
microservice: contract-lifecycle-management
dimension_id: Q-013
related_packs: [sox-404, eidas, esign]
date: 2026-05-21
---

# Contract State Machine

```
   Draft ---> Review ---> Approved ---> OutForSignature ---> Signed ---> Effective ---> Terminated
                                                                            |
                                                                            +-> Amended ---> Effective (new version)
                                                                            |
                                                                            +-> Renewed ---> Effective (new term)
                                                                            |
                                                                            +-> InDispute ---> Effective | Terminated | Settled
```

## States

### Draft

Contract created, body editable. Multiple authors may collaborate via Loro CRDT (per `legal-dimensions/redline-collaboration-crdt.md`). No external commitment.

Allowed transitions: → Review, → discard.

### Review

Contract submitted to reviewers (internal legal, counterparty). Redline events captured per IP-029.

Allowed transitions: → Draft (back-to-author), → Approved.

### Approved

All required approvals satisfied per `legal-dimensions/approval-routing-matrix.md`. Internal commitment complete; ready to send to counterparty for signature.

SOX-404 segregation gate: contract author ≠ contract approver.

Allowed transitions: → OutForSignature, → Draft (revoke approval).

### OutForSignature

Signature envelope generated; sent to signatories via e-signature provider. Signatories execute via WebAuthn / eID / wallet / paper. No further edits to the contract body permitted.

Allowed transitions: → Signed (all signatures collected), → Draft (recalled before signature).

### Signed

All required signatories have signed. Signature envelope sealed per `legal-dimensions/signature-envelope-canonical.md`. WORM lock applied if SOX-404 / SEC 17a-4.

Allowed transitions: → Effective (effective date reached; possibly immediate), → Terminated (cancellation before effective).

### Effective

Contract is in effect; obligations active per `state-machines/obligation-state-machine.md`. Renewal-risk scoring active per IP-028.

Allowed transitions: → Amended (substantive change), → Renewed (term renewal), → InDispute (dispute raised), → Terminated (term expired or terminated for cause).

### Amended

A new amendment is being negotiated. The current contract remains Effective during amendment drafting. Amendment is itself a child contract.

Allowed transitions: → Effective (amendment effective; replaces old).

### Renewed

Renewal of the current term. May be automatic (auto-renewal clause) or negotiated (renewal-with-renegotiation).

Allowed transitions: → Effective (new term).

### InDispute

A dispute (alleged breach, interpretive disagreement) has been raised. Legal hold typically applied per `state-machines/legal-hold-state-machine.md`.

Allowed transitions: → Effective (dispute resolved), → Terminated (terminated due to breach), → Settled (settled with new terms).

### Terminated

Contract terminated. Obligations cease (except surviving obligations per the contract terms). Retention clock begins per `legal-dimensions/retention-overlay-by-contract-type.md`.

Allowed transitions: none (terminal state, subject to retention).

### Settled

Dispute settled; settlement agreement creates a new contract. The original may or may not be terminated.

Allowed transitions: → Terminated, → Effective (if amended via settlement).

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ContractEdit",
  resource is Contract
) when {
  resource.state in ["OutForSignature", "Signed", "Effective",
                       "Terminated", "Amended", "Renewed", "InDispute", "Settled"]
};

forbid (
  principal,
  action == Action::"ContractApprove",
  resource is Contract
) when {
  resource.author_principal_id == principal.principal_id
  // segregation of duties
};

forbid (
  principal,
  action == Action::"ContractTerminate",
  resource is Contract
) when {
  resource.has_notice_and_cure_provision == true &&
  resource.notice_and_cure_state != "CURE_FAILED"
};
```

## State transition audit events

- `oya.contract.lifecycle.management.contract.state.draft`
- `oya.contract.lifecycle.management.contract.state.review`
- `oya.contract.lifecycle.management.contract.state.approved`
- `oya.contract.lifecycle.management.contract.state.out_for_signature`
- `oya.contract.lifecycle.management.contract.state.signed`
- `oya.contract.lifecycle.management.contract.state.effective`
- `oya.contract.lifecycle.management.contract.state.amended`
- `oya.contract.lifecycle.management.contract.state.renewed`
- `oya.contract.lifecycle.management.contract.state.in_dispute`
- `oya.contract.lifecycle.management.contract.state.terminated`
- `oya.contract.lifecycle.management.contract.state.settled`

## Composition with packs

- `sox-404`: state transitions must be approved with segregation-of-duties.
- `esign`: OutForSignature → Signed transition requires ESIGN intent capture + consumer disclosure (if applicable).
- `eidas`: OutForSignature → Signed requires AES or QES envelope per signatory's jurisdiction.
- `gdpr`: Terminated → erasure of PII-fields per Article 17 with retention exceptions.
- `sec-17a-4`: signed and effective contracts WORM-locked.
