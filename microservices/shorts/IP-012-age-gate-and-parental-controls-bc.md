---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-012-age-gate-and-parental-controls-bc
status: pending
owner: axis-shorts + council-privacy + ops-legal
depends_on: [IP-002]
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-pack-aware-age-gate]
---

# IP-012: age-gate + parental-controls BC end-to-end

## Intent

- `age-gate` BC: pack-aware age-gate; signup attestation; minor-protection routing.
- `parental-controls` BC: linked-account parental supervision; per-minor controls (screen-time, content-filter level, DM-restriction).

Per pack regulatory floor:
- pack-kr: minor < 14y per KR 청소년 보호법 + KR PIPA Art. 8.
- pack-eu: minor < 16y per GDPR Art. 8 (member states may lower to 13y).
- pack-us: minor < 13y per COPPA 15 USC §6501.
- pack-uk: minor protection per UK Online Safety Act 2023.
- pack-au: minor < 16y per AU Online Safety Act 2021 + BOSE 2022.
- pack-br: minor < 12y per LGPD Art. 14.
- pack-in: minor protection per DPDPA 2023 §9.
- pack-us (CA): CA AB-2273 minor protection.
- pack-us (UT): UT Social Media Regulation Act parental consent.

## ChangeSet boundary

7 + 7 = 14 crates.

## Concrete File Targets

Key entities: `AgeAttestation`, `AgeBracket`, `MinorProtectionPolicy`, `ParentalLink`, `ParentalControlPolicy`, `MinorScreenTime`.

Ports: `AgeAttestationStore`, `ParentalLinkStore`, `MinorProtectionPolicyEngine`.

Data class: `SENSITIVE_CHILD_PROTECTION` on age + parental rows; separate Postgres tables `shorts_age_attestations` + `shorts_parental_links`; restricted Cedar entitlement.

Minor-account defaults applied at signup:
- Chronological-only feed.
- Algorithmic-recommendation OFF.
- DM-restricted.
- Adult content restricted to `general_audience`.

## Acceptance Gates

```bash
cargo build -p oya-shorts-age-gate-rest
cargo build -p oya-shorts-parental-controls-rest
cargo nextest run -p oya-shorts-age-gate-{kernel,domain,usecase,adapter-postgres}
cargo nextest run -p oya-shorts-parental-controls-{kernel,domain,usecase,adapter-postgres}
cargo run -p oya-dev-cli -- gate validate pack-aware-age-gate --microservice shorts
```

E2E: minor signup on pack-eu requires parental consent attestation; parental-link established; minor-account defaults applied.

## Halt Conditions

- Pack threshold drift — verify regulatory current.
- Age-attestation table read by non-entitled actor — Sev-1.

## Next IP

[`IP-013-accessibility-captions-bc.md`](IP-013-accessibility-captions-bc.md)

## References

- PRD FR-18, FR-19.
- ADR-SHORTS-0006 (minor protection + age-gate).
- `runbooks/age-gate-bypass-incident.md`.
- GDPR Art. 8; COPPA 15 USC §6501; KR 청소년 보호법; UK OSA 2023; CA AB-2273; UT SMRA; LGPD Art. 14; DPDPA §9; AU OSA + BOSE 2022.
