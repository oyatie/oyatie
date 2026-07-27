---
id: ADR-0627
title: "Enforce ADR-0562's facade→core layering rule, keyed to survive the remaining capability migration"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-07-26
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0562, ADR-0615]
related: [ADR-0245, ADR-0280, ADR-0328, ADR-0512, ADR-0551, ADR-0554]
related_specs:
  - /specs/capability-registry.json
milestone: W0
---

# ADR-0627 — Enforce the facade→core layering rule, keyed to survive the remaining migration

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
2. **Relocation was never the enforcement.** Roughly 45% of crates have moved to capability-first
   homes, and the rule is violated at the same rate on both sides of the line. The gate is therefore
   needed independently of, and concurrently with, the migration rather than after it.

Separately, an unrelated observation constrains what this ADR may promise: a `frozen_empty` gate code
is **not** ungrandfatherable. Both firewall predicates
(`ci/facade/baseline-ratchet/src/lib.rs:584` and `:525`) read `!signoff.is_signed_off(...)` with no
`frozen_empty` special case, and the module documents that as deliberate. So no gate in this repo can
claim an irreversible mechanism. Recorded here because it bounds what any gate in this repo may
assert: a `frozen_empty` code is a signed-off-able speed bump, not a hard stop.

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

**4. The capability migration continues to completion.** This ADR originally recorded it as
intentionally paused at ~45%. That is **reversed** by founder decision 2026-07-27: the destination is a
single consistent tree, because dual state taxes every placement decision, every doc reference, and
every new crate's home — a standing cost on product work rather than on the migration.

This ADR therefore asserts **no pause** and does not amend ADR-0328. What it records instead is why
this gate ships *now* rather than after the migration, and why decision 2's package-name keying exists
**precisely because the remaining crates will move**: a buck2 label embeds its path, so a label-keyed
baseline would be invalidated wholesale by the next batch and the repair would be indistinguishable
from laundering. Package names survive relocation, so this baseline should outlive every remaining
move without a single re-key.

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
- The tree stays dual-state **until the migration completes**, which this ADR does not schedule — it
  only stops asserting a pause that is no longer the decision.
- **The baseline is migration-proof by construction.** Package-name keys mean the remaining moves need
  no re-key, no signoff, and no baseline churn.
- The 5 `facade_core_no_ports_layer` entries close by *introducing a ports layer*, not by relocating,
  so migration neither closes nor worsens them.

## Files introduced by this decision

- `ci/facade/facade-core-layering/BUCK`
- `ci/facade/facade-core-layering/Cargo.toml`
- `ci/facade/facade-core-layering/OWNERS`
- `ci/facade/facade-core-layering/facade-core-layering-policy.json`
- `ci/facade/facade-core-layering/src/lib.rs`
- `ci/facade/facade-core-layering/src/main.rs`
- `ci/facade/facade-core-layering/tests/facade_core_layering.rs`
- `docs/decisions/ADR-0627-facade-core-layering-gate.md`
