---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: sec-17a-4
authoritative_source: SEC Rule 17a-4(f) under the Securities Exchange Act of 1934
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# SEC 17a-4 Pack Overlay — CLM

SEC Rule 17a-4 (17 CFR § 240.17a-4) governs the preservation of records by broker-dealers. The 2023 amendment (effective May 2023) modernized the previous WORM (Write Once, Read Many) requirement, allowing either WORM media or an audit-trail system meeting the specified attributes.

## Active triggers

The `sec-17a-4` pack is **mandatory** when:

- `tenant.entity_type ∈ {broker_dealer_us, registered_investment_adviser, securities_dealer}`.
- `tenant.sec_registrant = true`.
- `contract.materiality_class ∈ {trading_record, customer_account_record, regulatory_communication}`.

## 17a-4(f) electronic storage requirements (P0 LEGAL — L-016)

The 2023-amended Rule 17a-4(f) permits electronic storage if the records are preserved on:

1. **WORM-compliant media** — non-rewriteable, non-erasable storage, OR
2. **Audit-trail system** — system that maintains a complete time-stamped audit trail of all modifications to the records, including:
   - The original record.
   - All modifications.
   - Identity of the person making each modification.
   - The date and time of each modification.

Both must additionally:

- Verify automatically the quality and accuracy of the storage media recording process.
- Serialize the original and, if applicable, duplicate units of storage media, and time-date for the required period of retention the information placed on such media.
- Have the capacity to readily download indexes and records.

## CLM implementation

CLM satisfies 17a-4(f) via the audit-trail-system option (canonical) with WORM-compliant media as an additional belt-and-braces option for higher tiers of evidence assurance.

### Audit-trail system

- Every contract state transition emits an immutable audit-chain event sealed with HSM-rooted signing key.
- The audit-chain is content-addressed (BLAKE3 root hash) and tamper-evident.
- Audit-chain events are replicated to at least three cells per ADR-0248.
- Every modification carries `principal_id` + `tenant_id` + `policy_decision_id` + `audit_event_class`.

### WORM-compliant media

For tenants requiring the WORM option (typical for larger broker-dealers):

- **SeaweedFS** with Compliance mode — non-rewriteable filer mode.
- **AWS S3 Object Lock** in Compliance mode — cannot be overridden by root user.
- **AWS S3 Glacier Vault Lock** — immutable for the locked period.
- **OCI Object Storage Retention Lock** in time-bound mode.
- **Azure Blob Storage immutable storage policies** in time-based retention mode.
- **NetApp SnapLock** in Compliance mode.
- **Dell EMC Centera / ECS** in Compliance mode.

The µservice's `legal-dimensions/worm-binding-model.md` documents the canonical binding per deployment context.

## 17a-4(b) retention periods

| Record class | Retention | Source |
|---|---|---|
| Customer agreements + contracts | 6 years (first 2 easily accessible) | 17a-4(b)(4) |
| Customer order tickets | 3 years (first 2 easily accessible) | 17a-4(b)(1) |
| Trial balances, financial statements | 6 years (first 2 easily accessible) | 17a-4(b)(5) |
| Compliance + supervisory records | 3 years from termination | 17a-4(b)(11) |
| Communications received and sent | 3 years (first 2 easily accessible) | 17a-4(b)(4) |
| Compensation arrangements | Life of broker-dealer + 6 years | 17a-4(d) |

The µservice applies the maximum applicable retention across all triggered subsections.

## 17a-4(f)(2)(ii) — undertaking from third-party preservation provider

If the records are stored via a third-party provider, the provider must file an undertaking with the SEC stating it will, upon SEC request, provide a copy of the records. Oyatie operates this preservation surface on behalf of broker-dealer tenants under a standing undertaking.

## 17a-4(f)(3)(vii) — third-party undertaking + designated officer

In addition to the third-party undertaking, the broker-dealer must designate at least one third party (typically a Designated Third Party, DTP) who has access to the records and can submit them to the SEC if the broker-dealer becomes incapacitated. CLM produces the DTP credentials at pack activation.

## Retention overlay table summary

When `sec-17a-4` is active:

- Default retention floor: 6 years easily accessible.
- WORM enrolment automatic for sealed contracts.
- DTP credential issuance at pack activation.
- Audit-chain seal events retained for the maximum retention period.

## Cedar gate fragment

```cedar
forbid (
  principal,
  action in [Action::"ContractDelete", Action::"ContractAlter"],
  resource is Contract
) when {
  resource.active_packs.contains("sec-17a-4") &&
  resource.retention_remaining_days > 0
};

permit (
  principal,
  action == Action::"RecordsExportToSEC",
  resource is Tenant
) when {
  principal.role in ["dtp", "compliance_officer", "sec_examiner"] &&
  principal.engagement_signed == true &&
  resource.active_packs.contains("sec-17a-4")
};
```

## Composition with other packs

- `sec-17a-4` + `sox-404`: both apply to public broker-dealers; retention floor = max(7, 6) = 7 years.
- `sec-17a-4` + `finra-4511`: FINRA Rule 4511 adopts SEC 17a-4 retention by reference; identical retention.
- `sec-17a-4` + `gdpr`: cross-Atlantic broker-dealers; US retention floor; GDPR Article 17(3)(b) legal-obligation exemption applies.

## Evidence on activation

- `oya.contract.lifecycle.management.pack.sec-17a-4.activated` audit event with the broker-dealer's CRD number and SEC registration details.
- Cedar policy compilation with retention-protected delete gates enabled.
- WORM storage enrolment.
- DTP credential issuance.

## Standards references

- 17 CFR § 240.17a-4 (full rule text).
- SEC Release No. 34-96034 (Oct 12, 2022) — Amendments to the rule effective May 3, 2023.
- FINRA Rule 4511 (Books and Records — General Requirements).
- FINRA Notice to Members 23-09 (March 2023) — SEC 17a-4 amendments guidance.
