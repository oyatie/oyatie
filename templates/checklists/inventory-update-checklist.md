---
doc_class: Checklist
checklist_id: CHK-INV
status: pending approval
purpose: |
  Inventory ledger update at every cutover / migration phase. Records source → archive → tombstone transitions for files, crates, contracts, and capabilities. Lifts from `.omc/scratch/inventory-draft-oyatie-cutover.md` shape.
lift_target: oyatie/templates/checklists/inventory-update.md
enforcing_fitness_lane: oya-governance-inventory-tracker
owner_team: axis-foundry
related:
  - .omc/scratch/inventory-draft-oyatie-cutover.md
  - .omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md
  - /templates/implementation-plan-template.md
---

# Inventory Update Checklist

> Append a row to the active inventory ledger for every artifact moved, archived, or deleted in a cutover / migration phase. The row is the audit row; absent rows are flagged by `oya-governance-inventory-tracker`.

## When to update

- Every cutover phase (e.g., M01-P08 agentic-pipeline cutover).
- Every migration phase (schema, crate-rename, contract-rename, capability-rename).
- Every archive event (file moved to `archive/` tree).
- Every retirement event (file removed from active tree).
- Every bootstrap-window invocation per `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md §Cutover bootstrap window`.

## Row schema

```yaml
- inventory_row_id: INV-NNNN
  date: YYYY-MM-DD
  phase: M01-P08 | M0N-P0M | ...
  ip_ref: IP-NNN-<slug>
  action: keep | move | archive | delete | rename | recreate-forbidden
  source_path: <repo-relative path>
  target_path: <repo-relative path | null>
  archive_path: archive/pre-grit-cutover-2026-05-12/... | null
  tombstone:
    enabled: true | false
    forbid_recreation_lane: oya-governance-legacy-path-recreation
  bootstrap_window: true | false
  invocation:
    primitive: grit | icm | oya-tooling-agent-read | human-orchestrator-carve-out
    command: "<sanctioned command or human-only carve-out description>"
    purpose: "<one-line>"
    actor:
      kind: agent | human-orchestrator
      id: "<agent-id | role>"
    agent_direct_vcs_or_forge: false
  audit_emission_id: EVT-INV-<ulid>
  rollback_path: "<exact reverse command, if applicable>"
  notes: "<optional one-paragraph>"
```

## Pre-flight (before the inventory action)

- [ ] **I1** Identify action class (`keep | move | archive | delete | rename | recreate-forbidden`).
- [ ] **I2** Determine `source_path`, `target_path`, `archive_path`.
- [ ] **I3** Determine if action requires `bootstrap_window: true` (only during phases P0.5 / P1 / P2 of the agentic-pipeline cutover, or other explicit carve-outs).
- [ ] **I4** Identify actor (agent vs human-orchestrator); if human-orchestrator action, confirm authorization per `docs/RACI-OWNERSHIP.md` row `human-orchestrator-cutover`. Direct VCS/forge tools are never agent-callable inventory primitives; record any human-only carve-out as `primitive: human-orchestrator-carve-out`.
- [ ] **I5** Draft the rollback command.

## During action

<!-- agent-instructions:start -->
- [ ] **I6** Emit `icm store -t direct-tool-invocations -c "<one-line rationale>" -i high -k "<primitive>,<context>"` **BEFORE** invocation (per `MASTERPLAN.md §2 principle 12`).
- [ ] **I7** Execute the action via the documented primitive (`grit` / `icm` / `oya-tooling-agent-read` for steady-state; documented carve-out commands for bootstrap-window or human-orchestrator).
- [ ] **I8** Capture stdout via `oya-tooling-agent-read run-evidence <cmd>`.
<!-- agent-instructions:end -->

## Post-action

- [ ] **I9** Append the row to the active inventory ledger (path TBD on lift: e.g., `docs/inventory/active-ledger.yaml`).
- [ ] **I10** Verify audit chain emitted `EVT-INV-<ulid>`. *Lane:* `oya-governance-audit-emission`.
- [ ] **I11** Confirm tombstone: if `action: archive | delete`, `oya-governance-legacy-path-recreation` lane refuses any future recreation at `source_path`.
- [ ] **I12** Confirm symmetry: if `action: move`, both `source_path` (now absent) and `target_path` (now present) are honored by the appropriate lanes. *Lane:* `oya-governance-inventory-tracker`.
- [ ] **I13** Update the IP `§Symbols to grit-claim` if grit symbols moved with the file (per ADR-0054-grit-scaffold-claim-pattern).
- [ ] **I14** Update `docs/CHANGELOG.md` if a canonical doc was moved/archived/deleted.

## Sample row (move + archive)

```yaml
- inventory_row_id: INV-0042
  date: 2026-05-13
  phase: M01-P08
  ip_ref: IP-007-archive-omx-ultragoal
  action: archive
  source_path: example/legacy/source-ledger.jsonl
  target_path: null
  archive_path: archive/pre-grit-cutover-2026-05-12/example-source-ledger.jsonl
  tombstone:
    enabled: true
    forbid_recreation_lane: oya-governance-legacy-path-recreation
  bootstrap_window: false
  invocation:
    primitive: oya-tooling-agent-read
    command: "oya-tooling-agent-read archive --from example/legacy/source-ledger.jsonl --to archive/pre-grit-cutover-2026-05-12/example-source-ledger.jsonl"
    purpose: "archive legacy ultragoal ledger per ADR-0053"
    actor:
      kind: agent
      id: planner-agent
  audit_emission_id: EVT-INV-01HX...
  rollback_path: "oya-tooling-agent-read unarchive --from <archive_path> --to <source_path>"
```

## Anti-patterns

- Skipping the row because "it's a small file" — every action gets a row.
- Bundling multiple actions in one row — one action per row, one rationale per row.
- Bootstrap-window invocation without `bootstrap_window: true` flag — auto-fails the lane.
- Recreating a legacy retired path — `oya-governance-legacy-path-recreation` refuses; escalate per `escalation-checklist.md` to amend the inventory if intentional.
