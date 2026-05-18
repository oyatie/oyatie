---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: tasks
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-TASKS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0003, ADR-TASKS-0004, ADR-TASKS-0005, ADR-TASKS-0006]
related_specs: [/specs/microservices/tasks.json, /specs/microservices/tasks/tasks.json]
owner_team: axis-tasks
date: 2026-05-17
doc_status: published
---

# Migration: `oya-connect-tasks-*` → `oya-tasks-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **tasks** µservice. It is the
consumer-facing companion to ADR-0134 (cross-µservice migration policy)
and ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-
proven in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-tasks-*` crate family under `microservices/tasks/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-TASKS accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3) |
| Reason | ADR-0132 no-suite forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 7-BC tasks surface (task-store / project-list / view-engine / dependency-graph / recurrence / search-index / importers) is only addressable at µservice granularity, not at Connect-suite granularity |
| Migration owner (Churn Rule) | axis-tasks |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 7 bounded-contexts of the `tasks` µservice live under
`microservices/tasks/src/crates/` per ADR-0131. The legacy
`oya-connect-tasks-domain` crate (the only legacy crate currently extant
per `find crates -maxdepth 1 -type d -name 'oya-connect-tasks-*'`) is
split per BC into per-domain crates.

### Crate import-path map

| Legacy `oya-connect-tasks-*` path | New `oya-tasks-*` path |
|---|---|
| `oya-connect-tasks-domain` | (split per BC; see note below) |
| `oya-connect-tasks-task-kernel` (Phase 2 adapter authoring) | `oya-tasks-task-store-kernel` |
| `oya-connect-tasks-task-domain` | `oya-tasks-task-store-domain` |
| `oya-connect-tasks-task-usecase` | `oya-tasks-task-store-usecase` |
| `oya-connect-tasks-task-api` | `oya-tasks-task-store-api` |
| `oya-connect-tasks-task-adapter` | `oya-tasks-task-store-adapter` |
| `oya-connect-tasks-task-adapter-postgres` | `oya-tasks-task-store-adapter-postgres` |
| `oya-connect-tasks-task-rest` | `oya-tasks-task-store-rest` |
| `oya-connect-tasks-task-worker` | `oya-tasks-task-store-worker` |
| `oya-connect-tasks-task-sdk` | `oya-tasks-task-store-sdk` |
| `oya-connect-tasks-task-app` | `oya-tasks-task-store-app` |
| `oya-connect-tasks-project-kernel` | `oya-tasks-project-list-kernel` |
| `oya-connect-tasks-project-domain` | `oya-tasks-project-list-domain` |
| `oya-connect-tasks-project-usecase` | `oya-tasks-project-list-usecase` |
| `oya-connect-tasks-project-api` | `oya-tasks-project-list-api` |
| `oya-connect-tasks-project-adapter` | `oya-tasks-project-list-adapter` |
| `oya-connect-tasks-project-adapter-postgres` | `oya-tasks-project-list-adapter-postgres` |
| `oya-connect-tasks-project-rest` | `oya-tasks-project-list-rest` |
| `oya-connect-tasks-project-app` | `oya-tasks-project-list-app` |
| `oya-connect-tasks-view-kernel` | `oya-tasks-view-engine-kernel` |
| `oya-connect-tasks-view-domain` | `oya-tasks-view-engine-domain` |
| `oya-connect-tasks-view-usecase` | `oya-tasks-view-engine-usecase` |
| `oya-connect-tasks-view-api` | `oya-tasks-view-engine-api` |
| `oya-connect-tasks-view-adapter` | `oya-tasks-view-engine-adapter` |
| `oya-connect-tasks-view-adapter-redis` | `oya-tasks-view-engine-adapter-redis` |
| `oya-connect-tasks-view-rest` | `oya-tasks-view-engine-rest` |
| `oya-connect-tasks-view-app` | `oya-tasks-view-engine-app` |
| `oya-connect-tasks-dependency-kernel` | `oya-tasks-dependency-graph-kernel` |
| `oya-connect-tasks-dependency-domain` | `oya-tasks-dependency-graph-domain` |
| `oya-connect-tasks-dependency-usecase` | `oya-tasks-dependency-graph-usecase` |
| `oya-connect-tasks-dependency-api` | `oya-tasks-dependency-graph-api` |
| `oya-connect-tasks-dependency-adapter` | `oya-tasks-dependency-graph-adapter` |
| `oya-connect-tasks-dependency-rest` | `oya-tasks-dependency-graph-rest` |
| `oya-connect-tasks-dependency-app` | `oya-tasks-dependency-graph-app` |
| `oya-connect-tasks-recurrence-kernel` | `oya-tasks-recurrence-kernel` |
| `oya-connect-tasks-recurrence-domain` | `oya-tasks-recurrence-domain` |
| `oya-connect-tasks-recurrence-usecase` | `oya-tasks-recurrence-usecase` |
| `oya-connect-tasks-recurrence-api` | `oya-tasks-recurrence-api` |
| `oya-connect-tasks-recurrence-adapter` | `oya-tasks-recurrence-adapter` |
| `oya-connect-tasks-recurrence-worker` | `oya-tasks-recurrence-worker` |
| `oya-connect-tasks-recurrence-app` | `oya-tasks-recurrence-app` |
| `oya-connect-tasks-search-kernel` | `oya-tasks-search-index-kernel` |
| `oya-connect-tasks-search-domain` | `oya-tasks-search-index-domain` |
| `oya-connect-tasks-search-usecase` | `oya-tasks-search-index-usecase` |
| `oya-connect-tasks-search-api` | `oya-tasks-search-index-api` |
| `oya-connect-tasks-search-adapter` | `oya-tasks-search-index-adapter` |
| `oya-connect-tasks-search-adapter-meilisearch` | `oya-tasks-search-index-adapter-meilisearch` |
| `oya-connect-tasks-search-worker` | `oya-tasks-search-index-worker` |
| `oya-connect-tasks-search-app` | `oya-tasks-search-index-app` |
| `oya-connect-tasks-importers-kernel` | `oya-tasks-importers-kernel` |
| `oya-connect-tasks-importers-domain` | `oya-tasks-importers-domain` |
| `oya-connect-tasks-importers-usecase` | `oya-tasks-importers-usecase` |
| `oya-connect-tasks-importers-api` | `oya-tasks-importers-api` |
| `oya-connect-tasks-importers-adapter` | `oya-tasks-importers-adapter` |
| `oya-connect-tasks-importers-adapter-csv` | `oya-tasks-importers-adapter-csv` |
| `oya-connect-tasks-importers-adapter-jira` | `oya-tasks-importers-adapter-jira` |
| `oya-connect-tasks-importers-adapter-asana` | `oya-tasks-importers-adapter-asana` |
| `oya-connect-tasks-importers-adapter-trello` | `oya-tasks-importers-adapter-trello` |
| `oya-connect-tasks-importers-adapter-linear` | `oya-tasks-importers-adapter-linear` |
| `oya-connect-tasks-importers-adapter-todoist` | `oya-tasks-importers-adapter-todoist` |
| `oya-connect-tasks-importers-rest` | `oya-tasks-importers-rest` |
| `oya-connect-tasks-importers-worker` | `oya-tasks-importers-worker` |
| `oya-connect-tasks-importers-app` | `oya-tasks-importers-app` |

> **`oya-connect-tasks-domain` split.** The legacy bundled crate (28.5KB
> at 2026-05-17) bundles task + project + view + dependency +
> recurrence + search + importer concerns into a single domain-layer
> crate. Per ADR-0131 + ADR-0105 (13-layer enum), the new layout splits
> the domain layer per bounded context. Migration imports from the
> legacy bundled `oya-connect-tasks-domain` must each pick the specific
> replacement BC; a one-line wholesale `use oya_connect_tasks_domain::*`
> import is not supported.

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in
`oya-connect-tasks-*`. They are therefore not part of the migration
surface — they are clean replacement-boundary features:

- **`oya-tasks-search-index-adapter-meilisearch`** — full Meilisearch
  cross-project search; the legacy surface had no full-text search
  ingestion path.
- **`oya-tasks-importers-adapter-{jira,asana,trello,linear,todoist}`** —
  importers per ADR-TASKS-0001; legacy had CSV only.
- **`oya-tasks-recurrence-worker`** — automated recurring-task
  materialisation; the legacy surface materialised lazily at read time.
- **`oya-tasks-view-engine-adapter-redis`** — presence + view-cursor
  CRDT-light state per ADR-TASKS-0004; the legacy surface had no
  realtime presence.
- **Dependency-graph cycle prevention at write time** — PRD FR-04 +
  AC-02 differentiator; legacy `oya-connect-tasks-*` had only lazy
  cycle detection at read time.
- **Workflow-engine bidirectional bridge** — per ADR-TASKS-0005;
  legacy had only unidirectional task → workflow trigger.
- **EU AI Act Annex III §4 Cedar refusal for auto-assign in employment
  context** — net-new boundary; legacy had no AI auto-assign at all.
- **Bidirectional create-task-from-{email,message,calendar-event}** —
  cross-µservice via Workflow per PRD §"Workflow events consumed";
  legacy had no inbound bridges.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_tasks_domain::{Task, TaskStatus, TaskPriority, DependencyEdge};
use oya_connect_tasks_domain::project::Project;
use oya_connect_tasks_domain::view::BoardColumn;
use oya_connect_tasks_domain::dependency::CycleDecision;

// AFTER
use oya_tasks_task_store_kernel::{Task, TaskStatus, TaskPriority};
use oya_tasks_dependency_graph_kernel::{DependencyEdge, CycleDecision};
use oya_tasks_project_list_kernel::Project;
use oya_tasks_view_engine_kernel::BoardColumn;
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-connect-tasks-domain = { workspace = true }

# AFTER
[dependencies]
oya-tasks-task-store-kernel       = { workspace = true }
oya-tasks-dependency-graph-kernel = { workspace = true }
oya-tasks-project-list-kernel     = { workspace = true }
oya-tasks-view-engine-kernel      = { workspace = true }
# Add other BC dependencies as your consumer touches them.
```

## Reason

The legacy `oya-connect-tasks-*` family was authored before the
following ADRs crystallised:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 — per-µservice SLO authority.** Tasks needs independent
   SLO targets per surface (task-list-render latency, task-create
   latency, task-update latency, search latency, bulk-update latency,
   recurring-materialise latency, webhook-fire latency, dependency-
   cycle-detection correctness 100%, auto-assign-fairness correctness).
   A `connect-*` umbrella SLO cannot honour those.
3. **ADR-0131 — per-µservice flat layout.** Tasks's IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget,
   incident-response, failure-modes, multi-region all need to live
   under one folder (`microservices/tasks/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA +
   근로기준법 retention) through pack-ksa each lives as
   `microservices/tasks/iac/kustomize/overlays/pack-<region>/`. They
   cannot share a folder root with mail / messenger / community.
5. **ADR-TASKS-0001 → ADR-TASKS-0006** — tasks-specific decisions
   (data model, dependency graph cycle prevention, rrule alignment,
   view engine + CRDT scope, workflow bridge, AI auto-assign EU AI Act
   bounds) need to live at per-µservice ADR granularity, not at the
   Connect suite level.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-connect-tasks-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
# Use this command per file as a guided rewrite (review every hit;
# manual disambiguation needed for the `oya-connect-tasks-domain`
# split case):
rg -l "oya_connect_tasks_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
# Inside your consumer crate:
cargo nextest run --features connect-tasks-strangler-canary
```

Run with the feature flag enabled to route through the new µservice;
run without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p99 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- log-line format (preserved verbatim during canary).
- state-transition ordering for `TaskStateChanged` events (per Hyrum's
  Law surface #1 below).
- dependency-graph cycle refusal shape (per Hyrum's Law surface #3).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports
AND the tasks µservice's Phase 3 canary reaches 100% traffic (per
ADR-0134), remove the legacy dependency from your `Cargo.toml`:

```toml
# Remove this line:
oya-connect-tasks-domain = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
# Per ADR-0134 Phase 4 verification:
cargo tree -e normal -p your-crate | grep oya-connect-tasks   # expect empty
rg "use oya_connect_tasks_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.tasks.*` | `tasks.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/tasks/slos/*.openslo.yaml` (per-µservice, 9 files) |
| Helm chart values key | `.Values.connect.tasks.*` | `.Values.tasks.*` |
| K8s namespace | `connect` | `tasks` |
| Cedar policy fragment path | `policy/connect/tasks/*.cedar` | `microservices/tasks/policy/*.cedar` |
| pack-kr overlay path | `policy/connect/tasks/pack-kr/*` | `microservices/tasks/iac/kustomize/overlays/pack-kr/*` + per-pack section in `threat-model.md` / `dpia.md` / `compliance.md` / `multi-region.md` |
| Workflow event prefix | `connect.tasks.*` | `tasks.*` (e.g., `tasks.task.lifecycle.v1`, `tasks.task.state.v1`, `tasks.task.dependency.v1`) |
| Ontology type prefix | `Connect.Tasks.*` | `Tasks.*` (e.g., `Tasks.Task`, `Tasks.Project`, `Tasks.DependencyEdge`, `Tasks.Sprint`, `Tasks.Milestone`, `Tasks.LegalHold`) |
| Telemetry metric prefix | `oya_connect_tasks_*` | `oya_tasks_*` |
| Tracing span attribute namespace | `connect.tasks.*` | `tasks.*` |
| RRULE engine choice | (legacy in-house) | `rrule-rs` 0.13.x per ADR-TASKS-0003 (aligned with calendar ADR-CAL-0002) |
| Search backend choice | (legacy Postgres trigram only) | Meilisearch 0.10.0 LTS per ADR-TASKS-0001 |
| CRDT runtime choice | (none in legacy) | Loro 1.x per ADR-TASKS-0004 (for collaborative-description editing only; not for board state) |

## Dual-context isolation invariant (preserved + strengthened)

The Personal ↔ Professional context isolation invariant from the
Bominal ADR-0231-0233 dual-context inheritance is preserved verbatim
in `oya-tasks-task-store-kernel`. Specifically:

- The `TaskContextBoundaryGuard` port trait keeps the same method
  signatures.
- Cross-context attempts (Professional → Personal task read) emit
  the same 403 + same audit-chain event variant
  (`TasksCrossContextRefused`).
- The kernel-layer refusal (not adapter-layer) invariant is preserved.
- **Strengthened**: cross-context attempts are also refused at the
  Cedar policy layer per `policy/event-isolation.md` (the calendar
  analog) → `policy/task-isolation.md`; the kernel refusal is the
  defence-in-depth backup.

This means downstream consumers that wrap the boundary guard via the
legacy import path will see identical refusal behaviour after
migration; no test rewrite needed for the isolation surface.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes
Removal Hard", these are the legacy tasks surfaces with observable
behaviour that may be depended on. Each is preserved verbatim during
the canary; consumers must re-test after Phase 5 removal in case they
had a long-tail dependency:

1. **Task state-transition ordering.** The legacy domain emitted
   `TaskStateChanged` events in `(old_state → new_state)` field order
   ASCII-ascending; the new µservice preserves that order
   deterministically. Consumers that pattern-match on event-payload
   serialisation order see no change. BUT if consumer assumes
   `(new_state → old_state)` order (a known bug in one downstream),
   they will see a swap.

2. **Custom-field type coercion.** Legacy coerced "1" (string) →
   1 (number) for a number-typed custom-field on write; new
   µservice REFUSES the coerce-from-string write with a
   `CustomFieldTypeMismatch::Refused` 422 per ADR-TASKS-0001
   "strict coercion" policy. Migration: pre-cast in the writer.
   Documented as a deliberate strengthening per
   `feedback_no_silent_regression`.

3. **Dependency-graph cycle-detection edge-cases.** Legacy
   tolerated cycles at write-time and lazy-detected them at read
   time; new µservice REFUSES cycle-creating writes per PRD AC-02
   + ADR-TASKS-0002 (correctness SLO = 100%; no error budget).
   Consumers that had latent cycles in their data will receive
   `DependencyCycle::Refused` 422 on the migrating write that
   would close the cycle. This is a deliberate strengthening; the
   legacy behaviour was a data-integrity hole.

4. **Webhook payload field ordering.** Legacy emitted webhook
   payloads with insertion-order field serialisation (Postgres jsonb
   dependent). New µservice emits with explicit field-ordering per
   the OpenAPI 3.2.0 schema; ordering is deterministic and stable
   across versions. Consumers that pattern-match on insertion order
   should re-test.

5. **Notification timing observable.** Legacy fired notifications
   synchronously with the task-write commit; new µservice fires
   asynchronously via worker fanout with p95 ≤ 200ms latency from
   commit. Consumers that observed sync-write-immediate-notification
   may see the notification arrive after the write returns success;
   this is invisible at the protocol level but observable in tests
   that race the read.

6. **Recurring-task materialisation horizon.** Legacy was
   unbounded; new µservice bounds to 5y horizon per ADR-TASKS-0003
   (aligned with calendar ADR-CAL-0002 + PRD-calendar AC-10).
   Consumers that submitted long-horizon recurring tasks to legacy
   and expected unbounded expansion will receive
   `MalformedRecurrence::BoundExceeded` on the new path. This is a
   deliberate strengthening; the legacy behaviour was a DoS surface.

7. **Importer assignee resolution.** Legacy importers mapped
   imported assignee strings by best-effort fuzzy match (Jaro-
   Winkler 0.85 threshold) against the tenant directory; new
   µservice REFUSES ambiguous assignee strings with an
   `ImportAssigneeAmbiguous::Refused` 422 + a per-row report. This
   is a deliberate strengthening; the legacy behaviour silently
   assigned tasks to the wrong user. Documented as a deliberate
   strengthening per `feedback_no_silent_regression`.

## Runbook continuity table

| Legacy runbook (under `policy/connect/tasks/runbooks/`) | New runbook (under `microservices/tasks/runbooks/`) | Status |
|---|---|---|
| `recurrence-materialisation-failure.md` | `recurring-task-materialisation-failure.md` | preserved + expanded with ADR-TASKS-0003 5y bound refusal |
| (no legacy counterpart) | `custom-field-schema-migration.md` | NEW per ADR-TASKS-0001 + Hyrum surface #2 |
| (no legacy counterpart) | `dependency-cycle-corruption.md` | NEW per ADR-TASKS-0002 + Hyrum surface #3 |
| (no legacy counterpart) | `search-index-rebuild.md` | NEW per ADR-TASKS-0001 + degraded-mode |
| (no legacy counterpart) | `bulk-edit-throttle.md` | NEW per PRD §"Bulk edit" + capacity-model |
| (no legacy counterpart) | `webhook-fanout-degraded.md` | NEW per Hyrum surface #4 + #5 |
| (no legacy counterpart) | `ai-assign-classifier-rollback.md` | NEW per ADR-TASKS-0006 + EU AI Act conformity-rollback procedure |

## Phases (per ADR-0134)

| Phase | Description | Status (tasks) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-TASKS passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-connect-tasks-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crate + Cargo.toml entry + spec pointer | pending | `cargo build --workspace` exits 0; no `oya_connect_tasks_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all sibling µservices reaching their own Phase 5 | pending | All HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

Per the deprecation-and-migration skill, every deprecation closeout must
satisfy these checks. Each is gated by a concrete command:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice tasks
  # expect: HG-TASKS accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/tasks/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-connect-tasks-domain --invert | grep -v 'oya-connect-tasks-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_tasks_" --type rust | rg -v "migration-adapter|legacy_in_process|tests/" | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connect-tasks-*" | wc -l   # expect 0
  test ! -f /specs/microservices/tasks.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase**
  (excluding historical ADR / RETIRED.md / git-log surfaces):
  ```bash
  rg "oya_connect_tasks" --type rust | rg -v "docs/decisions/|RETIRED.md|tests/golden/" | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (per Phase 5):
  ```bash
  test ! -f microservices/tasks/deprecation-notice.md          # expect file absent
  test ! -f microservices/tasks/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4 for the
core symbol surface for backward-compatible operations: the adapter
preserves the legacy symbol surface verbatim, including error variant
ordering for non-cycle, non-coerce-from-string cases, within the +5%
canary tolerance.

**There ARE four behavioural strengthenings** that may visibly differ
from legacy and are NOT preserved by the adapter (per
`feedback_no_silent_regression`):

1. **Dependency-graph cycle prevention at write time** (per
   ADR-TASKS-0002 + Hyrum #3). Consumers with latent cycles in
   their data will receive `DependencyCycle::Refused` 422 on the
   migrating write that would close the cycle. This is a deliberate
   strengthening; the legacy behaviour was a data-integrity hole.

2. **Custom-field strict type coercion** (per ADR-TASKS-0001 +
   Hyrum #2). Consumers writing `"1"` to a number-typed field will
   receive `CustomFieldTypeMismatch::Refused`. Strengthening: legacy
   was a silent-corruption surface.

3. **Recurring-task horizon bounded at 5y** (per ADR-TASKS-0003 +
   PRD aligned with calendar AC-10 + Hyrum #6). Consumers that
   submitted unbounded recurring tasks to legacy will receive
   `MalformedRecurrence::BoundExceeded`. Strengthening: legacy was
   a DoS surface.

4. **Importer assignee strict resolution** (per ADR-TASKS-0001 +
   Hyrum #7). Consumers relying on legacy's silent fuzzy-match
   assign-to-wrong-user behaviour will receive
   `ImportAssigneeAmbiguous::Refused` per row. Strengthening:
   legacy was a silent-misassign surface.

Phase 5 (code removal) **IS a breaking change** for any consumer that
did not migrate during the 5-month adapter+canary window. Per
`feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's
  `deprecation_date` (2026-05-17), so a target advisory removal date
  of **2026-11-17** (subject to the HG-TASKS retirement trigger
  gating).
- Owning axis (axis-tasks) ships migration ChangeSets for every
  internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/tasks.json`)
  receive a 6-month sunset window from this notice; the spec file's
  `deprecated: true` + `replacement_path:
  /specs/microservices/tasks/tasks.json` fields render in the
  agent-coordination dashboard.

## References

- ADR-0135: Connect super-app expansion into flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: Connect dissolution Strangler migration (operational policy).
- ADR-TASKS-0001: Task data model + custom fields (strict coercion).
- ADR-TASKS-0002: Dependency graph + cycle prevention at write time.
- ADR-TASKS-0003: Recurring task engine (rrule-rs alignment with calendar ADR-CAL-0002).
- ADR-TASKS-0004: View engine + board realtime (CRDT scope).
- ADR-TASKS-0005: Automation engine cross-µservice (workflow-engine bridge).
- ADR-TASKS-0006: AI auto-assign + EU AI Act Annex III §4 bounds.
- ADR-CAL-0002 — RRULE engine choice (sibling reference; align here).
- `microservices/tasks/PRD.md` — full target-state product definition.
- `microservices/tasks/PHASE-01-TASKS-FOUNDATION.md` — phase plan.
- `microservices/tasks/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
- RFC 5545 — iCalendar VTODO + RRULE subset (aligned).
- draft-ietf-jmap-tasks — JMAP Tasks (scheduled-for-distinct-tracked-work to M05).
- `microservices/calendar/migration-from-connect.md` — sibling reference template.
