---
id: ADR-0383
status: Superseded
planning_impact: false
deciders: founder, council-architecture, ops-platform
date: 2026-05-28
owner: ops-platform
supersedes: [ADR-0042]
superseded_by: [ADR-700]
related: [ADR-0186, ADR-0173, ADR-0211, ADR-0349]
related_specs: []
door: two-way
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0383 — Observability stack reconciliation: keep Loki / Tempo / Mimir / Grafana under AGPL-3

## Status

Accepted (2026-05-28). Supersedes ADR-0042.

## Context

Two accepted ADRs contradict each other on the permissibility of the Grafana Labs AGPL-3 stack
(Loki / Tempo / Mimir / Grafana):

**ADR-0042** (Observability stack — OTel + in-house UI, dated 2026-05-09, status: Proposed)
explicitly **forbids** Loki, Tempo, Mimir, and Grafana on the grounds that their 2024 AGPL-3
relicensing is "forbidden in our product surface per License Policy". ADR-0042 mandates
VictoriaMetrics (metrics), ClickHouse (logs), Jaeger (traces), and a long-horizon in-house Leptos
portal.

**ADR-0186** (Observability backplane layering, dated 2026-05-18, status: Accepted) **keeps**
Mimir 3.0, Loki 3.4, Tempo 2.x, and Grafana 12.x. ADR-0186 justifies this under the reasoning
that AGPL-3 is an OSI-approved open-source license and that oyatie's fully self-hosted deployment
within oya-cells satisfies the AGPL-3 network-clause obligations by virtue of the open-source
distribution licence bundled with the Helm chart. ADR-0186 was authored nine days after ADR-0042,
supersedes its storage-tier choice, and is the more concretely-architected of the two.

ADR-0211 §35 (in-house tech stack policy) independently codifies the same exception: "AGPL3 only
if self-hosted with network clause satisfied — see ADR-0186 Grafana cluster."

The contradiction must be resolved before PR #260 (observability AGPL-3 Helm stack) can merge.
ADR-0186 is the canonical decision; this ADR makes the resolution explicit and retires ADR-0042 as
superseded.

## Decision

**KEEP** the Grafana Labs Loki / Tempo / Mimir / Grafana stack as the canonical observability
storage and visualization layer, subject to all three of the following gates:

1. **Fully self-hosted in oya-cells.** Every Grafana Labs component runs inside an oyatie-operated
   cell. No traffic is routed to Grafana Cloud SaaS. No managed-service dependency on Grafana Labs
   infrastructure is introduced. The ArgoCD/dev-cell entrypoint is the umbrella chart at
   `microservices/observability/iac/k8s/helm/`; component chart modules and production value
   fragments remain under `microservices/observability/iac/helm/`. No Grafana Cloud or external
   SaaS deployment surface is permitted.

2. **Network clause satisfied by self-host posture (per ADR-0211 §35).** The AGPL-3 "network
   clause" (§13) requires that modified source be made available to users who interact with the
   software over a network. Oyatie's deployment serves only its own tenants from within its own
   cells using the unmodified upstream open-source Helm distribution. No modifications to Grafana
   Labs source code are made. This satisfies the clause under the same analysis that governs every
   major cloud provider's self-hosted AGPL-3 workload.

3. **Per-component lifecycle owned by ops-platform.** Each Grafana Labs component (Loki, Tempo,
   Mimir, Grafana) is a Class A dependency under ADR-0173 and ADR-0211. The ops-platform team
   owns version pinning, security-patch cadence, and the 30-day re-classification trigger if any
   component changes license terms (per ADR-0211 §181). Current pins: Mimir 3.0, Loki 3.4, Tempo
   2.x (post-MCP-server release), Grafana 12.x (per ADR-0186 table, reviewed 2026-05-18).

The ADR-0042 storage-tier alternatives (VictoriaMetrics, ClickHouse, Jaeger, in-house Leptos
portal) are **retired** as the canonical observability storage choices. The OTel SDK instrumentation
surface, per-cell observability namespace, per-tenant cost-attribution dashboards, and gen_ai
semantic conventions mandated by ADR-0042 remain valid and are carried forward by ADR-0186's
five-stage backplane.

## Consequences

- **ADR-0042 is superseded by this ADR.** It must not be cited as the authority on observability
  storage-tier choices. ADR-0186 is the canonical backplane architecture; ADR-0383 is the canonical
  license-reconciliation record.
- **PR #260 (observability AGPL-3 Helm stack) is unblocked.** The license contradiction that
  would have prevented its merge is resolved, provided the umbrella chart stays self-hosted,
  carries tenant controls, and fails closed for production while any dev-only storage backend is
  configured.
- **License-class annotation in PR #260's Chart.yaml** must read `mixed-internal+agpl3-self-hosted`
  (not `internal`). This reflects the accurate license posture: the chart wraps AGPL-3 upstream
  components deployed under the self-hosted exception.
- **Production promotion is separate from dev-cell rendering.** Dev-cell values may use local
  filesystem storage only when `config.environment != production`; production renders must refuse
  Loki test schema/filesystem storage, Tempo local backend, and Mimir filesystem blocks storage.

## Deliverables

- **D1** — Annotate the observability Helm chart(s) under
  `microservices/observability/iac/{k8s/helm,helm/}` with references to ADR-0186 and ADR-0383 in a
  `annotations:` block or chart `NOTES.txt`. Exit criteria: the ArgoCD umbrella chart carries both
  ADR references post-merge.
- **D2** — Correct the license-class annotation in PR #260's `Chart.yaml` from `internal` to
  `mixed-internal+agpl3-self-hosted`. Exit criteria: `grep license-class` in the chart returns
  `mixed-internal+agpl3-self-hosted`.

## Alternatives considered

### (a) Retain ADR-0042; reject AGPL-3 stack — REJECTED

This would require rewriting the observability backplane (already partially implemented per
ADR-0186) to use VictoriaMetrics + ClickHouse + Jaeger. The cost is high; ADR-0186 was accepted
by the full council; the in-house Leptos portal is W+24 horizon. ADR-0211 §35 already blessed the
AGPL-3 exception for self-hosted components. Rejecting it creates architectural churn without
license risk reduction.

### (b) Commercial Grafana Enterprise licensing — REJECTED

ADR-0042 proposed this as a fallback. It introduces a perpetual commercial dependency, per-seat
pricing risk, and is inconsistent with the sovereign-pack open-source doctrine. The self-hosted
AGPL-3 path is cleaner.

### (c) Partial keep: Grafana only; replace Loki/Tempo/Mimir — REJECTED

This fragments the LGTM stack without benefit. Loki, Tempo, and Mimir are purpose-built for OTLP
ingestion alongside Grafana and share the same operator-skill profile. Replacing them with
ClickHouse + Jaeger + VictoriaMetrics would require three additional bespoke Grafana data-source
plugins and lose native OTLP ingestion.

## References

- ADR-0042 — original observability stack decision (superseded by this ADR)
- ADR-0186 — observability backplane layering (canonical architecture; supersedes ADR-0042 storage
  choices)
- ADR-0173 — vendor lock-in avoidance and stack ownership (Class A/B/C dependency classification)
- ADR-0211 §35 — in-house tech stack policy (AGPL-3 self-hosted exception)
- ADR-0349 — Jenkins + ArgoCD self-hostable CI/CD substrate (deployment pipeline for the
  observability backplane Helm releases)
- Grafana Mimir — https://grafana.com/oss/mimir/ ; AGPL3 community edition
- Grafana Loki — https://grafana.com/oss/loki/ ; AGPL3 community edition
- Grafana Tempo — https://grafana.com/oss/tempo/ ; AGPL3 community edition
- Grafana — https://grafana.com/ ; AGPL3 community edition
- AGPL-3 full text — https://www.gnu.org/licenses/agpl-3.0.html (§13 network clause)

## Historical residual from ADR-42 (E3 fold 2026-08-06)

**Title:** ADR-0042-observability-stack-otel-and-in-house-ui

**Preserved decision gist:** We adopt **OpenTelemetry SDK** as the canonical instrumentation surface; **VictoriaMetrics** (Apache-2) as the metrics storage; **structured JSON logs via the `tracing` crate** for log emission; **per-cell observability namespace** for isolation; **per-tenant cost-attribution dashboards** at the FinOps layer; **per-capability gen_ai semantic conventions** for AI/agent telemetry; an **in-house Leptos observability portal** long-horizon; **commercial Grafana Enterprise licensing** as the Phase-1 / Phase-2 fallback if in-house portal is not ready by GA. ### OpenTelemetry SDK (Apache-2) ```rust //

_Source file archived after fold; full body in git history / docs/adr-archive/._
