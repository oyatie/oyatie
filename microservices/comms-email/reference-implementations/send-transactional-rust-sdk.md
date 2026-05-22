---
doc_class: ReferenceImplementation
microservice: comms-email
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Send a transactional email via the comms-email Rust SDK

A runnable example that submits a templated transactional email, polls for delivery, subscribes to bounce/open/click webhooks, and demonstrates idempotency + suppression — using `oya-comms-email-client` (target API; once IP-005 + IP-007 + IP-009 land).

## Cargo.toml

```toml
[package]
name = "comms-email-send-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-comms-email-client = { path = "../../crates/oya-comms-email-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
uuid = { version = "1.10", features = ["v7"] }
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_cedar_client::CedarPrincipal;
use oya_comms_email_client::{
    Address, CommsEmailClient, CommsEmailClientConfig, DeliveryEvent, SendRequest, TemplateVar,
};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("COMMS_EMAIL_PRINCIPAL_JWT")?;
    let config = CommsEmailClientConfig {
        api_endpoint: std::env::var("COMMS_EMAIL_API")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = CommsEmailClient::connect(config).await?;

    // 1. Send a transactional email.
    let idempotency_key = format!("welcome-{}", Uuid::now_v7());
    let send_receipt = client
        .send(SendRequest {
            from: Address {
                email: "noreply@mail.acme.example".into(),
                name: Some("Acme Notifications".into()),
            },
            to: vec![Address {
                email: "alex@example.com".into(),
                name: Some("Alex Tester".into()),
            }],
            reply_to: Some(Address {
                email: "support@mail.acme.example".into(),
                name: Some("Acme Support".into()),
            }),
            subject_template: "Welcome to Acme, {{first_name}}".into(),
            template_id: "welcome-v3".into(),
            template_vars: vec![
                TemplateVar::new("first_name", json!("Alex")),
                TemplateVar::new("confirm_url", json!("https://acme.example/confirm?t=abc123")),
                TemplateVar::new("trial_days", json!(14)),
            ],
            idempotency_key: idempotency_key.clone(),
            tags: vec!["welcome".into(), "v3-template".into()],
            click_tracking: true,
            open_tracking: true,
            unsubscribe_link: true,
        })
        .await?;
    info!(
        message_id = %send_receipt.message_id,
        accepted = send_receipt.accepted,
        suppressed = send_receipt.suppressed,
        "send accepted"
    );

    // 2. Re-send with same idempotency key (should be a no-op + return cached receipt).
    let resend_receipt = client
        .send(SendRequest {
            from: Address {
                email: "noreply@mail.acme.example".into(),
                name: Some("Acme Notifications".into()),
            },
            to: vec![Address {
                email: "alex@example.com".into(),
                name: Some("Alex Tester".into()),
            }],
            reply_to: None,
            subject_template: "different subject".into(),  // ignored due to idempotency
            template_id: "welcome-v3".into(),
            template_vars: vec![],
            idempotency_key: idempotency_key.clone(),
            tags: vec![],
            click_tracking: true,
            open_tracking: true,
            unsubscribe_link: true,
        })
        .await?;
    assert_eq!(resend_receipt.message_id, send_receipt.message_id);
    info!("idempotency confirmed; second send returned cached receipt");

    // 3. Subscribe to delivery events (bounce, open, click, complaint).
    let mut event_stream = client
        .subscribe_delivery_events(&send_receipt.message_id)
        .await?;

    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => match event {
                DeliveryEvent::Delivered { mx, smtp_response_code } => {
                    info!(mx = %mx, code = smtp_response_code, "delivered");
                }
                DeliveryEvent::Bounced { bounce_class, smtp_code, diagnostic } => {
                    warn!(
                        class = ?bounce_class,
                        smtp_code,
                        diagnostic = %diagnostic,
                        "bounced"
                    );
                    if bounce_class.is_hard() {
                        // Auto-suppression already added; we just log.
                        info!(recipient = %send_receipt.message_id, "recipient added to suppression");
                    }
                }
                DeliveryEvent::Opened { user_agent, ip } => {
                    info!(user_agent = %user_agent, ip = ?ip, "opened");
                }
                DeliveryEvent::Clicked { url, user_agent, ip } => {
                    info!(url = %url, user_agent = %user_agent, ip = ?ip, "clicked");
                }
                DeliveryEvent::Complained { feedback_loop_provider } => {
                    warn!(provider = %feedback_loop_provider, "complaint");
                }
                DeliveryEvent::Unsubscribed { reason } => {
                    info!(reason = ?reason, "unsubscribed");
                }
            },
            Err(e) => {
                warn!(error = ?e, "event stream error");
                break;
            }
        }
    }

    Ok(())
}
```

## Expected log output

```
INFO send accepted message_id=msg-7f3a9b2c accepted=true suppressed=false
INFO idempotency confirmed; second send returned cached receipt
INFO delivered mx=mx1.gmail.com code=250
INFO opened user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15" ip=Some(203.0.113.42)
INFO clicked url="https://acme.example/confirm?t=abc123" user_agent="Mozilla/5.0..." ip=Some(203.0.113.42)
```

## Direct HTTP alternative

```sh
curl -X POST https://api.comms-email.drill-syd-1.oyatie.local/v1/send \
  -H "Authorization: Bearer $JWT" \
  -H "X-Oya-Tenant-Id: drill-acme" \
  -H "Idempotency-Key: welcome-7f3a9b2c1234567" \
  -H "Content-Type: application/json" \
  -d '{
    "from": {"email": "noreply@mail.acme.example", "name": "Acme Notifications"},
    "to": [{"email": "alex@example.com", "name": "Alex Tester"}],
    "reply_to": {"email": "support@mail.acme.example"},
    "subject_template": "Welcome to Acme, {{first_name}}",
    "template_id": "welcome-v3",
    "template_vars": {
      "first_name": "Alex",
      "confirm_url": "https://acme.example/confirm?t=abc123",
      "trial_days": 14
    },
    "tags": ["welcome", "v3-template"],
    "click_tracking": true,
    "open_tracking": true,
    "unsubscribe_link": true
  }'
```

Expected response:

```json
{
  "message_id": "msg-7f3a9b2c",
  "accepted": true,
  "suppressed": false,
  "estimated_deliver_at": "2026-05-20T14:30:01Z"
}
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 1h --service comms-email
```

Expected events:

- `message_accepted`
- `dkim_signed`
- `message_dispatched_to_mta`
- `delivery_received` (when MX ACKs)
- `open_event` (if opened)
- `click_event` (if clicked)
- `bounce_received` (if bounced)
- `complaint_received` (if complained)

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks send permission. Fix at IAM. |
| `tenant_send_limit_exceeded` | Yes (wait + retry) | Tenant exceeded daily/per-minute envelope; back off + retry. |
| `recipient_suppressed` | No | The recipient is on the suppression list; do not send. |
| `dkim_sign_failed` | Yes | HSM unavailable; SDK retries with circuit-breaker. |
| `template_render_error` | No | Template has unbound var or syntax error; fix at caller. |
| `from_domain_not_authorised` | No | The from-domain DNS posture isn't validated; complete onboarding. |
| `idempotency_key_collision` | No | The idempotency key matches a different message's params (different recipients, different template); use new key. |
| `tenant_not_warmed` | No | Tenant IP-pool not at the day's target volume; wait or use lower-tier path. |

## Where this file lives

`microservices/comms-email/reference-implementations/send-transactional-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/comms-email/reference-implementations/send-example/` once IP-005 + IP-007 + IP-009 land.
