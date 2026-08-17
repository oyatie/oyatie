//! OpenBao transit adapter boundary for Cloud KMS.
//!
//! This crate is provider-specific by design: it translates the provider-neutral
//! `KmsProviderCryptoPort` contract into OpenBao transit request shapes without
//! letting OpenBao path, namespace, token, or audit-log details leak into the
//! Cloud KMS domain/API crates.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use secrets_kms_domain::{
    KmsProviderCryptoError, KmsProviderCryptoPort, KmsProviderCryptoReceipt,
    KmsProviderDecryptRequest, KmsProviderEncryptRequest, KmsProviderKind,
};

pub mod root_custody;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenBaoKmsAdapterConfigError {
    InvalidEndpoint,
    InvalidTransitMount,
    InvalidKeyName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBaoKmsAdapter {
    endpoint_origin: String,   // data_class: INTERNAL_ONLY
    transit_mount: String,     // data_class: INTERNAL_ONLY
    key_name: String,          // data_class: INTERNAL_ONLY
    namespace: Option<String>, // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBaoTransitCommand {
    pub method: &'static str,       // data_class: PUBLIC
    pub path: String,               // data_class: INTERNAL_ONLY
    pub namespace: Option<String>,  // data_class: INTERNAL_ONLY
    pub body_canonical: String,     // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl OpenBaoKmsAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        transit_mount: impl Into<String>,
        key_name: impl Into<String>,
    ) -> Result<Self, OpenBaoKmsAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let transit_mount = transit_mount.into();
        let key_name = key_name.into();
        validate_endpoint(&endpoint_origin)?;
        validate_path_segment(
            &transit_mount,
            OpenBaoKmsAdapterConfigError::InvalidTransitMount,
        )?;
        validate_path_segment(&key_name, OpenBaoKmsAdapterConfigError::InvalidKeyName)?;
        Ok(Self {
            endpoint_origin,
            transit_mount,
            key_name,
            namespace: None,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_namespace(
        mut self,
        namespace: impl Into<String>,
    ) -> Result<Self, OpenBaoKmsAdapterConfigError> {
        let namespace = namespace.into();
        validate_path_segment(
            &namespace,
            OpenBaoKmsAdapterConfigError::InvalidTransitMount,
        )?;
        self.namespace = Some(namespace);
        Ok(self)
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_key_ref(&self) -> String {
        format!("openbao/{}/{}", self.transit_mount, self.key_name)
    }

    pub fn encrypt_command(
        &self,
        request: &KmsProviderEncryptRequest,
    ) -> Result<OpenBaoTransitCommand, KmsProviderCryptoError> {
        request.validate()?;
        self.ensure_provider_key(&request.provider_key_ref)?;
        Ok(self.command(
            "encrypt",
            &request.request_id,
            &[
                ("plaintext_ref", request.plaintext_ref.as_str()),
                ("ciphertext_ref", request.ciphertext_ref.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("key_id", request.key_id.as_str()),
                ("data_class", request.data_class.label()),
                ("purpose", request.purpose.label()),
                ("actor", request.actor.as_str()),
                ("aad_fingerprint", request.aad_fingerprint.as_str()),
            ],
        ))
    }

    pub fn decrypt_command(
        &self,
        request: &KmsProviderDecryptRequest,
    ) -> Result<OpenBaoTransitCommand, KmsProviderCryptoError> {
        request.validate()?;
        self.ensure_provider_key(&request.provider_key_ref)?;
        Ok(self.command(
            "decrypt",
            &request.request_id,
            &[
                ("ciphertext_ref", request.ciphertext_ref.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("key_id", request.key_id.as_str()),
                ("data_class", request.data_class.label()),
                ("purpose", request.purpose.label()),
                ("actor", request.actor.as_str()),
            ],
        ))
    }

    fn command(
        &self,
        operation: &'static str,
        request_id: &str,
        fields: &[(&str, &str)],
    ) -> OpenBaoTransitCommand {
        OpenBaoTransitCommand {
            method: "POST",
            path: format!("/v1/{}/{}/{}", self.transit_mount, operation, self.key_name),
            namespace: self.namespace.clone(),
            body_canonical: canonical_body(fields),
            audit_evidence_ref: format!(
                "openbao-transit://{}/{}/{}/{}",
                origin_without_scheme(&self.endpoint_origin),
                self.transit_mount,
                self.key_name,
                request_id
            ),
        }
    }

    fn ensure_provider_key(&self, provider_key_ref: &str) -> Result<(), KmsProviderCryptoError> {
        let expected = self.provider_key_ref();
        if provider_key_ref == expected {
            Ok(())
        } else {
            Err(KmsProviderCryptoError::ProviderRejected {
                provider: KmsProviderKind::OpenBaoTransit,
                reason: "provider_key_ref does not match configured OpenBao transit key"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("openbao-transit-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl KmsProviderCryptoPort for OpenBaoKmsAdapter {
    fn provider_kind(&self) -> KmsProviderKind {
        KmsProviderKind::OpenBaoTransit
    }

    fn encrypt(
        &self,
        input: KmsProviderEncryptRequest,
    ) -> Result<KmsProviderCryptoReceipt, KmsProviderCryptoError> {
        let command = self.encrypt_command(&input)?;
        KmsProviderCryptoReceipt::encrypt(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.audit_evidence_ref,
        )
    }

    fn decrypt(
        &self,
        input: KmsProviderDecryptRequest,
    ) -> Result<KmsProviderCryptoReceipt, KmsProviderCryptoError> {
        let command = self.decrypt_command(&input)?;
        KmsProviderCryptoReceipt::decrypt(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.audit_evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), OpenBaoKmsAdapterConfigError> {
    if (value.starts_with("https://") || value.starts_with("http://")) && no_space_or_control(value)
    {
        Ok(())
    } else {
        Err(OpenBaoKmsAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_path_segment(
    value: &str,
    error: OpenBaoKmsAdapterConfigError,
) -> Result<(), OpenBaoKmsAdapterConfigError> {
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

fn origin_without_scheme(endpoint_origin: &str) -> &str {
    endpoint_origin
        .strip_prefix("https://")
        .or_else(|| endpoint_origin.strip_prefix("http://"))
        .unwrap_or(endpoint_origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::DataClass;
    use secrets_kms_domain::{KmsOperation, KmsPurpose};

    fn adapter() -> OpenBaoKmsAdapter {
        OpenBaoKmsAdapter::new("https://kms.oyatie.com", "transit", "object-key")
            .unwrap()
            .with_clock(1_700_000_000)
    }

    fn encrypt_request() -> KmsProviderEncryptRequest {
        KmsProviderEncryptRequest {
            request_id: "kmsprov_req_encrypt_001".to_string(),
            provider_key_ref: "openbao/transit/object-key".to_string(),
            key_id: "kms/alpha-region/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            plaintext_ref: "matref/ten_alpha/object/001".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: DataClass::PiiIdentifying,
            purpose: KmsPurpose::CloudObjectStorage,
            actor: "sp_storage".to_string(),
            aad_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn decrypt_request() -> KmsProviderDecryptRequest {
        KmsProviderDecryptRequest {
            request_id: "kmsprov_req_decrypt_001".to_string(),
            provider_key_ref: "openbao/transit/object-key".to_string(),
            key_id: "kms/alpha-region/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: DataClass::PiiIdentifying,
            purpose: KmsPurpose::CloudObjectStorage,
            actor: "sp_storage".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
        }
    }

    #[test]
    fn openbao_encrypt_command_targets_transit_without_secret_material() {
        let command = adapter().encrypt_command(&encrypt_request()).unwrap();

        assert_eq!(command.method, "POST");
        assert_eq!(command.path, "/v1/transit/encrypt/object-key");
        assert!(
            command
                .body_canonical
                .contains("plaintext_ref=matref/ten_alpha/object/001")
        );
        assert!(
            command
                .body_canonical
                .contains("aad_fingerprint=0123456789abcdef")
        );
        assert!(!command.body_canonical.contains("super-secret"));
        assert_eq!(
            command.audit_evidence_ref,
            "openbao-transit://kms.oyatie.com/transit/object-key/kmsprov_req_encrypt_001"
        );
    }

    #[test]
    fn openbao_decrypt_command_targets_transit_decrypt_path() {
        let command = adapter().decrypt_command(&decrypt_request()).unwrap();

        assert_eq!(command.path, "/v1/transit/decrypt/object-key");
        assert!(
            command
                .body_canonical
                .contains("ciphertext_ref=ct/ten_alpha/object/001")
        );
        assert!(!command.body_canonical.contains("plaintext_ref="));
    }

    #[test]
    fn openbao_port_returns_provider_receipts() {
        let adapter = adapter();
        let encrypt = adapter.encrypt(encrypt_request()).unwrap();
        assert_eq!(encrypt.provider, KmsProviderKind::OpenBaoTransit);
        assert_eq!(encrypt.operation, KmsOperation::Encrypt);
        assert_eq!(
            encrypt.provider_request_id,
            "openbao-transit-1700000000-kmsprov_req_encrypt_001"
        );

        let decrypt = adapter.decrypt(decrypt_request()).unwrap();
        assert_eq!(decrypt.provider, KmsProviderKind::OpenBaoTransit);
        assert_eq!(decrypt.operation, KmsOperation::Decrypt);
        assert_eq!(decrypt.material_ref, None);
    }

    #[test]
    fn openbao_rejects_drifted_provider_key_ref() {
        let mut request = encrypt_request();
        request.provider_key_ref = "openbao/transit/other-key".to_string();

        let error = adapter().encrypt(request).unwrap_err();
        assert!(matches!(
            error,
            KmsProviderCryptoError::ProviderRejected {
                provider: KmsProviderKind::OpenBaoTransit,
                ..
            }
        ));
    }
}
