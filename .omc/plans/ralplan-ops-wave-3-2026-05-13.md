---
purpose: Auto-backfilled purpose for ralplan-ops-wave-3-2026-05-13.md
---

---
doc_class: RalplanConsensusPlan
shape: anchor
status: Accepted
version: v5
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: /specs/cross-cutting/decision-principles.json + /specs/cross-cutting/forbidden-operations.json
authority_chain: docs/MASTERPLAN.md → ADR-0003 (Oyatie audit-chain) + Bominal ADR-0028 (audit-chain segments inheritance) + ADR-0045 (database tier) + ADR-0065 + ADR-0067 → ralplan-ops-portal-2026-05-13.md v7 Accepted → ralplan-ops-wave-2-2026-05-13.md v7 Accepted → this plan
parent_plan: .omc/plans/ralplan-ops-portal-2026-05-13.md (ops.oyatie.com 20-BC parent v7 Accepted; this plan is Wave 3 of 7)
companion_plans:
  - .omc/plans/ralplan-docs-portal-2026-05-13.md (Wave 1 sub-plan v7 Accepted)
  - .omc/plans/ralplan-ops-wave-2-2026-05-13.md (Wave 2 sub-plan v7 Accepted via critic r2 `b10gta4kj`)
codex_model: gpt-5.5 / xhigh
predecessor_dispatch: parent §8 follow-up #4
verification_round: critic r2 ✅ APPROVE (codex `b8qzjzqh5`; 9/9 PASS — all 4 previously WEAK/FAIL criteria promoted: 4 testable-acceptance PASS, 5 verification PASS, 7 expanded-test PASS, 9 user-mandated rules PASS; critical findings: None; required fixes: None); status Accepted; 8 §8 follow-ups dispatch begins next step; parent §8 #4 marked done; Wave 4 ralplan unblocked
---

# Implementation Plan: `ops.oyatie.com` Wave 3 — Database + Schema BCs

## §1 Principles (RALPLAN-DR; 5 principles; v1)

1. **Inherit, don't redefine.** Wave 3 inherits parent + Wave 1 + Wave 2 contracts verbatim: 6-tier visibility taxonomy; 4 M02-P20 Cedar fragments + `ops-system-only.cedar` Grafana-key extension from Wave 2; per-route p99 ≤500ms + shell overhead ≤50ms; canonical M03 paths; `SURFACE_CATALOG` per-IP Live-flip gate. No re-derivation.
2. **Workspace-surface composition.** Database + schema BCs ship as ops.workspace embedded surfaces #6 + #7 (after Wave 1 docs #1 + Wave 2 overview/dashboards/tech-stack/architecture #2-#5). Reverse-proxy via workspace-shell + tower-http; in-process fallback if overhead > 50ms p99.
3. **Visibility-tier split (per parent §6(a) Wave 3 row):** `database` BC has two surface modes:
   - **MVP (internal-public):** `/workspace/database` per-µservice schema overview + migration status (read-only metadata; no row data).
   - **Sample-data viewer (internal-private):** `/workspace/database/sample-data?µservice=X` — gated admin-only viewer for production debugging. Covered by M02-P20 `ops-internal-private.cedar`; lean-a9 red-team probe required day 1.
   `schema` BC is single-mode: `/workspace/schema` per-µservice SQL DDL + ER diagrams (internal-public; from `oya-ops-docs-extract-sql-migrations` Wave 1 G2 extractor).
4. **Manifest-derived rendering via shared read port.** Database + schema BCs read SQL migration facts through `SharedManifestReadPort` (the shared kernel introduced at Wave 2 IP-X1-catalog-integration). Sample-data viewer reads from per-µservice DB directly via a dedicated `DatabaseSampleViewerPort` (NEW Wave 3 port; tenant_id RLS-enforced per `oya-check-shardability`).
5. **Cross-BC fan-in via WorkflowBridgePort.** If overview surface needs to render a "database health" tile sourced from this BC, the call routes through `WorkflowBridgePort::query_tile_health` (extended at Wave 2 IP-X1; per `feedback_workflow_objectgraph_adapter_layer.md`). No direct kernel imports across BCs.

---

## §2 Decision Drivers (top 3)

1. **SRE day-2 debugging utility.** A common cause of incidents is "I need to see what's actually in the production database for tenant X µservice Y" — currently requires CLI access to per-cell DB pods. Wave 3 surfaces this safely with audit-chain logging + Cedar gate + RLS enforcement.
2. **Schema visibility drives correct change management.** SRE/Foundry teams need to see per-µservice DDL evolution (migration history, distribution columns, RLS policies) to validate that new code respects substrate contracts. `/workspace/schema` makes this visible without grep through migrations/.
3. **Wave 3 is small (2 BCs) — keeps M03 critical path tight.** This Wave's est. wall-clock is ~6h (vs Wave 2's ~24h for 4 BCs + IP-X1). Fast Wave dispatch unblocks Wave 4 (observability + health perf-gate) sooner.

---

## §3 Wave 3 Bounded-Context Inventory

### §3.1 BC inventory

| # | BC | Surface route | Visibility tier (MVP) | Phase + IP slot | Est. crates |
|---|----|---------------|-----------------------|------------------|--------------|
| 1 | database | `/workspace/database` (per-µservice schema overview + migration status) + `/workspace/database/sample-data` (admin sample-data viewer) | `/workspace/database` = internal-public; `/workspace/database/sample-data` = internal-private (M02-P20 `ops-internal-private.cedar` covers) | M03-P08 cross-axis-contracts IP-X6-ops-database-bc | 7 crates (architect r1 fix 3 — was 8; `database-config` folded into `application` layer; final: `kernel`, `application`, `adapter`, `adapter-sample-viewer`, `rest`, `pages`, `app`) |
| 2 | schema | `/workspace/schema` (per-µservice SQL DDL + ER diagrams from `oya-ops-docs-extract-sql-migrations`) | internal-public | M03-P08 cross-axis-contracts IP-X7-ops-schema-bc | 6 crates (architect r1 fix 3 — was 5; added own `-app` per Wave 2 critic r1 fix 2 pattern: `kernel`, `application`, `adapter`, `rest`, `pages`, `app`) |

**Total: 13 BC crates** (architect r1 fix 3 — internally consistent across §3 and §6) + 3 new fitness lanes (`oya-ops-database-check-data-leak-sample-viewer`, `oya-ops-database-check-sample-viewer-rls`, `oya-ops-database-check-migration-drift`) = **16 crates net new** at Wave 3. All `oya-ops-*` named.

### §3.2 Route contract table

| Route | Owner BC | Visibility tier | Behavior | Redirect / alias | OpenAPI contract |
|---|---|---|---|---|---|
| `/workspace/database` | database | internal-public | Per-µservice schema overview (table counts, RLS policies, distribution columns); reads from `SharedManifestReadPort.sql_migrations_section` | none | `contracts/ops-database.openapi.yaml` |
| `/workspace/database/sample-data` | database | **internal-private** (M02-P20 `ops-internal-private.cedar`) | Admin-gated sample-data viewer: reads top-N rows from per-µservice DB via `DatabaseSampleViewerPort`; **RLS-enforced (tenant_id filter mandatory)**; **audit-chain row per query** (who/what/when/which-µservice) | none | `contracts/ops-database.openapi.yaml` |
| `/workspace/database/api/v1/health` | database | internal-public | Tile-health endpoint called via `WorkflowBridgePort::query_tile_health` by overview BC; returns `TileHealth { migration_drift_count, rls_violation_count, distribution_column_violation_count }` | none | `contracts/ops-database.openapi.yaml` |
| `/workspace/schema` | schema | internal-public | Per-µservice SQL DDL viewer; ER-diagram rendered via mermaid.js from `SharedManifestReadPort.sql_migrations_section` | none | `contracts/ops-schema.openapi.yaml` |
| `/workspace/schema/api/v1/health` | schema | internal-public | Tile-health endpoint called via `WorkflowBridgePort::query_tile_health` | none | `contracts/ops-schema.openapi.yaml` |

---

## §4 Pre-mortem (4 scenarios — deliberate-mode required)

### Scenario 1 — Sample-data viewer leaks cross-tenant rows

**Outage shape:** Admin opens `/workspace/database/sample-data?µservice=tenancy` to debug an incident. The `DatabaseSampleViewerPort` impl forgets to apply the `tenant_id` RLS filter, returns rows from MULTIPLE tenants. Admin sees tenant A's data while debugging tenant B's incident → privacy contract violation per `feedback_no_silent_regression.md`.

**Detection:** `oya-check-shardability` lane (BLOCKER day 1 from M02-P20) MUST detect any sample-viewer query lacking `tenant_id` predicate at code review. Plus a Cedar red-team probe: `oya-check-data-leak-sample-viewer` (BLOCKER day 1 from this Wave 3) injects a synthetic two-tenant fixture and asserts the viewer returns ONLY one tenant's rows.

**Prevention (architect r1 fix 1 — `sample_viewer_ro` grant/RLS hardening added):**
- `DatabaseSampleViewerPort` trait MUST require `tenant_id` parameter in every query method signature; no default empty string.
- Adapter impl MUST use parameterized queries with RLS-active sessions (NEVER raw SQL).
- Per-query audit-chain row (Bominal ADR-0028 + Oyatie ADR-0003 inheritance — same-segment cohort signing) records `(principal_id, tenant_id, µservice, query_template, ts)`.
- Cedar fragment `ops-internal-private.cedar` (M02-P20) checks PrincipalRole AND TenantContext binding — admin role for a cell does NOT imply cross-tenant query authority.
- **`sample_viewer_ro` Postgres role hardening (architect r1 fix 1):**
  - Role attributes: **no `BYPASSRLS`**, **no `SUPERUSER`**, no `pg_read_all_data` / `pg_read_server_files` / `pg_signal_backend` grants. The role is REVOKE'd by default at cluster init; granted only specific table/view SELECT.
  - **Table/view allowlist:** `database-config.yaml::sample_viewer.allowed_relations` enumerates every (microservice, schema.table_or_view) pair the viewer may read. Tables NOT in the allowlist are inaccessible at the grant level (defense-in-depth even if RLS slips). Allowlist defaults to `[]` (zero relations); each µservice's IP-X6 follow-up amends the file with vetted entries.
  - **FORCE RLS / security-barrier views required:** every allowlisted relation MUST have `ALTER TABLE ... FORCE ROW LEVEL SECURITY` AND a non-empty `tenant_isolation` policy on `tenant_id`. Pre-deploy validator `oya-check-sample-viewer-rls` (NEW Wave 3 lane, BLOCKER day 1) inspects: (a) `sample_viewer_ro` role attributes, (b) every allowlisted relation has `relrowsecurity = true AND relforcerowsecurity = true`, (c) `pg_policies` shows a `tenant_isolation` policy referencing the principal's `tenant_id` session var, (d) no allowlisted relation has cross-tenant FKs without a parallel `tenant_id` predicate.
  - **Per-cell read-replica binding:** adapter connects to each cell's read-replica via mTLS-bound Istio sidecar (Bominal ADR-0044 inheritance); never to the primary; never via shared service-principal across cells (per-cell SecretReference to OpenBao path).

**Recovery:** Immediate revert of the offending PR; emit DSR cascade per ADR-0038 for the leaked-tenant data; rotate any service-principal secrets that touched the cell; review all sample-viewer queries in audit-chain for the affected period; re-run `oya-check-sample-viewer-rls` against all allowlisted relations.

### Scenario 2 — Schema view's mermaid-render p99 violates 500ms budget on large schema

**Outage shape:** `/workspace/schema?µservice=ontology` triggers mermaid.js render of ~500 tables with FK edges. SSR exceeds 500ms p99 because rendering happens server-side.

**Detection:** k6 per-route load + `ops_workspace_route_render_duration_seconds{route="/workspace/schema"}` Prometheus gauge.

**Prevention:**
- Mermaid render is CLIENT-side (WASM hydrate) for schemas with > 50 tables. SSR returns minimal HTML + per-µservice mermaid source as inline `<pre>`; client renders.
- For < 50 tables: SSR-render directly (still under 500ms p99).
- Per-µservice schema cached in `SharedManifestReadPort` (Wave 2 IP-X1 introduced) — never re-extract per request.

**Recovery:** Add request-level cache; profile; consider sharding the schema view by namespace if a single µservice exceeds 500 tables.

### Scenario 3 — Database sample-data viewer becomes attack surface against production DB

**Outage shape:** A compromised admin SSO session is used to repeatedly query sample-data viewer to exfiltrate production rows (one tenant at a time, slowly to evade rate limits).

**Detection:** Per-principal anomaly detector on audit-chain rows: alert if sample-viewer query count > 50/hour OR > 10 distinct µservices/day for any principal.

**Prevention:**
- **Rate limit:** 30 sample-viewer queries / principal / hour at the rest layer (tower-governor middleware).
- **Per-query justification:** UI requires a free-text "incident ID or ticket reference" before query executes; recorded in audit-chain.
- **Row-count cap:** Sample-viewer returns max 100 rows per query (NOT unbounded `SELECT *`). Configurable per-µservice via `database-config.yaml`.
- **Read-only DB role:** `DatabaseSampleViewerAdapter` connects with a `sample_viewer_ro` Postgres role that has SELECT-only + LIMIT-only permissions (no UPDATE/DELETE/INSERT/COPY).

**Recovery:** Disable sample-viewer route via feature flag; rotate compromised admin SSO + service-principal secrets; audit-chain forensics on the principal's recent queries; emit DSR cascade if rows touched contain PII.

### Scenario 4 — Schema view drifts from actual production DB schema

**Outage shape:** `/workspace/schema` displays migration V042 as applied, but production DB on cell-us-west-2 hasn't applied V042 yet (migration in-flight). SRE acts on stale schema info, makes wrong incident call.

**Detection:** `database-config.yaml` lists per-cell migration status; if any cell lags expected version by > 2 versions, surface RED tile health.

**Prevention:**
- `/workspace/database` health endpoint reports per-cell `applied_migration_version` (sourced from per-cell `flyway_schema_history` table via cell read-replica).
- Schema view shows BOTH "manifest version" (from `SharedManifestReadPort`, source-of-truth = migrations/ dir) AND "applied versions per cell" with explicit drift badge.
- `oya-check-migration-drift` lane (NEW Wave 3 fitness lane; **report-only at IP-X6 acceptance → BLOCKER at the next exit-gate-like cycle** — critic r1 fix 2; aligns with §6/§8 wording since M02-P22 already closed before Wave 3 dispatches) verifies per-cell `applied` matches `manifest_version` ± 2.

**Recovery:** Investigate why cell is lagging (failed deploy, manual hold?); manually apply pending migrations or roll back to known-good state.

---

## §5 Expanded test plan (deliberate mode)

| Layer | What |
|---|---|
| **Unit** | Each `oya-ops-{database,schema}-{kernel,application,adapter}` crate: golden-fixture port-impl assertions. ~20 unit tests total. |
| **Integration** | Per-BC: mount in workspace shell → render default route → assert visibility-tier (internal-public 200; anonymous 401; tenant-member 403 for /database/sample-data). |
| **E2E** | Playwright: admin user → /workspace/database/sample-data?µservice=tenancy → renders ≤100 rows; query requires incident-ID input; rate limit kicks in at 31st query/hour. |
| **Observability** | OTel: every request has `surface_id`, `bc`, `principal_role`, `visibility_tier`, `µservice` (when applicable), `tenant_id` (when applicable) span attrs; audit-chain row per /sample-data query. |
| **Performance (critic r1 fix 3)** | k6 per-route p99 ≤500ms for **`/workspace/database`, `/workspace/database/sample-data`, `/workspace/schema`** (all 3 page routes; `/sample-data` is the highest-risk Wave 3 surface and MUST be gated); per-route p99 ≤500ms also for tile-health API endpoints `/workspace/database/api/v1/health` + `/workspace/schema/api/v1/health`; per-route shell overhead p99 ≤50ms per ADR-0067 §5. No cross-surface SSR fanout (Wave 2 §4 Scenario 1 rule inherited). |
| **Security (Cedar red-team)** | Synthetic-tenant probe: tenant-member → 403 on /database/sample-data; internal-foundry → 200; anonymous → 401. Cross-tenant probe: admin queries sample-data → asserts no rows returned for any tenant_id other than the admin's bound tenant context. |
| **Security (data-leak probe)** | `oya-check-data-leak-sample-viewer` (NEW Wave 3 lane; BLOCKER day 1): two-tenant fixture; asserts sample-viewer returns rows for ONLY the requested tenant_id. |
| **Security (sample_viewer_ro grant/RLS validator) (architect r2 fix 1)** | `oya-check-sample-viewer-rls` (NEW Wave 3 lane; BLOCKER day 1): inspects `sample_viewer_ro` role attrs (no BYPASSRLS / no SUPERUSER / no `pg_read_all_data`); verifies every allowlisted relation in `database-config.yaml::sample_viewer.allowed_relations` has `relrowsecurity = true AND relforcerowsecurity = true`; verifies `pg_policies` shows `tenant_isolation` policy referencing principal's `tenant_id` session var; asserts no allowlisted relation has cross-tenant FKs without parallel `tenant_id` predicate. Per architect r1 fix 1. |
| **Migration-drift (critic r1 fix 2)** | `oya-check-migration-drift` (NEW Wave 3 lane; **report-only at IP-X6 acceptance → BLOCKER at the next exit-gate-like cycle**): asserts `applied_migration_version` matches `manifest_version ± 2` for every cell. |
| **Docs snapshot** | `oya-shared-documentation-check-cli --blocker` exits 0: 2 BC registrations present, 2 PRDs per BC, microservice record updated. |
| **No-silent-regression** | `lean-a10`: any new /workspace/database* route in `contracts/ops-database.openapi.yaml` carries ADR + version bump + sunset. |

---

## §6 Implementation surface

### §6(a) Crate inventory per BC (13 BC crates + 3 new fitness lanes = 16 crates net; architect r1 fix 2 + fix 3)

| BC | Crates |
|----|--------|
| database (7) | `oya-ops-database-kernel`, `oya-ops-database-application` (folds yaml-config module), `oya-ops-database-adapter`, `oya-ops-database-adapter-sample-viewer` (NEW separate adapter per ADR-0064 canonical-base + adapter pattern; isolates production-DB-read concern), `oya-ops-database-rest`, `oya-ops-database-pages`, `oya-ops-database-app` |
| schema (6) | `oya-ops-schema-kernel`, `oya-ops-schema-application`, `oya-ops-schema-adapter`, `oya-ops-schema-rest`, `oya-ops-schema-pages`, `oya-ops-schema-app` |
| fitness lanes (3 — NEW Wave 3; architect r1 fix 2 canonicalized lane names + M02-P22 BLOCKER-list wiring) | `oya-ops-database-check-data-leak-sample-viewer` (BLOCKER day 1 — registered in `registry/quality/lanes.yaml` as `lean-a-data-leak-sample-viewer` + added to M02-P22 BLOCKER list at IP-X6 acceptance gate), `oya-ops-database-check-sample-viewer-rls` (BLOCKER day 1 — validates `sample_viewer_ro` role attrs + FORCE RLS state on every allowlisted relation per architect r1 fix 1; registered as `lean-a-sample-viewer-rls`; added to M02-P22 BLOCKER list at IP-X6 acceptance), `oya-ops-database-check-migration-drift` (report-only at IP-X6 acceptance → BLOCKER at next M02 exit-gate-like cycle; registered as `lean-a-migration-drift`) |

**Total: 13 BC + 3 lanes = 16 crates net new at Wave 3.**

### §6(b) Phase / IP mapping (architect r2 fix 3 — IP-X7 depends on IP-X6 catalog prelude)

| Phase | IP | BC | Owner | Predecessor |
|---|---|---|---|---|
| M03-P08 cross-axis-contracts | IP-X6-ops-database-bc (catalog-prelude + main; see §6(c) Step 1 + Step 2) | database | council-foundry | Wave 2 aggregate gate complete |
| M03-P08 cross-axis-contracts | IP-X7-ops-schema-bc | schema | council-foundry | IP-X6 catalog-prelude `grit done` (architect r2 fix 3 — NOT fully parallel from dispatch; IP-X7 needs IP-X6's `SURFACE_CATALOG::Schema` registration + shell OpenAPI extension to be merge-safe) |

IP-X7 starts AFTER IP-X6 catalog prelude `grit done` (Step 1 of §6(c)). After that point, IP-X6 main + IP-X7 run in parallel under M03.W5.E + M03.W5.F. Symbol-disjoint by `oya-ops-{database,schema}-*` namespaces; shared workspace-catalog/contract symbols are serialized through IP-X6 catalog prelude.

### §6(c) Dispatch sequence (architect r1 fix 4 — shared workspace-catalog claim owner serialized)

**Step 1 (serial — IP-X6 catalog-extension prelude):** IP-X6-ops-database-bc lands FIRST. In addition to owning `crates/oya-ops-database-*`, IP-X6 owns the SHARED workspace-catalog claim space for BOTH Wave 3 surfaces (since database is the more substantive BC and authoring two parallel catalog edits creates merge-conflict risk per `feedback_no_silent_regression.md`):
- `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG::Database` (REGISTER with `status: ReservedComingSoon (pending IP-X6 smoke-test)`)
- `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG::Schema` (REGISTER with `status: ReservedComingSoon (pending IP-X7 smoke-test)`)
- `docs/standards/workspace-surfaces.md` (add 2 new rows; status ReservedComingSoon for both)
- `contracts/ops-workspace-shell.openapi.yaml` (extend with `/workspace/database`, `/workspace/database/sample-data`, `/workspace/schema`, 2 tile-health endpoints — semver bump per `feedback_no_silent_regression.md`)
- `registry/quality/lanes.yaml` (add 3 new lean lanes: `lean-a-data-leak-sample-viewer` BLOCKER day 1; `lean-a-sample-viewer-rls` BLOCKER day 1; `lean-a-migration-drift` report-only initially)
- `.github/workflows/ci-fitness-lanes.yml` (wire 3 new lane jobs)
- **M02-P22 BLOCKER-list extension** (architect r1 fix 2 + critic r1 fix 1): `.omc/plans/milestones/M02-substrate/phases/P22-m02-exit-gate/impl-plan.md` lane-flip table + acceptance-gates list extended with **all 3 new Wave 3 lanes**: `lean-a-data-leak-sample-viewer` (BLOCKER day 1 at IP-X6 acceptance) + `lean-a-sample-viewer-rls` (BLOCKER day 1 at IP-X6 acceptance) + `lean-a-migration-drift` (report-only at IP-X6 acceptance → BLOCKER at the next exit-gate-like cycle after M02-P22; flip cycle to be documented in the M02-P22 amendment text). This is a CROSS-Wave touch — coordinate with M02-P22 owner; the BLOCKER-list extension does NOT block M02-P22 itself but extends what gets verified there post-Wave-3-merge.

**Step 2 (parallel — IP-X6 finishes + IP-X7 starts):** After IP-X6 catalog prelude `grit done`, IP-X6 continues with `crates/oya-ops-database-*` + sample-viewer adapter + 3 fitness lane implementations + `migrations/ops/V004__ops_database_audit.sql`. IP-X7 starts in parallel: owns `crates/oya-ops-schema-*` only. Both run under wave M03.W5.E + M03.W5.F.

**Step 3 (Wave 3 aggregate gate):** After IP-X6 + IP-X7 both `grit done`, aggregate gate verifies `/workspace/database`, `/workspace/database/sample-data`, `/workspace/schema` + both tile-health endpoints all return 200 for internal-foundry principal; `SURFACE_CATALOG::Database` + `SURFACE_CATALOG::Schema` both `status: Live`; 3 new fitness lanes green; M02-P22 BLOCKER-list amendment landed. Then parent §8 #4 marked done.

**Symbol disjoint verification:**
- IP-X6 catalog prelude: `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG::{Database,Schema}` (NEW entries — disjoint from Wave 1 + Wave 2's existing Catalog::{Docs,Overview,Dashboards,TechStack,Architecture}), `docs/standards/workspace-surfaces.md` (2 NEW rows; disjoint from existing), `contracts/ops-workspace-shell.openapi.yaml` (additive route entries; per `feedback_no_silent_regression.md` semver bump rule), `registry/quality/lanes.yaml::lean-a-{data-leak-sample-viewer,sample-viewer-rls,migration-drift}` (3 NEW lane rows), `.github/workflows/ci-fitness-lanes.yml::lean-a-{data-leak-sample-viewer,sample-viewer-rls,migration-drift}` (3 NEW lane jobs), `.omc/plans/milestones/M02-substrate/phases/P22-m02-exit-gate/impl-plan.md::lane-flip-table` (extension — coordinate with M02-P22 owner).
- IP-X6 main: `crates/oya-ops-database-*` + `migrations/ops/V004__ops_database_audit.sql` + `database-config.yaml` schema definition.
- IP-X7: `crates/oya-ops-schema-*` only.

No overlap with Wave 1, Wave 2, or each other.

### §6(d) Cedar inventory (no new fragments at Wave 3)

The 4 minimum M02-P20 fragments + Wave 2 `ops-system-only.cedar` policy_set extension cover Wave 3 entirely:
- `/workspace/database` + `/workspace/schema` + tile-health endpoints → `ops-internal-public.cedar`
- `/workspace/database/sample-data` → `ops-internal-private.cedar`

Wave 5 still owns the 11-fragment expansion (tenant-tier overlays). Wave 3 introduces ZERO new Cedar fragments.

### §6(e) Wave 3 ↔ ops.workspace integration

Both BCs register as `WorkspaceSurface` entries — IP-X6 registers `database`, IP-X7 registers `schema`. Per-IP smoke-test flip from `ReservedComingSoon` → `Live`. Aggregate Wave 3 gate verifies both routes 200 + both Live before declaring Wave 3 complete.

### §6(f) Phase-spec / impl-plan authoring rules

Each IP authors `impl-plans/IP-X<N>-ops-<bc>-bc.md` with ADR-0063 §4 required sections + WorkflowBridgePort tile-health integration + RLS-enforcement test fixtures (database) + mermaid client-side render fallback (schema).

---

## §7 Risk register

| ID | Risk | Mitigation |
|----|------|-----------|
| R1 | Sample-data viewer cross-tenant leak (Pre-mortem §1) | `tenant_id` required-param in trait + `oya-check-shardability` + `oya-check-data-leak-sample-viewer` (NEW lean lane, BLOCKER day 1) + **`oya-check-sample-viewer-rls`** (NEW lean lane, BLOCKER day 1 — architect r1 fix 1: validates `sample_viewer_ro` role attrs + FORCE RLS state on every allowlisted relation; per architect r2 fix 1) + audit-chain per query + read-only DB role with allowlist + per-cell mTLS read-replica binding (Bominal ADR-0044). |
| R2 | Schema mermaid render p99 violation (Pre-mortem §2) | Client-side render for >50 tables; per-µservice cache via `SharedManifestReadPort`. |
| R3 | Sample-viewer attack surface (Pre-mortem §3) | Rate limit 30/principal/hour + per-query incident-ID justification + 100-row cap + read-only DB role. |
| R4 | Schema drift between manifest + production cells (Pre-mortem §4) | `oya-check-migration-drift` lane (NEW; **report-only at IP-X6 acceptance → BLOCKER at the next exit-gate-like cycle** — architect r4 fix 1; aligns with §4/§5/§6/§8 wording) + per-cell version surfaced in health endpoint. |
| R5 | Wave 3 IPs slip M03 timeline | Sub-stream parallel: 2 IPs concurrent under M03.W5.E + M03.W5.F; symbol-disjoint; smaller scope (13 crates) than Wave 2. |
| R6 | `DatabaseSampleViewerPort` introduces new cross-cell network surface | Per-cell read-replica connection (no writes ever); mTLS via Istio per Bominal ADR-0044; circuit breaker on slow cells. |

---

## §8 ADR record (v3; per ralplan step 6 contract; architect r2 fix 2 — stale-text refresh)

- **Decision**: Adopt **Option α** — 2 BC surfaces (database internal-public + sample-data internal-private; schema internal-public), **13 BC crates + 3 new fitness lanes = 16 crates net new** at Wave 3 (architect r1 fix 2 + architect r2 fix 2 — added `oya-check-sample-viewer-rls` 3rd lane), all `oya-ops-<bc>-*` named, IP-X6 catalog-prelude serial then IP-X6 main + IP-X7 parallel sub-stream M03.W5.E/F under M03-P08 (architect r2 fix 3 — IP-X7 depends on IP-X6 catalog prelude completion, NOT fully parallel from dispatch). No new Cedar fragments.
- **Drivers**: SRE day-2 debugging utility + schema visibility + small-scope/fast-Wave.
- **Alternatives considered**:
  - Option β: Sample-data viewer as standalone µservice (separate domain) — REJECTED; violates single-domain `ops.oyatie.com` per parent §6(a).
  - Option γ: Defer sample-data viewer to Wave 6 (capacity/finops) — REJECTED; SRE day-2 debugging is current pain point; can't wait for Wave 6.
  - Option δ: Sample-data viewer as read-only CSV export (no in-browser view) — REJECTED; defeats the UX purpose; export feature can be added later as IP-Y under same BC.
- **Why chosen**: Maximum SRE utility + tight scope; reuses Wave 2 SharedManifestReadPort + WorkflowBridgePort infrastructure.
- **Consequences**:
  - Positive: 2 surface chips light up in workspace shell; SRE has unified schema + database debugging surface; audit-chain inheritance gives forensic trail for sample-viewer queries.
  - Negative: Sample-viewer is a sensitive surface — **5 layered controls required** (Cedar + RLS + rate limit + 100-row cap + grant/RLS validator per architect r1 fix 1 + r2 fix 1; sample_viewer_ro Postgres role hardened with no BYPASSRLS / no pg_read_all_data / table allowlist defaults to empty / FORCE RLS + tenant_isolation policy on every allowlisted relation). 2 NEW BLOCKER-day-1 lean lanes required: `lean-a-data-leak-sample-viewer` + `lean-a-sample-viewer-rls`. M02-P22 BLOCKER-list extension required (cross-Wave amendment).
  - Neutral: Oyatie ADR-0003 audit-chain + Bominal ADR-0028 segment inheritance compose cleanly (architect r1 fix 5 — ADR citation qualified: ADR-0028 in this repo is Cloud-µservice; Bominal ADR-0028 is the audit-chain segment authority); ADR-0045 database tier strategy unchanged; Bominal ADR-0044 mTLS for cross-cell sample-viewer connections.
- **Follow-ups**:
  1. After Wave 3 reaches Accepted: dispatch **Wave 4 ralplan** (observability + health BCs; carries SSR/SSE/10k perf gate) — parent §8 #5.
  2. Update `docs/MASTERPLAN.md` §2.1 ops block (critic r1 fix 4 + architect r4 fix 2 — dispatchable spec aligned with L102): (a) append `database`, `schema` to `ops.bounded_contexts`; expected list grows to `["docs", "workspace", "overview", "dashboards", "tech-stack", "architecture", "database", "schema"]` after Wave 2 #2 catalog update lands; (b) update line 102 Wave 3 row to exactly: `Wave 3 (M03-P08 cross-axis-contracts IP-X6 + IP-X7): database, schema (2 BCs; per ralplan-ops-wave-3-2026-05-13.md v4; consolidated to M03-P08 in line with Wave 2; database BC has sample-viewer internal-private surface with 5 layered controls)` — exact-string match with the L102 update already landed at MASTERPLAN.md by IP-X6 catalog prelude.
  3. Amend `.omc/plans/M01-M03-parallelization-manifest.md` §12 with 2 Wave 3 IPs (IP-X6 catalog-prelude serial + IP-X6-main + IP-X7 parallel sub-stream under M03.W5.E/F; symbol-disjoint after IP-X6 catalog prelude `grit done`; total 2 IPs).
  4. Update `docs/standards/workspace-surfaces.md`: register 2 new rows with per-IP smoke-test gating.
  5. Author **3 new fitness lane registry rows** in `registry/quality/lanes.yaml` (architect r2 fix 2 — was 2): `lean-a-data-leak-sample-viewer` (BLOCKER day 1) + `lean-a-sample-viewer-rls` (BLOCKER day 1; NEW per architect r1 fix 1) + `lean-a-migration-drift` (report-only initially → BLOCKER at next M02-exit-gate-like cycle).
  6. Audit-chain schema bump (architect r1 fix 5 — ADR citation qualified): add `ops_database_sample_viewer_queries` table per Oyatie ADR-0003 + Bominal ADR-0028 inheritance (tenant_id RLS + distribution column per `oya-check-shardability`; FORCE RLS + `tenant_isolation` policy required at IP-X6 acceptance gate per architect r1 fix 1 prevention spec).
  7. Workspace shell OpenAPI contract semver bump for 5 new route entries (`/workspace/database`, `/workspace/database/sample-data`, `/workspace/database/api/v1/health`, `/workspace/schema`, `/workspace/schema/api/v1/health`).
  8. **M02-P22 BLOCKER-list amendment** (architect r2 fix 2 + r2 fix M02-P22 cross-Wave note): Wave 3 explicitly authors an addition to `.omc/plans/milestones/M02-substrate/phases/P22-m02-exit-gate/impl-plan.md` lane-flip table + acceptance-gates list + sibling-team onboarding section to include `lean-a-data-leak-sample-viewer` + `lean-a-sample-viewer-rls` + `lean-a-migration-drift` (matching the Wave 1 docs §8 #4 P22 amendment pattern). This is part of IP-X6 catalog-prelude scope per §6(c) Step 1.

---

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | **ITERATE** (gpt-5.5 xhigh; codex `bhe3hg6m9`; 5 required fixes: (1) §4 Scenario 1 missing `sample_viewer_ro` grant/RLS failure mode + prevention; (2) lane canonicalization + M02-P22 BLOCKER-list/registry wiring for 2 (now 3) new lanes; (3) crate-count drift §3 (8+5) vs §6 (7+6); (4) SURFACE_CATALOG::{Database,Schema} + workspace-surfaces.md + shell OpenAPI shared-claim ownership; (5) audit-chain ADR citation correction (ADR-0028 is Cloud here; audit-chain = Oyatie ADR-0003 + Bominal ADR-0028)) | _pending dispatch (after architect r2 re-review on v2)_ | v1 → v2 (closes all 5 architect r1 fixes: sample_viewer_ro hardening + `oya-ops-database-check-sample-viewer-rls` NEW lane + 3-lane M02-P22 wiring + crate-count internally consistent 13 BC + 3 lanes = 16 net + IP-X6 catalog-prelude serial owner of `SURFACE_CATALOG::{Database,Schema}` + ADR citations qualified) |
| 2 | **ITERATE** (gpt-5.5 xhigh; codex `bubyfppq1`; all 5 r1 fixes PASS; 3 cleanup residuals: (1) `lean-a-sample-viewer-rls` missing from §5 + §7 R1; (2) §8 stale "15 crates / 4 controls"; (3) §6(b) "both parallel" contradicts §6(c) IP-X6 prelude-first) | _pending_ | v2 → v3 (closes 3 r2 cleanup fixes: §5 row added + §7 R1 expanded + §8 refreshed to "16 crates / 5 controls / 3 lanes" + new §8 #8 M02-P22 amendment follow-up + §6(b) IP-X7 predecessor = IP-X6 catalog-prelude) |
| 3 | ✅ **APPROVE** (gpt-5.5 xhigh; codex `b8k1sndai`; all 3 r2 cleanup fixes PASS; no new residuals; "5 controls count internally coherent: Cedar + RLS + rate limit + 100-row cap + grant/RLS validator"; structural design intact; "Next: critic dispatch.") | **ITERATE** r1 (gpt-5.5 xhigh; codex `br8q00irb`; 9-criterion scoring: 1/2/3/6/8 PASS, 4/7/9 WEAK, 5 FAIL; 4 required fixes: M02-P22 amendment 3rd lane + migration-drift timing + sample-data perf gate + MASTERPLAN dispatch wording) | v3 → v4 (closes 4 critic r1 fixes; M02-P22 amendment text now lists 3 lanes; §4 + §5 migration-drift timing aligned to IP-X6 acceptance + next exit-gate-like cycle; §5 perf row covers `/workspace/database/sample-data` + tile-health endpoints; §8 #2 + MASTERPLAN.md L102 updated to M03-P08 IP-X6/X7) |
| consensus loop iteration 2 (architect re-review post critic-r1 on v4) | **ITERATE** r4 (gpt-5.5 xhigh; codex `bgf75y10e`; 2 cleanup fixes — critic r1 Fixes 1/3 PASS, Fix 2 PASS at §4/§5 sites but §7 R4 stale "P21→P22", Fix 4 PARTIAL — §8 #2 target string omits "sample-viewer + 5 controls" clause from MASTERPLAN L102) → ✅ **APPROVE** r5 (gpt-5.5 xhigh; codex `b0qlsyymh`; both r4 cleanup fixes PASS — §7 R4 timing aligned + §8 #2 exact-string compare with MASTERPLAN.md L102 = MATCH; no-new-issues PASS; no-substance-change PASS; "Next: critic r2 dispatch.") | ✅ **APPROVE** r2 (gpt-5.5 xhigh; codex `b8qzjzqh5`; **9/9 PASS** — all 4 previously WEAK/FAIL criteria promoted: 4 testable-acceptance PASS (perf row gates `/workspace/database/sample-data` + tile-health endpoints), 5 verification PASS (M02-P22 amendment 3-lane + §8 #2 MASTERPLAN exact-match), 7 expanded-test PASS (migration-drift timing consistent across §4/§5/§6/§7/§8), 9 user-mandated rules PASS (perf gate + Workflow adapter + no-silent-regression); critical findings: None; required fixes: None; "Wave 3 can flip from pending approval to Accepted; dispatch the eight §8 follow-ups, mark parent §8 #4 done, and unblock Wave 4 ralplan.") | v4 → v5 → status **Accepted**; 8 §8 follow-up dispatch begins next step; Wave 4 ralplan unblocked |

---

## §10 Iteration cap

Loop up to 5 iterations per ralplan-DR step 5. This is iteration 1. Headroom: 4 more iterations before cap.
