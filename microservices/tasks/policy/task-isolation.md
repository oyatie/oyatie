---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: tasks
policy_id: POLICY-task-isolation
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-tasks
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-TASKS-0006]
related_artifacts:
  - microservices/tasks/policy/tenant-scope.cedar
  - microservices/tasks/policy/ci-scope.cedar
  - microservices/tasks/policy/auditor-scope.cedar
  - microservices/tasks/policy/public-read.cedar
  - microservices/tasks/policy/dual-context-isolation.cedar
  - microservices/tasks/policy/data-residency.md
doc_status: published
---

# Task Isolation Policy — tasks µservice

## Purpose

Define the per-tenant + dual-context (Personal / Professional) isolation invariants that bound how task data may flow across:
1. Tenant boundary (Tenant-A ↛ Tenant-B; no cross-tenant grant for tasks — tasks are not a cross-tenant primitive).
2. Context boundary (Personal ↛ Professional except via explicit projection).
3. Cross-jurisdiction boundary (pack-kr ↛ pack-eu / pack-us / etc. except via SCC; tasks themselves never replicate cross-pack).
4. **AI-decision boundary (T2 auto-assign refused in employment-context until EU AI Act conformity).**

This policy is enforced by:
- Postgres per-tenant Row-Level Security (RLS) — DB-layer prevention.
- Tenant-DEK envelope encryption — cryptographic prevention.
- Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`, `dual-context-isolation.cedar`) — runtime authorization.
- Rust type-system separation (`PersonalTask` ≠ `ProfessionalTask` at the type level) — compile-time prevention.
- LEAN CI lanes (`oya-check-context-isolation`, `oya-check-rls-coverage`, `oya-check-search-index-tenant-prefix`, `oya-check-foundry-bridge-cedar-gate`) — change-time prevention.

## Isolation Invariants

### Invariant 1 — Per-tenant isolation

> No task row from Tenant-A is ever returned by any Postgres / Meilisearch query made on behalf of Tenant-B. Unlike calendar's free/busy projection, tasks has NO cross-tenant grant mechanism — tasks are an internal-to-tenant primitive only.

**Enforcement:**
- Postgres RLS policy `task_tenant_isolation`:
  ```sql
  CREATE POLICY task_tenant_isolation ON tasks_task
    USING (tenant_id = current_setting('app.current_tenant')::uuid);
  ```
- Application connection-pool sets `app.current_tenant` from OIDC subject; bypass requires DB superuser (audited via OpenBao JIT).
- Meilisearch per-tenant index name + master key prefix.
- LEAN check `oya-check-rls-coverage` refuses build if any new table lacks an RLS policy.
- LEAN check `oya-check-search-index-tenant-prefix` refuses build if any Meilisearch write skips tenant prefix.

### Invariant 2 — Context isolation (Personal vs Professional)

> No Personal-context task field (title, description, custom-field values, comments, assignee, recurrence, automation) is ever visible to a Professional-context query, and vice versa, unless the user has explicitly opted-in via the `personal_to_professional_grant` flag at field-level granularity.

**Enforcement:**
- Rust kernel types: `oya-tasks-task-store-kernel` declares `PersonalTask` and `ProfessionalTask` as separate structs (NOT variants of a shared enum); no shared parent type that allows leakage.
- Repository ports: `TaskRepository::find_personal()` returns `Vec<PersonalTask>`; `find_professional()` returns `Vec<ProfessionalTask>`. A `find_all()` shared method does NOT exist.
- Cedar policy `task-isolation`:
  ```
  forbid (
    principal in TasksRole::"professional_reader",
    action == Action::"read",
    resource in TaskContext::"Personal"
  );
  forbid (
    principal in TasksRole::"personal_reader",
    action == Action::"read",
    resource in TaskContext::"Professional"
  );
  ```
- LEAN check `oya-check-context-isolation` refuses build if any usecase queries both contexts in a single transaction.

### Invariant 3 — Tenant-DEK encryption-at-rest

> All Professional-context task content (title, description, custom-field values, comments) is encrypted at rest under the per-tenant DEK per Bominal ADR-0111 envelope-encryption pattern.

**Enforcement:**
- DEKs issued by OpenBao; rotated 90d.
- Task-store writer wraps content in `Encrypted<T>` type; ciphertext bound to DEK ID + integrity check.
- DEK rotation event re-encrypts active records; old DEKs available read-only.
- LEAN check `oya-check-dek-binding-integrity` validates ciphertext binding.
- Personal-context E2E available when tenant has declared E2E; key remains on tenant-controlled HSM.

### Invariant 4 — Search-index per-tenant prefix

> Every Meilisearch index for tasks is named `tasks_<tenant_id_hash>`, scoped by a per-tenant master key, and refuses cross-tenant query at the Meilisearch ACL level. Cross-tenant data leak via search-index (FM-13) is structurally impossible.

**Enforcement:**
- Meilisearch master key per-tenant; issued via OpenBao; rotated 30d.
- Per-tenant index name includes tenant_id_hash.
- Adapter validates tenant prefix at every write + query path.
- LEAN check `oya-check-search-index-tenant-prefix` validates adapter; property test exercises adapter against malicious payload.

### Invariant 5 — Cross-jurisdiction isolation

> Task data resident in jurisdiction-pack-A (e.g., pack-kr) is NOT replicated to jurisdiction-pack-B (e.g., pack-eu) at the tasks µservice level. Cross-pack task creation (e.g., via workflow-engine bridge from pack-eu workflow to pack-kr task) is refused at Cedar unless tenant-executed SCC is on file AND the cross-µservice handoff has been approved.

**Enforcement:**
- Postgres + Valkey + Meilisearch clusters per pack; cross-cluster replication forbidden by default.
- Ingress routing: per-tenant pack tag derived at OIDC-token issuance; REST routes to per-pack cluster.
- LEAN check refuses cross-pack route at config layer.
- See `policy/data-residency.md` + `multi-region.md` for full enforcement chain.

### Invariant 6 — Audit-chain emission completeness

> Every task lifecycle mutation (Create / Update / StateChanged / Assigned / Commented / DependencyDeclared / BulkEdited / RecurrenceMaterialised / LegalHold / Export / Import / TimerTick) emits an audit-chain record (Ed25519 + Merkle per Bominal ADR-0028) before the mutation is acknowledged.

**Enforcement:**
- Usecase orchestrators call `audit_chain.emit()` before `repository.commit()`; transactional ordering enforced by `TaskLifecyclePort` trait.
- LEAN check `oya-check-audit-emission-coverage` traces every mutating usecase to its audit-emission call site.
- Audit-chain µservice acks emission via Workflow event; missing ack within 30s triggers `held` state.

### Invariant 7 — Legal-hold preservation

> When a `LegalHold` is opened on a task, the task + comments + history + dependency edges + time-tracking entries + audit-chain records are preserved indefinitely even when retention would otherwise expire. Hard-deletion is blocked at the Postgres role level; only `purge-with-2-person-rule` admin operation can hard-delete.

**Enforcement:**
- Postgres role `app_tasks_writer` has no DELETE permission; soft-delete only via `is_deleted=true` column.
- Hard-delete via `purge` admin script: requires 2-person OpenBao JIT approval + audit-chain emission pre-purge.
- Periodic integrity scan: compare hold-set vs Postgres rows; mismatch = critical alert.

### Invariant 8 — Dependency-graph cycle prevention at write time

> A dependency-edge write that would create a cycle MUST be refused with `DependencyCycle::Refused` 422 at write time. This is a 100% correctness SLO (no error budget) per PRD AC-02 + ADR-TASKS-0002.

**Enforcement:**
- `oya-tasks-dependency-graph-domain` invariant: bounded BFS cycle-check before commit; refuse on cycle detection.
- Postgres CHECK constraint as defence-in-depth.
- LEAN check `oya-check-dependency-graph-cycle-prevention` validates adapter wiring.
- Property test: write any random valid edge → never produces a cycle in the resulting graph (invariant testing across 1M iterations).

### Invariant 9 — Custom-field strict type coercion

> Custom-field writes MUST refuse type-mismatched values (e.g., string "1" written to a number-typed field) with `CustomFieldTypeMismatch::Refused` 422 per ADR-TASKS-0001.

**Enforcement:**
- `oya-tasks-task-store-domain` invariant: schema validation before commit; refuse on type-mismatch.
- Postgres jsonb constraints at column level.
- LEAN check `oya-check-custom-field-type-strict`.

### Invariant 10 — EU AI Act Annex III §4 employment-context auto-assign refusal

> T2 auto-assign in employment-context is REFUSED at the Cedar layer until ADR-TASKS-0006 conformity-assessment ADR ships per pack (pack-eu, pack-us, pack-kr employment-context, pack-jp, pack-au).

**Enforcement:**
- Cedar policy `dual-context-isolation.cedar` (auto-assign refusal section): refuses Action::CalendarT2AutoAssign + Action::TasksT2AutoAssign in employment-context until ADR-TASKS-0006 admits.
- Per pack overlay: pack-eu refuses unconditionally; pack-us refuses until fairness-audit; pack-kr employment-context refuses; pack-jp refuses pending Labour Standards Act review; pack-au refuses pending Fair Work Act review.
- LEAN check `oya-check-foundry-bridge-cedar-gate` validates Cedar gate is wired.

## Policy Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Compile-time | Rust type system (`PersonalTask` ≠ `ProfessionalTask`) | `cargo build` |
| PR-time | LEAN CI lanes (`oya-check-context-isolation`, `oya-check-rls-coverage`, `oya-check-search-index-tenant-prefix`, `oya-check-dek-binding-integrity`, `oya-check-audit-emission-coverage`, `oya-check-dependency-graph-cycle-prevention`, `oya-check-custom-field-type-strict`, `oya-check-foundry-bridge-cedar-gate`) | `oya gate validate` |
| Runtime (DB) | Postgres RLS | DB session |
| Runtime (search) | Meilisearch per-tenant master key | search query |
| Runtime (app) | Cedar policy evaluation | API request |
| Runtime (crypto) | Tenant-DEK envelope encryption | DEK validation |
| Audit | Ed25519 audit-chain seal | post-commit |

## Cedar Policy Files

| File | Purpose |
|---|---|
| `policy/tenant-scope.cedar` | Per-tenant scoping; refuses cross-tenant read/write |
| `policy/ci-scope.cedar` | CI-runner scope; refuses non-system reads outside per-changeset evidence |
| `policy/auditor-scope.cedar` | Auditor JIT scope; refuses cross-tenant pivot during engagement; EU AI Act notified-body scope |
| `policy/public-read.cedar` | Public-collection access; default-deny |
| `policy/dual-context-isolation.cedar` | Personal vs Professional + AI auto-assign EU AI Act refusal |

## Cross-µservice Boundary

This policy is binding for the `tasks` µservice only. Cross-µservice flows (tasks → mail; tasks → messenger; tasks → calendar; tasks → drive; tasks → workflow-engine; tasks → foundry-runtime; tasks → audit-chain) carry the isolation invariants forward via Workflow event signatures (each event carries `tenant_id_hashed`, `context_kind`, `pack_tag`, `principal`, `created_by_workflow_run_id` for workflow-bridge attribution).

## Verification + Drift Detection

| Verification | Cadence | Owner |
|---|---|---|
| Unit tests on Cedar policies | per-PR | axis-tasks |
| Integration tests on RLS | per-PR | axis-tasks |
| Property test: dependency-graph cycle prevention | per-PR | axis-tasks |
| Property test: custom-field strict coercion | per-PR | axis-tasks |
| Property test: search-index per-tenant prefix | per-PR | axis-tasks |
| Pen-test: cross-tenant search-index leak | Annually | ops-security |
| Pen-test: dual-context isolation | Annually | ops-security |
| Threat-hunt: cross-tenant Meilisearch queries | Weekly | axis-tasks |
| Threat-hunt: Personal-context fields in Professional queries | Weekly | axis-tasks |
| Audit-chain emission coverage scan | Per-deploy | observability |
| DEK rotation drill | Quarterly | ops-security |
| AI fairness drift audit (auto-assign-fairness SLO) | Weekly | axis-tasks + council-privacy |

## References

- ADR-0028 (Bominal): audit chain.
- ADR-0117: data residency.
- ADR-0135: Connect unbundle (dual-context).
- ADR-0140: Cedar policy substrate.
- ADR-TASKS-0001 (data model + strict coercion).
- ADR-TASKS-0002 (dependency cycle prevention).
- ADR-TASKS-0006 (AI auto-assign EU AI Act bounds).
- Bominal ADR-0111: envelope encryption.
- Bominal ADR-0231-0233: connect-tasks board + dependency + recurring inheritance.
- `microservices/tasks/threat-model.md`, `dpia.md`, `policy/*.cedar`.
- `microservices/calendar/policy/event-isolation.md` — sibling reference template.
