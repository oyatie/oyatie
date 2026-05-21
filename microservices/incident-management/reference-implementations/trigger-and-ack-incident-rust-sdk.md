---
doc_class: ReferenceImplementation
microservice: incident-management
language: rust
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Reference — Trigger + acknowledge an incident via the oyatie incident-management Rust SDK

This walkthrough shows a monitoring-tool integration worker triggering a SEV-2 incident, the on-call acknowledging via the SDK, the substrate's escalation policy stopping further paging, and a post-mortem being authored after resolution.

## Cargo.toml

```toml
[package]
name = "monitoring-integration-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-incident-sdk = { path = "../../crates/oya-incident-sdk" }
oya-iam-sdk = { path = "../../crates/oya-iam-sdk" }
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
tokio = { version = "1.42", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.10", features = ["v4"] }
```

## src/main.rs

```rust
use anyhow::Context;
use chrono::Utc;
use oya_incident_sdk::{
    IncidentClient, IncidentTriggerRequest, IncidentSeverity, IncidentAckRequest,
    IncidentStateTransitionRequest, IncidentState, PostMortemPublishRequest,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

struct MonitoringWorker {
    incident_client: IncidentClient,
    iam_client: IamClient,
    tenant_id: String,
}

impl MonitoringWorker {
    async fn handle_alert(&self, alert: AlertPayload) -> anyhow::Result<String> {
        let trigger_req = IncidentTriggerRequest {
            tenant_id: self.tenant_id.clone(),
            external_alert_id: alert.alert_id.clone(),
            service_slug: alert.service_slug.clone(),
            severity: match alert.severity.as_str() {
                "critical" => IncidentSeverity::Sev1,
                "high" => IncidentSeverity::Sev2,
                "medium" => IncidentSeverity::Sev3,
                _ => IncidentSeverity::Sev4,
            },
            summary: alert.summary.clone(),
            source: "datadog-monitor-id-12345".into(),
            runbook_url: Some(alert.runbook_url.clone()),
            metadata: json!({
                "datadog_monitor_id": alert.alert_id,
                "datadog_event_url": alert.event_url,
                "current_error_rate": alert.current_value,
                "threshold": alert.threshold,
            }),
            idempotency_key: Uuid::new_v4().to_string(),
        };

        let triggered = self
            .incident_client
            .trigger_incident(trigger_req)
            .await
            .context("trigger_incident failed")?;

        info!(
            incident_id = %triggered.incident_id,
            severity = ?triggered.severity,
            on_call_paged = ?triggered.paged_principals,
            war_room_channel = ?triggered.war_room_channel,
            "incident triggered"
        );

        Ok(triggered.incident_id)
    }

    async fn acknowledge_incident(
        &self,
        incident_id: &str,
        acker_principal_id: &str,
    ) -> anyhow::Result<()> {
        self.incident_client
            .acknowledge(IncidentAckRequest {
                incident_id: incident_id.into(),
                acker_principal_id: acker_principal_id.into(),
                ack_method: "rust-sdk".into(),
                comment: Some("On it; investigating now".into()),
            })
            .await?;
        info!(incident_id = incident_id, acker = acker_principal_id, "acknowledged");
        Ok(())
    }

    async fn transition_to_mitigated(
        &self,
        incident_id: &str,
        mitigation_summary: &str,
    ) -> anyhow::Result<()> {
        self.incident_client
            .transition_state(IncidentStateTransitionRequest {
                incident_id: incident_id.into(),
                to_state: IncidentState::Mitigated,
                comment: Some(mitigation_summary.into()),
                mitigation_actions: vec!["rolled back deploy".into()],
            })
            .await?;
        Ok(())
    }

    async fn transition_to_resolved(&self, incident_id: &str) -> anyhow::Result<()> {
        self.incident_client
            .transition_state(IncidentStateTransitionRequest {
                incident_id: incident_id.into(),
                to_state: IncidentState::Resolved,
                comment: Some("Verified stable for 30 min after mitigation".into()),
                mitigation_actions: vec![],
            })
            .await?;
        Ok(())
    }

    async fn publish_post_mortem(&self, incident_id: &str) -> anyhow::Result<()> {
        let body = format!(
            "# Post-Mortem for {}\n\n## Summary\n\nCircuit-breaker threshold regression caused 28-min outage.\n\n## Root cause\n\nDeploy reduced threshold below baseline error rate.\n\n## Action items\n\n1. Add circuit-breaker checks to deploy review (owner: alice@, due 2026-05-28).\n2. Synthetic-validation pre-deploy (owner: bob@, due 2026-06-15).\n3. Update runbook ordering (owner: alice@, due 2026-05-25).\n",
            incident_id
        );
        self.incident_client
            .publish_post_mortem(PostMortemPublishRequest {
                incident_id: incident_id.into(),
                version: "1.0.0".into(),
                body_markdown: body,
                action_items: vec![
                    ("alice@your-tenant.com".into(), "Add circuit-breaker checks to deploy review".into(), "2026-05-28".into()),
                    ("bob@your-tenant.com".into(), "Synthetic-validation pre-deploy".into(), "2026-06-15".into()),
                    ("alice@your-tenant.com".into(), "Update runbook ordering".into(), "2026-05-25".into()),
                ],
            })
            .await?;
        info!(incident_id = incident_id, "post-mortem published");
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AlertPayload {
    alert_id: String,
    service_slug: String,
    severity: String,
    summary: String,
    runbook_url: String,
    event_url: String,
    current_value: f64,
    threshold: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("monitoring-integration-worker")?;

    let incident_client = IncidentClient::from_env()?;
    let iam_client = IamClient::from_env()?;
    let principal: Principal = iam_client.whoami().await?;

    let worker = MonitoringWorker {
        incident_client,
        iam_client,
        tenant_id: principal.tenant_id.clone(),
    };

    let alert = AlertPayload {
        alert_id: "datadog-monitor-12345-trigger-1".into(),
        service_slug: "cloud-billing-tax-app".into(),
        severity: "high".into(),
        summary: "5xx error rate 35% on cloud-billing-tax-app /api/v1/invoices".into(),
        runbook_url: "https://github.com/oyatie/oyatie/blob/dev/microservices/cloud-billing-tax-app/runbooks/availability-breach.md".into(),
        event_url: "https://app.datadoghq.com/event/event?id=12345".into(),
        current_value: 0.35,
        threshold: 0.05,
    };

    let incident_id = worker.handle_alert(alert).await?;

    // Simulate the on-call acknowledging within a few seconds
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    worker
        .acknowledge_incident(&incident_id, "user:alice@your-tenant.com")
        .await?;

    // Simulate the on-call mitigating + resolving
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    worker
        .transition_to_mitigated(&incident_id, "Rolled back deploy v2.3.5 → v2.3.4")
        .await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    worker.transition_to_resolved(&incident_id).await?;

    // Publish post-mortem
    worker.publish_post_mortem(&incident_id).await?;

    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"monitoring-worker@tenant-acme",
    action in [
        Action::"incident::trigger",
        Action::"incident::read",
        Action::"incident::ack",
        Action::"incident::transition_state",
        Action::"post_mortem::publish"
    ],
    resource in Tenant::"tenant_acme"
);
```

## Compliance evidence emitted

Every state transition emits to `audit-chain`:

```json
{
    "event_class": "incident::triggered",
    "tenant_id": "tenant_acme",
    "incident_id": "INC-2026-05-20-001",
    "severity": "SEV-2",
    "service": "cloud-billing-tax-app",
    "source": "datadog-monitor-id-12345",
    "paged_principals": ["alice@", "bob@"],
    "escalation_policy": "platform-sev2"
}
```

Plus `incident::acknowledged`, `incident::state_changed`, `post_mortem::published` etc. Full lifecycle is cryptographically anchored for SOC 2 + ISO 27001 + regulator audit purposes.

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_INCIDENT_API=https://incident-api.dev.<tenant>.oyatie.io \
OYA_IAM_API=https://iam-api.dev.<tenant>.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in the portal: portal → Incidents → search for the incident ID. You should see the full lifecycle + post-mortem + audit-chain anchor proofs.

## Notes

- Idempotency: every trigger includes an `idempotency_key`. Re-firing the same key within 60 s returns the existing incident_id instead of creating a duplicate. This is critical for monitoring tools that retry alerts on transient failures.
- The `external_alert_id` field links back to the source monitoring tool. Useful for cross-correlation in post-mortems ("Datadog monitor 12345 fired at 14:23 UTC → oyatie incident INC-2026-05-20-001 → resolved at 14:50 UTC").
- For high-volume monitoring integrations (1000+ alerts/sec), use the bulk-trigger endpoint that batches up to 100 incidents per call.
- The post-mortem body is Markdown; the substrate parses + structures it for the action-item tracker. Action items embedded as bullet lists with `(owner: X, due: YYYY-MM-DD)` are auto-extracted.
- For paid tenant_class tier with AI-triage, the trigger response includes `ai_suggested_classification` with confidence scores for service + likely cause. Use as a hint but the human IC has final authority.
