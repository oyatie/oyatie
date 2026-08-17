//! OCI KMS adapter boundary for Cloud KMS.
//!
//! This crate keeps OCI vault/key identifiers, operation paths, and evidence refs
//! outside the provider-neutral Cloud KMS domain/API crates while implementing
//! the shared `KmsProviderCryptoPort` contract.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use secrets_kms_domain::{
    KmsProviderCryptoError, KmsProviderCryptoPort, KmsProviderCryptoReceipt,
    KmsProviderDecryptRequest, KmsProviderEncryptRequest, KmsProviderKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciKmsAdapterConfigError {
    InvalidManagementEndpoint,
    InvalidVaultId,
    InvalidKeyId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciKmsAdapter {
    management_endpoint: String, // data_class: INTERNAL_ONLY
    vault_id: String,            // data_class: INTERNAL_ONLY
    key_id: String,              // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciKmsCommand {
    pub operation: &'static str, // data_class: PUBLIC
    pub method: &'static str,    // data_class: PUBLIC
    pub path: &'static str,      // data_class: INTERNAL_ONLY
    pub body_canonical: String,  // data_class: INTERNAL_ONLY
    pub evidence_ref: String,    // data_class: INTERNAL_ONLY
}

impl OciKmsAdapter {
    pub fn new(
        management_endpoint: impl Into<String>,
        vault_id: impl Into<String>,
        key_id: impl Into<String>,
    ) -> Result<Self, OciKmsAdapterConfigError> {
        let management_endpoint = management_endpoint.into();
        let vault_id = vault_id.into();
        let key_id = key_id.into();
        validate_endpoint(&management_endpoint)?;
        validate_ocid(
            &vault_id,
            "ocid1.vault.",
            OciKmsAdapterConfigError::InvalidVaultId,
        )?;
        validate_ocid(
            &key_id,
            "ocid1.key.",
            OciKmsAdapterConfigError::InvalidKeyId,
        )?;
        Ok(Self {
            management_endpoint,
            vault_id,
            key_id,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_key_ref(&self) -> String {
        format!("oci/{}/{}", self.vault_id, self.key_id)
    }

    pub fn encrypt_command(
        &self,
        request: &KmsProviderEncryptRequest,
    ) -> Result<OciKmsCommand, KmsProviderCryptoError> {
        request.validate()?;
        self.ensure_provider_key(&request.provider_key_ref)?;
        Ok(self.command(
            "Encrypt",
            "/20180608/encrypt",
            &request.request_id,
            &[
                ("keyId", self.key_id.as_str()),
                ("plaintext_ref", request.plaintext_ref.as_str()),
                ("ciphertext_ref", request.ciphertext_ref.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("cloud_key_id", request.key_id.as_str()),
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
    ) -> Result<OciKmsCommand, KmsProviderCryptoError> {
        request.validate()?;
        self.ensure_provider_key(&request.provider_key_ref)?;
        Ok(self.command(
            "Decrypt",
            "/20180608/decrypt",
            &request.request_id,
            &[
                ("keyId", self.key_id.as_str()),
                ("ciphertext_ref", request.ciphertext_ref.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("cloud_key_id", request.key_id.as_str()),
                ("data_class", request.data_class.label()),
                ("purpose", request.purpose.label()),
                ("actor", request.actor.as_str()),
            ],
        ))
    }

    fn command(
        &self,
        operation: &'static str,
        path: &'static str,
        request_id: &str,
        fields: &[(&str, &str)],
    ) -> OciKmsCommand {
        OciKmsCommand {
            operation,
            method: "POST",
            path,
            body_canonical: canonical_body(fields),
            evidence_ref: format!("oci-kms://{}/{}/{}", self.vault_id, self.key_id, request_id),
        }
    }

    fn ensure_provider_key(&self, provider_key_ref: &str) -> Result<(), KmsProviderCryptoError> {
        let expected = self.provider_key_ref();
        if provider_key_ref == expected {
            Ok(())
        } else {
            Err(KmsProviderCryptoError::ProviderRejected {
                provider: KmsProviderKind::OciKms,
                reason: "provider_key_ref does not match configured OCI KMS key".to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-kms-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl KmsProviderCryptoPort for OciKmsAdapter {
    fn provider_kind(&self) -> KmsProviderKind {
        KmsProviderKind::OciKms
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
            command.evidence_ref,
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
            command.evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), OciKmsAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciKmsAdapterConfigError::InvalidManagementEndpoint)
    }
}

fn validate_ocid(
    value: &str,
    prefix: &str,
    error: OciKmsAdapterConfigError,
) -> Result<(), OciKmsAdapterConfigError> {
    if value.starts_with(prefix) && no_space_or_control(value) {
        Ok(())
    } else {
        Err(error)
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
    use oya_data_boundary_kernel::DataClass;
    use secrets_kms_domain::{KmsOperation, KmsPurpose};

    const VAULT_ID: &str = "ocid1.vault.oc1.ap-chuncheon-1.testvault";
    const KEY_ID: &str = "ocid1.key.oc1.ap-chuncheon-1.testkey";

    fn adapter() -> OciKmsAdapter {
        OciKmsAdapter::new(
            "https://kms.ap-chuncheon-1.oci.oraclecloud.com",
            VAULT_ID,
            KEY_ID,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn encrypt_request() -> KmsProviderEncryptRequest {
        KmsProviderEncryptRequest {
            request_id: "kmsprov_req_encrypt_001".to_string(),
            provider_key_ref: format!("oci/{VAULT_ID}/{KEY_ID}"),
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
            provider_key_ref: format!("oci/{VAULT_ID}/{KEY_ID}"),
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
    fn oci_encrypt_command_targets_management_endpoint_shape() {
        let command = adapter().encrypt_command(&encrypt_request()).unwrap();

        assert_eq!(command.method, "POST");
        assert_eq!(command.operation, "Encrypt");
        assert_eq!(command.path, "/20180608/encrypt");
        assert!(
            command
                .body_canonical
                .contains("plaintext_ref=matref/ten_alpha/object/001")
        );
        assert!(command.body_canonical.contains(KEY_ID));
        assert!(!command.body_canonical.contains("super-secret"));
        assert_eq!(
            command.evidence_ref,
            format!("oci-kms://{VAULT_ID}/{KEY_ID}/kmsprov_req_encrypt_001")
        );
    }

    #[test]
    fn oci_decrypt_command_targets_decrypt_shape() {
        let command = adapter().decrypt_command(&decrypt_request()).unwrap();

        assert_eq!(command.operation, "Decrypt");
        assert_eq!(command.path, "/20180608/decrypt");
        assert!(
            command
                .body_canonical
                .contains("ciphertext_ref=ct/ten_alpha/object/001")
        );
        assert!(!command.body_canonical.contains("plaintext_ref="));
    }

    #[test]
    fn oci_port_returns_provider_receipts() {
        let adapter = adapter();
        let encrypt = adapter.encrypt(encrypt_request()).unwrap();
        assert_eq!(encrypt.provider, KmsProviderKind::OciKms);
        assert_eq!(encrypt.operation, KmsOperation::Encrypt);
        assert_eq!(
            encrypt.provider_request_id,
            "oci-kms-1700000000-kmsprov_req_encrypt_001"
        );

        let decrypt = adapter.decrypt(decrypt_request()).unwrap();
        assert_eq!(decrypt.provider, KmsProviderKind::OciKms);
        assert_eq!(decrypt.operation, KmsOperation::Decrypt);
        assert_eq!(decrypt.material_ref, None);
    }

    #[test]
    fn oci_rejects_drifted_provider_key_ref() {
        let mut request = encrypt_request();
        request.provider_key_ref = format!("oci/{VAULT_ID}/ocid1.key.oc1.ap-chuncheon-1.other");

        let error = adapter().encrypt(request).unwrap_err();
        assert!(matches!(
            error,
            KmsProviderCryptoError::ProviderRejected {
                provider: KmsProviderKind::OciKms,
                ..
            }
        ));
    }
}
