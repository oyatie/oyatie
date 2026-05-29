---
doc_class: ReferenceImplementation
microservice: api-gateway
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Deploy + canary-ramp + rollback an edge listener via the api-gateway Rust SDK

A runnable example that validates a listener config, deploys via canary-cohort-shifter, ramps to 100 %, and demonstrates rollback — using `oya-api-gateway-client` (target API; once IP-002 + IP-005 + IP-006 + IP-015 land).

## Cargo.toml

```toml
[package]
name = "edge-listener-deploy-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-api-gateway-client = { path = "../../crates/oya-api-gateway-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
anyhow = "1.0"
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_api_gateway_client::{
    ApiGatewayClient, ApiGatewayClientConfig, CanaryStepEvent, CanaryStepRequest, CertIssueRequest,
    ListenerConfig, ListenerValidateRequest, RollbackRequest,
};
use oya_cedar_client::CedarPrincipal;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("API_GATEWAY_PRINCIPAL_JWT")?;
    let config = ApiGatewayClientConfig {
        control_plane_endpoint: std::env::var("API_GATEWAY_CTL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(30),
    };
    let client = ApiGatewayClient::connect(config).await?;

    // 1. Load + validate the listener config.
    let listener_yaml = std::fs::read_to_string("./configs/acme-edge-v1.yaml")?;
    let listener: ListenerConfig = serde_yaml::from_str(&listener_yaml)?;

    let validation = client
        .validate_listener(ListenerValidateRequest {
            listener: listener.clone(),
        })
        .await?;
    if !validation.is_ok() {
        error!(errors = ?validation.errors, "listener validation failed");
        std::process::exit(1);
    }
    info!(listener = %listener.name, "listener config valid");

    // 2. Issue cert (ACME flow).
    let cert_receipt = client
        .issue_cert(CertIssueRequest {
            listener_name: listener.name.clone(),
            wait_for_completion: true,
        })
        .await?;
    info!(
        cert_serial = %cert_receipt.cert_serial,
        expiry = %cert_receipt.expiry,
        "cert issued"
    );

    // 3. Canary-deploy at 5 %.
    let step5 = client
        .canary_step(CanaryStepRequest {
            listener_name: listener.name.clone(),
            cohort_pct: 5,
            duration_seconds: 600,
            watch_slos: vec![
                "edge-latency".into(),
                "waf-block-rate".into(),
                "rate-limit-drop".into(),
            ],
        })
        .await?;
    info!(step_id = %step5.step_id, "5 % canary step started");

    // Subscribe to step events.
    let mut step_stream = client.subscribe_canary_events(&listener.name).await?;
    while let Some(event_result) = step_stream.next().await {
        match event_result {
            Ok(event) => match event {
                CanaryStepEvent::SloGreen { cohort_pct, slo_name, value } => {
                    info!(
                        cohort_pct,
                        slo = %slo_name,
                        value = ?value,
                        "SLO green at cohort step"
                    );
                }
                CanaryStepEvent::SloAmber { cohort_pct, slo_name, value, threshold } => {
                    warn!(
                        cohort_pct,
                        slo = %slo_name,
                        value = ?value,
                        threshold = ?threshold,
                        "SLO amber"
                    );
                }
                CanaryStepEvent::SloRed { cohort_pct, slo_name, value, threshold } => {
                    error!(
                        cohort_pct,
                        slo = %slo_name,
                        value = ?value,
                        threshold = ?threshold,
                        "SLO red; canary will halt"
                    );
                    // Decide: rollback or wait
                    let rb = client
                        .rollback(RollbackRequest {
                            listener_name: listener.name.clone(),
                            reason: format!("SLO red on {} at {}%", slo_name, cohort_pct),
                        })
                        .await?;
                    info!(rollback_id = %rb.rollback_id, "rollback initiated");
                    return Ok(());
                }
                CanaryStepEvent::StepCompleted { cohort_pct } => {
                    info!(cohort_pct, "step completed; ramping to next");
                    if cohort_pct >= 100 {
                        info!("canary at 100%; promotion complete");
                        break;
                    }
                    let next_pct = match cohort_pct {
                        5 => 15,
                        15 => 50,
                        50 => 100,
                        _ => 100,
                    };
                    let next_step = client
                        .canary_step(CanaryStepRequest {
                            listener_name: listener.name.clone(),
                            cohort_pct: next_pct,
                            duration_seconds: if next_pct == 100 { 600 } else { 900 },
                            watch_slos: vec![
                                "edge-latency".into(),
                                "waf-block-rate".into(),
                                "rate-limit-drop".into(),
                            ],
                        })
                        .await?;
                    info!(step_id = %next_step.step_id, cohort_pct = next_pct, "next step started");
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

## Expected log output (happy path)

```
INFO listener config valid listener=tenant-edge-acme-v1
INFO cert issued cert_serial=0x7f3a9b2c1234 expiry=2026-08-18T00:00:00Z
INFO 5 % canary step started step_id=step-abc1
INFO SLO green at cohort step cohort_pct=5 slo=edge-latency value=Latency { p99_ms: 28 }
INFO SLO green at cohort step cohort_pct=5 slo=waf-block-rate value=Pct(0.0)
INFO SLO green at cohort step cohort_pct=5 slo=rate-limit-drop value=Pct(0.0)
INFO step completed; ramping to next cohort_pct=5
INFO next step started step_id=step-abc2 cohort_pct=15
INFO SLO green at cohort step cohort_pct=15 slo=edge-latency value=Latency { p99_ms: 29 }
... (continues)
INFO step completed; ramping to next cohort_pct=50
INFO next step started step_id=step-abc4 cohort_pct=100
INFO SLO green at cohort step cohort_pct=100 slo=edge-latency value=Latency { p99_ms: 31 }
INFO step completed; ramping to next cohort_pct=100
INFO canary at 100%; promotion complete
```

## Expected log output (rollback path)

```
... initial steps green ...
WARN SLO amber cohort_pct=50 slo=edge-latency value=Latency { p99_ms: 38 } threshold=Latency { p99_ms: 35 }
ERROR SLO red; canary will halt cohort_pct=50 slo=edge-latency value=Latency { p99_ms: 52 } threshold=Latency { p99_ms: 35 }
INFO rollback initiated rollback_id=rb-xyz1
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 90m --service api-gateway
```

Expected events:

- `listener_validated` × 1
- `cert_issued` × 1
- `listener_canary_step_started` × 4 (5 %, 15 %, 50 %, 100 %)
- `listener_canary_step_completed` × 4
- `listener_promoted_full` × 1 (only on happy path)
- `listener_rolled_back` × 1 (only on rollback path)

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "listener": {
            "name": "tenant-edge-acme-v1",
            "tenant_id": "drill-acme",
            "bind": [{"host": "edge.acme.oyatie.io", "port": 443}],
            "tls": {"cert_source": {"type": "ACME"}, "min_version": "TLSv1.3"},
            "routes": [...]
        }
    }' \
    api-gateway-ctl.drill-syd-1.oyatie.local:9090 \
    oya.api_gateway.v1.ApiGatewayControlService/ValidateListener
```

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `listener_invalid_config` | No | Validation failed; fix config + retry. |
| `cert_acme_challenge_failed` | Yes | DNS-01 challenge couldn't publish; verify DNS provider permissions. |
| `cert_hsm_unavailable` | Yes (circuit-breaker) | HSM cluster degraded; SDK retries. |
| `canary_slo_red` | No (specific recovery) | Rollback per runbook. |
| `xds_push_nack` | No | Envoy NACK'd config; inspect Envoy logs. |
| `listener_already_exists` | No | Use `update_listener` instead. |
| `tenant_unauthorized` | No | Tenant lacks tenant_class eligibility or policy for the requested config (for example, asking paid-only behavior under demo_trial). |

## Where this file lives

`microservices/api-gateway/reference-implementations/edge-listener-deploy-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/api-gateway/reference-implementations/edge-deploy-example/` once IP-002 + IP-005 + IP-006 + IP-015 land.
