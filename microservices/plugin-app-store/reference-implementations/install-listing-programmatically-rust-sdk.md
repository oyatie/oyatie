---
doc_class: ReferenceImplementation
microservice: plugin-app-store
language: rust
related_adrs: [ADR-0316, ADR-0249]
date: 2026-05-20
doc_status: published
---

# Reference — Install a marketplace listing programmatically (Rust SDK)

Goal: from a tenant's automation worker, search the marketplace for a listing matching a use case, validate its security + permissions, install it with explicit consent, and verify the installation succeeded.

## Cargo.toml

```toml
[package]
name = "marketplace-installer"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-marketplace-sdk = { path = "../../crates/oya-marketplace-sdk" }
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
use anyhow::Context;
use oya_marketplace_sdk::{
    MarketplaceClient, ListingSearchRequest, ListingSearchResult, ListingDetailRequest,
    InstallRequest, InstallVerifyRequest, ConsentChallengeResponse,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use tracing::{error, info, warn};

struct MarketplaceInstaller {
    marketplace_client: MarketplaceClient,
    iam_client: IamClient,
    tenant_id: String,
}

impl MarketplaceInstaller {
    async fn find_and_install(
        &self,
        search_query: &str,
        category: &str,
        max_pricing_per_month_usd: f64,
    ) -> anyhow::Result<String> {
        // Step 1: search for matching listings
        let search_req = ListingSearchRequest {
            query: search_query.into(),
            category: Some(category.into()),
            max_pricing_per_month_usd: Some(max_pricing_per_month_usd),
            min_rating: Some(4.0),
            limit: 10,
        };
        let results = self
            .marketplace_client
            .search(search_req)
            .await
            .context("listing search failed")?;
        info!(
            results_count = results.listings.len(),
            "search results"
        );

        if results.listings.is_empty() {
            return Err(anyhow::anyhow!("no matching listings"));
        }

        // Step 2: filter + select the top candidate
        let candidate = results.listings.into_iter()
            .filter(|l| l.security_status == "clean" && l.license_compliant)
            .next()
            .ok_or_else(|| anyhow::anyhow!("no candidates meet security + license criteria"))?;

        info!(
            slug = %candidate.slug,
            publisher = %candidate.publisher,
            rating = candidate.rating,
            installs = candidate.install_count,
            "selected candidate"
        );

        // Step 3: fetch full detail to inspect permissions + dependencies
        let detail = self
            .marketplace_client
            .listing_detail(ListingDetailRequest { listing_id: candidate.id.clone() })
            .await?;

        // Step 4: validate the requested permissions are acceptable
        let acceptable_permissions = vec![
            "docs::document::read".to_string(),
            "docs::document::comment".to_string(),
        ];
        for perm in &detail.permissions_requested {
            if !acceptable_permissions.contains(perm) {
                warn!(permission = perm, "listing requests permission not in allowlist");
                return Err(anyhow::anyhow!(
                    "listing {} requests {} which is not pre-approved",
                    candidate.slug, perm
                ));
            }
        }

        // Step 5: review external dependencies for data-transfer concerns
        for dep in &detail.external_dependencies {
            info!(
                name = %dep.name,
                url = %dep.url,
                data_sent = ?dep.data_sent,
                "external dependency"
            );
        }

        // Step 6: install with explicit consent
        let install_req = InstallRequest {
            tenant_id: self.tenant_id.clone(),
            listing_id: candidate.id.clone(),
            version: candidate.latest_version.clone(),
            pricing_tier: "starter".into(),
            consent_response: ConsentChallengeResponse {
                permissions_consented: detail.permissions_requested.clone(),
                external_dependencies_acknowledged: true,
                privacy_policy_read: true,
                terms_of_service_accepted: true,
            },
            installer_principal_id: "user:tenant-admin@your-tenant.com".into(),
        };
        let installed = self
            .marketplace_client
            .install(install_req)
            .await
            .context("install failed")?;
        info!(
            install_id = %installed.install_id,
            artifact_url = %installed.artifact_url,
            "install initiated"
        );

        // Step 7: poll for install completion
        for _ in 0..30 {
            let status = self.marketplace_client.install_status(&installed.install_id).await?;
            match status.state.as_str() {
                "completed" => {
                    info!(install_id = %installed.install_id, "install completed");
                    return Ok(installed.install_id);
                }
                "failed" => {
                    return Err(anyhow::anyhow!("install failed: {:?}", status.failure_reason));
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(anyhow::anyhow!("install timed out"))
    }

    async fn verify_installation(&self, install_id: &str) -> anyhow::Result<()> {
        let verification = self
            .marketplace_client
            .install_verify(InstallVerifyRequest {
                install_id: install_id.into(),
                run_smoke_test: true,
            })
            .await?;
        if verification.passed {
            info!(install_id = install_id, "verified");
        } else {
            error!(install_id = install_id, error = ?verification.failure_reason, "verification failed");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("marketplace-installer")?;
    let marketplace_client = MarketplaceClient::from_env()?;
    let iam_client = IamClient::from_env()?;
    let principal: Principal = iam_client.whoami().await?;

    let installer = MarketplaceInstaller {
        marketplace_client,
        iam_client,
        tenant_id: principal.tenant_id.clone(),
    };

    let install_id = installer
        .find_and_install("AI document translation", "plugin", 30.0)
        .await?;
    installer.verify_installation(&install_id).await?;

    info!("done");
    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"installer@tenant-acme",
    action in [
        Action::"marketplace::listing::search",
        Action::"marketplace::listing::read",
        Action::"marketplace::install::create",
        Action::"marketplace::install::read",
        Action::"marketplace::install::verify"
    ],
    resource in Tenant::"tenant_acme"
);
```

## Compliance evidence emitted

Every install emits to audit-chain:

```json
{
    "event_class": "marketplace::install::completed",
    "tenant_id": "tenant_acme",
    "install_id": "ins_01HXYZ...",
    "listing_id": "lst_01HXYZ...",
    "listing_slug": "docs-translate-pro",
    "publisher_id": "acme-software-co",
    "version_installed": "1.0.0",
    "permissions_granted": ["docs::document::read", "docs::document::comment"],
    "sbom_digest": "sha256:...",
    "installer_principal_id": "user:tenant-admin@your-tenant.com",
    "consent_response_hash": "sha256:..."
}
```

This is the chain-of-custody for the install — what was installed, what permissions were granted, what consent was given.

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_MARKETPLACE_API=https://marketplace-api.dev.<tenant>.oyatie.io \
OYA_IAM_API=https://iam-api.dev.<tenant>.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in portal: Marketplace → Installs → search for install ID. See the full install log + permissions granted + SBOM.

## Notes

- The `acceptable_permissions` allowlist demonstrates a key security pattern: programmatic installs should validate the listing's permissions against an explicit allowlist BEFORE installing. The substrate enforces consent-prompts for interactive installs, but programmatic installs bypass UI; you must enforce in code.
- The consent_response is logged + cryptographically anchored. This is the audit evidence that the tenant agreed to the permissions before install.
- For complex multi-stage installs (e.g. installing an app that requires its own database schema), the substrate runs migrations as part of the install; failures are surfaced via the install_status endpoint.
- For paid listings, the install flow includes a Stripe Checkout redirect; the Rust SDK above doesn't handle that — for programmatic installs of paid listings, use the substrate's machine-token flow with pre-arranged payment.
