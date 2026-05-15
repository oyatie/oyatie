---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M05-P03
title: Regulator Attestation Per Region in Scope
status: stub
purpose: Achieve regulator-equivalent attestation per region in scope (KR CSAP + K-ISMS-P + KCMVP production; JP ISMAP; US FedRAMP; EU GAIA-X; etc.).
---

# M05-P03 — Regulator Attestation

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.8 W-Cloud-Stable + W-Search-Stable gates.

## Acceptance
- KR: CSAP + K-ISMS-P + KCMVP HSM all in production attested.
- ≥ 1 additional region attested (JP ISMAP / US FedRAMP / EU GAIA-X).
- Auditor experience: evidence-pack regeneration ≤ 4 hours from request.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | KR CSAP + K-ISMS-P + KCMVP production attestation | stub | [`IP-001-kr-attestation-production.md`](IP-001-kr-attestation-production.md) |
| IP-002 | Second-region attestation (JP/US/EU per council) | stub | [`IP-002-second-region-attestation.md`](IP-002-second-region-attestation.md) |

## Estimated parallelism
2 agents.

## Symbols-touched
`regional-packs/{kr,jp,us,eu}/attestation/`, `docs/COMPLIANCE-MATRIX.md` rows, `crates/oya-ops-compliance-attestation-pack-app`.

## Agent-handoff
```
icm store -t context-oyatie -c "M05-P03 complete: KR + second region regulator-attested; auditor evidence pack ≤4h" -i critical -k "M05,P03,attestation,complete"
```
