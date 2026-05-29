---
id: ADR-DRIVE-0006
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, council-architecture, ops-security, compliance, council-privacy
owner: compliance + axis-drive + ops-security
supersedes: []
superseded_by: []
related: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-DRIVE-0001, ADR-DRIVE-0004]
related_artifacts:
  - microservices/drive/PRD.md (§FR-12 immutability; §FR-21 legal-hold; AC-09 AC-10 worm correctness; AC-14 audit-chain)
  - microservices/drive/policy/tenant-scope.cedar (worm + legal-hold forbid clauses)
  - microservices/drive/runbooks/immutability-tier-violation.md
  - microservices/drive/slos/immutability-tier-correctness.openslo.yaml
purpose: |
  Lock in the WORM (write-once-read-many) immutability + legal-hold semantics
  for the drive µservice. Reach SEC 17a-4(f) + FINRA Rule 4511 + HIPAA
  §164.316 compliance posture. Match AWS S3 Object Lock + Box governance
  feature parity.
---

# ADR-DRIVE-0006: WORM immutability via object-lock compliance mode (AWS S3 Object Lock terminology); per-pack retention floor; legal-hold cascade; two-person-rule on any release path

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-drive §FR-12 + §FR-21 + AC-09 + AC-10 + AC-14 mandate WORM (write-once-read-many) immutability + legal-hold preservation that survives even tenant-root attempted deletion. SEC 17a-4(f), FINRA Rule 4511, and HIPAA §164.316 compliance gates on this functionality. Per the threat-model §T-T-03 + §T-E-04, WORM bypass is a zero-tolerance Sev-1.

Industry precedent:

| Competitor | WORM | Legal hold | Compliance mode |
|---|---|---|---|
| AWS S3 Object Lock | yes | yes | "compliance" (immutable even by root) + "governance" (admin-overridable) |
| Wasabi | yes | yes | compliance only |
| Backblaze B2 | yes | yes | compliance |
| Box | yes | yes | compliance + governance |
| Dropbox | partial | yes | governance only (admin can override) |
| Google Drive | no (governance only) | yes | no compliance mode |
| Proton Drive | no | yes | no |

Candidate WORM enforcement layers:
- **Application-layer only** — enforced by usecase code; bypassed by raw DB or raw object-store access. Insufficient.
- **DB-layer** — Postgres role permissions revoke DELETE/UPDATE on `immutability_record` + soft-delete only on files. Necessary but not sufficient (object-store still mutable).
- **Object-store-layer** — S3 Object Lock compliance mode at bucket + per-object retention. Sufficient for byte immutability.
- **Application + DB + Object-store layers (defence in depth)** — every layer refuses. Robust against single-layer bypass.

Regulatory citations:
- **SEC 17a-4(f)** — requires WORM storage for broker-dealer records. ([sec.gov/rules/final/34-44238](https://www.sec.gov/rules/final/34-44238.htm))
- **FINRA Rule 4511** — supplementary records retention. ([finra.org/rules-guidance/rulebooks/finra-rules/4511](https://www.finra.org/rules-guidance/rulebooks/finra-rules/4511))
- **HIPAA 45 CFR §164.316** — documentation retention ≥ 6y. ([hhs.gov/hipaa](https://www.hhs.gov/hipaa))
- **KR-FSS supervisory regulations** — 5y retention floor for financial-sector tenants.

Per ADR-0133 axis-2 security-conformance + axis-4 industry-citation, the chosen WORM strategy must match the AWS S3 Object Lock compliance-mode terminology + semantics.

## Decision

The drive µservice ships **defence-in-depth WORM enforcement across application + DB + object-store layers; per-pack retention floor; legal-hold cascade; two-person-rule on any release path**:

### WORM mode

- **Object-lock mode**: `compliance` (AWS S3 terminology).
- **Retention floor**: per-pack default + per-tenant override; minimum bounded by pack regulation (e.g., pack-us → 6y for broker-dealer SEC 17a-4; pack-us-healthcare → 6y for HIPAA §164.316; pack-kr → 5y for KR-FSS financial-sector).
- **Retention floor cannot be reduced** by any principal, including tenant-root.
- **Retention floor can be extended** only.
- **Maximum retention**: 99 years (storage-layer cap; per AWS S3 Object Lock semantics).
- **Governance mode**: NOT supported; the µservice ships compliance-mode-only to avoid the "admin override" loophole that competitors expose.

### Defence-in-depth enforcement

1. **Application layer**: `oya-drive-file-store-usecase` refuses purge / delete / overwrite when `worm_tier_enabled == true && now < worm_retention_floor`.
2. **Cedar policy layer**: `policy/tenant-scope.cedar` forbid clause on purge action when WORM open.
3. **Postgres layer**: application role has no DELETE/UPDATE on `immutability_record` table.
4. **Object-store layer**: per-object retention set at PutObject time via S3 Object Lock; all three backends (Garage / SeaweedFS / SeaweedFS per ADR-DRIVE-0001) support compliance-mode lock.

### Legal hold

- **Trigger**: tenant compliance officer opens hold on a file; emits audit-chain seal.
- **Effect**: file (and all versions) refuse purge until hold released — regardless of WORM tier state.
- **Release**: requires two-person rule (compliance officer + ops-security on-call); emits audit-chain seal; even after release, WORM retention floor still applies.

### Two-person rule on any release path

- **Hard-delete after retention floor**: requires compliance officer + ops-security on-call concurrent approval.
- **Legal-hold release**: requires compliance officer + ops-security on-call concurrent approval.
- **WORM retention floor extension reduction (any time)**: REFUSED — retention floor is monotonically non-decreasing.

### Periodic integrity scan

- Every 1h: scan all WORM-tier records in Postgres vs object-store presence; mismatch → `oya_drive_immutability_integrity_scan_delta_total` rises → Sev-1 alarm.

### Restoration

- If WORM violation detected within RPO (≤ 60s), runbook `immutability-tier-violation.md` documents emergency restoration from secondary-region replica.

## Alternatives Considered

### A. Application-layer-only enforcement

- **Pros**:
  - Simplest; no DB/object-store config needed.
- **Cons**:
  - Bypassable via raw DB or raw object-store access.
  - Doesn't satisfy SEC 17a-4(f) "non-rewriteable / non-erasable" requirement.
- **Rejected** outright; cannot pass SEC 17a-4 attestation.

### B. DB-layer + application-layer (no object-store object-lock)

- **Pros**:
  - Stops common bypass paths via the application.
- **Cons**:
  - Object-store still mutable; an admin with object-store credentials could bypass.
- **Rejected** for the same SEC 17a-4 reasoning.

### C. Governance mode (admin-overridable) + compliance mode optional

- **Pros**:
  - Match Dropbox / Google Drive feature set.
- **Cons**:
  - Doesn't satisfy SEC 17a-4(f) for broker-dealer tenants.
  - "Admin override" is the well-known WORM-bypass loophole.
- **Rejected** in favour of compliance-mode-only.

### D. Defence-in-depth compliance-mode + integrity-scan + two-person-rule  ← **CHOSEN**

- **Pros**:
  - Satisfies SEC 17a-4(f) + FINRA 4511 + HIPAA §164.316.
  - Match AWS S3 Object Lock compliance-mode semantics + Box governance feature parity.
  - No "admin override" loophole.
  - Periodic integrity scan + emergency restoration runbook close the residual-risk window.
- **Cons**:
  - Three layers to keep in sync; operator complexity.
  - Files under WORM cannot be deleted even with regulator approval before retention floor; tenant comms must clarify at WORM election time.
  - "Cannot reduce retention" is a deliberate design choice — tenants who elect WORM accept this constraint.
- **Accepted**.

## Consequences

### Positive

- **SEC 17a-4(f) attestation-ready**: compliance-mode object-lock + audit-chain Ed25519 seal + 6y retention floor.
- **FINRA Rule 4511 retention-ready**: same as SEC 17a-4 with FINRA-specific record classes.
- **HIPAA §164.316 documentation retention**: 6y WORM tier covers.
- **KR-FSS 5y retention** for financial-sector tenants: pack-kr overlay enforces.
- **Zero "admin override" loophole**: no governance mode.
- **Integrity scan**: catches any single-layer bypass within ≤ 1h.

### Negative

- **WORM is irreversible**: tenant cannot reduce retention even with regulator approval; tenant must understand this at election time. Mitigation: explicit two-person tenant-side election ceremony + audit-chain record.
- **DSR right-to-erasure tension**: per GDPR Art. 17, erasure must be honoured; per WORM, retention floor applies. Reconciliation: erasure refused while WORM open; tenant DPA + tenant-of-tenant comms must clarify. Documented in `dpia.md` R-08 + R-17.
- **Operational complexity**: three layers (app + DB + object-store) to maintain.
- **Storage cost**: WORM-tier objects cannot be auto-tiered to archive within retention floor; storage cost higher than non-WORM.

### Hyrum's Law

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law":
- **Legacy `oya-drive-domain` had no WORM tier**; only soft-delete + retention. Strangler-migration consumers using legacy soft-delete pattern need explicit WORM election to gain compliance-mode behaviour. Documented in `migration-from-connect.md` §"Net-new boundaries" + Hyrum #6.

### Operational

- **New CI lane**: `oya-governance-worm-enforcement-multi-layer` (BLOCKER) — refuses any file-purge code path that doesn't check all three enforcement layers.
- **New CI lane**: `oya-governance-worm-retention-monotonic` (BLOCKER) — refuses any code path that could reduce a retention floor.
- **Regression test**: `tests/worm-refuses-purge.rs` (BLOCKER) — explicitly validates AC-09.
- **Regression test**: `tests/worm-refuses-tenant-root.rs` (BLOCKER) — explicitly validates tenant-root cannot purge.
- **Regression test**: `tests/legal-hold-preserves.rs` (BLOCKER) — explicitly validates AC-10.
- **Periodic integrity scan job**: hourly; runs as a worker; mismatch alarm.
- **Runbook**: `immutability-tier-violation.md` Sev-1 path.
- **SLO**: `immutability-tier-correctness.openslo.yaml` — 100% correctness; zero-tolerance.

### Regulatory

- **SEC 17a-4(f)** — "non-rewriteable / non-erasable" storage requirement satisfied via compliance-mode object-lock.
- **FINRA Rule 4511** — supplementary records retention satisfied.
- **HIPAA 45 CFR §164.316** — ≥ 6y documentation retention satisfied.
- **KR-FSS** — 5y financial-sector retention floor satisfied via pack-kr overlay.
- **eIDAS 910/2014 Art. 26** — AdES audit-chain seal satisfies electronic-record integrity (combined with audit-chain Ed25519 per Bominal ADR-0028).

## Verification

- [ ] WORM refuses purge — `cargo nextest run --test worm_refuses_purge`.
- [ ] WORM refuses tenant-root purge — `cargo nextest run --test worm_refuses_tenant_root`.
- [ ] Retention floor monotonic — `cargo nextest run --test worm_retention_monotonic`.
- [ ] Legal hold preserves past retention — `cargo nextest run --test legal_hold_preserves`.
- [ ] Integrity scan delta = 0 — `cargo run -p oya-dev-cli -- gate validate worm-integrity-scan --microservice drive`.
- [ ] AC-09 + AC-10 + AC-14 — full E2E suite.

## References

- AWS S3 Object Lock specification — compliance mode + governance mode.
- AWS S3 Object Lock + SEC 17a-4 attestation guide.
- SEC 17a-4(f) — `sec.gov/rules/final/34-44238`.
- FINRA Rule 4511 — `finra.org/rules-guidance/rulebooks/finra-rules/4511`.
- HIPAA 45 CFR §164.316 — `hhs.gov/hipaa`.
- KR-FSS supervisory regulations (5y financial-sector retention).
- eIDAS Regulation (EU) 910/2014 Art. 26 (AdES integrity).
- Box governance documentation.
- Wasabi immutability specification.
- Backblaze B2 Object Lock specification.
- ADR-0028 (Bominal) — audit chain.
- ADR-0117 — data residency.
- ADR-0140 — Cedar policy enforcement (`tenant-scope.cedar` WORM + legal-hold forbid).
- ADR-DRIVE-0001 — object-storage substrate (Garage/SeaweedFS/SeaweedFS object-lock compliance support).
- ADR-DRIVE-0004 — encryption-at-rest (envelope ciphertext is what's WORM'd).
- `microservices/drive/PRD.md` §FR-12 + §FR-21 + AC-09 + AC-10 + AC-14.
- `microservices/drive/policy/tenant-scope.cedar`.
- `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`.
- `microservices/drive/runbooks/immutability-tier-violation.md`.
- `microservices/drive/threat-model.md` T-T-03 + T-E-04.
- `microservices/drive/dpia.md` R-08 + R-17.
- `microservices/drive/migration-from-connect.md` Hyrum #6.
- `feedback_no_silent_regression.md`.
