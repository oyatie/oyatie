---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-004-tenant-lifecycle-usecase
status: pending
owner: axis-tenancy
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: tenancy-tenant-lifecycle-usecase

## Intent

`-usecase` crate: orchestrators for `CreateTenant`, `ActivateTenant`, `SuspendTenant`, `ResumeTenant`, `RequestTenantDeletion`. Reads via ports; writes via ports; emits Workflow events.

## Concrete File Targets

| Path | Action |
|---|---|
| `tenancy-tenant-lifecycle-usecase/Cargo.toml` | create |
| `tenancy-tenant-lifecycle-usecase/src/{lib,create_tenant,activate_tenant,suspend_tenant,resume_tenant,request_deletion}.rs` | create (6 files) |
| catalog row | create |

## Code Shape

```rust
// src/create_tenant.rs
pub struct CreateTenantUseCase<TR, CA, ES> {
    tenant_repo: TR,
    cell_assigner: CA,
    event_sink: ES,
}
impl<TR: TenantRepository, CA: CellAssigner, ES: EventSink> CreateTenantUseCase<TR, CA, ES> {
    pub async fn execute(&self, req: CreateTenantRequest, actor: &Actor) -> Result<Tenant, UseCaseError> {
        // 1. Validate request + jurisdiction
        let jurisdiction = JurisdictionValidator::validate(&req.jurisdiction_code, &req.operator_attestation)?;
        // 2. Generate tenant_id (hashed)
        let tenant_id = TenantId::generate(/* canonical, salt from OpenBao */);
        // 3. Assign cell (least-loaded in jurisdiction)
        let cell_id = self.cell_assigner.assign(&jurisdiction).await?;
        // 4. Build Tenant + persist
        let tenant = Tenant { tenant_id, status: Created, jurisdiction_code: jurisdiction, plan_tier: req.plan_tier, cell_id, created_at: now(), activated_at: None };
        self.tenant_repo.create(&tenant).await?;
        // 5. Emit TenantCreated event
        self.event_sink.emit(TenantCreatedEvent::from(&tenant, actor)).await?;
        Ok(tenant)
    }
}
```

## Acceptance Gates

```bash
cargo check -p tenancy-tenant-lifecycle-usecase
cargo nextest run -p tenancy-tenant-lifecycle-usecase
```

## Test Plan

Per PHASE-01 usecase class: 1 per use case (happy + 2 sad) + ≥ 3 integration tests against mocked ports.

## Next IP

[`IP-005-tenant-lifecycle-adapter-postgres.md`](IP-005-tenant-lifecycle-adapter-postgres.md)
