---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-006-isolation-policy-rls-generator
status: pending
owner: axis-tenancy + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, governance-rls-no-superuser-bypass, governance-rls-force-on-tenant-tables]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: Isolation-policy RLS generator + enforcement

## Intent

Build the `tenancy-isolation-policy-{kernel,domain,usecase,adapter-postgres}` crates: RLS YAML manifest → Postgres DDL emission; FORCE ROW LEVEL SECURITY enforcement; tenant-bound-table registry; LEAN-check integration.

## Concrete File Targets

| Path | Action |
|---|---|
| `tenancy-isolation-policy-kernel/` | create — port traits + RlsPolicy entity + TenantBoundTable registry |
| `tenancy-isolation-policy-domain/` | create — RlsPolicy → DDL string rendering (pure) |
| `tenancy-isolation-policy-usecase/` | create — install / verify / audit orchestrators |
| `tenancy-isolation-policy-adapter-postgres/` | create — emits DDL via sqlx; reads `pg_policies` |
| `microservices/tenancy/policy/rls/<table>.yaml` | create — declarative RLS manifests for tenancy-owned tables (tenants, dsr_requests, audit_log) |
| `microservices/tenancy/policy/rls/waivers.yaml` | create — empty initial waiver register |
| Catalog rows | create — 4 catalog entries |

## Code Shape

```rust
// isolation-policy-domain/src/rls_renderer.rs
pub fn render_create_policy(table: &str, policy_name: &str, predicate: &str) -> String {
    format!(
        "CREATE POLICY {policy_name} ON {table} USING ({predicate});\n\
         ALTER TABLE {table} ENABLE ROW LEVEL SECURITY;\n\
         ALTER TABLE {table} FORCE ROW LEVEL SECURITY;",
        policy_name=policy_name, table=table, predicate=predicate)
}
// Constant predicate to match policy/rls-isolation.md Invariant RLS-02:
pub const CANONICAL_PREDICATE: &str = "tenant_id = current_setting('app.current_tenant_id')::text";
```

```rust
// isolation-policy-usecase/src/install.rs
pub async fn install_rls(deps: &Deps, manifest: &RlsManifest, change_id: ChangeId) -> Result<()> {
    let ddl = render_create_policy(&manifest.table, &manifest.policy_name, &manifest.predicate);
    deps.pg.execute(&ddl).await?;
    // Post-install validate:
    let row = deps.pg.query_one("SELECT relforcerowsecurity FROM pg_class WHERE relname = $1", &manifest.table).await?;
    if !row.relforcerowsecurity {
        return Err(InstallError::ForceRlsNotApplied);
    }
    deps.audit_chain.seal(RlsPolicyInstalledEnvelope::from(manifest, change_id)).await?;
    deps.event_sink.emit(RlsPolicyInstalledEvent::from(manifest, change_id)).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p tenancy-isolation-policy-usecase
cargo nextest run -p tenancy-isolation-policy-adapter-postgres
cargo run -p dev-cli -- gate validate rls-no-superuser-bypass
cargo run -p dev-cli -- gate validate rls-force-on-tenant-tables
```

## Test Plan

- Synthetic cross-tenant probe test: tenant-A sets context; queries; verifies zero tenant-B rows.
- DDL parse correctness: every rendered DDL passes `psql -f` lint.
- Post-install validator returns true on success, false when migration skipped FORCE.

## Halt Conditions

- Any rendered DDL omits `FORCE ROW LEVEL SECURITY` — refuse.
- Any code path connects as `postgres` superuser — refuse.


## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/tenancy/IP-006-isolation-policy-rls-generator.md` matched `emission`; anchors `microservices/tenancy/manifest.json, crates/tenancy-api/src/lib.rs`; type anchor `crates/tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Next IP

[`IP-007-isolation-policy-jwt-issuer.md`](IP-007-isolation-policy-jwt-issuer.md)
