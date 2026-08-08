---
doc_class: ReferenceImplementation
microservice: tenancy
language: rust
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Reference — Provision an oyatie tenant programmatically via the Rust SDK

Goal: from your platform-operator automation, programmatically provision a B2B tenant with KYB verification, pack auto-application, DR pairing, and audit-chain anchoring; then add an initial admin user and verify the tenant is operational.

## Cargo.toml

```toml
[package]
name = "tenant-provisioning-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-tenancy-sdk = { path = "../../crates/oya-tenancy-sdk" }
oya-iam-sdk = { path = "../../crates/oya-iam-sdk" }
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
tokio = { version = "1.42", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## src/main.rs

```rust
use anyhow::{anyhow, Context};
use oya_tenancy_sdk::{
    TenancyClient, TenantProvisionRequest, AudienceType, TenantProvisionStatus,
    DrPairingConfig, SubScopeCreateRequest, TenantInitialAdminInviteRequest,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use tracing::{info, warn};

struct TenantProvisioningWorker {
    tenancy_client: TenancyClient,
    iam_client: IamClient,
}

impl TenantProvisioningWorker {
    async fn provision_b2b_tenant_with_hipaa(
        &self,
        deal: TenantDeal,
    ) -> anyhow::Result<String> {
        let req = TenantProvisionRequest {
            legal_name: deal.legal_name.clone(),
            audience_type: AudienceType::B2bOrganization,
            country_code: deal.country_code.clone(),
            industry_code: Some(deal.industry_code.clone()),
            estimated_revenue_usd: Some(deal.estimated_revenue_usd),
            tax_id: Some(deal.tax_id.clone()),
            data_classes_attested: deal.data_classes.clone(),
            dpo_email: deal.dpo_email.clone(),
            privacy_officer_email: deal.privacy_officer_email.clone(),
            primary_contact_email: deal.primary_contact_email.clone(),
            data_residency_region: deal.data_residency_region.clone(),
            dr_pairing: Some(DrPairingConfig {
                dr_region: deal.dr_region.clone(),
                rpo_target_seconds: 60,
                rto_target_minutes: 15,
            }),
            requested_packs: vec![],
            external_id_mapping: None,
            requester_principal_id: "user:provisioning-worker@your-platform.com".into(),
        };
        let provisioning = self
            .tenancy_client
            .provision_tenant(req)
            .await
            .context("provision_tenant failed")?;
        info!(
            workflow_id = %provisioning.workflow_id,
            "provisioning workflow started"
        );

        // Poll for completion (KYB + pack-apply + DB provision + IAM binding can take 30-90 min)
        let mut last_status: Option<TenantProvisionStatus> = None;
        for _ in 0..(90 * 12) {
            let status = self
                .tenancy_client
                .get_provisioning_status(&provisioning.workflow_id)
                .await?;
            if last_status.as_ref().map(|s| &s.current_step) != Some(&status.current_step) {
                info!(step = %status.current_step, "step transition");
            }
            last_status = Some(status.clone());
            match status.state.as_str() {
                "completed" => {
                    info!(
                        tenant_id = %status.tenant_id.unwrap(),
                        packs_applied = ?status.packs_applied,
                        "tenant provisioned"
                    );
                    return Ok(status.tenant_id.unwrap());
                }
                "failed" => {
                    return Err(anyhow!("provisioning failed: {:?}", status.failure_reason));
                }
                "awaiting_baa_signature" => {
                    info!("awaiting BAA signature; tenant must sign via the email link before provisioning resumes");
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(anyhow!("provisioning timed out"))
    }

    async fn create_sub_scopes(
        &self,
        tenant_id: &str,
        sub_scope_names: &[&str],
    ) -> anyhow::Result<()> {
        for name in sub_scope_names {
            self.tenancy_client
                .create_sub_scope(SubScopeCreateRequest {
                    tenant_id: tenant_id.into(),
                    parent_scope: "root".into(),
                    name: (*name).into(),
                })
                .await?;
            info!(tenant_id = tenant_id, sub_scope = name, "sub-scope created");
        }
        Ok(())
    }

    async fn invite_initial_admin(
        &self,
        tenant_id: &str,
        admin_email: &str,
    ) -> anyhow::Result<()> {
        self.tenancy_client
            .invite_initial_admin(TenantInitialAdminInviteRequest {
                tenant_id: tenant_id.into(),
                email: admin_email.into(),
                role: "tenant_admin".into(),
            })
            .await?;
        info!(tenant_id = tenant_id, admin_email = admin_email, "admin invited");
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TenantDeal {
    legal_name: String,
    country_code: String,
    industry_code: String,
    estimated_revenue_usd: f64,
    tax_id: String,
    data_classes: Vec<String>,
    dpo_email: String,
    privacy_officer_email: String,
    primary_contact_email: String,
    data_residency_region: String,
    dr_region: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("tenant-provisioning-worker")?;
    let tenancy_client = TenancyClient::from_env()?;
    let iam_client = IamClient::from_env()?;

    let worker = TenantProvisioningWorker {
        tenancy_client,
        iam_client,
    };

    let deal = TenantDeal {
        legal_name: "MedCenter LLC".into(),
        country_code: "US".into(),
        industry_code: "healthcare-provider".into(),
        estimated_revenue_usd: 850_000_000.0,
        tax_id: "12-3456789".into(),
        data_classes: vec!["phi-us".into()],
        dpo_email: "dpo@medcenter.com".into(),
        privacy_officer_email: "privacy@medcenter.com".into(),
        primary_contact_email: "ceo@medcenter.com".into(),
        data_residency_region: "us-east-1".into(),
        dr_region: "us-west-2".into(),
    };

    let tenant_id = worker.provision_b2b_tenant_with_hipaa(deal).await?;
    worker
        .create_sub_scopes(&tenant_id, &["clinical", "billing", "admin"])
        .await?;
    worker.invite_initial_admin(&tenant_id, "ceo@medcenter.com").await?;

    info!(tenant_id = %tenant_id, "tenant fully provisioned");
    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"provisioning-worker@your-platform.com",
    action in [
        Action::"tenancy::tenant::provision",
        Action::"tenancy::tenant::read",
        Action::"tenancy::sub_scope::create",
        Action::"tenancy::invite::send",
        Action::"tenancy::dr_pair::configure"
    ],
    resource in Platform::"your-platform"
);
```

## Compliance evidence emitted

Every provisioning step emits to audit-chain:

```json
{
    "event_class": "tenancy::tenant::provisioned",
    "tenant_id": "tenant_medcenter_001",
    "workflow_id": "prov_wf_01HXYZ...",
    "audience_type": "b2b-organization",
    "country_code": "US",
    "data_residency_region": "us-east-1",
    "dr_region": "us-west-2",
    "packs_applied": ["HIPAA-Provider"],
    "kyb_evidence_id": "kyb_01HXYZ...",
    "baa_contract_id": "contract_01HXYZ...",
    "provisioner_principal_id": "user:provisioning-worker@your-platform.com"
}
```

This is the chain-of-custody for the tenant lifecycle.

## Run + verify

```sh
OYA_TENANCY_API=https://tenancy-api.dev.<platform>.oyatie.io \
OYA_IAM_API=https://iam-api.dev.<platform>.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in portal: portal → Tenants → search "MedCenter". See the new tenant + active sub-scopes + pending admin invitation.

## Notes

- The provisioning workflow is asynchronous; expect 30-90 min total elapsed (longer if BAA signature takes time).
- The `data_classes_attested` field drives pack auto-application; the substrate evaluates each pack's eligibility criteria and applies matching packs.
- `dr_region` MUST be in the same compliance zone as `data_residency_region` (us-east-1 + us-west-2 OK; us-east-1 + eu-central-1 BLOCKED unless tenant signs the cross-region transfer consent).
- For bulk provisioning (e.g. migrating 1 000 tenants from another platform), use the bulk-provision API which parallelizes provisioning and returns a single workflow ID covering the batch.
- For sovereign-pack tenants (paid tenant_class regulated-pack overlay), the provisioning workflow includes additional dual-control approval steps before completion.
