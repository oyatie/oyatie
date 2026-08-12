---
doc_class: JudgmentNote
title: naming_sweep execute beyond dual-emit (sweep-execute)
status: Accepted
date: 2026-08-11
ssot_todo: sweep-execute
---

# Beyond dual-emit (this tip)

| Row | Action on tip | Notes |
| --- | --- | --- |
| CI titles dual-emit | already on tip | protection flip = **PAUSE-AND-PAIR** |
| OP-adr-in-job-title | legacy job/step `name:` ADR numbers stripped | forever dual-emit titles unchanged |
| workflow file forever path | `merge-admission-required.yml` born (`workflow_call` only) + `oya-ci-required.yml` gains `workflow_call` | event triggers stay on legacy filename until quiet-window cutover (**BAN double full CI**) |
| fixture corpus | dir already forever `specs/fixtures/ci-baseline-ratchet/` | loader prefers forever path with legacy fallback (fixes tip that moved dir without loader) |
| gate ids / bins / brand-prefix crates | **deferred** | need alias targets + baseline table migration; not safe mid-babysit |

# Explicitly NOT done

- Branch-protection context rename flip
- Sole event-entry rename cutover (move triggers onto forever filename)
- `cloud-ci-*` `[[gates.enabled]]` id renames
- `oya-cloud-ci-*-bin` renames (need buck alias wave)
