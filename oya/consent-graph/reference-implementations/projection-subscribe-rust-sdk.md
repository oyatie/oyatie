---
doc_class: ReferenceImplementation
microservice: consent-graph
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Projection subscription via the consent-graph Rust SDK

A runnable example showing how a grantee tenant subscribes to a DataSharingAgreement's projection topic, processes incoming rows, and gracefully handles revocation — using `oya-consent-graph-client` (target API; once IP-010 + IP-011 land).

## Cargo.toml

```toml
[package]
name = "consent-graph-grantee-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-consent-graph-client = { path = "../../crates/oya-consent-graph-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::{DateTime, Utc};
use oya_cedar_client::CedarPrincipal;
use oya_consent_graph_client::{
    AgreementId, ConsentGraphClient, ConsentGraphClientConfig, ProjectionMessage,
    ProjectionSubscription,
};
use serde::Deserialize;
use tracing::{info, warn};

/// Row shape per the agreement's scope spec — fields explicitly allowed.
#[derive(Debug, Deserialize)]
struct OrderProjection {
    order_id: String,
    total_amount_cents: i64,
    currency: String,
    shipping_country_iso: String,
    created_at: DateTime<Utc>,
    status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Grantee-side principal.
    let principal = CedarPrincipal::from_env("GRANTEE_PRINCIPAL_JWT")?;
    let config = ConsentGraphClientConfig {
        cell_endpoint: std::env::var("CONSENT_GRAPH_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?, // grantee tenant
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = ConsentGraphClient::connect(config).await?;

    // The agreement we're subscribing to (granted to our tenant).
    let agreement_id = AgreementId::new("ag-draft-2026-05-20");

    // Verify the agreement is in `active` state before subscribing.
    let agreement = client.agreement_get(&agreement_id).await?;
    info!(
        agreement_id = %agreement.id,
        state = ?agreement.state,
        sharing_mode = ?agreement.sharing_modes,
        grantor = %agreement.grantor,
        grantee = %agreement.grantee,
        scope = ?agreement.scope_spec,
        "agreement details"
    );

    if !matches!(
        agreement.state,
        oya_consent_graph_client::AgreementState::Active
    ) {
        anyhow::bail!("agreement is not active; state={:?}", agreement.state);
    }

    // Subscribe to the projection topic. The SDK wraps the underlying Pulsar
    // subscription + applies Cedar enforcement at each row + handles revocation
    // gracefully.
    let mut subscription: ProjectionSubscription = client
        .projection_subscribe(&agreement_id)
        .await?;

    info!(
        topic = %subscription.topic(),
        "subscribed to projection topic"
    );

    // Process incoming projection messages.
    while let Some(message_result) = subscription.next().await {
        match message_result {
            Ok(ProjectionMessage::Row { fields, projected_at, .. }) => {
                // Deserialize per the agreement's scope spec.
                let order: OrderProjection = serde_json::from_value(fields)?;
                info!(
                    order_id = %order.order_id,
                    total_cents = order.total_amount_cents,
                    country = %order.shipping_country_iso,
                    projected_at = %projected_at,
                    "received projection row"
                );

                // Application-specific processing: store in grantee's local
                // database, update local dashboard, etc.
                // The agreement's retention spec applies — we must delete or
                // anonymize after 90 days.
            }
            Ok(ProjectionMessage::Revoked { revoked_at, reason, by }) => {
                warn!(
                    revoked_at = %revoked_at,
                    reason = %reason,
                    revoked_by = %by,
                    "agreement revoked; subscription will terminate"
                );

                // Application-specific: trigger local-retention cleanup per
                // the agreement's retention_at_grantee spec.
                client.local_post_revocation_cleanup(&agreement_id).await?;

                // Emit `grantee_post_revocation_deletion_attested` to the
                // bilateral chain.
                client
                    .attest_post_revocation_deletion(
                        &agreement_id,
                        oya_consent_graph_client::DeletionAttestation {
                            deletion_completed_at: Utc::now(),
                            deleted_row_count: 12_847, // application provides
                            deletion_method: "logical-delete + cold-tier-purge"
                                .into(),
                        },
                    )
                    .await?;
                break;
            }
            Ok(ProjectionMessage::AgreementPaused { paused_at, reason }) => {
                info!(
                    paused_at = %paused_at,
                    reason = %reason,
                    "agreement paused; subscription is suspended"
                );
                // The subscription will resume automatically when the agreement
                // resumes; the SDK handles the reconnect.
            }
            Err(e) => {
                warn!(error = ?e, "projection subscription error");
                // Retryable errors are handled transparently by the SDK; here
                // we'd see only fatal errors.
                break;
            }
        }
    }

    Ok(())
}
```

## Expected log output

```
INFO agreement details agreement_id=ag-draft-2026-05-20 state=Active sharing_mode=[Projection] grantor=drill-acme grantee=drill-partner-co
INFO subscribed to projection topic topic=persistent://drill-partner-co/consent-graph/ag-draft-2026-05-20-projections
INFO received projection row order_id=ord-12345 total_cents=49900 country=US projected_at=2026-05-20T13:42:00Z
INFO received projection row order_id=ord-12346 total_cents=89000 country=UK projected_at=2026-05-20T13:42:30Z
...
WARN agreement revoked; subscription will terminate revoked_at=2026-05-20T14:30:00Z reason="Business relationship ended" revoked_by=drill-acme
```

## Audit chain emission

```sh
# Grantee side
oya audit query --tenant drill-partner-co --since 1h --agreement-id ag-draft-2026-05-20
# Grantor side
oya audit query --tenant drill-acme --since 1h --agreement-id ag-draft-2026-05-20
```

Expected events:

- `projection_subscribed` (grantee)
- `projection_row_emitted` × N (grantor) ↔ `projection_row_received` × N (grantee)
- `agreement_revoked` (both sides)
- `projection_subscription_terminated` (grantee)
- `grantee_post_revocation_deletion_attested` (grantee → chain → grantor)

The bilateral chain has matching events at the same agreement_id on both sides.

## Direct Pulsar consumer alternative

Without the SDK, a grantee can consume the Pulsar topic directly with mTLS + an agreement-bound JWT:

```sh
pulsar-client consume \
    persistent://drill-partner-co/consent-graph/ag-draft-2026-05-20-projections \
    --service-url pulsar+ssl://drill-partner-co.pulsar.drill-syd-1:6651 \
    --auth-plugin org.apache.pulsar.client.impl.auth.AuthenticationToken \
    --auth-params "token:$AGREEMENT_BOUND_JWT" \
    --subscription-name grantee-app-1 \
    --num-messages 100
```

The SDK adds: Cedar pre-flight, revocation handling, deletion-attestation emission, retention tracking.

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks `consent-graph::projection::subscribe`. Fix at IAM. |
| `agreement_not_active` | No | Agreement must be active. Check state. |
| `geographic_constraint_violation` | No | Subscribe request from a region not permitted by the agreement. |
| `pulsar_connection_failed` | Yes (auto) | SDK reconnects with exponential backoff. |
| `agreement_revoked` | No (specific shutdown) | Trigger local cleanup + emit deletion attestation; do not retry. |
| `cell_unavailable` | Yes (circuit-breaker) | Cell down; SDK fails after 3 retries; opens for 30 s. |

## Where this file lives

`microservices/consent-graph/reference-implementations/projection-subscribe-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/consent-graph/reference-implementations/grantee-example/` once IP-010 + IP-011 land.
