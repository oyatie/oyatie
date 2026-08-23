---
doc_class: VisualizationSpec
shape: visualization
length_cap: 200
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Render every `EVT-*` audit-chain topic plus its emitter crate and its consumer
  crates. Source: per-crate `[package.metadata.oyatie.events]` blocks +
  `contracts/eventschema/*.yaml` + the doc-catalog event taxonomy.
  Lift to `docs/visualization/audit-chain.md` (D2 + Mermaid).
planned_enforcement_ref: governance-audit-chain-map
extends_crates:
  - governance-cohesion-fitness-kernel
  - intelligence-evidence-kernel
  - intelligence-architecture-map-kernel
companion_docs:
  - INDEX.md
  - architecture-map-kernel-spec.md
  - ../../docs/decisions/ADR-0709-general-live-apex.md
doc_status: published
---

# Visualization spec: audit-chain map

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

ADR-0003 (audit-chain and evidence emission) makes every regulated invocation auditable. ADR-0005 (eventing-backbone outbox-pattern) defines how events are emitted. The doc-catalog maintains the `EVT-*` event taxonomy. None of this is visible at-a-glance; today an engineer must read three docs to know who emits `EVT-AXIS-CONTRACT-CHANGE` and who consumes it. This spec renders the full map.

## 2. Inputs

- Every `crates/**/Cargo.toml` `[package.metadata.oyatie.events]` block:

```toml
[package.metadata.oyatie.events]
emits = ["EVT-AUDIT-CHAIN-WRITTEN", "EVT-EVIDENCE-RECORDED"]
consumes = ["EVT-CAPABILITY-INVOKED"]
```

- Every `contracts/eventschema/EVT-*.yaml` event schema (one file per topic).
- The doc-catalog event list (`docs/DOC-CATALOG.md` §1 `Update-triggering events` table).
- The audit-chain canonical store path (per ADR-0003).

## 3. Per-event-topic record shape

```yaml
event_id: EVT-CAPABILITY-INVOKED
schema_path: contracts/eventschema/EVT-CAPABILITY-INVOKED.yaml
emitter_crates:
  - intelligence-policy-kernel
  - intelligence-capability-kernel
consumer_crates:
  - intelligence-evidence-kernel
  - platform-audit-store-kernel
retention_days: 2555  # 7 years
data_class: CUSTOMER_CONFIDENTIAL
regulatory_scope: ["KR-PIPC", "KR-FSC"]
```

## 4. Output rendering

### 4.1 Primary: D2 (richer event-topology layout)

```d2
direction: right
emitters: {
  foundry-policy-kernel: {shape: rectangle}
  foundry-capability-kernel: {shape: rectangle}
}
topics: {
  EVT-CAPABILITY-INVOKED: {shape: queue; style.fill: orange}
  EVT-AUDIT-CHAIN-WRITTEN: {shape: queue; style.fill: orange}
}
consumers: {
  foundry-evidence-kernel: {shape: rectangle}
  audit-store-kernel: {shape: cylinder}
}
emitters.foundry-policy-kernel -> topics.EVT-CAPABILITY-INVOKED
emitters.foundry-capability-kernel -> topics.EVT-CAPABILITY-INVOKED
topics.EVT-CAPABILITY-INVOKED -> consumers.foundry-evidence-kernel
consumers.foundry-evidence-kernel -> topics.EVT-AUDIT-CHAIN-WRITTEN
topics.EVT-AUDIT-CHAIN-WRITTEN -> consumers.audit-store-kernel
```

Topics are queues; emitters point in, consumers point out. The chain visualizes the outbox→consumer flow per ADR-0005.

### 4.2 Secondary: Mermaid (mdbook-inline)

The equivalent Mermaid `graph LR` is emitted alongside for inline reading.

### 4.3 Per-topic detail page

For each `EVT-*` topic, a per-topic mdbook page `docs/site/src/visualization/audit-chain/<topic>.md` containing:
- The schema (rendered from `contracts/eventschema/<topic>.yaml`).
- The emitter list + consumer list.
- Retention + data-class + regulatory-scope.
- A worked example payload from `contracts/eventschema/<topic>.example.yaml`.

## 5. Validation gates (`governance-audit-chain-map`)

1. **Topic schema presence.** Every emitted topic has a schema at `contracts/eventschema/<topic>.yaml` (BLOCKER).
2. **Emitter ↔ schema parity.** The emitter crate's serialization signature matches the schema (BLOCKER; cross-fed from `schema-doc-pipeline.md`).
3. **Consumer reachability.** Every emitted topic has at least one declared consumer (HIGH; orphan events are leakage candidates).
4. **Doc-catalog parity.** Every `EVT-*` in `docs/DOC-CATALOG.md` §1 has a schema and ≥ 1 emitter (BLOCKER).
5. **Regulatory-scope coverage.** Topics tagged regulated (PIPC/FSC/MFDS/KISA/NIS/KCC) MUST have `retention_days` ≥ regulator floor (BLOCKER; KR-PIPC = 1095 days, KR-FSC = 1825 days per `docs/COMPLIANCE-MATRIX.md`).
6. **Generated drift.** Committed audit-chain map differs from re-rendered (BLOCKER).

## 6. Cross-tenant-leakage check

The pipeline statically inspects every consumer's event-handler signature: a consumer reading `EVT-*` topics tagged `CUSTOMER_CONFIDENTIAL` or stricter MUST extract `tenant_id` from the event envelope and pass it to every downstream call (verified via dataflow on the handler's `&TenantContext` parameter). Failure to thread tenant context → BLOCKER (cross-tenant-leakage risk).

## 7. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching any `[package.metadata.oyatie.events]` block or `contracts/eventschema/**` | Re-render; lane runs. |
| Nightly | Full re-render; orphan-topic sweep. |
| On regulatory update | Re-validate retention floors. |

## 8. Cross-references

- `architecture-map-kernel-spec.md` consumes the event graph as one of its sources.
- `intelligence-evidence-kernel` consumes the topic list at runtime to wire emitters.
- `docs/adr-archive/ADR-0003-audit-chain-and-evidence-emission.md is the constitutional authority.

## 9. Out-of-scope

- Live event throughput dashboards (covered by `cloud-observability-*`).
- Per-tenant audit-chain export (covered by `platform-audit-store-kernel`).
- Per-region event-routing topology (covered by future `cell-event-routing-spec.md`).
