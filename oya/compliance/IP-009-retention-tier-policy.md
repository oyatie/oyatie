---
microservice: compliance
ip: IP-009
title: Retention tier policy (hot / warm / cold + statutory minima + GDPR-erasure interplay)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0184, ADR-0209]
---

# IP-009 — Retention tier policy

## Purpose

Define + enforce per-framework retention tiers. Hot tier for active audit windows; warm tier for the past quarter; cold tier for statutory long-term retention. Handle the GDPR-erasure vs SOC 2-retention conflict (GDPR erasure overrides SOC 2 long retention for subject-identifiable data; non-subject artifacts unaffected).

## Acceptance criteria

1. `policy/retention-tier-policy.json` declares per-framework + per-artifact-kind retention windows.
2. Tier transition cron job validated (per IP-006).
3. GDPR erasure honors statutory minimum for non-subject artifacts; erases subject artifacts.
4. Retention enforcement gate: artifacts past max retention are deleted (audited).
5. ≥ 5 integration tests.

## Retention table

| Framework | Artifact kind | Hot (active) | Warm | Cold | Total min |
|---|---|---|---|---|---|
| SOC 2 | All | 90 days | 1 year | 7 years | 7 years |
| GDPR | DSAR record | 30 days | 6 months | 3 years | 3 years (Art. 30 RoPA) |
| GDPR | Subject data (under DSAR delete) | erased on request | erased | erased | 0 (Art. 17 erasure) |
| HIPAA | All | 90 days | 1 year | 6 years | 6 years (statutory) |
| PCI-DSS | All | 90 days | 1 year | 3 years | 3 years |

## GDPR-erasure-vs-SOC-2-retention conflict resolution

Per Art. 17, subject can demand erasure; SOC 2 requires retention. Resolution:

- **Non-subject-linked SOC 2 evidence** (CI artifact hashes, deploy receipts) — retain 7 years; subject erasure doesn't touch these.
- **Subject-linked evidence** (DSAR completion record for subject S) — pseudonymize after 30 days; the artifact persists with the pseudonym (proves DSAR was honored); raw subject identity erased.
- **Subject's primary records** (in other µservices' Ontology projections) — erased on Art. 17 cascade.

## Risk + mitigation

- **Risk:** statutory minimum miss → regulatory penalty. **Mitigation:** retention gate fails-closed; manual override requires ADR exception.
- **Risk:** retention exceeded by accident (storage churn) — Sev-2 not Sev-1 because no compliance penalty.

## Acceptance evidence

`evidence/ip-009-retention-tier-policy-acceptance.json`.

## Cross-references

- ADR-0184 — storage tier layering.
- ADR-0209 — substrate authority.
- IP-006 — SeaweedFS storage adapter.
