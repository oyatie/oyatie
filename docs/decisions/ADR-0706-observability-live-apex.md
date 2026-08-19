---
doc_status: published
id: ADR-0706
title: "Live observability, SLO, and progressive-delivery telemetry"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0114, ADR-0180, ADR-0186, ADR-0210, ADR-0263]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0706-D1
    description: "Live apex source-of-truth for topic observability: Live observability, SLO, and progressive-delivery telemetry."
    exit_criteria: "docs/decisions/ADR-0706-observability-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0706: Live observability, SLO, and progressive-delivery telemetry

## Status

**Accepted** — live consolidated source-of-truth entry for topic `observability` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **5** Accepted ADRs in the `observability` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `observability` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Decision D-5 — AGPL boundary: split by plane, not by component (2026-08-19)

The observability stack is not licence-homogeneous, and until now nothing recorded which half
may face a tenant. `observability/iac/helm/` carries **grafana, loki, mimir, pyroscope, alloy and
oncall — all AGPLv3** since Grafana Labs relicensed in 2021. Its siblings in the same tree —
**prometheus, clickhouse, opencost, otel-tailsampling-collector, alertmanager — are Apache-2.0**.
`deny.toml`'s allow-list does not contain AGPL and never reached these anyway: it governs Cargo
dependencies, and these are charts.

The boundary is drawn by **plane**, not by component:

1. **Ops console — AGPL permitted.** Grafana, OnCall and Pyroscope may serve internal SRE use.
   There is no distribution and no network provision to a third party, so AGPL §13 does not
   engage, and these are the best tools for the job. Removing them would cost real capability for
   no risk reduction.

2. **Tenant-facing product surface — Apache-2.0 only.** No AGPL component may be embedded in,
   proxied by, or **queried by** a tenant-facing route: the console, the `observability`
   capability's public API, per-tenant rollups, tenant SLO dashboards.

3. **The data plane follows the product, not the console.** This is the clause that is easy to get
   wrong. If a tenant-facing surface **queries** Mimir or Loki, the AGPL program is being provided
   over a network even though no tenant ever sees a Grafana page. So the metrics and log stores
   backing the product are Apache-2.0 — Prometheus/Thanos or VictoriaMetrics for metrics,
   ClickHouse or OpenSearch for logs — and Grafana reads those same stores as an ops client.

**Why not simply accept AGPL tenant-side.** Modification risk is permanent rather than one-off:
the moment anyone patches Grafana or writes a plugin that is a derivative work, oyatie owes *its*
source, and that becomes a boundary someone must police forever. AGPL prohibitions are routine in
the enterprise and public-sector procurement the `kr` / `eu` / `us-healthcare` / `ksa` / `uae`
packs target. And it would contradict a posture this repository has already taken twice —
OpenBao instead of Vault, OpenTofu instead of Terraform, both forks chosen to avoid licence risk.
`.github/CONTRIBUTING.md` already states that AGPL is not permitted in product code; this clause
says where the product boundary runs.

**Enforcement, because a ruling nothing executes is the failure this clause exists to avoid.**
The chart/image licence gate carries **two allow-lists keyed by plane** — tenant-facing admits the
Apache-2.0 family only; ops-internal additionally admits AGPL-3.0. Every chart declares which
plane it serves, and an undeclared chart **fails closed**. A route-level check asserts that no
tenant-facing ingress path resolves to an ops-internal workload. That gate does not exist yet:
licence policy today reaches Cargo crates only, never charts or images.

**Cheap now, expensive later.** The `observability` Argo Application currently points at
`microservices/observability/iac/k8s/helm`, a path the reorg deleted, so **nothing is deployed
against this decision today**. Four Apache-2.0 charts are already in the same tree. After tenants
are on those dashboards the same change is a migration.

## Preserved member gists

- **ADR-114** (ADR-0114-canary-observability-rollback): A canary gate runs between every promotion event. The gate emits one of four verdicts: `PROMOTE`, `ROLLBACK`, `EXTEND_OBSERVATION`, `ESCALATE`. The verdict conditions whether the downstream promotion workflow advances. ### 1. Cohort selection (per-cell) Oyatie's cell architecture (per ADR-0033 + cell-domain crates) gives a natural canary mechanism:
- **ADR-180** (ADR-0180-slo-composition-inheritance-arithmetic): Oyatie declares **SLO composition arithmetic** as a first-class manifest concern. Every parent product (Workflow Studio, Intelligence, Super-App, etc.) declares its composition rule; every blocking child µservice's SLO is verified to satisfy the parent's budget. ### Composition rules 1. **Serial composition (call chain).** Parent SLO availability ≤ `pro
- **ADR-186** (ADR-0186-observability-backplane-layering): Oyatie adopts a **five-stage observability backplane** in which each stage owns one concern: ### Stage 1 — Collection: OpenTelemetry Collector (single binary) - **OpenTelemetry Collector** is the canonical collector across the fleet. Single binary; receivers + processors + exporters configured via OTLP. Deployed in two roles: - **Agent mode** as a 
- **ADR-210** (ADR-0210-otel-tail-sampling): ### Two-stage sampling **Stage A — Head sampling (per agent collector, DaemonSet):** - Default: **1% always-on baseline** at the per-µservice agent collector (per ADR-0186 Stage 1). - Configurable per µservice via `manifest.json` `observability.trace_sampling_recipe.head_bps` (basis points; default 100 = 1%). - Random sampling decision at the entry
- **ADR-263** (ADR-0263-observability-emission-contract): The fifteen decisions below collectively form the emission contract. Every µservice MUST honour every decision; the CI lanes listed in §Verification enforce. ### D-1. Three Pillars: metrics + logs + traces Every oyatie µservice emits **three signal streams**, each governed by the contract sections below: 1. **Metrics** — Prometheus exposition forma

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-263 residual

**ADR-0263-observability-emission-contract** — The fifteen decisions below collectively form the emission contract. Every µservice MUST honour every decision; the CI lanes listed in §Verification enforce. ### D-1. Three Pillars: metrics + logs + traces Every oyatie µservice emits **three signal streams**, each governed by the contract sections below: 1. **Metrics** — Prometheus exposition format (text 0.0.4 minimum; OpenMetrics 1.0.0 preferred

### ADR-180 residual

**ADR-0180-slo-composition-inheritance-arithmetic** — Oyatie declares **SLO composition arithmetic** as a first-class manifest concern. Every parent product (Workflow Studio, Intelligence, Super-App, etc.) declares its composition rule; every blocking child µservice's SLO is verified to satisfy the parent's budget. ### Composition rules 1. **Serial composition (call chain).** Parent SLO availability ≤ `product(children_in_chain.availability)`. 2. **Parall

### ADR-210 residual

**ADR-0210-otel-tail-sampling** — ### Two-stage sampling **Stage A — Head sampling (per agent collector, DaemonSet):** - Default: **1% always-on baseline** at the per-µservice agent collector (per ADR-0186 Stage 1). - Configurable per µservice via `manifest.json` `observability.trace_sampling_recipe.head_bps` (basis points; default 100 = 1%). - Random sampling decision at the entry point (root span); the decision propagates as a t

### ADR-114 residual

**ADR-0114-canary-observability-rollback** — A canary gate runs between every promotion event. The gate emits one of four verdicts: `PROMOTE`, `ROLLBACK`, `EXTEND_OBSERVATION`, `ESCALATE`. The verdict conditions whether the downstream promotion workflow advances. ### 1. Cohort selection (per-cell) Oyatie's cell architecture (per ADR-0033 + cell-domain crates) gives a natural canary mechanism: each cell is independent; different cells can pin

### ADR-186 residual

**ADR-0186-observability-backplane-layering** — Oyatie adopts a **five-stage observability backplane** in which each stage owns one concern: ### Stage 1 — Collection: OpenTelemetry Collector (single binary) - **OpenTelemetry Collector** is the canonical collector across the fleet. Single binary; receivers + processors + exporters configured via OTLP. Deployed in two roles: - **Agent mode** as a per-node DaemonSet — receives metrics/logs/traces 
