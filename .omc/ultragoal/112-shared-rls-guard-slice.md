# #112 — Productize the RLS-enforceability guard into a shared postgres lib — architect-scoped 2026-06-22 (dev f764a28cd)

Founder "manual-twice → productize": the boot-time RLS-enforceability guard is hand-written TWICE (tenancy + SCIM adapters) and ALREADY DIVERGED on a correctness-critical keyword (`'MEMBER'` vs `'USAGE'`). Extract ONE shared guard. ZERO new crates (extend two existing libs) → zero born-accounting. Also folds in the #799 FORCE-RLS hardening + unifies the keyword on the correct `'USAGE'`.

## Placement (HARD constraint: kernel-purity gate ADR-0547 denies sqlx on *-kernel)
- **PURE half → `libs/oya-shared-postgres-command-kernel/src/lib.rs`** (has NO `[dependencies]` today — MUST stay sqlx-free): the shared error enum, the SQL-string consts, the pure decision fns. Inherits the kernel's always-on `*-unittest`.
- **EXECUTOR half → `libs/oya-shared-postgres-command-adapter-sqlx/src/lib.rs`** (already deps sqlx + the kernel): the `&PgPool` async guard. NO new dep.

## Kernel additions (pure, sqlx-free)
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RlsEnforceabilityError {
    Unenforceable { role: String },                      // rolsuper||rolbypassrls
    RoleMismatch { role: String, expected: String },     // not USAGE-member of runtime role
    RoleSwitchInEffect { session_role: String, current_role: String }, // session_user!=current_user
    RlsNotForced { table: String, row_security: bool, force_row_security: bool }, // #799 hardening
    GovernedTableMissing { table: String },
}   // + Display + std::error::Error (Tier-3: no panic)

pub const RLS_ROLE_PROBE_SQL: &str =
  "SELECT current_user::text AS role_name, session_user::text AS session_role, \
   rolsuper, rolbypassrls, pg_has_role(current_user, $1, 'USAGE') AS is_runtime_member \
   FROM pg_roles WHERE rolname = current_user";   // 'USAGE' = has_privs_of_role (CORRECT; verify via PG docs)
pub const RLS_TABLE_FORCED_PROBE_SQL: &str =
  "SELECT c.relrowsecurity AS row_security, c.relforcerowsecurity AS force_row_security \
   FROM pg_class c JOIN pg_namespace n ON c.relnamespace = n.oid \
   WHERE n.nspname = $1 AND c.relname = $2";

pub fn evaluate_rls_role_flags(role: &str, rolsuper: bool, rolbypassrls: bool, is_runtime_member: bool) -> Result<(), RlsEnforceabilityError>;
pub fn evaluate_rls_forced(qualified_table: &str, row_security: bool, force_row_security: bool) -> Result<(), RlsEnforceabilityError>;
```
Move the DB-free `evaluate_rls_enforceability` unit tests from BOTH adapters here (dedup), now testing `evaluate_rls_role_flags`; add `evaluate_rls_forced` tests (forced ok; not-enabled→err; not-forced→err).

## sqlx-adapter addition (the executor)
```rust
/// Fail-closed boot guard: (1) current_user can't bypass RLS + is USAGE-member of runtime_role,
/// (2) every governed table has ENABLE+FORCE RLS. Run AFTER connect, BEFORE serving.
/// governed_tables are SCHEMA-qualified ("schema.table"); split on the FIRST '.' for nspname+relname.
pub async fn assert_rls_enforceable(pool: &sqlx::PgPool, runtime_role: &str, governed_tables: &[&str]) -> Result<(), RlsEnforceabilityError>;
```
Binds RLS_ROLE_PROBE_SQL → evaluate_rls_role_flags (+ the session_user==current_user check → RoleSwitchInEffect); then loops governed_tables binding RLS_TABLE_FORCED_PROBE_SQL → evaluate_rls_forced (fetch_optional; None→GovernedTableMissing). sqlx errors → a clear shared/adapter error (fail-closed). Tier-3 no unwrap/expect/panic.

## Retrofit (thin delegates; call sites compile UNCHANGED)
- **tenancy adapter** `tenant-lifecycle-store-postgres/src/lib.rs`: gut the local `evaluate_rls_enforceability` (308-327) + the body of the `assert_rls_enforceable(&self)` METHOD (235-278); keep the method as a 1-line delegate → `assert_rls_enforceable(&self.pool, RUNTIME_ROLE, &[TENANTS_TABLE, APPLIED_TABLE, OPERATIONS_TABLE])` mapped via `From<RlsEnforceabilityError>`. Facade call `store.assert_rls_enforceable()` (tenant-lifecycle-app/src/lib.rs:897) UNCHANGED.
- **SCIM adapter** `identity-scim-store-postgres/src/lib.rs`: gut local predicate (260-279) + the FREE-fn body (186-239); keep the free fn `assert_rls_enforceable(&PgPool)` as a 1-line delegate → shared guard with `&[USERS_TABLE, GROUPS_TABLE]` mapped. Facade call `assert_rls_enforceable(&pool)` (identity-service/src/server.rs:284) UNCHANGED.
- **Error mapping (Q2 decision):** KEEP each adapter's existing `PgStoreConnectError`/`PgScimConnectError` (their `MissingDatabaseUrl`/`Sqlx` variants + facade `BootError::Store(e.to_string())`/`StartError::Store(...)` stay byte-identical). Keep `RlsUnenforceable`/`RlsRoleMismatch` variants (existing `matches!` tests keep compiling) + ADD `RlsNotForcedOnTable { table }`. Add `From<RlsEnforceabilityError>` per adapter: Unenforceable→RlsUnenforceable; RoleMismatch→RlsRoleMismatch; RoleSwitchInEffect→Sqlx(detail) (preserves today's behavior); RlsNotForced/GovernedTableMissing→RlsNotForcedOnTable. Each adapter keeps a THIN call-site/mapping test.

## Deps / buck / gates (part of done)
- Both adapters add `oya-shared-postgres-command-adapter-sqlx` to Cargo.toml `[dependencies]` AND BUCK deps (rust_library + *-unittest + *-live targets). BUCK label `//libs/oya-shared-postgres-command-adapter-sqlx:oya-shared-postgres-command-adapter-sqlx`. Keep cargo↔buck parity (target-parity gate).
- NO new tracked file/crate. Regen Cargo.lock (`cargo metadata --offline`; likely no change — workspace-internal edge).
- Run locally BEFORE push: kernel-purity (CRITICAL — verify the kernel still has ZERO sqlx in its dep closure; the executor lives in the sqlx adapter), firewall GO-LIVE, freshness (lock+faces ADR-0539), affected-set (ADR-0554), target-parity, the `*-unittest` + (env-gated, skip-clean) `*-live` tests, face-settle --verify.
- Live tests (`tests/live_rls.rs` in both adapters: `live_app_role_has_no_bypassrls`, `live_assert_rls_enforceable_*`) call the public method/free-fn → unchanged, must still pass (both 0001 migrations already ENABLE+FORCE RLS so the new FORCE check passes). OPTIONAL RED live test (drop FORCE on one table → RlsNotForcedOnTable) — only if it doesn't expand scope; else a #112 follow-up.

## Clean-arch / doctrine
- Single decision SSOT; the two ergonomic surfaces (tenancy method, SCIM free fn) are thin intentional delegates. Ports unchanged. `'USAGE'` correctness fix delivered uniformly. FORCE-RLS check = stronger fail-closed (catches a mis-migrated/RLS-disabled DB at boot). Independent of #113 (role provisioning) — do #112 first; it makes a missing #113 fail loudly.
- This is SECURITY-CRITICAL refactor of MERGED tenant-isolation code in TWO verticals → after build+gates green, STOP, do NOT self-approve; orchestrator runs adversarial review (verify: no fail-open introduced, both adapters' fail-closed contract byte-identical, the FORCE-RLS query correct, no regression to live tests).

## Process discipline
Fresh worktree off origin/dev (f764a28cd); NEVER touch the canonical checkout. buck2 for build/test (cargo hook-blocked; cargo metadata allowed for lock). Commit trailer EXACTLY `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
