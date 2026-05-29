---
microservice: observability
ip: IP-031
title: Tail-sample fidelity regression test (errors + slow + new-endpoint preserved)
status: Drafting
owner: axis-observability
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0210]
---

# IP-031 — Tail-sample fidelity test

## Purpose

Regression test: inject 1000 traces with mix of (i) errors, (ii) slow > p99 threshold, (iii) new-endpoint, (iv) baseline. Assert tail processor preserves 100% of (i), (ii), (iii) + ~1% of (iv).

## Acceptance criteria

1. Test harness emits 1000 traces via OTLP gRPC into the Tail Sampling Processor gateway.
2. Query Tempo for delivered traces.
3. Assert preservation rates:
   - error traces: 100%
   - p99 slow: 100%
   - new-endpoint (within 30-day window): 100%
   - audit-event: 100%
   - SLO-burn-window: 100%
   - baseline: 1% ± 0.5%
4. Run on every PR via CI.
5. Failure budget: 0 (any drop in fidelity fails CI).

## Cross-references

- ADR-0210 — tail sampling policy.
- IP-029 — Collector config.

## Wave 15 substance conversion

### A. Problem this IP closes

Sampling policy is only trustworthy if tests prove that critical traces survive. A config that looks correct can still drop error, slow, audit, SLO-burn, or new-endpoint traces under queue pressure.
The old IP had a useful outline but no binding to real service fields, SLOs, runbooks, or promotion gates.
This IP closes the regression harness that keeps IP-029 and IP-030 honest.

### B. Approach

Build a deterministic OTLP fixture harness that emits labeled traces into the tail-sampling gateway and queries the downstream trace store or collector export sink for delivered traces.
The harness uses observability's canonical fields from `crates/oya-observability-domain/src/lib.rs` and the policy names from IP-029.
It runs on recipe/config changes and before promotion of the observability collector stack.
Failures are blockers because they directly weaken release-gate and incident evidence.

### C. Deliverables

- Add test harness under the eventual observability test crate or integration test path.
- Emit fixture traces for error, p99 slow, audit event, SLO-burn, new endpoint, baseline, high-cardinality noisy path, and forbidden/redacted data-class path.
- Query delivered traces from Tempo/ClickHouse/exporter test sink.
- Add assertions for 100 percent preservation of critical classes and configured baseline tolerance.
- Add CI gate name and runbook link for failure triage.
- Bind SLO `microservices/observability/slos/tail-sample-fidelity.openslo.yaml`.

### D. Implementation steps

1. Create fixture trace generator with stable trace IDs and span attributes from `fields::*` constants.
2. Emit 1,000 baseline traces plus critical-class traces with deterministic counts.
3. Mark error traces using status/error attributes and `error.type`.
4. Mark slow traces with latency beyond p99 threshold.
5. Mark audit traces using data class/audit markers that must never be sampled away.
6. Mark SLO-burn traces with the SLO policy labels used by IP-029.
7. Mark new-endpoint traces with route/method/service labels and TTL context.
8. Query the downstream sink and count delivered trace IDs by class.
9. Fail on any missing critical trace; allow baseline tolerance only for baseline class.
10. On failure, link to `trace-sampling-loss-investigation.md` and `tail-sampling-buffer-saturated.md`.

### E. Acceptance

- Error traces preserve 100 percent.
- Slow p99 traces preserve 100 percent.
- Audit-event traces preserve 100 percent.
- SLO-burn traces preserve 100 percent.
- New-endpoint traces preserve 100 percent during the TTL window.
- Baseline traces sample at configured rate within tolerance.
- Test fails if forbidden/redacted payloads bypass exposure policy.

### F. Evidence

- `crates/oya-observability-domain/src/lib.rs` fields and `TelemetryLogExposure`.
- `microservices/observability/slos/tail-sample-fidelity.openslo.yaml`.
- `microservices/observability/runbooks/trace-sampling-loss-investigation.md`.
- `microservices/observability/runbooks/tail-sampling-buffer-saturated.md`.
- `microservices/observability/IP-029-tail-sampling-processor-config.md`.
- `microservices/observability/IP-030-sample-recipe-per-microservice.md`.

### G. Counterpart closure

| Counterpart | Fidelity expectation | This IP closure |
|---|---|---|
| Datadog | errors and slow traces survive sampling | 100 percent error/slow assertions |
| New Relic | service-level evidence stays queryable | SLO-burn preservation assertion |
| Grafana Tempo | sampled traces are verifiably delivered | downstream sink count by trace ID |
| Honeycomb | rare high-value events are retained | audit/new-endpoint preservation assertions |
| GitHub | CI blocks evidence-loss regressions | fidelity harness fails promotion-related PRs |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-031-tail-sample-fidelity-test.md` matched `p99, SLO`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
