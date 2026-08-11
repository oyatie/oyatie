---
doc_class: JudgmentNote
title: Elevate oyatie-0s8 capability-root hub deltas off integ/cell
status: Accepted
date: 2026-08-11
related_artifacts:
  - cell/manifest.json
ssot_todo: tip-free-packets
---

# Hub elevate (envelope hygiene)

Stripped out-of-envelope hub edits from `integ/cell` tip so Claim/hub-exclusivity can pass.
Prior tip incorrectly removed `.claude/workflows/` from `specs/reachability-registry.json`
while adding `cell/manifest.json`, which firewall-redded as 6 unreachable/unjustified
regressions under `.claude/workflows/**` — that removal must never return on this tip.

## Elevated (do NOT re-add on this tip)

- `specs/reachability-registry.json` — add exact `prefix: cell/manifest.json` (oyatie-0s8 / ADR-0562); do **not** delete `.claude/workflows/`
- `ci/facade/product-protocol-policy/product-protocol-policy.json` — expected_total/live_v1 +1 for `cell/manifest.json`

Forever owners: reachability → `integ/specs` (tip-free); product-protocol → `integ/ci`.
Consumer unblocked: product rails depending on `cell`.
