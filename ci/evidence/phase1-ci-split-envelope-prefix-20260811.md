---
doc_class: JudgmentNote
title: Phase1 ci-split-profiles + envelope-prefix-firewall
status: Accepted
date: 2026-08-11
ssot: domain_stack_integ_model_9b63d851.plan.md#merge-admission--domain-green-phase0-absorb
---

# Phase1 encode (on Phase0 tip)

## Forever public status string (ONE merge authority)

`merge-admission-required` — dual-emitted; matches forever workflow filename.
BP context flip `oya-ci-required` → `merge-admission-required` = **PAUSE-AND-PAIR** (document only; not flipped here).

## ci-split-profiles

| Profile | Public string | Merge authority? | Event triggers (this tip) |
| --- | --- | --- | --- |
| A domain | `domain.<envelope>.stabilize` | NO (LOCAL_GREEN) | none — `domain-stabilize.yml` is `workflow_call` only |
| B admission | `merge-admission-required` | YES (ADMIT_READY/SHIP) | still hosted by `oya-ci-required.yml` (BAN double CI) |

Quiet cutover checklist (do not execute without founder quiet window):

1. Dual-emit `merge-admission-required` green on recent tips.
2. Move `pull_request`/`push`/`merge_group`/`workflow_dispatch` onto `merge-admission-required.yml` via body move or `uses:`.
3. Leave `oya-ci-required.yml` as `workflow_call`-only (or delete after uses:).
4. PAUSE-AND-PAIR: flip required check context; retire dual-emit shims.
5. Never enable full event triggers on both files at once.

## envelope-prefix-firewall

`admission.policy` producer (`resolve_reachability`) now consumes
`specs/integ-branch-envelopes.json#roots.*.envelope_globs` as prefix allow
(source tag `envelope-prefix-ownership`). In-domain paths (e.g. `compute/**`, `iac/**`)
pass reachability **without** per-file tip-free / reachability-registry rows.

REACHED ⇒ JUSTIFIED still holds — envelope reach clears both `unreachable` and `unjustified`.

Transitional tip-free rows on integ/specs remain optional redundancy until this tip is on trunk;
they are no longer required for LOCAL_GREEN honesty.

## Explicit non-goals this commit

- Protection flip / moving event triggers (quiet cutover only)
- OWNERS generation from envelopes (separate todo)
- Buck2 domain-affected rewire (separate todo)
- Ship consolidate ritual automation (separate todo)
- Cargo.lock edits (BAN)
