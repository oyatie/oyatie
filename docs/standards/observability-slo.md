---
doc_class: Standard
title: OpenSLO Authoring Standard (cross-cutting)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-observability + council-architecture
deciders: axis-observability, council-architecture, ops-sre-reliability
related_adrs: [ADR-0139, ADR-0131]
related_specs: [/specs/agentic-slo-gated-promotion.json]
applies_to: every microservice in microservices/ that promotes past dev
enforced_by: oya-governance-openslo-conformance CI lane (BLOCKER)
review_cadence: annually + on every OpenSLO spec version update
doc_status: published
---

# Standard: OpenSLO Authoring (cross-cutting)

## Purpose

Cross-cutting rules for authoring OpenSLO manifests at `microservices/<ms>/slos/*.openslo.yaml`. Mandatory for every µservice that wants to promote past `dev` (per ADR-0139 fail-closed default: no manifest → `verdict=rejected`).

## Version Pinning (latest LTS as of 2026-05-17)

Authors MUST verify against vendor LTS-release docs at deploy time; anywhere upstream advanced between this document's review cadence and deploy, surface the delta in a follow-up PR. Version changes that move outside the current LTS line require an ADR.

### Specs + protocols

| Standard | Pinned LTS (2026-05-17) | Source-of-truth | Notes |
|---|---|---|---|
| OpenSLO spec | v1.0 (stable LTS) | `github.com/OpenSLO/OpenSLO` tag v1.0.0 | v2 in working-draft; not yet LTS. Do not adopt until v2.0.0 release tag exists. |
| OpenTelemetry semantic conventions | v1.36.0 | `opentelemetry.io/docs/specs/semconv/` | Verify at deploy; the OTel project has had monthly releases through 2025–2026. |
| OpenTelemetry Protocol (OTLP) | v1.5.0 | `opentelemetry.io/docs/specs/otlp/` | gRPC + HTTP/JSON transports both stable. |
| Prometheus exposition format + remote-write | v2.55 (long-term-support line) | `prometheus.io/docs/specs/` | PromQL features tracked via Prometheus 3.x. |
| OpenAPI Specification | 3.2.0 | `spec.openapis.org/oas/v3.2.0` | Tooling support via openapi-generator 7.10+. |
| AsyncAPI Specification | 3.1.0 | `asyncapi.com/docs/reference/specification/v3.1.0` | Latest stable. |
| gRPC | v1.69+ wire-compatible | `grpc.io` | Per-language client libs LTS lines tracked separately. |
| Google SRE Workbook burn-rate model | 2018 edition, ch. 5 (canonical; no v2 published) | `sre.google/workbook/alerting-on-slos/` | Reference values normative. |
| Cedar policy language | v4.2.0 LTS | `cedarpolicy.com` / `github.com/cedar-policy/cedar` | All fragments at `microservices/<ms>/policy/*.cedar` author against v4.2 schema. |

### Grafana stack components (Layer-A)

All pinned to the current LTS line. Verify against `grafana.com/docs/<component>/latest/` at deploy time.

| Component | Pinned LTS (2026-05-17) | Notes |
|---|---|---|
| Grafana | 12.0 LTS (2024 release; LTS through 2026) | UI + dashboard runtime. |
| Grafana Mimir | 3.3 (LTS line) | TSDB; multi-tenancy enforcement. |
| Grafana Loki | 3.5 (LTS line) | Logs. |
| Grafana Tempo | 3.0 (LTS line) | Traces. |
| Grafana Pyroscope | 1.12 (LTS line) | Continuous profiling. |
| Grafana Alloy | 1.6 (LTS line) | OTel collector. Replaces Grafana Agent (retired). |
| Grafana OnCall | latest (now part of Grafana IRM stack) | Paging. |
| Prometheus Alertmanager | 0.28 (LTS line) | Alert routing. |
| Prometheus | 3.x (3.2 LTS line) | When self-querying outside Mimir; Mimir embeds Prometheus 3.x query path. |

### Container + orchestration

| Component | Pinned LTS | Notes |
|---|---|---|
| Kubernetes | 1.32 (LTS line; 1.31 also supported) | Cluster runtime. |
| containerd | 1.7 LTS line | Container runtime. |
| Istio | 1.24 LTS line | Service mesh + traffic-split. |
| Envoy | 1.32 LTS line | Sidecar proxy. |
| OpenBao | 2.x (latest stable) | Secret manager; OpenBao is the LTS-forked successor of HashiCorp Vault. |

### Rust toolchain (oyatie µservice crates)

| Component | Pinned | Notes |
|---|---|---|
| Rust toolchain | 1.97.1 (per `rust-toolchain.toml` + workspace `Cargo.toml`) | Latest stable as of 2026-06; bump when stable moves. |
| Rust edition | 2024 | Per ADR-0056 + workspace `Cargo.toml`. |
| rustfmt edition / style_edition | 2024 | Per `rustfmt.toml`. |
| `cargo-deny` | latest stable | Supply-chain lane. |
| `cargo-nextest` | latest stable | Test runner. |
| `protoc` + `prost`/`tonic` | latest stable | gRPC code-generation. |
| `openapi-generator-cli` | 7.10+ | OpenAPI → SDK generation. |

### Compliance frameworks (no LTS; pinned by issuance date)

| Framework | Pinned issuance | Notes |
|---|---|---|
| SOC 2 Type 2 | 2017 TSC + 2022 Points of Focus | Industry-standard issuance set. |
| ISO 27001:2022 + ISO 27002:2022 | 2022 revision (current) | Next major ISO revision is committee-draft only; no pre-emption. |
| GDPR | Regulation 2016/679 (2018 enforcement; current) | EDPB guidelines layered via per-pack overlay. |
| HIPAA | 45 CFR Parts 160 + 162 + 164 (2024 OCR enforcement guidance applied) | Verify against current OCR enforcement notices at audit time. |
| KR PIPA | 2020 + 2023-09 amendments | Currently in force. |
| KR-ISMS-P | 2023 revision | KISA-published. |
| KR 전자문서법 | 2021 amendment | Current. |
| APPI | 2022 (改正個人情報保護法 enforcement) | Current. |
| PDPA (SG) | 2012 + 2020 amendment | Current. |
| Privacy Act 1988 (AU) + APRA-CPS 234 | 2022 review applied | Current. |
| DPDPA 2023 (IN) + RBI Master Direction 2023 | Current. |
| LGPD (BR) + ANPD methodology | 2018 + 2023 ANPD updates | Current. |
| UAE PDPL Federal Decree-Law 45/2021 | Current. |
| KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity Framework 2017 | Current. |
| eIDAS | 910/2014 + eIDAS 2.0 (Regulation 2024/1183) | Verify which version applies per tenant. |
| NIS2 | Directive 2022/2555 (transposed; 2024 onwards) | Current. |
| DORA | Regulation 2022/2554 (enforcement 2025-01) | Current. |
| OpenSSF SLSA | Level 3 target | Per supply-chain.json. |

### Verification

- `oya-governance-version-pinning-conformance` CI lane (Slice D follow-up) refuses Cargo.toml + Helm-values + Dockerfile references to versions outside the LTS lines above without an ADR.
- Quarterly version-pinning refresh PR: ops-sre-reliability sweeps the LTS lines; bumps any that have moved.

## SLI Catalog (canonical, every µservice declares at least these four)

Every µservice MUST author one OpenSLO manifest per SLI below, unless an explicit waiver is recorded in `microservices/<ms>/slos/waivers.md`.

| SLI | What it measures | Indicator template | Minimum target |
|---|---|---|---|
| **availability** | Fraction of requests returning non-5xx | `sum(rate(http_requests_total{job="<svc>",status!~"5.."}[5m])) / sum(rate(http_requests_total{job="<svc>"}[5m]))` | 99.9% (production); 99.5% (staging) |
| **latency** | Fraction of requests under p99 budget | `histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{job="<svc>"}[5m])) by (le)) < <budget_seconds>` | p99 ≤ 200ms (production); 500ms (staging) |
| **correctness** | Service-defined; e.g., consistency-check pass rate | service-defined; CI lane validates expression is non-empty | 99.99% (production) |
| **freshness** | Service-defined; e.g., event-age max over rolling window for stream processors | service-defined | service-defined |

µservices NOT serving HTTP traffic (workers, batch jobs) author analogous SLIs (e.g., `job_completion_rate`, `batch_run_duration_p99`); the template is service-shape-driven.

## Burn-Rate Threshold Convention

Per Google SRE Workbook ch. 5 + `/specs/agentic-slo-gated-promotion.json` §`openslo_manifest_profile.alert_burn_rates`:

| Alert | Burn rate | Lookback window | Alert window | Action | Verdict on fire |
|---|---|---|---|---|---|
| fast-burn-page | 14.4× | 1h | 5min | page | held |
| slow-burn-page | 6× | 6h | 30min | page | held |
| ticket-burn | 3× | 3d | 6h | ticket | held |
| budget-exhausted | 1× | 30d | 1d | informational | held |

These are **mandatory minimums**. Per-µservice manifests may add stricter thresholds (e.g., 2× burn over 5min for HIPAA-scope µservices); cannot remove or weaken the defaults without a council-architecture-approved exception.

## Manifest Authoring Rules

### File location

`microservices/<ms>/slos/<sli>.openslo.yaml`. One file per SLI.

### Required frontmatter (OpenSLO v1.0 spec)

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: <ms>-<sli>
  displayName: "<Microservice> — <SLI human-readable>"
  labels:
    microservice: <ms>
    sli: <availability|latency|correctness|freshness>
    pack: pack-kr  # or other; defaults from µservice's pack-routing
    data_classes: BEHAVIORAL_TENANT_PRODUCT  # per Bominal ADR-0028 taxonomy
spec:
  service: <ms>
  indicator:
    metadata:
      name: <ms>-<sli>-sli
    spec:
      ratioMetric:  # for ratio-based SLIs (availability + most correctness)
        good:
          metricSource:
            type: Prometheus
            spec:
              query: <PromQL expression>
        total:
          metricSource:
            type: Prometheus
            spec:
              query: <PromQL expression>
      # OR thresholdMetric for latency-style SLIs
  objectives:
    - displayName: <readable>
      target: 0.999  # 99.9%
      timeSliceWindow: 5m  # optional, for windowed SLOs
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences  # or Timeslices
```

### Required `data_class` label

Every manifest MUST declare `metadata.labels.data_classes` matching at least one value from the Bominal ADR-0028 data-class taxonomy. The label drives downstream OTel SDK redaction + Cedar policy decisions.

### Forbidden patterns

The `oya-governance-openslo-conformance` lane (BLOCKER) refuses:

1. Manifest without `metadata.labels.microservice` (cannot route to pack).
2. Manifest with `target < 0.9` (sub-90% targets are unprofessional and likely manifest authoring bug).
3. Manifest with `target > 0.99999` (5-nines is operationally impossible for most services; require explicit council-approved waiver).
4. Manifest whose PromQL references metric series outside the µservice's owning namespace (cross-µservice indicator MUST go through Workflow/Ontology).
5. Manifest without `data_classes` label.
6. Manifest with `wildcard` (`*`) selectors on `tenant` label.
7. Manifest setting `multitenancy_enabled: false` anywhere (impossible by spec; rejected as suspect).
8. Manifest whose `timeWindow.duration` exceeds the pack's retention (per `data-residency.md`).

## PR Review Process

1. New OpenSLO manifest authored as part of a PR.
2. PR auto-tagged with `slo-engine-author-review` label.
3. CODEOWNERS includes:
   - µservice owner (axis owning the µservice)
   - axis-observability (review for burn-rate threshold sanity)
   - council-architecture (review for cross-µservice consistency)
4. CI lanes that MUST green:
   - `cargo fmt --check`
   - `oya gate validate openslo-conformance`
   - `oya gate validate data-class-coverage` (the manifest's labels match the µservice's declared data classes)
   - `oya gate validate openslo-promql-feasibility` (the PromQL expression returns non-empty data against representative Mimir snapshot)
5. Merge advances `microservices/<ms>/slos/*.openslo.yaml` through git PR; `slo-engine-worker` consumes the `OpenSloManifestUpdated` event (per `microservices/observability/contracts/asyncapi/eligibility-events.yaml`) and hot-reloads.

## SLI Type-Specific Guidance

### Availability

```yaml
indicator:
  spec:
    ratioMetric:
      good:
        metricSource:
          type: Prometheus
          spec:
            query: |
              sum(rate(http_requests_total{job="<ms>",status!~"5.."}[5m]))
      total:
        metricSource:
          type: Prometheus
          spec:
            query: |
              sum(rate(http_requests_total{job="<ms>"}[5m]))
```

### Latency

```yaml
indicator:
  spec:
    ratioMetric:
      good:
        metricSource:
          type: Prometheus
          spec:
            query: |
              sum(rate(http_request_duration_seconds_bucket{job="<ms>",le="0.2"}[5m]))
      total:
        metricSource:
          type: Prometheus
          spec:
            query: |
              sum(rate(http_request_duration_seconds_count{job="<ms>"}[5m]))
```

Express as "fraction of requests under budget" (ratio) rather than "p99 < budget" (threshold) — the ratio shape composes better with multi-window burn-rate math.

### Correctness

Service-defined. Examples:
- payment service: `sum(rate(payment_reconciled_total[5m])) / sum(rate(payment_initiated_total[5m]))`.
- workflow service: `sum(rate(workflow_step_completed_total[5m])) / sum(rate(workflow_step_started_total[5m]))`.

### Freshness

Service-defined. Examples:
- stream processor: `time() - max(event_consumed_at_max{job="<ms>"}) < 60`.
- batch job: `time() - max(job_last_completed_at{job="<ms>"}) < 3600`.

## Per-Pack Considerations

Authors do not need to write per-pack manifests; the manifest applies universally and is pack-scoped at evaluation time by the worker. However, packs may impose stricter overlays:

- pack-us-healthcare (HIPAA): minimum target 99.95% for any PHI-touching SLI; minimum retention 6y on audit-relevant data.
- pack-kr (KR-FSS for financial-services tenants): minimum target 99.95%; audit log retention 5y.
- pack-eu (NIS2 for in-scope tenants): minimum target 99.9% + incident-reporting timeline integration.

Pack overlays at `regional-packs/<pack>/observability-slo-overlay.md` (per-pack team owned).

## Versioning + Sunset

| Action | Procedure |
|---|---|
| Add SLI to a µservice | New manifest file; PR-reviewed; lane validates |
| Change target | Bump major version in manifest filename if loosening (e.g., `availability.openslo.yaml` → `availability-v2.openslo.yaml`); old + new live in parallel for 1 cycle; audit-chain records transition |
| Remove SLI | Move manifest to `microservices/<ms>/slos/retired/<sli>.openslo.yaml`; engine retains historical verdicts; new evaluations no longer triggered |
| OpenSLO spec major version bump (v1 → v2) | Repo-wide migration via dedicated ADR; reference Sloth or Nobl9 conversion tooling for cross-walk |

## References

- ADR-0139 (Agentic SLO-gated promotion).
- ADR-0131 (Per-microservice flat layout).
- `/specs/agentic-slo-gated-promotion.json`.
- `microservices/observability/PRD.md`.
- `microservices/observability/contracts/openapi/slo-engine.yaml` `/validate-openslo` endpoint.
- OpenSLO v1.0 — `github.com/OpenSLO/OpenSLO`.
- OpenTelemetry semconv — `opentelemetry.io/docs/specs/semconv/`.
- Google SRE Workbook ch. 5 — `sre.google/workbook/alerting-on-slos/`.
- Sloth (OpenSLO → Prometheus rules generator) — `github.com/slok/sloth`.
- Nobl9 OpenSLO converter — `nobl9.com`.
