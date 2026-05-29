---
microservice: observability
ip: IP-029
title: OTel Tail Sampling Processor config (gateway tier deployment)
status: Drafting
owner: axis-observability
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0139, ADR-0186, ADR-0210]
---

# IP-029 — OTel Tail Sampling Processor config

## Purpose

Deploy the OTel Collector gateway tier with Tail Sampling Processor configured per ADR-0210 closed policy set. Helm chart at `iac/helm/otel-tailsampling-collector/`.

## Acceptance criteria

1. Helm chart deployed; 3-replica minimum; HPA on memory pressure.
2. Tail policies wired: `status_code=ERROR`, `latency_p99`, `slo_burn`, `audit_event`, `new_endpoint_warmup`, `random_baseline`.
3. `decision_wait: 30s`; memory budget: 256 MiB processor + 256 MiB headroom = 512 MiB request.
4. Per-µservice manifest override flows from `manifest.json` `observability.trace_sampling_recipe`.
5. High-traffic escape hatch: drop `head_bps` to 10 (0.1%) when µservice sustained > 5,000 req/sec; drop to 1 (0.01%) at > 50,000 req/sec.
6. ≥ 5 integration tests: error-trace-preserved + slow-trace-preserved + new-endpoint-preserved + baseline-1pct-sampled + memory-budget-respected.

## Cross-references

- ADR-0210 — tail-sampling policy.
- ADR-0186 — observability backplane.
- `iac/helm/otel-tailsampling-collector/`.

## Wave 15 substance conversion

### A. Problem this IP closes

Tail sampling decides which traces survive under load. If it is generic, Oyatie loses the traces needed to prove SLO burn, audit events, new endpoints, failures, and promotion eligibility.
The previous IP named policy labels but did not bind them to actual observability fields, SLO files, ClickHouse/Tempo retention, runbooks, or counterpart parity against Datadog, New Relic, Grafana, and Honeycomb.
This IP closes the gateway-tier sampling configuration that protects critical evidence while controlling telemetry cost and cardinality.

### B. Approach

Deploy an OTel Collector gateway tail-sampling processor with closed policy set: error, p99 latency, SLO burn, audit event, new endpoint warmup, random baseline, and service-specific overrides.
Use service-local telemetry field names from `crates/oya-observability-domain/src/lib.rs` and SLO manifests under `microservices/observability/slos/`.
High-traffic services get adaptive head sampling before tail sampling, but critical evidence classes remain preserved.
Config lives under canonical IaC once the ADR-0328 OpenTofu context exists; the Helm path is packaging evidence, not the full provisioning substrate.

### C. Deliverables

- Add or update OTel Collector tail-sampling config under the actual chart/config path.
- Add policy definitions for `status_code=ERROR`, latency p99, SLO burn, audit event, new endpoint warmup, random baseline, and service override.
- Add manifest field documentation for `observability.trace_sampling_recipe`.
- Add metrics for decision wait, sampled spans, dropped spans, memory pressure, policy hit counts, and per-service override use.
- Add runbook links to `tail-sampling-buffer-saturated.md` and `trace-sampling-loss-investigation.md`.
- Add SLO `microservices/observability/slos/tail-sample-fidelity.openslo.yaml` as fidelity evidence.

### D. Implementation steps

1. Read `crates/oya-observability-domain/src/lib.rs` and map canonical fields to OTel attributes.
2. Define policy matching for `error.type`, status error, latency threshold, audit data class, and SLO-burn window.
3. Configure `decision_wait: 30s` and explicit memory budget with queue backpressure.
4. Add `new_endpoint_warmup` keyed by service, route, method, and 30-day TTL.
5. Add `random_baseline` default sampling and high-traffic head-rate escape hatch.
6. Add service override recipe shape to manifest validation.
7. Add resource labels for tenant class, cell, region, and deployment context without raw tenant ID leakage.
8. Add config validation and dry-run fixtures for each policy branch.
9. Wire collector metrics to dashboards and alert on buffer saturation.
10. Record ADR-0328 gap if only Helm exists and OpenTofu context modules are absent.

### E. Acceptance

- Error, slow, audit, SLO-burn, and new-endpoint traces are preserved in fixture tests.
- Baseline traces sample at configured rate with tolerance documented in IP-031.
- Memory budget pressure produces alerts before trace-loss.
- Manifest recipe override validates and propagates to config.
- IaC evidence distinguishes Helm packaging from required OpenTofu provisioning.

### F. Evidence

- `crates/oya-observability-domain/src/lib.rs` canonical telemetry fields and exposure rules.
- `microservices/observability/slos/tail-sample-fidelity.openslo.yaml`.
- `microservices/observability/runbooks/tail-sampling-buffer-saturated.md`.
- `microservices/observability/runbooks/trace-sampling-loss-investigation.md`.
- `microservices/observability/feature-parity-matrix-2026-05-20.md` Datadog/New Relic/Grafana/Honeycomb trace capability references.
- `microservices/observability/coherence-audit-2026-05-20.md` ADR-0328 IaC gap.

### G. Counterpart closure

| Counterpart | Trace retention expectation | This IP closure |
|---|---|---|
| Datadog | keep error/latency traces under load | error and p99 preservation policies |
| New Relic | OTel traces preserve service-level evidence | SLO-burn and new-endpoint policies |
| Grafana Tempo | sampled trace store with cost control | gateway tail sampling and baseline rate |
| Honeycomb | wide-event debugging and anomaly traces | audit/new-endpoint/error preservation |
| GitHub | CI trace evidence for release gates | sampled traces retain promotion-critical failures for review |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-029-tail-sampling-processor-config.md` matched `p99, SLO`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/observability/IP-029-tail-sampling-processor-config.md` matched `cost`; anchors `microservices/observability/manifest.json, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
