---
doc_class: ReferenceImplementation
microservice: tenancy
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Tenant lifecycle + parent-child permit via the tenancy Rust SDK

A runnable example that:

1. Creates a parent tenant.
2. Transitions through the lifecycle states.
3. Creates a child tenant.
4. Builds a parent-child relationship.
5. Grants a scoped permit.
6. Demonstrates Cedar enforcement.
7. Triggers DSR cascade.
8. Verifies audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "tenancy-lifecycle-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tenancy-client = { path = "../../../../crates/tenancy-client" }
audit-chain-client = { path = "../../../../crates/audit-chain-client" }
cedar-client = { path = "../../../../crates/cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use tenancy_client::{
    TenancyClient, TenancyClientConfig,
    TenantCreate, TenantKind, TenantTransitionCommand,
    RelationshipCreate, RelationshipType,
    PermitCreate, ActionNamespace, ResourceScope,
    LifecycleLockCreate, LifecycleLockType,
};
use cedar_client::CedarPrincipal;
use chrono::Utc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Construct the client.
    let principal = CedarPrincipal::from_env("TENANCY_ADMIN_JWT")?;
    let client = TenancyClient::connect(TenancyClientConfig {
        cell_endpoint: std::env::var("TENANCY_ENDPOINT")?,
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Create parent tenant.
    let parent = client.tenant_create(TenantCreate {
        tenant_id: "acme-holdings".into(),
        kind: TenantKind::RegulatedB2b,
        display_name: "Acme Holdings".into(),
        requesting_principal: "u-ceo@acme-holdings.com".into(),
        requested_pack_set: vec!["default".into(), "sox".into(), "gdpr".into()],
    }).await?;
    info!("Parent tenant created: {} (state={})", parent.tenant_id, parent.state);

    // 3. Traverse lifecycle to active (using fast-track for the example; production uses real KYB).
    client.tenant_transition(TenantTransitionCommand {
        tenant_id: parent.tenant_id.clone(),
        from_state: "requested".into(),
        to_state: "kyb_pending".into(),
        requesting_principal: "u-ceo@acme-holdings.com".into(),
        reason: "Submitted for KYB review".into(),
        evidence: Some("kyb-provider-receipt:persona:ref-12345".into()),
        command_id: format!("cmd-{}", Utc::now().timestamp_millis()),
    }).await?;

    client.tenant_transition(TenantTransitionCommand {
        tenant_id: parent.tenant_id.clone(),
        from_state: "kyb_pending".into(),
        to_state: "provisioning".into(),
        requesting_principal: "u-ceo@acme-holdings.com".into(),
        reason: "KYB cleared".into(),
        evidence: Some("kyb-evidence-file:./kyb-acme.json".into()),
        command_id: format!("cmd-{}", Utc::now().timestamp_millis()),
    }).await?;

    client.tenant_transition(TenantTransitionCommand {
        tenant_id: parent.tenant_id.clone(),
        from_state: "provisioning".into(),
        to_state: "active".into(),
        requesting_principal: "u-ceo@acme-holdings.com".into(),
        reason: "Provisioning complete; all µservices ready".into(),
        evidence: None,
        command_id: format!("cmd-{}", Utc::now().timestamp_millis()),
    }).await?;
    info!("Parent tenant now active");

    // 4. Create child tenant (similar lifecycle; abbreviated).
    let child = client.tenant_create(TenantCreate {
        tenant_id: "acme-pharma".into(),
        kind: TenantKind::HealthcareProvider,
        display_name: "Acme Pharma".into(),
        requesting_principal: "u-pharma-admin@acme-pharma.com".into(),
        requested_pack_set: vec!["default".into(), "sox".into(), "gdpr".into(), "hipaa".into()],
    }).await?;
    // ... (fast-track to active)
    client.tenant_fast_track_to_active(&child.tenant_id).await?;
    info!("Child tenant created + active");

    // 5. Create parent-child relationship.
    let relationship = client.relationship_create(RelationshipCreate {
        parent_tenant_id: parent.tenant_id.clone(),
        child_tenant_id: child.tenant_id.clone(),
        relationship_type: RelationshipType::Owns,
        starts_at: Utc::now().to_rfc3339(),
        ends_at: "2099-12-31T23:59:59Z".into(),
        pack_scope: vec!["default".into(), "sox".into(), "gdpr".into()],   // NOT hipaa
        requesting_principal: "u-board-chair@acme-holdings.com".into(),
    }).await?;
    info!("Relationship created: id={}, type={:?}",
          relationship.relationship_id, relationship.relationship_type);

    // 6. Grant scoped permit: parent can read billing summaries but NOT data.
    let permit = client.permit_create(PermitCreate {
        relationship_id: relationship.relationship_id.clone(),
        action_namespace: ActionNamespace::from_actions(vec![
            "cloud-billing::summary::read",
            "cloud-billing::invoice::list",
        ]),
        resource_scope: ResourceScope::TenantScope(child.tenant_id.clone()),
        purpose: "consolidated_financial_reporting".into(),
        expires_at: "2027-01-01T00:00:00Z".into(),
        approved_by: "u-cfo@acme-holdings.com".into(),
    }).await?;
    info!("Permit granted: id={}, audit_event_id={}",
          permit.grant_id, permit.audit_event_id);

    // 7. Test the permit (would be called by cloud-billing µservice via Cedar).
    let cedar_decision = client.permit_evaluate(
        &permit.grant_id,
        "cloud-billing::summary::read",
        &child.tenant_id,
    ).await?;
    info!("Cedar decision for billing read: {} (permit_id={})",
          cedar_decision.decision, cedar_decision.permit_id_used);

    // 8. Try an action NOT in the permit (should deny).
    let denied = client.permit_evaluate(
        &permit.grant_id,
        "drive::file::decrypt",  // NOT granted
        &child.tenant_id,
    ).await?;
    info!("Cedar decision for drive::file::decrypt: {} (denied as expected)",
          denied.decision);

    // 9. Initiate child offboarding (DSR cascade).
    let offboarding = client.tenant_transition(TenantTransitionCommand {
        tenant_id: child.tenant_id.clone(),
        from_state: "active".into(),
        to_state: "offboarding".into(),
        requesting_principal: "u-pharma-admin@acme-pharma.com".into(),
        reason: "Tenant requested termination".into(),
        evidence: Some("termination-letter-2026-05-20.pdf".into()),
        command_id: format!("cmd-{}", Utc::now().timestamp_millis()),
    }).await?;
    info!("Offboarding initiated; DSR cascade started");

    // 10. Wait for downstream µservice acks (in production, this is event-driven).
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    let cascade_status = client.offboarding_cascade_status(&child.tenant_id).await?;
    info!("Offboarding cascade: {} services acknowledged",
          cascade_status.acknowledged_services.len());

    Ok(())
}
```

## Expected output (against a paid tenant_class baseline cell)

```
INFO Parent tenant created: acme-holdings (state=requested)
INFO Parent tenant now active
INFO Child tenant created + active
INFO Relationship created: id=r_acme_holdings_pharma, type=Owns
INFO Permit granted: id=pg_billing_001, audit_event_id=ae_ten_permit_granted_001
INFO Cedar decision for billing read: permit (permit_id=pg_billing_001)
INFO Cedar decision for drive::file::decrypt: deny (denied as expected)
INFO Offboarding initiated; DSR cascade started
INFO Offboarding cascade: 5 services acknowledged
```

## HTTP alternative (curl)

```sh
# 1. Create tenant
curl -X POST https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/tenants \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "tenant_id":"acme-holdings",
        "kind":"regulated_b2b",
        "display_name":"Acme Holdings",
        "requesting_principal":"u-ceo@acme-holdings.com",
        "requested_pack_set":["default","sox","gdpr"]
    }'

# 2. Lifecycle transition
curl -X POST https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/tenants/acme-holdings/transitions \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "from_state":"requested",
        "to_state":"kyb_pending",
        "requesting_principal":"u-ceo@acme-holdings.com",
        "reason":"Submitted for KYB review",
        "evidence":"kyb-provider-receipt:persona:ref-12345",
        "command_id":"cmd-001"
    }'

# 3. Create relationship
curl -X POST https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/relationships \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "parent_tenant_id":"acme-holdings",
        "child_tenant_id":"acme-pharma",
        "relationship_type":"owns",
        "starts_at":"2026-05-20T00:00:00Z",
        "ends_at":"2099-12-31T23:59:59Z",
        "pack_scope":["default","sox","gdpr"]
    }'

# 4. Create permit
curl -X POST https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/relationships/r_acme_holdings_pharma/permits \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "action_namespace":["cloud-billing::summary::read","cloud-billing::invoice::list"],
        "resource_scope":"tenant=acme-pharma",
        "purpose":"consolidated_financial_reporting",
        "expires_at":"2027-01-01T00:00:00Z",
        "approved_by":"u-cfo@acme-holdings.com"
    }'

# 5. Lifecycle lock
curl -X POST https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/tenants/acme-holdings/locks \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "lock_type":"incident_freeze",
        "reason":"Security investigation",
        "created_by":"u-incident-commander@acme.com",
        "expires_at":"2026-05-27T00:00:00Z"
    }'

# 6. Read lifecycle state
curl -X GET https://tenancy.prod-us-east-1.oyatie.local/v1/tenancy/tenants/acme-holdings/lifecycle \
    -H "Authorization: Bearer $TENANCY_ADMIN_JWT"
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `tenancy::*` permission |
| `lifecycle_state_invalid` | 409 | No | Wrong from_state or guard failed |
| `lifecycle_lock_active` | 423 | No | Locked; cannot transition until lock released |
| `relationship_cycle_detected` | 422 | No | Hierarchy would create cycle |
| `relationship_depth_exceeded` | 422 | No | Conglomerate depth exceeds tenant_class depth limit (5/10) |
| `permit_pack_scope_invalid` | 422 | No | Permit pack scope exceeds child pack restrictions |
| `sovereign_child_veto` | 403 | No | Child pack denies parent override |
| `idempotent_command_replay` | 200 | N/A | Same command_id; returning cached result |
| `kyb_required` | 422 | No | Cannot transition to provisioning without KYB evidence |
| `legal_hold_active` | 423 | No | Cannot offboard while legal hold exists |
| `cryptoshred_blocked_by_retention` | 423 | No | Retention window not expired |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `tenant_create` | `tenancy.tenant.created.v1` |
| `tenant_transition` | `tenancy.lifecycle.transitioned.v1` |
| `relationship_create` | `tenancy.relationship.created.v1` |
| `permit_create` | `tenancy.permit.granted.v1` |
| `permit_revoke` | `tenancy.permit.revoked.v1` |
| `lifecycle_lock_create` | `tenancy.lifecycle.lock.created.v1` |
| `lifecycle_lock_release` | `tenancy.lifecycle.lock.released.v1` |
| `tenant_offboarding` | `tenancy.offboarding.cascade.requested.v1` |
| `divestiture_initiate` | `tenancy.divestiture.initiated.v1` |
| `tenant_cryptoshred` | `tenancy.cryptoshred.scheduled.v1` |
| Cedar deny anywhere | `tenancy.cedar.denied.v1` |

## Where this file lives

`microservices/tenancy/reference-implementations/lifecycle-and-permit-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/tenancy/reference-implementations/lifecycle-example/` once `tenancy-client` ships.
