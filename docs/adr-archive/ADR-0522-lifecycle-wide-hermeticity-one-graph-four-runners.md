---
id: ADR-0522
title: "Lifecycle-wide hermeticity: one buck2 graph, four runners (build·CI·CD·dev-env)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0392, ADR-0516]
amends: [ADR-0392]
related: [ADR-0392, ADR-0515, ADR-0516, ADR-0523, ADR-0524, ADR-0525]
related_specs:
  - /specs/cloud-toolchain-target.json
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0522: Lifecycle-wide hermeticity — one buck2 graph, four runners

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 1 of ADR-0516. **Amends / extends ADR-0392** (generalizes "the build graph" to
"one lifecycle graph, four runners"); **complements ADR-0515**.

## Context

ADR-0392 made buck2 the canonical BUILD graph, and ADR-0515 made the firewall / CI hermetic and
shell-free. But the four lifecycle stages — build, CI, CD, dev-env — still risk drifting onto separate
substrates, the cardinal false-green sin being a runner that re-implements a narrower command subset.
Component 1 of the Agentic Delivery Fabric (ADR-0516) is lifecycle-wide hermeticity, which this ADR
ratifies as architecture rather than an emergent CI fact.

## Decision

`buck2 build //...` and `buck2 test //...` are the SINGLE source of truth for what gets built and
verified across the entire engineering lifecycle. BUILD (dev), CI, CD, and DEV-ENV are four **RUNNERS**
of that one target graph; the only per-runner difference is a THIN, ideally GENERATED adapter (a forge
YAML, an owned-runner control-plane Job spec, an Argo CD manifest, a `.cargo`/`.envrc` shim). The gate
LOGIC is a buck2 `rust_test` target (the same Rust crate the cargo lanes test today). Every runner
invokes `//...` or its affected-scoped subset; **NO runner re-implements a narrower command subset**
(the cardinal false-green sin).

Per-stage target shape:

- **BUILD** — every artifact has a buck2 rule with declared inputs (no `cargo build` orchestrated by
  bash).
- **CI** — the forge workflow does `buck2 test //...` and nothing else; fold the parallel cargo CIs in.
- **CD** — split into a buck2-native artifact BUILD (content-addressed OCI/image rules) + a
  declarative RECONCILE (Argo CD / CAPI / tofu desired-state, NEVER a `deploy.sh` buck2 action); the
  seam is "buck2 builds the hashed artifact → the GitOps manifest pins that hash."
- **DEV-ENV** — a fresh clone + a pinned toolchain = a green build, zero setup script.

## Drivers

- Founder apex directive: hermeticity must apply to build · CI · CD · dev-env, with "it just works"
  and 0-to-minimal shell/CLI.
- The hermetic-just-works doctrine: a clean checkout builds + runs reproducibly, no external/prebuilt
  blobs, no manual steps, CI/firewall-enforced.
- "Git is transitional": the forge YAML and the owned-runner Job are interchangeable adapters over one
  graph.

## Alternatives considered

- **"Hermetic by any means" — kernel on cargo+bash, only cloud on buck2** — rejected as the canonical
  posture: it leaves two enforcement substrates (the antithesis; viable only if the founder wants
  "hermetic" rather than "one substrate").
- **Per-runner bespoke logic** — rejected (false-green; violates the one-canonical-CI Principle 1).
- **Keeping CD as a buck2 `deploy` action** — rejected: a deploy is a side-effecting reconcile,
  correctly outside the pure graph.

## Consequences

Every `run:` / `sh` / Makefile target that does real work becomes a retirement target into a buck2
target; the residue is the explicitly-justified irreducible-glue ledger (ADR-0523); the kernel must be
buckified from zero BUCK targets (ADR-0524). One graph means a single affected-set driver, a single
RBE/cache key, and the work-area hash can later unify SCM-id + buck2 cache key + CD artifact hash
(ADR-0517). **Amends/extends ADR-0392** by generalizing the build graph to one lifecycle graph driven
by four runners; **complements ADR-0515** (its lifecycle-wide operationalization). door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md (RATIFY-TO-ADR). Detail under Component 1 of
ADR-0516; amends/extends ADR-0392; complements ADR-0515.*
