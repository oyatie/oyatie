---
doc_class: ReferenceImplementation
microservice: compliance
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Publish pack + evaluate multi-pack conflict via the compliance Rust SDK

A runnable example that:

1. Authenticates as a compliance owner principal.
2. Publishes a pack overlay version.
3. Activates multiple packs on a tenant.
4. Evaluates effective pack policy.
5. Triggers + resolves a multi-pack conflict.
6. Generates transparency report.
7. Creates a DSAR with multi-pack conflict resolution.
8. Verifies audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "compliance-pack-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-compliance-client = { path = "../../../../crates/oya-compliance-client" }
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
use oya_compliance_client::{
    ComplianceClient, ComplianceClientConfig,
    PackPublish, PackActivation, EffectivePolicyEvaluate,
    DsarCreate, DsarRequestClass,
    TransparencyReport,
    PackRuleSet, RuleRestrictionLevel,
};
use oya_cedar_client::CedarPrincipal;
use chrono::Utc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let principal = CedarPrincipal::from_env("COMPLIANCE_OWNER_JWT")?;
    let client = ComplianceClient::connect(ComplianceClientConfig {
        cell_endpoint: std::env::var("COMPLIANCE_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(60),
    }).await?;

    // 1. Publish a GDPR pack overlay (this would normally be done by the pack maintainer team).
    let gdpr_pack = client.pack_publish(PackPublish {
        pack_id: "gdpr".into(),
        version: "2026.05.20".into(),
        jurisdiction: "eu".into(),
        rules_path: "./packs/gdpr-rules.yaml".into(),
        cedar_policies_path: "./packs/gdpr-policies.cedar".into(),
        scorecard_refs_path: "./packs/gdpr-scorecards.yaml".into(),
        legal_basis: "EU GDPR 2016/679 + UK GDPR + EU AI Act 2024/1689 references".into(),
    }).await?;
    info!("GDPR pack published: id={}, version={}, rules_count={}",
          gdpr_pack.pack_id, gdpr_pack.version, gdpr_pack.rules_count);

    // 2. Publish HIPAA pack overlay.
    let hipaa_pack = client.pack_publish(PackPublish {
        pack_id: "hipaa".into(),
        version: "2026.05.20".into(),
        jurisdiction: "us-hipaa".into(),
        rules_path: "./packs/hipaa-rules.yaml".into(),
        cedar_policies_path: "./packs/hipaa-policies.cedar".into(),
        scorecard_refs_path: "./packs/hipaa-scorecards.yaml".into(),
        legal_basis: "45 CFR Parts 160 + 164 (HIPAA Privacy + Security + Breach Notification Rules)".into(),
    }).await?;
    info!("HIPAA pack published: id={}", hipaa_pack.pack_id);

    // 3. Activate both packs on the tenant.
    let gdpr_activation = client.tenant_pack_activate(PackActivation {
        tenant_id: "acme-corp".into(),
        pack_id: "gdpr".into(),
        version: "2026.05.20".into(),
        soak_seconds: 60,
    }).await?;
    info!("GDPR activated: audit_event_id={}", gdpr_activation.audit_event_id);

    let hipaa_activation = client.tenant_pack_activate(PackActivation {
        tenant_id: "acme-corp".into(),
        pack_id: "hipaa".into(),
        version: "2026.05.20".into(),
        soak_seconds: 60,
    }).await?;
    info!("HIPAA activated: audit_event_id={}", hipaa_activation.audit_event_id);

    // Wait for soak period.
    tokio::time::sleep(std::time::Duration::from_secs(65)).await;

    // 4. Evaluate effective policy for a specific decision.
    let policy_eval = client.effective_policy_evaluate(EffectivePolicyEvaluate {
        primitive: "data_retention".into(),
        action: "delete".into(),
        data_class: "PHI".into(),
        jurisdiction: "us".into(),
    }).await?;
    info!("Effective policy for PHI deletion: decision={}, winning_rule={}, restriction_level={}",
          policy_eval.decision, policy_eval.winning_rule_id, policy_eval.restriction_level);

    // 5. Create a DSAR with multi-pack conflict.
    let dsar = client.dsar_create(DsarCreate {
        request_class: DsarRequestClass::GdprErasureArt17,
        subject_id: "u-bob@acme-corp.com".into(),
        subject_verified_via: "passkey-aal3".into(),
        scope: "full".into(),
        justification: "Subject withdrew consent".into(),
        due_by: (Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
    }).await?;
    info!("DSAR created: id={}, state={}, audit_event_id={}",
          dsar.dsar_id, dsar.state, dsar.audit_event_id);

    // 6. Wait for cascade + conflict resolution to complete.
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let dsar_status = client.dsar_status(&dsar.dsar_id).await?;
    info!("DSAR status: state={}, data_class_decisions:", dsar_status.state);
    for decision in &dsar_status.data_class_decisions {
        info!("  - {}: erasure_decision={}, winning_rule={}, restriction_level={}",
              decision.data_class, decision.erasure_decision,
              decision.winning_rule_id, decision.restriction_level);
    }

    // 7. Generate transparency report for the PHI denial.
    let report = client.transparency_report_generate(TransparencyReport {
        conflict_id: dsar_status.transparency_report_ref
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no transparency report"))?
            .clone(),
        include_legal_basis_all_packs: true,
        include_historical_precedent: true,
        include_subject_appeal_pathway: true,
    }).await?;
    info!("Transparency report: winning_pack={}, winning_step={}, legal_basis={}",
          report.winning_pack, report.winning_step, report.winning_legal_basis);

    // 8. Finalize the DSAR + sign + deliver bundle.
    let finalized = client.dsar_finalize(
        &dsar.dsar_id,
        "./dsar-bundle.zip",
    ).await?;
    info!("DSAR finalized: bundle_path={}, bundle_size_bytes={}, audit_event_id={}",
          finalized.bundle_path, finalized.bundle_size_bytes, finalized.audit_event_id);

    Ok(())
}
```

## Expected output (against a paid tenant_class cell)

```
INFO GDPR pack published: id=gdpr, version=2026.05.20, rules_count=247
INFO HIPAA pack published: id=hipaa
INFO GDPR activated: audit_event_id=ae_comp_gdpr_activated_001
INFO HIPAA activated: audit_event_id=ae_comp_hipaa_activated_001
INFO Effective policy for PHI deletion: decision=deny, winning_rule=rule_hipaa_530_minimum_retention, restriction_level=10
INFO DSAR created: id=dsar_acme_001, state=collecting, audit_event_id=ae_comp_dsar_created_001
INFO DSAR status: state=completed, data_class_decisions:
INFO   - PII_SENSITIVE: erasure_decision=permit, winning_rule=rule_gdpr_art17_001, restriction_level=5
INFO   - PII_FINANCIAL: erasure_decision=permit, winning_rule=rule_gdpr_art17_001, restriction_level=5
INFO   - PII_GENERAL: erasure_decision=permit, winning_rule=rule_gdpr_art17_001, restriction_level=5
INFO   - PHI: erasure_decision=deny, winning_rule=rule_hipaa_530_minimum_retention, restriction_level=10
INFO Transparency report: winning_pack=hipaa, winning_step=1, legal_basis=45 CFR § 164.530(j) HIPAA Privacy Rule retention requirement
INFO DSAR finalized: bundle_path=./dsar-bundle.zip, bundle_size_bytes=248000000, audit_event_id=ae_comp_dsar_completed_001
```

## HTTP alternative (curl)

```sh
# 1. Publish pack
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/packs \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "pack_id":"gdpr",
        "version":"2026.05.20",
        "jurisdiction":"eu",
        "rules_yaml_b64":"<base64 YAML rules>",
        "cedar_policies_b64":"<base64 Cedar policies>",
        "scorecard_refs_yaml_b64":"<base64 scorecard refs>",
        "legal_basis":"EU GDPR 2016/679"
    }'

# 2. Activate pack on tenant
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/tenants/acme-corp/packs/gdpr/activate \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "version":"2026.05.20",
        "soak_seconds":60
    }'

# 3. Evaluate effective policy
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/tenants/acme-corp/effective-policy/evaluate \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "primitive":"data_retention",
        "action":"delete",
        "data_class":"PHI",
        "jurisdiction":"us"
    }'

# 4. Create DSAR
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/dsar \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "request_class":"gdpr_erasure_art17",
        "subject_id":"u-bob@acme-corp.com",
        "subject_verified_via":"passkey-aal3",
        "scope":"full",
        "due_by":"2026-06-19T00:00:00Z"
    }'

# 5. Get DSAR status
curl -X GET https://compliance.prod-us-east-1.oyatie.local/v1/compliance/dsar/dsar_acme_001 \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT"

# 6. Generate transparency report
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/transparency-reports \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "conflict_id":"cf_acme_001",
        "include_legal_basis_all_packs":true,
        "include_historical_precedent":true,
        "include_subject_appeal_pathway":true
    }'

# 7. Pack hotfix
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/packs/gdpr/hotfixes \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "base_version":"2026.05.20",
        "hotfix_version":"2026.05.20-hotfix-1",
        "rules_yaml_b64":"<base64 updated rules>",
        "reason":"EU Commission published clarifying guidance on Art 32",
        "skip_soak_check":true
    }'

# 8. Regulator request evaluate
curl -X POST https://compliance.prod-us-east-1.oyatie.local/v1/compliance/regulator-requests/{request_id}/evaluate \
    -H "Authorization: Bearer $COMPLIANCE_OWNER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "request_class":"hhs_breach_notification_followup",
        "regulator_jurisdiction":"us-hipaa",
        "evidence_request":"breach-event-2026-05-10-evidence-bundle"
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `compliance::*` permission |
| `pack_version_immutable` | 422 | No | Cannot modify published pack; create new version |
| `pack_publish_missing_legal_basis` | 422 | No | Pack rules must cite legal authority |
| `pack_activation_baa_required` | 422 | No | HIPAA pack requires BAA evidence |
| `pack_soak_period_active` | 503 | Yes (after soak) | Wait for 60s soak |
| `effective_policy_projection_stale` | 503 | Yes (after refresh) | Wait for projection convergence |
| `pack_conflict_hard_stop` | 403 | No | Hard-stop rule blocks the action (e.g., HIPAA blocks PHI erasure) |
| `dsar_subject_verification_required` | 401 | No | Subject must verify identity (passkey/etc.) |
| `dsar_pack_residency_violation` | 403 | No | DSAR cross-jurisdiction transfer requires Art 49 evidence |
| `tenant_policy_cannot_weaken_regulator_floor` | 403 | No | Per ADR-COMP-001 § Cedar forbid |
| `pack_hotfix_emergency_reason_required` | 422 | No | Hotfix without soak requires emergency reason |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `pack_publish` | `compliance.pack.published.v1` |
| `pack_activate` | `compliance.pack.activated.v1` |
| `pack_hotfix` | `compliance.pack.hotfix.published.v1` |
| `effective_policy_evaluate` | `compliance.effective-policy.evaluated.v1` |
| `effective_policy_changed` | `compliance.effective-policy.changed.v1` |
| `pack_conflict_detected` | `compliance.pack-conflict.detected.v1` |
| `pack_conflict_resolved` | `compliance.pack-conflict.resolved.v1` |
| `dsar_create` | `compliance.dsar.created.v1` |
| `dsar_cascade_started` | `compliance.dsar.cascade.started.v1` |
| `dsar_completed` | `compliance.dsar.completed.v1` |
| `dpia_initiate` | `compliance.dpia.initiated.v1` |
| `transparency_report_generated` | `compliance.transparency-report.generated.v1` |
| `regulator_request_evaluate` | `compliance.regulator-request.evaluated.v1` |
| `breach_notification_started` | `compliance.breach-notification.started.v1` |
| Cedar deny anywhere | `compliance.cedar.denied.v1` |

## Where this file lives

`microservices/compliance/reference-implementations/pack-publish-and-conflict-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/compliance/reference-implementations/pack-example/` once `oya-compliance-client` ships.
