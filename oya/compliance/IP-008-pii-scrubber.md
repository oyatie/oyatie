---
microservice: compliance
ip: IP-008
title: PII scrubber (DSAR export redaction + k-anonymity + format-preserving encryption)
status: Drafting
authority_tier: 3
owner: axis-security
co_owners: [axis-compliance]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0209]
---

# IP-008 — PII scrubber

## Purpose

Scrub PII from DSAR export payloads before delivery, applying:

- **Pseudonymization** — replace subject identifiers with stable pseudonyms.
- **Redaction** — drop fields not requested by the subject.
- **k-anonymity** — when records contain quasi-identifiers (zip + DOB + gender), suppress / generalize until k≥5.
- **Format-preserving encryption (FPE)** — for fields the subject needs in original format (e.g., phone, SSN-like ids) but where storage requires encryption-at-rest.

## Acceptance criteria

1. Per-field scrub policy declared at `policy/pii-scrub-policy.json` (closed set of field-name → action).
2. k-anonymity validator: returns minimal `k` for a record set; blocks export when `k<5`.
3. FPE adapter (FF1 / AES-FPE) for phone + similar fields; key in OpenBao per ADR-0145.
4. Integration tests: scrub-applied-on-export + k-anonymity-blocks + FPE-roundtrip + raw-PII-never-emitted + cross-tenant-scrub-isolation.
5. ≥ 6 integration tests.

## Scrub-policy shape

```json
{
  "field": "subject.email",
  "scrub_kind": "pseudonymize",
  "pseudonym_key_secret": "secret/compliance/email-pseudonym-key"
}
{
  "field": "subject.dob",
  "scrub_kind": "generalize",
  "generalize": {"granularity": "year"}
}
{
  "field": "subject.phone",
  "scrub_kind": "fpe",
  "fpe_algorithm": "ff1-aes128",
  "fpe_key_secret": "secret/compliance/phone-fpe-key"
}
```

## k-anonymity check

For a DSAR export containing N records:

1. Compute the multiset of quasi-identifier tuples (zip, DOB-year, gender).
2. `k = min(count(tuple))` across all tuples.
3. If `k < 5`, generalize one column (e.g., zip → first 3 digits) and recompute.
4. Block export if k stays < 5 after all generalizations.

## Risk + mitigation

- **Risk:** scrub policy gap — a new field added to Ontology without scrub rule. **Mitigation:** advisory gate scans Ontology schema vs scrub-policy; flags unknown fields.
- **Risk:** FPE key compromise. **Mitigation:** keys in OpenBao; quarterly rotation; key version embedded in ciphertext.

## Acceptance evidence

`evidence/ip-008-pii-scrubber-acceptance.json`.

## Cross-references

- ADR-0145 — identity + secret substrate.
- ADR-0209 — substrate authority.
- IP-003 — DSAR pipeline (consumer).
