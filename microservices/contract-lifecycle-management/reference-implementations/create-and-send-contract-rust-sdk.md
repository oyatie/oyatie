---
doc_class: ReferenceImplementation
microservice: contract-lifecycle-management
language: rust
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Reference — Create + send a contract via the oyatie CLM Rust SDK

Goal: from a tenant's back-office worker, create a Master Service Agreement contract programmatically, attach DPA + BAA overlays based on customer data, send for QES signature, and wait for the signed event.

## Cargo.toml

```toml
[package]
name = "msa-creator-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-clm-sdk = { path = "../../crates/oya-clm-sdk" }
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
use anyhow::{anyhow, Context};
use chrono::Utc;
use oya_clm_sdk::{
    ClmClient, ContractCreateRequest, ContractSendRequest, ContractWaitForSignedRequest,
    ContractTypeRef, SignatureClass,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use serde_json::json;
use tracing::{info, warn};

struct MsaCreatorWorker {
    clm_client: ClmClient,
    iam_client: IamClient,
    tenant_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CustomerRecord {
    legal_name: String,
    registered_address: String,
    country_code: String,
    data_classes: Vec<String>,
    signer_email: String,
    signer_name: String,
    dpo_email: Option<String>,
    privacy_officer_email: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DealRecord {
    customer: CustomerRecord,
    service_description: String,
    fees_eur: f64,
    term_months: u32,
    governing_law: String,
    requester_principal_id: String,
}

impl MsaCreatorWorker {
    async fn create_and_send(&self, deal: DealRecord) -> anyhow::Result<String> {
        // Step 1: build the contract create payload
        let create_req = ContractCreateRequest {
            tenant_id: self.tenant_id.clone(),
            contract_type: ContractTypeRef::new("Master Service Agreement", "v1.2.0"),
            requester_principal_id: deal.requester_principal_id.clone(),
            initial_field_values: json!({
                "customer": {
                    "legal_name": deal.customer.legal_name,
                    "registered_address": deal.customer.registered_address,
                    "country_code": deal.customer.country_code,
                    "data_classes": deal.customer.data_classes,
                    "dpo_email": deal.customer.dpo_email,
                    "privacy_officer_email": deal.customer.privacy_officer_email,
                    "signer": {
                        "email": deal.customer.signer_email,
                        "name": deal.customer.signer_name,
                    },
                },
                "services": {
                    "description": deal.service_description,
                    "fees_eur": deal.fees_eur,
                },
                "effective_date": Utc::now().date_naive().to_string(),
                "term_months": deal.term_months,
                "governing_law": deal.governing_law,
            }),
            workflow_name: "MSA standard".into(),
        };

        let created = self
            .clm_client
            .create_contract(create_req)
            .await
            .context("create_contract failed")?;

        info!(
            contract_id = %created.contract_id,
            overlays_attached = ?created.attached_overlays,
            signature_class = ?created.derived_signature_class,
            "contract created"
        );

        // Step 2: validate overlays
        let expected_overlays = self.compute_expected_overlays(&deal);
        if created.attached_overlays != expected_overlays {
            warn!(
                expected = ?expected_overlays,
                got = ?created.attached_overlays,
                "overlay mismatch"
            );
        }

        // Step 3: advance through workflow (assume Draft -> Review -> Approve -> Send)
        self.clm_client
            .advance_workflow(&created.contract_id, "Review")
            .await?;
        self.clm_client
            .advance_workflow(&created.contract_id, "Approve")
            .await?;

        // Step 4: send for signature
        let send_req = ContractSendRequest {
            contract_id: created.contract_id.clone(),
            signature_class_override: None,
            counterparty_signer_email: deal.customer.signer_email.clone(),
            counterparty_signer_name: deal.customer.signer_name.clone(),
            cc_emails: vec![deal.requester_principal_id.clone()],
            expiry_days: 30,
        };

        let sent = self
            .clm_client
            .send_for_signature(send_req)
            .await
            .context("send_for_signature failed")?;

        info!(
            contract_id = %created.contract_id,
            envelope_id = %sent.envelope_id,
            provider = %sent.provider,
            "contract sent for signature"
        );

        // Step 5: wait for signed (async; this can take days; in production we'd subscribe to the event)
        let signed = self
            .clm_client
            .wait_for_signed(ContractWaitForSignedRequest {
                contract_id: created.contract_id.clone(),
                timeout_seconds: 30 * 86_400,
            })
            .await?;

        info!(
            contract_id = %created.contract_id,
            signed_at = %signed.signed_at,
            counterparty_ip = %signed.counterparty_ip,
            "contract signed"
        );

        Ok(created.contract_id)
    }

    fn compute_expected_overlays(&self, deal: &DealRecord) -> Vec<String> {
        let mut overlays = Vec::new();
        if deal.customer.data_classes.iter().any(|c| c == "personal-data-eu") {
            overlays.push("dpa-gdpr".to_string());
        }
        if deal.customer.data_classes.iter().any(|c| c == "phi-us") {
            overlays.push("baa-hipaa".to_string());
        }
        if deal.customer.data_classes.iter().any(|c| c == "personal-data-kr") {
            overlays.push("kr-pipa-overseas-transfer".to_string());
        }
        overlays
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("msa-creator-worker")?;

    let clm_client = ClmClient::from_env()?;
    let iam_client = IamClient::from_env()?;
    let principal: Principal = iam_client.whoami().await?;

    let worker = MsaCreatorWorker {
        clm_client,
        iam_client,
        tenant_id: principal.tenant_id.clone(),
    };

    let deal = DealRecord {
        customer: CustomerRecord {
            legal_name: "Beispiel GmbH".into(),
            registered_address: "Friedrichstraße 1, 10117 Berlin, Germany".into(),
            country_code: "DE".into(),
            data_classes: vec!["personal-data-eu".into()],
            signer_email: "signer@beispiel.de".into(),
            signer_name: "Dr. Hans Müller".into(),
            dpo_email: Some("dpo@beispiel.de".into()),
            privacy_officer_email: None,
        },
        service_description: "Cloud computing services per Order Form #1".into(),
        fees_eur: 60_000.0,
        term_months: 36,
        governing_law: "DE-Berlin".into(),
        requester_principal_id: "user:requester@your-tenant.com".into(),
    };

    let contract_id = worker.create_and_send(deal).await?;
    info!(contract_id = %contract_id, "done");

    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"msa-creator-worker@tenant-acme",
    action in [
        Action::"contract::create",
        Action::"contract::read",
        Action::"contract::advance_workflow",
        Action::"contract::send_for_signature",
        Action::"contract::wait_for_signed"
    ],
    resource in Tenant::"tenant_acme"
);
```

## Compliance evidence emitted

Every contract lifecycle event emits to `audit-chain`:

```json
{
    "event_class": "contract::created",
    "tenant_id": "tenant_acme",
    "contract_id": "contract_01HXYZ...",
    "contract_type": "Master Service Agreement",
    "contract_type_version": "v1.2.0",
    "attached_overlays": ["dpa-gdpr"],
    "derived_signature_class": "QES",
    "initial_field_values_hash": "0xABCD...",
    "requester_principal_id": "user:requester@your-tenant.com"
}
```

Plus events for each workflow advance + signature send + signature received. The full chain provides regulator-grade evidence of the contract lifecycle.

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_CLM_API=https://clm-api.dev.oyatie.io \
OYA_IAM_API=https://iam-api.dev.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

In a second terminal, watch the contract in the portal:
- Open `https://clm-admin.<your-tenant>.oyatie.io`
- Search for the contract by its newly-generated ID.
- Observe the workflow advancing through stages.
- Once the counterparty signs (via DocuSign email), the portal shows "Signed".

Verify in `audit-chain`:
```sh
oya audit-chain query --tenant tenant_acme \
    --event-class "contract::*" \
    --contract-id <contract_id>
```

## Notes

- The example sends the contract to a real email address — for testing, use `signature-test@<your-domain>.com` or DocuSign sandbox accounts.
- Production: the `wait_for_signed` should be replaced with a Pulsar consumer subscribing to the `contract.signed` topic — polling is wasteful for contracts that take days to sign.
- For QES (eIDAS Art. 28) signatures, the counterparty must have an EU Trust List-listed eID (typically national ID smartcard or remote QES service). DocuSign EU Premium Tier supports this; the substrate routes automatically based on the derived signature class.
- For high-volume tenants, batch contract creation via `create_contracts_bulk` (up to 1 000 contracts/batch with retries).
