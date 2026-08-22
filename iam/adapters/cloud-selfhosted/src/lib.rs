//! Self-hosted / colo IAM identity-provider adapter boundary.
//!
//! This crate keeps site, cell, and realm placement for an Oyatie-operated IAM
//! control plane outside the provider-neutral Cloud IAM domain/API crates while
//! implementing the provider-neutral identity-provider sync port used by external-cloud adapters.
//! It emits deterministic request shapes only; live on-prem/colo smoke remains a
//! separate promotion gate.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use iam_cloud_domain::{
    CloudIamProviderKind, IamProviderIdentityProviderError, IamProviderIdentityProviderOperation,
    IamProviderIdentityProviderPort, IamProviderIdentityProviderSyncReceipt,
    IamProviderIdentityProviderSyncRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelfHostedIamIdentityProviderAdapterConfigError {
    InvalidEndpoint,
    InvalidSiteRef,
    InvalidCellRef,
    InvalidRealmRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedIamIdentityProviderAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    site_ref: String,         // data_class: PUBLIC
    cell_ref: String,         // data_class: PUBLIC
    realm_ref: String,        // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedIamIdentityProviderCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl SelfHostedIamIdentityProviderAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        site_ref: impl Into<String>,
        cell_ref: impl Into<String>,
        realm_ref: impl Into<String>,
    ) -> Result<Self, SelfHostedIamIdentityProviderAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let site_ref = site_ref.into();
        let cell_ref = cell_ref.into();
        let realm_ref = realm_ref.into();
        validate_endpoint(&endpoint_origin)?;
        validate_segment(
            &site_ref,
            SelfHostedIamIdentityProviderAdapterConfigError::InvalidSiteRef,
        )?;
        validate_segment(
            &cell_ref,
            SelfHostedIamIdentityProviderAdapterConfigError::InvalidCellRef,
        )?;
        validate_segment(
            &realm_ref,
            SelfHostedIamIdentityProviderAdapterConfigError::InvalidRealmRef,
        )?;
        Ok(Self {
            endpoint_origin,
            site_ref,
            cell_ref,
            realm_ref,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_identity_provider_ref(&self, identity_provider_id: &str) -> String {
        format!(
            "selfhosted-idp://{}/{}/{}/{}",
            self.site_ref, self.cell_ref, self.realm_ref, identity_provider_id
        )
    }

    pub fn sync_command(
        &self,
        request: &IamProviderIdentityProviderSyncRequest,
    ) -> Result<SelfHostedIamIdentityProviderCommand, IamProviderIdentityProviderError> {
        request.validate()?;
        self.ensure_provider_identity_provider(
            &request.provider_identity_provider_ref,
            &request.identity_provider.id.value.value,
        )?;
        let (operation, method) = match request.operation {
            IamProviderIdentityProviderOperation::Upsert => ("UpsertIdentityProvider", "PUT"),
            IamProviderIdentityProviderOperation::Delete => ("DeleteIdentityProvider", "DELETE"),
        };
        let created_at = request
            .identity_provider
            .created_at_epoch_seconds
            .value
            .to_string();
        let requested_at = request.requested_at_epoch_seconds.to_string();
        Ok(SelfHostedIamIdentityProviderCommand {
            operation,
            method,
            endpoint_origin: self.endpoint_origin.clone(),
            path: format!(
                "/v1/sites/{}/cells/{}/iam/realms/{}/identity-providers/{}",
                self.site_ref,
                self.cell_ref,
                self.realm_ref,
                request.identity_provider.id.value.value
            ),
            body_canonical: canonical_body(&[
                ("site_ref", self.site_ref.as_str()),
                ("cell_ref", self.cell_ref.as_str()),
                ("realm_ref", self.realm_ref.as_str()),
                (
                    "identity_provider_id",
                    request.identity_provider.id.value.value.as_str(),
                ),
                ("tenant_id", request.tenant_id.as_str()),
                ("kind", request.identity_provider.kind.value.label()),
                (
                    "issuer_uri",
                    request.identity_provider.issuer_uri.value.as_str(),
                ),
                (
                    "audience",
                    request.identity_provider.audience.value.as_str(),
                ),
                (
                    "verification_material_ref",
                    request
                        .identity_provider
                        .verification_material_ref
                        .value
                        .as_str(),
                ),
                ("created_at_epoch_seconds", created_at.as_str()),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
                ("requested_at_epoch_seconds", requested_at.as_str()),
            ]),
            provider_evidence_ref: format!(
                "selfhosted-idp://{}/{}/{}/{}/{}",
                self.site_ref,
                self.cell_ref,
                self.realm_ref,
                request.identity_provider.id.value.value,
                request.request_id
            ),
        })
    }

    fn ensure_provider_identity_provider(
        &self,
        provider_identity_provider_ref: &str,
        identity_provider_id: &str,
    ) -> Result<(), IamProviderIdentityProviderError> {
        let expected = self.provider_identity_provider_ref(identity_provider_id);
        if provider_identity_provider_ref == expected {
            Ok(())
        } else {
            Err(IamProviderIdentityProviderError::ProviderRejected {
                provider: CloudIamProviderKind::SelfHostedOidcControlPlane,
                reason:
                    "provider_identity_provider_ref does not match configured self-hosted IAM realm"
                        .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("selfhosted-iam-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl IamProviderIdentityProviderPort for SelfHostedIamIdentityProviderAdapter {
    fn provider_kind(&self) -> CloudIamProviderKind {
        CloudIamProviderKind::SelfHostedOidcControlPlane
    }

    fn sync_identity_provider(
        &self,
        input: IamProviderIdentityProviderSyncRequest,
    ) -> Result<IamProviderIdentityProviderSyncReceipt, IamProviderIdentityProviderError> {
        let command = self.sync_command(&input)?;
        IamProviderIdentityProviderSyncReceipt::from_request(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), SelfHostedIamIdentityProviderAdapterConfigError> {
    if (value.starts_with("https://") || value.starts_with("http://")) && no_space_or_control(value)
    {
        Ok(())
    } else {
        Err(SelfHostedIamIdentityProviderAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_segment(
    value: &str,
    error: SelfHostedIamIdentityProviderAdapterConfigError,
) -> Result<(), SelfHostedIamIdentityProviderAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn no_space_or_control(value: &str) -> bool {
    !value
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte == b' ')
}

fn canonical_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam_cloud_domain::{IdentityProvider, IdentityProviderCreate, IdentityProviderKind};

    fn adapter() -> SelfHostedIamIdentityProviderAdapter {
        SelfHostedIamIdentityProviderAdapter::new(
            "https://iam.colo-alpha.oyatie.example",
            "site-alpha",
            "cell-alpha-region-a-001",
            "realm-ten-alpha",
        )
        .unwrap()
        .with_clock(1_700_000_030)
    }

    fn identity_provider() -> IdentityProvider {
        IdentityProvider::new(IdentityProviderCreate {
            id: "idp_alpha_oidc".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region_pack: "pack-alpha".to_string(),
            kind: IdentityProviderKind::Oidc,
            issuer_uri: "https://idp.alpha.example/oidc".to_string(),
            audience: "urn:oyatie:cloud".to_string(),
            verification_material_ref: "jwks/alpha-oidc".to_string(),
            created_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn sync_request(
        operation: IamProviderIdentityProviderOperation,
    ) -> IamProviderIdentityProviderSyncRequest {
        let adapter = adapter();
        IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-002".to_string(),
            provider_identity_provider_ref: adapter
                .provider_identity_provider_ref("idp_alpha_oidc"),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-002".to_string(),
            requested_at_epoch_seconds: 1_700_000_030,
            operation,
            identity_provider: identity_provider(),
        }
    }

    #[test]
    fn builds_deterministic_selfhosted_identity_provider_upsert_command() {
        let adapter = adapter();
        let command = adapter
            .sync_command(&sync_request(IamProviderIdentityProviderOperation::Upsert))
            .expect("self-hosted sync command is deterministic");

        assert_eq!(command.operation, "UpsertIdentityProvider");
        assert_eq!(command.method, "PUT");
        assert_eq!(
            command.path,
            "/v1/sites/site-alpha/cells/cell-alpha-region-a-001/iam/realms/realm-ten-alpha/identity-providers/idp_alpha_oidc"
        );
        assert!(command.body_canonical.contains("kind=oidc"));
        assert!(command.body_canonical.contains("site_ref=site-alpha"));
        assert_eq!(
            command.provider_evidence_ref,
            "selfhosted-idp://site-alpha/cell-alpha-region-a-001/realm-ten-alpha/idp_alpha_oidc/iam-idp-sync-002"
        );
    }

    #[test]
    fn returns_provider_receipts_and_rejects_wrong_selfhosted_refs() {
        let adapter = adapter();
        let receipt = adapter
            .sync_identity_provider(sync_request(IamProviderIdentityProviderOperation::Upsert))
            .expect("self-hosted provider receipt is valid");

        assert_eq!(
            receipt.provider,
            CloudIamProviderKind::SelfHostedOidcControlPlane
        );
        assert_eq!(
            receipt.provider_request_id,
            "selfhosted-iam-1700000030-iam-idp-sync-002"
        );
        assert_eq!(receipt.identity_provider_kind, IdentityProviderKind::Oidc);
        assert_eq!(receipt.sync_status.label(), "synchronized");

        let delete_receipt = adapter
            .sync_identity_provider(sync_request(IamProviderIdentityProviderOperation::Delete))
            .expect("self-hosted delete receipt is valid");
        assert_eq!(delete_receipt.sync_status.label(), "delete_synchronized");

        let mut wrong_ref = sync_request(IamProviderIdentityProviderOperation::Upsert);
        wrong_ref.provider_identity_provider_ref =
            "selfhosted-idp://site-alpha/other-cell/realm-ten-alpha/idp_alpha_oidc".to_string();
        assert_eq!(
            adapter.sync_command(&wrong_ref),
            Err(IamProviderIdentityProviderError::ProviderRejected {
                provider: CloudIamProviderKind::SelfHostedOidcControlPlane,
                reason:
                    "provider_identity_provider_ref does not match configured self-hosted IAM realm"
                        .to_string(),
            })
        );
    }
}
