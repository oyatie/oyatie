---
doc_class: JurisdictionOverlay
microservice: contract-lifecycle-management
dimension_id: L-005
authoritative_source: Uniform Electronic Transactions Act (NCCUSL 1999) + per-state adoption
related_packs: [esign]
date: 2026-05-21
---

# UETA Inter-State Framework — US States

UETA was promulgated by NCCUSL in 1999. As of 2026-05-21, 49 US states + DC + USVI + Puerto Rico have adopted UETA. New York has not adopted UETA but enacted the Electronic Signatures and Records Act (ESRA, NY Tech Law § 304) which is substantively similar.

## Adoption table

| Jurisdiction | UETA adopted | Effective date | Variations |
|---|---|---|---|
| Alabama | Yes | 2001 | Standard |
| Alaska | Yes | 2004 | Standard |
| Arizona | Yes | 2000 | Adds explicit retention rules |
| Arkansas | Yes | 2001 | Standard |
| California | Yes | 1999 | Civil Code § 1633.1-.17; adds consumer notice rules |
| Colorado | Yes | 2002 | Standard |
| Connecticut | Yes | 2002 | Standard |
| Delaware | Yes | 2000 | Standard |
| DC | Yes | 2001 | Standard |
| Florida | Yes | 2000 | Standard |
| Georgia | Yes | 2009 | Adopted late; substantive |
| Hawaii | Yes | 2000 | Standard |
| Idaho | Yes | 2000 | Standard |
| Indiana | Yes | 2000 | Standard |
| Iowa | Yes | 2000 | Standard |
| Kansas | Yes | 2000 | Standard |
| Kentucky | Yes | 2000 | Standard |
| Louisiana | Yes | 2001 | Adds notarial requirements |
| Maine | Yes | 2000 | Standard |
| Maryland | Yes | 2000 | Standard |
| Massachusetts | Yes | 2004 | Standard |
| Michigan | Yes | 2000 | Standard |
| Minnesota | Yes | 2000 | Standard |
| Mississippi | Yes | 2001 | Standard |
| Missouri | Yes | 2003 | Standard |
| Montana | Yes | 2001 | Standard |
| Nebraska | Yes | 2000 | Standard |
| Nevada | Yes | 2001 | Standard |
| New Hampshire | Yes | 2001 | Standard |
| New Jersey | Yes | 2001 | Standard |
| New Mexico | Yes | 2003 | Standard |
| **New York** | **NO** | n/a | ESRA / NY Tech Law § 304 (substantively similar but distinct) |
| North Carolina | Yes | 2000 | Standard |
| North Dakota | Yes | 2001 | Standard |
| Ohio | Yes | 2000 | Standard |
| Oklahoma | Yes | 2000 | Standard |
| Oregon | Yes | 2001 | Standard |
| Pennsylvania | Yes | 2000 | Standard |
| Puerto Rico | Yes | 1998 | Pre-UETA territorial law substantively similar |
| Rhode Island | Yes | 2000 | Standard |
| South Carolina | Yes | 2004 | Standard |
| South Dakota | Yes | 2000 | Standard |
| Tennessee | Yes | 2001 | Standard |
| Texas | Yes | 2001 | Business & Commerce Code Chapter 322 |
| US Virgin Islands | Yes | 2003 | Standard |
| Utah | Yes | 2000 | Standard |
| Vermont | Yes | 2003 | Standard |
| Virginia | Yes | 2000 | Standard |
| Washington | Yes | 2020 | Substantive overhaul; updated for cloud era |
| West Virginia | Yes | 2001 | Standard |
| Wisconsin | Yes | 2004 | Standard |
| Wyoming | Yes | 2001 | Standard |
| Illinois | Yes | 2021 | Adopted late (Electronic Commerce Security Act repealed 2021) |

## UETA core provisions

### § 7 — Legal recognition

A record or signature may not be denied legal effect or enforceability solely because it is in electronic form.

### § 8 — Provision of information in writing

If a law requires information to be in writing, an electronic record satisfies the requirement if accessible and retainable.

### § 9 — Attribution and effect of electronic record and electronic signature

(a) An electronic record or electronic signature is attributable to a person if it was the act of the person.
(b) The effect of an electronic record or electronic signature is determined from the context and surrounding circumstances at the time of its creation, execution, or adoption.

### § 10 — Effect of change or error

If a change or error in an electronic record occurs in a transmission between parties, the burden is on the sender to prove the record received is the record sent.

### § 11 — Notarization and acknowledgment

If a law requires notarization, acknowledgment, verification, or oath, the requirement is satisfied if the electronic signature of the person authorized to perform those acts, together with all other information required to be included, is attached to or logically associated with the signature or record.

### § 12 — Retention of electronic records; originals

(a) An electronic record satisfies a law that requires retention if the record accurately reflects the information set forth in the original.
(b) A requirement to retain a record in its original form is satisfied by an electronic record.

### § 13 — Admissibility in evidence

Evidence of a record or signature may not be excluded solely because it is in electronic form.

### § 14 — Automated transactions

An electronic agent may be a party to a transaction. The transaction's terms are deemed agreed if the parties (a) had reason to know the electronic agent's actions or (b) confirmed acceptance.

### § 15 — Time and place of sending and receipt

Default rule: a record is received when (i) it enters an information processing system that the recipient has designated for receipt and (ii) it is in a form capable of being processed by that system.

## Per-state variations the µservice tracks

### California (Civil Code § 1633.1-.17)

- Adds consumer notice rules requiring explicit consumer consent for electronic delivery of consumer notices.
- Composes with `gdpr` and CCPA/CPRA when the consumer is California-resident.

### Louisiana

- Adds notarial requirements for certain contract types; the µservice integrates with Louisiana-licensed remote online notary providers (RON).

### New York (ESRA / NY Tech Law § 304)

- Substantively similar to UETA but distinct.
- Specifically excludes wills, codicils, testamentary trusts.
- Requires the parties to agree to use electronic signatures (similar to UETA § 5(b)).

### Washington (2020 overhaul)

- Explicit recognition of distributed ledger / blockchain signatures.
- Updated automated-transaction rules for cloud-era electronic agents.

### Texas (Business & Commerce Code Chapter 322)

- Adopts UETA largely as-is.
- Adds specific real-estate transaction signing requirements (texas.gov/RON).

### Illinois (2021 overhaul)

- Repealed earlier Electronic Commerce Security Act.
- Adopted modern UETA verbatim.

## Cedar gate composition

For US contracts, the µservice applies the federal ESIGN floor + UETA per-state overlay:

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.active_packs.contains("esign") &&
  resource.governing_law.country == "US" &&
  resource.governing_law.state == "NY" &&
  resource.ueta_or_esra_evidence == null
};

forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.governing_law.state == "LA" &&
  resource.contract_type in ["real_estate_purchase", "real_estate_lease", "marriage_contract"] &&
  resource.louisiana_notarial_evidence == null
};
```

## Per-state delta tracking

When the µservice detects a state-law change (e.g. new state adopts UETA or amends), the change is recorded in `jurisdictions/ueta-states-changelog.md` and the affected tenants are notified at next login. Existing contracts are not automatically re-validated; new contracts use the new rule.

## Audit event

`oya.contract.lifecycle.management.jurisdiction.ueta_state_resolved` with dimensions:

- tenant_id, contract_id, governing_law_state
- ueta_adopted, esra_applies, variation_set
- audit_event_id

## Standards references

- Uniform Electronic Transactions Act (1999) — NCCUSL.
- 15 USC § 7001-7006 (ESIGN Act).
- NY Tech Law § 304 (ESRA).
- Per-state codifications listed above.
