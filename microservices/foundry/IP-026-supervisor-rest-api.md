---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-011-rest-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-check-openapi-conformance, oya-check-cedar-fragment-coverage]
depends_on: [IP-005, IP-006, IP-008, IP-010]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: REST surface (axum-based; OIDC + Cedar)

## Intent

REST handler + route layer per `contracts/openapi/foundry-supervisor.yaml`. Bound to `-api` typed contracts. OIDC bearer auth + Cedar policy gate on every route. axum-based.

Implements `-rest` crate for each BC: agent-fleet-lifecycle, capability-deployment, autonomy-policy-enforcement, kill-switch-circuit-breaker (supervision-event-bus has no REST; bus-only).

## Concrete File Targets

`microservices/foundry/src/crates/oya-foundry-supervisor-{agent-fleet-lifecycle,capability-deployment,autonomy-policy-enforcement,kill-switch-circuit-breaker}-rest/`.

## Key code

```rust
// rest/src/handlers/engage_kill_switch.rs
pub async fn engage_kill_switch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<EngageRequest>,
) -> Result<Json<KillSwitch>, RestError> {
    // 1. OIDC bearer extraction + verification
    let principal = state.oidc.verify(&headers)?;

    // 2. X-Scope-OrgID match
    let scope_id = headers.get("X-Scope-OrgID")
        .ok_or(RestError::MissingTenant)?
        .to_str()?;
    if principal.tenant_id != scope_id {
        return Err(RestError::TenantMismatch);
    }

    // 3. X-Signature-Bundle extraction (comma-separated)
    let signatures = parse_signatures(&headers)?;
    if req.scope == KillSwitchScope::Fleet && signatures.len() < 2 {
        return Err(RestError::TwoPersonRuleRequired);
    }

    // 4. Cedar policy evaluation
    let decision = state.cedar.evaluate(
        &principal,
        &Action::EngageKillSwitch,
        &req.as_resource(),
    ).await?;
    if !decision.permit {
        return Err(RestError::Forbidden { reason: decision.reason });
    }

    // 5. Invoke usecase
    let engaged = state.engage_usecase
        .engage(req.scope, req.target, req.reason, signatures)
        .await?;

    Ok(Json(engaged))
}
```

## Acceptance Gates

```bash
cargo check / build / clippy / nextest per crate
cargo run -p oya-dev-cli -- gate validate openapi-conformance --microservice foundry-supervisor
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-supervisor
```

## Test Plan

Per route: happy + auth-fail (no OIDC) + tenant-mismatch + cedar-deny + insufficient-signatures (kill-switch-fleet only). Plus cross-route flow (admit → list → rollback) e2e.

## Halt Conditions

- Route bypasses Cedar evaluation.
- Two-person rule not enforced at REST layer.
- OpenAPI conformance violation.

## Next IP

[`IP-012-supervisor-self-slos.md`](IP-012-supervisor-self-slos.md)

## References

- PRD §"Bounded Contexts" + `contracts/openapi/foundry-supervisor.yaml`.
- ADR-0140 (retired per ADR-0145) (Cedar).
- `policy/tenant-scope.cedar` + `policy/auditor-scope.cedar` + `policy/public-read.cedar`.
- axum — `docs.rs/axum`.
