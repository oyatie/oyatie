---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: sox-404
authoritative_source: Sarbanes-Oxley Act of 2002 (Pub. L. 107-204) §§ 302, 404, 802 + 18 USC § 1520
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# SOX-404 Pack Overlay — CLM

The Sarbanes-Oxley Act of 2002 imposes internal-control, attestation, and record-retention requirements on:

- Public companies subject to Section 13 or 15(d) of the Securities Exchange Act of 1934.
- Any accountant who conducts an audit of such a company.

CLM is in scope because executed contracts are audit-relevant records under PCAOB Auditing Standard 1105 (Audit Evidence) and SOX § 802 mandates retention of working papers and documents used in connection with the audit.

## Active triggers

The `sox-404` pack is **mandatory** when:

- `tenant.entity_type ∈ {public_company_us, audit_firm, financial_services_us}`.
- `contract.materiality_class ∈ {audit_relevant, material_revenue, material_expense}` (CLM material-relevance classifier per `taxonomies/clause-family-taxonomy.md` financial-materiality section).
- `tenant.declared_jurisdictions` includes the US and `tenant.is_sec_registrant = true`.

## § 302 — corporate responsibility for financial reports

CEO and CFO must personally certify each quarterly and annual report. CLM contributes by:

- Surfacing all material contracts (>$100k or per tenant-defined threshold) on a CEO/CFO certification dashboard.
- Producing a quarter-end contract-state evidence export sealed with the audit-chain root hash.

## § 404 — management assessment of internal controls

Internal Control over Financial Reporting (ICFR) under PCAOB AS 2201. CLM contributes:

- Segregation of duties: contract author ≠ contract approver ≠ contract signer (enforced by Cedar gate).
- Authorization controls: every contract over the tenant-defined materiality threshold requires N-of-M approval per `legal-dimensions/approval-routing-matrix.md`.
- Completeness and accuracy: contract draft → review → approval → signature transitions produce immutable audit-chain entries.
- Restriction of access: tenant-scoped Cedar default-deny (ADR-0243).
- Period-end close: automated SOX-404 evidence export at fiscal period close.

## § 802 — destruction, alteration, falsification of records (18 USC §§ 1519, 1520)

§ 802 imposes criminal penalties (up to 20 years imprisonment, fines up to $5M for entities) for knowingly destroying, altering, falsifying records with intent to obstruct an investigation. CLM enforces:

- **Seven-year retention** for any document the auditor obtained, prepared, used, or revised in connection with an audit.
- **WORM storage** for sealed signature packets and audit-relevant contract artefacts (see `legal-dimensions/worm-binding-model.md` and `packs/sec-17a-4/README.md` for broker-dealer overlap).
- **Tamper-evident audit chain** sealed with HSM-rooted signing key.
- **Cedar deny on delete** for any contract with `active_packs.contains("sox-404") && retention_remaining_days > 0`.

## Retention overlay

| Document class | Retention | Source |
|---|---|---|
| Executed audit-relevant contract | 7 years from termination | SOX § 802 / 18 USC § 1520 |
| Working papers + supporting documents | 7 years from audit report date | SOX § 802 |
| Auditor communications attached to a contract | 7 years from contract termination | SOX § 802 |
| Material contract draft (pre-execution) | 7 years from latest revision | Auditor evidence floor |
| Approval evidence (N-of-M chain) | 7 years from approval | ICFR control evidence |
| Audit-chain seal events | 7 years from event time | Tamper-evident requirement |

If `sec-17a-4` is also active (broker-dealer scope), the retention floor extends to the SEC 17a-4(f) requirement (3 years easily accessible + 3 years total; longer for trading records).

## Cedar gate fragment

```cedar
forbid (
  principal,
  action in [Action::"ContractDelete", Action::"ContractAlter", Action::"ContractFalsify"],
  resource is Contract
) when {
  resource.active_packs.contains("sox-404") &&
  resource.retention_remaining_days > 0
};

forbid (
  principal,
  action == Action::"ContractApprove",
  resource is Contract
) when {
  resource.active_packs.contains("sox-404") &&
  resource.materiality_class in ["audit_relevant", "material_revenue", "material_expense"] &&
  resource.author_principal_id == principal.principal_id
};
```

## Composition with other packs

- `sox-404` + `gdpr`: SOX seven-year retention overrides GDPR storage limitation for SOX-relevant contracts.
- `sox-404` + `sec-17a-4`: SEC 17a-4(f) WORM and trading-record retention extends beyond SOX seven years for broker-dealer scope.
- `sox-404` + `esign`: signatures and approvals follow ESIGN; SOX adds the segregation-of-duties Cedar gate.

## Evidence on activation

Activation of the `sox-404` pack emits:

- `oya.contract.lifecycle.management.pack.sox404.activated` audit event with `materiality_threshold` and `period_end_dates`.
- Cedar policy compilation with segregation-of-duties + retention-protected delete gates enabled.
- WORM storage enrollment for sealed signature packets.
- A quarterly export task schedule in workflow-engine.

## PCAOB references

- PCAOB AS 1105 — Audit Evidence.
- PCAOB AS 2201 — An Audit of Internal Control Over Financial Reporting That Is Integrated with An Audit of Financial Statements.
- PCAOB AS 1215 — Audit Documentation.
- AICPA SAS 142 — Audit Evidence.
