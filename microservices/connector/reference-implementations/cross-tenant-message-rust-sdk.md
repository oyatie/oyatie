---
doc_class: ReferenceImplementation
microservice: connector
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Cross-tenant message + MLS encryption + Cedar disclosure via the connect Rust SDK

A runnable example that establishes a federation peer, joins an MLS group, sends + receives encrypted messages with disclosure-rule enforcement — using `oya-connector-client` (target API; once IP-005 + IP-007 + IP-008 + IP-009 land).

## Cargo.toml

```toml
[package]
name = "connect-federation-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-connector-client = { path = "../../crates/oya-connector-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_cedar_client::CedarPrincipal;
use oya_connector_client::{
    ChannelMessage, ConnectClient, ConnectClientConfig, CrossTenantEvent, DataClass,
    DisclosureBaseline, FederationPeerRequest, MessageMetadata, MessageSendRequest,
};
use serde_json::json;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("CONNECT_PRINCIPAL_JWT")?;
    let config = ConnectClientConfig {
        api_endpoint: std::env::var("CONNECT_API")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(15),
    };
    let client = ConnectClient::connect(config).await?;

    // 1. Initiate federation peer-request (as tenancy-admin).
    let peer_request = client
        .federation_peer_request(FederationPeerRequest {
            peer_tenant_id: "drill-beta-vendor".into(),
            intent: "ongoing-supplier-relationship".into(),
            proposed_channels: vec![
                "supplier-status".into(),
                "supplier-billing".into(),
            ],
            disclosure_baseline: DisclosureBaseline::ExplicitOptIn,
            duration_days: 365,
            justification: "Q3 2026 supplier engagement".into(),
        })
        .await?;
    info!(
        request_id = %peer_request.request_id,
        awaiting = %peer_request.awaiting_principal,
        "peer-request submitted"
    );

    // 2. (In real use, wait for peer admin signoff. For this example, assume already accepted.)
    let federation_status = client.federation_status("drill-beta-vendor").await?;
    if !federation_status.handshake_completed {
        warn!("peer-request not yet accepted; exiting");
        return Ok(());
    }
    info!(
        peer = "drill-beta-vendor",
        channels_bridged = federation_status.channels_bridged.len(),
        mls_epoch = federation_status.mls_epoch,
        "federation active"
    );

    // 3. Send a cross-tenant message with PUBLIC data-class (passes disclosure).
    let send_receipt = client
        .send_message(MessageSendRequest {
            channel: "supplier-status".into(),
            body: "Q3 SKU XYZ-123 shipment ETA confirmation needed".into(),
            metadata: MessageMetadata {
                data_class: DataClass::Public,
                thread_root: None,
                attachments: vec![],
            },
        })
        .await?;
    info!(
        message_id = %send_receipt.message_id,
        mls_encrypted = send_receipt.mls_encrypted,
        peer_delivered = send_receipt.peer_delivered,
        "message sent + delivered"
    );

    // 4. Try a PHI message (should be denied by disclosure rule).
    let phi_attempt = client
        .send_message(MessageSendRequest {
            channel: "supplier-status".into(),
            body: "Patient SSN 555-12-3456".into(),
            metadata: MessageMetadata {
                data_class: DataClass::Phi,
                thread_root: None,
                attachments: vec![],
            },
        })
        .await;

    match phi_attempt {
        Ok(_) => warn!("unexpected: PHI message went through; investigate"),
        Err(e) if e.is_disclosure_denied() => {
            info!(
                reason = %e.reason(),
                "PHI denied by disclosure rule — expected"
            );
        }
        Err(e) => warn!(error = ?e, "unexpected error"),
    }

    // 5. Subscribe to cross-tenant events.
    let mut event_stream = client.subscribe_cross_tenant_events().await?;
    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => match event {
                CrossTenantEvent::MessageReceived(ChannelMessage {
                    channel,
                    from_tenant,
                    from_principal,
                    body,
                    metadata,
                    occurred_at,
                }) => {
                    info!(
                        channel = %channel,
                        from_tenant = %from_tenant,
                        from = %from_principal,
                        data_class = ?metadata.data_class,
                        occurred_at = %occurred_at,
                        body_preview = body.chars().take(60).collect::<String>(),
                        "cross-tenant message received"
                    );
                }
                CrossTenantEvent::MemberAdded { channel, peer_tenant, member } => {
                    info!(channel = %channel, peer = %peer_tenant, member = %member, "peer member added");
                }
                CrossTenantEvent::MlsRekeyCompleted { channel, new_epoch } => {
                    info!(channel = %channel, epoch = new_epoch, "MLS rekey");
                }
                CrossTenantEvent::DisclosureRuleViolation { channel, principal, reason } => {
                    warn!(channel = %channel, principal = %principal, reason = %reason, "disclosure violation");
                }
            },
            Err(e) => {
                warn!(error = ?e, "stream error");
                break;
            }
        }
    }

    Ok(())
}
```

## Expected log output

```
INFO peer-request submitted request_id=fed-req-7f3a9b2c awaiting="tenancy-admin@drill-beta-vendor"
... (after peer accepts)
INFO federation active peer=drill-beta-vendor channels_bridged=2 mls_epoch=1
INFO message sent + delivered message_id=msg-abc123 mls_encrypted=true peer_delivered=true
INFO PHI denied by disclosure rule — expected reason="disclosure-rule EXPLICIT-OPT-IN requires PHI opt-in; peer drill-beta-vendor has NO opt-in"
INFO cross-tenant message received channel=supplier-status from_tenant=drill-beta-vendor from=supplier-rep@drill-beta-vendor data_class=Public occurred_at=2026-05-20T14:42:18Z body_preview="Confirming SKU XYZ-123 ships..."
INFO MLS rekey channel=supplier-status epoch=2
```

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "channel": "supplier-status",
        "body": "Q3 shipment ETA confirmation needed",
        "metadata": {
            "data_class": "PUBLIC",
            "thread_root": null,
            "attachments": []
        }
    }' \
    connect-api.drill-syd-1.oyatie.local:9090 \
    oya.connector.v1.ConnectService/SendMessage
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --service connect
```

Expected events:

- `federation_peer_request_created`
- `federation_handshake_completed` (mirrored)
- `mls_group_established` (mirrored)
- `disclosure_check_passed` (per-message)
- `cross_tenant_relay_started`
- `message_delivered_to_peer`
- `cross_tenant_message_received_from_peer` (mirror of peer's send)
- `disclosure_check_failed` (the PHI attempt)
- `mls_rekey_completed`

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `disclosure_rule_violation` | No | Message violates disclosure-rule baseline OR per-channel rule; review data-class metadata. |
| `peer_not_active` | No | Federation peer not active; check status. |
| `mls_group_not_joined` | No | The principal is not in the MLS group; add via channel-member-add first. |
| `tenant_rate_limit_exceeded` | Yes (back off) | Cross-tenant rate limit exceeded; SDK backs off + retries. |
| `peer_grace_period_expired` | No | Federation revoke completed; re-establish if needed. |
| `mls_rekey_in_progress` | Yes (wait + retry) | Rekey in flight; SDK waits 50 ms and retries. |
| `audit_chain_stall` | Yes | Audit chain backlog; SDK queues. |

## Where this file lives

`microservices/connector/reference-implementations/cross-tenant-message-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/connector/reference-implementations/federation-example/` once IP-005 + IP-007 + IP-008 + IP-009 land.
