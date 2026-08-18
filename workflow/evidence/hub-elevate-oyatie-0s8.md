---
doc_class: JudgmentNote
title: Elevate oyatie-0s8 capability-root hub deltas off integ/workflow
status: Accepted
date: 2026-08-10
related_artifacts:
  - workflow/manifest.json
ssot_todo: tip-free-packets
---

# Hub elevate (envelope hygiene)

Stripped out-of-envelope hub edits from `integ/workflow` tip so Claim/hub-exclusivity can pass.

## Elevated (do NOT re-add on this tip)

- `specs/reachability-registry.json` — add exact `prefix: workflow/manifest.json` (oyatie-0s8 / ADR-0562)
- `ci/facade/product-protocol-policy/product-protocol-policy.json` — expected_total/live_v1 +1 for `workflow/manifest.json`

Forever owners: reachability → `integ/specs` (#1644 tip-free); product-protocol → `integ/ci` (#1646).
Consumer unblocked: product rails depending on `workflow`.

## Stabilize receipt (PR #1651)

- **When:** 2026-08-11
- **Dirty:** YES → restacked onto `origin/dev` @ `713dc2ea1` (#1933); tip merge-base CLEAN
- **Tip:** `d5fbf709d` `integ(workflow): restack onto origin/dev after #1933`
- **Domain fence:** `git diff --name-only origin/dev...HEAD` = `workflow/**` only (Cargo.lock / cross-integ / deepen: BAN honored)
- **In-domain CI:** prior real red was firewall `unjustified`+`unreachable` (+3 new paths × 2 codes). Paths still uncovered on base: `workflow/manifest.json`, `workflow/evidence/hub-elevate-oyatie-0s8.md`, `workflow/stale-path-hygiene-note.md`. Fix remains OOB tip-free: `specs/reachability-registry.json` prefix `workflow/manifest.json` (integ/specs) + product-protocol expected_total/live_v1 bump when all five roots land (integ/ci HOLD comment). Do NOT re-add hubs on this tip.
- **Land:** BAN merge-red — not Land-clean; no merge/APPROVE from this lane.
- **Consumer:** babysit/orchestrator — open tip-free integ/specs hub packet (or land-window with #1652–#1655) before re-admit; then re-run oya-ci-required.

