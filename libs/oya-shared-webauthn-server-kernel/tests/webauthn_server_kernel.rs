#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for `oya-shared-webauthn-server-kernel`.
//!
//! Uses a stub `WebauthnRpAdapter` that returns deterministic credentials
//! and sign counts. The cryptographic correctness of CBOR/COSE/attestation
//! parsing belongs to `webauthn-rs` (or whatever Phase-2 adapter replaces
//! it) and is verified there, not here.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};

use oya_shared_webauthn_server_kernel::{
    Aaguid, AttestationConveyance, AuthenticationChallenge, AuthenticationResponse, CoseKeyCbor,
    Credential, CredentialId, InMemoryChallengeStore, InMemoryCredentialStore, Mediation, PackTier,
    ReferenceWebauthnServer, RegistrationChallenge, RegistrationResponse, TenantId, Transport,
    UserId, WebauthnError, WebauthnRpAdapter, WebauthnServer,
};

struct StubAdapter {
    rp_id: String,
    rp_name: String,
    fixed_aaguid: Aaguid,
    challenge_counter: AtomicU32,
    challenge_mode: ChallengeMode,
    /// Fail registration when client_data starts with "deny:"
    /// Fail authentication when client_data starts with "denyauth:"
    sign_counter: AtomicU32,
}

#[derive(Clone, Copy)]
enum ChallengeMode {
    Stable,
    Empty,
    Fail,
}

impl StubAdapter {
    fn new(aaguid: Aaguid) -> Self {
        Self {
            rp_id: "oyatie.dev".into(),
            rp_name: "Oyatie".into(),
            fixed_aaguid: aaguid,
            challenge_counter: AtomicU32::new(0),
            challenge_mode: ChallengeMode::Stable,
            sign_counter: AtomicU32::new(0),
        }
    }

    fn with_challenge_mode(mut self, challenge_mode: ChallengeMode) -> Self {
        self.challenge_mode = challenge_mode;
        self
    }
}

impl WebauthnRpAdapter for StubAdapter {
    fn generate_challenge(&self) -> Result<Vec<u8>, WebauthnError> {
        match self.challenge_mode {
            ChallengeMode::Stable => {
                let mut challenge: Vec<u8> = (0u8..32).collect();
                let counter = self.challenge_counter.fetch_add(1, Ordering::SeqCst);
                challenge[0] = counter as u8;
                Ok(challenge)
            }
            ChallengeMode::Empty => Ok(Vec::new()),
            ChallengeMode::Fail => Err(WebauthnError::ChallengeGenerationFailed(
                "stub rng unavailable".into(),
            )),
        }
    }
    fn rp_id(&self) -> &str {
        &self.rp_id
    }
    fn rp_name(&self) -> &str {
        &self.rp_name
    }

    fn verify_registration(
        &self,
        challenge: &RegistrationChallenge,
        response: &RegistrationResponse,
        allowlist: Option<&BTreeSet<Aaguid>>,
    ) -> Result<Credential, WebauthnError> {
        if response.client_data_json_b64url.starts_with("deny:") {
            return Err(WebauthnError::AttestationInvalid("stub deny".into()));
        }
        if let Some(expected) = response
            .client_data_json_b64url
            .strip_prefix("expect-challenge:")
            && expected != challenge.challenge_b64url
        {
            return Err(WebauthnError::AttestationInvalid(format!(
                "stub challenge mismatch: expected {expected}, got {}",
                challenge.challenge_b64url
            )));
        }
        if let Some(al) = allowlist
            && !al.contains(&self.fixed_aaguid)
        {
            return Err(WebauthnError::AaguidNotAllowlisted(self.fixed_aaguid));
        }
        // Synthesise a credential ID from challenge for determinism.
        let cred_id = CredentialId(format!("cred-{}", challenge.challenge_id).into_bytes());
        Ok(Credential {
            credential_id: cred_id,
            tenant_id: TenantId(String::new()), // filled by kernel
            user_id: UserId(String::new()),     // filled by kernel
            public_key: oya_shared_webauthn_server_kernel::CoseKeyCbor(vec![0x42; 64]),
            aaguid: self.fixed_aaguid,
            transports: response.transports.clone(),
            attestation_format: "packed".into(),
            backup_eligible: true,
            backup_state: true,
            sign_count: 0,
            last_used_at_unix: 0,
            created_at_unix: 0,
        })
    }

    fn verify_authentication(
        &self,
        _challenge: &AuthenticationChallenge,
        response: &AuthenticationResponse,
        _stored: &Credential,
    ) -> Result<u32, WebauthnError> {
        if response.client_data_json_b64url.starts_with("denyauth:") {
            return Err(WebauthnError::AssertionInvalid("stub deny".into()));
        }
        // Honor an explicit sign_count override via prefix.
        if let Some(rest) = response.client_data_json_b64url.strip_prefix("count:")
            && let Some((n, _)) = rest.split_once(':')
            && let Ok(parsed) = n.parse::<u32>()
        {
            return Ok(parsed);
        }
        Ok(self.sign_counter.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

fn srv(
    aaguid: Aaguid,
    allowlist: BTreeSet<Aaguid>,
) -> ReferenceWebauthnServer<StubAdapter, InMemoryChallengeStore, InMemoryCredentialStore> {
    let mut s = ReferenceWebauthnServer::new(
        StubAdapter::new(aaguid),
        InMemoryChallengeStore::default(),
        InMemoryCredentialStore::default(),
    );
    s.aaguid_allowlist = allowlist;
    s
}

fn tenant() -> TenantId {
    TenantId("tenant-acme".into())
}
fn user() -> UserId {
    UserId("user-1".into())
}

struct PutFailingCredentialStore;

impl oya_shared_webauthn_server_kernel::CredentialStore for PutFailingCredentialStore {
    fn get(
        &self,
        _tenant: &TenantId,
        _cred: &CredentialId,
    ) -> Result<Option<Credential>, WebauthnError> {
        Ok(None)
    }

    fn put(&self, _cred: &Credential) -> Result<(), WebauthnError> {
        Err(WebauthnError::StorageUnavailable {
            operation: "credential.put",
            detail: "postgres write unavailable".into(),
        })
    }

    fn revoke(&self, _tenant: &TenantId, _cred: &CredentialId) -> Result<(), WebauthnError> {
        Ok(())
    }

    fn list_for_user(
        &self,
        _tenant: &TenantId,
        _user: &UserId,
    ) -> Result<Vec<Credential>, WebauthnError> {
        Ok(Vec::new())
    }
}

struct UpdateFailingCredentialStore {
    credential: Credential,
}

impl oya_shared_webauthn_server_kernel::CredentialStore for UpdateFailingCredentialStore {
    fn get(
        &self,
        tenant: &TenantId,
        cred: &CredentialId,
    ) -> Result<Option<Credential>, WebauthnError> {
        Ok(
            (self.credential.tenant_id == *tenant && self.credential.credential_id == *cred)
                .then_some(self.credential.clone()),
        )
    }

    fn put(&self, _cred: &Credential) -> Result<(), WebauthnError> {
        Err(WebauthnError::StorageUnavailable {
            operation: "credential.put",
            detail: "postgres update unavailable".into(),
        })
    }

    fn revoke(&self, _tenant: &TenantId, _cred: &CredentialId) -> Result<(), WebauthnError> {
        Ok(())
    }

    fn list_for_user(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Vec<Credential>, WebauthnError> {
        Ok(
            (self.credential.tenant_id == *tenant && self.credential.user_id == *user)
                .then_some(self.credential.clone())
                .into_iter()
                .collect(),
        )
    }
}

#[test]
fn pack_tier_attestation_requirements() {
    assert_eq!(
        PackTier::SandboxOrDev.required_attestation(),
        AttestationConveyance::None
    );
    assert_eq!(
        PackTier::PackStandard.required_attestation(),
        AttestationConveyance::Indirect
    );
    assert_eq!(
        PackTier::PackRegulated.required_attestation(),
        AttestationConveyance::Direct
    );
    assert_eq!(
        PackTier::AcrCritical.required_attestation(),
        AttestationConveyance::Direct
    );
    assert!(!PackTier::PackStandard.requires_aaguid_allowlist());
    assert!(PackTier::PackRegulated.requires_aaguid_allowlist());
    assert!(PackTier::AcrCritical.requires_aaguid_allowlist());
}

#[test]
fn happy_path_register_then_authenticate() {
    let aaguid = Aaguid([1u8; 16]);
    let server = srv(aaguid, BTreeSet::new());
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Alice",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .expect("begin reg");
    assert_eq!(chal.attestation, AttestationConveyance::Indirect);

    let cred = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackStandard,
            &RegistrationResponse {
                challenge_id: chal.challenge_id.clone(),
                client_data_json_b64url: "ok-data".into(),
                attestation_object_b64url: "ok-attest".into(),
                transports: vec![Transport::Internal],
            },
            1_700_000_000,
        )
        .expect("finish reg");
    assert_eq!(cred.tenant_id, tenant());
    assert_eq!(cred.user_id, user());

    let auth_chal = server
        .begin_authentication(
            &tenant(),
            vec![cred.credential_id.clone()],
            Mediation::Optional,
            1_700_000_050,
        )
        .expect("begin auth");
    let authed = server
        .finish_authentication(
            &tenant(),
            &AuthenticationResponse {
                challenge_id: auth_chal.challenge_id,
                credential_id: cred.credential_id.clone(),
                client_data_json_b64url: "ok".into(),
                authenticator_data_b64url: "ok".into(),
                signature_b64url: "ok".into(),
                user_handle_b64url: None,
            },
            1_700_000_100,
        )
        .expect("finish auth");
    assert_eq!(authed.last_used_at_unix, 1_700_000_100);
    assert!(authed.sign_count >= 1);
}

#[test]
fn begin_registration_fails_closed_when_key_seam_returns_empty_challenge() {
    let server = ReferenceWebauthnServer::new(
        StubAdapter::new(Aaguid([20u8; 16])).with_challenge_mode(ChallengeMode::Empty),
        InMemoryChallengeStore::default(),
        InMemoryCredentialStore::default(),
    );
    let err = server
        .begin_registration(
            &tenant(),
            &user(),
            "Grace",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .unwrap_err();
    assert!(matches!(err, WebauthnError::ChallengeGenerationFailed(_)));
}

#[test]
fn begin_authentication_fails_closed_when_key_seam_errors() {
    let server = ReferenceWebauthnServer::new(
        StubAdapter::new(Aaguid([21u8; 16])).with_challenge_mode(ChallengeMode::Fail),
        InMemoryChallengeStore::default(),
        InMemoryCredentialStore::default(),
    );
    let err = server
        .begin_authentication(&tenant(), vec![], Mediation::Conditional, 1_700_000_000)
        .unwrap_err();
    assert!(matches!(err, WebauthnError::ChallengeGenerationFailed(_)));
}

#[test]
fn finish_registration_fails_closed_when_credential_store_write_fails() {
    let server = ReferenceWebauthnServer::new(
        StubAdapter::new(Aaguid([22u8; 16])),
        InMemoryChallengeStore::default(),
        PutFailingCredentialStore,
    );
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Heidi",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .expect("begin");
    let err = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackStandard,
            &RegistrationResponse {
                challenge_id: chal.challenge_id,
                client_data_json_b64url: "ok".into(),
                attestation_object_b64url: "ok".into(),
                transports: vec![Transport::Internal],
            },
            1_700_000_001,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        WebauthnError::StorageUnavailable {
            operation: "credential.put",
            ..
        }
    ));
}

#[test]
fn finish_authentication_fails_closed_when_credential_store_update_fails() {
    let mut credential = make_credential(Aaguid([23u8; 16]));
    credential.tenant_id = tenant();
    credential.user_id = user();
    credential.credential_id = CredentialId(b"credential-with-production-store".to_vec());
    let server = ReferenceWebauthnServer::new(
        StubAdapter::new(Aaguid([23u8; 16])),
        InMemoryChallengeStore::default(),
        UpdateFailingCredentialStore {
            credential: credential.clone(),
        },
    );

    let chal = server
        .begin_authentication(
            &tenant(),
            vec![credential.credential_id.clone()],
            Mediation::Optional,
            1_700_000_000,
        )
        .expect("begin auth");
    let err = server
        .finish_authentication(
            &tenant(),
            &AuthenticationResponse {
                challenge_id: chal.challenge_id,
                credential_id: credential.credential_id,
                client_data_json_b64url: "count:1:end".into(),
                authenticator_data_b64url: "x".into(),
                signature_b64url: "x".into(),
                user_handle_b64url: None,
            },
            1_700_000_050,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        WebauthnError::StorageUnavailable {
            operation: "credential.put",
            ..
        }
    ));
}

#[test]
fn registration_challenge_ids_are_distinct_for_same_user_and_second() {
    let server = srv(Aaguid([10u8; 16]), BTreeSet::new());
    let first = server
        .begin_registration(
            &tenant(),
            &user(),
            "Grace",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .expect("first begin");
    let second = server
        .begin_registration(
            &tenant(),
            &user(),
            "Grace",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .expect("second begin");

    assert_ne!(
        first.challenge_b64url, second.challenge_b64url,
        "test adapter must issue distinct ceremony challenges"
    );
    assert_ne!(
        first.challenge_id, second.challenge_id,
        "registration challenge IDs must not collide for same tenant/user/second"
    );

    let second_cred = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackStandard,
            &RegistrationResponse {
                challenge_id: second.challenge_id,
                client_data_json_b64url: format!("expect-challenge:{}", second.challenge_b64url),
                attestation_object_b64url: "ok-attest".into(),
                transports: vec![Transport::Internal],
            },
            1_700_000_000,
        )
        .expect("finish second registration against second challenge");
    let first_cred = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackStandard,
            &RegistrationResponse {
                challenge_id: first.challenge_id,
                client_data_json_b64url: format!("expect-challenge:{}", first.challenge_b64url),
                attestation_object_b64url: "ok-attest".into(),
                transports: vec![Transport::Internal],
            },
            1_700_000_000,
        )
        .expect("finish first registration against first challenge");
    assert_ne!(second_cred.credential_id, first_cred.credential_id);
}

#[test]
fn regulated_pack_enforces_aaguid_allowlist() {
    let aaguid = Aaguid([2u8; 16]);
    // Empty allowlist → reject
    let server = srv(aaguid, BTreeSet::new());
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Bob",
            PackTier::PackRegulated,
            1_700_000_000,
        )
        .expect("begin");
    assert_eq!(chal.attestation, AttestationConveyance::Direct);
    let err = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackRegulated,
            &RegistrationResponse {
                challenge_id: chal.challenge_id,
                client_data_json_b64url: "ok".into(),
                attestation_object_b64url: "ok".into(),
                transports: vec![Transport::Usb],
            },
            1_700_000_000,
        )
        .unwrap_err();
    assert!(matches!(err, WebauthnError::AaguidNotAllowlisted(_)));
}

#[test]
fn regulated_pack_with_allowlisted_aaguid_accepts() {
    let aaguid = Aaguid([3u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let server = srv(aaguid, al);
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Carol",
            PackTier::PackRegulated,
            1_700_000_000,
        )
        .expect("begin");
    let cred = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackRegulated,
            &RegistrationResponse {
                challenge_id: chal.challenge_id,
                client_data_json_b64url: "ok".into(),
                attestation_object_b64url: "ok".into(),
                transports: vec![Transport::Usb],
            },
            1_700_000_000,
        )
        .expect("finish");
    assert_eq!(cred.aaguid, aaguid);
}

#[test]
fn rejects_attestation_when_adapter_denies() {
    let server = srv(Aaguid([4u8; 16]), BTreeSet::new());
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Dave",
            PackTier::SandboxOrDev,
            1_700_000_000,
        )
        .expect("begin");
    let err = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::SandboxOrDev,
            &RegistrationResponse {
                challenge_id: chal.challenge_id,
                client_data_json_b64url: "deny:reason".into(),
                attestation_object_b64url: "x".into(),
                transports: vec![],
            },
            1_700_000_000,
        )
        .unwrap_err();
    assert!(matches!(err, WebauthnError::AttestationInvalid(_)));
}

#[test]
fn rejects_sign_count_regression() {
    let aaguid = Aaguid([5u8; 16]);
    let server = srv(aaguid, BTreeSet::new());
    let chal = server
        .begin_registration(
            &tenant(),
            &user(),
            "Eve",
            PackTier::PackStandard,
            1_700_000_000,
        )
        .expect("begin");
    let cred = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::PackStandard,
            &RegistrationResponse {
                challenge_id: chal.challenge_id,
                client_data_json_b64url: "ok".into(),
                attestation_object_b64url: "ok".into(),
                transports: vec![Transport::Internal],
            },
            1_700_000_000,
        )
        .expect("finish");

    // First authentication: sign_count goes 0 → some N
    let auth_chal = server
        .begin_authentication(
            &tenant(),
            vec![cred.credential_id.clone()],
            Mediation::Optional,
            1_700_000_050,
        )
        .expect("begin");
    let r = server
        .finish_authentication(
            &tenant(),
            &AuthenticationResponse {
                challenge_id: auth_chal.challenge_id,
                credential_id: cred.credential_id.clone(),
                client_data_json_b64url: "count:10:end".into(),
                authenticator_data_b64url: "x".into(),
                signature_b64url: "x".into(),
                user_handle_b64url: None,
            },
            1_700_000_100,
        )
        .expect("auth1");
    assert_eq!(r.sign_count, 10);

    // Second authentication presenting LOWER sign count = cloned-authenticator alarm
    let auth_chal2 = server
        .begin_authentication(
            &tenant(),
            vec![cred.credential_id.clone()],
            Mediation::Optional,
            1_700_000_150,
        )
        .expect("begin");
    let err = server
        .finish_authentication(
            &tenant(),
            &AuthenticationResponse {
                challenge_id: auth_chal2.challenge_id,
                credential_id: cred.credential_id,
                client_data_json_b64url: "count:5:end".into(),
                authenticator_data_b64url: "x".into(),
                signature_b64url: "x".into(),
                user_handle_b64url: None,
            },
            1_700_000_200,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        WebauthnError::SignCountRegression {
            stored: 10,
            presented: 5
        }
    ));
}

#[test]
fn rejects_unknown_credential_at_authenticate() {
    let server = srv(Aaguid([6u8; 16]), BTreeSet::new());
    let chal = server
        .begin_authentication(&tenant(), vec![], Mediation::Conditional, 1_700_000_000)
        .expect("begin");
    let err = server
        .finish_authentication(
            &tenant(),
            &AuthenticationResponse {
                challenge_id: chal.challenge_id,
                credential_id: CredentialId(b"missing".to_vec()),
                client_data_json_b64url: "ok".into(),
                authenticator_data_b64url: "x".into(),
                signature_b64url: "x".into(),
                user_handle_b64url: None,
            },
            1_700_000_000,
        )
        .unwrap_err();
    assert!(matches!(err, WebauthnError::CredentialNotFound(_)));
}

#[test]
fn challenge_not_found_returns_distinct_error() {
    let server = srv(Aaguid([7u8; 16]), BTreeSet::new());
    let err = server
        .finish_registration(
            &tenant(),
            &user(),
            PackTier::SandboxOrDev,
            &RegistrationResponse {
                challenge_id: "nonexistent".into(),
                client_data_json_b64url: "ok".into(),
                attestation_object_b64url: "ok".into(),
                transports: vec![],
            },
            1_700_000_000,
        )
        .unwrap_err();
    assert!(matches!(err, WebauthnError::ChallengeNotFound(_)));
}

#[test]
fn conditional_ui_uses_empty_allow_credentials() {
    let server = srv(Aaguid([8u8; 16]), BTreeSet::new());
    let chal = server
        .begin_authentication(&tenant(), vec![], Mediation::Conditional, 1_700_000_000)
        .expect("begin");
    assert!(chal.allow_credentials.is_empty());
    assert!(matches!(chal.mediation, Mediation::Conditional));
}

#[test]
fn exclude_credentials_returns_user_existing_set() {
    let aaguid = Aaguid([9u8; 16]);
    let server = srv(aaguid, BTreeSet::new());

    // Register two credentials for the user.
    for _ in 0..2 {
        let chal = server
            .begin_registration(
                &tenant(),
                &user(),
                "Frank",
                PackTier::PackStandard,
                1_700_000_000,
            )
            .expect("begin");
        server
            .finish_registration(
                &tenant(),
                &user(),
                PackTier::PackStandard,
                &RegistrationResponse {
                    challenge_id: chal.challenge_id,
                    client_data_json_b64url: "ok".into(),
                    attestation_object_b64url: "ok".into(),
                    transports: vec![Transport::Internal],
                },
                1_700_000_000,
            )
            .expect("finish");
    }

    let chal3 = server
        .begin_registration(
            &tenant(),
            &user(),
            "Frank",
            PackTier::PackStandard,
            1_700_000_001,
        )
        .expect("begin");
    // The exclude_credentials in the 3rd challenge should already contain
    // the previous registrations.
    assert!(!chal3.exclude_credentials.is_empty());
}

// ---------------------------------------------------------------------------
// Attestation-policy gate: PackTier::admit_credential
// ---------------------------------------------------------------------------

/// Build a minimal `Credential` for policy-gate tests. No I/O; pure construction.
fn make_credential(aaguid: Aaguid) -> Credential {
    Credential {
        credential_id: CredentialId(b"test-cred".to_vec()),
        tenant_id: TenantId("t".into()),
        user_id: UserId("u".into()),
        public_key: CoseKeyCbor(vec![0x01]),
        aaguid,
        transports: vec![Transport::Internal],
        attestation_format: "packed".into(),
        backup_eligible: false,
        backup_state: false,
        sign_count: 0,
        last_used_at_unix: 0,
        created_at_unix: 0,
    }
}

#[test]
fn sandbox_or_dev_always_admits() {
    let cred = make_credential(Aaguid::ZERO); // even zero AAGUID is fine here
    assert!(
        PackTier::SandboxOrDev
            .admit_credential(AttestationConveyance::None, &cred, None)
            .is_ok()
    );
    // Higher conveyances also pass
    assert!(
        PackTier::SandboxOrDev
            .admit_credential(AttestationConveyance::Direct, &cred, None)
            .is_ok()
    );
}

#[test]
fn pack_standard_indirect_admits() {
    let cred = make_credential(Aaguid([1u8; 16]));
    assert!(
        PackTier::PackStandard
            .admit_credential(AttestationConveyance::Indirect, &cred, None)
            .is_ok()
    );
}

#[test]
fn pack_standard_none_rejects() {
    let cred = make_credential(Aaguid([1u8; 16]));
    let err = PackTier::PackStandard
        .admit_credential(AttestationConveyance::None, &cred, None)
        .unwrap_err();
    assert!(
        matches!(
            err,
            WebauthnError::AttestationLevelInsufficient {
                required: AttestationConveyance::Indirect,
                actual: AttestationConveyance::None,
            }
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn regulated_direct_allowlisted_admits() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    assert!(
        PackTier::PackRegulated
            .admit_credential(AttestationConveyance::Direct, &cred, Some(&al))
            .is_ok()
    );
}

#[test]
fn regulated_direct_zero_aaguid_rejects() {
    let aaguid = Aaguid::ZERO;
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    let err = PackTier::PackRegulated
        .admit_credential(AttestationConveyance::Direct, &cred, Some(&al))
        .unwrap_err();
    assert!(
        matches!(err, WebauthnError::AaguidNotAllowlisted(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn regulated_direct_not_in_allowlist_rejects() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(Aaguid([2u8; 16])); // different AAGUID in allowlist
    let cred = make_credential(aaguid);
    let err = PackTier::PackRegulated
        .admit_credential(AttestationConveyance::Direct, &cred, Some(&al))
        .unwrap_err();
    assert!(
        matches!(err, WebauthnError::AaguidNotAllowlisted(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn regulated_none_rejects_conveyance() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    let err = PackTier::PackRegulated
        .admit_credential(AttestationConveyance::None, &cred, Some(&al))
        .unwrap_err();
    assert!(
        matches!(err, WebauthnError::AttestationLevelInsufficient { .. }),
        "unexpected error: {err:?}"
    );
}

#[test]
fn critical_enterprise_admits() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    assert!(
        PackTier::AcrCritical
            .admit_credential(AttestationConveyance::Enterprise, &cred, Some(&al))
            .is_ok()
    );
}

#[test]
fn critical_direct_admits() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    assert!(
        PackTier::AcrCritical
            .admit_credential(AttestationConveyance::Direct, &cred, Some(&al))
            .is_ok()
    );
}

#[test]
fn critical_indirect_rejects() {
    let aaguid = Aaguid([1u8; 16]);
    let mut al = BTreeSet::new();
    al.insert(aaguid);
    let cred = make_credential(aaguid);
    let err = PackTier::AcrCritical
        .admit_credential(AttestationConveyance::Indirect, &cred, Some(&al))
        .unwrap_err();
    assert!(
        matches!(err, WebauthnError::AttestationLevelInsufficient { .. }),
        "unexpected error: {err:?}"
    );
}

#[test]
fn regulated_nil_allowlist_rejects() {
    let aaguid = Aaguid([1u8; 16]);
    let cred = make_credential(aaguid);
    let err = PackTier::PackRegulated
        .admit_credential(AttestationConveyance::Direct, &cred, None)
        .unwrap_err();
    assert!(
        matches!(err, WebauthnError::AaguidNotAllowlisted(_)),
        "unexpected error: {err:?}"
    );
}
