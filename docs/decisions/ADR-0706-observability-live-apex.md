---
id: ADR-706
title: "Live observability, SLO, and progressive-delivery telemetry"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-114, ADR-180, ADR-186, ADR-210, ADR-263]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
---
# ADR-706: Live observability, SLO, and progressive-delivery telemetry

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

## Preserved member gists

- **ADR-114** (ADR-0114-canary-observability-rollback): A canary gate runs between every promotion event. The gate emits one of four verdicts: `PROMOTE`, `ROLLBACK`, `EXTEND_OBSERVATION`, `ESCALATE`. The verdict conditions whether the downstream promotion workflow advances. ### 1. Cohort selection (per-cell) Oyatie's cell architecture (per ADR-0033 + cell-domain crates) gives a natural canary mechanism:
- **ADR-180** (ADR-0180-slo-composition-inheritance-arithmetic): Oyatie declares **SLO composition arithmetic** as a first-class manifest concern. Every parent product (Workflow Studio, Foundry, Super-App, etc.) declares its composition rule; every blocking child µservice's SLO is verified to satisfy the parent's budget. ### Composition rules 1. **Serial composition (call chain).** Parent SLO availability ≤ `pro
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

**ADR-0180-slo-composition-inheritance-arithmetic** — Oyatie declares **SLO composition arithmetic** as a first-class manifest concern. Every parent product (Workflow Studio, Foundry, Super-App, etc.) declares its composition rule; every blocking child µservice's SLO is verified to satisfy the parent's budget. ### Composition rules 1. **Serial composition (call chain).** Parent SLO availability ≤ `product(children_in_chain.availability)`. 2. **Parall

### ADR-210 residual

**ADR-0210-otel-tail-sampling** — ### Two-stage sampling **Stage A — Head sampling (per agent collector, DaemonSet):** - Default: **1% always-on baseline** at the per-µservice agent collector (per ADR-0186 Stage 1). - Configurable per µservice via `manifest.json` `observability.trace_sampling_recipe.head_bps` (basis points; default 100 = 1%). - Random sampling decision at the entry point (root span); the decision propagates as a t

### ADR-114 residual

**ADR-0114-canary-observability-rollback** — A canary gate runs between every promotion event. The gate emits one of four verdicts: `PROMOTE`, `ROLLBACK`, `EXTEND_OBSERVATION`, `ESCALATE`. The verdict conditions whether the downstream promotion workflow advances. ### 1. Cohort selection (per-cell) Oyatie's cell architecture (per ADR-0033 + cell-domain crates) gives a natural canary mechanism: each cell is independent; different cells can pin

### ADR-186 residual

**ADR-0186-observability-backplane-layering** — Oyatie adopts a **five-stage observability backplane** in which each stage owns one concern: ### Stage 1 — Collection: OpenTelemetry Collector (single binary) - **OpenTelemetry Collector** is the canonical collector across the fleet. Single binary; receivers + processors + exporters configured via OTLP. Deployed in two roles: - **Agent mode** as a per-node DaemonSet — receives metrics/logs/traces 
