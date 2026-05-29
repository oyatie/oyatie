---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-004-event-store-adapter-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, sql-migration-test, oya-governance-data-residency]
---

# IP-004: Event-store Postgres adapter

## A. Problem
The event-store ports need a durable adapter that preserves tenant RLS, legal-hold rows, retention markers, and encrypted personal/professional event content without turning database details into domain rules.

## B. Approach
Implement `oya-calendar-event-store-adapter-postgres` as the only Postgres-backed event repository for the foundation path. Use RLS keyed by `tenant_id`, OpenBao tenant-DEK references, idempotency-key uniqueness, and transaction boundaries that protect event mutation plus audit outbox enqueue.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-event-store-adapter-postgres.yaml` | Existing catalog anchor. |
| `src/crates/oya-calendar-event-store-adapter-postgres/` | Planned adapter path named by manifest/catalog. |
| Postgres migrations under the adapter crate | Event, attendee, hold, retention, outbox, and tzdb-version tables. |
| `policy/tenant-scope.cedar` and `policy/data-residency.md` | Policy contract the adapter must not bypass. |

## D. Ordered implementation steps
1. Define migrations with tenant, home-cell, context, data-class, and audit-correlation columns.
2. Add RLS policies for tenant and context isolation.
3. Implement `EventRepository` and `LegalHoldStore` against transactions.
4. Store encrypted payloads through SecretReference/DEK handles rather than plaintext keys.
5. Add migration tests for RLS denial, idempotency conflict, and legal-hold preservation.
6. Add adapter tests using synthetic tenants and explicit role switching.
7. Verify the adapter crate has no REST, workflow, or product-microservice imports.

## E. Acceptance
- `cargo nextest run -p oya-calendar-event-store-adapter-postgres` passes.
- SQL migration tests prove cross-tenant reads and writes are denied.
- `cargo run -p oya-dev-cli -- gate validate data-residency --microservice calendar` passes.
- `cargo run -p oya-dev-cli -- gate validate statelessness --microservice calendar` passes.
- Restore behavior remains compatible with `runbooks/calendar-restore.md`.

## F. Evidence
- PRD storage and legal-hold requirements: `microservices/calendar/PRD.md`.
- Catalog: `microservices/calendar/catalog/oya-calendar-event-store-adapter-postgres.yaml`.
- Policy files: `policy/event-isolation.md`, `policy/tenant-scope.cedar`, `policy/data-residency.md`.
- Operational files: `multi-region.md`, `backfill-replay.md`, `runbooks/calendar-restore.md`.

## G. Counterpart comparison
Outlook and Google provide enterprise retention through suite controls, but not a tenant-visible adapter contract. This IP turns the comparison into concrete Postgres RLS, legal-hold, and data-residency tests, which is the minimum needed before claiming better auditability than Google Vault or Microsoft eDiscovery.

## H. Foundation delivery expansion
- Deliverable detail: migrations create events, attendees, holds, retention markers, idempotency keys, and outbox rows.
- Deliverable detail: each table carries tenant, context, home-cell, data-class, and audit-correlation columns.
- Deliverable detail: RLS policies deny cross-tenant reads and writes before repository code runs.
- Deliverable detail: encrypted payload columns store DEK references rather than inline keys.
- Deliverable detail: transaction code persists event mutation and outbox enqueue atomically.
- Deliverable detail: restore metadata includes schema version and tzdb version.
- Deliverable detail: adapter errors preserve safe diagnostics for operators without leaking event content.
- Deliverable detail: Slack Enterprise Grid calendar interop is comparison pressure for tenant-segmented storage.

## I. Acceptance expansion
- Acceptance detail: migration tests must create two tenants and prove cross-tenant SELECT/UPDATE denial.
- Acceptance detail: idempotency tests must prove duplicate keys do not duplicate outbox rows.
- Acceptance detail: legal-hold tests must prove delete/update refusal survives transaction rollback paths.
- Acceptance detail: residency tests must prove home-cell and pack columns are populated on every write.
- Acceptance detail: statelessness checks must prove no process-local event cache is required for correctness.
- Acceptance detail: restore tests must replay outbox rows without violating RLS.
- Acceptance detail: adapter crate checks must reject REST, workflow, and unrelated product imports.
- Acceptance detail: Slack/Google/Outlook comparisons must be limited to enterprise isolation and auditability.

## J. Evidence expansion
- Evidence detail: capture nextest output for Postgres adapter tests.
- Evidence detail: capture SQL migration verification output with RLS fixture names.
- Evidence detail: capture data-residency gate output for calendar.
- Evidence detail: cite `multi-region.md` for home-cell expectations.
- Evidence detail: cite `backfill-replay.md` for outbox replay behavior.
- Evidence detail: cite `runbooks/calendar-restore.md` for restore operator steps.
- Evidence detail: cite Slack as collaboration-suite pressure while preserving Postgres as the first-party adapter boundary.
