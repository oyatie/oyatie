---
doc_class: Policy
title: Tenant Isolation Policy (foundry-eval)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry
deciders: ops-security, axis-foundry, council-privacy, council-architecture
related_adrs: [ADR-0024, ADR-0028, ADR-0117, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-eval/threat-model.md
  - microservices/foundry-eval/dpia.md
  - microservices/foundry-eval/policy/tenant-scope.cedar
  - microservices/foundry-eval/policy/data-residency.md
review_cadence: quarterly + on every cross-tenant feature
doc_status: published
---

# Tenant Isolation Policy (foundry-eval µservice)

## Purpose

Define the load-bearing invariants that enforce per-tenant isolation across foundry-eval's eval-runs, parity reports, replay traces, golden outputs, and per-subject DEKs. Compromise of any one invariant escalates to Sev-1.

## Invariants

### TI-01 — ClickHouse partition by tenant_id

Every row in `parity_analytics`, `eval_run_results`, and `replay_divergence` tables carries `tenant_id` as part of the primary-key prefix. Queries lacking a `tenant_id` filter are rejected at the SQL-rewrite layer.

**Enforcement**: ClickHouse role-based access; `oya-check-clickhouse-tenant-partition-conformance` LEAN lane validates `ORDER BY` includes tenant_id at deploy.

### TI-02 — KEK per tenant

Every per-subject DEK is wrapped by exactly one per-tenant KEK; KEKs are KMS-resident; no KEK touches more than one tenant; cross-tenant KEK reuse is a Sev-1 incident.

**Enforcement**: KMS key naming convention `oya-foundry-eval-kek-<tenant_hash>`; `oya-check-kms-kek-per-tenant-conformance` LEAN lane.

### TI-03 — S3 object key prefixes scope tenant data

Every golden-output object key is prefixed `goldens/<tenant_hash>/<capability_id>/<case_id>` (and similarly for replay-traces `replay/<tenant_hash>/<capability_id>/<day>/<sample_id>`). S3 IAM policies restrict per-SA access to prefixes matching the SA's bound tenant.

**Enforcement**: IAM policy review + LEAN check `oya-check-s3-prefix-tenant-conformance`.

### TI-04 — Postgres row-level security

The eval-set metadata table `eval_sets` carries `tenant_id`; row-level security policies enforce that reads/writes match `current_setting('app.tenant_id')`.

**Enforcement**: Postgres CREATE POLICY at schema deploy; integration tests verify cross-tenant queries return empty.

### TI-05 — Cedar tenant-scope policy (`policy/tenant-scope.cedar`)

Every read action on eval-run / parity / replay / verdict resources requires `principal.tenant_id == resource.tenant_id`. Default-deny + explicit FORBID for cross-tenant unless tenant_id matches.

**Enforcement**: Cedar policy attached to slo-engine-rest; integration tests assert cross-tenant 403.

### TI-06 — Network policy: per-tenant pod cannot reach other-tenant resources

Per-case ephemeral pods carry `oya-tenant-id` label; NetworkPolicy restricts egress to (a) provider model API allowlist + (b) S3/ClickHouse paths matching the pod's tenant.

**Enforcement**: Kubernetes NetworkPolicy review + automated CIS Benchmark scan.

### TI-07 — Cross-tenant aggregates require DP-noise

Any cross-tenant aggregate exposed in dashboards or public reports must pass through differential-privacy aggregation (ε ≤ 1 per published aggregate) per `policy/dp-analysis.md`.

**Enforcement**: `oya-check-dp-noise-on-cross-tenant-aggregates` LEAN lane + DP-analysis publication.

### TI-08 — Per-subject DEK shred breaks replay, never reaches other tenants

When DSR shred fires for `subject_id` belonging to `tenant_id=T`, only T's per-subject DEKs are deleted; other tenants' DEKs are untouched. Cross-tenant shred is structurally impossible because per-tenant KEK boundaries prevent cross-tenant DEK access in the first place.

**Enforcement**: DSR cascade runner emits `EvalSubjectShred{tenant_id, subject_id, shredded_at}`; audit-chain consumer verifies tenant_id consistency.

## Verification

- Pen-test annually: attempt to read tenant-B data while authenticated as tenant-A.
- LEAN lane suite: `oya-check-{clickhouse-tenant-partition,kms-kek-per-tenant,s3-prefix-tenant,dp-noise-on-cross-tenant-aggregates}-conformance`.
- Integration test: `tests/integration/cross-tenant-isolation.rs` covers each invariant.

## Incident Response

Any violation of an invariant triggers `runbooks/parity-regression-triage.md` (default) escalating to `runbooks/security-incident.md` if cross-tenant exposure is suspected. Sev-1 by default; reclassified after scoping.

## References

- ADR-0140 (Cedar policy enforcement)
- threat-model.md T-I-01, T-I-03, T-I-04, T-I-05, T-L-01
- dpia.md R-02
- policy/tenant-scope.cedar
- policy/data-residency.md
