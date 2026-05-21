---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-015: Self-observability wiring for audit-chain

acceptance_lanes: [metrics-contract, dashboard-json, alert-rules, recursive-audit]

## §A Problem
Self-observability is separate from SLO document existence: audit-chain must emit metrics and recursive audit events from emission, sealing, verification, query, and retention paths. The stamped IP did not specify which metrics exist, which dashboards use them, or how self-observability avoids high-cardinality tenant leakage.

## §B Approach
Instrument the concrete service paths with low-cardinality metrics and recursive audit events. Emission reports accept/reject latency and authorization denials; sealing reports period cycle, HSM errors, unsealed backlog, and root-publication lag; verification reports verdict reasons; query/export reports scoped read/export latency; retention reports sweep and redaction outcomes.

## §C Deliverables
- Metric contract doc under `microservices/audit-chain/observability/metrics.md` or equivalent service-local surface.
- Dashboard updates for `dashboards/emission-rate.json`, `seal-latency.json`, and `verification-failure-rate.json`.
- Alert rules for HSM unreachable, seal backlog, root channel mismatch, verification failure spike, and retention sweep missed.
- Recursive audit event classes for `AuditChainSelfSloStateChanged`, `SealPublicationLagged`, `VerificationFailureSpikeDetected`, and `RetentionSweepMissed`.
- Runbook links to restart, HSM key rotation, Merkle discrepancy, and regulator export failure.

## §D Implementation Steps
1. Define metric names with labels: pack, cell, period_state, route, outcome, reason; exclude raw tenant_id and principal_id.
2. Add instrumentation hooks in usecase boundaries, not pure domain code.
3. Map every alert to a runbook in `runbooks/`.
4. Ensure SLO burn events call emission SDK and do not bypass audit-chain because they are self-events.
5. Check dashboard JSON for panels tied to manifest SLOs.
6. Add tests or static checks for metric label allowlist.

## §E Acceptance
- `rg 'tenant_id|principal_id' microservices/audit-chain/dashboards` returns no high-cardinality label use except explanatory text.
- Every alert rule references a runbook path that exists.
- `manifest.json` SLO names map to an OpenSLO file and dashboard panel.

## §F Evidence
- `microservices/audit-chain/dashboards/*.json`.
- `microservices/audit-chain/slos/*.openslo.yaml`.
- `microservices/audit-chain/runbooks/audit-chain-restart.md` and related runbooks.

## §G Counterparts
Datadog and Splunk excel at operational audit observability, while AWS CloudTrail and Google Cloud Audit Logs expose service-health and delivery surfaces indirectly. This IP gives Oyatie the necessary self-observability while keeping GitHub-pinned cryptographic proof as the audit-chain-specific differentiator.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `self observability slo wiring`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `self observability slo wiring` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-015-self-observability-slo-wiring.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-015-self-observability-slo-wiring.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
