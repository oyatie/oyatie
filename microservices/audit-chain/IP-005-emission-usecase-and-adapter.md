---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-005: Emission usecase, API, adapter, REST, SDK, and app

acceptance_lanes: [cargo-nextest, openapi-contract, asyncapi-contract, idempotency, lean-a2]

## §A Problem
Audit emission is on the synchronous critical path for every state-changing service call. The old plan listed crates but did not pin the behavior: authorization must be checked, request fingerprints must deduplicate safely, append-only chain writes must not fork, and outbox records must publish `audit.event.emit` without leaking producer payloads.

## §B Approach
Use the existing `oya-audit-chain-usecase` implementation as the source behavior and split it into clean layers: API DTOs, usecase orchestration, append adapter, REST transport, SDK client, and app composition. The usecase accepts only validated envelopes, calls `AuditChain::append_classifications`, appends via the adapter, and writes an outbox record on topic `oya.platform.audit`.

## §C Deliverables
- `crates/oya-audit-chain-emission-api` request/response DTOs based on `AuditEventEmitAppRequest` and `AuditEventEmitSuccessResponse`.
- `crates/oya-audit-chain-emission-usecase` orchestrator preserving authorization and idempotency behavior.
- `crates/oya-audit-chain-emission-adapter` append writer backed by Postgres/S3 in production and file adapter in fixtures.
- `crates/oya-audit-chain-emission-rest` OpenAPI-bound handler for non-mesh clients.
- `crates/oya-audit-chain-emission-sdk` Rust producer wrapper; future TS/Python are generated clients only, not authored backend logic.
- `crates/oya-audit-chain-emission-app` composition root.

## §D Implementation Steps
1. Lift constants `AUDIT_EVENT_EMIT_SURFACE`, `AUDIT_EVENT_TOPIC`, and contract refs into API/usecase modules.
2. Preserve authorization tenant/producer mismatch behavior and map it to 403.
3. Preserve idempotency replay handling: same fingerprint returns prior result, different fingerprint rejects.
4. Use the adapter boundary for durable append; never let REST or SDK write files directly.
5. Emit outbox records with `audit_schema_version`, `outbox_sequence`, and payload ref from the usecase result.
6. Add contract tests for REST/SDK DTO compatibility.

## §E Acceptance
- `cargo test -p oya-audit-chain-usecase` passes.
- `cargo test -p oya-audit-chain-file-adapter` passes for append/replay while production storage is not present.
- OpenAPI and AsyncAPI files named in `manifest.json` resolve or the IP remains blocked.

## §F Evidence
- `crates/oya-audit-chain-usecase/src/lib.rs` current orchestrator.
- `crates/oya-audit-chain-file-adapter/src/lib.rs` append ledger behavior.
- `microservices/audit-chain/sdk-plan.md` SDK rollout constraints.

## §G Counterparts
AWS CloudTrail and Google Cloud Audit Logs absorb producer writes behind managed APIs; Microsoft Purview Audit uses management activity APIs and blob fetches. Oyatie closes the producer side with an SDK/REST surface but adds stronger idempotent seal linkage and GitHub-pinned root verification.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `emission usecase and adapter`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `emission usecase and adapter` can be checked outside the primary storage plane. |

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
- Trigger evidence: `microservices/audit-chain/IP-005-emission-usecase-and-adapter.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-005-emission-usecase-and-adapter.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
