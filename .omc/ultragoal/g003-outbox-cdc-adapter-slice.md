# G003 slice — oya-data-outbox-adapter-sqlx (CDC change-stream polling adapter) — architect-scoped 2026-06-22

Advances G003 (persistence substrate oya-data, pending; gates G005 IdP durable + G006 tenancy per canonical_build_sequence Phase-0→Phase-1). The SQL write side is done (libs/oya-data-sql-adapter-sqlx); the open seam is the CDC read/relay side: oya-data-outbox-kernel::ChangeStreamSource has ONLY a RecordingChangeStream reference impl, NO real adapter. ADR-0510 transitional (outbox-polling behind the W5 changefeed-shaped trait); ADR-0536 D-10/D-13. NO founder gate (additive adapter behind an accepted port).

## What EXISTS (consume, don't rebuild)
- libs/oya-data-outbox-kernel/src/lib.rs — ChangeStreamSource trait (the CDC port), ChangeRecord, ChangeBatch (+ ChangeBatch::validate enforces ordering + checkpoint monotonicity), OutboxEvent, INSERT_OUTBOX_EVENT_SQL (targets oya_data_outbox.outbox_events). Only RecordingChangeStream ref impl exists.
- libs/oya-data-sql-adapter-sqlx/src/lib.rs — the DONE transitional SQL adapter (RLS apply_session_scope, HLC stamp_commit, env-gated live cross-tenant-deny harness run_live_rls_cross_tenant_probe). MIRROR its async-surface-over-sync-kernel-trait split + its BUCK.
- libs/oya-shared-transactional-outbox-adapter-sqlx/src/lib.rs — EXACT precedent drain adapter: claim_pending_sql (~:307-314 FOR UPDATE SKIP LOCKED), mark_published/mark_dead_letter, set_tenant_scope_for_rls, over PgPool; BUCK:3 globs migrations/**/*.sql.
- libs/oya-data-sql-kernel/src/clock.rs — HlcTimestamp.
- libs/oya-shared-postgres-command-kernel/src/lib.rs:9 — SET_LOCAL_TENANT_SQL (set_config oyatie.tenant_id, RLS).

## The change (new crate libs/oya-data-outbox-adapter-sqlx)
- `SqlxChangeStreamSource { pool: PgPool }` implementing the polling CDC adapter; async surface mirroring `ChangeStreamSource::poll_changes(tenant_id, checkpoint: HlcTimestamp, limit) -> ChangeBatch` (mirror the sql-adapter's async-over-sync-kernel split — sync kernel traits stay IO-free reference impls).
- Parameterized poll query (NO value interpolation) against oya_data_outbox.outbox_events: rows strictly-after the checkpoint, ORDER BY HLC commit timestamp, LIMIT $n → ChangeBatch{records, resume_from}. Apply tenant RLS scope FIRST (SET_LOCAL_TENANT_SQL) in the SAME tx (mirror apply_session_scope).
- Map rows → ChangeRecord; build+validate via ChangeBatch::validate (kernel-enforced ordering/monotonicity).
- Production DDL migration migrations/0001_outbox_events.sql for oya_data_outbox.outbox_events (the table INSERT_OUTBOX_EVENT_SQL targets — currently NO migration anywhere; carry an HLC commit-timestamp column for order/checkpoint; RLS policies per the tenant-isolation pattern: PERMISSIVE tenant policy + RESTRICTIVE deny-on-empty-GUC, FORCE RLS — mirror the auth sqlx adapter's migration). BUCK srcs glob migrations/**/*.sql.

## Clean-arch
Port (ChangeStreamSource) = the owned W5 engine-native changefeed w/ HLC checkpoints — UNCHANGED at cutover (litmus: only this adapter is replaced by the engine changefeed; consumers never observe the impl). Adapter = transient ADR-0510 Postgres polling, owns all SQL/poll/RLS impedance. Deps: the two kernels + sqlx + tokio (dev). NO peer-adapter coupling.

## Tests (RED/GREEN)
- Unit (DB-free, default): poll SQL fully parameterized; checkpoint strictly-after; limit honored; produced ChangeBatch passes validate; tenant filter present. (mirror sql-adapter pure-unit style.)
- Integration (env-gated OYA_*_LIVE_POSTGRES, mirror run_live_rls_cross_tenant_probe): two tenants' outbox rows on containerized PG → tenant-scoped poll sees ONLY its own (RLS), commit-ordered, resume_from advances, resume-from-checkpoint at-least-once. Default buck2 test stays DB-free.

## Doctrine / done-bar
- aws-lc-rs TLS if the pool uses TLS (ring FORBIDDEN, ADR-0506) — reuse the sql-adapter's TLS setup.
- Tier-3: no unwrap/expect/panic in prod; #![forbid(unsafe_code)].
- buck2: rust_library + rust_test mirroring oya-data-sql-adapter-sqlx/BUCK (deps: two kernels + third-party//:sqlx + tokio). Auto-swept workspace member via libs/oya-* glob (ADR-0538) — no root Cargo.toml edit.
- BORN-ACCOUNTING (the #789/#793 lesson — do it FULLY or firewall GO-LIVE goes RED): OWNERS (ancestor or new), ADR justification governed-surfaces VERBATIM paths for EVERY new tracked path incl. Cargo.toml/BUCK/src/lib.rs/migrations/0001_outbox_events.sql, capability mapping (check how libs/oya-data-* crates map — membership-lint), reachability for the .sql (non-crate file — the #793 Helm lesson: needs explicit reachability if not covered by a prefix), catalog record iff required for libs adapters (check peers). Then regen Cargo.lock offline (cargo metadata --offline), materialize faces, run firewall GO-LIVE locally (firewall_is_green_on_the_live_corpus_with_the_baseline) → regressions=0 BEFORE push, face-settle --verify byte-identical.
- Use an existing ADR for justification (the oya-data / ADR-0536 / ADR-0510 cluster — find the owning persistence ADR; do NOT mint a new one unless required).
