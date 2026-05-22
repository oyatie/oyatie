---
doc_class: ReferenceImplementation
microservice: itsm
language: rust
related_adrs: [ADR-0316]
date: 2026-05-20
doc_status: published
---

# Reference — Submit an ITIL v4 Normal Change via the oyatie itsm Rust SDK

Goal: from a tenant's DevOps worker, programmatically submit a Normal Change ticket for a planned database upgrade, attach the CMDB impact analysis, await CAB approval, and execute the change with rollback hooks.

## Cargo.toml

```toml
[package]
name = "change-submission-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-itsm-sdk = { path = "../../crates/oya-itsm-sdk" }
oya-iam-sdk = { path = "../../crates/oya-iam-sdk" }
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
tokio = { version = "1.42", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## src/main.rs

```rust
use anyhow::Context;
use chrono::{Duration, Utc};
use oya_itsm_sdk::{
    ItsmClient, ChangeSubmitRequest, ChangeType, ChangeRiskClass, CmdbImpactRequest,
    ChangeAdvanceStateRequest, ChangeState, ChangeExecuteRequest, ChangeVerifyRequest,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use serde_json::json;
use tracing::{info, warn};

struct ChangeSubmissionWorker {
    itsm_client: ItsmClient,
    iam_client: IamClient,
    tenant_id: String,
}

impl ChangeSubmissionWorker {
    async fn submit_database_upgrade_change(&self) -> anyhow::Result<String> {
        // Step 1: query the CMDB for affected CIs
        let impact = self
            .itsm_client
            .cmdb_impact_analysis(CmdbImpactRequest {
                tenant_id: self.tenant_id.clone(),
                affected_ci_ids: vec!["ci_postgres_primary_prod_us_east".into()],
                hops: 3,
            })
            .await
            .context("cmdb impact analysis failed")?;
        info!(
            direct = impact.direct_cis.len(),
            one_hop = impact.one_hop_cis.len(),
            two_hop = impact.two_hop_cis.len(),
            services = impact.business_services_affected.len(),
            customers_estimated = impact.estimated_customer_impact,
            "CMDB impact analysis complete"
        );

        // Step 2: prepare the change submission
        let scheduled_start = Utc::now() + Duration::days(7);
        let scheduled_end = scheduled_start + Duration::hours(2);

        let submit_req = ChangeSubmitRequest {
            tenant_id: self.tenant_id.clone(),
            title: "Upgrade primary PostgreSQL 16.6 → 17.0".into(),
            description: "\
Quarterly database engine upgrade. Tested in dev for 2 weeks; staging for 1 week. \
No schema changes; pure engine upgrade. Replication-aware procedure used.".into(),
            change_type: ChangeType::Normal,
            risk_class: ChangeRiskClass::Medium,
            affected_ci_ids: vec!["ci_postgres_primary_prod_us_east".into()],
            scheduled_start,
            scheduled_end,
            execution_runbook_url:
                "https://github.com/oyatie/oyatie/blob/dev/microservices/cloud-data/runbooks/postgres-engine-upgrade.md".into(),
            rollback_plan: serde_json::to_string(&json!({
                "triggers": ["error_rate > 5x baseline for > 5 min", "replica lag > 30 s"],
                "procedure": [
                    "switch primary back to v16.6 via patroni failover",
                    "restore from 6:00 UTC snapshot if data corruption detected",
                    "rerun smoke tests"
                ],
                "verification": "error rate baseline + replica lag < 1 s",
                "communications": ["#cloud-data-oncall slack", "@cto", "status-page update"]
            }))?,
            verification_plan: "Run post-upgrade smoke test suite; verify replica lag normal; verify p99 query latency unchanged".into(),
            requester_principal_id: "user:db-platform@your-tenant.com".into(),
            tags: vec!["quarterly".into(), "postgresql".into()],
        };

        let submitted = self
            .itsm_client
            .submit_change(submit_req)
            .await
            .context("submit_change failed")?;
        info!(
            change_id = %submitted.change_id,
            risk_class = ?submitted.risk_class,
            cab_review_date = ?submitted.cab_scheduled_review,
            "change submitted for CAB review"
        );

        Ok(submitted.change_id)
    }

    async fn wait_for_cab_approval(&self, change_id: &str) -> anyhow::Result<()> {
        info!(change_id = change_id, "polling for CAB approval (timeout: 7 days)");
        for _ in 0..(7 * 24 * 60) {
            let status = self.itsm_client.get_change(change_id).await?;
            match status.state {
                ChangeState::CabApproved => {
                    info!(change_id = change_id, approver_count = status.cab_approver_count, "approved");
                    return Ok(());
                }
                ChangeState::CabRejected => {
                    return Err(anyhow::anyhow!("change rejected by CAB: {:?}", status.rejection_reason));
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            }
        }
        Err(anyhow::anyhow!("CAB approval timed out"))
    }

    async fn execute_change(&self, change_id: &str) -> anyhow::Result<()> {
        self.itsm_client
            .advance_state(ChangeAdvanceStateRequest {
                change_id: change_id.into(),
                to_state: ChangeState::InProgress,
                comment: Some("Starting upgrade per the runbook".into()),
            })
            .await?;
        // ... actual upgrade execution invoked by the runbook ...
        self.itsm_client
            .execute_change(ChangeExecuteRequest {
                change_id: change_id.into(),
                execution_evidence: serde_json::json!({
                    "command_log": "executed via runbook; full log at /tmp/upgrade-log.txt",
                    "duration_seconds": 1800,
                }),
            })
            .await?;
        Ok(())
    }

    async fn verify_change(&self, change_id: &str) -> anyhow::Result<()> {
        let verify = self
            .itsm_client
            .verify_change(ChangeVerifyRequest {
                change_id: change_id.into(),
                verification_evidence: serde_json::json!({
                    "smoke_test_pass": true,
                    "replica_lag_seconds": 0.4,
                    "p99_query_latency_unchanged": true,
                }),
            })
            .await?;
        if verify.passed {
            self.itsm_client
                .advance_state(ChangeAdvanceStateRequest {
                    change_id: change_id.into(),
                    to_state: ChangeState::Closed,
                    comment: Some("Verified post-change; closing".into()),
                })
                .await?;
            info!(change_id = change_id, "closed successfully");
        } else {
            warn!(change_id = change_id, "verification failed; rolling back");
            self.itsm_client
                .advance_state(ChangeAdvanceStateRequest {
                    change_id: change_id.into(),
                    to_state: ChangeState::BackedOut,
                    comment: Some("Verification failed; executed rollback".into()),
                })
                .await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("change-submission-worker")?;
    let itsm_client = ItsmClient::from_env()?;
    let iam_client = IamClient::from_env()?;
    let principal: Principal = iam_client.whoami().await?;

    let worker = ChangeSubmissionWorker {
        itsm_client,
        iam_client,
        tenant_id: principal.tenant_id.clone(),
    };

    let change_id = worker.submit_database_upgrade_change().await?;
    worker.wait_for_cab_approval(&change_id).await?;
    worker.execute_change(&change_id).await?;
    worker.verify_change(&change_id).await?;
    info!(change_id = %change_id, "change lifecycle complete");
    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"change-worker@tenant-acme",
    action in [
        Action::"itsm::cmdb::query",
        Action::"itsm::change::submit",
        Action::"itsm::change::read",
        Action::"itsm::change::advance_state",
        Action::"itsm::change::execute",
        Action::"itsm::change::verify"
    ],
    resource in Tenant::"tenant_acme"
);
```

## Compliance evidence emitted

Every change lifecycle event emits to audit-chain (`itsm::change::submitted`, `itsm::change::cab_review_started`, `itsm::change::cab_approved`, `itsm::change::executed`, `itsm::change::verified`, `itsm::change::closed`). This is the ITIL v4 + ISO 20000-1 + SOC 2 CC8.1 audit-trail.

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_ITSM_API=https://itsm-api.dev.<tenant>.oyatie.io \
OYA_IAM_API=https://iam-api.dev.<tenant>.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in the portal: portal → Changes → search for the change ID. Full lifecycle visible.

## Notes

- CMDB impact analysis is a synchronous call; expect 200-500 ms for typical scope (≤ 100 affected CIs).
- The change submission triggers a workflow that auto-routes to CAB based on risk class.
- CAB approval is asynchronous; in production, subscribe to the `itsm.change.cab_approved` topic instead of polling.
- The rollback_plan field is required for Normal + Emergency changes; substrate rejects submissions without one.
- Execution evidence + verification evidence are JSON-typed; submit any structured data your runbook produces.
