---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-014
authoritative_source: FCPA 15 USC §§ 78dd-1 et seq. + UK Bribery Act 2010 + OECD Anti-Bribery Convention
related_packs: [sox-404, soc-2]
date: 2026-05-21
---

# FCPA / UK Bribery Act Anti-Corruption Detection

The Foreign Corrupt Practices Act (FCPA, 15 USC §§ 78dd-1, 78dd-2, 78dd-3, 78m, 78ff) and the UK Bribery Act 2010 impose anti-bribery obligations on US-issuer companies and any organization doing business in the UK. CLM contributes by detecting bribery-risk clauses, enforcing anti-corruption certifications, and producing audit evidence of due diligence.

## FCPA anti-bribery provisions

- **§ 78dd-1** (Issuers): public companies with shares registered with the SEC must not corruptly offer, pay, or authorize payment to foreign officials.
- **§ 78dd-2** (Domestic Concerns): US persons (citizens, residents, US-incorporated) face the same prohibition.
- **§ 78dd-3** (Persons other than Issuers or Domestic Concerns): non-US persons acting within US territory.
- **Books and Records (§ 78m(b)(2)(A))**: issuers must maintain books and records that accurately reflect transactions.
- **Internal Accounting Controls (§ 78m(b)(2)(B))**: issuers must devise and maintain internal accounting controls sufficient to provide reasonable assurances.

## UK Bribery Act provisions

- **Section 1**: General offence of bribing another person.
- **Section 2**: Offence relating to being bribed.
- **Section 6**: Offence of bribing a foreign public official.
- **Section 7**: Failure of commercial organizations to prevent bribery (strict liability with "adequate procedures" defence).

## Detection in CLM

### Anti-corruption clause family

`taxonomies/clause-family-taxonomy.md` registers `AntiCorruption` as a canonical clause family. Standard anti-corruption clauses cover:

- Representation that no improper payments have been made.
- Covenant not to offer or pay bribes (FCPA + UKBA + local equivalent).
- Right to audit books and records on suspicion.
- Right to terminate on breach without further obligation.
- Indemnity for breach.
- Annual certification by counterparty.

### Risk clause detection

The AI redlining pipeline (per `legal-dimensions/ai-redlining-prompt-template.md`) detects clauses that present bribery risk:

- Vague consulting / agent fees ("introduction commission", "facilitation payments", "success fee" without clear deliverable).
- Payments to politically-exposed persons (PEPs) — cross-checked against the World-Check / Dow Jones PEP database via the `governance` substrate.
- Cash payment terms in high-corruption-risk jurisdictions (CPI < 50 per Transparency International).
- Payments to offshore jurisdictions without clear business reason.
- Indemnification of counterparty for "any legal proceedings" without bribery carve-out.
- Lack of audit rights.
- Lack of termination right for bribery.

Flagged clauses surface for legal review before contract execution.

### Counterparty due diligence

Per `counterparty-mdm/counterparty-mdm.md`, every counterparty is screened against:

- OFAC SDN List.
- World-Check Pro PEP and sanctions database.
- FCPA enforcement action database (DOJ + SEC).
- UK SFO + UKBA enforcement database.

High-risk counterparties (PEP, sanctions match, prior FCPA/UKBA enforcement) trigger enhanced due diligence:

- Beneficial ownership disclosure required.
- Source of funds verification.
- References from prior commercial counterparties.
- Annual re-certification.

## Certification overlay

When `fcpa_overlay = active` or `ukba_overlay = active`, the contract template library injects a Certification block requiring the counterparty to attest:

```
The [Counterparty], on its behalf and on behalf of its directors, officers,
employees, agents, and any third parties acting on its behalf, hereby
represents, warrants, and certifies that, in connection with this Agreement
and any related transaction:

(a) No payment, gift, or other benefit has been or will be offered, promised,
    paid, given, or authorized, directly or indirectly, to any Foreign
    Official, Domestic Public Official, or any other person, in violation of
    the FCPA, the UK Bribery Act 2010, or any other applicable anti-bribery
    or anti-corruption law.

(b) [Counterparty] has implemented an anti-corruption compliance program
    that includes, at minimum: (i) a written anti-bribery policy; (ii)
    employee training; (iii) due diligence on third parties; (iv) financial
    controls; (v) reporting mechanisms; and (vi) periodic risk assessment.

(c) [Counterparty] shall maintain books, records, and accounts that accurately
    and fairly reflect all transactions and dispositions of assets in
    reasonable detail.

(d) [Counterparty] shall, upon reasonable notice, permit [Tenant] or its
    designated auditors to audit [Counterparty]'s books, records, and
    accounts to verify compliance with this certification.

(e) [Counterparty] shall promptly notify [Tenant] in writing of any actual
    or suspected breach of this certification.
```

The certification is signed annually and stored as a `fcpa_ukba_certification` artefact bound to the master contract.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.active_packs.contains("sox-404") &&
  resource.contract.requires_anti_corruption_certification == true &&
  resource.fcpa_ukba_certification == null
};

forbid (
  principal,
  action == Action::"ContractExecute",
  resource is Contract
) when {
  resource.counterparty.sanctions_check_state matches "CheckedFlagged" &&
  resource.counterparty.sanctions_match.list == "FCPA_DOJ_ENFORCEMENT"
};
```

## Retention

FCPA books-and-records and certifications retained for 7 years (FCPA + SOX-404 retention floor).

## Audit events

- `oya.contract.lifecycle.management.anti_corruption.risk_clause_flagged`
- `oya.contract.lifecycle.management.anti_corruption.certification_executed`
- `oya.contract.lifecycle.management.anti_corruption.counterparty_pep_match`
- `oya.contract.lifecycle.management.anti_corruption.audit_rights_invoked`

## Standards references

- 15 USC §§ 78dd-1, 78dd-2, 78dd-3, 78m, 78ff (FCPA).
- UK Bribery Act 2010 (especially §§ 1, 2, 6, 7).
- OECD Convention on Combating Bribery of Foreign Public Officials in International Business Transactions (1997).
- US DOJ + SEC FCPA Resource Guide (2nd ed. 2020).
- UK Ministry of Justice Section 7 Guidance ("adequate procedures").
- ISO 37001:2016 Anti-bribery management systems.
- Transparency International Corruption Perceptions Index (CPI).
