---
id: ADR-0512
status: Superseded
superseded_by: [ADR-0701]
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
amended_by:
  - ADR-0562
relates:
  - ADR-0392
  - ADR-0550
  - ADR-0525
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0512: Canonical monorepo pattern — vertical-slice nesting, one workspace, bounded-context crates, dependency-rule modules, Buck2 graph

## Status

Accepted — 2026-05-29 (founder-locked). Supersedes ADR-0357 (vertical-slice nesting, was Proposed) and ADR-0509 (single-crate-per-service); amends ADR-0131; the build-graph clauses align with ADR-0392 (Buck2), with later CI orchestration refinement in ADR-0525.

**Amendment — 2026-06-02 (platform-readiness pure split):** the top-level service root is no longer
`microservices/<ms>/`. Canonical service homes are `{oya,cloud}/<service>/` with shared cross-cutting libraries under
`libs/<lib>/`. `microservices/` is legacy only and must be removed after P0.1/P0.6 prove all migration packets are
complete. The colocation/crate/workspace/Buck2 rules below remain binding with the updated root.

**Amendment — 2026-06-11 (ADR-0550 layout doctrine):** clauses 3/4 are narrowed. Crate =
bounded-context remains the sizing default and module-level layering remains the rule *inside* a
crate, but two boundaries are crate boundaries by mandate, never modules: the ADR-0510
transient-tech seam (kernel vs adapter) and the composition root (app) — per the founder
structure directive of 2026-06-10 (ADR-0543, PR #686) and the kernel-purity gate (ADR-0547),
which enforces the seam on the crate dependency graph where a module seam is invisible. The
"no layer-per-crate" rejection stands everywhere except these two mandated seams. See ADR-0550 D1.

**Amendment — 2026-06-14 (ADR-0562 capability-first repo organization):** the top-level
`{oya,cloud}/<service>/` + `libs/<lib>/` root assumption is **superseded** by the capability-first
shape. The tree is organized by capability (one top dir per registered system; faces
core/ports/adapters/facade inside it); `libs/` **dissolves** into capability homes (single-capability
shared code) + `base/` (>=3-consumer cross-capability primitives, admission-gated). The surviving
invariants are unchanged: one root Cargo workspace, one-version policy, crate = bounded context, the
Buck2 graph as the parallelism/containment substrate. **Carve-out (ADR-0562 §8 Fork 2):** the
kuberos kernel becomes top-level `kernel/` as a **sanctioned nested/excluded Cargo workspace** —
clause 2's "no nested `[workspace]` tables" rule and the `workspace-topology` gate are amended to
whitelist `kernel/` (analogous to the release-image cargo exception; the `no_std`+custom-sysroot rung
cannot share the std-targeted root lockfile). This is the ONLY sanctioned nested workspace. ADR-0562
is the governing reorg ADR.

**Ratification stamp — 2026-07-10 (ADR-0562 Accepted).** ADR-0562 was ratified/Accepted by the
founder on 2026-07-10, so the supersession above is now **live and scoped**: only clause 1
(**Layout** — the `{oya,cloud}/<service>/` + `libs/<lib>/` root assumption) is superseded by
capability-first. Clauses 2–5 (one root Cargo workspace minus the `kernel/` carve-out, crate =
bounded context, clean-architecture-as-modules, and Buck2-graph parallelism) remain **binding and
unchanged**. Clause 6 (Enforcement)'s superseded `libs/`-layout predicates (`libs/` accepted / flat
`crates/` rejected) are now governed by ADR-0562 (`libs/` dissolves into capability homes + `base/`);
its non-layout enforcement (workspace-topology validation — no nested `[workspace]` tables,
duplicate package names, non-member crate dirs, missing `BUCK` targets) remains binding. This is a
scoped amendment (`amended_by: ADR-0562`), NOT a full supersession of ADR-0512, which stays
**Accepted**. (Contrast ADR-0550, which was a
layout-only doctrine and is therefore superseded *in full* by ADR-0562.)

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

1. **Layout.** Service code lives at `{oya,cloud}/<service>/crates/<crate>/`; product-facing/domain services live
   under `oya/`, platform/tenant substrate services under `cloud/`, and shared cross-cutting libraries under
   `libs/<lib>/`. Each service co-locates contracts/ci/iac/manifest. A flat top-level `crates/` directory is
   **forbidden**. `microservices/` is legacy/removal-candidate and must be empty after verified migration. The
   `foundry` name remains eradicated.
2. **Workspace topology.** **One** root Cargo workspace, one `Cargo.lock`, one-version policy. **No nested `[workspace]` tables.** Containment and parallelism come from the Buck2 build graph, not from fragmenting Cargo workspaces — per-microservice workspaces fragment one-version, block atomic cross-service refactors, degrade rust-analyzer, and are ignored by Buck2 regardless.
3. **Granularity: crate = bounded context.** A single-concern service is ONE crate; a multi-bounded-context service is one crate per bounded context (e.g. `itsm` = on-call-schedule / escalation-policy / incident-room / status-update / postmortem, plus an optional umbrella). EVERY service crate — including a single-concern service's sole crate — lives at `{oya,cloud}/<service>/crates/oya-<...>/`; the service directory is ALWAYS a pure container with no crate at its root (this amends the earlier "src/ at ms-root" idea, founder-decided 2026-05-29 for uniformity). The crate directory basename MUST equal the `[package].name` (enforced by `workspace-topology` R7). **No layer-per-crate** (ADR-0357's over-engineering: clean-arch layers are not crates) and **no whole-service mega-crate** (a coarse Buck2 target whose any-edit rebuilds everything).
4. **Internal structure = clean architecture as MODULES, not crates.** Keep the dependency rule — `domain` (pure) / `ports` (traits) / `adapters` (impls) / `api` (handlers) as `mod`s, `pub(crate)` enforcing inward-only dependencies — applied **proportional to complexity** (a CRUD service does not earn five layers). This preserves testability and adapter-swappability without crate ceremony; uniform enough for agent-generated services + mechanical gate checks.
5. **Build / parallelism.** Buck2 fine-grained per-crate targets + remote execution + hermetic content-addressed caching (ADR-0392/0408) is the destination; sccache is the interim shared cache. Because Buck2 is rustc-direct, per-target rustc tuning (codegen-units/LTO/opt/target-cpu), affected-targets test selection via graph query, one polyglot graph (Rust + proto/OpenAPI codegen + Leptos/WASM IDP + container images + IaC), dependency-rule-as-`visibility`, and the `no_std`/custom-sysroot path for the bespoke-kernel ambition all become available. The canonical crate=bounded-context boundary is exactly what Buck2 targets mirror.
6. **Enforcement.** The `architecture-boundaries` gate requires service code under `{oya,cloud}/<service>/crates/` or shared code under `libs/<lib>/` (flat `crates/` rejected; `libs/` accepted). Workspace-topology validation fails on: a flat `crates/` directory; any nested `[workspace]` table; duplicate package names; a crate directory that is not a root workspace member; legacy `microservices/` service code after the migration checkpoint; and (forward) a missing `BUCK` target. These validations are ported into Rust gate crates and cloud-ci/oya-ci required contexts; `oya` CLI entrypoints are retirement/migration wrappers, not CI authority.

## Consequences

**Positive:** one coherent build graph; path-evident ownership and navigability; Buck2-ready fine-grained caching/parallelism with crate=bounded-context as the dial; the dependency rule enforced as modules (and forward, as Buck2 visibility) rather than crate sprawl; the layout is mechanically gate-enforced so it cannot drift again. **Negative/cost:** a one-time consolidation (strip 28 nested workspaces, relocate the `itsm` umbrella, eradicate flat `crates/`, regenerate members, fix inter-crate contract drift) plus ongoing gate maintenance. **Neutral:** package names, dependency graph, and the one-version policy are unchanged; the change is mechanical + manifest-level, not a code-behavior change.

**Process rule (hard):** a structural migration of this class MUST run as a dedicated, exclusive, post-acceptance change on a stable tree, with the Prow/cloud-ci required context green before and after — never merged concurrently in a PR drain. Violating this is what broke `dev` and motivated this ADR.

## Historical residual from ADR-509 (E3 fold 2026-08-06)

**Title:** Hyperscaler service decomposition pattern (single-crate-per-service + mod-based subsystems)

**Preserved decision gist:** ### 1. Single-crate-per-service is the canonical decomposition `microservices/<service>/Cargo.toml` is ONE crate. `src/main.rs` compiles ONE binary per service. There is no per-use-case compile boundary inside a service. ### 2. Subsystem mods decompose by SUBSYSTEM, not by use case Under `src/`, directories represent subsystems (e.g., `auth/`, `oidc/`, `webauthn/`, `realms/`, `users/`, `storage/`, `rest/`, `grpc/`, `observability/`). A subsystem may contain multiple use cases as functions or sub-modules; the subsystem boundary is the architectural seam, not the use case. ### 3. Proto / OpenAPI

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-357 (E3 fold 2026-08-06)

**Title:** ADR-0357-vertical-slice-monorepo-nesting

**Preserved decision gist:** Adopt vertical-slice nesting: a service's crates move to `microservices/<ms>/crates/oya-<service>-<layer>` so code, contracts, ci, iac, and manifest are co-located. Package names are unchanged (so imports and `registry/catalog` keys are stable); only physical paths and `Cargo.toml` member paths + `catalog` path fields change. The `architecture-boundaries` gate flips to enforce code under `microservices/<ms>/crates/` instead of flat `crates/`. Top-level naming (two buckets, services vs libs): deployable service code lives under `microservices/<ms>/crates/oya-<service>-<layer>` (co-located with 

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-15 (E3 fold 2026-08-06)

**Title:** ADR-0015-architectural-flattening-target

**Preserved decision gist:** We adopt **flat-crates** as the canonical target, **`oya-<context>-<role>[-<capability>]`** as the naming convention, **a closed role taxonomy** with explicit dep direction, **a boundary validator** that hard-fails forbidden edges, and **a forward-only migration posture** from the legacy tree. **Live baseline as of 2026-05-11:** the Cargo workspace has 64 members, every workspace member lives under `crates/oya-*`, every workspace member has a `registry/catalog/<crate>.yaml` record, and top-level `modules/`, `services/`, `platform/`, and `tools/` are absent. Remaining ADR-0015 work is additive 

_Source file archived after fold; full body in git history / docs/adr-archive/._
