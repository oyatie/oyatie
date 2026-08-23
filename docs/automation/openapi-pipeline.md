---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Render every `contracts/openapi/*.yaml` to both Redoc (reference) and Swagger UI
  (interactive) views in the mdbook; cross-validate runtime bindings against
  schema bindings; treat OpenAPI 3.1 YAML as the single source of truth for every
  HTTP-shaped Oyatie API surface.
planned_enforcement_ref: governance-openapi-publish
extends_crates:
  - intelligence-openapi-kernel
  - intelligence-mdbook-kernel
  - intelligence-api-semver-kernel
companion_docs:
  - INDEX.md
  - rustdoc-pipeline.md
doc_status: published
---

# Pipeline: OpenAPI auto-publish

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

Treat every `contracts/openapi/*.yaml` as the sole source of truth for the corresponding HTTP surface: render reference (Redoc) and interactive (Swagger UI) views, generate runtime bindings (Rust `axum` + TS `fetch` clients), generate schema bindings (Rust `serde` types), and cross-validate runtime against schema. The kernel refuses divergence.

## 2. Inputs

- `contracts/openapi/<surface>.yaml` (OpenAPI 3.1 only; 3.0 is rejected with `MUST migrate` error).
- Per-surface `contracts/openapi/<surface>.metadata.toml` (owner_axis, consumer_axes, semver-policy, deprecation-window).
- Workspace crate manifests claiming a surface via `[package.metadata.oyatie.openapi-surface] = "<surface>"`.

## 3. Outputs

- `docs/site/src/api/openapi/<surface>/redoc.md` (rendered reference; auto-generated header).
- `docs/site/src/api/openapi/<surface>/swagger.md` (interactive viewer embed).
- `docs/site/src/api/openapi/<surface>/changelog.md` (per-version diff lifted from `intelligence-api-semver-kernel`).
- `crates/<surface>-api/src/generated/<surface>_runtime.rs` (axum routes + handler signatures; checked-in, regenerated on contract change).
- `crates/<surface>-api/src/generated/<surface>_schema.rs` (serde-derived request/response types).
- `clients/typescript/<surface>/src/generated/*.ts` (fetch client + types).
- JSON sidecar `docs/site/src/api/openapi/_index.json`.

## 4. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching `contracts/openapi/**` | Regenerate runtime + schema bindings + Redoc/Swagger pages; semver-kernel runs diff. |
| Per-PR touching `crates/*-api/**` | Bindings drift check (handler signature must match generated). |
| Nightly | Full regeneration; full cross-validation across surfaces; orphan-surface detection. |

## 5. Validation gates (`governance-openapi-publish`)

1. **3.1-only.** Any spec declaring `openapi: 3.0.x` fails immediately (BLOCKER).
2. **Runtime ↔ schema parity.** For every operation in the spec, the generated runtime route exists and the handler signature accepts the generated schema type (BLOCKER).
3. **Hand-written generated drift.** Any file under `*/generated/**` whose hash differs from the regenerated artifact (BLOCKER). The pipeline writes; humans do not.
4. **Semver gate.** Breaking changes (per `intelligence-api-semver-kernel`) without an explicit `x-oyatie-breaking-change: <ADR-id>` extension (BLOCKER).
5. **Consumer-axis declaration.** Every surface declares `consumer_axes:` in its metadata.toml; absent = HIGH.
6. **Redoc/Swagger render.** Generated mdbook pages pass `intelligence-mdbook-kernel::validate_mdbook_source`.

## 6. Cross-binding parity algorithm

For each operation `(method, path)` in the spec:
1. Parse spec → canonical `(method, path, request_schema, response_schemas, params)`.
2. Parse generated runtime → routes registered + handler signature.
3. Parse implementing crate (via cargo-expand or `proc-macro2` scan) → handler symbol exists, takes generated request type, returns one of the generated response types.
4. Mismatch on any step → `OpenApiParityError { surface, operation, reason }`.

## 7. Deprecation-window enforcement

`metadata.toml::deprecation-window-days` (default 180). Any operation marked `deprecated: true` in the spec must have a `x-oyatie-deprecated-at: <ISO date>` extension; if `now - deprecated-at > window`, the lane raises BLOCKER demanding removal or extension via ADR.

## 8. Out-of-scope

- gRPC / connect surfaces (separate `proto-pipeline.md` once `intelligence-proto-kernel` lands).
- WebSocket / SSE protocols (tracked under `intelligence-eventing-protocols-kernel`).
- Internal cross-crate Rust APIs (covered by `rustdoc-pipeline.md`).
