---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-011: Verification stack for inclusion proofs and signatures

acceptance_lanes: [cargo-test, proof-mutation-tests, openapi-contract, offline-verify-fixture]

## §A Problem
Verification is the tenant-facing proof that audit-chain is more than append-only logging. The previous IP named seven crates but did not pin the exact checks: trusted key epoch, Ed25519 signature, Merkle inclusion, prior-root chain, pack residency, and structured failure reasons.

## §B Approach
Build a pure verification kernel/domain plus API/REST/SDK that accepts an event envelope, claimed proof, claimed root, signature, period id, pack, and tenant partition. The verifier reads only published roots and public keys, then returns a verdict with reason codes; it never mutates state.

## §C Deliverables
- `crates/oya-audit-chain-verification-kernel` with `MerkleVerifier`, `RootRegistry`, and `KeyResolver` ports.
- `crates/oya-audit-chain-verification-domain` proof and signature checks using `MerkleTree::verify_proof` and `Ed25519Signature::verify_with_trusted_keys`.
- `crates/oya-audit-chain-verification-api` verdict DTOs with failure reasons such as `key_epoch_mismatch`, `signature_invalid`, `proof_invalid`, `prior_root_missing`, and `pack_mismatch`.
- `crates/oya-audit-chain-verification-rest` and SDK surfaces for tenant/auditor use.
- Offline fixture under `reference-implementations/emit-and-verify-rust-sdk.md`.

## §D Implementation Steps
1. Resolve public key by `(pack, tenant_partition, period_id)` before signature verification.
2. Verify Ed25519 root signature before proof walking.
3. Use leaf count and leaf index to reject shortened proof paths.
4. Walk prior-root chain to genesis or requested lower bound.
5. Reject cross-pack root/public-key mixtures even if a signature verifies.
6. Emit `VerificationFailed` audit event only outside the pure verifier, in the usecase boundary.

## §E Acceptance
- `cargo test -p oya-audit-chain-domain ed25519 merkle` passes.
- Mutation fixture flips each proof component and receives `verified=false` with specific reason.
- REST contract includes no mutation endpoint in verification stack.

## §F Evidence
- `crates/oya-audit-chain-domain/src/lib.rs` Ed25519 verification methods.
- `crates/oya-audit-chain-domain/src/merkle_tree.rs` proof verification.
- `microservices/audit-chain/policy/seal-integrity.md` SI-13 through SI-15.

## §G Counterparts
CloudTrail offers integrity validation for log files, while Google Cloud Audit Logs and Microsoft Purview Audit require trust in provider access paths. This IP is where Oyatie exceeds those counterparts by letting a tenant verify a GitHub-pinned root, public key epoch, Ed25519 signature, and Merkle proof offline.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `verification stack`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `verification stack` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-011-verification-stack.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
