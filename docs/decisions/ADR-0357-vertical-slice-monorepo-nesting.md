# ADR-0357: Vertical-slice monorepo nesting (co-locate crate code under microservices/<ms>/crates/)

## Status

> **Status:** Proposed

## Date

2026-05-25

## Context

The repository is a single Cargo-workspace monorepo that is already substantially aligned with hyperscaler monorepo practice (Google google3/Bazel, Meta Buck2): fine-grained per-crate build units, mechanically-enforced dependency visibility and layering (the `architecture-boundaries` gate + 13-layer enum, ADR-0105/0106), per-directory ownership (CODEOWNERS + codeowners-mirror), a one-version policy (single workspace + single `Cargo.lock`), trunk-based development with a serial merge queue (ADR-0111), and — as of the wave-3 CI-farm work — a shared remote build cache (sccache → SeaweedFS, ADR-0349 + `specs/ci-farm-substrate-canonical.json`).

The one material deviation from hyperscaler practice is code locality. Today all 546 code crates live in a flat `crates/oya-*` directory, while each service's metadata (manifest, contracts, ci, iac, PRD) lives separately under `microservices/<ms>/`. Hyperscalers co-locate a service's code with its configuration under one nested domain path so that a vertical slice is navigable and ownership is path-evident. A flat 546-crate directory is the navigability anti-pattern, and it splits code from its service home.

This conflicts internally: ADR-0131's stated intent was `microservices/<ms>/` with a code root, but the `architecture-boundaries` gate currently enforces flat `crates/` (it has rejected microservice-local code paths). The conflict must be resolved as part of adopting nesting.

## Decision

Adopt vertical-slice nesting: a service's crates move to `microservices/<ms>/crates/oya-<service>-<layer>` so code, contracts, ci, iac, and manifest are co-located. Package names are unchanged (so imports and `registry/catalog` keys are stable); only physical paths and `Cargo.toml` member paths + `catalog` path fields change. The `architecture-boundaries` gate flips to enforce code under `microservices/<ms>/crates/` instead of flat `crates/`.

Sequencing: execute as a single dedicated mechanical migration AFTER the wave-3 worktree consolidation (enterprise + workflow + backbone bundles) has landed green, so the migration runs once on a stable tree rather than against a moving target of ~188 in-flight crates. Until then, flat `crates/` remains canonical and gate-enforced.

## Consequences

Positive: hyperscaler-grade vertical-slice locality and navigability; ownership becomes path-evident; closes the ADR-0131 intent-vs-enforcement conflict. Negative/cost: a large one-time path migration (~734 crates) touching `Cargo.toml` members, `registry/catalog` path fields, and the `architecture-boundaries` gate; must be a discrete, well-tested change with `oya verify --ci-required` green before and after. Neutral: package names, dependency graph, and one-version policy are unchanged; the migration is mechanical (git mv + path-field updates), not a code change.
