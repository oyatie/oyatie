---
doc_class: JudgmentNote
title: naming_sweep dual-emit CI titles (sweep-execute)
status: Accepted
date: 2026-08-11
ssot_todo: sweep-execute
---

# Dual-emitted forever check titles (this tip)

Mechanism: job `if:` + `uses:` only (automation-language-policy blocks new workflow `run:` keys).

| Legacy | Forever |
| --- | --- |
| `oya-ci-required` | `merge-admission-required` |
| `freshness (lock + generated faces, ADR-0539)` | `generated-artifact-freshness (lock + faces)` |
| `cloud-ci-firewall (baseline ratchet + gate-registration meta-test)` | `admission-baseline-ratchet (+ gate-registration)` |
| `gate · affected-set (ADR-0554, binding workspace coverage)` | `affected-target-set (binding workspace coverage)` |
| `integrity-canary (cold, from-empty — ADR-0556 D2 / ADR-0560)` | `cache-integrity-canary (cold from-empty)` |

# Explicitly NOT done here

- Branch-protection context flip → **PAUSE-AND-PAIR** (founder only).
- Workflow file rename `oya-ci-required.yml` → `merge-admission-required.yml` → needs `workflow_call` extraction (BAN double full CI).
- Crate/bin/fixture path renames → destination-integ waves; judgments already `done`.
