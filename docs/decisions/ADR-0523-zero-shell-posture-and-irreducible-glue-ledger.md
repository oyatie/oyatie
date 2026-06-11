---
id: ADR-0523
title: "Zero-shell posture + the closed irreducible-glue ledger (minimal not zero; pinned; reproducible); refines ADR-0515 D3 no-shell-bar-a-narrow-exception"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: []
depends_on: [ADR-0515, ADR-0522]
amends: []
related: [ADR-0515, ADR-0516, ADR-0522, ADR-0524, ADR-0525, ADR-0526]
related_specs:
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

# ADR-0523: Zero-shell posture + the closed irreducible-glue ledger

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Refines ADR-0515 D3 ("no shell bar a documented, justified, narrowly-scoped exception") by defining
precisely WHAT minimal residue is permitted and why. Detail under Component 1 of ADR-0516.

## Context

ADR-0515 D3 forbids shell "bar a documented, justified, narrowly-scoped exception" but does not
enumerate the exception set, so a zero-shell dogma could either delete a load-bearing pinned bootstrap
or, conversely, let shell sprawl back in under "as-needed." ADR-0522 makes every real-work shell target
a retirement target; this ADR closes the set of what may legitimately remain.

## Decision

The lifecycle target is ZERO shell/CLI glue that does real work — every `build.sh` / `cargo` / Makefile
orchestrator is a defect to retire into a buck2 target — EXCEPT a closed, authoritative
**IRREDUCIBLE-GLUE LEDGER** of six items that genuinely cannot be a pure in-graph buck2 action:

1. **Toolchain bootstrap** — the buck2 binary itself + the first rustc/QEMU/musl downloads (a build
   tool cannot build itself in-graph). Pin buck2 by release tag; move the CI `curl` into a
   sha256-pinned `download_file` in the toolchains cell; pin rustc via `rust-toolchain.toml`.
2. **The scm-facts emitter** — git is inherently ambient and CANNOT be a pure function of declared
   inputs inside a hermetic action; resolution = ONE emitter step at the graph edge whose committed,
   content-addressed output every in-graph action is pure over. (The rename + the `ScmFactsSource`
   trait is OWNED by ADR-0526 and not re-litigated here; the hermetic-boundary itself is ADR-0525.)
3. **CI checkout** — `actions/checkout fetch-depth:0` (adapter-level; required so the emitter can
   derive last-touch; shallow collapses ages → false-green).
4. **Hardware/endpoint CD bring-up** — `gen-media.sh`, CAPI `init.sh`, bare-metal `up.sh` ("boot media
   on a physical node" is inherently side-effecting / one-shot); keep as documented runbook steps with
   pinned `talosctl` / `clusterctl` / `tofu`.
5. **reindeer buckify** — the standard out-of-graph third-party BUCK generator; pin reindeer, fold the
   `*.patch` hand-edits into reindeer fixups or a checked-in `select()` overlay, wrap as
   `buck2 run //tools:buckify`.
6. **Agent-hook exec shims** — Claude/Codex project hook APIs execute configured commands, so a hook
   may retain a minimal shell shim only to locate and `exec` a repo-local Rust binary under
   `tools/hooks/bin/`, with no policy logic and fail-open warning behavior when the binary is absent.
   First authorized row: `tools/hooks/main-checkout-guard.sh` for FRIC-022 / FRIC-1781062867; the
   policy implementation is `tools/oya-checkout-guard-app`.

RULE: everything NOT in this ledger is a retirement target; any PR that touches a ledger item is a
one-way door requiring re-justification.

## Drivers

- The directive says "0 to minimal" — the ledger IS the minimal.
- The pre-mortem: a zero-shell dogma must not delete a pinned bootstrap or the `fetch-depth:0`
  checkout.

## Alternatives considered

- **Literal zero-shell** — rejected (breaks hermeticity by removing the pinned bootstrap and the
  `fetch-depth:0` checkout).
- **Unbounded shell-as-needed** — rejected (the sprawl this posture eliminates).

## Consequences

Named retirement set: the cargo CI lanes, the per-program `build.sh` × N, `run-qemu-*.sh`,
`conformance-probe.sh`, `diff-oracle*.sh`, the kernel verify-gate scripts, `build-carriers.sh`-as-
orchestrator, the tracked `out/*.elf` blobs, `.envrc` + `bin/oya` manual PATH.

**KEY TENSION (Consensus Addendum):** reproducibility OUTRANKS shell-count — "minimal shell" may
legitimately mean adopting a minimal Nix flake for QEMU/talosctl if `download_file` pinning proves
non-reproducible across host classes, which the founder must accept as MORE external machinery, not
less (carried as OQ-10 in ADR-0521; ADR-0524 may be deferred on this basis). This ADR
complements/operationalizes ADR-0515's "no shell, declarative gitops" firewall posture by defining
precisely what minimal residue is permitted and why. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md (RATIFY-TO-ADR). Refines ADR-0515 D3. Item (2)
supplied by ADR-0525.*
