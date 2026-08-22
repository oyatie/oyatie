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
    verified_by: "presubmit"
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

## Decision D-5 — AGPL boundary in observability: split by plane (2026-08-19)

**This clause AMENDS the product-licence policy carried by [ADR-0705](ADR-0705-product-protocol-live-apex.md)**
(ADR-0013's three-tier table and its dev-only carve-out). It does not create a second licence
authority: ADR-0705 remains where product-licence tiers live, and this clause supplies the one
thing that table cannot express — *where the product boundary runs inside a stack that is not
licence-homogeneous*. Read the two together.

### The stack is mixed, measured 2026-08-19 against upstream repository metadata

`observability/iac/helm/` holds **15 charts**. Licences below are the UPSTREAM projects', taken
from each repository's own licence field, not from chart names — every chart in that directory is
an oyatie-authored wrapper named `observability-*`, so the wrapper name says nothing about
what it deploys.

| Upstream | Licence |
|---|---|
| Grafana, Loki, Mimir, Pyroscope, **Tempo** | **AGPL-3.0** |
| Grafana OnCall | **AGPL-3.0**, and the upstream repository is **archived** |
| **Grafana Alloy** | **Apache-2.0** |
| Prometheus, Alertmanager, ClickHouse, OpenCost, OTel collector | Apache-2.0 |
| `statuspage`, `timescaledb-extension`, `axe-pa11y-runner` | oyatie-authored wrappers; upstream images carry their own licences and are not classified here |

Two corrections to the first draft of this clause are recorded rather than silently fixed, because
both were the kind of error this ADR exists to prevent. **Alloy was listed as AGPL-3.0 and is
Apache-2.0** — it is the fleet-wide collector, so the misclassification would have forced a
needless replacement of the one component that did not need one. **Tempo was omitted entirely**,
which left the trace pillar unassigned by a clause whose whole purpose is assigning pillars.

### The boundary is by plane, not by component

1. **Ops console — AGPL permitted.** Grafana and Pyroscope may serve internal SRE use. There is no
   distribution and no network provision to a third party, so AGPL §13 does not engage. This is
   the ADR-0705 dev-only carve-out **extended to ops-internal use**, and naming it as an extension
   is the point: it is a widening of an existing carve-out, not a new permission invented here.
2. **Tenant-facing product surface — Tier-1 permissive only**, per ADR-0705. No AGPL component may
   be embedded in, proxied by, or **queried by** a tenant-facing route.
3. **The data plane follows the product, not the console.** If a tenant-facing surface *queries*
   Mimir, Loki or **Tempo**, the AGPL program is provided over a network even though no tenant
   sees a Grafana page. So the stores backing the product are permissive — Prometheus/Thanos or
   VictoriaMetrics for metrics, ClickHouse or OpenSearch for logs, **and an equally permissive
   trace store where traces are tenant-visible** — with Grafana reading those same stores as an
   ops client.
4. **Grafana OnCall is a separate problem from its licence.** Its upstream repository is archived,
   so it fails the maintenance limb of the ADR-0709 D-6 quality bar regardless of plane. Do not
   resolve it by confining it to ops.

### Why not accept AGPL tenant-side

Modification risk is permanent, not one-off: the moment anyone patches Grafana or writes a plugin
that is a derivative work, oyatie owes *its* source, and that becomes a boundary someone must
police forever. AGPL prohibitions are routine in the procurement the `kr` / `eu` / `us-healthcare`
/ `ksa` / `uae` packs target. And it would contradict a posture this repository has already taken
twice — OpenBao instead of Vault, OpenTofu instead of Terraform.

### Enforcement, and the gap

The chart/image licence gate carries **two allow-lists keyed by plane**: tenant-facing admits the
ADR-0705 Tier-1 set only; ops-internal additionally admits AGPL-3.0. Every chart declares which
plane it serves, and an undeclared chart **fails closed**. A route-level check asserts that no
tenant-facing ingress path resolves to an ops-internal workload.

**That gate does not exist.** Licence policy today reaches Cargo crates only — and `deny.toml`
itself is read by nothing on the merge path. Tracked as `oyatie-f2fg`.

### Not claimed

**Nothing here asserts what is or is not deployed.** Cluster readback is unavailable. What the
tree shows is narrower and is all that is claimed: the `observability` Argo Application's declared
source path does not exist on `origin/dev`, so no desired state is rendered from it. Whether a
workload survives from an earlier sync is unknown, and this clause does not depend on the answer.

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
