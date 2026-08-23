---
doc_class: VisualizationSpec
shape: visualization
length_cap: 200
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Render every workspace crate, every cross-crate dependency, and every public
  API surface as a layered DAG (kernel → runtime → adapter → app). Source: Cargo
  workspace metadata + `governance-cohesion-fitness-kernel` contract records +
  rustdoc link graph. Lift to `docs/visualization/service-map.md` as D2 + SVG.
planned_enforcement_ref: governance-service-map
extends_crates:
  - governance-cohesion-fitness-kernel
  - intelligence-architecture-map-kernel
  - intelligence-catalog-kernel
companion_docs:
  - INDEX.md
  - architecture-map-kernel-spec.md
  - tech-stack-map-spec.md
doc_status: published
---

# Visualization spec: service map

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

A finer-grained view than `product-map-spec.md`: not "what products", but "what crates, with what dependencies, on what layer." Service-map is the artifact a new engineer reads before touching the workspace, and the artifact ops engineers reference during incident response.

## 2. Inputs

- Workspace `Cargo.toml` `[workspace.members]`.
- Each crate's `Cargo.toml` `[package.metadata.oyatie]` block: `axis`, `layer ∈ {kernel, domain, usecase, app, adapter, infrastructure, cli, rest, grpc, worker, sdk, api}`, `role`.
- Each crate's `Cargo.toml` `[dependencies]` and `[dev-dependencies]` keys (filtered to workspace-internal crates only — external deps live in `tech-stack-map-spec.md`).
- The cross-crate link graph from `rustdoc-pipeline.md`.
- The cross-axis contract records from `governance-cohesion-fitness-kernel`.

## 3. Layer convention

| Layer | Role | Examples |
|---|---|---|
| `kernel` | Pure value-object; no I/O | `governance-cohesion-fitness-kernel`, `governance-runbook-freshness-kernel` |
| `domain` | Business invariants and port traits | `identity-domain` |
| `usecase` | Application/use-case orchestration over domain ports | `identity-usecase`, `audit-chain-usecase` |
| `app` | Deployable/composition root; composes usecases + adapters/surfaces; never imports another app | `foundation-app`, `cloud-billing-app` |
| `adapter` | Provider-specific I/O (storage, network, KMS, AI) | `intelligence-evidence-adapter-file`, `intelligence-run-adapter-file` |
| `rest` / `grpc` / `api` | External-surface handlers | `intelligence-rest`, `cloud-iam-api` |
| `worker` | Queue/scheduled entrypoints | `intelligence-ci-worker` |

Layer is declared in `[package.metadata.oyatie.layer]` / catalog `role`. The pipeline rejects crates without a catalog role. The active dependency rule is inward-only: `kernel <- domain <- usecase <- app`; `app -> app` is a blocker.

## 4. Output rendering

### 4.1 Primary: D2 (richer-layout-aware)

```d2
direction: down
kernel: {
  cohesion-fitness: {shape: cylinder}
  runbook-freshness: {shape: cylinder}
}
domain: {
  identity-domain: {shape: rectangle}
}
usecase: {
  identity-usecase: {shape: rectangle}
  subagent-runtime-usecase: {shape: rectangle}
}
adapter: {
  evidence-adapter-file: {shape: page}
  run-adapter-file: {shape: page}
}
api: {
  foundry-api: {shape: cloud}
}
app: {
  foundation-app: {shape: rectangle; style.bold: true}
}
app.foundation-app -> usecase.identity-usecase
usecase.identity-usecase -> domain.identity-domain
domain.identity-domain -> kernel.runbook-freshness
adapter.evidence-adapter-file -> kernel.cohesion-fitness: {style.stroke-dash: 2}
```

Layered top-down; cross-layer edges only flow downward (upstream-of) by design.

### 4.2 Secondary: Mermaid (inline mdbook)

For the mdbook chapter, the equivalent Mermaid `graph TB` is emitted alongside. D2 SVG is the canonical artifact; Mermaid is the inline-readable fallback.

### 4.3 Per-layer subviews

The pipeline also emits per-layer subviews `docs/visualization/service-map-<layer>.svg` for layer-specific deep-dives.

## 5. Validation gates (`governance-service-map`)

1. **Layer declaration.** Every workspace crate has catalog `role` plus `[package.metadata.oyatie.layer]` where present (BLOCKER).
2. **Downstream-only edges.** A `kernel` crate MUST NOT depend on outer layers; `domain` must not depend on `usecase`/`app`/adapters/surfaces; `usecase` must not depend on concrete adapters or apps; `app -> app` is a BLOCKER. The pipeline enforces strict inward dependency direction.
3. **Cycle ban.** Workspace-internal dependency cycles → BLOCKER (cargo already rejects, but the pipeline asserts the rendered DAG is acyclic for visual clarity).
4. **Catalog presence.** Every crate exists in the registry catalog (cross-validated via `intelligence-catalog-kernel`).
5. **Generated drift.** Committed service map differs from re-rendered (BLOCKER).
6. **SVG presence.** Every rendered diagram has a CI-produced SVG sibling.

## 6. Public-API annotation

For each `api` and `app` layer crate, the pipeline pulls the surface count from `rustdoc-pipeline.md`'s `_index.json` and renders it as a node label suffix: `foundry-api (84 pub items, 12 routes)`. This makes the relative surface size visible at a glance.

## 7. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching any `Cargo.toml` or `crates/**/src/lib.rs` | Re-render; lane runs. |
| Nightly | Full re-render; archive snapshot. |
| On layer-metadata change | Re-validate downstream-only-edges rule. |

## 8. Cross-references

- `architecture-map-kernel-spec.md` consumes this map's DAG to attribute cross-axis contracts.
- `tech-stack-map-spec.md` extends with external-dependency edges.
- `dependency-graph-spec.md` is the planning-tier analog (subplan/phase/IP DAG); this is the code-tier analog.

## 9. Out-of-scope

- Runtime call graph (covered by `audit-chain-map-spec.md` for event flow; runtime RPC graph is future).
- Per-version diff visualization (covered by `intelligence-api-semver-kernel`).
- Service ownership map (covered by `docs/RACI-OWNERSHIP.md`).
