---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-002: Self-SLO manifests for audit-chain

acceptance_lanes: [openslo-validate, dashboard-json-validate, slo-burn-alerts, self-audit-emission]

## §A Problem
Audit-chain is the evidence backbone, so its own SLOs must measure proof freshness, sealing latency, storage availability, and verification correctness. The prior plan had useful line volume but still mixed generic SLO language with no explicit mapping from `manifest.json` SLO rows to dashboard files and OpenSLO manifests.

## §B Approach
Bind every audit-chain SLI in `manifest.json` to a concrete OpenSLO file under `slos/` and a dashboard panel under `dashboards/`. The SLOs must distinguish synchronous emission acceptance from asynchronous seal publication; otherwise the service can appear healthy while roots lag and tenants lose CloudTrail/Purview-grade audit freshness.

## §C Deliverables
- `slos/seal-write-latency.openslo.yaml` tracks synchronous append receipt latency.
- `slos/seal-cycle-latency.openslo.yaml` tracks period close to signed-root publication.
- `slos/merkle-chain-verification-latency.openslo.yaml` tracks pure verification response time.
- `slos/chain-of-custody-integrity-correctness.openslo.yaml` tracks intact proof verification ratio.
- `dashboards/emission-rate.json`, `dashboards/seal-latency.json`, and `dashboards/verification-failure-rate.json` expose the same labels without high-cardinality tenant ids.

## §D Implementation Steps
1. Parse `manifest.json` SLO rows and fail the PR if any listed SLO file is absent.
2. Normalize labels to pack, cell, period_state, and outcome; keep tenant ids out of Prometheus labels.
3. Add alert expressions for unsealed-buffer age, HSM signing failures, root-publication lag, and verification-failed spikes.
4. Emit audit-chain self events for SLO-state transitions so an SLO incident has a seal.
5. Wire dashboards to the SLO file names rather than ad hoc metric names.
6. Document reduced claims when only logical tests are present and no staging load evidence exists.

## §E Acceptance
- `rg 'oya_audit_chain_' microservices/audit-chain/slos microservices/audit-chain/dashboards` finds the same core metric families.
- `cargo test -p oya-audit-chain-usecase audit_event_emit` covers the accepted emit path that feeds the SLOs.
- `oya gate validate authority-cohesion --microservice audit-chain` can trace manifest SLOs to files.

## §F Evidence
- `microservices/audit-chain/manifest.json` SLO array.
- `microservices/audit-chain/PRD.md` availability and performance targets.
- `microservices/audit-chain/performance-benchmark-numbers-2026-05-20.md` benchmark caveats.

## §G Counterparts
AWS CloudTrail and Google Cloud Audit Logs publish service limits and delivery behavior; Microsoft Purview Audit documents availability delays for audit records. This IP makes Oyatie's claim narrower and testable: seal-cycle and verification SLOs, plus GitHub-pinned root lag, not a broad statement that audit is always instant.

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
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `self slo manifest`.
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
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `self slo manifest` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-002-self-slo-manifest.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-002-self-slo-manifest.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
