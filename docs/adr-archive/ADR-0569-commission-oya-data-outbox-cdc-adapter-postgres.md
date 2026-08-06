---
id: ADR-0569
title: "Commission the oya-data outbox CDC change-stream Postgres adapter (oya-data-outbox-adapter-postgres) behind the ChangeStreamSource port"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-22
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-0704]
amends: []
depends_on: [ADR-0510, ADR-0536, ADR-0537, ADR-0550, ADR-0555, ADR-0562]
related: [ADR-0083, ADR-0105, ADR-0131, ADR-0506, ADR-0553, ADR-0559, ADR-0567]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: oya-data outbox CDC adapter

# ADR-0569: Commission the oya-data outbox CDC change-stream Postgres adapter

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — additive adapter behind
an already-accepted port; removable by deleting the crate without unwinding any SSOT; the producer
remains the sole face generator).**

## Context

Story G003 (the owned `oya-data` persistence substrate; ADR-0536 D-10 change streams / D-13
messaging) has its SQL WRITE side commissioned: `libs/oya-data-sql-adapter-sqlx` is the ADR-0510
transitional Postgres implementation of the SQL port, and `libs/oya-data-outbox-kernel` defines
the transactional-outbox shapes plus the CDC `ChangeStreamSource` port. The open seam is the CDC
READ/relay side: `oya-data-outbox-kernel::ChangeStreamSource` ships ONLY a `RecordingChangeStream`
reference implementation — there is no real adapter that polls a durable outbox table.

The port models the W5 engine's native changefeed with HLC checkpoints (CockroachDB changefeed /
Spanner change-stream shape). The transitional implementation (ADR-0510) is outbox polling behind
the same trait, so consumers checkpoint with HLC timestamps and never observe the engine swap.
The table `oya_data_outbox.outbox_events` that the kernel's `INSERT_OUTBOX_EVENT_SQL` targets has
no production migration anywhere — this ADR commissions both the adapter and that migration.

## Decision

Commission **`libs/oya-data-outbox-adapter-postgres`** — the ADR-0510 transitional Postgres (via
sqlx) realization of `oya-data-outbox-kernel::ChangeStreamSource`. It absorbs ALL engine impedance
behind the unchanged port; only this adapter is replaced by the engine-native changefeed at W5.

### D1 — `SqlxChangeStreamSource { pool: PgPool }`, async-over-sync-kernel split

The adapter's async `poll_changes(tenant_id, checkpoint: HlcTimestamp, limit) -> ChangeBatch`
mirrors the sync kernel trait 1:1 (the sync kernel traits stay reserved for IO-free reference
impls, matching the data SQL kernel/adapter split). Rows map to the kernel `ChangeRecord` shape;
the assembled `ChangeBatch` is validated by the kernel's ordering/monotonicity invariants before
return, so the adapter relies on the kernel invariant rather than re-checking ordering itself.

### D2 — Fully parameterized, strictly-after-checkpoint, stream-position-ordered poll

The poll SELECTs over `oya_data_outbox.outbox_events`: tenant-scoped (`tenant_id = $1`), rows
STRICTLY AFTER the monotone `commit_logical` stream-position checkpoint (`commit_logical > $2`),
ordered SOLELY by `commit_logical` (`ORDER BY commit_logical`), limited (`LIMIT $3`). No tenant id,
checkpoint position, or limit ever enters the SQL text — every dynamic input is a bound `$n`
placeholder. The next checkpoint is the last delivered record's stream position (at-least-once
resumable; re-polling from the same checkpoint may re-deliver — D-13 per-key ordering only).

The checkpoint/resume position is a dedicated opaque monotone `StreamPosition` newtype on the
kernel port (a CDC offset, NOT an HLC), sourced from the global `commit_logical`
GENERATED-ALWAYS-AS-IDENTITY bigint sequence and carried as `u64` WITHOUT narrowing
(security-audit amendment): a single global IDENTITY is already a strict, unique, monotone total
order — so it is sufficient on its own. The earlier `(commit_wall_nanos, commit_logical)` ordering
was corrected for two defects the audit found: (1) `commit_logical` narrowed into an HLC `u32`
logical field would error past ~4.3B rows and wedge the poll forever (a u32 cannot carry the
bigint sequence), and a global sequence is not a valid per-wall HLC tie-counter; (2) the
non-monotone, statement-time `clock_timestamp()` wall as the primary sort/filter key could let an
NTP step-back or long transaction emit a later row with a smaller wall, silently skipping it past
the strictly-after filter (under-delivery, violating outbox at-least-once). `commit_wall_nanos` is
retained as an informational `ChangeRecord` field only — never the order/checkpoint key.

### D3 — RLS tenant isolation (the load-bearing security property; ADR-0567 precedent)

The tenant RLS scope runs FIRST in the same transaction as the poll, reusing
`oya-shared-postgres-command-kernel::SET_LOCAL_TENANT_SQL`
(`SELECT set_config('oyatie.tenant_id', $1, true)`). The migration `0001_outbox_events.sql`
mirrors the auth durable stores (ADR-0567) EXACTLY: ENABLE + FORCE ROW LEVEL SECURITY under TWO
policies — a PERMISSIVE tenant-isolation policy (USING + WITH CHECK on the session GUC; required,
because a RESTRICTIVE-only set is deny-all) AND a RESTRICTIVE policy that hard-denies any access
when the GUC is unset or empty (so a missing per-tx scope can never fall through to an open scan).
Both policies scope `TO oya_data_outbox_runtime` — the production runtime role, NOT `PUBLIC`.

The runtime-role CONTRACT ships IN the migration set (security-audit amendment):
`0000_runtime_role.sql` (ordered first) idempotently creates `oya_data_outbox_runtime`
(`NOLOGIN NOBYPASSRLS`, guarded by a `pg_roles` existence check since `CREATE ROLE` has no
`IF NOT EXISTS`) and grants it `USAGE` on the schema; `0001_outbox_events.sql` grants it
`SELECT, INSERT` on the table (the CDC poll only reads, producers only append; outbox rows are
immutable so no `UPDATE/DELETE`). This closes the audit BLOCKER where the policies targeted a role
that was never provisioned (undefined production RLS posture) while the live test validated a
different target (`PUBLIC`). The env-gated live harness now applies the COMMITTED migrations
verbatim, runs the poll AS `oya_data_outbox_runtime` (via `SET ROLE` on every pooled connection,
the deploy contract being that the login role is a member of the runtime role), and asserts the app
role lacks `rolbypassrls` (ADR-0567 D3) — so the test exercises the EXACT production policy set. The
runtime role carries no BYPASSRLS or RLS would be silently skipped; a `CHECK (tenant_id <> '')` is
defense-in-depth.

### D4 — Doctrine bar

- **Clean-arch face-direction** (ADR-0131): the adapter depends inward on the kernel ports only.
- **aws-lc-rs TLS only** (ADR-0506 crypto-backend-purity gate): zero `ring` activation.
- **Tier-3** (ADR-0083): no `unwrap`/`expect`/`panic` in prod; `#![forbid(unsafe_code)]`;
  fail-closed on a malformed row (a negative `commit_logical` stream position or `commit_wall_nanos`
  is a typed Adapter error, never a silent coercion).
- **buck2 primary build** (founder directive): `rust_library` + `rust_test`, `migrations/**/*.sql`
  in the srcs glob.
- **Naming** (ADR-0105 §Adopted Patterns): the backend qualifier is `postgres` — the external
  SYSTEM — not `sqlx`, which is the in-process driver/toolkit; `oya-data-outbox-adapter-postgres`
  is a recognized backend-qualified adapter whose effective role is `adapter`.

### D5 — Ownership + justification manifest (ADR-0555 D2)

Owner: `libs/oya-data-outbox-adapter-postgres/OWNERS` = `axis-cloud-platform` — the `data`
capability owner (ADR-0562 capability-registry: `libs/oya-data-*` maps to the `data` capability,
the owned data-plane substrate, the same team that owns the iam/ + tenancy/ persistence-substrate
trees). The crate's `.rs` sources are reachable via the `libs/oya-*` cargo-members glob (ADR-0538);
the non-crate `migrations/0001_outbox_events.sql` is reachable via that SAME member-dir prefix (the
cargo-members reachability covers the whole member directory, not only Rust files). No catalog
record is minted: the direct adapter peer `oya-data-sql-adapter-sqlx` carries none (the gate-tool
default). The runtime-role contract migration `0000_runtime_role.sql` is reachable via the SAME
member-dir prefix and justified by this decision (security-audit amendment, see D3). Files
commissioned by this decision:

`libs/oya-data-outbox-adapter-postgres/BUCK`,
`libs/oya-data-outbox-adapter-postgres/Cargo.toml`,
`libs/oya-data-outbox-adapter-postgres/OWNERS`,
`libs/oya-data-outbox-adapter-postgres/migrations/0000_runtime_role.sql`,
`libs/oya-data-outbox-adapter-postgres/migrations/0001_outbox_events.sql`,
`libs/oya-data-outbox-adapter-postgres/src/lib.rs`.

## Precedent

- **ADR-0567 service-commissioning pattern**: sqlx + Postgres + RLS adapter in the ADR-0550
  kernel/adapter shape, born-accounted via a D5 ownership + justification manifest with the
  PERMISSIVE + RESTRICTIVE-deny-on-empty-GUC + FORCE RLS migration mirrored verbatim.
- **ADR-0536 D-10 / D-13**: the changefeed-shaped `ChangeStreamSource` port and the
  transactional-outbox → CDC relay doctrine (CockroachDB changefeed / Spanner change-stream;
  Debezium outbox practice).
- **ADR-0510 transitional-impl-behind-interface**: outbox polling is the transient posture; the
  port is unchanged at the W5 engine cutover.
- **Proven patterns, Rust reimplementation**: Debezium / CDC change-data-capture, reimplemented
  Rust-native and HLC-checkpointed behind the owned port.

## Rejected alternatives

- **A LISTEN/NOTIFY push adapter instead of polling.** Rejected for the transitional rung: polling
  with an HLC checkpoint is the changefeed-shaped, resumable, at-least-once contract the port
  models; LISTEN/NOTIFY is lossy on disconnect and does not carry a resumable checkpoint.
- **Naming the crate `oya-data-outbox-adapter-sqlx` to mirror the three existing `-sqlx` siblings.**
  Rejected: those `-adapter-sqlx` crates are grandfathered naming debt (frozen in the BNF baseline),
  not a blessed pattern; per ADR-0105 the backend qualifier is the external system (`postgres`),
  and `sqlx` is the driver. The `-postgres` name is born-clean against the naming gate.

## Consequences

- The CDC read/relay seam of G003 is closed behind the unchanged `ChangeStreamSource` port; a
  consumer relay can drain `oya_data_outbox.outbox_events` with at-least-once HLC-checkpointed
  resumption today and migrate to the engine-native changefeed at W5 with no consumer change.
- The production `oya_data_outbox.outbox_events` migration exists for the first time, with RLS
  tenant isolation correct (PERMISSIVE grant + RESTRICTIVE empty-GUC-deny + FORCE RLS).
- Born-accounted at creation (ADR-0555): owned, justified (this ADR), reachable, target-paritied;
  no new BNF naming debt; aws-lc-rs-only TLS.

## References

- ADR-0510 (transitional impl behind interface), ADR-0536 (substrate decision matrix, D-10/D-13),
  ADR-0537 (dogfood bootstrap order), ADR-0550 (repository layout / kernel-adapter shape),
  ADR-0555 (unaccounted artifacts unmergeable — structural accounting), ADR-0562 (capability-first
  organization + closed capability registry), ADR-0567 (auth durable Postgres+RLS stores — the
  mirrored migration + manifest precedent), ADR-0105 (layer-enum + backend-qualified adapter
  naming), ADR-0506 (aws-lc-rs crypto-backend purity), ADR-0083 (Tier-3 no-panic), ADR-0538
  (globbed workspace membership + cargo-members reachability).
