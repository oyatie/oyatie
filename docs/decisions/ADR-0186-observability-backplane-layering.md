---
id: ADR-0186
status: Accepted
deciders: council-architecture, ops-sre-reliability, axis-observability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: [ADR-0263, ADR-0341]
related: [ADR-0139, ADR-0145, ADR-0148, ADR-0153, ADR-0180-slo-composition-inheritance-arithmetic, ADR-0182, ADR-0183, ADR-0184, ADR-0185]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

# ADR-0186 — Observability backplane layering: collection / storage / query / alert / SLO authoring; zero overlap

## Status

Accepted (2026-05-18). Mandates a five-stage observability backplane in which each stage is owned by exactly one component. No stage reaches across boundaries.

## Context

Per ADR-0145 (Invariant 2 — OpenTelemetry trace propagation) and ADR-0148 (Hubble flow obs + ztunnel telemetry + waypoint Envoy access logs), oyatie generates four signal types fleet-wide: metrics, logs, traces, and continuous profiles. ADR-0139 prescribes 4-window burn-rate SLO alerting. ADR-0153 already names the LGTM stack at a high level; this ADR formalizes the layered backplane shape.

The hyperscaler bar for observability:

- **Consistency** — same signal-collection shape across all 32 µservices and every cell.
- **Quality** — specialized backend per signal type; no shoehorning logs into a metric store or traces into a log store.
- **Scalability** — horizontally-scalable storage backends per signal type.
- **Maintainability** — one query/visualization pane; one alert-routing path.
- **Integration** — OpenTelemetry OTLP is the single ingest protocol; OpenSLO is the single SLO authoring source.

Anti-patterns this ADR forecloses:

1. Per-µservice ad-hoc collection (Prometheus scraping + Filebeat + Jaeger + Pyroscope all configured independently) — config sprawl; no canonical pipeline.
2. Shared storage backend across signal types (one OpenSearch cluster for metrics + logs + traces) — cost + query-pattern mismatch + retention-policy collision.
3. Manual SLO authoring per µservice — drift between intent and Prometheus burn-rate rules.

## Decision

Oyatie adopts a **five-stage observability backplane** in which each stage owns one concern:

### Stage 1 — Collection: OpenTelemetry Collector (single binary)

- **OpenTelemetry Collector** is the canonical collector across the fleet. Single binary; receivers + processors + exporters configured via OTLP. Deployed in two roles:
  - **Agent mode** as a per-node DaemonSet — receives metrics/logs/traces from local pods via OTLP gRPC (port 4317) and the application's tracing-client-kernel.
  - **Gateway mode** as a Deployment in the `observability` µservice namespace — receives agent forwarding, processes (batching, tail-sampling, attribute enrichment with cell/tenant/SPIFFE-ID context), exports to the specialized storage backends.
- **Grafana Alloy** is permitted as an alternate distribution of OpenTelemetry Collector (binary-compatible) for environments where Grafana Cloud integration is the operator-skill anchor; oyatie's canonical reference is the upstream OpenTelemetry Collector + Alloy where Grafana Labs operator experience favors it.

### Stage 2 — Storage (specialized per signal type)

Each signal type ships to its specialized backend:

| Signal | Backend | Pin (2026-05-18) | Retention |
|---|---|---|---|
| Metric (hot path; query-friendly) | **Prometheus** | 3.12 (LTS 3.5 through July 2026) | 15 days |
| Metric (long-retention; horizontally scalable) | **Grafana Mimir** | 3.0 | 1 year |
| Log | **Grafana Loki** | 3.4 | 30 days (configurable per-namespace) |
| Trace | **Grafana Tempo** | latest 2.x (post-MCP-server) | 7 days hot; 90 days cold |
| Continuous profile (optional, per-µservice opt-in) | **Pyroscope** (folded into Grafana ecosystem) | latest | 14 days |

Prometheus serves short-retention high-cardinality queries; Mimir handles long-retention and federation. Loki uses object-storage backend (S3-compatible) with native OTLP ingestion (per recent Loki releases). Tempo stores trace blobs in object storage with native OTLP ingestion.

### Stage 3 — Query / visualization: Grafana (single pane of glass)

- **Grafana 12.x** is the canonical query and visualization pane across all storage backends.
- One Grafana instance per cell (cell-µservice scoped); cross-cell federation via Grafana Mimir for metrics; per-cell Loki/Tempo for logs/traces (federation via Loki ruler + Tempo multi-tenant if cross-cell trace correlation needed).
- Dashboards live as JSON in source control at `microservices/<ms>/dashboards/` (per ADR-0131 flat layout); Grafana provisioning reads from the canonical path.

### Stage 4 — Alert routing: AlertManager → PagerDuty + Opsgenie (vendor-neutral via webhook)

- **AlertManager** receives Prometheus + Mimir burn-rate alerts.
- Routes via vendor-neutral webhooks to **PagerDuty** + **Opsgenie** (multi-vendor for vendor-failure resilience).
- Per-severity routing: `page` → on-call rotation; `ticket` → Jira via incident-management µservice; `silent` → Slack/Discord per team preference.
- AlertManager configuration lives at `microservices/observability/iac/helm/observability/templates/alertmanager-config.yaml`.

### Stage 5 — SLO authoring: OpenSLO v1alpha → sloth → Prometheus burn-rate rules

- **OpenSLO** v1alpha is the canonical SLO authoring source. Per ADR-0130 + ADR-0180-slo-composition-inheritance-arithmetic, every µservice ships OpenSLO YAML at `microservices/<ms>/slos/*.openslo.yaml`.
- **sloth** compiles OpenSLO sources to Prometheus burn-rate rules using ADR-0139's 4-window alerting (1h fast / 6h medium / 1d slow / 3d trickle).
- The generated PrometheusRule CRs land at `microservices/<ms>/iac/helm/<ms>/templates/slo-burn-rate-rules.yaml`.
- Sloth runs in CI per ADR-0098 LTS-rotation cadence; SLO drift between OpenSLO source and emitted PrometheusRule fails the build.

### Self-monitoring (second-tier observability)

The observability µservice's own SLOs are scraped by a **second-tier federated Prometheus** in a separate `cell-meta` namespace. This prevents the failure mode where the observability backplane fails silently because it's monitoring itself. Stage 1-4 of the cell-meta tier use the same components; storage is bounded retention; no further nesting (cell-meta does not monitor itself — accepted single-point loss).

## Alternatives considered

### (a) Datadog / New Relic SaaS — REJECTED

- **Pros:** managed; rich UI; integrated alerting.
- **Cons:** vendor lock-in; SaaS sends fleet telemetry off-cluster; cost scales unpredictably with cardinality; conflicts with oyatie's open-standard primitive doctrine; conflicts with sovereign-pack data-residency requirements (data must stay within pack-jurisdictional cells).
- **Rejected**: lock-in + sovereignty.

### (b) Elastic Observability (ELK + APM) — REJECTED

- **Pros:** mature; integrated.
- **Cons:** Elasticsearch licensing churn (SSPL since 2021; re-OSS'd 2024 under AGPL3 + SSPL + ELv2 tri-license); AGPL3 server-side-network-clause obligations don't align with permissive-license preference. Logstash/Beats ingestion is heavier than OpenTelemetry agent.
- **Rejected**: licensing complexity.

### (c) Shared backend for all signal types (e.g. one OpenSearch cluster) — REJECTED

- **Pros:** single storage tier.
- **Cons:** query patterns + retention + cost-per-signal-type are fundamentally different; mixing them in one backend optimizes for none. Hyperscaler practice is specialized backends per signal type.
- **Rejected**: anti-pattern.

### (d) Per-µservice ad-hoc collection (each µservice picks its own collector + backend) — REJECTED

- **Pros:** maximum µservice autonomy.
- **Cons:** config sprawl; no canonical pipeline; cross-µservice correlation impossible without a unified collection + storage path; SLO authoring fragments.
- **Rejected**: cannot meet consistency invariant.

### (e) **CHOSEN: LGTM stack with OpenTelemetry Collector + sloth compiler**

- **Pros:**
  - Specialized backend per signal type.
  - OpenTelemetry Collector is the open standard; OTLP is the open ingest protocol.
  - All five backends are open-source + permissive-license (LGTM is Apache 2.0; Prometheus is Apache 2.0).
  - Grafana single pane of glass.
  - OpenSLO + sloth keeps SLO source declarative + compiled.
  - Multi-vendor alert routing (PagerDuty + Opsgenie) for vendor-failure resilience.
- **Cons:** five components + collector to operate. Mitigation: each component has a Helm chart canonical in `microservices/observability/iac/helm/`; ops-sre-reliability operates the backplane as a substrate service.
- **Accepted**.

## Consequences

### Positive

1. **Specialized backend per signal type.** Each backend purpose-built; query + retention + cost-per-signal-type all separately optimized.
2. **OpenTelemetry Collector is the single ingest protocol.** No per-µservice exporter sprawl.
3. **Grafana single pane of glass.** All backend queries unified at Stage 3.
4. **Open-standard primitives.** LGTM (Apache 2.0) + Prometheus (Apache 2.0) + OpenTelemetry (Apache 2.0) + OpenSLO + sloth (open).
5. **OpenSLO + sloth keeps SLO authoring declarative.** Drift between intent and emitted PrometheusRule fails CI.
6. **Self-monitoring via second-tier federated Prometheus.** Observability backplane's own SLOs are scraped by a separate tier.

### Negative

1. **Five components to operate.** Mitigation: each component has a Helm chart canonical in `microservices/observability/iac/helm/`; ops-sre-reliability operates the backplane as a substrate service; CNCF-graduated components have aligned operator-skill profiles.
2. **Cross-cell trace correlation requires Tempo multi-tenant federation.** Mitigation: per-cell Tempo with cross-cell federation via Tempo's native multi-tenant query routing.

### Operational

1. The `observability` µservice's Helm charts:
   - `otel-collector-agent/` — per-node DaemonSet.
   - `otel-collector-gateway/` — gateway Deployment.
   - `prometheus/` — Prometheus 3.12 (LTS 3.5).
   - `mimir/` — Mimir 3.0.
   - `loki/` — Loki 3.4.
   - `tempo/` — Tempo latest 2.x.
   - `grafana/` — Grafana 12.x.
   - `alertmanager/` — AlertManager with PagerDuty + Opsgenie webhooks.
2. Per-µservice OpenSLO sources live at `microservices/<ms>/slos/*.openslo.yaml`. The `oya-governance-slo-coverage` lane (existing) validates the source-to-emitted-PrometheusRule mapping.
3. Alert webhook URLs come from OpenBao secrets at `secret/observability/pagerduty-webhook-url` + `secret/observability/opsgenie-webhook-url`.
4. Hubble (Cilium) flow observability ships via OTel exporter to the gateway Collector at Stage 1; ztunnel (Istio Ambient) telemetry ships via OTel; waypoint Envoy access logs ship via the OTel logs receiver.
5. Per-tenant + per-cell labels are added at Stage 1 (gateway Collector processor) for downstream slicing.

## In-house roadmap

Per user directive 2026-05-18 (in-house-stack policy), this ADR's observability components classify as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **OpenTelemetry Collector** | KEEP (Apache 2.0; CNCF Graduated) | THE standard collector + OTLP protocol; Google Cloud, AWS, Azure all support OTLP-native ingestion. | None planned. Adapter at `crates/oya-shared-otel-collector-config-kernel` wraps Collector config for theoretical swap. |
| **Grafana Alloy** (alt Collector distribution) | KEEP (Apache 2.0; Grafana Labs) | Binary-compatible OTel Collector distribution; permitted alt for environments where Grafana operator skill is anchor. | None planned. |
| **Prometheus 3.12 / LTS 3.5** | KEEP (Apache 2.0; CNCF Graduated 2018) | THE metric standard. AWS Managed Prometheus, Google Cloud Managed Service for Prometheus, Azure Managed Prometheus — all use upstream Prometheus. | None planned. |
| **Grafana Mimir 3.0** | KEEP (AGPL3 community edition; CNCF + Grafana Labs) | THE long-retention horizontally-scalable Prometheus-compatible store. AGPL3 with network-clause; oyatie's deployment is self-hosted within cells (server-network-clause obligations satisfied by the open-source distribution). | None planned. |
| **Grafana Loki 3.4** | KEEP (AGPL3 community edition) | THE log store paired with OTLP-native ingestion. | None planned. |
| **Grafana Tempo (2.x)** | KEEP (AGPL3 community edition) | THE trace store paired with OTLP. | None planned. |
| **Grafana 12.x** | KEEP (AGPL3 community edition) | THE query/visualization pane. | None planned. |
| **AlertManager** | KEEP (Apache 2.0; Prometheus subproject) | THE alert routing standard. | None planned. |
| **OpenSLO v1alpha** | KEEP (Apache 2.0; Linux Foundation) | THE SLO authoring standard. | None planned — oyatie's SLO sources ARE oyatie's in-house value running on the standard schema. |
| **Sloth** | KEEP (Apache 2.0; community) | THE OpenSLO → Prometheus burn-rate compiler. | None planned. |
| **PagerDuty + Opsgenie** (alert sinks) | Vendor-coupled at the alert egress | Both consumed via webhook (vendor-neutral protocol) at the AlertManager edge. Multi-vendor on purpose for vendor-failure resilience. | No in-house replacement. The webhook protocol IS the abstraction; vendor switch is a one-line AlertManager config change. |

The Grafana Labs AGPL3 license cluster (Mimir / Loki / Tempo / Grafana) is acceptable under oyatie's open-standard doctrine because (1) it is OSS (AGPL3 IS an OSI-approved open-source license), (2) oyatie's deployment is self-hosted within cells (no commercial SaaS exposure), (3) the AGPL3 network-clause obligations are satisfied by the open-source distribution license bundled with the Helm chart. The licensing direction is the opposite of Redis's drift to non-OSS.

The IS-the-standard pattern: oyatie's in-house engineering effort goes into **per-µservice OpenSLO sources, dashboards-as-code, alert-routing webhooks, and the second-tier federated self-monitoring Prometheus** — all Oya-native — running on KEEP-classified standard backends. This is exactly how AWS Managed Prometheus + Azure Managed Prometheus + Google Cloud Managed Service for Prometheus are built: standard upstream Prometheus, in-house operator + product surface.

Why no in-house metric/log/trace store: rebuilding Prometheus / Loki / Tempo from scratch would reimplement projects with thousands of production-shipped deployments + multi-vendor managed services + decades of cumulative engineering. The engineering cost would not produce a better outcome. The Helm-chart-based self-hosted deployment IS oyatie's in-house value (data residency, cell-tier scaling, federation topology, runbook discipline).

## Rollback

Each stage rolls back independently:

- **Stage 1 (Collector) rollback:** drop the OTel Collector Helm release; per-µservice tracing-client-kernel reverts to local file logging temporarily. Mesh remains up; signals stop flowing.
- **Stage 2 (storage) rollback:** per-backend Helm rollback; retention windows shrink; cross-cell federation degrades gracefully.
- **Stage 3 (Grafana) rollback:** drop the Grafana Helm release; storage backends still serve direct queries (curl / Loki LogQL / etc.).
- **Stage 4 (alerting) rollback:** drop AlertManager; Prometheus continues evaluating rules but no routing fires; manual triage required during the rollback window.
- **Stage 5 (SLO authoring) rollback:** revert OpenSLO source; sloth regenerates Prometheus rules on next CI run.

`git revert` + Flux reconciliation handles each stage. No persisted state is lost outside per-backend retention windows.

## References

- OpenTelemetry — https://opentelemetry.io/ ; Apache 2.0; CNCF Graduated.
- Grafana Alloy (OTel Collector distribution) — https://grafana.com/docs/alloy/
- Prometheus — https://prometheus.io/ ; current 3.12 (latest), LTS 3.5 through July 2026; Apache 2.0.
- Grafana Mimir — https://grafana.com/oss/mimir/ ; current 3.0; AGPL3 (Mimir community edition).
- Grafana Loki — https://grafana.com/oss/loki/ ; current 3.4.
- Grafana Tempo — https://grafana.com/oss/tempo/ ; current 2.x (post-MCP-server release).
- Grafana — https://grafana.com/ ; current 12.x.
- AlertManager — Prometheus alert routing.
- OpenSLO — https://openslo.com/ ; v1alpha spec.
- Sloth — https://sloth.dev/ ; SLO compiler.
- PagerDuty webhook integration — https://www.pagerduty.com/
- Opsgenie webhook integration — https://www.atlassian.com/software/opsgenie
- ADR-0130 — agentic SLO-gated promotion (OpenSLO mandatory authoring).
- ADR-0139 — 4-window burn-rate SLO alerting.
- ADR-0145 — inter-microservice communication reform (Invariant 2 traces).
- ADR-0148 — service-mesh canonical (Hubble + ztunnel + waypoint signals flow into this backplane).
- ADR-0153 — observability backplane high-level reference.
- ADR-0180-slo-composition-inheritance-arithmetic — SLO inheritance arithmetic.
- ADR-0182 — API gateway (gateway access logs flow into this backplane).
- ADR-0184 — storage tier layering (Postgres / Valkey / Meilisearch observability flows in).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
