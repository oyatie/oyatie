# G005 SLICE-2 — Identity SCIM durable store wiring (composition-root) — architect-scoped 2026-06-22 (dev 9d2b2d0ab)

Advances G005 (IdP). Faithful mirror of G006 SLICE-1 (#798): wire the durable Postgres SCIM stores into the identity SCIM serving composition root, fail-closed, behind the UNCHANGED `UserStore`/`GroupStore` kernel ports (cutover litmus: oya-data/G003 swaps the adapter later). ADR-0510 transitional Postgres. **AUTH-005 is NOT a blocker** — the SCIM routes are already fail-closed (Bearer + `scim.manage` scope + Cedar `identity.scim.Manage` PEP + cross-tenant 403 before the store).

## What EXISTS on origin/dev (consume; verified)
- Composition root: `iam/facade/identity-service/src/server.rs::start()` (async; the ONE boot path for main.rs + e2e) builds `ScimSurfaceState::new(...)` + merges `build_scim_router(...)` at ~server.rs:215-222. `StartError` enum ~server.rs:33-37 (add `Store(String)`).
- `iam/facade/identity-service/src/users/mod.rs:62-106` — `ScimSurfaceState<R,D,A,S>` (generic over the 4 authz ports) but the SCIM `server` field is HARDCODED `ReferenceScimServer<InMemoryUserStore, InMemoryGroupStore, CounterIdGen>` (line 69, ctor 94-99). Handlers (11) + `build_scim_router` are generic over `<R,D,A,S>`; `build_scim_router` returns erased `axum::Router`. The fail-closed guard is users/mod.rs:156-242,286-292 (cross-tenant 403 at 196-208).
- `libs/oya-shared-scim-server-kernel/src/lib.rs` — `UserStore` (lib.rs:455, `Send+Sync`), `GroupStore` (lib.rs:482), `ReferenceScimServer<U,G,I>` already fully generic (lib.rs:681-690).
- `iam/adapters/identity-scim-store-postgres/src/lib.rs` — `PgScimUserStore` (impl UserStore, lib.rs:135/151) + `PgScimGroupStore` (impl GroupStore, lib.rs:351/362), both `Clone+Debug`, both `from_pool(pool)`. `connect_pool(url)->Result<PgPool,PgScimConnectError>` (lib.rs:96). NO `connect()`-returns-store, NO RLS guard yet. migrations/0001_identity_scim_store.sql: FORCE RLS, policies `TO identity_scim_runtime`. NO 0000_runtime_role.sql. tests/live_rls.rs has store-layer RLS cross-tenant + BYPASSRLS-absent proofs (reference, do NOT duplicate).
- MIRROR pattern (now on dev): tenancy adapter `assert_rls_enforceable`/`evaluate_rls_enforceability`/`RUNTIME_ROLE` (tenancy/adapters/tenant-lifecycle-store-postgres/src/lib.rs:61,235-330) + tenancy facade `build_postgres_router`/`select_store_kind`/fail-closed `serve()` (tenancy/facade/tenant-lifecycle-app/src/lib.rs:879-906,977-1056) + tenancy tests (acceptance.rs:610-720).

## The change (composition-root + adapter RLS guard; NO new tracked file)
1. **Adapter RLS guard** (`identity-scim-store-postgres/src/lib.rs`, mirror tenancy verbatim w/ name swaps):
   - `pub const RUNTIME_ROLE: &str = "identity_scim_runtime";` (MUST match migration 0001 `TO <role>`; add a cross-ref comment in 0001).
   - 2 new `PgScimConnectError` variants: `RlsUnenforceable{role}` + `RlsRoleMismatch{role,expected}` (+ Display).
   - `pub async fn assert_rls_enforceable(pool: &PgPool) -> Result<(), PgScimConnectError>` (FREE fn taking `&PgPool` to avoid duplicating across both stores): `SELECT current_user, session_user, rolsuper, rolbypassrls, pg_has_role(current_user,$1,'MEMBER') FROM pg_roles WHERE rolname=current_user` (bind RUNTIME_ROLE); session_user≠current_user → fail-closed; then pure `evaluate_rls_enforceability(role,rolsuper,rolbypassrls,is_member)` (super/bypass→RlsUnenforceable; !member→RlsRoleMismatch; else Ok).
   - DB-free unit tests for `evaluate_rls_enforceability` (all combos) + empty-url (mirror tenancy adapter tests).
2. **Generalize `ScimSurfaceState<R,D,A,S>` → `<R,D,A,S,U,G>`** (`U:UserStore,G:GroupStore`): field becomes `ReferenceScimServer<U,G,CounterIdGen>`; ctor takes `users:U, groups:G` (+ id-gen). Thread `<U,G>` mechanically through the 11 handlers + `build_scim_router`. Blast radius = 2 files (server.rs + users/mod.rs). Update in-memory call sites (server.rs:215, users/mod.rs:696 test) to pass `InMemoryUserStore::default()/InMemoryGroupStore::default()`.
3. **Fail-closed store selection in `start()`**: extract pure `select_scim_store_kind(Option<String>)->ScimStoreSelection` (mirror tenancy `select_store_kind`); read `OYA_BACKBONE_POSTGRES_URL`; non-empty → `connect_pool(url)` → `assert_rls_enforceable(&pool)` → `PgScimUserStore::from_pool(pool.clone())`+`PgScimGroupStore::from_pool(pool)` → durable `ScimSurfaceState`; absent/empty → in-memory dev. **NEVER fall back to in-memory when a URL is configured** (propagate `StartError::Store`). Differentiate startup log (`store=postgres`/`inmemory`).

## Tests (RED→GREEN; ride EXISTING files only)
- Adapter: DB-free predicate + empty-url unit tests in `identity-scim-store-postgres/src/lib.rs` tests mod.
- Facade DB-free: `select_scim_store_kind` unit tests (Some/None/empty/whitespace) in `server.rs` test mod; empty-url fail-closed proof.
- Facade live (env-gated `OYA_BACKBONE_LIVE_POSTGRES`, mirror tenancy acceptance.rs:691, ride `iam/facade/identity-service/tests/e2e_service.rs`): boot service against `OYA_BACKBONE_POSTGRES_URL`, `POST /scim/v2/ten_acme/Users`, rebuild fresh service over SAME url, `GET` user back → durable across rebuild. Cross-tenant = REAL facade PEP **403** (verified `ten_acme` token → `/scim/v2/ten_other/Users` → 403 before store). Store-layer RLS already in adapter live_rls.rs — reference, do NOT duplicate. Default `buck2 test` stays DB-free (skip clean).
- Existing `scim_surface_guards_and_provisions_on_the_live_socket` (e2e_service.rs:275) stays green (in-memory branch, no url).

## Deps / born-accounting / buck2 (part of done)
- `iam/facade/identity-service/Cargo.toml`: add `identity-scim-store-postgres = { path = "../../adapters/identity-scim-store-postgres" }` + `sqlx.workspace = true` (composition root holds the `PgPool` + the guard takes `&PgPool`). NOTE crate name is `identity-scim-store-postgres` (NOT iam-prefixed).
- `iam/facade/identity-service/BUCK`: add `//iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres` + `third-party//:sqlx` to the rust_library AND the test targets (unittest + e2e as needed); add adapter to the binary target if buck requires it (mirror tenancy binary). Keep cargo↔buck parity.
- Regen Cargo.lock (`cargo metadata --offline`); expect graph-edge change (sqlx already present via adapter, likely no new external crates).
- NO new tracked file (all edits to existing Cargo.toml/BUCK/src/*.rs/tests/*.rs) → firewall GO-LIVE should not regress. Still: materialize faces + run firewall GO-LIVE + freshness (ADR-0539) + affected-set (ADR-0554) + face-settle --verify locally BEFORE push (buck2-build-green≠CI-green).

## Clean-arch / honesty
- `UserStore`/`GroupStore` ports + `ReferenceScimServer` UNCHANGED (cutover litmus holds). Only `ScimSurfaceState` (a composition wrapper) gains generic params.
- HONEST scope: NO `0000_runtime_role.sql` for the SCIM adapter (same deferred gap as tenancy) — the guard REFUSES a bad role but `identity_scim_runtime` must still EXIST provisioned NOBYPASSRLS. PR states: "durable wiring + fail-closed RLS-enforceability guard; role provisioning deferred (mirror oya-data-outbox / tenancy)." Do NOT overclaim "durable + isolated".

## Process discipline
- Fresh worktree off origin/dev (9d2b2d0ab); NEVER touch the canonical checkout. buck2 for build/test (cargo hook-blocked; cargo metadata allowed for lock). Commit trailer EXACTLY `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. After build+gates green: STOP, do NOT self-approve — orchestrator runs adversarial review before merge.
