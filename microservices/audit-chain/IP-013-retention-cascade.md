---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-013: Retention cascade and redaction-proof handling

acceptance_lanes: [cargo-test, retention-policy-tests, dsr-cascade-tests, proof-preservation]

## §A Problem
Retention in audit-chain is harder than deleting rows: GDPR/KR/HIPAA windows must be honored while Merkle proofs remain explainable. The previous plan named retention-cascade but did not specify redaction tokens, DSR cascade sources, hard-delete grace, or how proof continuity survives redaction.

## §B Approach
Build retention as a worker/usecase around `RetentionPolicy`, `DsrCascade`, `RedactionToken`, and `RetentionRun`. The worker reads `policy/retention-matrix.yaml`, receives DSR cascades from tenancy/cloud-secrets, writes redaction markers that preserve proof of the original record's existence, and emits `RetentionApplied`.

## §C Deliverables
- `crates/oya-audit-chain-retention-cascade-kernel` policy and cascade ports.
- `crates/oya-audit-chain-retention-cascade-domain` retention-window and redaction-token rules.
- `crates/oya-audit-chain-retention-cascade-usecase` daily sweep and DSR cascade orchestration.
- `crates/oya-audit-chain-retention-cascade-worker` pack-scoped worker loop.
- `crates/oya-audit-chain-retention-cascade-adapter` retention-matrix and storage mutation adapters.

## §D Implementation Steps
1. Parse retention windows by pack and data class from `policy/retention-matrix.yaml`.
2. Reject any cascade that would shorten HIPAA/KR/SOC2 retention below policy.
3. For DSR erasure, write a redaction token and seal the redaction event before hiding payload material.
4. Keep Merkle proof material sufficient to prove the original event existed and was lawfully redacted.
5. Emit `RetentionApplied` and self-audit worker activity.
6. Add runbook hooks for `runbooks/retention-cascade.md` and regulator export edge cases.

## §E Acceptance
- Tests cover pack-us-healthcare six-year retention, pack-kr three-year retention, GDPR erasure token, expired DSR replay, and cross-pack denial.
- `runbooks/retention-cascade.md` resolves from the IP and names redaction-token recovery.
- Proof verification still returns a structured redacted verdict rather than broken chain.

## §F Evidence
- `microservices/audit-chain/policy/retention-matrix.yaml`.
- `microservices/audit-chain/runbooks/retention-cascade.md`.
- `microservices/audit-chain/PRD.md` FR-06 and FR-07.

## §G Counterparts
Microsoft Purview Audit Premium and AWS CloudTrail Lake expose long-retention policies; Google log buckets support configurable retention. This IP adds the Oyatie-specific DSR redaction proof so compliance deletion does not destroy GitHub-pinned chain verifiability.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `retention cascade`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `retention cascade` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-013-retention-cascade.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
