---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-013-sdk-rust-and-ts
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, npm-build, npm-test]
depends_on: [IP-011]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: SDK (Rust first-party + TypeScript generated)

## Intent

Rust SDK crates per BC (M01) + TypeScript SDK published to internal npm registry (M01+1 launch; scaffold in M01). Per `sdk-plan.md`.

## Concrete File Targets

Rust crates:
- `microservices/foundry/src/crates/oya-foundry-supervisor-{bc}-sdk/`

TypeScript SDK (scaffold):
- `microservices/foundry/sdk-generation/typescript/{package.json, src/, openapi-generator-config.yaml}`

## Key code (Rust SDK)

```rust
// agent-fleet-lifecycle-sdk/src/lib.rs
pub struct Client {
    base_url: Url,
    oidc: Box<dyn OidcTokenProvider>,
    tenant_id: TenantId,
    http: reqwest::Client,
}

impl Client {
    pub async fn get_fleet_state(&self) -> Result<FleetState, ClientError> {
        let url = self.base_url.join(&format!("/api/v1/tenants/{}/fleet-state", self.tenant_id))?;
        let token = self.oidc.acquire().await?;
        let resp = self.http.get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Scope-OrgID", self.tenant_id.as_str())
            .send().await?;
        resp.json().await.map_err(Into::into)
    }
    // ... other methods
}
```

Two-person rule type-system enforcement (carries from IP-008 into SDK):

```rust
// kill-switch-circuit-breaker-sdk/src/lib.rs
impl Client {
    pub async fn engage_kill_switch_fleet_wide(
        &self,
        reason: EngageReason,
        signature_1: SignatureBundle,
        signature_2: SignatureBundle,  // compiler enforces both
    ) -> Result<KillSwitch, ClientError> { ... }

    pub async fn engage_kill_switch_scope_local<S: ScopeKindNotFleet>(
        &self,
        scope: S,
        target: TargetId,
        reason: EngageReason,
        signature: SignatureBundle,
    ) -> Result<KillSwitch, ClientError> { ... }
}
```

## Acceptance Gates

```bash
cargo check / build / clippy / nextest per Rust SDK crate
(cd microservices/foundry/sdk-generation/typescript && npm run build && npm test)
```

## Test Plan

Per `PHASE-01-CONTROL-PLANE-LANDING.md` §"Per-IP Test Coverage Threshold":
- Rust SDK: 1 per method + retry + auth-fail; ≥ 2 against staging rest; 90 % line.
- TS SDK: build + lint + 1 integration test per public method.

## Halt Conditions

- SDK allows single-signature fleet-wide engage to compile.

## Next IP

[`IP-014-app-composition-root.md`](IP-014-app-composition-root.md)

## References

- `sdk-plan.md`.
- `contracts/openapi/foundry-supervisor.yaml`.
- ADR-0105 §"sdk layer".
