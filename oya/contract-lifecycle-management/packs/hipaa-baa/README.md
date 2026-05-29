---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: hipaa-baa
authoritative_source: HIPAA Privacy Rule 45 CFR Part 160 + 164; Security Rule § 164.308(b)(3)
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# HIPAA-BAA Pack Overlay — CLM

HIPAA (the Health Insurance Portability and Accountability Act of 1996, as amended by HITECH Act of 2009 and the Omnibus Final Rule of 2013) governs Protected Health Information (PHI) in the U.S. healthcare ecosystem. A Business Associate Agreement (BAA) is the contract instrument by which a HIPAA-covered entity delegates PHI access to a business associate (or a business associate to a sub-business-associate).

## Active triggers

The `hipaa-baa` pack is **mandatory** when any of:

- `tenant.entity_type ∈ {covered_entity, business_associate, sub_business_associate}` per 45 CFR § 160.103.
- `contract.contract_type == "baa"`.
- `contract.body` references PHI access or PHI processing.

## BAA contract type (P0 LEGAL — L-007)

The BAA is registered in `taxonomies/contract-type-taxonomy.md` as a first-class contract type. The CLM template library ships a canonical BAA template covering all elements required by 45 CFR § 164.504(e):

1. **Permitted uses and disclosures of PHI**: the BA may use PHI only for the purposes specified in the contract.
2. **Prohibition on additional uses**: no use beyond the contract except as required by law.
3. **Safeguards**: appropriate safeguards to prevent use or disclosure of PHI.
4. **Reporting of unauthorized use or disclosure**: notification within stated periods.
5. **Sub-contractor flow-down**: any sub-BA must be bound by the same restrictions.
6. **Access to PHI**: provide individual access per § 164.524.
7. **Amendment of PHI**: provide amendment per § 164.526.
8. **Accounting of disclosures**: provide accounting per § 164.528.
9. **Internal practices**: make available for HHS review.
10. **Termination return / destruction of PHI**: return or destroy PHI at contract termination.

## § 164.308(b)(3) written-BAA evidence

The Security Rule requires written assurance that the BA will appropriately safeguard ePHI. CLM produces:

- BAA execution evidence (signed BAA artefact + AdES envelope).
- Reference from each ePHI-processing data flow to the governing BAA.
- BAA renewal / amendment audit trail.

## Sub-BA flow-down

When a BA needs to delegate work to a sub-BA, the BA-to-sub-BA BAA inherits the constraints of the upstream BAA. CLM enforces flow-down by:

- Detecting the upstream BAA reference when authoring a sub-BAA.
- Pre-populating the sub-BAA template with the stricter of: upstream BAA terms vs sub-BA standard terms.
- Refusing to seal a sub-BAA that weakens the upstream constraints.

## Breach notification

§ 164.410 imposes a notification-to-CE within 60 days of breach discovery. CLM emits a `oya.contract.lifecycle.management.baa.breach_notification` audit event that cross-emits to the `governance` substrate for HHS reporting.

## Retention overlay

| Document class | Retention | Source |
|---|---|---|
| Executed BAA | 6 years from termination | § 164.530(j)(2) |
| BAA amendment | 6 years from amendment | § 164.530(j)(2) |
| Breach notification artefact | 6 years from breach | § 164.530(j)(2) |
| Accounting of disclosures | 6 years from disclosure | § 164.528(a)(1) |

## Residency overlay

When `hipaa-baa` is active and `deployment_context ∈ {oyatie-public-cloud, aws-guest, oci-guest}`:

- Storage and processing of PHI restricted to the BA's declared HIPAA-compliant cells (US-only for most BAs).
- AWS / OCI / Azure must operate under their respective HIPAA BAAs with Oyatie.
- Cross-region replication restricted to US-based cells unless a sub-BA exception is recorded.

When `deployment_context ∈ {on-prem, colo}`:

- Tenant-controlled physical security and access controls per § 164.310.
- Encryption-at-rest mandatory (FIPS 140-2 Level 1 minimum; FIPS 140-2 Level 3 for ePHI involving 50,000+ records per breach notification rule).

## Cedar gate fragment

```cedar
forbid (
  principal,
  action == Action::"ContractRead",
  resource is Contract
) when {
  resource.active_packs.contains("hipaa-baa") &&
  resource.contains_phi == true &&
  principal.hipaa_role !in ["covered_entity_workforce", "business_associate_workforce", "sub_ba_workforce"]
};

forbid (
  principal,
  action == Action::"BAAExecute",
  resource is BAAContract
) when {
  resource.active_packs.contains("hipaa-baa") &&
  resource.required_baa_elements_satisfied == false
};
```

## Tenant-class composition

- `tenant_class=demo_trial`: BAA template available for review; BAA execution gated off (BAAs require paid tenant + signed Oyatie BAA upstream).
- `tenant_class=paid + jurisdiction_pack=hipaa-baa`: full BAA execution available with HSM-backed signature + WORM retention.

## Composition with other packs

- `hipaa-baa` + `gdpr`: cross-Atlantic healthcare data flows; HIPAA US-only residency typically takes precedence on US-side ePHI; GDPR Article 28 applies on EU-side personal data; the BAA serves as the GDPR Article 28 DPA on the US side.
- `hipaa-baa` + `sox-404`: healthcare public companies (e.g. UnitedHealth, CVS) combine both; SOX retention extends to seven years for audit-relevant BAAs.

## Evidence on activation

Activation of the `hipaa-baa` pack emits:

- `oya.contract.lifecycle.management.pack.hipaa-baa.activated` audit event with the tenant's HIPAA role declaration.
- Cedar policy compilation with PHI-access gates enabled.
- WORM enrolment for executed BAAs.
- Cross-emit to `audit-chain` to mark the tenant as HIPAA-in-scope.

## Standards references

- 45 CFR Parts 160, 162, 164 (HIPAA Privacy, Security, Breach Notification Rules).
- HITECH Act 2009 (Title XIII of ARRA).
- HHS OCR HIPAA Audit Protocol.
- NIST SP 800-66 Rev. 2 (Implementing the HIPAA Security Rule).
