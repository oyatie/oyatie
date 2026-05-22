---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-013
authoritative_source: Attorney-Client Privilege common law + FRE 502 + jurisdiction-specific privilege rules
related_packs: [sox-404, sec-17a-4]
date: 2026-05-21
---

# Privilege Tagging Overlay

CLM contains legal advice, redline annotations from outside or inside counsel, approval rationale, and work-product. These may be protected by attorney-client privilege (ACP) or attorney work-product doctrine. Privileged content must be tagged and excluded from discovery requests that do not waive privilege.

## Privilege classifications

```
enum PrivilegeClassification {
  None,                                        // public, no privilege claimed
  AttorneyClientPrivilege {
    attorney_principal_id: PrincipalId,
    client_principal_id: PrincipalId,
    purpose: PrivilegePurpose,                 // legal_advice | litigation_strategy | regulatory_advice
  },
  WorkProductDoctrine {
    attorney_principal_id: PrincipalId,
    anticipated_litigation_ref: LitigationRef,
    prepared_for: PrincipalId,
    fact_or_opinion: WorkProductScope,         // fact (lesser protection) | opinion (greater protection)
  },
  JointDefense {                                // common-interest privilege
    parties: [LegalEntityRef],
    joint_defense_agreement_ref: ContractId,
  },
  SettlementCommunication {                     // FRE 408
    settlement_negotiation_ref: NegotiationRef,
  },
  SelfCriticalAnalysis {                        // varying protection across jurisdictions
    purpose: String,
  },
  TradeSecret {                                 // 18 USC § 1836 / state UTSA
    classified_at: Date,
  },
  None,
}
```

## Tagged fields

Privilege tagging applies at the following CLM artefact granularities:

| Artefact | Field that can be privileged |
|---|---|
| Contract | `body.legal_advice_annotations`, `body.privileged_appendices` |
| Clause | `clause.lawyer_comments`, `clause.legal_advice_text` |
| Redline event | `redline.author_comment`, `redline.legal_basis` |
| Approval rationale | `approval.legal_review_summary`, `approval.risk_assessment` |
| Counterparty communication | If counsel-to-counsel: full text |
| Internal communication | If between in-house counsel and business: full text |

The default classification is `None`. Authors may upgrade to ACP or WorkProduct at creation; downgrade requires general counsel approval.

## Access control

```
permit (
  principal,
  action == Action::"PrivilegedContentRead",
  resource is PrivilegedAnnotation
) when {
  resource.privilege_classification matches "AttorneyClientPrivilege" &&
  (
    principal.principal_id == resource.attorney_principal_id ||
    principal.principal_id == resource.client_principal_id ||
    principal.role in ["general_counsel", "managing_partner"] ||
    principal.privilege_waiver_granted == true
  )
};

permit (
  principal,
  action == Action::"PrivilegedContentRead",
  resource is PrivilegedAnnotation
) when {
  resource.privilege_classification matches "WorkProductDoctrine" &&
  (
    principal.principal_id == resource.attorney_principal_id ||
    principal.assignment_to_matter == resource.anticipated_litigation_ref
  )
};

forbid (
  principal,
  action == Action::"PrivilegedContentExport",
  resource is Contract
) when {
  resource.contains_privileged_content == true &&
  principal.export_purpose != "internal_review" &&
  resource.privilege_waiver_granted == false
};
```

## Discovery export with privilege exclusion

When the µservice produces an e-discovery export (per `state-machines/legal-hold-state-machine.md` PRESERVATION_OBLIGATION_ACTIVE state), the export pipeline:

1. Tags each artefact with its privilege classification.
2. For each artefact in scope of the discovery request, applies the privilege filter:
   - `None`: include verbatim.
   - `AttorneyClientPrivilege`: replace with privilege log entry: `[REDACTED — ACP — log entry # NNN]`.
   - `WorkProductDoctrine`: replace with privilege log entry.
   - `JointDefense`: replace with privilege log entry; cross-emit notice to all joint-defense parties.
   - `SettlementCommunication`: FRE 408 — if discovery is in same matter, redact; if cross-matter, may be discoverable; surface for case-by-case review.
   - `TradeSecret`: redact + propose protective order.
3. Generates a privilege log enumerating each redaction with: artefact id, classification, attorney, client, date, subject matter (generic), basis for privilege.

## Inadvertent disclosure (FRE 502)

When a privileged item is accidentally produced, FRE 502(b) provides a clawback if (a) inadvertent, (b) reasonable steps to prevent, (c) prompt rectification. The µservice supports:

- Inadvertent-disclosure flagging at export.
- Clawback notice issuance.
- Re-export with the inadvertently-disclosed item correctly redacted.

## Privilege waiver tracking

When privilege is waived (explicitly by the holder), the µservice records:

- `waived_by`: principal who waived.
- `waived_at`: timestamp.
- `waiver_scope`: full / specific matter / specific party.
- `waiver_evidence`: written waiver artefact.

Waiver is recorded but cannot be retroactively un-waived (FRE 502(a) subject-matter waiver applies to all communications on the same subject).

## Tenant-class composition

- `tenant_class=demo_trial`: privilege tagging available read-only; ACP gate enforced; full-feature privilege log limited to paid tenants.
- `tenant_class=paid`: full privilege workflow + log + waiver tracking + clawback.

## Composition with packs

- `gdpr`: privilege logs themselves may contain PII; subject to GDPR retention.
- `sec-17a-4`: privilege does not exempt from regulator production absent specific privilege assertion to the regulator.
- `sox-404`: audit-relevant privilege is preserved; auditor may access ACP under controlled engagement letter.

## Audit events

- `oya.contract.lifecycle.management.privilege.tagged`
- `oya.contract.lifecycle.management.privilege.waived`
- `oya.contract.lifecycle.management.privilege.redacted_at_export`
- `oya.contract.lifecycle.management.privilege.inadvertent_disclosure`

## Standards references

- FRE 502 (federal rules of evidence) — limitations on waiver.
- FRE 408 (settlement communications).
- Upjohn Co. v. United States, 449 U.S. 383 (1981) — corporate ACP scope.
- Hickman v. Taylor, 329 U.S. 495 (1947) — work-product doctrine.
- Defend Trade Secrets Act 18 USC § 1836.
- UK Civil Procedure Rules Part 31 + LPP authorities.
- CCBE Charter of Core Principles of the European Legal Profession.
