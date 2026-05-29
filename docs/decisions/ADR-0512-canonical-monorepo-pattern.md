---
id: ADR-0512
status: Accepted
planning_impact: true
date: 2026-05-29
owners:
  - council-architecture
  - founder
supersedes:
  - ADR-0357
  - ADR-0509
amends:
  - ADR-0131
relates:
  - ADR-0392
  - ADR-0408
---
# ADR-0512: Canonical monorepo pattern — vertical-slice nesting, one workspace, bounded-context crates, dependency-rule modules, Buck2 graph

## Status

Accepted — 2026-05-29 (founder-locked). Supersedes ADR-0357 (vertical-slice nesting, was Proposed) and ADR-0509 (single-crate-per-service); amends ADR-0131; the build-graph clauses align with ADR-0392/0408 (Buck2).

## Date

2026-05-29

## Context

The repository is a single Rust monorepo (~716 crates) targeting hyperscaler-grade discipline: parallel development, modular and contained builds, navigable ownership, and a one-version policy. Three internal decisions were in tension and had drifted into an inconsistent on-disk reality:

1. **ADR-0357 (Proposed)** adopted vertical-slice *nesting* — move service code to `microservices/<ms>/crates/oya-<service>-<layer>` and shared code to `libs/<lib>/` — to fix the flat-`crates/` navigability anti-pattern and close ADR-0131's intent-vs-enforcement gap. It mandated execution as a *single dedicated migration on a stable tree*, with flat `crates/` remaining canonical until then.
2. **ADR-0509** mandated *single-crate-per-service with mod-based subsystems*, rejecting per-use-case clean-architecture crate sprawl.
3. **ADR-0131** intended `microservices/<ms>/` as the code root but the `architecture-boundaries` gate enforced flat `crates/`.

The trigger: a PR-backlog drain accidentally merged the ADR-0357 migration **concurrently with 37 in-flight feature PRs**, violating ADR-0357's own sequencing. The result was a structurally broken workspace — a clobbered root manifest, 28 nested `[workspace]` tables (an emergent drift toward per-microservice Cargo workspaces), one service collision (`itsm`), a dead `foundry` crate, and inter-crate contract drift. Recovering required deciding the canonical pattern once and enforcing it.

Two genuine forks had to be resolved with the founder, against the criteria *hyperscaler-pattern · parallelism · modular/contained/maintainable · Rust+Buck2*:
- **Workspace topology**: one root Cargo workspace vs per-microservice workspaces.
- **Granularity**: ADR-0357 layer-per-crate vs ADR-0509 single-crate-per-service.

A clarifying fact about the destination build system: **Buck2 does not call Cargo — it reads `BUCK` files and invokes `rustc` directly** (explicit `--extern`/deps per target). So each `rust_library`/`rust_binary` target is one crate is one `rustc` action, the Cargo workspace is an interim + rust-analyzer/one-version artifact that Buck2 is indifferent to, and crate size is the build-parallelism dial. This decouples build optimization from Cargo and is the basis for the topology and granularity choices below.

## Decision

Adopt the canonical pattern **"vertical-slice monorepo · one workspace · bounded-context crates · dependency-rule modules · Buck2 graph"**:

1. **Layout.** Service code lives at `microservices/<ms>/crates/<crate>/`; shared cross-cutting libraries at `libs/<lib>/`; co-located with each service's contracts/ci/iac/manifest. A flat top-level `crates/` directory is **forbidden**. The `foundry` name remains eradicated.
2. **Workspace topology.** **One** root Cargo workspace, one `Cargo.lock`, one-version policy. **No nested `[workspace]` tables.** Containment and parallelism come from the Buck2 build graph, not from fragmenting Cargo workspaces — per-microservice workspaces fragment one-version, block atomic cross-service refactors, degrade rust-analyzer, and are ignored by Buck2 regardless.
3. **Granularity: crate = bounded context.** A single-concern service is ONE crate; a multi-bounded-context service is one crate per bounded context (e.g. `itsm` = on-call-schedule / escalation-policy / incident-room / status-update / postmortem, plus an optional umbrella). EVERY service crate — including a single-concern service's sole crate — lives at `microservices/<ms>/crates/oya-<...>/`; `microservices/<ms>/` is ALWAYS a pure container with no crate at its root (this amends the earlier "src/ at ms-root" idea, founder-decided 2026-05-29 for uniformity). The crate directory basename MUST equal the `[package].name` (enforced by `workspace-topology` R7). **No layer-per-crate** (ADR-0357's over-engineering: clean-arch layers are not crates) and **no whole-service mega-crate** (a coarse Buck2 target whose any-edit rebuilds everything).
4. **Internal structure = clean architecture as MODULES, not crates.** Keep the dependency rule — `domain` (pure) / `ports` (traits) / `adapters` (impls) / `api` (handlers) as `mod`s, `pub(crate)` enforcing inward-only dependencies — applied **proportional to complexity** (a CRUD service does not earn five layers). This preserves testability and adapter-swappability without crate ceremony; uniform enough for agent-generated services + mechanical gate checks.
5. **Build / parallelism.** Buck2 fine-grained per-crate targets + remote execution + hermetic content-addressed caching (ADR-0392/0408) is the destination; sccache is the interim shared cache. Because Buck2 is rustc-direct, per-target rustc tuning (codegen-units/LTO/opt/target-cpu), affected-targets test selection via graph query, one polyglot graph (Rust + proto/OpenAPI codegen + Leptos/WASM IDP + container images + IaC), dependency-rule-as-`visibility`, and the `no_std`/custom-sysroot path for the bespoke-kernel ambition all become available. The canonical crate=bounded-context boundary is exactly what Buck2 targets mirror.
6. **Enforcement.** The `architecture-boundaries` gate requires code under `microservices/<ms>/crates/` or `libs/<lib>/` (flat `crates/` rejected; `libs/` accepted). A new `oya gate validate workspace-topology` lane fails on: a flat `crates/` directory; any nested `[workspace]` table; duplicate package names; a crate directory that is not a root workspace member; and (forward) a missing `BUCK` target. Both wire into `oya verify --ci-required` / `AGGREGATED_VALIDATE_LANES`.

## Consequences

**Positive:** one coherent build graph; path-evident ownership and navigability; Buck2-ready fine-grained caching/parallelism with crate=bounded-context as the dial; the dependency rule enforced as modules (and forward, as Buck2 visibility) rather than crate sprawl; the layout is mechanically gate-enforced so it cannot drift again. **Negative/cost:** a one-time consolidation (strip 28 nested workspaces, relocate the `itsm` umbrella, eradicate flat `crates/`, regenerate members, fix inter-crate contract drift) plus ongoing gate maintenance. **Neutral:** package names, dependency graph, and the one-version policy are unchanged; the change is mechanical + manifest-level, not a code-behavior change.

**Process rule (hard):** a structural migration of this class MUST run as a dedicated, exclusive, post-acceptance change on a stable tree, with `oya verify --ci-required` green before and after — never merged concurrently in a PR drain. Violating this is what broke `dev` and motivated this ADR.
