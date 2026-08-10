---
doc_class: JudgmentNote
title: Elevate oyatie-0s8 capability-root hub deltas off integ/comms
status: Accepted
date: 2026-08-10
related_artifacts:
  - comms/manifest.json
ssot_todo: tip-free-packets
---

# Hub elevate (envelope hygiene)

Stripped out-of-envelope hub edits from `integ/comms` tip so Claim/hub-exclusivity can pass.

## Elevated (do NOT re-add on this tip)

- `specs/reachability-registry.json` — add exact `prefix: comms/manifest.json` (oyatie-0s8 / ADR-0562)
- `ci/facade/product-protocol-policy/product-protocol-policy.json` — expected_total/live_v1 +1 for `comms/manifest.json`

Forever owners: reachability → `integ/specs` (#1644 tip-free); product-protocol → `integ/ci` (#1646).
Consumer unblocked: product rails depending on `comms`.
