---
doc_class: ReferenceImplementation
microservice: ops-dashboard-control-center
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Declare incident + rollback + export signed evidence pack via the ODCC Rust SDK

A runnable example that authenticates as an SRE operator, declares an incident, holds + rolls back a deployment, attaches evidence, and exports an HSM-signed + L1/L2-notarized evidence pack — using `oya-odcc-client` (target API; once IP-002 + IP-003 + IP-006 + IP-008 + IP-009 land).

## Cargo.toml

```toml
[package]
name = "odcc-operator-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-odcc-client = { path = "../../crates/oya-odcc-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
oya-webauthn-client = { path = "../../crates/oya-webauthn-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
uuid = { version = "1.10", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use oya_cedar_client::CedarPrincipal;
use oya_odcc_client::{
    DeploymentHoldRequest, DeploymentRollbackRequest, DeploymentRollbackStrategy,
    EvidencePackExportRequest, EvidencePackSigningMode, EvidencePackNotarizationTarget,
    IncidentClassification, IncidentDeclareRequest, IncidentResolveRequest, IncidentSeverity,
    OdccClient, OdccClientConfig, StepUpTier,
};
use oya_webauthn_client::WebAuthnAuthenticator;
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // 1. Operator authentication setup.
    let principal = CedarPrincipal::from_env("ODCC_PRINCIPAL_JWT")?;
    let authenticator = WebAuthnAuthenticator::from_env("WEBAUTHN_DEVICE_ID")?;
    let config = OdccClientConfig {
        api_endpoint: std::env::var("ODCC_API")?,
        cell: std::env::var("OYA_CELL_ID")?,
        principal,
        authenticator,
        request_timeout: std::time::Duration::from_secs(30),
    };
    let client = OdccClient::connect(config).await?;

    // 2. Verify operator scope + step-up freshness.
    let whoami = client.whoami().await?;
    info!(
        principal = %whoami.principal_id,
        permits = whoami.odcc_permits.len(),
        step_up_fresh = whoami.step_up_freshness.is_fresh(),
        "operator session ready"
    );

    // 3. Declare a SEV-2 incident.
    let incident_id_key = Uuid::new_v4().to_string();
    let declare_req = IncidentDeclareRequest {
        idempotency_key: incident_id_key.clone(),
        cell_id: "drill-syd-1".into(),
        severity: IncidentSeverity::Sev2,
        classification: IncidentClassification::CustomerImpacting,
        title: "messenger fanout p99 above SLO during v2.34.0 canary".into(),
        first_detected: Utc::now() - Duration::minutes(5),
        observed_by: "alert-manager-rule-msgr-fanout-p99".into(),
        commander_rotation: "oncall-sre-syd-rotation".into(),
        comm_channel: "#incident-2026-05-20-msgr-fanout".into(),
        evidence_refs: vec![
            "evidence/grafana/board-msgr-fanout-2026-05-20.png".into(),
            "evidence/sentry/issue-msgr-fanout-12345".into(),
        ],
        suspected_causes: vec!["messenger@v2.34.0 canary rollout".into()],
    };
    let declare_resp = client.incident_declare(declare_req).await?;
    info!(
        incident_id = %declare_resp.incident_id,
        audit_seal = %declare_resp.audit_seal_ref,
        state = ?declare_resp.state,
        "incident declared"
    );

    // 4. Place a deployment hold on the canary.
    let hold_req = DeploymentHoldRequest {
        idempotency_key: Uuid::new_v4().to_string(),
        service: "messenger".into(),
        version: "v2.34.0".into(),
        cell_id: "drill-syd-1".into(),
        rationale: format!(
            "incident {}: p99 latency violation at 12% rollout",
            declare_resp.incident_id
        ),
        hold_duration: Duration::minutes(60),
        linked_incident: Some(declare_resp.incident_id.clone()),
    };
    let hold_resp = client.deployment_hold(hold_req).await?;
    info!(
        hold_expires = %hold_resp.hold_expires,
        canary_state = ?hold_resp.canary_state,
        audit_seal = %hold_resp.audit_seal_ref,
        "deployment hold applied"
    );

    // 5. Investigate (simulated; in real life this is human work).
    info!("investigating: checking fanout worker pool depth board...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    info!("root cause identified: fanout-worker-pool-starvation introduced in commit 0x4f8c2d3a");

    // 6. Execute rollback. This requires Tier-2 step-up (Yubikey tap).
    let rollback_req = DeploymentRollbackRequest {
        idempotency_key: Uuid::new_v4().to_string(),
        service: "messenger".into(),
        from_version: "v2.34.0".into(),
        to_version: "v2.33.7".into(),
        cell_id: "drill-syd-1".into(),
        rollback_rationale: format!(
            "incident {}: messenger@v2.34.0 confirmed to introduce fanout-worker-pool-starvation; rolling back to v2.33.7 last-known-good per commit 0x4f8c2d3a being the offending change",
            declare_resp.incident_id
        ),
        rollback_strategy: DeploymentRollbackStrategy::RapidTrafficShift100Pct,
        evidence_refs: vec![
            "evidence/grafana/board-msgr-fanout-worker-pool-depth-2026-05-20.png".into(),
            "evidence/sentry/issue-msgr-fanout-worker-pool-starvation".into(),
            "evidence/git/commit-0x4f8c2d3a-introduces-regression".into(),
        ],
        linked_incident: Some(declare_resp.incident_id.clone()),
        step_up_tier: StepUpTier::Tier2,
    };
    info!("rollback command requires Tier-2 step-up. TAP YOUR YUBIKEY NOW...");
    let rollback_resp = client.deployment_rollback(rollback_req).await?;
    info!(
        active_version_after = %rollback_resp.active_version_after,
        traffic_shifted_at = %rollback_resp.traffic_shifted_at,
        audit_seal = %rollback_resp.audit_seal_ref,
        "rollback completed"
    );

    // 7. Resolve the incident.
    let resolve_req = IncidentResolveRequest {
        idempotency_key: Uuid::new_v4().to_string(),
        incident_id: declare_resp.incident_id.clone(),
        resolution_cause: "fanout-worker-pool-starvation introduced by messenger@v2.34.0 commit 0x4f8c2d3a; rolled back to v2.33.7".into(),
        resolution_time: Utc::now(),
        mitigation_applied: "deployment rollback to v2.33.7".into(),
        root_cause_confirmed_by: vec![
            "grafana-board-fanout-worker-pool-depth".into(),
            "sentry-issue-msgr-fanout-worker-pool-starvation".into(),
            "git-bisect".into(),
        ],
        follow_up_actions: vec![
            "messenger team to author fix patch for worker-pool-depth".into(),
            "re-canary v2.34.1 with worker-pool-depth alarm threshold lowered".into(),
        ],
    };
    let resolve_resp = client.incident_resolve(resolve_req).await?;
    info!(
        state = ?resolve_resp.state,
        resolved_at = %resolve_resp.resolved_at,
        audit_seal = %resolve_resp.audit_seal_ref,
        "incident resolved"
    );

    // 8. Export an evidence pack covering the incident window.
    let export_req = EvidencePackExportRequest {
        idempotency_key: Uuid::new_v4().to_string(),
        tenant_id: "drill-acme".into(),
        period_start: Utc::now() - Duration::hours(2),
        period_end: Utc::now(),
        frameworks: vec![
            "SOC2-CC7.2".into(),
            "SOC2-CC7.4".into(),
            "ISO27001-A.16.1".into(),
        ],
        evidence_scope: vec![
            "audit-chain-seals".into(),
            "cedar-decisions".into(),
            "operator-actions".into(),
            "deployment-records".into(),
            "incident-records".into(),
            "evidence-refs".into(),
        ],
        incident_id_filter: Some(declare_resp.incident_id.clone()),
        signing_mode: EvidencePackSigningMode::Hsm {
            partition: "syd-hsm-cluster-prod-1/odcc-evidence-key-v3".into(),
        },
        notarize_to: vec![
            EvidencePackNotarizationTarget::AwsQldb {
                ledger: "oyatie-odcc-evidence-ledger".into(),
            },
            EvidencePackNotarizationTarget::PolygonZkEvm {
                contract: "0xab12cd34evidence-anchor-contract".into(),
            },
        ],
        requester: "compliance-officer-acme".into(),
        case_ref: "SOC2-2026-Q2-INCIDENT-RESPONSE-EVIDENCE".into(),
    };
    let export_resp = client.evidence_pack_export(export_req).await?;
    info!(
        ticket_id = %export_resp.ticket_id,
        state = ?export_resp.state,
        estimated_completion_sec = export_resp.estimated_completion_sec,
        "evidence pack export queued"
    );

    // 9. Poll for completion + verify.
    let mut completed = false;
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let status = client.evidence_pack_status(&export_resp.ticket_id).await?;
        info!(state = ?status.state, "polling export status");
        if status.is_complete() {
            info!(
                pack_size_bytes = status.pack_size_bytes,
                content_hash = %status.content_hash,
                hsm_signature = %status.hsm_signature,
                qldb_anchor = ?status.qldb_anchor,
                polygon_anchor = ?status.polygon_anchor,
                download_url = %status.download_url,
                expires_at = %status.expires_at,
                "evidence pack ready"
            );
            completed = true;
            break;
        }
    }
    if !completed {
        warn!("evidence pack export not complete within 150s window");
    }

    Ok(())
}
```

## Expected log output

```
INFO operator session ready principal=oncall-sre-syd-rotation permits=12 step_up_fresh=true
INFO incident declared incident_id=inc-2026-05-20-msgr-fanout-001 audit_seal=ed25519-seal:0x7f3a9b2c... state=Declared
INFO deployment hold applied hold_expires=2026-05-20T15:30:00Z canary_state=FrozenAt12Pct audit_seal=ed25519-seal:0xab12cd34...
INFO investigating: checking fanout worker pool depth board...
INFO root cause identified: fanout-worker-pool-starvation introduced in commit 0x4f8c2d3a
INFO rollback command requires Tier-2 step-up. TAP YOUR YUBIKEY NOW...
INFO rollback completed active_version_after=v2.33.7 traffic_shifted_at=2026-05-20T14:42:23Z audit_seal=ed25519-seal:0xfedcba98...
INFO incident resolved state=Resolved resolved_at=2026-05-20T14:45:00Z audit_seal=ed25519-seal:0x12abcd34...
INFO evidence pack export queued ticket_id=evp-2026-05-20-7f3a9b2c state=Queued estimated_completion_sec=90
INFO polling export status state=Signing
INFO polling export status state=Notarizing
INFO evidence pack ready pack_size_bytes=4456789 content_hash=sha256:0x7f3a9b2c... hsm_signature=0xab12cd34... qldb_anchor=Some("hash-tree-anchor:oyatie-odcc-evidence/1234567") polygon_anchor=Some("tx:0xfedcba98...") download_url=https://evp.syd-1.oyatie.local/packs/evp-2026-05-20-7f3a9b2c expires_at=2026-05-27T15:15:00Z
```

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Cell-Id: drill-syd-1" \
    -H "X-Oya-Idempotency-Key: $(uuidgen)" \
    -H "X-Oya-Webauthn-Assertion: $WEBAUTHN_ASSERTION" \
    -d '{
        "severity": "SEV2",
        "classification": "CUSTOMER_IMPACTING",
        "title": "messenger fanout p99 above SLO during v2.34.0 canary",
        "commander_rotation": "oncall-sre-syd-rotation",
        "comm_channel": "#incident-2026-05-20-msgr-fanout"
    }' \
    odcc-api.drill-syd-1.oyatie.local:9090 \
    oya.odcc.v1.OdccService/IncidentDeclare
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --service odcc
```

Expected events:

- `incident_declared` (with full request body hash)
- `step_up_completed` (Tier 1)
- `deployment_hold_applied` (with rationale + duration)
- `step_up_completed` (Tier 2 for hold)
- `deployment_rollback_executed` (with from-version + to-version + rationale + evidence-refs)
- `step_up_completed` (Tier 2 for rollback)
- `incident_resolved` (with resolution-cause + mitigation)
- `evidence_pack_export_requested` (with frameworks + scope + signing-mode)
- `evidence_pack_signing_started`
- `evidence_pack_signed_with_hsm` (with hsm-partition-id + content-hash)
- `evidence_pack_notarized_qldb`
- `evidence_pack_notarized_polygon_zkevm`
- `evidence_pack_export_completed`

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `step_up_required` | No | Operator must complete WebAuthn step-up + retry. |
| `step_up_freshness_expired` | No | Step-up freshness expired (60-min window); refresh + retry. |
| `idempotency_key_conflict` | No | Duplicate idempotency key with different request body. Use new key. |
| `idempotency_key_repeat` | Yes (returns cached response) | Duplicate idempotency key with same request body → cached response returned. |
| `tenant_scope_violation` | No | Operator's tenant scope doesn't include the resource. Re-scope or escalate. |
| `audit_chain_unreachable` | Yes (back off) | Audit chain temporarily unreachable; SDK retries with back-off. After 5 retries, fail. |
| `hsm_partition_unavailable` | Yes (back off) | HSM partition unavailable; alarm fires; retry after 30 s. |
| `cross_pack_action_forbidden` | No | Pack policy forbids the action. Escalate or use cross-pack permit. |
| `tier_3_step_up_partner_timeout` | No | 2-person Tier-3 partner didn't complete step-up within 5 min. Restart. |

## Where this file lives

`microservices/ops-dashboard-control-center/reference-implementations/declare-incident-and-rollback-with-evidence-pack-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/ops-dashboard-control-center/reference-implementations/odcc-operator-example/` once IP-002 + IP-003 + IP-006 + IP-008 + IP-009 land.
