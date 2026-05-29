---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-008: HSM signing adapter

acceptance_lanes: [cargo-test, pkcs11-contract, openbao-lease, key-rotation-drill, secret-no-log]

## §A Problem
The HSM adapter is the boundary where a typed root becomes a legally meaningful signature. The stamped IP named an HSM adapter but did not specify key handles, OpenBao lease binding, rotation overlap, or how the adapter avoids bringing private key bytes into process memory.

## §B Approach
Implement `oya-audit-chain-sealing-adapter-hsm` against `SignerPort`: it receives a pack/tenant/period root, selects the active key id from `PackEpoch`, signs via PKCS#11/KMIP with a SPIFFE-bound OpenBao certificate, and returns only public signature metadata. Local tests use `ed25519-dalek` seeds from `oya-audit-chain-domain` as a deterministic fake, never as production key handling.

## §C Deliverables
- `crates/oya-audit-chain-sealing-adapter-hsm/src/lib.rs` adapter implementing `SignerPort`.
- `HsmKeyHandle` and `OpenBaoLeaseRef` value types with redacted `Debug`.
- Fake signer test adapter using `Ed25519SigningKey::from_seed_bytes` for deterministic tests only.
- Rotation-overlap tests: outgoing and incoming key acceptance during overlap; old key rejected after retirement for new periods.
- Runbook link to `runbooks/hsm-key-rotation.md`.

## §D Implementation Steps
1. Model key handle strings separately from public verification keys.
2. Authenticate signing session via SPIFFE/OpenBao context; reject missing lease, expired lease, or pack mismatch.
3. Return `Ed25519Signature` metadata compatible with `verify_with_trusted_keys`.
4. Ensure no adapter error logs contain seed, private key bytes, PKCS#11 PIN, or OpenBao token.
5. Record key id and period id in the SealRecord for later KeyResolver lookup.
6. Add an integration fixture for HSM unreachable -> degraded unsealed buffer behavior.

## §E Acceptance
- `cargo test -p oya-audit-chain-domain ed25519` passes for signature verification behavior.
- New adapter tests prove redacted debug output for key handles and leases.
- `rg 'private_key|seed|pin|token' crates/oya-audit-chain-sealing-adapter-hsm/src` is reviewed for no logging or serialization.

## §F Evidence
- `crates/oya-audit-chain-domain/src/lib.rs` Ed25519 key/signature types.
- `microservices/audit-chain/policy/seal-integrity.md` SI-06 through SI-12.
- `microservices/audit-chain/runbooks/hsm-key-rotation.md`.

## §G Counterparts
AWS CloudTrail does not expose HSM-rooted Ed25519 per-event roots to tenants; Google and Microsoft use provider-managed key systems. Oyatie's HSM adapter closes that gap by producing signatures that tenants can verify against GitHub-pinned public-key manifests.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `sealing adapter hsm`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `sealing adapter hsm` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-008-sealing-adapter-hsm.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
