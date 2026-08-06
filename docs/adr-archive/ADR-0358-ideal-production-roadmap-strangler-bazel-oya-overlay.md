---
id: ADR-0358
status: Superseded
planning_impact: true
date: 2026-05-25
owners:
  - council-architecture
supersedes: []
superseded_by: [ADR-0392, ADR-0408]
amends: []
amendment_note: "2026-05-29 (founder decision): §2 toolchain build-graph + CI engine reversed from Bazel rules_rust to Buck2. Superseded-by ADR-0392 (Buck2 canonical build graph) + ADR-0408 (Buck2-driven CI/CD). ONLY §2's build-graph/CI engine is reversed; §1 strangler-fig, §3 define-production-100-first, and §4 masterplan planning authority remain in force."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0358: Ideal 0→100 production roadmap — strangler-fig migration, Bazel rules_rust + oya governance overlay, define-production-100-first

## Status

Proposed — 2026-05-25.

> **Amendment (2026-05-29, founder decision):** §2's toolchain build-graph + CI engine is **reversed from Bazel `rules_rust` to Buck2** and is **Superseded-by ADR-0392** (Buck2 canonical build graph) and **ADR-0408** (Buck2-driven CI/CD). Only §2's build-graph/CI engine is reversed; §1 (strangler-fig), §3 (define-production-100-first), and §4 (masterplan planning authority) remain in force. The Bazel rationale below (incl. the Buck2/Reindeer objection) is preserved as the superseded record; ADR-0392 §2 confronts and accepts that objection explicitly.

## Date

2026-05-25

## Context

The platform must reach hyperscaler-grade production quality for the Oyatie Cloud substrate and the microservices that dogfood it as tenants (ADR-0242). The current state is a 750-crate single Cargo workspace with a bespoke `oya` verify/gate engine whose `oya verify --ci-required` re-runs the full workspace build+test locally (and duplicates the cargo mirrors inside `gate run-all`), which does not scale and is not how hyperscalers build (Google TAP affected-targets + Bazel RBE; Amazon Apollo staged rollout; Microsoft rings; Oracle blue-green). The masterplan must encode the *ideal* long-term, parallel-conscious 0→100 plan and reconsider prior choices against that bar rather than accrete short-term patches.

Research grounding: Google TAP (affected-target presubmit) + Bazel remote build execution; Amazon Builders' Library continuous delivery (one-box → cell → region, bake time, automatic metric-gated rollback); Microsoft deployment rings; Oracle OCI DevOps blue-green/canary; Nygard/AWS/Azure ADR immutability+supersession; SSOT single-master + the "constitution-with-amendments" anti-pattern; Rust CI tooling (cargo-nextest partition sharding, sccache remote cache, cargo-deny, cargo-machete, bacon for local). Bazel `rules_rust` is chosen over Buck2 because Buck2 requires Reindeer to vendor all Cargo deps (hostile to Git/code-review) and is less battle-tested in OSS, while `rules_rust` supports Cargo.toml-as-SSOT (`crate_universe`) with mature RBE.

## Decision

1. **Migration posture = strangler-fig.** Spec the ideal target architecture and contracts first; build the ideal alongside the current tree; migrate service-by-service behind stable contracts (Workflow/Ontology adapter layer, OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3). Trunk stays shippable; lanes run in parallel; a service cuts over only when green. No big-bang rewrite, no indefinite evolve-in-place.
2. **Toolchain overhaul = Bazel `rules_rust` build graph + `oya` governance overlay.** Bazel provides the build/test DAG, hermetic remote cache, remote build execution, and affected-target selection (Cargo.toml stays the dependency SSOT via `crate_universe`). `oya` is rebuilt as a thin governance/verify orchestrator that delegates build/test to `bazel query`/`bazel test` and runs only the bespoke governance gates — retiring the duplicated cargo-mirror `verify`/`run-all` engine.
3. **Define production-100 first.** The first roadmap phase defines the production exit bar collaboratively: scale targets, compliance packs, deployment model (ADR-0254 spectrum), first production workload (FD-001 candidate), and the SLO/DR/security/cost exit gates. Every maturity claim stays blocked_until_required_evidence_is_green per hyperscaler-gates.json; no production-readiness assertion until that bar's evidence is green.
4. **The masterplan `ideal_production_roadmap` section is the single planning authority** for this effort; this ADR and the per-phase ADRs bind into it (planning-ssot-coverage gate).

## Consequences

Positive: a long-term, parallel-conscious, research-grounded path to production with an always-green trunk; a path toward hyperscaler-pattern build/CI parity (RBE + affected-targets + merge-queue speculative + progressive delivery), with parity claims blocked_until_required_evidence_is_green; the bespoke toolchain shrinks to governance-only. Negative/cost: adopting Bazel across 750 first-party crates is a substantial program (watch `rules_rs` for first-party scaling); the strangler period runs two architectures in parallel; the production-100 bar is intentionally undefined until the first phase completes. Neutral: package names and the ADR/decision corpus are unchanged; this ADR is doctrine, not the migration execution itself.
