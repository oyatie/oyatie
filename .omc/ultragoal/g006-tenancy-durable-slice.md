# G006 SLICE-1 — Tenancy lifecycle durable store wiring (composition-root) — architect-scoped 2026-06-22

Advances G006 (tenancy substrate) + FD-001 packaging_order #1. Smallest blast radius, ZERO founder-gated surface.
Composition-root-only swap: the durable adapter (`PgTenantLifecycleStore`) is already merged + complete; the open seam
is that the facade composition root only ever constructs the in-memory store. This wires the durable store behind
`DATABASE_URL`, fail-closed. Port `TenantLifecycleStore` is UNCHANGED (cutover litmus holds: oya-data/G003 swaps the
adapter later with zero facade/port change). ADR-0510 transitional Postgres behind the owned-shaped port.

## What EXISTS on origin/dev (consume, do NOT rebuild) — verified dde8f566f
- `tenancy/facade/tenant-lifecycle-app/src/lib.rs`
  - `build_router<S>(provider: TenantLifecycleProvider<S>, authorizer: SharedAuthorizer, platform_admin_token, tenant_operator_token) -> Router` — generic over the store; mounts the REST routes (~lib.rs:795+).
  - `build_inmemory_router(platform_admin_token, tenant_operator_token) -> Result<Router, BootError>` (~lib.rs:832) — composes `TenantLifecycleProvider::new(InMemoryTenantLifecycleStore::new())` + `PdpTenantLifecycleAuthorizer::from_seed_bundle()`.
  - `enum BootError { Bind, Serve, Authz, NoCredentialConfigured }` (~lib.rs:846) — add `Store` here.
  - `async fn serve(listen_addr) -> Result<(), BootError>` (~lib.rs:889) — reads `ENV_PLATFORM_ADMIN_TOKEN`/`ENV_TENANT_OPERATOR_TOKEN` via `normalize_token(std::env::var(...).ok())`, refuses `NoCredentialConfigured`, then `build_inmemory_router(...)` → bind → `axum::serve`.
  - existing `#[cfg(test)] mod tests` (inline unit tests).
- `tenancy/facade/tenant-lifecycle-app/tests/acceptance.rs` — EXISTING tracked integration test file (rust_test target `tenancy-tenant-lifecycle-app-acceptance`, crate `tenancy_tenant_lifecycle_app_acceptance`). Put new tests HERE — do NOT create a new tests/ file (keeps born-accounting surface zero).
- `tenancy/adapters/tenant-lifecycle-store-postgres/src/lib.rs` (MERGED, complete durable adapter)
  - `PgTenantLifecycleStore` impl `TenantLifecycleStore` (kernel port). RLS GUC-first per-tx (`SET_LOCAL_TENANT_SQL`), `validate_tenant_id` deny-blank, decode/encode, `tenant_of_resource_name`.
  - `pub async fn connect(database_url: &str) -> Result<Self, PgStoreConnectError>` (lib.rs:145) — `PgPoolOptions::new().max_connections(8).connect(url)`, aws-lc-rs rustls TLS via workspace `sqlx` feature `tls-rustls-aws-lc-rs` (ring forbidden, ADR-0506).
  - `pub fn from_pool(pool: PgPool) -> Self` (lib.rs:~130).
  - `pub enum PgStoreConnectError { MissingDatabaseUrl, Sqlx(String) }` (lib.rs:79) + Display + Error.
  - migrations/0001_tenant_lifecycle_store.sql — ENABLE+FORCE RLS, PERMISSIVE tenant policy + RESTRICTIVE `require_tenant_guc` deny-on-empty-GUC across all 3 tables.
  - tests/live_rls.rs — env-gated (`OYA_BACKBONE_LIVE_POSTGRES`) live RLS cross-tenant probe (precedent to mirror).
- `tenancy/facade/tenant-lifecycle-app/{Cargo.toml,BUCK}` — current deps list (no postgres adapter yet).

## The change (composition-root only; NO new crate, NO new tracked file)
1. `Cargo.toml` — add dep `tenancy-tenant-lifecycle-store-postgres = { path = "../../adapters/tenant-lifecycle-store-postgres" }`.
   (No new external crates: the adapter is already a workspace member, so sqlx/etc. are already in the tree — but the
   app's `[[package]]` dependency edge changes, so **Cargo.lock MUST be regenerated** via `cargo metadata --offline`.)
2. `src/lib.rs`:
   - Add `BootError::Store(String)` (Display: "tenant store unavailable, refusing to serve: {e}") — fail-closed.
   - Add `pub async fn build_postgres_router(database_url: &str, platform_admin_token: Option<String>, tenant_operator_token: Option<String>) -> Result<Router, BootError>` mirroring `build_inmemory_router`: `PgTenantLifecycleStore::connect(database_url).await.map_err(|e| BootError::Store(e.to_string()))?` → `build_router(TenantLifecycleProvider::new(store), Arc::new(authorizer), ...)`. Same fail-closed authz compile as the in-memory path.
   - In `serve()`: after the credential check, read `DATABASE_URL` (define `const ENV_DATABASE_URL` — grep sibling composition roots first for the repo's exact convention; prefer the existing name if one is established). Branch:
     - `Some(url)` (non-empty after normalize) → `build_postgres_router(&url, ...).await?` — **on connect error, propagate `BootError::Store` and REFUSE to serve. NEVER fall back to in-memory** (silent durability downgrade = data-loss anti-pattern the founder bar forbids).
     - `None` → `build_inmemory_router(...)?` (single-node dev bring-up; keep the existing log line, add `store=inmemory` vs `store=postgres`).
   - Update the `serve` doc comment to state the DATABASE_URL→postgres / absent→in-memory selection + fail-closed-on-store-error.

## Tests (RED→GREEN; ride existing files only)
- DB-free fail-closed unit (in `src/lib.rs` tests mod or `tests/acceptance.rs`): `build_postgres_router("")` → `Err(BootError::Store(_))` (empty URL maps from `PgStoreConnectError::MissingDatabaseUrl`). Proves the wiring is fail-closed without a database. (If you assert the in-memory path is unaffected, reuse the existing acceptance fixture.)
- Env-gated live acceptance (in `tests/acceptance.rs`, mirror `live_rls.rs` gating + the buck2 `option_env!("OYA_BACKBONE_LIVE_POSTGRES")` skip-with-stderr pattern, NOT `env!`): against a live PG, drive the REST surface through `build_postgres_router` — register a tenant, then build a FRESH router over the SAME url and GET it back (proves DURABILITY across router rebuild, the thing in-memory cannot do); plus a cross-tenant 404/deny (RLS). Default `buck2 test` stays DB-free (skips cleanly when the env is unset).

## BUCK wiring (target-parity + affected-set; part of done)
- Add `//tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres` to the deps of:
  - `rust_library` `tenancy-tenant-lifecycle-app`
  - `rust_binary` `tenancy-tenant-lifecycle` (it links the lib; add only if the binary references the symbol directly — it doesn't, so likely NOT needed; verify with `buck2 build`)
  - `rust_test` `tenancy-tenant-lifecycle-app-unittest` (compiles lib with --test; needs it iff the DB-free test lives in lib.rs)
  - `rust_test` `tenancy-tenant-lifecycle-app-acceptance` (needs it for the live test)
- sqlx is TRANSITIVE via the adapter's own BUCK — do NOT add `third-party//:sqlx` to the facade directly.
- No new rust_test target (tests ride existing src/lib.rs + tests/acceptance.rs targets) → target-parity unaffected by new targets; just keep cargo↔buck dep parity.

## Clean-arch / doctrine
- Port `TenantLifecycleStore` UNCHANGED → cutover litmus holds (oya-data/G003 owned changefeed-backed store swaps the adapter; facade/composition unaffected).
- aws-lc-rs rustls only (ADR-0506, ring forbidden) — inherited from the adapter's sqlx feature; assert zero ring activation if the ring-free check runs in the affected cone.
- Tier-3: no unwrap/expect/panic in prod; `#![forbid(unsafe_code)]` (already on the crate).
- API-driven config (DATABASE_URL env) = standard 12-factor composition-root config, NOT a CLI surface (no new CLI; consistent with sibling operators reading OYA_*_NAMESPACE etc.).
- born-accounting: NO new tracked files (all edits to existing Cargo.toml/BUCK/src/lib.rs/tests/acceptance.rs) → firewall GO-LIVE should not regress. STILL: regen Cargo.lock (cargo metadata --offline), materialize faces, run firewall GO-LIVE (`firewall_is_green_on_the_live_corpus_with_the_baseline`) + freshness + affected-set gates locally BEFORE push (buck2-build-green ≠ CI-green), face-settle --verify byte-identical.

## KNOWN GAP to surface in the PR + adversarial review (do NOT silently overclaim)
- migration `0001` has ENABLE+FORCE RLS + PERMISSIVE+RESTRICTIVE policies but provides **NO runtime-role provisioning** (no `0000_runtime_role.sql`, unlike G003 #797's `CREATE ROLE ... NOBYPASSRLS`). A superuser/`BYPASSRLS` `DATABASE_URL` would silently skip tenant isolation. This is PRE-EXISTING in the tenancy adapter (AUTH-005 era), not introduced by this composition-root slice. PR claim must be honest: "wires the durable store fail-closed behind DATABASE_URL; RLS isolation depends on a correctly-provisioned NOBYPASSRLS role — role provisioning is a tracked follow-up (mirror #797), NOT in this slice's scope." Record the follow-up task; do NOT expand this slice to add the migration (blast-radius discipline) unless the adversarial review rules it a blocker for wiring.

## Worktree / process discipline (NON-NEGOTIABLE)
- NEVER git-mutate the canonical checkout /Users/jasonlee/Developer/oyatie. Work on a FRESH worktree off origin/dev (`git worktree add`), branch, implement, push, open PR against dev.
- Commit trailer EXACTLY: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
- Build/verify with buck2 (cargo build/test are hook-blocked; `cargo metadata --offline` is allowed for the lock).
- After build green + local gates green: STOP. Do not self-approve. Orchestrator runs a separate adversarial security/code review pass before merge.
