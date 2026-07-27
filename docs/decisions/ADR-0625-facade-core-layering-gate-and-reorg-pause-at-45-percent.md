---
id: ADR-0625
title: "Enforce ADR-0562's facade→core layering rule, and record the capability migration as intentionally paused at ~45%"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-07-26
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0328]
depends_on: [ADR-0562, ADR-0615]
related: [ADR-0245, ADR-0280, ADR-0512, ADR-0551, ADR-0554]
related_specs:
  - /specs/capability-registry.json
milestone: W0
---

# ADR-0625 — Enforce the facade→core layering rule; record the migration as paused at ~45%

## Context

ADR-0562 makes one rule central: a `facade` crate reaches its own capability's `core` **only through
`ports`**. That rule has been stated since ADR-0562 was accepted and enforced by **nothing**.

Measured on `origin/dev` at 2026-07-26, from the authoritative buck2 graph:

| | count |
|---|---|
| facade packages with a same-capability `facade → core` edge | **35** |
| …of which sit in a capability with **no `ports/` layer at all** | **5** (`compute` 3, `intelligence` 2) |
| genuine violations of the rule as written | **30** |
| target-level edges behind those 35 packages | 156 |

Two facts shaped the design:

1. **A `Cargo.toml` scan does not see the rule.** `intelligence/facade/worker/BUCK` carries the edge
   at lines 8 and 22 with **zero** Cargo path-dependencies. A manifest-keyed gate is blind to it.
   Detection must read the build graph.
2. **The migration that was supposed to fix this has stopped.** Roughly 45% of crates have moved to
   capability-first homes. Finishing changes no architectural fact: a fully migrated tree is exactly
   as unenforced as today's, because the enforcement was never the relocation — it was the rule.

Separately, an unrelated observation constrains what this ADR may promise: a `frozen_empty` gate code
is **not** ungrandfatherable. Both firewall predicates
(`ci/facade/baseline-ratchet/src/lib.rs:584` and `:525`) read `!signoff.is_signed_off(...)` with no
`frozen_empty` special case, and the module documents that as deliberate. So no gate in this repo can
make the pause mechanically irreversible; this ADR therefore records the pause rather than claiming
to enforce it.

## Decision

**1. Ship a gate for the layering rule.** `ci/facade/facade-core-layering` freezes the 35 current
violators and makes a **new** one born-blocking. Two codes, deliberately separate:

- `facade_core_direct_dep` — the 30 genuine violations.
- `facade_core_no_ports_layer` — the 5 in `compute` and `intelligence`, which ADR-0562 §10.6 declares
  dependency-legal *while no ports layer exists*. A distinct code means introducing a ports layer
  **closes that code cleanly** instead of reading as a regression against the other.

**2. Baseline keys are cargo package names.** Not paths, and not buck2 target labels — a label embeds
the path (`//intelligence/facade/worker:…`), so one future capability move would invalidate every
baselined entry at once and the repair would be indistinguishable from laundering.

**3. Detection statically parses `BUCK`.** Verified to reproduce the `buck2 uquery` result exactly —
35 packages, identical per-capability split — *before* the gate was written. Static parsing keeps the
gate hermetic (no subprocess), which `gate-self-conformance` requires.

**4. Record the capability migration as intentionally paused at ~45%.** This amends **ADR-0328**, the
sequence authority. (It does **not** amend ADR-0562 §10: ADR-0562:37 states "the migration contract
in §10 is reference, not execution.") The remaining crates stay where they are until a *product*
reason touches them. What is given up, explicitly and not deferred: legacy `cloud/`, `oya/`, `libs/`,
`tools/` keep crates indefinitely; `base/` is not created; `libs/oya-shared-*` (51) and
`libs/oya-http-*` (8) stay put; the capability registry stays partly stale.

This is a **two-way door**. Resuming is a normal decision, not a repair.

## Alternatives considered

- **Finish the migration first, then enforce.** Rejected: enforcement does not depend on relocation,
  and each batch actively *removes* crates from tier enforcement (`owning_service()` classified only
  `cloud`/`oya` until #1423). Finishing first means enforcing later over a larger unenforced surface.
- **Gate the rule with a `frozen_empty` code so the pause cannot be reversed silently.** Rejected on
  evidence: `frozen_empty` is signoff-exempt (see Context), and the signoff door already carries two
  `pending-founder-ratification` entries written by agents. Claiming it as a mechanism would be false.
- **Include cross-capability `facade → core` edges.** Rejected *for now*: 12 packages / 21 edges reach
  another capability's core (`marketplace/facade/dev-cli` alone reaches 9). That is a different rule
  with a different remedy — it is about capability coupling, not face layering — and folding it in
  would triple the baseline while blurring what a violation means. Recorded here as known and
  ungated.

## Consequences

- The 35 violators persist but **cannot grow**. The baseline is shrink-only, and a repaired entry is
  reported until its baseline row is removed, so repairs cannot silently leave the slot open.
- The tree stays dual-state indefinitely.
- The pause is a **record, not a mechanism**. A future agent reading ADR-0562 will find an unfinished
  mandate; this ADR is what tells them it was deliberate. If migration resumes, the package-name keys
  survive the moves.

## Files introduced by this decision

- `ci/facade/facade-core-layering/BUCK`
- `ci/facade/facade-core-layering/Cargo.toml`
- `ci/facade/facade-core-layering/OWNERS`
- `ci/facade/facade-core-layering/facade-core-layering-policy.json`
- `ci/facade/facade-core-layering/src/lib.rs`
- `ci/facade/facade-core-layering/src/main.rs`
- `ci/facade/facade-core-layering/tests/facade_core_layering.rs`
- `docs/decisions/ADR-0625-facade-core-layering-gate-and-reorg-pause-at-45-percent.md`
