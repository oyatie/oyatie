---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-010-tenancy-rest-and-sdk
status: pending
owner: axis-tenancy
acceptance_lanes: [cargo-check, cargo-nextest, openapi-conformance, oya-governance-tenancy-cedar-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: Tenancy REST surface + Rust SDK

## Intent

Implement the REST API per `contracts/openapi/tenancy.yaml` across `oya-tenancy-{tenant-lifecycle,isolation-policy,dsr-cascade}-rest` crates; author the Rust SDK at `oya-tenancy-tenant-lifecycle-sdk` per `sdk-plan.md`. Cedar policy evaluation on every request.

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-tenancy-tenant-lifecycle-rest/src/{routes/*,middleware/cedar.rs}` | create |
| `oya-tenancy-isolation-policy-rest/src/{routes/*}` | create |
| `oya-tenancy-dsr-cascade-rest/src/{routes/*}` | create |
| `oya-tenancy-tenant-lifecycle-sdk/src/{client,types}.rs` | create |
| Catalog rows | create — 4 entries |

## Code Shape

```rust
// tenant-lifecycle-rest/src/middleware/cedar.rs
pub async fn cedar_authorize<B>(
    State(state): State<AppState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let principal = extract_principal(&req)?;
    let action = derive_action(&req);
    let resource = derive_resource(&req);
    let decision = state.cedar.is_authorized(&principal, &action, &resource).await?;
    if decision.decision() == Decision::Deny {
        record_metric!("oya_tenancy_cedar_deny_total", 1);
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}
```

```rust
// sdk/src/client.rs
pub struct TenancyClient {
    base_url: Url,
    oidc_provider: Box<dyn OidcTokenProvider>,
    pack: Pack,
    http: reqwest::Client,
}

impl TenancyClient {
    pub async fn create_tenant(&self, req: CreateTenantRequest) -> Result<Tenant, SdkError> {
        let token = self.oidc_provider.get_token().await?;
        let resp = self.http
            .post(self.base_url.join("/api/v1/tenants")?)
            .bearer_auth(token)
            .json(&req)
            .send().await?;
        // retry on 5xx + 429 with exponential backoff; circuit-breaker
        ...
    }
    // ... 12 more methods per sdk-plan.md
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-tenancy-tenant-lifecycle-rest
cargo nextest run -p oya-tenancy-tenant-lifecycle-sdk
cargo run -p oya-dev-cli -- gate validate openapi-conformance --microservice tenancy
cargo run -p oya-dev-cli -- gate validate tenancy-cedar-coverage
```

## Test Plan

- Per PHASE-01 rest class: 1 per route (happy + auth-fail + tenant-mismatch).
- SDK: 1 per public method (happy + retry + auth-fail).
- E2E: SDK client → REST → Postgres → response cycle.
- OpenAPI conformance via Schemathesis-style fuzzing.


## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-010-tenancy-rest-and-sdk.md` matched `openapi`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Next IP

[`IP-011-audit-chain-integration.md`](IP-011-audit-chain-integration.md)
