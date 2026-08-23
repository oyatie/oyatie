//! OCI Identity Domain adapter boundary for Cloud IAM identity providers.
//!
//! This crate translates the provider-neutral Cloud IAM identity-provider sync
//! contract into deterministic OCI Identity Domain request shapes. It does not
//! hold credentials, call OCI SDKs, or perform network I/O; credentialed live
//! smoke stays a separate promotion gate.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use iam_domain::{
    CloudIamProviderKind, IamProviderIdentityProviderError, IamProviderIdentityProviderOperation,
    IamProviderIdentityProviderPort, IamProviderIdentityProviderSyncReceipt,
    IamProviderIdentityProviderSyncRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciIamIdentityProviderAdapterConfigError {
    InvalidEndpoint,
    InvalidIdentityDomainRef,
    InvalidCompartmentRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciIamIdentityProviderAdapter {
    endpoint_origin: String,     // data_class: INTERNAL_ONLY
    identity_domain_ref: String, // data_class: INTERNAL_ONLY
    compartment_ref: String,     // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciIamIdentityProviderCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl OciIamIdentityProviderAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        identity_domain_ref: impl Into<String>,
        compartment_ref: impl Into<String>,
    ) -> Result<Self, OciIamIdentityProviderAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let identity_domain_ref = identity_domain_ref.into();
        let compartment_ref = compartment_ref.into();
        validate_endpoint(&endpoint_origin)?;
        validate_segment(
            &identity_domain_ref,
            OciIamIdentityProviderAdapterConfigError::InvalidIdentityDomainRef,
        )?;
        validate_segment(
            &compartment_ref,
            OciIamIdentityProviderAdapterConfigError::InvalidCompartmentRef,
        )?;
        Ok(Self {
            endpoint_origin,
            identity_domain_ref,
            compartment_ref,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_identity_provider_ref(&self, identity_provider_id: &str) -> String {
        format!(
            "oci-iam-idp://{}/{}",
            self.identity_domain_ref, identity_provider_id
        )
    }

    pub fn sync_command(
        &self,
        request: &IamProviderIdentityProviderSyncRequest,
    ) -> Result<OciIamIdentityProviderCommand, IamProviderIdentityProviderError> {
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
        Ok(OciIamIdentityProviderCommand {
            operation,
            method,
            endpoint_origin: self.endpoint_origin.clone(),
            path: format!(
                "/20160918/identityDomains/{}/identityProviders/{}",
                self.identity_domain_ref, request.identity_provider.id.value.value
            ),
            body_canonical: canonical_body(&[
                ("identity_domain_ref", self.identity_domain_ref.as_str()),
                ("compartment_ref", self.compartment_ref.as_str()),
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
                "oci-iam-idp://{}/{}/{}",
                self.identity_domain_ref,
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
                provider: CloudIamProviderKind::OciIdentityDomain,
                reason:
                    "provider_identity_provider_ref does not match configured OCI Identity Domain"
                        .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-iam-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl IamProviderIdentityProviderPort for OciIamIdentityProviderAdapter {
    fn provider_kind(&self) -> CloudIamProviderKind {
        CloudIamProviderKind::OciIdentityDomain
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

fn validate_endpoint(value: &str) -> Result<(), OciIamIdentityProviderAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciIamIdentityProviderAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_segment(
    value: &str,
    error: OciIamIdentityProviderAdapterConfigError,
) -> Result<(), OciIamIdentityProviderAdapterConfigError> {
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
    use iam_domain::{IdentityProvider, IdentityProviderCreate, IdentityProviderKind};

    fn adapter() -> OciIamIdentityProviderAdapter {
        OciIamIdentityProviderAdapter::new(
            "https://identity.ap-chuncheon-1.oci.example",
            "identity-domain-alpha",
            "ocid1.compartment.oc1..alpha",
        )
        .unwrap()
        .with_clock(1_700_000_020)
    }

    fn identity_provider() -> IdentityProvider {
        IdentityProvider::new(IdentityProviderCreate {
            id: "idp_alpha_saml".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region_pack: "pack-alpha".to_string(),
            kind: IdentityProviderKind::Saml,
            issuer_uri: "https://idp.alpha.example/saml".to_string(),
            audience: "urn:oyatie:cloud".to_string(),
            verification_material_ref: "cert/alpha-saml-signing".to_string(),
            created_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn sync_request(
        operation: IamProviderIdentityProviderOperation,
    ) -> IamProviderIdentityProviderSyncRequest {
        let adapter = adapter();
        IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-001".to_string(),
            provider_identity_provider_ref: adapter
                .provider_identity_provider_ref("idp_alpha_saml"),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-001".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
            operation,
            identity_provider: identity_provider(),
        }
    }

    #[test]
    fn builds_deterministic_oci_identity_provider_upsert_command() {
        let adapter = adapter();
        let command = adapter
            .sync_command(&sync_request(IamProviderIdentityProviderOperation::Upsert))
            .expect("OCI sync command is deterministic");

        assert_eq!(command.operation, "UpsertIdentityProvider");
        assert_eq!(command.method, "PUT");
        assert_eq!(
            command.path,
            "/20160918/identityDomains/identity-domain-alpha/identityProviders/idp_alpha_saml"
        );
        assert!(command.body_canonical.contains("kind=saml"));
        assert!(
            command
                .body_canonical
                .contains("verification_material_ref=cert/alpha-saml-signing")
        );
        assert_eq!(
            command.provider_evidence_ref,
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001"
        );
    }

    #[test]
    fn returns_provider_receipts_and_rejects_wrong_oci_domain_refs() {
        let adapter = adapter();
        let receipt = adapter
            .sync_identity_provider(sync_request(IamProviderIdentityProviderOperation::Upsert))
            .expect("OCI provider receipt is valid");

        assert_eq!(receipt.provider, CloudIamProviderKind::OciIdentityDomain);
        assert_eq!(
            receipt.provider_request_id,
            "oci-iam-1700000020-iam-idp-sync-001"
        );
        assert_eq!(receipt.identity_provider_kind, IdentityProviderKind::Saml);
        assert_eq!(receipt.operation.label(), "upsert");

        let delete_receipt = adapter
            .sync_identity_provider(sync_request(IamProviderIdentityProviderOperation::Delete))
            .expect("OCI delete receipt is valid");
        assert_eq!(delete_receipt.sync_status.label(), "delete_synchronized");

        let mut wrong_ref = sync_request(IamProviderIdentityProviderOperation::Upsert);
        wrong_ref.provider_identity_provider_ref =
            "oci-iam-idp://other-domain/idp_alpha_saml".to_string();
        assert_eq!(
            adapter.sync_command(&wrong_ref),
            Err(IamProviderIdentityProviderError::ProviderRejected {
                provider: CloudIamProviderKind::OciIdentityDomain,
                reason:
                    "provider_identity_provider_ref does not match configured OCI Identity Domain"
                        .to_string(),
            })
        );
    }
}
