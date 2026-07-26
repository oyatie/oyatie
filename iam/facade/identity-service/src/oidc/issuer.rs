//! OIDC issuer delivery surface over `oya-identity-oidc-issuer-kernel`.
//!
//! RFC 7517 JWKS publication (`/oauth/v2/keys`, the kernel's canonical
//! `jwks_uri` path) + an RFC 9068 access-token mint use-case. The kernel owns
//! every policy decision (claim validation, TTL ceilings, key lifecycle, JWKS
//! filtering); this module only serializes and signs.
//!
//! ## Signing custody (G02 attach point)
//! [`Es256FileSigner`] implements the kernel's [`JwsSigner`] PORT over a
//! deployment-mounted PKCS#8 key — the transitional custody adapter. The G02
//! KMS lane replaces it behind the SAME trait (per-tenant KEKs, enclave
//! custody); nothing in this module changes at that cutover.

use std::collections::BTreeMap;
use std::sync::Arc;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use iam_identity_oidc_issuer_kernel::{
    AccessTokenClaims, AccessTokenSpec, Algorithm, Audience, IssuerError, IssuerUrl, JwsSigner,
    Signature, SigningKey, Subject, build_access_token_claims, build_jwks, current_signing_key,
};

/// `GET` — published JWKS (the kernel's canonical `jwks_uri` path).
pub const JWKS_ROUTE: &str = "/oauth/v2/keys";
/// `GET` — legacy JWKS alias retained during client migration.
pub const LEGACY_JWKS_ROUTE: &str = "/oauth/jwks";

// =====================================================================
// ES256 file signer (transitional custody behind the JwsSigner port)
// =====================================================================

/// [`JwsSigner`] over a single ES256 PKCS#8 (DER) key. Construction parses and
/// validates the key, so `sign` failures are limited to kid mismatch (typed)
/// and the practically-unreachable RNG failure (mapped to the kernel's
/// documented malformed-signature sentinel).
pub struct Es256FileSigner {
    kid: String,
    key_pair: EcdsaKeyPair,
    rng: SystemRandom,
}

// Manual Debug: the key pair must never reach a log line; only the kid is
// shown (EcdsaKeyPair has no Debug impl either way).
impl std::fmt::Debug for Es256FileSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Es256FileSigner")
            .field("kid", &self.kid)
            .finish_non_exhaustive()
    }
}

impl Es256FileSigner {
    /// Parse a PKCS#8 DER ES256 private key.
    ///
    /// # Errors
    /// Returns [`IssuerError::InvalidKid`] for an empty kid and
    /// [`IssuerError::MalformedClientAssertion`] (the kernel's malformed-key
    /// sentinel) when the DER is not a valid P-256 key.
    pub fn from_pkcs8_der(kid: impl Into<String>, der: &[u8]) -> Result<Self, IssuerError> {
        let kid = kid.into();
        if kid.trim().is_empty() {
            return Err(IssuerError::InvalidKid);
        }
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, der)
            .map_err(|_| IssuerError::MalformedClientAssertion("invalid PKCS#8 ES256 key"))?;
        Ok(Self {
            kid,
            key_pair,
            rng: SystemRandom::new(),
        })
    }

    /// The signer's key id.
    #[must_use]
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// RFC 7517 public components (`crv`/`x`/`y`) for [`SigningKey::provision`].
    #[must_use]
    pub fn public_components(&self) -> BTreeMap<String, String> {
        let public = self.key_pair.public_key().as_ref();
        // Uncompressed SEC1 point: 0x04 || X (32 bytes) || Y (32 bytes).
        let mut components = BTreeMap::new();
        components.insert("crv".to_owned(), "P-256".to_owned());
        if public.len() == 65 {
            components.insert("x".to_owned(), URL_SAFE_NO_PAD.encode(&public[1..33]));
            components.insert("y".to_owned(), URL_SAFE_NO_PAD.encode(&public[33..65]));
        }
        components
    }

    /// Provision the kernel-side [`SigningKey`] record for this signer.
    ///
    /// # Errors
    /// Propagates [`IssuerError::InvalidKid`] from the kernel.
    pub fn signing_key(&self) -> Result<SigningKey, IssuerError> {
        SigningKey::provision(&self.kid, Algorithm::Es256, self.public_components())
    }
}

impl JwsSigner for Es256FileSigner {
    fn sign(&self, signing_input: &[u8], kid: &str) -> Result<Signature, IssuerError> {
        if kid != self.kid {
            return Err(IssuerError::InvalidKid);
        }
        let signature = self
            .key_pair
            .sign(&self.rng, signing_input)
            .map_err(|_| IssuerError::MalformedClientAssertion("jws signing failed"))?;
        Signature::new(URL_SAFE_NO_PAD.encode(signature.as_ref()))
    }
}

// =====================================================================
// Issuer state + mint use-case
// =====================================================================

/// Issuer-side state: the issuer identity, the kernel key bundle, and the
/// signer behind the [`JwsSigner`] port.
pub struct IssuerState {
    issuer_url: IssuerUrl,
    keys: Vec<SigningKey>,
    signer: Arc<dyn JwsSigner>,
    now_provider: fn() -> i64,
}

/// Default access-token TTL (five minutes — workload tokens are short-lived;
/// revocation is CAEP + denylist, not long-lived credentials).
pub const DEFAULT_ACCESS_TOKEN_TTL_SECONDS: i64 = 300;

impl IssuerState {
    /// Assemble issuer state. `keys` should contain the signer's activated
    /// [`SigningKey`] (plus any rotated-out keys still published for
    /// verification grace).
    pub fn new(
        issuer_url: IssuerUrl,
        keys: Vec<SigningKey>,
        signer: Arc<dyn JwsSigner>,
        now_provider: fn() -> i64,
    ) -> Self {
        Self {
            issuer_url,
            keys,
            signer,
            now_provider,
        }
    }

    /// Mint a signed RFC 9068 workload access token (`typ: at+jwt`).
    ///
    /// # Errors
    /// Propagates kernel claim-validation errors ([`IssuerError`]) and signer
    /// failures; refuses when no key is in the signing state.
    pub fn mint_access_token(
        &self,
        subject: &str,
        tenant_id: &str,
        audience: &str,
        scopes: Vec<String>,
        ttl_seconds: i64,
    ) -> Result<String, IssuerError> {
        let key = current_signing_key(&self.keys).ok_or(IssuerError::InvalidKid)?;
        let now = (self.now_provider)();
        let claims = build_access_token_claims(AccessTokenSpec {
            issuer: self.issuer_url.clone(),
            audience: Audience::single(audience)?,
            subject: Subject::new(subject)?,
            tenant_id: tenant_id.to_owned(),
            scopes,
            issued_at_epoch_seconds: now,
            expires_at_epoch_seconds: now.saturating_add(ttl_seconds),
            purpose: None,
            data_class: None,
        })?;
        let header = serde_json::json!({
            "alg": key.algorithm().as_str(),
            "typ": "at+jwt",
            "kid": key.kid(),
        });
        let payload = access_token_claims_json(&claims);
        let signing_input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.to_string().as_bytes()),
            URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes()),
        );
        let signature = self.signer.sign(signing_input.as_bytes(), key.kid())?;
        Ok(format!("{signing_input}.{}", signature.as_str()))
    }
}

/// Serialize [`AccessTokenClaims`] to its RFC 9068 JSON object. Optional
/// claims are omitted (never `null`) so validators applying RFC 8725 strict
/// parsing see only present members.
fn access_token_claims_json(claims: &AccessTokenClaims) -> serde_json::Value {
    let mut object = serde_json::json!({
        "iss": claims.iss,
        "aud": claims.aud,
        "sub": claims.sub,
        "iat": claims.iat,
        "exp": claims.exp,
        "nbf": claims.nbf,
        "scope": claims.scope,
        "tenant_id": claims.tenant_id,
        "token_type": claims.token_type,
    });
    if let (Some(purpose), serde_json::Value::Object(map)) = (&claims.purpose, &mut object) {
        map.insert("purpose".to_owned(), serde_json::json!(purpose));
    }
    if let (Some(data_class), serde_json::Value::Object(map)) = (&claims.data_class, &mut object) {
        map.insert("data_class".to_owned(), serde_json::json!(data_class));
    }
    object
}

// =====================================================================
// Routes
// =====================================================================

/// Shared handle for the issuer routes.
pub type SharedIssuerState = Arc<IssuerState>;

async fn jwks_handler(State(state): State<SharedIssuerState>) -> Json<serde_json::Value> {
    let jwks = build_jwks(&state.keys);
    let keys: Vec<serde_json::Value> = jwks
        .keys()
        .iter()
        .map(|key| {
            let mut entry = serde_json::json!({
                "kid": key.kid,
                "kty": key.kty,
                "alg": key.alg,
                "use": key.key_use,
            });
            if let serde_json::Value::Object(map) = &mut entry {
                for (name, value) in &key.public_components {
                    map.insert(name.clone(), serde_json::json!(value));
                }
            }
            entry
        })
        .collect();
    Json(serde_json::json!({ "keys": keys }))
}

/// Build the issuer router (JWKS publication).
pub fn build_issuer_router(state: SharedIssuerState) -> Router {
    Router::new()
        .route(JWKS_ROUTE, get(jwks_handler))
        .route(LEGACY_JWKS_ROUTE, get(jwks_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED, UnparsedPublicKey};
    use axum::http::StatusCode;

    const ISSUER: &str = "https://idp.oyatie.com";
    const KID: &str = "oya-identity-k1";

    fn test_signer() -> Es256FileSigner {
        let rng = SystemRandom::new();
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
        Es256FileSigner::from_pkcs8_der(KID, pkcs8.as_ref()).expect("signer")
    }

    fn test_state(signer: Es256FileSigner) -> IssuerState {
        let mut key = signer.signing_key().expect("signing key");
        key.activate(1_700_000_000).expect("activate");
        IssuerState::new(
            IssuerUrl::new(ISSUER).expect("issuer url"),
            vec![key],
            Arc::new(signer),
            || 1_700_000_100,
        )
    }

    #[test]
    fn refuses_invalid_pkcs8() {
        assert!(Es256FileSigner::from_pkcs8_der(KID, b"not a key").is_err());
        assert_eq!(
            Es256FileSigner::from_pkcs8_der("  ", b"x").unwrap_err(),
            IssuerError::InvalidKid
        );
    }

    #[test]
    fn signer_refuses_foreign_kid() {
        let signer = test_signer();
        assert_eq!(
            signer.sign(b"input", "other-kid").unwrap_err(),
            IssuerError::InvalidKid
        );
    }

    #[test]
    fn public_components_carry_p256_point() {
        let components = test_signer().public_components();
        assert_eq!(components.get("crv").map(String::as_str), Some("P-256"));
        assert!(components.contains_key("x"));
        assert!(components.contains_key("y"));
    }

    #[test]
    fn minted_token_verifies_against_published_jwks() {
        let signer = test_signer();
        let state = test_state(signer);

        let token = state
            .mint_access_token(
                "wl_secrets_sync",
                "ten_acme",
                "oya-cloud-kms",
                vec!["cloud.kms.decrypt".into()],
                DEFAULT_ACCESS_TOKEN_TTL_SECONDS,
            )
            .expect("mint");

        let mut parts = token.split('.');
        let (header_b64, payload_b64, signature_b64) = (
            parts.next().expect("header"),
            parts.next().expect("payload"),
            parts.next().expect("signature"),
        );
        assert!(parts.next().is_none(), "exactly three JWS segments");

        let header: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(header_b64).expect("b64"))
                .expect("header json");
        assert_eq!(header["alg"], "ES256");
        assert_eq!(header["typ"], "at+jwt");
        assert_eq!(header["kid"], KID);

        let payload: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload_b64).expect("b64"))
                .expect("payload json");
        assert_eq!(payload["iss"], ISSUER);
        assert_eq!(payload["sub"], "wl_secrets_sync");
        assert_eq!(payload["tenant_id"], "ten_acme");
        assert_eq!(payload["scope"], "cloud.kms.decrypt");
        assert_eq!(payload["exp"], 1_700_000_100i64 + 300);
        assert!(payload.get("purpose").is_none(), "absent, never null");

        // Verify the signature against the PUBLISHED JWKS components — the
        // full issue->publish->verify loop with no shared private state.
        let jwks = build_jwks(&state.keys);
        let published = jwks.find(KID).expect("published key");
        let x = URL_SAFE_NO_PAD
            .decode(published.public_components.get("x").expect("x"))
            .expect("x b64");
        let y = URL_SAFE_NO_PAD
            .decode(published.public_components.get("y").expect("y"))
            .expect("y b64");
        let mut point = vec![0x04];
        point.extend_from_slice(&x);
        point.extend_from_slice(&y);
        let signing_input = format!("{header_b64}.{payload_b64}");
        UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, &point)
            .verify(
                signing_input.as_bytes(),
                &URL_SAFE_NO_PAD.decode(signature_b64).expect("sig b64"),
            )
            .expect("signature verifies against published JWKS");
    }

    #[test]
    fn mint_refuses_when_no_key_is_signing() {
        let signer = test_signer();
        // Key bundle left NotYetActive: nothing is in the signing state.
        let key = signer.signing_key().expect("signing key");
        let state = IssuerState::new(
            IssuerUrl::new(ISSUER).expect("issuer url"),
            vec![key],
            Arc::new(signer),
            || 1_700_000_100,
        );
        assert_eq!(
            state
                .mint_access_token("wl_x", "ten_a", "aud", vec![], 300)
                .unwrap_err(),
            IssuerError::InvalidKid
        );
    }

    #[tokio::test]
    async fn discovery_is_unmounted_while_jwks_remains_available() {
        use tower::ServiceExt as _;

        let signer = test_signer();
        let state = Arc::new(test_state(signer));
        let router = build_issuer_router(state);

        let response = router
            .clone()
            .oneshot(
                axum::http::Request::get("/.well-known/openid-configuration")
                    .body(axum::body::Body::empty())
                    .expect("request"),
            )
            .await
            .expect("discovery responds");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = router
            .oneshot(
                axum::http::Request::get(JWKS_ROUTE)
                    .body(axum::body::Body::empty())
                    .expect("request"),
            )
            .await
            .expect("jwks responds");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .expect("body");
        let document: serde_json::Value = serde_json::from_slice(&body).expect("json");
        let keys = document["keys"].as_array().expect("keys");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0]["kid"], KID);
        assert_eq!(keys[0]["use"], "sig");
        assert_eq!(keys[0]["crv"], "P-256");
    }
}
