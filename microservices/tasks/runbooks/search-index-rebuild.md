---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-search-index-rebuild
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-04, FM-13]
related_dashboards: [throughput-and-engagement]
doc_status: published
---

# Runbook — Search-Index Rebuild

## When this runbook fires

- Meilisearch cluster outage detected (`tasks_search_index_cluster_health != green`), OR
- `tasks_search_query_p99_ms > 1s` for > 5 min (degraded mode active), OR
- Rebuild job state `failed`, OR
- Cross-tenant data leak suspected (FM-13 trigger; Sev-1).

## Symptoms

- Cross-project search degraded; tenant sees direct-Postgres-trigram fallback.
- Search results slow but functional.
- For FM-13: cross-tenant data appearing in search results (Sev-1, immediate Cedar policy lockdown).

## Probable causes

1. Meilisearch cluster instability (node failure / OOM / disk-full).
2. Schema-incompatible rebuild attempt mid-flight.
3. Cedar policy change in search-projection invalidated per-tenant indexes.
4. Cross-tenant data leak (FM-13; tenant-prefix bug; Sev-1).

## Triage (within 15 min)

1. Acknowledge OnCall page.
2. Check cluster health:
   ```bash
   kubectl exec -n tasks meilisearch-0 -- curl localhost:7700/health
   ```
3. Check rebuild job state:
   ```bash
   oya tasks search-index status --pack <pack>
   ```
4. **If Sev-1 cross-tenant leak suspected**:
   - Immediately Cedar-deny search queries via `oya tasks policy deny --action search_query --pack <pack> --audit-reason "cross-tenant-leak-investigation"`
   - Page council-privacy + ops-security.
   - Run `oya tasks search-index audit --pack <pack> --tenant-prefix-verify`.

## Mitigation steps

### Step 1 — Degraded mode (already auto-active)

Confirm direct-Postgres-trigram fallback is active:
```promql
tasks_search_degraded_mode_active{pack="<pack>"} == 1
```

Tenant sees slower but functional search.

### Step 2 — Restore Meilisearch cluster

If node failure:
```bash
kubectl scale deployment -n tasks oya-tasks-meilisearch --replicas=5
```

If disk-full:
```bash
helm upgrade oya-tasks --set meilisearch.storage.size=1Ti
```

### Step 3 — Trigger full rebuild from Postgres

```bash
oya tasks search-index rebuild --tenant <hashed-id> --audit-reason "RB-search-index-rebuild"
```

Or pack-wide:
```bash
oya tasks search-index rebuild --pack <pack> --audit-reason "RB-search-index-rebuild"
```

Verify per AC-09 target: 10M tasks → ≤30 min.

### Step 4 — Per-tenant index validation

After rebuild, validate per-tenant index isolation:
```bash
oya tasks search-index audit --pack <pack> --tenant-prefix-verify
```

Expected: every index name carries correct tenant_id_hash prefix.

### Step 5 — Re-enable search queries

Once rebuild + audit complete:
```bash
oya tasks policy allow --action search_query --pack <pack> --audit-reason "post-rebuild"
```

### Step 6 — If FM-13 (cross-tenant leak)

Per `incident-response.md` §"Cross-tenant data leak (Sev-1)":
- Within 24h: notify affected tenants.
- Within 72h: GDPR DPA notification + PIPC + per-pack regulators.
- Post-incident review within 5 business days.
- Update LEAN check `oya-check-search-index-tenant-prefix` if a new defect class discovered.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `tasks_search_index_cluster_health` | green | within 10 min |
| `tasks_search_query_p99_ms` | < 300ms | within 30 min post-rebuild |
| `tasks_search_degraded_mode_active` | 0 | post-rebuild |
| Per-tenant index isolation audit | pass | post-mitigation |

## Post-incident review

- Was the rebuild timing within AC-09 target (30 min for 10M tasks)?
- Should degraded-mode auto-throttle search rate to prevent cascading load?
- If FM-13: was the LEAN check + property test adequate?

## Drills

- Quarterly: simulated Meilisearch cluster outage; verify degraded mode + rebuild flow.
- Annual: simulated cross-tenant leak (red-team).

## References

- `failure-modes.md` FM-04, FM-13.
- ADR-TASKS-0001 (search backend choice).
- PRD AC-09 (rebuild ≤30 min for 10M tasks).
- `dashboards/throughput-and-engagement.json`.
