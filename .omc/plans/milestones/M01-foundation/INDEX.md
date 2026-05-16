---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M01
title: Foundation
wave: W-Foundation
status: complete
owner: council-architecture + tenancy-identity
purpose: Ship the foundation correctness layer — every cross-axis contract correct from day one, no product surface ships before this milestone passes.
acceptance_authority: docs/ROADMAP.md §2.1
---

# M01 — Foundation

## Purpose
Every cross-axis contract correct from day one. Tenancy µservice kernel, identity µservice kernel, audit chain, plane separation, Data Use Boundary, cell architecture, eventing backbone, Cedar policy substrate, schema-class annotation, license policy, regional pack architecture, architectural flattening — all foundation ADRs Accepted; LEAN check crates hard-fail on violations.

## Status
**complete for M01 Foundation acceptance.** Product-surface implementation for M02/M03 remains gated, but the M01 foundation contracts are now accepted against live Cargo workspace metadata and BNF v4.1 package names.

Current evidence checkpoint (2026-05-14): G1 = P01/P02/P03 is complete and usable; G2 = P04/P05/P06 is complete after the G1 contract surface. Fresh focused tests over the G1/G2 package set pass (65/65), M-CC-P01 is at or beyond P5 with banned-primitives/archive-orphan/authoritative-tracked lanes green, and the latest full workspace closeout passes `./scripts/check.sh` under Rust 1.95.0 / edition 2024 / rustfmt 2024. M-CC-P00 remains planned, not ready; this acceptance closeout uses a scoped waiver for evidence reconciliation only and does not authorize broad fanout or new foundation scaffolds until P00 is ready or explicitly waived.

## Scope
Per [`docs/ROADMAP.md`](../../../../docs/ROADMAP.md) §2.1 W-Foundation gate criteria + the 10 compound principles from [`../../MASTERPLAN.md`](../../MASTERPLAN.md) §2. Foundation µservice crates target the BNF v4.1 names: `oya-tenancy-*`, `oya-identity-*`, `oya-audit-chain-*`, `oya-eventing-*`, `oya-cell-*`, `oya-regional-pack-*`, etc. (no `oya-platform-*` prefix).

## Dependencies
None (M01 is the root of the dependency graph). Before broad fanout or new foundation scaffolds: M-CC-P01 must be at ≥P5 and M-CC-P00 must be ready or explicitly waived. Current evidence: M-CC-P01 is foundation-cleared/P5+ by live lane output; M-CC-P00 is only accepted-for-masterplan-P00/planned, so the only active waiver is for M01 acceptance reconciliation of already-implemented G1/G2 contracts.

## Acceptance gate
- All [`docs/ROADMAP.md`](../../../../docs/ROADMAP.md) §2.1 gate criteria met.
- Data Use Boundary ADR Accepted.
- Tenancy + Identity µservice kernels at flat-crates target with Cedar RBAC/ABAC.
- Audit chain (ADR-0003) hash-chained + emission contract published.
- Plane separation enforced (every catalog record declares plane).
- Cell architecture ADR + cell-routing primitive operational.
- Ontology property tiers (ADR-0006..0112) all Accepted.
- Eventing backbone (outbox + Kafka topic registry) operational.
- Schema-class annotation lane (`oya-check-data-class`) green.
- License policy ADR Accepted; license-policy lane hard-fails on AGPL/GPL/SSPL/BUSL/RSAL.
- Regional pack architecture ADR + `crates/oya-regional-pack-domain` shipped.
- Architectural flattening (ADR-0015) — `oya-check-architecture -- naming-collision` green.

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Data Use Boundary + Tenancy Kernel | complete | [`phases/P01-data-use-boundary-tenancy/INDEX.md`](phases/P01-data-use-boundary-tenancy/INDEX.md) |
| P02 | Identity Kernel + Cedar Policy Substrate | complete | [`phases/P02-identity-cedar/INDEX.md`](phases/P02-identity-cedar/INDEX.md) |
| P03 | Audit Chain + Evidence Emission | complete | [`phases/P03-audit-chain-evidence/INDEX.md`](phases/P03-audit-chain-evidence/INDEX.md) |
| P04 | Eventing Backbone + Outbox + Ontology | complete | [`phases/P04-eventing-object-graph/INDEX.md`](phases/P04-eventing-object-graph/INDEX.md) |
| P05 | Cell Architecture + Plane Separation Enforcement | complete | [`phases/P05-cell-plane/INDEX.md`](phases/P05-cell-plane/INDEX.md) |
| P06 | Regional Pack Architecture + Flattening Ratchet | complete | [`phases/P06-regional-pack-flattening/INDEX.md`](phases/P06-regional-pack-flattening/INDEX.md) |

## Parallelism strategy
P01..P06 partition by µservice with disjoint crate sets. Executed order: **G1** = {P01, P02, P03} (foundational µservice kernels: tenancy, identity, audit-chain) first; **G2** = {P04, P05, P06} (consume G1 ports: eventing+ontology, cell+plane, regional-pack+flattening) only after G1 contracts were usable. This milestone is not permission for M02/M03 implementation fanout; M02/M03 still require M-CC-P00 readiness or explicit waiver plus the usual wave gates.

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

First-claim seed symbol for the first contributor: `crates/oya-tenancy-domain/src/lib.rs::Tenant` (after P01 IP-001 scaffold-claim per ADR-0054; BNF v4.1 name).
