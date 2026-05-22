---
doc_class: StateMachine
microservice: contract-lifecycle-management
dimension_id: L-011 + Q-005
authoritative_source: FRCP Rule 37(e) + 18 USC § 1519 + spoliation case law
related_packs: [sox-404, sec-17a-4, gdpr, hipaa-baa]
date: 2026-05-21
---

# Legal Hold State Machine

When litigation is reasonably anticipated, parties must preserve potentially-relevant evidence (FRCP Rule 37(e); Pension Comm. of Univ. of Montreal Pension Plan v. Banc of America Sec., 685 F. Supp. 2d 456 (S.D.N.Y. 2010)). Failure to preserve can result in spoliation sanctions including adverse inference, monetary penalties, and case dispositive sanctions. Legal hold is the contract-evidence-preservation regime that suspends ordinary retention destruction.

## States

```
                    +---------------+
                    |  NORMAL       |  default state; ordinary retention rules apply
                    +---------------+
                            |
                            | apply_hold(litigation_ref)
                            v
              +---------------------------+
              | HOLD_APPLIED              |  litigation anticipated; preservation begins
              | (preservation active)     |
              +---------------------------+
                    |              ^
                    | identify_party()
                    v              |
              +---------------------------+
              | LITIGATION_PARTY          |  matter named; party identified
              |  IDENTIFIED               |
              +---------------------------+
                    |              ^
                    | activate_obligation()
                    v              |
              +---------------------------+
              | PRESERVATION_OBLIGATION_  |  full preservation; no destruction; no edit
              |   ACTIVE                  |
              +---------------------------+
                    |
                    | release_hold(release_authority, release_reason)
                    v
              +---------------------------+
              | HOLD_RELEASED_            |  release; retention clock resumes
              |   WITH_AUDIT              |
              +---------------------------+
```

## State definitions

### NORMAL

Default. Ordinary retention rules per `legal-dimensions/retention-overlay-by-contract-type.md` apply. Contracts can be amended, redlined, and (when retention expires) destroyed.

### HOLD_APPLIED

A legal-hold steward has applied a hold to one or more contracts based on reasonable anticipation of litigation. Triggers include:

- Threatened lawsuit received from counterparty.
- Internal investigation initiated (FCPA, SOX, securities, employment).
- Regulator subpoena or document request.
- Insurance claim notification.
- Bankruptcy of counterparty.

While in HOLD_APPLIED:

- Contract cannot be destroyed regardless of retention.
- Contract cannot be amended (only annotated for hold purposes).
- All copies, including backups and replicas, are marked for preservation.
- A `LegalHoldRecord` is created with `hold_id`, `applied_by`, `applied_at`, `anticipated_litigation_summary`, `affected_contract_ids`.

### LITIGATION_PARTY_IDENTIFIED

The litigation has progressed past anticipation; an opposing party is named and the matter has a docket reference. Triggers include:

- Lawsuit filed (case caption issued).
- Counterparty served with complaint.
- Regulator case opened.

Adds to LegalHoldRecord:

- `matter_caption: String`
- `docket_reference: String?`
- `opposing_party: [LegalEntityRef]`
- `claims_summary: String`
- `discovery_scope: DiscoveryScope`

### PRESERVATION_OBLIGATION_ACTIVE

Full preservation obligation under FRCP Rule 26(f). All metadata, prior versions, audit-chain events, redline history, and communications attached to the contract must be preserved. Triggers include:

- Discovery scheduling order.
- Litigation hold notice issued to opposing party (mutual hold).
- Production request received.

Adds:

- `preservation_scope: PreservationScope` (full / metadata-only / specific date range)
- `production_deadline: Timestamp<RFC3339>?`
- `e_discovery_export_format: ESIFormat` (typically EDRM XML or Concordance Load File)

### HOLD_RELEASED_WITH_AUDIT

The matter is resolved (settlement, judgment, dismissal, regulator closure). Hold is released and ordinary retention resumes. The release requires:

- Authority (general counsel / outside counsel / regulatory liaison).
- Reason (settled, dismissed, statute of limitations expired, etc.).
- Audit-chain event recording the release.

Adds:

- `released_by: PrincipalId`
- `released_at: Timestamp<RFC3339>`
- `release_reason: ReleaseReason`
- `final_disposition: FinalDisposition`

## Cedar gate

```cedar
forbid (
  principal,
  action in [Action::"ContractDelete", Action::"ContractAlter",
             Action::"ClauseRemove", Action::"RedlineDelete",
             Action::"AuditEventPurge"],
  resource is Contract
) when {
  resource.legal_hold_state in ["HOLD_APPLIED",
                                  "LITIGATION_PARTY_IDENTIFIED",
                                  "PRESERVATION_OBLIGATION_ACTIVE"]
};

permit (
  principal,
  action == Action::"LegalHoldApply",
  resource is Contract
) when {
  principal.role in ["legal_hold_steward", "general_counsel",
                       "compliance_officer", "outside_counsel"] &&
  principal.engagement_signed == true
};

permit (
  principal,
  action == Action::"LegalHoldRelease",
  resource is LegalHoldRecord
) when {
  principal.role in ["general_counsel", "compliance_officer"] &&
  resource.release_authority_satisfied == true
};
```

## E-discovery export

When in PRESERVATION_OBLIGATION_ACTIVE state, the µservice exposes an e-discovery export:

- EDRM XML 1.2 format (canonical).
- Concordance Load File (DAT + OPT) for vendor compatibility.
- Custodian metadata + chain-of-custody attestation.
- Hash-verified bundle.

The export bundles:

- Contract body (immutable version pinned at hold time + all subsequent versions).
- All redline history.
- All clause history.
- All obligation history.
- All audit-chain events relating to the contract.
- All approvals + signature packets.
- All counterparty communications (when attached).

## Audit events

- `oya.contract.lifecycle.management.legal_hold.applied`
- `oya.contract.lifecycle.management.legal_hold.party_identified`
- `oya.contract.lifecycle.management.legal_hold.preservation_active`
- `oya.contract.lifecycle.management.legal_hold.released`
- `oya.contract.lifecycle.management.legal_hold.discovery_export`

Each carries the standard tenant + principal + matter dimensions.

## Mandatory retention during hold

Per Pension Committee + Zubulake duty:

- All contract versions preserved.
- All audit-chain events preserved.
- All backups containing the contract preserved (cross-emit to backup substrate).
- All cell replicas preserved.

The µservice cross-emits to the `audit-chain` and `backup` substrate µservices to ensure backup retention is extended.

## Spoliation risk metric

The µservice tracks a spoliation-risk metric per tenant:

- Number of contracts in hold.
- Average hold duration.
- Holds released without full e-discovery export.
- Holds released without authority record.

High spoliation-risk metric values trigger a governance escalation.

## Standards references

- FRCP Rule 26(f), 34, 37(e).
- 18 USC § 1519 (destruction, alteration, falsification of records in federal investigations).
- Pension Comm. of Univ. of Montreal Pension Plan v. Banc of America Sec., 685 F. Supp. 2d 456 (S.D.N.Y. 2010).
- Zubulake v. UBS Warburg, 217 F.R.D. 309 (S.D.N.Y. 2003) — Zubulake duty.
- The Sedona Conference Principles.
- EDRM Electronic Discovery Reference Model.
