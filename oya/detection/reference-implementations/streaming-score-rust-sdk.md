---
doc_class: ReferenceImplementation
microservice: detection
language: Rust + Cedar + ONNX
date: 2026-05-20
doc_status: published
---

# Reference implementation — Streaming payment-fraud detection via the detection Rust SDK

A runnable example that emits a synthetic payment transaction to a streaming Flink job, receives the score + mitigation verdict, and reads the SHAP-class explanation — using the `oya-detection-client` crate (target API; once IP-001 + IP-002 + IP-007 + IP-008 land).

## Cargo.toml

```toml
[package]
name = "detection-streaming-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-detection-client = { path = "../../crates/oya-detection-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = "0.4"
uuid = { version = "1.10", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::Utc;
use oya_cedar_client::CedarPrincipal;
use oya_detection_client::{
    DetectionClient, DetectionClientConfig, MitigationAction, PaymentEvent, ScoreResult,
};
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // 1. Construct the detection client. Bound to a Cedar principal that carries
    //    `detection::payment-fraud::score` permission.
    let principal = CedarPrincipal::from_env("DETECTION_PRINCIPAL_JWT")?;
    let config = DetectionClientConfig {
        cell_endpoint: std::env::var("DETECTION_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(5),
        max_retries: 2,
    };
    let client = DetectionClient::connect(config).await?;

    // 2. Construct a synthetic payment event.
    let event = PaymentEvent {
        event_id: Uuid::new_v4().to_string(),
        event_time: Utc::now(),
        user_id: "drill-customer-z".into(),
        payment_id: format!("pay-{}", Uuid::new_v4()),
        amount_cents: 350,    // $3.50 — small denomination, common in card-testing
        currency: "USD".into(),
        card_bin: "424242".into(),
        card_country_iso: "US".into(),
        merchant_country_iso: "US".into(),
        principal_country_iso: "US".into(),
        is_card_not_present: true,
        ip_address: "203.0.113.42".into(),
        device_fingerprint: "fp-abc123".into(),
    };

    // 3. Score the event synchronously. The streaming substrate (Flink) reads from
    //    Pulsar; the synchronous-score SDK call is wired through a "synchronous"
    //    Pulsar topic that bypasses normal Flink batch-async + emits the verdict
    //    via a per-request response topic. p99 ≤ 200ms.
    let result: ScoreResult = client
        .score(oya_detection_client::ScoreRequest {
            family: "payment-fraud".into(),
            event: event.clone(),
            timeout: std::time::Duration::from_millis(500),
        })
        .await?;

    info!(
        score = result.score,
        confidence = result.confidence,
        mitigation = ?result.mitigation,
        rule_attribution = ?result.rule_attribution,
        model_card_id = ?result.model_card_id,
        "score result"
    );

    // 4. Branch on mitigation.
    match result.mitigation {
        MitigationAction::Allow => {
            info!("transaction allowed; no friction");
        }
        MitigationAction::StepUpAuth => {
            info!(
                challenge_kind = ?result.step_up_challenge_kind,
                "3DS challenge required"
            );
        }
        MitigationAction::Block => {
            warn!(
                reason_codes = ?result.reason_codes,
                "transaction blocked; reason codes available for ECOA Reg B notice"
            );
            // Surface to user with the reason codes.
        }
        MitigationAction::HumanReview => {
            info!(
                queue = ?result.review_queue,
                expected_review_seconds = ?result.expected_review_seconds,
                "queued for human review"
            );
        }
        MitigationAction::LogOnly => {
            info!("logged only; no mitigation applied (shadow mode)");
        }
    }

    // 5. Fetch the SHAP-class explanation for the decision.
    if let Some(explanation) = client.explain_decision(&result.decision_id).await? {
        info!("decision explanation:");
        for (feature, contribution) in explanation.feature_contributions {
            info!("  {} = {:+.4}", feature, contribution);
        }
        info!("counter-factuals:");
        for cf in explanation.counter_factuals {
            info!("  {}", cf);
        }
    }

    Ok(())
}
```

## Expected log output (for a velocity-fired block)

```
INFO score result score=0.94 confidence=1.000 mitigation=Block rule_attribution=Some("payment-fraud-card-testing/Rule 1") model_card_id=Some("payment-fraud-v1") 
WARN transaction blocked; reason codes available for ECOA Reg B notice reason_codes=["velocity-burst-detected", "small-denomination-pattern"]
INFO decision explanation:
INFO   rule_of_n_small_charges_600s = +0.6200
INFO   account_age_days = +0.1800
INFO   is_card_not_present = +0.0900
INFO   merchant_country_match = +0.0500
INFO   ip_velocity_country = +0.0400
INFO counter-factuals:
INFO   If amount_cents was > 1000: rule 1 would not fire
INFO   If account_age_days >= 30: rule 3 would not fire
```

## Audit chain emission

After `client.score()` returns, the audit chain contains:

```sh
oya audit query --tenant drill-acme --since 5m --event-class detection_*,mitigation_*,cedar_*
```

Expected events (Ed25519-signed):

- `detection_score_emitted` (with score + features + rule + model attribution).
- `cedar_decision_audit` (with policy ID + decision + reasons).
- `mitigation_action_emitted` (with action + reason codes).
- `decision_explanation_generated` (with SHAP contributions; async; lands within ~ 200 ms after the score).

## Direct gRPC alternative (until the SDK lands)

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "family": "payment-fraud",
        "event": {
            "event_id": "...",
            "event_time": "2026-05-20T13:42:00Z",
            "user_id": "drill-customer-z",
            "amount_cents": 350,
            ...
        }
    }' \
    detection.drill-syd-1.oyatie.local:9090 \
    oya.detection.v1.DetectionService/Score
```

## Error handling — what to retry

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks `detection::*::score` permission. Fix at IAM. |
| `event_validation_failed` | No | Event schema invalid. Fix at caller. |
| `model_card_missing` | No | The model has not been deployed yet OR the cell is on demo_trial tenant_class (no models). Deploy or downgrade to rules-only call. |
| `quota_exceeded` | Yes (auto, exponential backoff) | Tenant hit per-second quota. Backoff and retry. |
| `pipeline_timeout` | No (the timeout is the budget) | The pipeline is overloaded; fail-fast at the caller; the SDK does NOT silently auto-retry score requests (each retry is another scoring decision that emits another audit event). |
| `cell_unavailable` | Yes (with circuit-breaker) | Cell is down; SDK fails after 3 retries; circuit-breaker opens for 30 s. |

## Where this file lives in the µservice

`microservices/detection/reference-implementations/streaming-score-rust-sdk.md` (this file).

The runnable Cargo project will land at `microservices/detection/reference-implementations/streaming-example/` once IP-001 + IP-002 + IP-007 + IP-008 land the synchronous-score path + the SDK.
