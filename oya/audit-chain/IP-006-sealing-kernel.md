---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-006: Sealing kernel ports and key epochs

acceptance_lanes: [cargo-check, port-location, hsm-key-model, no-io-in-kernel]

## §A Problem
The sealing kernel is where audit-chain's non-repudiation promise becomes a type contract. The previous short IP did not name the pack epoch, signer, root publisher, or key resolver invariants needed to make HSM-backed Ed25519 verifiable after rotation.

## §B Approach
Define pure port traits and data structures for Merkle sealing: `MerkleEngine`, `SignerPort`, `RootPublisher`, `ObjectStoreWriter`, `IndexWriter`, `SealRecord`, `SigningKeyRef`, and `PackEpoch`. The kernel cannot import HSM, S3, Postgres, Mimir, or GitHub clients; it only declares inputs/outputs and error states.

## §C Deliverables
- `crates/oya-audit-chain-sealing-kernel/src/lib.rs` with zero-I/O sealing types.
- `PackEpoch` maps pack id, tenant partition, period range, active key id, retiring key id, and overlap window.
- `SealRecord` carries period id, leaf count, Merkle root, prior root, signature ref, root-publication refs, and status.
- `SignerPort` accepts root bytes and returns key id plus signature; it never exposes private key material.
- `RootPublisher` returns three-channel publication refs for WORM, Mimir, and GitHub-pinned manifests.

## §D Implementation Steps
1. Extract naming from `PRD.md` bounded-context table and `policy/seal-integrity.md` SI-06 through SI-13.
2. Make period and pack ids explicit newtypes to avoid cross-pack signing mistakes.
3. Represent overlap rotation as data, not comments, so verification can reject key epoch mismatch.
4. Keep all adapters behind traits and ensure kernel has no transport imports.
5. Add tests for invalid empty pack, empty tenant partition, and impossible rotation windows.
6. Wire catalog entries for `oya-audit-chain-sealing-kernel.yaml` to the crate path.

## §E Acceptance
- `cargo check -p oya-audit-chain-sealing-kernel` passes once created.
- `rg 'pkcs|sqlx|s3|mimir|git2|reqwest' crates/oya-audit-chain-sealing-kernel/src` returns no matches.
- `oya gate validate port-location --microservice audit-chain` identifies sealing ports in kernel only.

## §F Evidence
- `microservices/audit-chain/policy/seal-integrity.md` SI-06 through SI-13.
- `microservices/audit-chain/PRD.md` sealing bounded context and FR-02/FR-03/FR-09.
- `microservices/audit-chain/catalog/oya-audit-chain-sealing-kernel.yaml`.

## §G Counterparts
CloudTrail exposes digest validation after delivery, while Google and Microsoft rely on provider-controlled audit stores. This IP defines the Oyatie kernel contract for HSM-rooted Ed25519 and GitHub-pinned root publication so verification does not depend on trusting the provider store.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `sealing kernel`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `sealing kernel` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-006-sealing-kernel.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
