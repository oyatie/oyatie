use intelligence_credential_resolver_domain::{
    CredentialAudience, CredentialHandle, CredentialHandleIssueRequest, CredentialProvider,
    SecretReference, SecretReferenceKind, SecretReferenceValidationError,
};

#[test]
fn parses_openbao_secret_reference_bound_to_tenant_and_provider() {
    let reference = SecretReference::parse(
        "${openbao:secret/ten_a/intelligence/provider/openai}",
        "ten_a",
        CredentialProvider::OpenAi,
    )
    .expect("valid openbao reference");

    assert_eq!(reference.kind(), SecretReferenceKind::OpenBaoPath);
    assert_eq!(reference.bound_tenant(), "ten_a");
    assert_eq!(reference.provider(), CredentialProvider::OpenAi);
    assert_eq!(
        reference.canonical_ref(),
        "openbao://secret/ten_a/intelligence/provider/openai"
    );
}

#[test]
fn rejects_tenant_drift_raw_secret_material_and_wrong_provider_path() {
    assert_eq!(
        SecretReference::parse(
            "${openbao:secret/ten_b/intelligence/provider/openai}",
            "ten_a",
            CredentialProvider::OpenAi,
        ),
        Err(SecretReferenceValidationError::TenantMismatch)
    );
    assert_eq!(
        SecretReference::parse("sk-test-raw", "ten_a", CredentialProvider::OpenAi),
        Err(SecretReferenceValidationError::RawSecretMaterialRejected)
    );
    assert_eq!(
        SecretReference::parse(
            "${openbao:secret/ten_a/intelligence/provider/anthropic}",
            "ten_a",
            CredentialProvider::OpenAi,
        ),
        Err(SecretReferenceValidationError::ProviderMismatch)
    );
}

#[test]
fn rejects_ambiguous_whitespace_around_refs_and_tenant_ids() {
    assert_eq!(
        SecretReference::parse(
            " ${openbao:secret/ten_a/intelligence/provider/openai}",
            "ten_a",
            CredentialProvider::OpenAi,
        ),
        Err(SecretReferenceValidationError::RawSecretMaterialRejected)
    );
    assert_eq!(
        SecretReference::parse(
            "${openbao:secret/ten_a/intelligence/provider/openai}",
            " ten_a",
            CredentialProvider::OpenAi,
        ),
        Err(SecretReferenceValidationError::InvalidTenant)
    );
}

#[test]
fn issues_short_lived_opaque_handle_without_value_accessor() {
    let handle = CredentialHandle::issue(CredentialHandleIssueRequest {
        handle_id: "handle://ten_a/openai/gen-7".to_owned(),
        tenant_id: "ten_a".to_owned(),
        provider: CredentialProvider::OpenAi,
        audience: CredentialAudience::ProviderDispatch,
        issued_at_epoch_seconds: 100,
        expires_at_epoch_seconds: 160,
        generation: 7,
        sidecar_signature_ref: "sigref://openbao/handle/7".to_owned(),
    })
    .expect("valid short-lived handle");

    assert_eq!(handle.bound_tenant(), "ten_a");
    assert_eq!(handle.bound_provider(), CredentialProvider::OpenAi);
    assert!(!handle.is_expired(159));
    assert!(handle.is_expired(160));
    assert!(handle.is_valid_for(
        "ten_a",
        CredentialProvider::OpenAi,
        CredentialAudience::ProviderDispatch,
        159
    ));
    let debug = format!("{handle:?}");
    assert!(!debug.contains("handle://ten_a/openai/gen-7"));
    assert!(!debug.contains("sigref://openbao/handle/7"));
    assert!(!debug.contains("sk-"));
}

#[test]
fn rejects_unbounded_or_inverted_handle_ttl() {
    let mut request = CredentialHandleIssueRequest {
        handle_id: "handle://ten_a/openai/gen-7".to_owned(),
        tenant_id: "ten_a".to_owned(),
        provider: CredentialProvider::OpenAi,
        audience: CredentialAudience::ProviderDispatch,
        issued_at_epoch_seconds: 100,
        expires_at_epoch_seconds: 161,
        generation: 7,
        sidecar_signature_ref: "sigref://openbao/handle/7".to_owned(),
    };

    assert!(CredentialHandle::issue(request.clone()).is_err());
    request.expires_at_epoch_seconds = 99;
    assert!(CredentialHandle::issue(request).is_err());
}
