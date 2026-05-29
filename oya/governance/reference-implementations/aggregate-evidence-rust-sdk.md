---
doc_class: ReferenceImplementation
microservice: governance
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Aggregate evidence + evaluate retention via the governance Rust SDK

A runnable example that:

1. Authenticates as a governance auditor principal.
2. Subscribes a tenant to multiple packs.
3. Creates retention policies with restriction levels.
4. Evaluates retention for a sample event.
5. Queries the governance projection for evidence.
6. Generates a transparency report.
7. Creates an expiring bypass grant.
8. Verifies audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "governance-aggregate-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-governance-client = { path = "../../../../crates/oya-governance-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
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
use oya_governance_client::{
    GovernanceClient, GovernanceClientConfig,
    RetentionPolicyCreate, RetentionEvaluate, EventClass, DataClass,
    EvidenceQuery, EvidenceQueryFilter,
    TransparencyReport,
    BypassGrantCreate,
};
use oya_cedar_client::CedarPrincipal;
use chrono::Utc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let principal = CedarPrincipal::from_env("GOVERNANCE_AUDITOR_JWT")?;
    let client = GovernanceClient::connect(GovernanceClientConfig {
        cell_endpoint: std::env::var("GOVERNANCE_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(60),
    }).await?;

    // 1. Create retention policy for the default pack.
    let default_policy = client.retention_policy_create(RetentionPolicyCreate {
        pack_id: "default".into(),
        event_class: EventClass::DriveFileUploadedV1,
        data_class: DataClass::PiiSensitive,
        minimum_duration_iso: "P1Y".into(),
        max_duration_iso: "P5Y".into(),
        restriction_level: 3,
        legal_basis: "Default pack baseline".into(),
        legal_hold_behavior: "block-delete".into(),
        delete_approval_class: "dual_approval".into(),
    }).await?;
    info!("Default retention policy created: {}", default_policy.policy_id);

    // 2. Create stricter HIPAA retention policy.
    let hipaa_policy = client.retention_policy_create(RetentionPolicyCreate {
        pack_id: "hipaa".into(),
        event_class: EventClass::DriveFileUploadedV1,
        data_class: DataClass::PiiSensitive,
        minimum_duration_iso: "P6Y".into(),
        max_duration_iso: "P6Y".into(),
        restriction_level: 8,
        legal_basis: "45 CFR § 164.530(j); HIPAA Privacy Rule 6-y documentation retention".into(),
        legal_hold_behavior: "block-delete".into(),
        delete_approval_class: "regulator_attested".into(),
    }).await?;
    info!("HIPAA retention policy created: {}", hipaa_policy.policy_id);

    // 3. Evaluate retention for a sample event (suppose drive emitted one).
    let retention_decision = client.retention_evaluate(RetentionEvaluate {
        source_event_id: "ae_drive_file_uploaded_alice_001".into(),
        event_class: EventClass::DriveFileUploadedV1,
        data_class: DataClass::PiiSensitive,
    }).await?;
    info!("Retention decision: winning_rule={}, effective_until={}, reason={}",
          retention_decision.winning_rule_id,
          retention_decision.effective_retention_until,
          retention_decision.reason_code);

    // 4. Query the governance projection for evidence.
    let evidence_query = client.evidence_query(EvidenceQuery {
        filter: EvidenceQueryFilter {
            pack_id: Some("hipaa".into()),
            microservice: Some("drive".into()),
            event_class_prefix: Some("drive.file.".into()),
            from_time: Some("2026-05-01T00:00:00Z".into()),
            to_time: Some("2026-05-20T23:59:59Z".into()),
            tenant_token: Some(client.tenant_token()),
            principal_id: Some("u-alice@acme-corp.com".into()),
        },
        limit: 100,
        sort: "occurred_at_desc".into(),
    }).await?;
    info!("Evidence query returned {} events", evidence_query.events.len());

    // 5. Generate transparency report for the retention conflict.
    let report = client.transparency_report_generate(TransparencyReport {
        source_event_id: "ae_drive_file_uploaded_alice_001".into(),
        include_legal_basis: true,
        include_historical_precedent: true,
    }).await?;
    info!("Transparency report: winning_pack={}, restriction_level_winning={}, legal_basis_winning={}",
          report.winning_pack, report.restriction_level_winning, report.legal_basis_winning);

    // 6. Create a time-bounded bypass grant for a CI lane.
    let bypass = client.bypass_grant_create(BypassGrantCreate {
        lane_id: "docs-coverage".into(),
        action: "skip-docs-coverage-check".into(),
        reason: "Hotfix shipping; auto-generated file false-positive".into(),
        expires_at: (Utc::now() + chrono::Duration::days(1)).to_rfc3339(),
        approved_by: vec![
            "u-team-lead@acme-corp.com".into(),
            "u-engineering-director@acme-corp.com".into(),
        ],
    }).await?;
    info!("Bypass grant: id={}, expires_at={}, audit_event_id={}",
          bypass.grant_id, bypass.expires_at, bypass.audit_event_id);

    Ok(())
}
```

## Expected output (against a paid tenant_class cell)

```
INFO Default retention policy created: rp_default_drive_001
INFO HIPAA retention policy created: rp_hipaa_drive_001
INFO Retention decision: winning_rule=rp_hipaa_drive_001, effective_until=2032-05-20T14:32:17Z, reason=higher_restriction_wins
INFO Evidence query returned 84 events
INFO Transparency report: winning_pack=hipaa, restriction_level_winning=8, legal_basis_winning=45 CFR § 164.530(j); HIPAA Privacy Rule 6-y documentation retention
INFO Bypass grant: id=bg_acme_001, expires_at=2026-05-21T14:32:17Z, audit_event_id=ae_gov_bypass_granted_001
```

## HTTP alternative (curl)

```sh
# 1. Create retention policy
curl -X POST https://governance.prod-us-east-1.oyatie.local/v1/governance/retention/policies \
    -H "Authorization: Bearer $GOVERNANCE_AUDITOR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "pack_id":"hipaa",
        "event_class":"drive.file.uploaded.v1",
        "data_class":"PII_SENSITIVE",
        "minimum_duration":"P6Y",
        "max_duration":"P6Y",
        "restriction_level":8,
        "legal_basis":"45 CFR § 164.530(j)",
        "legal_hold_behavior":"block-delete",
        "delete_approval_class":"regulator_attested"
    }'

# 2. Evaluate retention
curl -X POST https://governance.prod-us-east-1.oyatie.local/v1/governance/retention/evaluate \
    -H "Authorization: Bearer $GOVERNANCE_AUDITOR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "source_event_id":"ae_drive_file_uploaded_alice_001",
        "event_class":"drive.file.uploaded.v1",
        "data_class":"PII_SENSITIVE"
    }'

# 3. Query evidence
curl -X GET "https://governance.prod-us-east-1.oyatie.local/v1/governance/evidence?pack_id=hipaa&microservice=drive&from=2026-05-01T00:00:00Z&to=2026-05-20T23:59:59Z&limit=100" \
    -H "Authorization: Bearer $GOVERNANCE_AUDITOR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp"

# 4. Replay aggregation
curl -X POST https://governance.prod-us-east-1.oyatie.local/v1/governance/aggregation/replay \
    -H "Authorization: Bearer $GOVERNANCE_OPERATOR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "from_event_time":"2026-05-01T00:00:00Z",
        "to_event_time":"2026-05-20T23:59:59Z",
        "projection_version":"governance-evidence-index-v1",
        "partition_topic":"governance.evidence.partition.prod-us-east-1.shard-001",
        "replay_rate_per_sec":5000
    }'

# 5. Create bypass grant
curl -X POST https://governance.prod-us-east-1.oyatie.local/v1/governance/bypass-grants \
    -H "Authorization: Bearer $GOVERNANCE_ADMIN_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "lane_id":"docs-coverage",
        "action":"skip-docs-coverage-check",
        "reason":"Hotfix shipping",
        "expires_at":"2026-05-21T14:32:17Z",
        "approved_by":["u-team-lead@acme-corp.com","u-engineering-director@acme-corp.com"]
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `governance::*` permission |
| `retention_shorten_requires_dual_approval` | 403 | No | Provide second co-approver |
| `projection_freshness_red` | 503 | Yes (after replay) | Destructive op denied; replay needed |
| `legal_hold_active` | 423 | No | Cannot delete during hold |
| `pack_set_invalid` | 422 | No | Tenant not subscribed to pack |
| `replay_throughput_exceeded` | 429 | Yes (auto, backoff) | Replay rate-limited; will retry |
| `bypass_grant_expiry_exceeds_pack_max` | 422 | No | Reduce expiry; pack maximum is typically 24h |
| `source_hash_verification_failed` | 422 | No | Audit-chain anchor mismatch; investigate |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `retention_policy_create` | `governance.retention.policy.changed.v1` |
| `retention_evaluate` | `governance.retention.decision.evaluated.v1` |
| `retention_policy_shorten` | `governance.retention.policy.shortened.v1` |
| `evidence_query` | `governance.evidence.queried.v1` |
| `aggregation_replay_start` | `governance.aggregation.replay.started.v1` |
| `aggregation_replay_complete` | `governance.aggregation.replay.completed.v1` |
| `bypass_grant_create` | `governance.bypass.granted.v1` |
| `bypass_grant_revoke` | `governance.bypass.revoked.v1` |
| `bypass_grant_expire` | `governance.bypass.expired.v1` |
| `partition_stale` | `governance.aggregation.partition.stale.v1` |
| Cedar deny anywhere | `governance.cedar.denied.v1` |

## Where this file lives

`microservices/governance/reference-implementations/aggregate-evidence-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/governance/reference-implementations/aggregate-example/` once `oya-governance-client` ships.
