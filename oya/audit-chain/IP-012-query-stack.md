---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-012: Tenant-scoped audit query and export stack

acceptance_lanes: [cargo-test, cedar-scope-tests, pagination-tests, export-bundle-verify]

## §A Problem
Audit query is not a generic search endpoint. It handles tenant forensics, regulator exports, and audit-of-audits while preserving residency and Cedar scope. The stamped IP did not distinguish query repository, export bundle construction, auditor engagement scope, or read emission.

## §B Approach
Implement query as a Cedar-gated read stack: API DTOs define filters, query-usecase evaluates tenant/auditor scope, Postgres adapter pages records by tenant/time/event/principal/entity, export builder freezes a Merkle-rooted evidence bundle, and the usecase emits an audit event for every read/export.

## §C Deliverables
- `crates/oya-audit-chain-query-kernel` ports: `AuditQueryRepository`, `ExportBuilder`, and `AuditorEngagementResolver`.
- `crates/oya-audit-chain-query-api` types: `AuditQuery`, `QueryResult`, `ExportBundle`, `AuditorEngagement`.
- `crates/oya-audit-chain-query-adapter-postgres` indexed range-scan implementation.
- `crates/oya-audit-chain-query-rest` tenant/auditor endpoints with pagination.
- `crates/oya-audit-chain-query-sdk` verifier-aware client for export bundles.

## §D Implementation Steps
1. Define filters around `tenant_id`, time range, event type, principal, entity, pack, and period.
2. Evaluate `policy/auditor-scope.cedar`, `tenant-scope.cedar`, and data-residency rules before repository calls.
3. Return sealed/unsealed state per event so callers know whether proof is available.
4. Build export bundles with root, inclusion proof, public key ref, and engagement id.
5. Emit read/export audit events through the emission usecase to preserve audit-of-audits.
6. Add pagination and max-window guardrails to avoid Splunk-style unbounded scans in M01.

## §E Acceptance
- Tests cover same-tenant read, cross-tenant denial, expired auditor engagement, residency mismatch, pagination cursor replay, and export proof verification.
- `microservices/audit-chain/runbooks/audit-export.md` names the same export bundle fields.
- Query p99 target in PRD remains bounded to single-tenant 30-day windows.

## §F Evidence
- `microservices/audit-chain/policy/auditor-scope.cedar` and `tenant-scope.cedar`.
- `microservices/audit-chain/runbooks/audit-export.md`.
- `microservices/audit-chain/PRD.md` FR-05 and FR-10.

## §G Counterparts
Splunk and Datadog are stronger at SIEM-style search, while AWS CloudTrail Lake and Google Log Analytics provide managed query stores. This IP keeps Oyatie honest: M01 delivers scoped forensic queries and signed export bundles, not a broad Splunk replacement; GitHub-pinned proofs remain the differentiator.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `query stack`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `query stack` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-012-query-stack.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-012-query-stack.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
