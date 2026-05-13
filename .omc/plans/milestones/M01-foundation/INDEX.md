---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M01
title: Foundation
wave: W-Foundation
status: open
owner: council-architecture + platform-tenancy-identity
purpose: Ship the foundation correctness layer — every cross-axis contract correct from day one, no product surface ships before this milestone passes.
acceptance_authority: docs/ROADMAP.md §2.1
---

# M01 — Foundation

## Purpose
Every cross-axis contract correct from day one. Tenant kernel, identity kernel, audit chain, plane separation, Data Use Boundary, cell architecture, eventing backbone, Cedar policy substrate, schema-class annotation, license policy, regional pack architecture, architectural flattening — all foundation ADRs Accepted; fitness functions hard-fail on violations.

## Status
**open.** No product surface (Foundry, Cloud, SaaS, Workspace, Search) may merge to `main` until this milestone passes.

## Scope
Per [`docs/ROADMAP.md`](../../../../docs/ROADMAP.md) §2.1 W-Foundation gate criteria + the 10 compound principles from [`../../MASTERPLAN.md`](../../MASTERPLAN.md) §2. Foundation crates target the `crates/oya-platform-*` namespace.

## Dependencies
None (M01 is the root of the dependency graph). Depends only on M-CC-P01 (agentic-pipeline cutover) being at ≥P5 merged so banned-primitives lane is active before any new foundation crate scaffolds.

## Acceptance gate
- All [`docs/ROADMAP.md`](../../../../docs/ROADMAP.md) §2.1 gate criteria met.
- Data Use Boundary ADR Accepted.
- Tenant + Identity kernels at flat-crates target with Cedar RBAC/ABAC.
- Audit chain (ADR-0003) hash-chained + emission contract published.
- Plane separation enforced (every catalog record declares plane).
- Cell architecture ADR + cell-routing primitive operational.
- Object Graph property tiers (ADR-0006..0112) all Accepted.
- Eventing backbone (outbox + Kafka topic registry) operational.
- Schema-class annotation lane (`oya-foundry-fitness-data-class`) green.
- License policy ADR Accepted; license-policy lane hard-fails on AGPL/GPL/SSPL/BUSL/RSAL.
- Regional pack architecture ADR + `crates/oya-platform-regional-pack-kernel` shipped.
- Architectural flattening (ADR-0015) — `oya-foundry-fitness-flat-crates-guard` green.

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Data Use Boundary + Tenancy Kernel | stub | [`phases/P01-data-use-boundary-tenancy/INDEX.md`](phases/P01-data-use-boundary-tenancy/INDEX.md) |
| P02 | Identity Kernel + Cedar Policy Substrate | stub | [`phases/P02-identity-cedar/INDEX.md`](phases/P02-identity-cedar/INDEX.md) |
| P03 | Audit Chain + Evidence Emission | stub | [`phases/P03-audit-chain-evidence/INDEX.md`](phases/P03-audit-chain-evidence/INDEX.md) |
| P04 | Eventing Backbone + Outbox + Object Graph | stub | [`phases/P04-eventing-object-graph/INDEX.md`](phases/P04-eventing-object-graph/INDEX.md) |
| P05 | Cell Architecture + Plane Separation Enforcement | stub | [`phases/P05-cell-plane/INDEX.md`](phases/P05-cell-plane/INDEX.md) |
| P06 | Regional Pack Architecture + Flattening Ratchet | stub | [`phases/P06-regional-pack-flattening/INDEX.md`](phases/P06-regional-pack-flattening/INDEX.md) |

## Parallelism strategy
P01..P06 partition by bounded context with disjoint crate suffix sets, so up to 6 phases can run as parallel batches in two waves: **G1** = {P01, P02, P03} (foundational kernels; share `oya-platform-*-kernel` but distinct context suffix); **G2** = {P04, P05, P06} (consume G1 ports; can fan out once G1 ≥ 50% merged). Target: 3-5 agents per active phase; ≤ 5 concurrent IPs per phase.

## Hyperscaler practices adopted (per [`../../MASTERPLAN.md`](../../MASTERPLAN.md) §2 principle 6, 9)
- AWS Working-Backwards / PRFAQ for each phase entry.
- Google Design Doc per phase (lands as `phases/<PNN>/DESIGN-DOC.md` before any IP merges).
- SRE postmortem-blameless on any P-internal incident.
- Microsoft 1ES-templated pipelines for CI lanes.
- Oracle Engineering-Excellence-Council–style merge gate (council-architecture signs every IP).
- Rust toolchain: `cargo-deny`, `cargo-audit`, `cargo-nextest`, `cargo-semver-checks`, `sccache`, `cargo-llvm-cov`.
- Sigstore/SLSA + OpenTelemetry + Distroless inherited from M-CC-P06/P08.

## Agent-navigability-pointer
A fresh agent picks up M01 by:
1. `icm recall -t context-oyatie -k "M01 foundation"` to load checkpointed state.
2. Read this INDEX, then pick a P0N with no open prerequisites.
3. Read `phases/<PNN>/INDEX.md`, then pick an IP-NNN with `agent-prerequisites` met.
4. `grit claim --agent <id> --intent "M01-P0N IP-NNN <one-line>" <file::Identifier>` against the IP's listed symbols.

First-claim seed symbol for the first contributor: `crates/oya-platform-tenant-kernel/src/lib.rs::Tenant` (after P01 IP-001 scaffold-claim per ADR-0054).
