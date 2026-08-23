---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Publish every `///` rustdoc comment in every workspace crate as a per-crate
  API reference inside the mdbook, with a cross-crate link graph and a
  nightly + per-PR delta surface. This is the canonical Rust API doc pipeline
  for Oyatie — no hand-written API reference docs are permitted to compete.
planned_enforcement_ref: governance-rustdoc-publish
extends_crates:
  - intelligence-mdbook-kernel
  - governance-readme-doc-coverage-kernel
companion_docs:
  - INDEX.md
  - ../../docs/DOC-CATALOG.md
doc_status: published
---

# Pipeline: rustdoc auto-publish

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

Take every workspace crate's `cargo doc --no-deps --document-private-items=false` JSON output and lift it into the `intelligence-mdbook-kernel`-validated source tree at `docs/site/src/api/rust/<crate>/`. No hand-authored Rust API reference is permitted; the kernel rejects unlisted markdown sources, so any drift fails CI.

## 2. Inputs (sources of truth)

- Every `crates/<crate>/src/**/*.rs` `///` comment.
- `crates/<crate>/Cargo.toml` `[package]` metadata (name, version, description, keywords).
- Workspace `Cargo.toml` `[workspace.members]` list.
- Per-crate `data_class:` field annotations (cross-fed from `schema-doc-pipeline.md`).

## 3. Outputs

- Per-crate mdbook chapter `docs/site/src/api/rust/<crate>/index.md` (struct list, enum list, fn list, trait list, public re-exports).
- Per-crate symbol-level chapters `docs/site/src/api/rust/<crate>/<module>/<symbol>.md` for any item whose doc-comment exceeds 200 chars.
- Cross-crate link graph `docs/site/src/api/rust/_link-graph.md` showing every `use other::Symbol` edge between workspace crates.
- JSON sidecar `docs/site/src/api/rust/_index.json` with `{crate, version, symbol_count, doc_coverage_pct, generated_at}`.

## 4. Trigger matrix

| Event | Action |
|---|---|
| Per-PR (any `crates/**` change) | Re-render touched crates only; mdbook-kernel re-validates source tree. |
| Nightly | Full re-render across all workspace members; full link-graph rebuild. |
| Per-tag release | Snapshot of the rustdoc tree archived to `docs/site/archive/<version>/api/rust/`. |

## 5. Validation gates (the `governance-rustdoc-publish` lane)

The lane consumes the JSON sidecar plus the rendered source tree and refuses to pass when any of the following hold:

1. **Doc-coverage floor.** Public items doc-coverage per crate < 85% (BLOCKER). Computed from `cargo doc` JSON: items with non-empty `docs` over total public items.
2. **Orphan symbol.** A symbol exists in `_index.json` whose rendered chapter is missing from the mdbook source tree.
3. **Broken cross-crate link.** A `use other_crate::Symbol` edge resolves to a symbol absent from `other_crate`'s `_index.json`.
4. **Hand-authored intrusion.** Any markdown under `docs/site/src/api/rust/` lacks the `<!-- generated-by: rustdoc-pipeline -->` magic header (HIGH).
5. **mdbook-kernel rejection.** The generated tree fails `intelligence-mdbook-kernel::validate_mdbook_source` for any reason.

## 6. Cross-crate link graph algorithm

For every workspace member crate, parse `cargo doc` JSON `paths` table; emit edge `(source_crate, target_crate, symbol)` whenever `source_crate`'s rendered HTML references a `target_crate`'s `paths` entry. Aggregate into Mermaid graph + Graphviz dot inside `_link-graph.md`. The `architecture-map-kernel` consumes this graph as one of its inputs.

## 7. Per-PR delta surface

The PR comment posts a table with `<crate>: +N symbols, -M symbols, coverage Δ pct`. Computed by diffing the previous merge-base `_index.json` against the head `_index.json`. Surfaces hidden-removal of public symbols (potential semver break) for human review.

## 8. Out-of-scope

- Private-item docs (workspace policy: rustdoc is the public surface).
- Cross-language API reference (covered by `openapi-pipeline.md` for HTTP/gRPC).
- Per-symbol example code execution (deferred to `cargo-doctest` lane in `governance-quality-lane-kernel`).
