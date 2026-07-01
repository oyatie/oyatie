//! WebAuthn Level-3 relying-party server kernel.
//!
//! Authority: ADR-0188 (passkey/WebAuthn substrate), ADR-0187 (Zitadel IdP).
//!
//! This kernel exposes a vendor-neutral `WebauthnServer` trait + state
//! machines for registration and authentication ceremonies. The concrete
//! cryptographic and CBOR/COSE work is delegated to a [`WebauthnRpAdapter`]
//! (today wired to `webauthn-rs` v0.5+; per ADR-0188 §In-house roadmap, the
//! protocol is a W3C standard and the OSS library is commodity, so the
//! kernel stays adapter-agnostic to permit a Phase-2 swap if upstream
//! becomes unmaintained).
//!
//! The kernel handles the surface every µservice needs:
//!
//! - Begin/finish registration (server-side state stored as
//!   [`RegistrationChallenge`]).
//! - Begin/finish authentication (server-side state stored as
//!   [`AuthenticationChallenge`]).
//! - AAGUID allowlist enforcement (regulated packs).
//! - Sign-count monotonic-increase enforcement (replay defense per
//!   W3C WebAuthn §6.1.1).
//! - Credential backup state (BE / BS flags) tracking for sync detection.
//! - Conditional UI hint (mediation) on authentication-begin.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Tenant-scoped identifier; every credential and challenge is bound to one.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TenantId(pub String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CredentialId(pub Vec<u8>);

/// AAGUID per W3C WebAuthn §5.1 (128-bit authenticator-model identifier).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Aaguid(pub [u8; 16]);

impl Aaguid {
    pub const ZERO: Self = Self([0u8; 16]);
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 16]
    }
}

/// COSE_Key envelope (CBOR-encoded) — opaque to the kernel; the adapter
/// owns deserialisation + signature verification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoseKeyCbor(pub Vec<u8>);

/// Attestation transport (RFC 9268 sect 5.8.4 + W3C WebAuthn §5.8.4).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    Usb,
    Nfc,
    Ble,
    Internal,
    Hybrid, // caBLE / cross-device
    SmartCard,
}

/// Attestation conveyance preference per W3C WebAuthn §5.4.7.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttestationConveyance {
    None,
    Indirect,
    Direct,
    Enterprise,
}

/// Pack tier drives AAGUID allowlist + attestation enforcement.
/// Maps to ADR-0188 §"Attestation policy".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackTier {
    SandboxOrDev,
    PackStandard,
    PackRegulated,
    AcrCritical,
}

impl PackTier {
    pub fn required_attestation(self) -> AttestationConveyance {
        match self {
            Self::SandboxOrDev => AttestationConveyance::None,
            Self::PackStandard => AttestationConveyance::Indirect,
            Self::PackRegulated | Self::AcrCritical => AttestationConveyance::Direct,
        }
    }

    pub fn requires_aaguid_allowlist(self) -> bool {
        matches!(self, Self::PackRegulated | Self::AcrCritical)
    }

    /// Evaluate whether `credential` may be admitted for this pack tier.
    ///
    /// Returns `Ok(())` when all policy checks pass.
    ///
    /// # Errors
    ///
    /// - `WebauthnError::AttestationLevelInsufficient` — presented conveyance
    ///   is weaker than the tier requires (`None < Indirect < Direct < Enterprise`).
    /// - `WebauthnError::AaguidNotAllowlisted` — the tier requires an allowlist
    ///   and either the AAGUID is zero, no allowlist was supplied, or the AAGUID
    ///   is absent from the supplied allowlist.
    ///
    /// The `aaguid_allowlist` argument is ignored for tiers that do not require
    /// one (`SandboxOrDev`, `PackStandard`).
    pub fn admit_credential(
        self,
        presented_conveyance: AttestationConveyance,
        credential: &Credential,
        aaguid_allowlist: Option<&BTreeSet<Aaguid>>,
    ) -> Result<(), WebauthnError> {
        // Step 1: attestation-level check.
        let required = self.required_attestation();
        if conveyance_level(presented_conveyance) < conveyance_level(required) {
            return Err(WebauthnError::AttestationLevelInsufficient {
                required,
                actual: presented_conveyance,
            });
        }

        // Step 2: AAGUID allowlist check (regulated / critical tiers only).
        if self.requires_aaguid_allowlist() {
            if credential.aaguid.is_zero() {
                return Err(WebauthnError::AaguidNotAllowlisted(Aaguid::ZERO));
            }
            match aaguid_allowlist {
                None => return Err(WebauthnError::AaguidNotAllowlisted(credential.aaguid)),
                Some(list) if !list.contains(&credential.aaguid) => {
                    return Err(WebauthnError::AaguidNotAllowlisted(credential.aaguid));
                }
                Some(_) => {}
            }
        }

        Ok(())
    }
}

/// Numeric strength of an attestation conveyance for ordering comparisons.
/// `None=0 < Indirect=1 < Direct=2 < Enterprise=3`.
fn conveyance_level(c: AttestationConveyance) -> u8 {
    match c {
        AttestationConveyance::None => 0,
        AttestationConveyance::Indirect => 1,
        AttestationConveyance::Direct => 2,
        AttestationConveyance::Enterprise => 3,
    }
}

/// Persisted credential record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    pub credential_id: CredentialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub public_key: CoseKeyCbor,
    pub aaguid: Aaguid,
    pub transports: Vec<Transport>,
    pub attestation_format: String,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub sign_count: u32,
    pub last_used_at_unix: i64,
    pub created_at_unix: i64,
}

/// Mediation hint sent to the browser (W3C WebAuthn §5.1.4).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mediation {
    Silent,
    Optional,
    Required,
    Conditional, // autofill / passive
}

/// Returned to the browser by `begin_registration`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub challenge_id: String, // server-side cookie / session ID
    pub challenge_b64url: String,
    pub rp_id: String,
    pub rp_name: String,
    pub user_id: UserId,
    pub user_display_name: String,
    pub attestation: AttestationConveyance,
    pub timeout_ms: u32,
    pub exclude_credentials: Vec<CredentialId>, // prevent dup-register
}

/// Returned to the browser by `begin_authentication`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub challenge_id: String,
    pub challenge_b64url: String,
    pub rp_id: String,
    pub allow_credentials: Vec<CredentialId>, // empty = conditional UI
    pub mediation: Mediation,
    pub timeout_ms: u32,
    pub user_verification: UserVerification,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserVerification {
    Required,
    Preferred,
    Discouraged,
}

/// Browser-supplied registration response (post-attestation).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub challenge_id: String,
    pub client_data_json_b64url: String,
    pub attestation_object_b64url: String,
    pub transports: Vec<Transport>,
}

/// Browser-supplied authentication response (assertion).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub challenge_id: String,
    pub credential_id: CredentialId,
    pub client_data_json_b64url: String,
    pub authenticator_data_b64url: String,
    pub signature_b64url: String,
    pub user_handle_b64url: Option<String>,
}

/// Adapter contract — concrete cryptography + CBOR/COSE work happens here.
/// Wired to `webauthn-rs` v0.5+ today; ADR-0188 §In-house roadmap permits a
/// Phase-2 swap if upstream becomes unmaintained.
pub trait WebauthnRpAdapter: Send + Sync {
    fn generate_challenge(&self) -> Result<Vec<u8>, WebauthnError>;
    fn rp_id(&self) -> &str;
    fn rp_name(&self) -> &str;

    /// Validates attestation and returns the extracted credential. Performs
    /// AAGUID allowlist check if `allowlist.is_some()`.
    fn verify_registration(
        &self,
        challenge: &RegistrationChallenge,
        response: &RegistrationResponse,
        allowlist: Option<&BTreeSet<Aaguid>>,
    ) -> Result<Credential, WebauthnError>;

    /// Verifies an assertion. Returns the *new* sign-count (must be > stored).
    fn verify_authentication(
        &self,
        challenge: &AuthenticationChallenge,
        response: &AuthenticationResponse,
        stored: &Credential,
    ) -> Result<u32, WebauthnError>;
}

/// Failure-mode enum, distinct variants per error class for caller policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebauthnError {
    ChallengeGenerationFailed(String),
    ChallengeNotFound(String),
    ChallengeExpired,
    StorageUnavailable {
        operation: &'static str,
        detail: String,
    },
    AttestationInvalid(String),
    AaguidNotAllowlisted(Aaguid),
    AttestationLevelInsufficient {
        required: AttestationConveyance,
        actual: AttestationConveyance,
    },
    AssertionInvalid(String),
    SignCountRegression {
        stored: u32,
        presented: u32,
    },
    CredentialNotFound(CredentialId),
    TenantMismatch {
        expected: TenantId,
        actual: TenantId,
    },
    UserMismatch {
        expected: UserId,
        actual: UserId,
    },
    BackupStateNotPermitted,
    Internal(String),
}

impl std::fmt::Display for WebauthnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChallengeGenerationFailed(s) => {
                write!(f, "webauthn: challenge generation failed: {s}")
            }
            Self::ChallengeNotFound(c) => write!(f, "webauthn: challenge not found '{c}'"),
            Self::ChallengeExpired => write!(f, "webauthn: challenge expired"),
            Self::StorageUnavailable { operation, detail } => write!(
                f,
                "webauthn: storage unavailable during {operation}: {detail}"
            ),
            Self::AttestationInvalid(s) => write!(f, "webauthn: attestation invalid: {s}"),
            Self::AaguidNotAllowlisted(a) => write!(
                f,
                "webauthn: AAGUID {:?} not in allowlist for this pack tier",
                a.0
            ),
            Self::AttestationLevelInsufficient { required, actual } => write!(
                f,
                "webauthn: attestation level {actual:?} is below required {required:?}"
            ),
            Self::AssertionInvalid(s) => write!(f, "webauthn: assertion invalid: {s}"),
            Self::SignCountRegression { stored, presented } => write!(
                f,
                "webauthn: sign-count regression (stored={stored}, presented={presented}) — cloned authenticator?"
            ),
            Self::CredentialNotFound(_) => write!(f, "webauthn: credential not found"),
            Self::TenantMismatch { .. } => write!(f, "webauthn: tenant mismatch"),
            Self::UserMismatch { .. } => write!(f, "webauthn: user mismatch"),
            Self::BackupStateNotPermitted => {
                write!(f, "webauthn: backup state not permitted for this pack tier")
            }
            Self::Internal(s) => write!(f, "webauthn: internal: {s}"),
        }
    }
}

impl std::error::Error for WebauthnError {}

/// Server contract that µservice handlers call. Every method takes
/// `now_unix` explicitly — the kernel never reads the wall clock, so
/// callers can deterministically test TTL behaviour and the kernel is
/// `#![forbid(unsafe_code)]`-clean with no hidden time source.
pub trait WebauthnServer: Send + Sync {
    fn begin_registration(
        &self,
        tenant: &TenantId,
        user_id: &UserId,
        user_display_name: &str,
        pack_tier: PackTier,
        now_unix: i64,
    ) -> Result<RegistrationChallenge, WebauthnError>;

    fn finish_registration(
        &self,
        tenant: &TenantId,
        user_id: &UserId,
        pack_tier: PackTier,
        response: &RegistrationResponse,
        now_unix: i64,
    ) -> Result<Credential, WebauthnError>;

    fn begin_authentication(
        &self,
        tenant: &TenantId,
        allow_credentials: Vec<CredentialId>,
        mediation: Mediation,
        now_unix: i64,
    ) -> Result<AuthenticationChallenge, WebauthnError>;

    fn finish_authentication(
        &self,
        tenant: &TenantId,
        response: &AuthenticationResponse,
        now_unix: i64,
    ) -> Result<Credential, WebauthnError>;
}

/// Pluggable store contract (Postgres in production; in-memory in tests).
pub trait CredentialStore: Send + Sync {
    fn get(
        &self,
        tenant: &TenantId,
        cred: &CredentialId,
    ) -> Result<Option<Credential>, WebauthnError>;
    fn put(&self, cred: &Credential) -> Result<(), WebauthnError>;
    fn revoke(&self, tenant: &TenantId, cred: &CredentialId) -> Result<(), WebauthnError>;
    fn list_for_user(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Vec<Credential>, WebauthnError>;
}

/// In-memory store, suitable for tests + reference. Production uses a
/// Postgres-backed impl with RLS + per-tenant connection pooling.
pub struct InMemoryCredentialStore {
    inner: std::sync::Mutex<Vec<Credential>>,
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self {
            inner: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn get(
        &self,
        tenant: &TenantId,
        cred: &CredentialId,
    ) -> Result<Option<Credential>, WebauthnError> {
        let g = self
            .inner
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "credential.get",
                detail: e.to_string(),
            })?;
        Ok(g.iter()
            .find(|c| c.tenant_id == *tenant && c.credential_id == *cred)
            .cloned())
    }

    fn put(&self, cred: &Credential) -> Result<(), WebauthnError> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "credential.put",
                detail: e.to_string(),
            })?;
        if let Some(slot) = g
            .iter_mut()
            .find(|c| c.tenant_id == cred.tenant_id && c.credential_id == cred.credential_id)
        {
            *slot = cred.clone();
        } else {
            g.push(cred.clone());
        }
        Ok(())
    }

    fn revoke(&self, tenant: &TenantId, cred: &CredentialId) -> Result<(), WebauthnError> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "credential.revoke",
                detail: e.to_string(),
            })?;
        g.retain(|c| !(c.tenant_id == *tenant && c.credential_id == *cred));
        Ok(())
    }

    fn list_for_user(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Vec<Credential>, WebauthnError> {
        let g = self
            .inner
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "credential.list_for_user",
                detail: e.to_string(),
            })?;
        Ok(g.iter()
            .filter(|c| c.tenant_id == *tenant && c.user_id == *user)
            .cloned()
            .collect())
    }
}

/// Pluggable challenge store (TTL'd; in-memory in tests, Redis in
/// production).
pub trait ChallengeStore: Send + Sync {
    fn put_registration(
        &self,
        c: &RegistrationChallenge,
        now_unix: i64,
    ) -> Result<(), WebauthnError>;
    fn take_registration(
        &self,
        id: &str,
        now_unix: i64,
    ) -> Result<RegistrationChallenge, WebauthnError>;
    fn put_authentication(
        &self,
        c: &AuthenticationChallenge,
        now_unix: i64,
    ) -> Result<(), WebauthnError>;
    fn take_authentication(
        &self,
        id: &str,
        now_unix: i64,
    ) -> Result<AuthenticationChallenge, WebauthnError>;
}

#[derive(Default)]
pub struct InMemoryChallengeStore {
    reg: std::sync::Mutex<Vec<(RegistrationChallenge, i64)>>,
    auth: std::sync::Mutex<Vec<(AuthenticationChallenge, i64)>>,
}

const CHALLENGE_TTL_SECONDS: i64 = 300;

impl ChallengeStore for InMemoryChallengeStore {
    fn put_registration(
        &self,
        c: &RegistrationChallenge,
        now_unix: i64,
    ) -> Result<(), WebauthnError> {
        let mut g = self
            .reg
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "challenge.put_registration",
                detail: e.to_string(),
            })?;
        g.push((c.clone(), now_unix));
        Ok(())
    }
    fn take_registration(
        &self,
        id: &str,
        now_unix: i64,
    ) -> Result<RegistrationChallenge, WebauthnError> {
        let mut g = self
            .reg
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "challenge.take_registration",
                detail: e.to_string(),
            })?;
        let pos = g
            .iter()
            .position(|(c, _)| c.challenge_id == id)
            .ok_or_else(|| WebauthnError::ChallengeNotFound(id.to_owned()))?;
        let (c, stored_at) = g.remove(pos);
        if now_unix.saturating_sub(stored_at) > CHALLENGE_TTL_SECONDS {
            return Err(WebauthnError::ChallengeExpired);
        }
        Ok(c)
    }
    fn put_authentication(
        &self,
        c: &AuthenticationChallenge,
        now_unix: i64,
    ) -> Result<(), WebauthnError> {
        let mut g = self
            .auth
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "challenge.put_authentication",
                detail: e.to_string(),
            })?;
        g.push((c.clone(), now_unix));
        Ok(())
    }
    fn take_authentication(
        &self,
        id: &str,
        now_unix: i64,
    ) -> Result<AuthenticationChallenge, WebauthnError> {
        let mut g = self
            .auth
            .lock()
            .map_err(|e| WebauthnError::StorageUnavailable {
                operation: "challenge.take_authentication",
                detail: e.to_string(),
            })?;
        let pos = g
            .iter()
            .position(|(c, _)| c.challenge_id == id)
            .ok_or_else(|| WebauthnError::ChallengeNotFound(id.to_owned()))?;
        let (c, stored_at) = g.remove(pos);
        if now_unix.saturating_sub(stored_at) > CHALLENGE_TTL_SECONDS {
            return Err(WebauthnError::ChallengeExpired);
        }
        Ok(c)
    }
}

/// Reference WebAuthn server implementation; production deployments wire
/// the `WebauthnRpAdapter` to `webauthn-rs` v0.5+.
pub struct ReferenceWebauthnServer<A, S, C> {
    pub adapter: A,
    pub challenge_store: S,
    pub credential_store: C,
    pub aaguid_allowlist: BTreeSet<Aaguid>,
    pub challenge_timeout_ms: u32,
}

impl<A, S, C> ReferenceWebauthnServer<A, S, C>
where
    A: WebauthnRpAdapter,
    S: ChallengeStore,
    C: CredentialStore,
{
    pub fn new(adapter: A, challenge_store: S, credential_store: C) -> Self {
        Self {
            adapter,
            challenge_store,
            credential_store,
            aaguid_allowlist: BTreeSet::new(),
            challenge_timeout_ms: 60_000,
        }
    }
}

impl<A, S, C> WebauthnServer for ReferenceWebauthnServer<A, S, C>
where
    A: WebauthnRpAdapter,
    S: ChallengeStore,
    C: CredentialStore,
{
    fn begin_registration(
        &self,
        tenant: &TenantId,
        user_id: &UserId,
        user_display_name: &str,
        pack_tier: PackTier,
        now_unix: i64,
    ) -> Result<RegistrationChallenge, WebauthnError> {
        let challenge_bytes = self.adapter.generate_challenge()?;
        if challenge_bytes.is_empty() {
            return Err(WebauthnError::ChallengeGenerationFailed(
                "empty challenge bytes from key seam".into(),
            ));
        }
        let challenge_b64url = b64url_encode_local(&challenge_bytes);
        let exclude_credentials = self
            .credential_store
            .list_for_user(tenant, user_id)?
            .into_iter()
            .map(|c| c.credential_id)
            .collect();
        let chal = RegistrationChallenge {
            // The challenge_id MUST be unique per ceremony. Include the
            // WebAuthn challenge entropy itself so concurrent registration
            // begins for the same tenant/user in the same second cannot
            // collide in a challenge store keyed only by challenge_id.
            challenge_id: format!(
                "reg:{}:{}:{}:{}",
                tenant.0, user_id.0, now_unix, challenge_b64url
            ),
            challenge_b64url,
            rp_id: self.adapter.rp_id().to_owned(),
            rp_name: self.adapter.rp_name().to_owned(),
            user_id: user_id.clone(),
            user_display_name: user_display_name.to_owned(),
            attestation: pack_tier.required_attestation(),
            timeout_ms: self.challenge_timeout_ms,
            exclude_credentials,
        };
        self.challenge_store.put_registration(&chal, now_unix)?;
        Ok(chal)
    }

    fn finish_registration(
        &self,
        tenant: &TenantId,
        user_id: &UserId,
        pack_tier: PackTier,
        response: &RegistrationResponse,
        now_unix: i64,
    ) -> Result<Credential, WebauthnError> {
        let challenge = self
            .challenge_store
            .take_registration(&response.challenge_id, now_unix)?;
        if challenge.user_id != *user_id {
            return Err(WebauthnError::UserMismatch {
                expected: challenge.user_id.clone(),
                actual: user_id.clone(),
            });
        }
        let allowlist = if pack_tier.requires_aaguid_allowlist() {
            Some(&self.aaguid_allowlist)
        } else {
            None
        };
        let mut cred = self
            .adapter
            .verify_registration(&challenge, response, allowlist)?;
        cred.tenant_id = tenant.clone();
        cred.user_id = user_id.clone();
        cred.created_at_unix = now_unix;
        cred.last_used_at_unix = now_unix;
        self.credential_store.put(&cred)?;
        Ok(cred)
    }

    fn begin_authentication(
        &self,
        tenant: &TenantId,
        allow_credentials: Vec<CredentialId>,
        mediation: Mediation,
        now_unix: i64,
    ) -> Result<AuthenticationChallenge, WebauthnError> {
        let challenge_bytes = self.adapter.generate_challenge()?;
        if challenge_bytes.is_empty() {
            return Err(WebauthnError::ChallengeGenerationFailed(
                "empty challenge bytes from key seam".into(),
            ));
        }
        let suffix = b64url_encode_local(&challenge_bytes[..8.min(challenge_bytes.len())]);
        let chal = AuthenticationChallenge {
            challenge_id: format!("auth:{}:{}:{}", tenant.0, now_unix, suffix),
            challenge_b64url: b64url_encode_local(&challenge_bytes),
            rp_id: self.adapter.rp_id().to_owned(),
            allow_credentials,
            mediation,
            timeout_ms: self.challenge_timeout_ms,
            user_verification: UserVerification::Required,
        };
        self.challenge_store.put_authentication(&chal, now_unix)?;
        Ok(chal)
    }

    fn finish_authentication(
        &self,
        tenant: &TenantId,
        response: &AuthenticationResponse,
        now_unix: i64,
    ) -> Result<Credential, WebauthnError> {
        let challenge = self
            .challenge_store
            .take_authentication(&response.challenge_id, now_unix)?;
        let stored = self
            .credential_store
            .get(tenant, &response.credential_id)?
            .ok_or_else(|| WebauthnError::CredentialNotFound(response.credential_id.clone()))?;
        if stored.tenant_id != *tenant {
            return Err(WebauthnError::TenantMismatch {
                expected: tenant.clone(),
                actual: stored.tenant_id,
            });
        }
        let new_sign_count = self
            .adapter
            .verify_authentication(&challenge, response, &stored)?;
        if new_sign_count <= stored.sign_count && (stored.sign_count != 0 || new_sign_count != 0) {
            // sign_count == 0 stays 0 for cloned-authenticator-tolerant Yubikeys
            // (W3C WebAuthn §6.1.1 — implementations MAY use 0); only enforce
            // monotonic when both sides have a non-zero counter.
            if stored.sign_count != 0 {
                return Err(WebauthnError::SignCountRegression {
                    stored: stored.sign_count,
                    presented: new_sign_count,
                });
            }
        }
        let mut updated = stored;
        updated.sign_count = new_sign_count;
        updated.last_used_at_unix = now_unix;
        self.credential_store.put(&updated)?;
        Ok(updated)
    }
}

/// Local b64url encoder; intentionally duplicated to avoid a kernel-level
/// dependency on the oidc kernel (the two crates are independent
/// substrates).
fn b64url_encode_local(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(input.len() * 4 / 3 + 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input {
        buf = (buf << 8) | u32::from(b);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            out.push(ALPHABET[((buf >> bits) & 0x3F) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buf << (6 - bits)) & 0x3F) as usize] as char);
    }
    out
}
