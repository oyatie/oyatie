---
microservice: observability
ip: IP-030
title: Per-µservice trace sampling recipe (manifest field + CD propagation)
status: Drafting
owner: axis-observability
date: 2026-05-18
related_adrs: [ADR-0186, ADR-0210]
---

# IP-030 — Per-µservice trace sampling recipe

## Purpose

Each µservice declares its trace sampling recipe in `manifest.json`. CD propagates the recipe to the OTel Tail Sampling Processor values.yaml + per-µservice agent collector head sampling rate.

## Acceptance criteria

1. `manifest.json` `observability.trace_sampling_recipe` shape declared in ADR-0210.
2. CD step regenerates `iac/helm/otel-tailsampling-collector/values.yaml` from per-µservice manifest at promotion time.
3. Per-µservice agent collector `head_bps` configured from manifest.
4. New-endpoint TTL (30-day) tracked per (µservice, route).
5. ≥ 4 integration tests.

## Cross-references

- ADR-0210 — tail sampling policy.
- IP-029 — Collector config.

## Wave 15 substance conversion

### A. Problem this IP closes

Tail sampling cannot be one global knob. Community, payments, tenancy, observability, and workflow have different risk and traffic profiles, and a single sample rate would either drop critical evidence or explode cost.
The previous IP said each µservice declares a recipe but did not define the field shape, validation behavior, propagation path, or relationship to service manifests.
This IP closes the per-µservice trace sampling recipe contract.

### B. Approach

Add an `observability.trace_sampling_recipe` object to each service manifest or to a generated service registry view, with schema-controlled values for baseline rate, high-traffic escape hatch, preserved route classes, audit class preservation, new-endpoint TTL, and SLO-burn preservation.
The CD/promotion path reads recipes, validates them, and renders collector config for the OTel tail-sampling gateway.
Recipe changes are promotion-affecting because they can alter evidence fidelity.

### C. Deliverables

- Define `observability.trace_sampling_recipe` schema in the appropriate specs file or service manifest schema.
- Update `microservices/observability/manifest.json` with its own recipe.
- Add validation for baseline bps, high-traffic thresholds, preserved policy names, route-class list, and TTL.
- Add propagation logic evidence from manifest to collector values/config.
- Add tests for invalid rates, missing preserved evidence classes, and service override conflicts.
- Update `microservices/observability/contracts/metric-naming-convention.md` if metric/label names are affected.

### D. Implementation steps

1. Define the recipe fields: `baseline_bps`, `high_traffic_escape_hatch`, `preserve_errors`, `preserve_slo_burn`, `preserve_audit_events`, `new_endpoint_ttl_days`, and optional route overrides.
2. Set conservative defaults that preserve evidence before cost optimization.
3. Add schema validation for integer ranges and known policy names.
4. Add service manifest example for observability and one fixture for a high-volume service.
5. Render recipe into IP-029 collector policy config with deterministic ordering.
6. Reject recipe changes that remove error/audit/SLO-burn preservation.
7. Add diff evidence so reviewers can see which collector policies changed.
8. Add promotion gate hook requiring tail-sampling fidelity tests after recipe changes.
9. Add documentation for OCI Always Free profile constraints without reducing critical preservation.
10. Add rollback behavior to restore prior recipe and collector config.

### E. Acceptance

- Recipe schema rejects invalid bps and unknown policies.
- No recipe can disable error, audit, or SLO-burn preservation.
- CD/render step produces deterministic collector config for IP-029.
- Recipe changes trigger IP-031 fidelity tests.
- Observability's own manifest carries a concrete recipe or a tracked gap.

### F. Evidence

- `microservices/observability/manifest.json`.
- `microservices/observability/contracts/metric-naming-convention.md`.
- `microservices/observability/slos/tail-sample-fidelity.openslo.yaml`.
- `microservices/observability/IP-029-tail-sampling-processor-config.md`.
- `microservices/observability/coherence-audit-2026-05-20.md` manifest and ADR-0328 findings.

### G. Counterpart closure

| Counterpart | Sampling/config expectation | This IP closure |
|---|---|---|
| Datadog | service-level retention and ingestion controls | per-service recipe and rate validation |
| New Relic | OTel data preservation choices | mandatory preserved classes |
| Grafana | Alloy/Tempo config as code | deterministic rendered collector config |
| Honeycomb | retain high-value traces while controlling volume | route/policy-specific sampling recipe |
| GitHub | reviewable config changes in PRs | deterministic recipe diffs make sampling changes auditable |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-030-sample-recipe-per-microservice.md` matched `SLO, payment`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/observability/IP-030-sample-recipe-per-microservice.md` matched `cost`; anchors `microservices/observability/manifest.json, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
