---
id: ADR-0525
title: "oya-ci hermetic buck2 execution: git-facts boundary (Option C, committed content-addressed face) + buck2-native gates + RBE/CAS hyperscale CI"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
amended_by: [ADR-0526]
depends_on: [ADR-0515, ADR-0392, ADR-0522]
amends: []
related: [ADR-0515, ADR-0392, ADR-0516, ADR-0522, ADR-0523, ADR-0526, ADR-0527]
related_specs:
  - /specs/phase0-ci-enforcement-baseline.json
  - /specs/masterplan.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0525: oya-ci hermetic buck2 execution — git-facts boundary, buck2-native gates, RBE/CAS

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Depends on / refines ADR-0515 D2/D3/D4/D5 and ADR-0392. Operationalizes the CI/CD reversal formerly
recorded as the (now-folded) ADR-0408. Supplies item (2) of the ADR-0523 irreducible-glue ledger.

## Context

Git is inherently ambient and cannot be a pure function of declared inputs inside a hermetic buck2
action. ADR-0515 established the firewall and the no-shell posture, and ADR-0392 made buck2 the
canonical build graph, but the concrete hermetic-execution model — how the producer and gates become
pure functions, where ambient git access lives, and how CI scales on RBE/CAS — was left abstract (the
CI/CD half formerly in the Proposed ADR-0408 predated this substrate).

## Decision

Four condensed parts:

**D1 — git-facts boundary (Option C).** Push ALL ambient git access to ONE out-of-graph emitter and
make every downstream action consume a frozen, content-addressed snapshot. A NEW non-hermetic
`rust_binary` `oya-cloud-ci-git-facts-emitter` (the four `Command::new("git")` calls moved verbatim out
of the producer) emits a COMMITTED canonical-JSON face `git-facts.generated.json` (schema
`oya-ci/git-facts/v1`: head_commit, head_time_secs, tracked_paths, last_touch_commit,
commit_author_ts_secs). It runs only as a CI `git-facts-regen` pre-step + a local regen hook, NEVER
inside a cacheable buck2 action. Tamper-evidence reuses the EXISTING `registry-drift` byte-diff.
Chosen over Option A (`.git` as a declared buck2 action input — poisons cache, ships `.git` to RBE) and
Option B (CI-only uploaded artifact — local build non-reproducible, violates ADR-0515 D3).

**D2 — producer + gates become pure functions of declared, content-addressed inputs.** Delete the four
git callsites + the `CliError::Git` variant from the producer; add a REQUIRED `--git-facts <path>` arg;
declare the tracked corpus as buck2 `srcs`/data. ERADICATE `env!("CARGO")` from all gate self-tests and
`cargo run` at runtime. Pure face-reader gates read the COMMITTED face via `$(location ...)`-resolved
env; regen/drift gates invoke the buck2-BUILT producer via `$(exe ...)`, never cargo, never git.

**D3 — cloud-native hyperscale CI on buck2 RBE/CAS, required-context NAME preserved.** CI runs
`buck2 test @affected` (the existing fail-closed affected-gate uquery-owner→rdeps closure) on RBE
(NativeLink recommended — Rust-native, self-hostable, aligns with ADR-0515 D5) + a remote CAS
read-through cache. The cargo→buck2 cutover is the migration's only door: swap the `oya-ci-required`
fan-in `needs:` from cargo legs to buck2 legs in ONE founder-paired commit while keeping the context
NAME `oya-ci-required` constant (branch protection untouched; one-line revert; cargo lanes kept as
non-required shadow for one soak then deleted).

**D4 — hard hermeticity invariants.** No `local_only`, no cargo fallback, no ambient git in any
action, no env-specific shims. Byte-parity preserved end-to-end (the hermetic producer reproduces
today's faces byte-for-byte).

## Drivers

- Founder directive: "hermetic and cloud native only, hyperscale pattern, refactor and redesign if it
  does not fit; must run buck2."
- ADR-0515 D3 (the git-facts emitter + affected-gate `.sh` ARE the documented narrow exception, run at
  the CI edge, not in actions), D4 (warm-cache / delta-cold throughput; hermetic actions or the cache
  is poisoned), D5 (owned-runner destination).

## Alternatives considered

- **Option A** (`.git` as a declared action input) — rejected (poisons cache, ships `.git` to RBE).
- **Option B** (CI-only uploaded artifact) — rejected (local build non-reproducible; violates
  ADR-0515 D3).
- **`local_only` buck2 + cargo fallback** (the founder-rejected product-plan idea; verified NEVER
  committed — zero `local_only = True` in any source BUCK file) — FORECLOSED by D4. Recorded so the
  rejected idea cannot resurface; NOT a supersession of committed canon.

## Consequences

The producer's declared-input set is effectively the whole tracked tree (correct — it is a whole-tree
analyzer), while the expensive gate consumers get PRECISE one-face inputs that cache-hit. Staging:
P0 (emitter + producer) → P1 (buck2-native gates) → P2 (byte/verdict-parity proof) → P3 (RE/CAS) →
DOOR (required-context content swap) → D5 (owned runner + Rust affected-driver port). Sequenced AFTER
the faces-only settle (orthogonal). This ADR supplies item (2) of the ADR-0523 glue ledger and is the
boundary ADR-0526 renames to scm-facts. It operationalizes the CI/CD reversal formerly carried by the
Proposed ADR-0408 (concrete affected-gate + NativeLink RE + required-context content swap); ADR-0408's
abstract framing is re-authored here and absorbed by ADR-0522/0523. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: OYA-CI-HERMETIC-EXECUTION-DESIGN.md
(RATIFY-TO-ADR). Refines ADR-0515 + ADR-0392; re-authors the former ADR-0408 CI/CD reversal; renamed
by ADR-0526.*
