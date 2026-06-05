---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-007-tenant-context-usecase-rest
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, openapi-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: tenant-context usecase + domain + adapter + rest

## Intent

Combined IP: ship tenant-context BC's domain/usecase/adapter/rest layers
in one ChangeSet (smaller surface than shell-routing). Resolves
hostname + JWT claim into a `Principal` + `TenantContext`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-tenant-context-domain/{Cargo.toml,src/lib.rs}` | create — pure tenant-context validation |
| `microservices/application/src/crates/oya-application-tenant-context-usecase/{Cargo.toml,src/lib.rs}` | create — ResolveTenantContextUseCase |
| `microservices/application/src/crates/oya-application-tenant-context-adapter/{Cargo.toml,src/lib.rs}` | create — calls tenancy SDK + OpenBao for tenant→pack mapping |
| `microservices/application/src/crates/oya-application-tenant-context-api/{Cargo.toml,src/lib.rs}` | create — protocol-neutral types |
| `microservices/application/src/crates/oya-application-tenant-context-rest/{Cargo.toml,src/lib.rs}` | create — middleware crate (no public endpoints; injects into axum) |
| 5 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
pub struct ResolveTenantContextUseCase<R: TenantResolver, B: TenantBindingStore> {
    resolver: R, binding: B, openbao: OpenBaoClient,
}

impl<R, B> ResolveTenantContextUseCase<R, B> where R: TenantResolver, B: TenantBindingStore {
    pub async fn resolve(&self, host: &str, jwt_claim: &str) -> Result<Principal, UseCaseError> {
        let ctx = self.resolver.from_host(host).await?;
        self.resolver.verify_jwt_claim(jwt_claim, &ctx).await?;
        let binding = self.binding.fetch(&ctx.tenant_id).await?;
        Ok(Principal::from(binding, ctx))
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-tenant-context-usecase --all-features
cargo nextest run -p oya-application-tenant-context-adapter --all-features
cargo nextest run -p oya-application-tenant-context-rest --all-features
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice application
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_resolve_host_to_tenant` | hostname → tenant lookup |
| `test_jwt_claim_mismatch_rejected` | host vs JWT |
| `test_pack_residency_inheritance` | tenant pack propagates |
| `test_fallback_cache_when_openbao_down` | degraded mode |

## Next IP

[`IP-008-auth-gateway-kernel-domain.md`](IP-008-auth-gateway-kernel-domain.md)
