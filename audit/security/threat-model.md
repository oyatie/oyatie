---
doc_class: ThreatModel
microservice: audit-chain
version: 1.0.0
status: Proposed
date: 2026-05-20
owner: axis-audit-chain + council-security
related_oyatie_adrs:
  - ADR-0003
  - ADR-0009
  - ADR-0145
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0297
  - ADR-0313
  - ADR-0319
---

# Audit-Chain Security Threat Model

This document covers the audit-chain substrate security posture for event
acceptance, append-only storage, hash chain construction, Merkle root minting,
HSM signing, WORM storage, verification, query, retention, replication, and
cross-service emission confirmation. Audit-chain is the evidence backbone for
Oyatie: if this substrate can be forged, backdated, erased, or made ambiguous,
every compliance and security claim that depends on sealed evidence becomes
unreliable.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| AUD-A01 | AuditEventPayload | Canonical serialized event body from source microservice. | WAL/Postgres and WORM blob | Preserve integrity and source attribution. |
| AUD-A02 | AuditEventEnvelope | event_id, tenant_id, source_microservice, trace_id, span_id, audit_id, schema_version, cell_id. | WAL/Postgres | Enforce ADR-0263 event contract. |
| AUD-A03 | AppendOnlyWalRecord | Durable acceptance order, idempotency key, event hash, predecessor pointer. | Postgres WAL table | Prevent retroactive insert and deletion. |
| AUD-A04 | HashChainLink | Per-tenant/per-cell previous hash, current hash, sequence number. | Postgres and seal worker state | Prevent hash-chain break. |
| AUD-A05 | MerkleTreeLeaf | Event hash leaf, tenant partition, period, sequence range. | WORM object and generated proofs | Preserve inclusion proof. |
| AUD-A06 | MerkleRootSeal | Root hash, period id, tenant partition, prior root, signature, signer key id. | Postgres, WORM object, root publication | Prevent Merkle-root forge. |
| AUD-A07 | HsmSigningKeyHandle | HSM key handle, public key fingerprint, epoch, rotation state. | HSM and key resolver | Prevent signing-key compromise. |
| AUD-A08 | VerificationProof | Inclusion proof, root id, public key epoch, verification result. | Verification API and cache | Prevent false verification. |
| AUD-A09 | RootPublicationRecord | Mimir metric, Git-pinned manifest, public root digest, publish time. | Mimir and evidence manifest | Detect root divergence. |
| AUD-A10 | RetentionCascadeRecord | DSR or retention action, reason, policy id, affected event references. | Retention store and WORM marker | Prevent unauthorized erasure. |
| AUD-A11 | ReplicationCursor | Multi-region cursor, source/destination cell, root epoch, lag, checksum. | Replication state store | Prevent multi-region replication tampering. |
| AUD-A12 | QueryAccessGrant | Tenant/auditor access token, scope, expiry, export bundle. | OpenBao and query DB | Prevent evidence disclosure. |
| AUD-A13 | CedarPolicyDecision | Emit, query, verify, retention, and auditor-scope policy outcome. | Policy decision log | Prevent unauthorized audit access. |
| AUD-A14 | AuditEmissionEnvelope | Audit-chain's own ADR-0263 telemetry and self-audit events. | audit-chain | Preserve self-monitoring. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| AUD-I01 | Emit API | `../contracts/openapi/audit-chain.yaml` | Internal workload service | Accepts signed or mTLS-authenticated events. |
| AUD-I02 | Seal Worker | `../IP-010-sealing-worker-app.md` | Audit-chain worker | Builds Merkle roots and signs via HSM. |
| AUD-I03 | Verification API | `../IP-011-verification-stack.md` | Tenant, auditor, internal service | Verifies inclusion proofs and roots. |
| AUD-I04 | Query API | `../IP-012-query-stack.md` | Tenant operator or auditor | Reads scoped audit events and exports evidence. |
| AUD-I05 | Retention Cascade API | `../IP-013-retention-cascade.md` | Privacy/legal workflows | Applies retention and DSR handling. |
| AUD-I06 | HSM Signing Interface | `../IP-008-sealing-adapter-hsm.md` | Sealing worker | Signs root with key handle. |
| AUD-I07 | WORM Storage Interface | `../IP-009-sealing-adapter-postgres-s3.md` | Sealing worker | Stores immutable root and tree data. |
| AUD-I08 | Root Publication Channel | `../dashboards/verification-failure-rate.json` | Audit-chain and observability | Publishes root metrics and divergence alerts. |
| AUD-I09 | Cross-Service Adapter | `../IP-014-cross-microservice-emission-adapter.md` | Workload services | Normalizes event emission into audit-chain. |
| AUD-I10 | Audit Event Bridge | `../contracts/asyncapi/audit-events.yaml` | Audit-chain service | Emits AuditEmitted, SealMinted, VerificationFailed, RetentionApplied, KeyRotated. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| AUD-D01 | Postgres | Durable WAL, event index, roots, query store | Retroactive insert or loss | Append-only policy and verification. |
| AUD-D02 | WORM object storage | Immutable tree blobs and raw evidence | Tamper or loss of proof material | `../runbooks/merkle-seal-recovery.md`. |
| AUD-D03 | HSM/KMS | Root signing | Merkle-root forge if compromised | `../runbooks/hsm-key-rotation.md`. |
| AUD-D04 | Cedar policy-engine | Emit/query/retention/auditor scope | Unauthorized evidence access | `../policy/auditor-scope.cedar`. |
| AUD-D05 | SPIFFE/mTLS | Workload identity for emitters and workers | Spoofed emitter or worker | ADR-0145. |
| AUD-D06 | Observability/Mimir | Root publication and alerting | Missed divergence | `../dashboards/verification-failure-rate.json`. |
| AUD-D07 | OpenBao | Auditor tokens and retention approvals | Evidence disclosure or unauthorized action | Auditor-scope policy. |
| AUD-D08 | Multi-region replication substrate | Region/cell replication | Replication tampering or lag | `../multi-region.md`. |
| AUD-D09 | Source microservices | Event payload correctness | Garbage-in, sealed garbage | Content validator and source policy. |
| AUD-D10 | Git/evidence manifest | Root pinning | Root publication ambiguity | Git-pinned manifest discipline. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| AUD-B01 | Workload emission boundary | Any microservice | Audit-chain emit API | Spoofed event, wrong tenant, unregistered class. |
| AUD-B02 | Tenant boundary | Tenant A event/query | Tenant B event/query | Cross-tenant evidence disclosure. |
| AUD-B03 | Cell boundary | Source cell emitter | Home audit partition | Per-cell tamper and sequence divergence. |
| AUD-B04 | WAL boundary | Emit API | Append-only WAL/Postgres | Retroactive insert, deletion, or reorder. |
| AUD-B05 | Hash-chain boundary | WAL sequence | Hash chain and leaf builder | Hash-chain break. |
| AUD-B06 | Merkle boundary | Leaf set | Merkle root and proof generator | Merkle-root forge or omitted leaf. |
| AUD-B07 | HSM signing boundary | Sealing worker | HSM signing key handle | Unauthorized root signature. |
| AUD-B08 | WORM storage boundary | Sealing worker | S3/Object Lock/WORM bucket | Tree or root overwrite/delete. |
| AUD-B09 | Root publication boundary | Sealed root | Mimir/Git-pinned publication | Divergent roots or missing publication. |
| AUD-B10 | Verification boundary | Tenant/auditor request | Verification API | False positive verification. |
| AUD-B11 | Query/export boundary | Tenant/auditor | Scoped event query/export | Evidence disclosure or overbroad export. |
| AUD-B12 | Retention boundary | DSR/retention workflow | Retention cascade engine | Unauthorized redaction or retention bypass. |
| AUD-B13 | Multi-region replication boundary | Primary region/cell | Replica region/cell | Replication tampering and forked roots. |
| AUD-B14 | Self-audit boundary | Audit-chain state change | Audit-chain self events | Missing self-evidence. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-S01 | AuditEventEnvelope | AUD-B01 | Emitter spoofs source_microservice or tenant_id. | False evidence attributed to another service or tenant. |
| AUD-S02 | QueryAccessGrant | AUD-B11 | Attacker spoofs auditor or tenant operator. | Evidence disclosure. |
| AUD-S03 | HsmSigningKeyHandle | AUD-B07 | Malicious worker spoofs sealing identity to request signature. | Merkle-root forge. |
| AUD-S04 | ReplicationCursor | AUD-B13 | Replica accepts spoofed primary cursor. | Multi-region replication tampering. |
| AUD-S05 | RootPublicationRecord | AUD-B09 | Attacker publishes fake root metric or manifest. | Verification confusion. |
| AUD-S06 | VerificationProof | AUD-B10 | Client supplies forged proof or stale public key epoch. | False verification. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-T01 | AppendOnlyWalRecord | AUD-B04 | Retroactive insert with old timestamp or sequence. | Backdated evidence. |
| AUD-T02 | AppendOnlyWalRecord | AUD-B04 | Event deletion or reorder in WAL/index. | Broken audit history. |
| AUD-T03 | HashChainLink | AUD-B05 | Hash-chain predecessor pointer changed. | Hash-chain break. |
| AUD-T04 | MerkleRootSeal | AUD-B06 | Root generated over incomplete or altered leaf set. | Merkle-root forge. |
| AUD-T05 | HsmSigningKeyHandle | AUD-B07 | Key epoch or key handle changed during seal. | Invalid or attacker-controlled root signature. |
| AUD-T06 | WORM object | AUD-B08 | Root/tree object overwritten or lifecycle policy weakened. | Proof material tamper. |
| AUD-T07 | ReplicationCursor | AUD-B13 | Replica receives forked root or altered cursor. | Multi-region tampering. |
| AUD-T08 | RetentionCascadeRecord | AUD-B12 | Retention action redacts protected event. | Evidence erasure. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-R01 | AuditEventPayload | AUD-B01 | Source service denies emitting event. | Source accountability gap. |
| AUD-R02 | MerkleRootSeal | AUD-B06 | Sealing worker denies producing root. | Root provenance gap. |
| AUD-R03 | VerificationProof | AUD-B10 | Auditor denies verification result or export contents. | Evidence challenge. |
| AUD-R04 | RetentionCascadeRecord | AUD-B12 | Privacy/legal actor denies redaction request. | Compliance dispute. |
| AUD-R05 | HsmSigningKeyHandle | AUD-B07 | Operator denies key rotation or emergency disable. | Key custody gap. |
| AUD-R06 | ReplicationCursor | AUD-B13 | Region owner denies replication lag or fork. | DR evidence gap. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-I01 | AuditEventPayload | AUD-B11 | Tenant or auditor export includes another tenant's events. | Cross-tenant evidence disclosure. |
| AUD-I02 | AuditEventPayload | AUD-B14 | Audit-chain self telemetry logs raw source payload PII. | Observability privacy breach. |
| AUD-I03 | QueryAccessGrant | AUD-B11 | Auditor token leaks or is over-scoped. | Evidence disclosure. |
| AUD-I04 | HsmSigningKeyHandle | AUD-B07 | Key handle or admin credential leaks. | Signature abuse. |
| AUD-I05 | RetentionCascadeRecord | AUD-B12 | DSR subject hash or legal details exposed. | Privacy breach. |
| AUD-I06 | RootPublicationRecord | AUD-B09 | Publication reveals sensitive tenant/cell topology. | Targeting aid. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-DOS01 | Emit API | AUD-B01 | Emission flood exhausts WAL, validators, or queue. | Missing audit evidence across services. |
| AUD-DOS02 | Sealing worker | AUD-B06 | Large period or malformed event causes seal latency. | Delayed roots. |
| AUD-DOS03 | HsmSigningKeyHandle | AUD-B07 | HSM outage blocks root signing. | Unsealed periods. |
| AUD-DOS04 | WORM object | AUD-B08 | Object store unavailable. | Root/proof storage outage. |
| AUD-DOS05 | VerificationProof | AUD-B10 | Expensive verification/export query flood. | Auditor and tenant verification outage. |
| AUD-DOS06 | ReplicationCursor | AUD-B13 | Replication lag or retry storm. | DR/root divergence. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| AUD-E01 | QueryAccessGrant | AUD-B11 | Tenant reader escalates to auditor or cross-tenant export. | Evidence disclosure. |
| AUD-E02 | RetentionCascadeRecord | AUD-B12 | Non-privacy role triggers redaction or retention override. | Evidence deletion. |
| AUD-E03 | HsmSigningKeyHandle | AUD-B07 | Worker gains HSM admin or rotation authority. | Root signing compromise. |
| AUD-E04 | CedarPolicyDecision | AUD-B01 | Emitter bypasses class registration or tenant policy. | Unvalidated evidence accepted. |
| AUD-E05 | RootPublicationRecord | AUD-B09 | Observability writer overrides root publication. | Tamper signal suppression. |
| AUD-E06 | ReplicationCursor | AUD-B13 | Replica operator promotes forked root as canonical. | Multi-region evidence fork. |

## DREAD Scoring

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | AUD-T04 | Merkle root forged over incomplete or altered leaves. | 10 | 7 | 7 | 10 | 8 | 42 |
| 2 | AUD-T01 | Retroactive insert into append-only WAL. | 10 | 8 | 7 | 9 | 8 | 42 |
| 3 | AUD-T03 | Hash-chain break hidden from verifier. | 10 | 7 | 7 | 10 | 7 | 41 |
| 4 | AUD-S03 | Spoofed sealing worker obtains HSM signature. | 10 | 6 | 7 | 10 | 7 | 40 |
| 5 | AUD-T07 | Multi-region replication tampering. | 9 | 7 | 7 | 9 | 7 | 39 |
| 6 | AUD-E03 | Worker gains HSM admin or rotation authority. | 10 | 6 | 6 | 10 | 6 | 38 |
| 7 | AUD-I01 | Cross-tenant audit export disclosure. | 9 | 7 | 6 | 8 | 7 | 37 |
| 8 | AUD-T08 | Unauthorized retention redaction. | 9 | 6 | 6 | 9 | 7 | 37 |
| 9 | AUD-DOS01 | Emission flood causes missing evidence. | 9 | 9 | 8 | 8 | 3 | 37 |
| 10 | AUD-DOS03 | HSM outage blocks signing. | 9 | 7 | 5 | 10 | 5 | 36 |
| 11 | AUD-S01 | Emitter spoofs tenant or source service. | 8 | 7 | 7 | 8 | 6 | 36 |
| 12 | AUD-S05 | Fake root publication. | 8 | 7 | 6 | 8 | 6 | 35 |
| 13 | AUD-E01 | Tenant reader escalates to auditor export. | 8 | 6 | 6 | 8 | 6 | 34 |
| 14 | AUD-DOS05 | Verification/export query flood. | 7 | 8 | 7 | 7 | 4 | 33 |
| 15 | AUD-I02 | Self telemetry leaks raw payload PII. | 8 | 6 | 5 | 8 | 5 | 32 |

## Attack Trees

### Opportunistic Adversary: Emit Flood to Evidence Gap

- Goal: create a period where important events are delayed or missing.
  - Path O1: compromise or abuse a noisy service endpoint.
  - Path O2: emit high-cardinality or oversized events.
  - Path O3: exhaust validators, WAL writers, or storage.
  - Path O4: force sealing worker behind period deadline.
  - Path O5: exploit delayed root publication during incident.
- Required break: per-source and per-tenant rate limits are absent or too high.
- Required break: audit-chain SLOs do not page on emission backlog.
- Detection pivot: `AuditEmitted`, `AbuseDefenceQuotaExceeded`, seal latency dashboard.

### Targeted Adversary: Retroactive Insert and Root Forge

- Goal: add backdated event and produce credible proof.
  - Path T1: gain write access to WAL/index or DB admin role.
  - Path T2: insert event with old accepted_at or sequence.
  - Path T3: alter hash predecessor pointers.
  - Path T4: recalculate Merkle root and attempt HSM signature.
  - Path T5: publish forged root in observability or manifest.
- Required break: append-only controls fail.
- Required break: hash-chain and prior-root checks fail.
- Required break: HSM signs untrusted root or root publication lacks cross-check.
- Detection pivot: `VerificationFailed`, `SealMinted`, root-publication divergence.

### Insider Adversary: Retention Abuse

- Goal: delete or redact damaging audit evidence.
  - Path I1: obtain privacy/legal workflow role.
  - Path I2: create DSR or retention cascade with overbroad selector.
  - Path I3: apply redaction to regulated or legal-hold records.
  - Path I4: suppress retention-applied event.
  - Path I5: export altered proof set.
- Required break: retention Cedar policy allows non-owner or broad scope.
- Required break: retention cascade does not seal before/after state.
- Detection pivot: `RetentionApplied`, `OfficeBoundaryClearanceRequested`, `OfficeBoundaryAttemptDenied`.

### Nation-State Adversary: Multi-Region Fork

- Goal: split canonical evidence between primary and replica regions.
  - Path N1: compromise replication channel or replica operator.
  - Path N2: alter replication cursor or root epoch.
  - Path N3: allow replica to mint or serve forked proof.
  - Path N4: exploit region failover to promote forked root.
  - Path N5: use ambiguity to challenge evidence in regulated incident.
- Required break: replica root comparison and prior-root continuity fail.
- Required break: failover promotion lacks independent root verification.
- Detection pivot: replication checksum alert, `VerificationFailed`, and multi-region dashboard.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| AUD-S01 | mTLS/SPIFFE emitter identity and tenant policy. | ADR-0145, ADR-0244 | `../IP-014-cross-microservice-emission-adapter.md`; `../policy/tenant-scope.cedar`. |
| AUD-T01 | Append-only WAL with accepted_at and sequence discipline. | ADR-0003 | `../IP-003-emission-kernel.md`; `../IP-004-emission-domain.md`. |
| AUD-T03 | Hash-chain predecessor validation and verification stack. | ADR-0003 | `../IP-011-verification-stack.md`. |
| AUD-T04 | Merkle tree seal generation with prior-root continuity. | ADR-0003 | `../IP-007-sealing-domain-merkle.md`; `../runbooks/merkle-root-discrepancy-investigation.md`. |
| AUD-S03 | HSM signing adapter and key rotation protocol. | ADR-0243 | `../IP-008-sealing-adapter-hsm.md`; `../runbooks/hsm-key-rotation.md`. |
| AUD-T06 | WORM storage and seal recovery runbook. | ADR-0009 | `../IP-009-sealing-adapter-postgres-s3.md`; `../runbooks/merkle-seal-recovery.md`. |
| AUD-I01 | Auditor scope Cedar policy and export scoping. | ADR-0243 | `../policy/auditor-scope.cedar`; `../runbooks/audit-export.md`. |
| AUD-T08 | Retention cascade requires policy and sealed event. | ADR-0243, ADR-0263 | `../IP-013-retention-cascade.md`; `../runbooks/retention-cascade.md`. |
| AUD-T07 | Multi-region cursor and checksum verification. | ADR-0009 | `../multi-region.md`; `../runbooks/chain-replay-from-snapshot-protocol.md`. |
| AUD-S05 | Root publication cross-check between WORM, Mimir, and manifest. | ADR-0263 | `../dashboards/verification-failure-rate.json`. |
| AUD-DOS03 | HSM rotation and degraded signing response. | ADR-0243 | `../runbooks/hsm-key-rotation.md`. |
| AUD-DOS05 | Verification and export rate limiting. | ADR-0297 | `../policy/public-read.cedar`; `../policy/auditor-scope.cedar`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| AUD-RR01 | Source microservice can emit semantically false but structurally valid event. | source service owner | Source event validators and downstream evidence review. | New event class. |
| AUD-RR02 | HSM outage can delay root signing. | axis-audit-chain | Seal backlog paging and recovery runbook. | HSM SLO burn. |
| AUD-RR03 | WORM retention can preserve incorrectly emitted data longer than desired. | council-privacy | Retention cascade with sealed redaction markers. | DSR request. |
| AUD-RR04 | Multi-region replication lag can temporarily delay verification in remote region. | ops-sre-reliability | Lag dashboard and failover gate. | Region incident. |
| AUD-RR05 | Auditor exports intentionally disclose scoped evidence. | ops-compliance | JIT token, narrow scope, watermark, and export audit trail. | Auditor access request. |
| AUD-RR06 | Root publication metrics can expose coarse tenant/cell activity. | axis-observability | Aggregated labels and PII-free publication. | Dashboard change. |
| AUD-RR07 | Cryptographic library CVE could invalidate confidence in prior roots. | council-security | Key rotation, replay verification, and external review. | Crypto CVE. |
| AUD-RR08 | Large legal or incident replay may stress query/verification path. | axis-audit-chain | Replay-from-snapshot protocol and rate controls. | Replay request. |
| AUD-RR09 | Retention exceptions vary by jurisdiction and pack. | ops-legal | Pack-specific retention policy and legal review. | Pack activation. |
| AUD-RR10 | Git-pinned manifest is only as strong as repository governance. | governance owner | Signed commits and independent WORM copy. | Governance incident. |

## Specific Telemetry for Detection

ADR-0263 detection telemetry must include `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` for state-changing
audit-chain events. Cedar denial events include policy id, principal, action,
resource, decision, and denied reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| AUD-S01 | Source service mismatch, tenant mismatch, unregistered event class. | `AuditEmitted`, `AbuseDefenceSpoofDetected` | Spoofed emitter. |
| AUD-T01 | accepted_at older than current sequence, predecessor mismatch. | `VerificationFailed` | Retroactive insert. |
| AUD-T03 | Hash-chain predecessor mismatch. | `VerificationFailed` | Hash-chain break. |
| AUD-T04 | Root mismatch between leaf set and signed root. | `SealMinted`, `VerificationFailed` | Merkle-root forge. |
| AUD-S03 | HSM sign request from unexpected worker identity. | `KeyRotated`, `AbuseDefenceAttestationFailed` | Signing boundary attack. |
| AUD-T06 | WORM overwrite or retention policy change. | `VerificationFailed`, object store alert | WORM tamper. |
| AUD-T07 | Replication checksum mismatch or lag threshold. | replication cursor alert, `VerificationFailed` | Multi-region replication tamper. |
| AUD-T08 | Retention cascade over protected class or legal hold. | `RetentionApplied`, `OfficeBoundaryAttemptDenied` | Unauthorized redaction. |
| AUD-I01 | Query/export includes foreign tenant. | `OfficeBoundaryAttemptDenied`, `ConglomeratePersonalTenantBoundaryRefused` | Cross-tenant evidence disclosure. |
| AUD-DOS01 | Emit backlog, validator failure, rate-limit hit. | `AbuseDefenceQuotaExceeded`, `AuditEmitted` lag | Emission DoS. |
| AUD-DOS03 | HSM latency, signing error, key handle unavailable. | `AbuseDefenceVendorOutage`, `KeyRotated` | Signing outage. |
| AUD-S05 | Root metric differs from WORM or manifest. | root publication alert, `VerificationFailed` | Fake root publication. |

## Threat Coverage Ledger

### AUD-COV01: Emit authenticity coverage

- Threats covered: AUD-S01, AUD-E04, AUD-R01.
- Asset coverage: AuditEventEnvelope, AuditEventPayload, CedarPolicyDecision.
- Boundary coverage: AUD-B01 and AUD-B02.
- Required control evidence: mTLS/SPIFFE, tenant scope, registered event class, source_microservice validation.
- Detection evidence: `AuditEmitted`, `AbuseDefenceSpoofDetected`, and class validator failure.

### AUD-COV02: Retroactive insert coverage

- Threats covered: AUD-T01, AUD-T02.
- Asset coverage: AppendOnlyWalRecord and HashChainLink.
- Boundary coverage: AUD-B04 and AUD-B05.
- Required control evidence: accepted_at sequence, predecessor hash, append-only DB policy.
- Detection evidence: `VerificationFailed` and hash-chain continuity alert.

### AUD-COV03: Hash-chain coverage

- Threats covered: AUD-T03 and AUD-R02.
- Asset coverage: HashChainLink and MerkleTreeLeaf.
- Boundary coverage: AUD-B05.
- Required control evidence: previous hash validation, sequence range check, verifier recompute.
- Detection evidence: `VerificationFailed` and seal latency anomaly.

### AUD-COV04: Merkle root coverage

- Threats covered: AUD-T04, AUD-S06.
- Asset coverage: MerkleRootSeal, MerkleTreeLeaf, VerificationProof.
- Boundary coverage: AUD-B06 and AUD-B10.
- Required control evidence: root recomputation, prior-root continuity, inclusion proof validation.
- Detection evidence: `SealMinted`, `VerificationFailed`, and merkle-root discrepancy runbook.

### AUD-COV05: HSM signing coverage

- Threats covered: AUD-S03, AUD-T05, AUD-E03, AUD-DOS03.
- Asset coverage: HsmSigningKeyHandle.
- Boundary coverage: AUD-B07.
- Required control evidence: signer identity, key epoch, key rotation event, HSM admin separation.
- Detection evidence: `KeyRotated`, HSM access audit, and `AbuseDefenceAttestationFailed`.

### AUD-COV06: WORM storage coverage

- Threats covered: AUD-T06 and AUD-DOS04.
- Asset coverage: WORM object, MerkleRootSeal, VerificationProof.
- Boundary coverage: AUD-B08.
- Required control evidence: Object Lock/WORM mode, immutable root copy, recovery playbook.
- Detection evidence: object store alert and `VerificationFailed`.

### AUD-COV07: Root publication coverage

- Threats covered: AUD-S05, AUD-E05.
- Asset coverage: RootPublicationRecord.
- Boundary coverage: AUD-B09.
- Required control evidence: Mimir/WORM/Git cross-check, limited publisher identity, publication SLO.
- Detection evidence: root divergence dashboard and `VerificationFailed`.

### AUD-COV08: Retention coverage

- Threats covered: AUD-T08, AUD-E02, AUD-R04.
- Asset coverage: RetentionCascadeRecord.
- Boundary coverage: AUD-B12.
- Required control evidence: privacy/legal role, policy id, before/after seal, legal-hold deny.
- Detection evidence: `RetentionApplied`, `OfficeBoundaryClearanceRequested`, and retention cascade runbook.

### AUD-COV09: Multi-region coverage

- Threats covered: AUD-S04, AUD-T07, AUD-E06, AUD-DOS06.
- Asset coverage: ReplicationCursor and RootPublicationRecord.
- Boundary coverage: AUD-B13.
- Required control evidence: cursor signature, checksum, prior-root continuity, failover gate.
- Detection evidence: replication lag alert and `VerificationFailed`.

### AUD-COV10: Query/export coverage

- Threats covered: AUD-I01, AUD-I03, AUD-E01, AUD-DOS05.
- Asset coverage: QueryAccessGrant and VerificationProof.
- Boundary coverage: AUD-B10 and AUD-B11.
- Required control evidence: auditor-scope Cedar, JIT token, export watermark, rate limits.
- Detection evidence: audit export event, `OfficeBoundaryAttemptDenied`, and query SLO burn.

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| Audit-chain restart | `../runbooks/audit-chain-restart.md` |
| Audit export issue | `../runbooks/audit-export.md` |
| HSM key rotation | `../runbooks/hsm-key-rotation.md` |
| Merkle root discrepancy | `../runbooks/merkle-root-discrepancy-investigation.md` |
| Merkle seal recovery | `../runbooks/merkle-seal-recovery.md` |
| Signature verification failure | `../runbooks/signature-verification-failure.md` |
| Regulator evidence export failure | `../runbooks/regulator-evidence-export-failure.md` |
| Retention cascade | `../runbooks/retention-cascade.md` |
| Chain replay from snapshot | `../runbooks/chain-replay-from-snapshot-protocol.md` |

## Cross-References

- Root service architecture: `../ARCHITECTURE.md`.
- Product requirements: `../PRD.md`.
- Audit events contract: `../contracts/asyncapi/audit-events.yaml`.
- Audit-chain OpenAPI contract: `../contracts/openapi/audit-chain.yaml`.
- Audit-chain proto contract: `../contracts/proto/audit-chain.proto`.
- Emission kernel: `../IP-003-emission-kernel.md`.
- Emission domain: `../IP-004-emission-domain.md`.
- Emission usecase and adapter: `../IP-005-emission-usecase-and-adapter.md`.
- Sealing kernel: `../IP-006-sealing-kernel.md`.
- Sealing domain Merkle: `../IP-007-sealing-domain-merkle.md`.
- Sealing adapter HSM: `../IP-008-sealing-adapter-hsm.md`.
- Postgres/S3 sealing adapter: `../IP-009-sealing-adapter-postgres-s3.md`.
- Sealing worker app: `../IP-010-sealing-worker-app.md`.
- Verification stack: `../IP-011-verification-stack.md`.
- Query stack: `../IP-012-query-stack.md`.
- Retention cascade: `../IP-013-retention-cascade.md`.
- Cross-microservice emission adapter: `../IP-014-cross-microservice-emission-adapter.md`.
- Auditor scope policy: `../policy/auditor-scope.cedar`.
- Tenant scope policy: `../policy/tenant-scope.cedar`.
- Public read policy: `../policy/public-read.cedar`.
- Verification failure dashboard: `../dashboards/verification-failure-rate.json`.
- ADR-0263 observability emission contract: `../../../docs/decisions/ADR-0706-observability-live-apex.md`.
- ADR-0003 audit-chain and evidence emission: `../../../docs/decisions/ADR-0709-general-live-apex.md`.
- ADR-0243 Cedar as universal gate: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244 tenant as universal scoping primitive: `../../../docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0297 abuse defence baseline: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.

## Checkpoint Notes

- This document does not modify audit-chain decisions or runbooks.
- It treats audit-chain self-events as mandatory evidence for its own state changes.
- It assumes unregistered audit-event classes are refused before enforcement promotion.
- It accepts source-service semantic truth as upstream while enforcing structural integrity here.
