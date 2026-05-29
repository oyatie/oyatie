---
doc_class: ReferenceImplementation
microservice: forms
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Submit form + payment + warehouse-export via the forms Rust SDK

A runnable example that submits a form with payment, polls submission status, subscribes to webhook events, and triggers warehouse-export — using `oya-forms-client` (target API; once IP-005 + IP-007 + IP-009 + IP-014 land).

## Cargo.toml

```toml
[package]
name = "forms-submit-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-forms-client = { path = "../../crates/oya-forms-client" }
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
use oya_forms_client::{
    FormsClient, FormsClientConfig, PaymentMethod, SubmissionRequest, SubmissionStatus,
    WebhookEvent,
};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("FORMS_PRINCIPAL_JWT")?;
    let config = FormsClientConfig {
        api_endpoint: std::env::var("FORMS_API")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(30),
    };
    let client = FormsClient::connect(config).await?;

    // 1. Submit a form.
    let idempotency_key = Uuid::now_v7().to_string();
    let submission_receipt = client
        .submit(SubmissionRequest {
            form_name: "acme-2026-conference-registration".into(),
            field_values: vec![
                ("first-name".into(), json!("Alex")),
                ("last-name".into(), json!("Tester")),
                ("email".into(), json!("alex@example.com")),
                ("phone".into(), json!("+15551234567")),
                ("company".into(), json!("Acme Corp")),
                ("role".into(), json!("Engineer")),
                ("ticket-tier".into(), json!("Early-bird (199 USD; before May 31)")),
                ("dietary-restriction".into(), json!("Vegetarian")),
                ("session-tracks".into(), json!(["Engineering", "Product"])),
            ],
            payment: Some(PaymentMethod::StripeToken {
                token: std::env::var("STRIPE_TEST_TOKEN")?,
                stripe_customer_id: None,
            }),
            captcha_token: std::env::var("CAPTCHA_TOKEN").ok(),
            idempotency_key: idempotency_key.clone(),
            consent_signoffs: vec![
                "privacy-policy-2026-05".into(),
                "terms-of-service-2026-05".into(),
            ],
        })
        .await?;
    info!(
        submission_id = %submission_receipt.submission_id,
        status = ?submission_receipt.status,
        payment_intent_id = ?submission_receipt.payment_intent_id,
        "submission accepted"
    );

    // 2. Idempotency check (re-submit with same key returns cached receipt).
    let resubmit_receipt = client
        .submit(SubmissionRequest {
            form_name: "acme-2026-conference-registration".into(),
            field_values: vec![],  // ignored due to idempotency
            payment: None,
            captcha_token: None,
            idempotency_key: idempotency_key.clone(),
            consent_signoffs: vec![],
        })
        .await?;
    assert_eq!(resubmit_receipt.submission_id, submission_receipt.submission_id);
    info!("idempotency confirmed");

    // 3. Subscribe to submission events.
    let mut event_stream = client
        .subscribe_submission_events(&submission_receipt.submission_id)
        .await?;

    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => match event {
                WebhookEvent::PaymentInitiated { payment_intent_id, amount_cents, currency } => {
                    info!(
                        payment_intent_id = %payment_intent_id,
                        amount_cents,
                        currency = %currency,
                        "payment initiated"
                    );
                }
                WebhookEvent::PaymentSucceeded { payment_intent_id, amount_cents } => {
                    info!(
                        payment_intent_id = %payment_intent_id,
                        amount_cents,
                        "payment succeeded"
                    );
                }
                WebhookEvent::PaymentFailed { payment_intent_id, error_code, error_message } => {
                    warn!(
                        payment_intent_id = %payment_intent_id,
                        error_code,
                        error_message = %error_message,
                        "payment failed"
                    );
                }
                WebhookEvent::SubmissionCompleted { warehouse_targets } => {
                    info!(
                        warehouse_targets = ?warehouse_targets,
                        "submission committed; warehouse export queued"
                    );
                }
                WebhookEvent::WarehouseExportSucceeded { target } => {
                    info!(target = %target, "warehouse export succeeded");
                }
                WebhookEvent::WarehouseExportFailed { target, error_message } => {
                    warn!(target = %target, error_message = %error_message, "warehouse export failed");
                }
                WebhookEvent::NotificationDispatched { target, channel } => {
                    info!(target = %target, channel = %channel, "notification sent");
                }
                WebhookEvent::DsarRequest { subject_email, request_type } => {
                    info!(
                        subject_email = %subject_email,
                        request_type = ?request_type,
                        "DSAR request"
                    );
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
INFO submission accepted submission_id=sub-7f3a9b2c status=PaymentPending payment_intent_id=Some("pi_test_abc")
INFO idempotency confirmed
INFO payment initiated payment_intent_id="pi_test_abc" amount_cents=19900 currency="USD"
INFO payment succeeded payment_intent_id="pi_test_abc" amount_cents=19900
INFO submission committed; warehouse export queued warehouse_targets=["bigquery://acme-data/conferences.registrations", "snowflake://acme-warehouse/CONFERENCES.REGISTRATIONS"]
INFO notification sent target="registration@acme.example" channel="email"
INFO notification sent target="conferences" channel="slack-webhook"
INFO warehouse export succeeded target="bigquery://acme-data/conferences.registrations"
INFO warehouse export succeeded target="snowflake://acme-warehouse/CONFERENCES.REGISTRATIONS"
```

## Direct HTTP alternative

```sh
curl -X POST https://api.forms.drill-syd-1.oyatie.local/v1/forms/acme-2026-conference-registration/submit \
  -H "Authorization: Bearer $JWT" \
  -H "X-Oya-Tenant-Id: drill-acme" \
  -H "Idempotency-Key: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{
    "field_values": {
        "first-name": "Alex",
        "last-name": "Tester",
        "email": "alex@example.com",
        "phone": "+15551234567",
        "company": "Acme Corp",
        "role": "Engineer",
        "ticket-tier": "Early-bird (199 USD; before May 31)",
        "dietary-restriction": "Vegetarian",
        "session-tracks": ["Engineering", "Product"]
    },
    "payment": {
        "type": "stripe-token",
        "token": "tok_visa_test"
    },
    "consent_signoffs": ["privacy-policy-2026-05", "terms-of-service-2026-05"]
}'
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --service forms
```

Expected events:

- `submission_started`
- `cedar_check_passed`
- `captcha_validated`
- `payment_initiated`
- `payment_succeeded`
- `submission_completed`
- `consent_recorded`
- `warehouse_export_queued`
- `warehouse_export_completed` (per target)
- `notification_sent` (per notification)

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission OR form is not published. |
| `form_not_found` | No | Form name doesn't exist or not published. |
| `field_validation_failed` | No | Required field missing or value invalid. |
| `captcha_failed` | No | Captcha challenge failed; user re-tries. |
| `payment_card_declined` | No | Card declined by issuer; user uses different card. |
| `payment_3ds_required` | Pending (user action) | Strong Customer Authentication; user redirected. |
| `warehouse_export_failed` | Yes | Backoff + retry; check target health. |
| `consent_required_not_provided` | No | One or more required consents not signed off. |
| `idempotency_key_collision` | No | Same key with different params; use new key. |
| `tenant_quota_exceeded` | No | Submissions/month cap; upgrade tier. |
| `file_upload_too_large` | No | Exceeds per-file or per-submission limit. |

## Where this file lives

`microservices/forms/reference-implementations/submit-form-and-export-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/forms/reference-implementations/forms-example/` once IP-005 + IP-007 + IP-009 + IP-014 land.
