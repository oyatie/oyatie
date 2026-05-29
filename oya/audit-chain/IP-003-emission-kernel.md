---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-003: Emission kernel types and ports

acceptance_lanes: [cargo-check, cargo-test, data-class, port-location, no-io-in-kernel]

## §A Problem
The kernel must define the stable audit event contract used by every microservice without importing adapters or transport code. The existing Rust crate already has concrete substance in `AuditEvent`, `TenantShardId`, `AuditChain`, Ed25519 key types, and data-class fields; the IP needs to name those objects rather than describing a generic kernel layer.

## §B Approach
Keep `crates/oya-audit-chain-domain/src/lib.rs` as the current kernel-compatible source until the flat `oya-audit-chain-emission-kernel` crate exists. The extracted kernel must preserve append-only sequence/hash behavior, tenant shard validation, `Plane`, `Purpose`, `DataClass`, and explicit `AuditChainError` variants while moving I/O to adapters.

## §C Deliverables
- `crates/oya-audit-chain-domain/src/lib.rs` remains the source behavior reference for `AuditEvent`, `AuditChain`, `TenantShardId`, `MerkleRoot`, and Ed25519 structs.
- New `crates/oya-audit-chain-emission-kernel/src/lib.rs` rehomes pure types and traits only.
- Port traits: `AuditEmitter`, `WalWriter`, and `PrincipalResolver` with no Postgres, S3, HTTP, or eventing imports.
- Compatibility re-export from the old domain crate until downstream crates move.

## §D Implementation Steps
1. Inventory public types and errors in `crates/oya-audit-chain-domain/src/lib.rs` before extraction.
2. Move only type definitions, validation, and sealed port traits to the kernel crate.
3. Keep signature generation and Merkle math out of emission-kernel except references to opaque root/signature structs.
4. Preserve data-class comments or attributes on every field.
5. Add tests for empty tenant id, tenant-shard mismatch, empty signing key id, and duplicate trusted key rejection.
6. Run dependency-direction gates to prove the kernel has no adapter/eventing imports.

## §E Acceptance
- `cargo test -p oya-audit-chain-domain` passes before and after the compatibility re-export.
- `cargo check -p oya-audit-chain-emission-kernel` passes once the new crate exists.
- `rg 'std::fs|reqwest|sqlx|tokio' crates/oya-audit-chain-emission-kernel/src` returns no matches.

## §F Evidence
- `crates/oya-audit-chain-domain/src/lib.rs` current type and error surface.
- `microservices/audit-chain/PRD.md` FR-01 and bounded-context table.
- `crates/oya-shared-audit-chain-client-kernel/src/lib.rs` cross-service trait precedent.

## §G Counterparts
AWS CloudTrail SDKs expose a stable event contract for producers, while Google Cloud Audit Logs and Microsoft Purview Audit normalize activity records behind APIs. This IP closes that contract layer for Oyatie and keeps the GitHub-pinned proof channel as a downstream seal concern, not a producer responsibility.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `emission kernel`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `emission kernel` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-003-emission-kernel.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-003-emission-kernel.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
