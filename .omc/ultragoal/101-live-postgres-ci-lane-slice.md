# #101 — Live-Postgres CI lane: make the dark cross-tenant-deny/RLS/CDC/SCIM tests ACTUALLY RUN + gate — 2026-06-22 (dev b9114dc7c)

Founder Stop-hook (2026-06-22) REJECTED the prior "founder-gated/await" framing: this is a literal CI job a PR adds NOW; the work to start IS opening the PR. HIGHEST-LEVERAGE slice per the evidence-based G001-G013 status — it converts THREE stories' (G03/G05/G06) integration tests from FALSE-GREEN (env-gated early-return AND absent from the CI gate matrix → pass having never touched a DB) into real, merge-gating evidence. Directly addresses the founder's G003 dispute ("cross-tenant-deny proven against a containerized DB") + the masterplan false-green prohibition + memory buck2-build-green≠CI-green.

## The dark tests this turns ON (verify exact names + env in the code)
- `libs/oya-data-sql-adapter-sqlx` — `run_live_rls_cross_tenant_probe` / `live_rls_cross_tenant_deny_when_enabled` (env `OYA_DATA_LIVE_POSTGRES`, `OYA_DATA_POSTGRES_ADMIN_URL`, `OYA_DATA_POSTGRES_APP_URL`).
- `libs/oya-data-outbox-adapter-postgres` — `run_live_cdc_cross_tenant_probe` (env `OYA_OUTBOX_LIVE_POSTGRES`, `OYA_OUTBOX_POSTGRES_ADMIN_URL`, `OYA_OUTBOX_POSTGRES_APP_URL`); migrations `0000_runtime_role.sql` + `0001_outbox_events.sql`.
- `tenancy/adapters/tenant-lifecycle-store-postgres/tests/live_rls.rs` + facade durability (env `OYA_BACKBONE_LIVE_POSTGRES`, `OYA_BACKBONE_POSTGRES_URL`=admin/superuser, `OYA_BACKBONE_POSTGRES_APP_URL`=app role).
- `iam/adapters/identity-scim-store-postgres/tests/live_rls.rs` + `iam/facade/identity-service` live durability (same OYA_BACKBONE_* family).
VERIFY every env-var name + the enable predicate + which URL is admin vs app by reading each test's harness BEFORE writing the workflow (do not assume).

## The change
1. New job in `.github/workflows/oya-ci-required.yml`, e.g. `gate-live-postgres`, modeled on the existing `gate`/`buck2` jobs:
   - `services: postgres:` using `postgres:16` (NOT Citus initially; leave any `LIVE_POSTGRES_REQUIRE_CITUS_ENV` UNSET), with a health-check (`pg_isready`) gate before the test step.
   - Bootstrap step: apply each durable adapter's `0000_runtime_role.sql` as the ADMIN/superuser connection so the NOBYPASSRLS app role exists (the env-gated tests apply their own table migrations 0001+ via the admin/app URLs — confirm per-adapter how the harness applies migrations; some apply 0000 themselves via include_str! + the shared split_migration_statements). Create the app-role login + `GRANT <runtime_role> TO <app_login>` if the harness expects a pre-existing app login (mirror each adapter's tests/live_rls.rs setup expectations).
   - `env:` set ALL three families (OYA_BACKBONE_*, OYA_DATA_*, OYA_OUTBOX_*): the enable flags = truthy, admin URL = superuser DSN, app URL = the NON-superuser app-role DSN (CRITICAL: app URL must be a NOBYPASSRLS role or the shared RLS-enforceability guard correctly refuses — that's the point).
2. RUNNER choice (document the doctrine rationale): PREFER buck2 (canonical) — run the `-live` rust_test targets with the env set IF buck2 test permits localhost network to the service container. If buck2's test sandbox BLOCKS network (likely), use `cargo test --locked -p <pkg> -- --test-threads=1` as a TRANSITIONAL BRIDGE lane (explicit precedent: the `app-shell-codegen` non-Rust bridge lane, documented "transitional until the native runner has first-class support"), and NOTE the cloud-native ephemeral-env destination. Determine which works; document the choice + why. Packages (verify exact cargo names from each Cargo.toml): oya-data-sql-adapter-sqlx, oya-data-outbox-adapter-postgres, the tenancy + SCIM store-postgres adapters, identity-service, tenant-lifecycle-app.
3. GATING: add the new job to the `oya-ci-required` fan-in `needs:` list so it ACTUALLY gates merge (else it repeats false-green at the branch-protection layer). BUT only after the tests are confirmed green (see done-bar) — the lane ships green+gating in the same PR.

## RED-proof (MANDATORY — prove the tests are no longer no-ops)
Locally (or in a scratch run): temporarily break ONE RLS policy (e.g. drop a RESTRICTIVE require-tenant-guc policy, or point the app URL at a BYPASSRLS role) and confirm a live test FAILS. Then revert. This proves the lane catches a real isolation regression (not an early-return). Report the RED evidence.

## Fix what the tests reveal
Turning the dark tests on may surface REAL pre-existing failures (latent bugs the dark tests would have caught). Fix the straightforward ones in this slice. If a failure is a large/unrelated fix, report it (carve to a follow-up) — but the lane must end GREEN to merge (so either fix or, if a test is genuinely wrong, correct the test with justification — do NOT weaken a real assertion to pass).

## Verification reality (be honest)
cargo test is HOOK-BLOCKED locally; the executor's sandbox may lack a live PG. So local verification is LIMITED to: workflow YAML validity, env-var-name correctness (grepped from the test code), migration/role bootstrap order, package-name existence, and (if docker+network available) actually spinning a PG + running the -live targets via buck2/cargo. If you CANNOT run the live tests locally, BUILD the lane correctly + open the PR + state clearly that the live tests' first real execution is the PR's CI run (the new lane), which I will monitor. Do NOT claim the live tests pass if you didn't run them.

## born-accounting / pipeline-discipline
Workflow-file change + any new bootstrap script: ensure gate-registration meta-test + automation-ratchet gate still pass; if any cloud-ci face references the gate set, regen faces (`infra/ci/materialize-cloud-ci-generated-faces.sh`) + run freshness + registry-drift locally. If a new bootstrap .sh is added, that conflicts with the no-shell doctrine — prefer a Rust bootstrap binary OR an inline workflow `run:` step (workflows are already shell); document as transitional. NO new untracked files.

## Process
Fresh worktree off origin/dev; NEVER touch the canonical checkout; commit trailer EXACTLY `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. After the lane is built + (locally-verifiable parts) green: STOP, do NOT self-approve — orchestrator reviews + monitors the PR's first live-lane CI run.
