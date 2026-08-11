---
doc_class: JudgmentNote
title: Elevate oyatie-0s8 capability-root hub deltas off integ/comms
status: Accepted
date: 2026-08-10
updated: 2026-08-11
related_artifacts:
  - comms/manifest.json
  - comms/evidence/hub-elevate-oyatie-0s8.md
  - comms/messenger/reorg-unit-judgments.v1.json
  - comms/stale-path-hygiene-note.md
ssot_todo: tip-free-packets
pr: 1653
---

# Hub elevate (envelope hygiene)

Stripped out-of-envelope hub edits from `integ/comms` tip so Claim/hub-exclusivity can pass.
Domain fence: `comms/**` only — do NOT deepen into `specs/**` or `ci/**` on this tip.

## CI evidence (run 31493102298 @ tip b77fee615)

Firewall `unjustified` + `unreachable` regressions (+4 / +4) name exactly these new artifacts:

- `comms/manifest.json`
- `comms/evidence/hub-elevate-oyatie-0s8.md`
- `comms/messenger/reorg-unit-judgments.v1.json`
- `comms/stale-path-hygiene-note.md`

Product-protocol: `entire_governed_manifest_corpus_is_inventoried_and_protocol_compatible` — live 97 vs expected 96 (delta = `comms/manifest.json`).

## Elevated (do NOT re-add on this tip)

- `specs/reachability-registry.json` — register exact prefixes for all four paths above (oyatie-0s8 / ADR-0555 / ADR-0562). `REACHED` clears both `unreachable` and `unjustified`.
- `ci/facade/product-protocol-policy/product-protocol-policy.json` — `expected_total` 96→97 (+ classify `comms/manifest.json`); bump `expected_live_v1_total` only if the new manifest is live_v1.

Forever owners: reachability → `integ/specs` (#1644 tip-free / #1933 plane); product-protocol → `integ/ci` (#1646).
Consumer unblocked: product rails depending on `comms` once those tips land and this PR restacks green.

## BAN on this tip

- `Cargo.lock` · cross-integ deepen · merge while `oya-ci-required` red
