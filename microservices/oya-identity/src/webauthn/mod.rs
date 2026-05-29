//! WebAuthn subsystem for oya-identity.
//!
//! Phase-1 RP per ADR-0507 (webauthn-rs); Tier-2 bespoke destination = oya-webauthn.
//! Single-crate-per-service pattern per ADR-0509.

use webauthn_rs::prelude::*;

pub struct WebAuthnService {
    rp: Webauthn,
}

impl WebAuthnService {
    /// Build a WebAuthnService for the given relying-party origin.
    /// Origin example: `https://identity.oyatie.example` — passed from config.
    pub fn new(rp_id: &str, rp_origin: &str, rp_name: &str) -> Result<Self, WebauthnError> {
        let rp_origin = Url::parse(rp_origin).map_err(|_| WebauthnError::Configuration)?;
        let rp = WebauthnBuilder::new(rp_id, &rp_origin)?
            .rp_name(rp_name)
            .build()?;
        Ok(Self { rp })
    }

    /// Begin a passkey registration ceremony.
    /// Returns the CreationChallengeResponse to send to the browser
    /// and the PasskeyRegistration state to persist (server-side).
    pub fn start_registration(
        &self,
        user_unique_id: Uuid,
        user_name: &str,
        user_display_name: &str,
        existing_credentials: Option<Vec<CredentialID>>,
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration), WebauthnError> {
        self.rp.start_passkey_registration(
            user_unique_id,
            user_name,
            user_display_name,
            existing_credentials,
        )
    }

    /// Finish a passkey registration ceremony.
    pub fn finish_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<Passkey, WebauthnError> {
        self.rp.finish_passkey_registration(reg, state)
    }

    /// Begin an authentication ceremony.
    pub fn start_authentication(
        &self,
        credentials: &[Passkey],
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), WebauthnError> {
        self.rp.start_passkey_authentication(credentials)
    }

    /// Finish an authentication ceremony.
    pub fn finish_authentication(
        &self,
        auth: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> Result<AuthenticationResult, WebauthnError> {
        self.rp.finish_passkey_authentication(auth, state)
    }
}

pub fn init() {
    // TODO(ADR-0476): real init hooked into composition root + config
}
