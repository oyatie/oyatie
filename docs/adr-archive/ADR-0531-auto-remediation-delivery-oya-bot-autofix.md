---
id: ADR-0531
title: "Auto-remediation delivery + safety model: the oya-bot-autofix PROPOSE-only fleet member, shrink-only burn-down, HMAC webhook kernel, full-fan-in human merge gate; consistent with ADR-0091 default-deny write-gate (referenced by canonical name)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0515, ADR-0528, ADR-0529]
amends: []
related: [ADR-0091, ADR-0515, ADR-0516, ADR-0528, ADR-0529, ADR-0535]
related_specs:
  - /specs/masterplan.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0531: Auto-remediation delivery — the oya-bot-autofix PROPOSE-only fleet member

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 2 of ADR-0516. Consistent with ADR-0091's default-deny write-gate posture,
referenced here by its canonical name `oya-governance-pr-merge-gate-kernel`.

## Context

ADR-0528 makes `remediate()` produce *described* edits; some privileged process must APPLY them. That
process is a security and supply-chain surface (auto-fix on a third party's repo). The dangerous
write capability must live in exactly one auditable, PROPOSE-only place behind a clean HMAC kernel and
a human merge gate — never the legacy/forbidden-vocab webhook gateway.

## Decision

The privileged delivery process that APPLIES `remediate()` output (ADR-0528) is `oya-bot-autofix`, a
member of the repo-automation-bot fleet alongside `oya-bot-depupdate` / `oya-bot-release`, sharing the
host, signed-capability trust model, and merge-gate kernel. Five binding safety invariants:

1. **DETERMINISTIC + REPRODUCIBLE** — same face ⇒ byte-identical edits, tested with the RED/GREEN
   fixture discipline; the producer's determinism (committed==regenerated, registry-drift, ADR-0515)
   is the precondition.
2. **PROPOSED, SIGNED, REVIEWABLE PR** — never silent in-place mutation; built on the clean
   `oya-shared-webhook-delivery-kernel` (HMAC-SHA256), NOT any forbidden-vocab gateway; merge gated by
   the FULL `oya-ci-required` fan-in via the `oya-governance-pr-merge-gate-kernel` (the canonical name
   for ADR-0091's write-gate); wide-blast AutoFix (renames, deletions) requires human approve and is
   NEVER auto-merged.
3. **DRY-RUN/PREVIEW** — `oya-ci fix --dry-run` renders the diff without proposing.
4. **REVERSIBLE + IDEMPOTENT** — each PR is a revertible commit; re-running `remediate()` on the
   post-fix face MUST converge to `None`.
5. **BURNS DOWN the ratchet** — the bot proactively closes `tolerated` baselined keys via scheduled
   (cadenced, not reactive-only) burn-down PRs, moving keys `tolerated → fixed` and SHRINKING the
   committed baseline; it NEVER adds keys, so it can never touch `ratchet_growth`/`_sign_off_additions`
   (the ADR-0515 firewall predicates need no edit).

HARD SAFETY FENCE: the bot has write access to PROPOSE ONLY — it cannot merge (gated kernel) and cannot
bypass any gate (a bot's CI verdict can only ADD findings); `remediate()` runs in the same no-write
default-deny sandbox as `evaluate_keyed`. THIRD-PARTY SUPPLY-CHAIN: a third-party gate-pack ships its
remediation through the same signed + attested + capability-declared contract; a pack requesting WRITE
capability is default-DENIED at registration; the bot's PR is subject to the adopter's full fan-in +
human merge.

## Governed surfaces

The following repo paths are governed by this ADR. The accounting gate validates that each is justified
(this ADR is the justification reference):

`Cargo.lock`
`registry/catalog/oya-bot-autofix-app.yaml`
`specs/capability-registry.json`
`tools/oya-bot-autofix-app/BUCK`
`tools/oya-bot-autofix-app/Cargo.toml`
`tools/oya-bot-autofix-app/OWNERS`
`tools/oya-bot-autofix-app/src/lib.rs`
`tools/oya-bot-autofix-app/src/main.rs`
`tools/oya-bot-autofix-app/tests/dry_run.rs`

## Drivers

- The auto-fix-on-a-third-party-repo trust/security + supply-chain surface demands the dangerous
  capability live in exactly one auditable, PROPOSE-only place behind the clean HMAC kernel and the
  human merge gate.
- The firewall's small trusted base must stay literally unedited (ADR-0528).
- Burn-down is the engine that DRIVES the shrink the ratchet already permits.

## Alternatives considered

- **Bot writes to main directly** — rejected (silent, unreviewed — Option C of ADR-0528).
- **Reuse the legacy/forbidden-vocab webhook gateway** — rejected (forbidden by the vocab-eradication
  campaign).
- **Per-finding reactive-only fixes** — rejected (debt never burns down; scheduled cadence chosen).

## Consequences

A conformance test asserts (a) a write-capability gate-pack is REJECTED at registration and (b) an
`oya-bot-autofix` PR CANNOT merge while any gate is RED; `oya-bot-autofix` is one bot in the fleet
(W2); the engine is hermetic + cacheable because `remediate()` runs as a sandboxed buck2 action with
declared inputs. Consistent with ADR-0091 (referenced by the canonical
`oya-governance-pr-merge-gate-kernel` name, never the legacy heading). The dep/release bots of the
fleet are detailed in ADR-0535. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE.md (RATIFY-TO-ADR). Consistent with
ADR-0091 (canonical name oya-governance-pr-merge-gate-kernel). Detail under Component 2 of ADR-0516.*
