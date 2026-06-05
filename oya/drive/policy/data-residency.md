---
doc_class: PolicyArtifact
template_id: TPL-POLICY-DOC
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: ops-security + council-privacy + axis-drive
related_adrs: [ADR-0117, ADR-0140 (retired per ADR-0145), ADR-DRIVE-0001, ADR-DRIVE-0006]
doc_status: published
---

# Data Residency — drive µservice

## Purpose

Define per-pack data-residency posture and cross-pack transfer constraints for drive bytes + metadata + audit records.

## Default posture

**Bytes pinned to pack region.** A tenant's file bytes, metadata, search index, preview cache, virus-scan verdict, DLP verdict, audit-chain seals, and share-link records all reside in the tenant's pack region. Cross-pack replication forbidden by default.

## Pack-to-region mapping

| Pack | Primary region | Secondary region (DR; same pack) | Object-store backend |
|---|---|---|---|
| pack-kr | ap-seoul-1 | ap-seoul-2 | Garage (edge-distributed) |
| pack-eu | eu-frankfurt-1 | eu-amsterdam-1 | Garage |
| pack-us | us-east-1 | us-west-2 | Garage |
| pack-us-healthcare | us-east-1-hipaa | us-west-2-hipaa | SeaweedFS (single-cluster HIPAA-eligible) |
| pack-jp | ap-tokyo-1 | ap-osaka-1 | Garage |
| pack-sg | ap-singapore-1 | ap-jakarta-1 | Garage |
| pack-au | ap-sydney-1 | ap-melbourne-1 | Garage |
| pack-in | ap-mumbai-1 | ap-hyderabad-1 | Garage |
| pack-br | sa-east-1 | sa-saopaulo-2 | Garage |
| pack-ae | me-dubai-1 | me-abudhabi-1 | Garage |
| pack-ksa | me-riyadh-1 | me-jeddah-1 | Garage |

## Cross-pack transfer (gated)

Cross-pack transfer is permitted ONLY when:

1. Tenant DPA includes SCC clause (GDPR Arts. 44–46 / KR PIPA Art. 17 / equivalent per pack).
2. Tenant explicitly opts-in via tenant-portal cross-pack-grant flow.
3. Cedar policy `cross_pack_reader` admits per-request (`tenant-scope.cedar` cross-pack permit clause).
4. Audit-chain seal emitted on every cross-pack transfer event.
5. Workflow event `CrossPackTransferInitiated` raised; consumed by council-privacy review queue.

## Cross-pack transfer types

| Type | Cedar-gated | Use case |
|---|---|---|
| Cross-pack read (share-link viewer in different pack) | `cross_pack_reader` | external collaboration |
| Cross-pack restore (DR) | `cross_pack_restore_grant` | tenant DR with SCC |
| Cross-pack backup-replication | `cross_pack_backup_grant` | per-tenant DR option |

## Per-pack restrictions

### pack-kr (KR PIPA Art. 17)

- Cross-pack transfer of sensitive PII (Art. 23) refused outright.
- Cross-pack transfer of any data requires SCC-equivalent (KR PIPA Art. 17).
- KR-FSS tenants: 5y retention floor; cross-pack retention reset forbidden.

### pack-eu (GDPR Chapter V)

- Cross-pack transfer to adequate countries (UK, JP, KR, SG, AU [under APRR], NZ, IL, AR, UY, CA-PIPEDA, Faroe, Andorra, Guernsey, Jersey, Isle of Man) permitted with SCC.
- Cross-pack transfer to non-adequate countries requires Transfer Impact Assessment (TIA) + supplementary measures (Schrems II).

### pack-us-healthcare (HIPAA)

- Cross-pack transfer outside US-healthcare cluster refused outright.
- Cross-pack transfer requires BAA in place at destination.

### pack-us (SEC 17a-4 broker-dealer)

- WORM-tier files cannot transfer cross-pack except via SEC-supervised DR procedure.

### pack-jp (APPI Art. 24)

- Cross-pack transfer to "adequate" countries only; cross-pack consent-gated.

### pack-sg (PDPA §26)

- Cross-pack transfer permitted to "comparable protection" jurisdictions.

### pack-au (Privacy Act APP 8)

- Cross-pack transfer subject to APP 8 accountability.

### pack-in (DPDPA §16)

- Cross-pack transfer to whitelisted countries (DPDPA §16 + MeitY rules).

### pack-br (LGPD Art. 33)

- Cross-pack transfer requires ANPD-approved mechanism.

### pack-ae (UAE PDPL Art. 22)

- Cross-pack transfer requires UAE Data Office adequacy or equivalence.

### pack-ksa (KSA PDPL Art. 29)

- Cross-pack transfer requires SDAIA-approved mechanism.

## Enforcement layers

### Layer 1 — Ingress

- Per-pack DNS / edge proxy refuses requests carrying a different pack claim.
- LEAN check `oya-check-pack-pinning` refuses build if any drive crate hard-codes a cross-pack route.

### Layer 2 — Cedar

- `tenant-scope.cedar` cross-pack forbid clauses + cross-pack-grant permit clauses.

### Layer 3 — Storage

- Per-pack object-store backend deployed in the pack's region only.
- Cross-pack replication off by default; explicit Cedar grant required to enable.

## Cross-pack transfer audit

Every cross-pack transfer event emits to audit-chain with:
- `transfer_initiator_principal`
- `from_pack`
- `to_pack`
- `scc_pointer` (DPA SCC clause version)
- `tenant_dpa_version`
- `cedar_policy_admission_decision`
- `file_id_or_pattern`
- `transfer_purpose`

## Verification

```bash
buck2 build //:quality-lane-registry-authority-check # lane=pack-pinning --microservice drive
buck2 build //:quality-lane-registry-authority-check # lane=cross-pack-cedar-coverage --microservice drive
cargo nextest run -p oya-drive-file-store-domain -- pack_pinning_invariant
```

## References

- ADR-0117 — Cloud-native infrastructure / data residency.
- ADR-0140 — Cedar policy enforcement.
- ADR-DRIVE-0001 — Object-storage substrate selection (Garage replication topology).
- ADR-DRIVE-0006 — Immutability + WORM policy.
- `policy/tenant-scope.cedar`.
- `microservices/drive/multi-region.md`.
- `microservices/drive/threat-model.md`.
- `microservices/drive/compliance.md` per-pack overlays.
- GDPR Arts. 44–50; KR PIPA Art. 17; HIPAA 45 CFR §164.502; APPI Art. 24; PDPA §26; APP 8; DPDPA §16; LGPD Art. 33; UAE PDPL Art. 22; KSA PDPL Art. 29.
- Schrems II (CJEU C-311/18).
