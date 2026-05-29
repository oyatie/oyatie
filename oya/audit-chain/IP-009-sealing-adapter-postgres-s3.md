---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-009: Sealing Postgres index and S3 WORM adapters

acceptance_lanes: [cargo-test, sql-migration-check, object-lock-check, replay-consistency]

## §A Problem
Sealing storage has two different jobs: queryable index records and immutable proof blobs. The prior IP compressed Postgres and S3 into a short file without specifying how root records stay append-only, how WORM blobs map to periods, or how replay detects divergence.

## §B Approach
Build separate adapters for the `IndexWriter` and `ObjectStoreWriter` traits. Postgres stores `SealRecord` metadata by pack, tenant partition, period, key id, and root hash. S3-compatible WORM storage stores raw period leaves, proof material, signed roots, and public-key epoch manifests. The file adapter's current `ChainDiverged` replay check remains the local fixture for append-only behavior.

## §C Deliverables
- `crates/oya-audit-chain-sealing-adapter-postgres` migrations for `audit_seal_records` and `audit_key_epochs`.
- `crates/oya-audit-chain-sealing-adapter-s3` object-key builder for `pack/tenant_partition/period/root.json` and proof blobs.
- Object Lock policy tests for compliance mode and retention per `policy/retention-matrix.yaml`.
- Replay fixture connecting `FileAuditLedger::append_chain` divergence behavior to production append-only rules.
- Catalog entries updated for adapter-postgres and adapter-s3.

## §D Implementation Steps
1. Design SQL rows as insert-only; do not provide update/delete repository methods.
2. Make object keys carry pack, tenant partition, period, and root hash to avoid cross-pack collision.
3. Persist prior-root hash with every seal record for chain walking.
4. Write failed-storage behavior: if WORM write succeeds but index write fails, recovery replays WORM blob into Postgres; if WORM fails, period remains unsealed.
5. Bind retention policy to pack and data class; reject shorter-than-policy lifecycle.
6. Add tests for duplicate period/root insert, mismatched prior root, and missing WORM blob.

## §E Acceptance
- `cargo test -p oya-audit-chain-file-adapter file_ledger` passes as local append-only guard.
- SQL migration linter rejects update/delete privileges for sealer role.
- Object key builder tests cover pack-kr and cross-pack forbidden cases.

## §F Evidence
- `crates/oya-audit-chain-file-adapter/src/lib.rs` replay and `ChainDiverged` logic.
- `microservices/audit-chain/policy/retention-matrix.yaml`.
- `microservices/audit-chain/policy/seal-integrity.md` SI-02 through SI-05.

## §G Counterparts
CloudTrail delivers digest files to S3 and supports integrity validation; Google routes audit logs to buckets, BigQuery, or Pub/Sub; Microsoft Purview exports audit records through APIs. This IP gives Oyatie equivalent storage/export surfaces plus per-event Merkle proof blobs and GitHub-pinned roots.

## Stop Conditions
Do not promote this IP on line count alone. Stop if a cited path is absent, a counterpart claim cannot be traced to `competitor-parity-matrix.md` or `feature-parity-matrix-2026-05-20.md`, or a verification command above cannot run in the current checkout.


## Wave 15 Detailed Reviewer Map

### Domain vocabulary that must appear in the implementation PR
- Pack-local chain: implementation must preserve `(pack, tenant_partition, period)` as a first-class tuple, not hide it inside a generic tenant string.
- Seal lifecycle: implementation must distinguish accepted, unsealed, sealed, published, verified, redacted, and retained states where this IP touches those transitions.
- Evidence linkage: every emitted or derived record must carry an audit id, period id, root or prior-root reference when applicable, and a provenance pointer to the producing service.
- Residency boundary: pack movement is forbidden unless the IP explicitly names a tenant-initiated export path and a receiving-tenant compliance basis.
- Key material boundary: public keys may be published; private keys, HSM handles, OpenBao leases, and provider credentials must stay out of serializable responses and logs.
- Audit-of-audit: mutating or privileged read behavior introduced by this IP must itself produce an audit event rather than relying on operator notes.

### File-reference checks before implementation starts
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `sealing adapter postgres s3`.
- Re-read `microservices/audit-chain/ARCHITECTURE.md` for layer placement, runtime assumptions, and cross-product import constraints.
- Re-read `microservices/audit-chain/manifest.json` for catalog, SLO, and contract pointers; do not invent a crate or contract absent from the manifest without updating the manifest in the same change.
- Re-read `microservices/audit-chain/policy/seal-integrity.md` when the IP touches roots, proofs, keys, HSM, publication, or verifier behavior.
- Re-read `microservices/audit-chain/competitor-parity-matrix.md` and `feature-parity-matrix-2026-05-20.md` before making any CloudTrail, Google Cloud Audit Logs, Microsoft Purview Audit, Splunk, Datadog, Vault, or GitHub comparison.
- Re-read the existing Rust crates under `crates/oya-audit-chain-*` and `crates/oya-shared-audit-chain-client-kernel` so the implementation extends live behavior instead of replacing it with a parallel scaffold.

### Negative tests or static checks expected
- Cross-tenant or cross-pack input is denied before storage or signing work begins.
- Duplicate idempotency material returns the prior result only when the canonical fingerprint matches exactly.
- Tampered proof, tampered signature, stale key epoch, or missing prior root returns a structured failure rather than a generic internal error.
- Missing GitHub-pinned root/key publication keeps the period below the claim boundary even if Postgres and WORM writes succeeded.
- A downstream outage pauses or degrades explicitly; it must not silently mark the audit action complete.
- High-cardinality fields such as tenant id and principal id are not exported as metrics labels.

### Counterpart comparison rows
| Counterpart | Relevant capability | Audit-chain requirement for this IP |
|---|---|---|
| AWS CloudTrail | Delivered audit records and integrity validation | Preserve immutable event/root evidence and make the trust boundary explicit. |
| Google Cloud Audit Logs | Admin/Data/System/Policy audit taxonomy and routed log sinks | Keep event classes and export routing typed; do not collapse policy-denied and data-access events. |
| Microsoft Purview Audit | Search/export, retention policies, and investigation workflows | Keep query/export/read paths scoped, retained, and auditor-engagement aware. |
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `sealing adapter postgres s3` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-009-sealing-adapter-postgres-s3.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
