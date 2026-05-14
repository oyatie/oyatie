use oya_data_boundary_kernel::{DataClass, DataClassification, Purpose};
use oya_identity_domain::{
    CredentialRequest, CredentialRequestKind, IdentityError, Principal, issue_credential,
};

#[test]
fn sts_credentials_are_short_lived_scope_bound_and_redacted() {
    let principal =
        Principal::human("ten_alpha".into(), "usr_admin".into()).expect("human principal is valid");
    let credential = issue_credential(CredentialRequest {
        principal: principal.clone(),
        kind: CredentialRequestKind::Sts,
        purpose: Purpose::CapabilityInvocation,
        scopes: vec!["cloud.iam.read".into(), "foundry.invoke".into()],
        ttl_seconds: 900,
        issued_at_epoch_seconds: 1_000,
    })
    .expect("STS credential is valid");

    assert_eq!(credential.principal, principal);
    assert_eq!(credential.purpose.value, Purpose::CapabilityInvocation);
    assert_eq!(credential.expires_at_epoch_seconds.value, 1_900);
    assert_eq!(
        credential.scopes.data_class,
        DataClassification::from(DataClass::InternalOnly)
    );
    assert!(credential.is_active(1_899));
    assert!(!credential.is_active(1_900));
    assert!(!format!("{credential:?}").contains("bearer"));
    assert!(credential.token_fingerprint.value.starts_with("sts1:"));
}

#[test]
fn identity_kernel_rejects_long_lived_or_unscoped_credentials() {
    let principal = Principal::service(
        "ten_alpha".into(),
        "sp_foundry".into(),
        "cap.provider.openai".into(),
    )
    .expect("service principal is valid");

    assert_eq!(
        issue_credential(CredentialRequest {
            principal: principal.clone(),
            kind: CredentialRequestKind::LongLivedApiKey,
            purpose: Purpose::CapabilityInvocation,
            scopes: vec!["foundry.invoke".into()],
            ttl_seconds: 900,
            issued_at_epoch_seconds: 1_000,
        }),
        Err(IdentityError::LongLivedCredentialForbidden)
    );
    assert_eq!(
        issue_credential(CredentialRequest {
            principal: principal.clone(),
            kind: CredentialRequestKind::Sts,
            purpose: Purpose::CapabilityInvocation,
            scopes: vec![],
            ttl_seconds: 900,
            issued_at_epoch_seconds: 1_000,
        }),
        Err(IdentityError::MissingCredentialScope)
    );
    assert_eq!(
        issue_credential(CredentialRequest {
            principal,
            kind: CredentialRequestKind::Sts,
            purpose: Purpose::CapabilityInvocation,
            scopes: vec!["foundry.invoke".into()],
            ttl_seconds: 3_601,
            issued_at_epoch_seconds: 1_000,
        }),
        Err(IdentityError::TokenTtlTooLong)
    );
}

#[test]
fn principal_constructors_validate_identity_shapes() {
    assert_eq!(
        Principal::human("tenant".into(), "usr_admin".into()),
        Err(IdentityError::InvalidTenantId)
    );
    assert_eq!(
        Principal::human("ten_alpha".into(), "user".into()),
        Err(IdentityError::InvalidUserId)
    );
    assert_eq!(
        Principal::service("ten_alpha".into(), "service".into(), "cap.provider".into()),
        Err(IdentityError::InvalidServicePrincipalId)
    );
}
