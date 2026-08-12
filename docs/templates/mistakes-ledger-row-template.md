---
doc_class: Template
template_id: TPL-MFL
status: Accepted
date: 2026-05-12
purpose: |
  Canonical MFL-NNNN row shape for `docs/MISTAKES-LEDGER.md`. Every prevention-class learning gets exactly one row. The `mechanical_prevention` field is mandatory; process-only fixes are forbidden per CONSTITUTION §Decision principles Do.3.
enforcing_fitness_lane: oya-governance-mistakes-ledger-cite
owner_team: council-architecture
related:
  - docs/MISTAKES-LEDGER.md
  - docs/standards/prevention-doctrine.md
  - docs/INCIDENT-MANAGEMENT.md
  - docs/CONSTITUTION.md
adrs_cited:
  - ADR-0052  # inventory ledger (audit row contract parallels MFL row contract)
  - ADR-0053  # sanctioned primitives (mechanical-prevention examples)
doc_status: published
---

# MISTAKES-LEDGER row template

## Row schema (markdown table form)

The active ledger uses a table. New rows append to `docs/MISTAKES-LEDGER.md §3 Active ledger`:

```
| ID | Date | Mistake (1 line) | System gap (1 line) | Mechanical prevention | Shipped on | Link |
| MFL-NNNN | YYYY-MM-DD | <one line, no PII> | <what system/process/contract was missing> | <CI lane / hook / validator / fitness function name> | YYYY-MM-DD or `(target with <wave-gate>)` | <PR / ADR / runbook / postmortem link> |
```

## Field contract

| Field | Required | Format | Rule |
|---|---|---|---|
| `ID` | yes | `MFL-NNNN` sequential | Never reuse; never renumber. Next free slot at authoring time. |
| `Date` | yes | `YYYY-MM-DD` | When the mistake **surfaced**, not when prevention shipped. |
| `Mistake` | yes | ≤ 140 chars | No PII. No agent/human name. Describe the *failure mode*. |
| `System gap` | yes | ≤ 140 chars | What system, process, or contract was missing. |
| `Mechanical prevention` | **yes** | CI-lane name, hook script path, validator name, fitness function name | Process-only fixes are **REJECTED**. If you can only think of a process fix, escalate per `docs/standards/prevention-doctrine.md §6`. |
| `Shipped on` | conditional | `YYYY-MM-DD` if shipped; `(target with <wave-gate>)` if future-prevention | Future-prevention rows are permitted with a target wave-gate; must be reviewed quarterly. |
| `Link` | yes | PR# / ADR-#### / runbook path / postmortem path | At least one link. |

## YAML form (for `machine-readable/mistakes.json` mirror)

```yaml
- id: MFL-NNNN
  date: YYYY-MM-DD
  mistake: |
    One-line description of the failure mode. No PII. No names.
  system_gap: |
    One-line description of the missing system / process / contract.
  mechanical_prevention:
    kind: ci_lane | hook | validator | fitness_function | schema_check | runtime_gate
    name: oya-governance-<lane> | scripts/hooks/<hook>.mjs | <validator-name>
    enforced_at: pre-commit | pr-time | merge-gate | runtime
  shipped_on: YYYY-MM-DD | "target:<wave-gate>"
  links:
    - kind: pr | adr | runbook | postmortem | issue
      ref:  "#NNN" | ADR-#### | runbooks/<path>.md | incidents/<id>
  related_mfl: [MFL-NNNN, ...]
  pattern_cluster: <optional cluster-id assigned at quarterly council review>
```

## Example

```
| MFL-0010 | 2026-05-09 | RUNBOOKS-INDEX referenced 49 P0 runbook files that did not exist on disk after cleanup | No CI gate verifying RUNBOOKS-INDEX entries resolve to real files | `oya-governance-runbook-index-resolves` lane + per-runbook stub authoring at index-update time | shipped 2026-05-09 | per Codex Round 2 verdict + RUNBOOKS-INDEX §1 |
```

## Authoring rules

1. **One row per failure mode**, never bundle two distinct mistakes into one row.
2. **No PII, no personal names** — root cause is *systems and processes*, not people (CONSTITUTION §Decision principles).
3. **Mechanical prevention is mandatory** — if you cannot name a CI lane / hook / validator / runtime gate, file an escalation (`templates/checklists/escalation-checklist.md`). ADR-0053 sanctioned primitives are valid prevention mechanisms.
4. **Cite at least one link** — PR, ADR, runbook, or postmortem.
5. **Quarterly council review** — patterns across rows (≥ 3 rows in the same cluster) trigger meta-prevention per `docs/standards/prevention-doctrine.md §6`.
6. **PR cite rule** — PRs that ship a mechanical prevention for a prior failure **MUST** cite the new MFL row in `## Traceability` per D17 of `docs/AGENTS.md §Done-Definition checklist`.
