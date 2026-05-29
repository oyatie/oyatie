---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-010: Sealing worker, usecase, API, adapter, and app

acceptance_lanes: [cargo-nextest, worker-cycle-test, hsm-outage-test, root-publication-test]

## §A Problem
The sealing worker converts accepted events into published proof. A stamped plan that only says `worker daemon` misses the operational contract: period closure, unsealed backlog, HSM unreachable behavior, root publication, and recursive audit of seal minting.

## §B Approach
Implement a period-driven worker that reads accepted event batches, builds a Merkle root, signs with the HSM adapter, writes Postgres/S3 records, publishes roots to Mimir and GitHub-pinned manifests, and emits `SealMinted`. It must be idempotent by `(pack, tenant_partition, period_id)` and pause rather than forge partial seals.

## §C Deliverables
- `crates/oya-audit-chain-sealing-usecase` period sealing orchestrator.
- `crates/oya-audit-chain-sealing-worker` timer/queue loop with bounded concurrency by pack and tenant partition.
- `crates/oya-audit-chain-sealing-api` typed command/result structures for manual replay and worker cycle calls.
- `crates/oya-audit-chain-sealing-app` composition root with HSM, Postgres, S3, Mimir, and GitHub publication adapters.
- Runbook integration with `runbooks/merkle-seal-recovery.md` and `runbooks/signature-verification-failure.md`.

## §D Implementation Steps
1. Select only periods whose wall-clock has elapsed and whose accepted events are durable.
2. Build leaf hashes from canonical event envelopes and preserve event order or explicit sorted order consistently with IP-007.
3. Call `SignerPort`; on HSM outage keep events unsealed and update `audit_chain_unsealed_buffer_depth_seconds`.
4. Write WORM blob before marking Postgres period sealed; publish roots to Mimir and GitHub-pinned manifest after durable writes.
5. Emit `SealMinted` and recursive audit event for worker action.
6. Make replay idempotent: a duplicate worker cycle returns the existing seal when root, key, and leaf count match.

## §E Acceptance
- Worker tests cover empty period, single-leaf period, odd-leaf period, HSM outage, WORM write failure, and duplicate replay.
- `dashboards/seal-latency.json` has a panel for unsealed backlog and root-publication lag.
- `cargo test -p oya-audit-chain-domain merkle` remains green.

## §F Evidence
- `microservices/audit-chain/PRD.md` FR-02, FR-03, FR-09 and availability degraded-mode wording.
- `microservices/audit-chain/policy/seal-integrity.md` SI-02 through SI-04.
- `microservices/audit-chain/runbooks/merkle-seal-recovery.md`.

## §G Counterparts
AWS CloudTrail digest publication can lag by minutes and remains AWS-mediated; Microsoft Purview Audit documents 60-90 minute availability for core events. Oyatie's worker closes a sharper gap: one-second target seal cycles with GitHub-visible root publication and explicit degraded unsealed state.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `sealing worker app`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `sealing worker app` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-010-sealing-worker-app.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
