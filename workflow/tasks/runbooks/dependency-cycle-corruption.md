---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-dependency-cycle-corruption
status: Accepted
date: 2026-05-17
owner_team: axis-tasks
severity_applicable: [Sev-2]
related_failure_modes: [FM-03]
related_dashboards: [throughput-and-engagement]
doc_status: published
---

# Runbook — Dependency-Cycle Corruption

## When this runbook fires

- `tasks_dependency_cycle_refused_count > 0` sustained on a previously-quiescent project (cluster of refusals adjacent task IDs), OR
- Tenant reports "I cannot add a dependency edge to my project; oyatie says cycle but I don't see one", OR
- Pre-import cycle scan refuses an import that previously imported cleanly elsewhere.

## Symptoms

- Tenant cannot mutate dependency-graph in affected project until corrupted edges identified + removed.
- `DependencyCycle::Refused` 422 errors on legitimate-looking edge writes.
- Pre-import cycle scan blocks legitimate imports.

## Probable causes

1. Legacy data imported pre-cycle-prevention contains cycles; new writes encounter them per ADR-TASKS-0002.
2. Race condition between concurrent edge writes created a brief cycle before serialisation.
3. Bug in cycle-detection algorithm (unlikely but possible).

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected project:
   ```promql
   tasks_dependency_cycle_refused_count{project_id_hashed!=""}
   ```
3. Run cycle-detection scan on project:
   ```bash
   oya tasks dependency-graph scan --project <project-id> --report-corrupt-edges --audit-reason "RB-dependency-cycle-corruption"
   ```
4. Review report: which edges form which cycles?

## Mitigation steps

### Step 1 — Surface corrupt edges to tenant operator

Generate per-cycle report:
```bash
oya tasks dependency-graph cycles --project <project-id> --format json > /tmp/cycles.json
```

Tenant operator reviews + decides which edges to remove.

### Step 2 — Remove corrupt edges (tenant-approved)

```bash
oya tasks dependency-graph remove-edge --from <task-id> --to <task-id> --kind <kind> --audit-reason "RB-dependency-cycle-corruption"
```

Per cycle, remove the minimal edge set to break the cycle.

### Step 3 — Re-validate

```bash
oya tasks dependency-graph scan --project <project-id> --audit-reason "post-mitigation"
```

Expected: zero cycles.

### Step 4 — Reactivate write path

The dependency-graph is now cycle-free; new writes resume without refusal.

### Step 5 — If global pattern (multiple projects affected by same root cause)

Run global re-scan per `backfill-replay.md` §"Dependency-graph cycle re-scan".

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| Cycles detected in project | 0 | post-mitigation |
| `tasks_dependency_cycle_refused_count` on legitimate writes | 0 | within 5 min |
| Bounded BFS cycle-check p99 | < 50ms | unchanged |

## Post-incident review

- Was the pre-import cycle scan effective for the originating source?
- Should we re-run cycle scan globally?
- Update LEAN check `oya-check-dependency-graph-cycle-prevention` if a new attack pattern was discovered.
- Was the corruption due to legacy data (Hyrum #3 deliberate strengthening) or genuine bug?

## Drills

- Bi-annual: simulated legacy-import with known cycles in synthetic tenant; verify scan + report + remediation flow.

## References

- `failure-modes.md` FM-03.
- ADR-TASKS-0002 (dependency-cycle prevention at write time).
- Hyrum #3 in `migration-from-connect.md`.
- `dashboards/throughput-and-engagement.json`.
