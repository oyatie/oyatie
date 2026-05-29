# IP-003 — VNA blob substrate (13-nine durability + erasure coding)

`scope: oya-imaging-adapter-cloud-storage + oya-imaging-vna-federation-app`
`wave_target: 16-imaging-substrate`
`adr_binding: ADR-MS-003 + ADR-0248 (cellular shape) + ADR-0251 (compliance packs)`

## Objective

Stand up the VNA blob substrate with 13-nine durability via erasure coding 14+4 across cells per pack policy. Support per-tenant envelope encryption with KMS-wrapped DEKs (BYOK opt-in per ADR-0255 §D-4).

## Scope

1. `oya-imaging-adapter-cloud-storage` — DICOM pixel data blob persistence.
2. Object-key pattern: `<tenant>/<study_uid>/<series_uid>/<instance_uid>.dcm`.
3. Envelope encryption with per-tenant DEK + KMS-wrapped KEK.
4. Erasure-coded 14+4 placement.
5. Cross-AZ replication.
6. Cross-cell-within-pack replication for sovereign cells.
7. Deep-archive tiering after configurable age (default 2 years).
8. Study-level cryptographic shred for GDPR right-to-erasure (FR-PACS-007).
9. Deduplication by SOP Instance UID + transfer syntax.

## Acceptance criteria

- Durability test (Monte-Carlo simulation) shows ≥13 nines at 14+4 + cross-cell.
- Cryptographic shred test: erased blob cannot be recovered via any of the 18 erasure slices; KMS DEK is rotated out.
- Envelope encryption test: blob ciphertext is opaque without KMS access.
- Pack-policy enforcement test: GDPR pack cells do not replicate to non-EU regions.

## Dependencies

- IP-001.
- `cloud-storage` µservice.
- `cloud-kms` µservice.
- `compliance` pack overlay.

## Risks

- Erasure-coding latency vs. p95 < 1s image-pull SLO; mitigate with edge cache + tiered deep-archive.
- KMS-wrapped DEK rotation under load.
- Cross-cell replication consistency.

## Out-of-scope

- Federation (IP-004).

## Testing strategy

- Durability simulation.
- Pack-policy negative tests.
- Cryptographic-shred verification.

## Estimated effort

- 8–10 person-weeks.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/imaging/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `KR-PIPA-2023-amendment` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=semi-annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-003-vna-blob-substrate.md:39` - - Erasure-coding latency vs. p95 < 1s image-pull SLO; mitigate with edge cache + tiered deep-archive..
