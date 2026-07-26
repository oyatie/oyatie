#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for `oya-identity-oidc-issuer-kernel`.
//!
//! These tests exercise the pure-domain surface of IP-002: issuer metadata,
//! JWKS publication filtering, signing-key lifecycle monotonicity, ID-token /
//! access-token claim construction, bounded clock-skew tolerance, and
//! structural validation of `client_assertion` / refresh requests.
//!
//! No crypto, no clock, no I/O — every fact is deterministic.

use std::collections::BTreeMap;

use iam_identity_oidc_issuer_kernel::{
    AccessTokenSpec, AcrLevel, ActiveIntrospectionClaims, Algorithm, Audience, ClientAssertion,
    ClockSkewTolerance, ID_TOKEN_CLAIMS_SCHEMA_VERSION, IdTokenSpec, IntrospectionRequest,
    IntrospectionResponse, IssuerError, IssuerUrl, JwsSigner, MAX_ACCESS_TOKEN_TTL_SECONDS,
    MAX_CLOCK_SKEW_SECONDS, MAX_ID_TOKEN_TTL_SECONDS, RefreshRequest, Signature, SigningKey,
    SigningKeyState, Subject, TokenTypeHint, VERIFICATION_GRACE_SECONDS, VerificationGrace,
    build_access_token_claims, build_id_token_claims, build_introspection_response,
    build_issuer_metadata, build_jwks, check_temporal_window, current_signing_key,
    select_verification_key,
};

fn rsa_components() -> BTreeMap<String, String> {
    let mut components = BTreeMap::new();
    components.insert("n".to_owned(), "dummy-modulus".to_owned());
    components.insert("e".to_owned(), "AQAB".to_owned());
    components
}

fn ec_components() -> BTreeMap<String, String> {
    let mut components = BTreeMap::new();
    components.insert("crv".to_owned(), "P-256".to_owned());
    components.insert("x".to_owned(), "x-coord".to_owned());
    components.insert("y".to_owned(), "y-coord".to_owned());
    components
}

fn issuer() -> IssuerUrl {
    IssuerUrl::new("https://identity-kr.oyatie.com").expect("issuer ok")
}

/// Stub signer that produces a deterministic signature based on the kid.
struct StubSigner;
impl JwsSigner for StubSigner {
    fn sign(&self, signing_input: &[u8], kid: &str) -> Result<Signature, IssuerError> {
        if kid.is_empty() {
            return Err(IssuerError::InvalidKid);
        }
        let len = signing_input.len();
        Signature::new(format!("stub-{kid}-{len}"))
    }
}

#[test]
fn issuer_metadata_includes_required_oidc_fields() {
    let meta = build_issuer_metadata(issuer(), &[Algorithm::Rs256, Algorithm::Es256]).expect("ok");
    assert_eq!(meta.issuer.as_str(), "https://identity-kr.oyatie.com");
    assert_eq!(
        meta.jwks_uri,
        "https://identity-kr.oyatie.com/oauth/v2/keys".to_owned()
    );
    assert_eq!(
        meta.authorization_endpoint,
        "https://identity-kr.oyatie.com/oauth/authorize".to_owned()
    );
    assert_eq!(
        meta.token_endpoint,
        "https://identity-kr.oyatie.com/oauth/v2/token".to_owned()
    );
    assert!(meta.userinfo_endpoint.is_some());
    assert!(
        meta.response_types_supported.contains(&"code".to_owned()),
        "authorization-code response type is reserved for future provider metadata"
    );
    assert!(
        !meta
            .response_types_supported
            .contains(&"id_token".to_owned()),
        "implicit flow must not be declared"
    );
    assert!(
        meta.scopes_supported.contains(&"openid".to_owned()),
        "openid scope must be supported"
    );
    assert!(
        meta.acr_values_supported.contains(&"critical".to_owned()),
        "all ACR levels must be advertised"
    );
    assert_eq!(
        meta.id_token_signing_alg_values_supported,
        vec!["RS256", "ES256"]
    );
    assert!(
        meta.grant_types_supported
            .contains(&"refresh_token".to_owned()),
        "refresh_token grant must be advertised"
    );
}

#[test]
fn issuer_metadata_strips_trailing_slash() {
    let url = IssuerUrl::new("https://identity-eu.oyatie.com/").expect("ok");
    let meta = build_issuer_metadata(url, &[Algorithm::Rs256]).expect("ok");
    assert_eq!(
        meta.jwks_uri,
        "https://identity-eu.oyatie.com/oauth/v2/keys".to_owned(),
        "trailing slash should be normalised"
    );
}

#[test]
fn jwks_contains_only_active_and_rotated_out_keys() {
    let mut active = SigningKey::provision("k-active", Algorithm::Rs256, rsa_components())
        .expect("provision ok");
    active.activate(1_700_000_000).expect("activate ok");

    let pre =
        SigningKey::provision("k-future", Algorithm::Es256, ec_components()).expect("provision ok");

    let mut rotated = SigningKey::provision("k-rotated", Algorithm::Es256, ec_components())
        .expect("provision ok");
    rotated.activate(1_700_000_100).expect("activate ok");
    rotated.rotate_out().expect("rotate ok");

    let mut retired = SigningKey::provision("k-retired", Algorithm::Rs256, rsa_components())
        .expect("provision ok");
    retired.activate(1_700_000_200).expect("activate ok");
    retired.rotate_out().expect("rotate ok");
    retired.retire().expect("retire ok");

    let jwks = build_jwks(&[active, pre, rotated, retired]);
    let kids: Vec<&str> = jwks.keys().iter().map(|key| key.kid.as_str()).collect();
    // Only Active + RotatedOut are published; sorted by kid (deterministic).
    assert_eq!(kids, vec!["k-active", "k-rotated"]);
    // NotYetActive must NOT appear.
    assert!(jwks.find("k-future").is_none());
    // Retired must NOT appear.
    assert!(jwks.find("k-retired").is_none());

    let active_jwk = jwks.find("k-active").expect("present");
    assert_eq!(active_jwk.kty, "RSA");
    assert_eq!(active_jwk.alg, "RS256");
    assert_eq!(active_jwk.key_use, "sig");
    assert_eq!(
        active_jwk.public_components.get("e"),
        Some(&"AQAB".to_owned())
    );

    let rotated_jwk = jwks.find("k-rotated").expect("present");
    assert_eq!(rotated_jwk.kty, "EC");
    assert_eq!(rotated_jwk.alg, "ES256");
    assert_eq!(
        rotated_jwk.public_components.get("crv"),
        Some(&"P-256".to_owned())
    );
}

#[test]
fn signing_key_transitions_are_monotonic() {
    let mut key =
        SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("provision ok");
    assert_eq!(key.state(), SigningKeyState::NotYetActive);

    // Forward transitions succeed.
    key.activate(1_700_000_000).expect("activate ok");
    assert_eq!(key.state(), SigningKeyState::Active);
    key.rotate_out().expect("rotate ok");
    assert_eq!(key.state(), SigningKeyState::RotatedOut);
    key.retire().expect("retire ok");
    assert_eq!(key.state(), SigningKeyState::Retired);

    // No revival from Retired (terminal).
    assert!(matches!(
        key.rotate_out(),
        Err(IssuerError::IllegalKeyTransition { .. })
    ));

    // No revival from RotatedOut → Active.
    let mut other = SigningKey::provision("k2", Algorithm::Rs256, rsa_components()).expect("ok");
    other.activate(1).expect("ok");
    other.rotate_out().expect("ok");
    assert!(matches!(
        other.activate(2),
        Err(IssuerError::IllegalKeyTransition { .. })
    ));
}

#[test]
fn current_signing_key_returns_active_only() {
    let pre = SigningKey::provision("k0", Algorithm::Rs256, rsa_components()).expect("ok");
    let mut active = SigningKey::provision("k1", Algorithm::Rs256, rsa_components()).expect("ok");
    active.activate(1).expect("ok");
    let mut rotated = SigningKey::provision("k2", Algorithm::Rs256, rsa_components()).expect("ok");
    rotated.activate(2).expect("ok");
    rotated.rotate_out().expect("ok");
    let keys = vec![pre, active, rotated];
    let current = current_signing_key(&keys).expect("active present");
    assert_eq!(current.kid(), "k1");
    // No active key → None.
    let only_retired =
        vec![SigningKey::provision("k-x", Algorithm::Rs256, rsa_components()).expect("ok")];
    assert!(current_signing_key(&only_retired).is_none());
}

#[test]
fn id_token_claims_include_iss_aud_sub_iat_exp_nonce() {
    let claims = build_id_token_claims(IdTokenSpec {
        issuer: issuer(),
        audience: Audience::single("oya-application").expect("ok"),
        subject: Subject::new("usr_abc").expect("ok"),
        tenant_id: "ten_acme".to_owned(),
        issued_at_epoch_seconds: 1_700_000_000,
        expires_at_epoch_seconds: 1_700_003_600,
        nonce: "n-xyz".to_owned(),
        acr: AcrLevel::Sensitive,
        purpose: Some("login".to_owned()),
        data_class: Some("INTERNAL_ONLY".to_owned()),
    })
    .expect("ok");
    // Required OIDC claims:
    assert_eq!(claims.iss, "https://identity-kr.oyatie.com");
    assert_eq!(claims.aud, vec!["oya-application".to_owned()]);
    assert_eq!(claims.sub, "usr_abc");
    assert_eq!(claims.iat, 1_700_000_000);
    assert_eq!(claims.exp, 1_700_003_600);
    assert_eq!(claims.nonce, "n-xyz");
    // ADR-0244 + ADR-0189 supersets:
    assert_eq!(claims.tenant_id, "ten_acme");
    assert_eq!(claims.acr, "sensitive");
    assert_eq!(claims.purpose.as_deref(), Some("login"));
    assert_eq!(claims.data_class.as_deref(), Some("INTERNAL_ONLY"));
    // nbf defaults to iat.
    assert_eq!(claims.nbf, claims.iat);
    assert_eq!(claims.schema_version, ID_TOKEN_CLAIMS_SCHEMA_VERSION);
}

#[test]
fn id_token_rejects_malformed_inputs() {
    let issuer_url = issuer();
    let audience = Audience::single("aud").expect("ok");
    let subject = Subject::new("usr_x").expect("ok");

    // iat must be > 0.
    let spec = IdTokenSpec {
        issuer: issuer_url.clone(),
        audience: audience.clone(),
        subject: subject.clone(),
        tenant_id: "ten_acme".to_owned(),
        issued_at_epoch_seconds: 0,
        expires_at_epoch_seconds: 100,
        nonce: "n".to_owned(),
        acr: AcrLevel::Routine,
        purpose: None,
        data_class: None,
    };
    assert_eq!(
        build_id_token_claims(spec),
        Err(IssuerError::InvalidIssuedAt)
    );

    // exp must be strictly > iat.
    let spec = IdTokenSpec {
        issuer: issuer_url.clone(),
        audience: audience.clone(),
        subject: subject.clone(),
        tenant_id: "ten_acme".to_owned(),
        issued_at_epoch_seconds: 100,
        expires_at_epoch_seconds: 100,
        nonce: "n".to_owned(),
        acr: AcrLevel::Routine,
        purpose: None,
        data_class: None,
    };
    assert!(matches!(
        build_id_token_claims(spec),
        Err(IssuerError::InvalidExpiry { .. })
    ));

    // Lifetime ceiling.
    let spec = IdTokenSpec {
        issuer: issuer_url,
        audience,
        subject,
        tenant_id: "ten_acme".to_owned(),
        issued_at_epoch_seconds: 1,
        expires_at_epoch_seconds: MAX_ID_TOKEN_TTL_SECONDS + 100,
        nonce: "n".to_owned(),
        acr: AcrLevel::Routine,
        purpose: None,
        data_class: None,
    };
    assert!(matches!(
        build_id_token_claims(spec),
        Err(IssuerError::TokenLifetimeTooLong { .. })
    ));
}

#[test]
fn access_token_claims_carry_scopes_and_token_type() {
    let claims = build_access_token_claims(AccessTokenSpec {
        issuer: issuer(),
        audience: Audience::many(vec!["oya-foundry".to_owned(), "oya-ops".to_owned()]).expect("ok"),
        subject: Subject::new("usr_abc").expect("ok"),
        tenant_id: "ten_acme".to_owned(),
        scopes: vec!["openid".to_owned(), "email".to_owned()],
        issued_at_epoch_seconds: 1_700_000_000,
        expires_at_epoch_seconds: 1_700_003_500,
        purpose: Some("api".to_owned()),
        data_class: Some("PUBLIC".to_owned()),
    })
    .expect("ok");
    assert_eq!(claims.iss, "https://identity-kr.oyatie.com");
    assert_eq!(claims.aud, vec!["oya-foundry", "oya-ops"]);
    assert_eq!(claims.scope, "openid email");
    assert_eq!(claims.token_type, "at+jwt");
    assert_eq!(claims.tenant_id, "ten_acme");
}

#[test]
fn access_token_lifetime_ceiling_enforced() {
    let result = build_access_token_claims(AccessTokenSpec {
        issuer: issuer(),
        audience: Audience::single("oya-foundry").expect("ok"),
        subject: Subject::new("usr_x").expect("ok"),
        tenant_id: "ten_acme".to_owned(),
        scopes: vec!["openid".to_owned()],
        issued_at_epoch_seconds: 1,
        expires_at_epoch_seconds: MAX_ACCESS_TOKEN_TTL_SECONDS + 10,
        purpose: None,
        data_class: None,
    });
    assert!(matches!(
        result,
        Err(IssuerError::TokenLifetimeTooLong { .. })
    ));
}

#[test]
fn clock_skew_tolerance_is_bounded() {
    // Within range works.
    let skew = ClockSkewTolerance::new(60).expect("ok");
    assert_eq!(skew.seconds(), 60);

    // Negative rejected.
    assert_eq!(
        ClockSkewTolerance::new(-1),
        Err(IssuerError::NegativeClockSkew)
    );

    // Above ceiling rejected.
    let ceiling_overflow = MAX_CLOCK_SKEW_SECONDS + 1;
    match ClockSkewTolerance::new(ceiling_overflow) {
        Err(IssuerError::ClockSkewTooWide {
            requested_seconds,
            ceiling_seconds,
        }) => {
            assert_eq!(requested_seconds, ceiling_overflow);
            assert_eq!(ceiling_seconds, MAX_CLOCK_SKEW_SECONDS);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn check_temporal_window_within_skew() {
    let skew = ClockSkewTolerance::new(60).expect("ok");
    // exact-boundary: now == exp + skew is still inside.
    assert!(check_temporal_window(260, 50, 200, skew).is_ok());
    // one-second past skew tolerance -> Expired.
    assert!(matches!(
        check_temporal_window(261, 50, 200, skew),
        Err(IssuerError::Expired { .. })
    ));
    // not-yet-valid: now + skew < nbf -> NotYetValid.
    assert!(matches!(
        check_temporal_window(0, 100, 200, skew),
        Err(IssuerError::NotYetValid { .. })
    ));
    // 39 + 60 = 99 < 100 → still NotYetValid.
    assert!(matches!(
        check_temporal_window(39, 100, 200, skew),
        Err(IssuerError::NotYetValid { .. })
    ));
    // 40 + 60 = 100 ≥ 100 → ok.
    assert!(check_temporal_window(40, 100, 200, skew).is_ok());
}

#[test]
fn client_assertion_parse_rejects_malformed() {
    // Happy path.
    let ok = ClientAssertion::parse("aaa.bbb.ccc").expect("ok");
    assert_eq!(ok.signing_input(), "aaa.bbb");

    // Wrong segment count.
    assert!(matches!(
        ClientAssertion::parse("aaa.bbb"),
        Err(IssuerError::MalformedClientAssertion(_))
    ));
    assert!(matches!(
        ClientAssertion::parse("aaa.bbb.ccc.ddd"),
        Err(IssuerError::MalformedClientAssertion(_))
    ));

    // Empty segment.
    assert!(matches!(
        ClientAssertion::parse(".bbb.ccc"),
        Err(IssuerError::MalformedClientAssertion(_))
    ));
    assert!(matches!(
        ClientAssertion::parse("aaa..ccc"),
        Err(IssuerError::MalformedClientAssertion(_))
    ));
    assert!(matches!(
        ClientAssertion::parse("aaa.bbb."),
        Err(IssuerError::MalformedClientAssertion(_))
    ));

    // Empty input.
    assert!(matches!(
        ClientAssertion::parse(""),
        Err(IssuerError::MalformedClientAssertion(_))
    ));
}

#[test]
fn refresh_request_validates_required_fields() {
    let req = RefreshRequest::validate("rt-token", "client-1", None).expect("ok");
    assert_eq!(req.refresh_token, "rt-token");
    assert_eq!(req.client_id, "client-1");
    assert!(req.requested_scope.is_none());

    // Empty refresh_token rejected.
    assert!(matches!(
        RefreshRequest::validate("", "client-1", None),
        Err(IssuerError::MalformedRefreshRequest(_))
    ));
    // Empty client_id rejected.
    assert!(matches!(
        RefreshRequest::validate("rt", "", None),
        Err(IssuerError::MalformedRefreshRequest(_))
    ));
    // Whitespace-only scope when provided rejected.
    assert!(matches!(
        RefreshRequest::validate("rt", "client", Some("   ".to_owned())),
        Err(IssuerError::MalformedRefreshRequest(_))
    ));
}

#[test]
fn symmetric_algorithms_are_refused_at_issuer() {
    assert_eq!(
        Algorithm::parse("HS256"),
        Err(IssuerError::SymmetricAlgorithmForbidden)
    );
    assert_eq!(
        Algorithm::parse("HS384"),
        Err(IssuerError::SymmetricAlgorithmForbidden)
    );
    assert_eq!(
        Algorithm::parse("HS512"),
        Err(IssuerError::SymmetricAlgorithmForbidden)
    );
}

#[test]
fn jws_signer_port_is_pluggable() {
    let signer = StubSigner;
    let signing_input = b"header.payload";
    let sig = signer.sign(signing_input, "k1").expect("ok");
    assert!(sig.as_str().starts_with("stub-k1-"));
    // The signer is responsible for kid checks; missing kid → InvalidKid.
    assert_eq!(signer.sign(signing_input, ""), Err(IssuerError::InvalidKid));
}

#[test]
fn acr_values_supported_advertise_full_ladder() {
    let meta = build_issuer_metadata(issuer(), &[Algorithm::Rs256]).expect("ok");
    let expected = vec!["routine", "elevated", "sensitive", "critical"];
    let advertised: Vec<&str> = meta
        .acr_values_supported
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(advertised, expected);
}

#[test]
fn audience_single_and_many_reject_empty() {
    assert!(matches!(
        Audience::single(""),
        Err(IssuerError::InvalidAudience)
    ));
    assert!(matches!(
        Audience::many(vec![]),
        Err(IssuerError::InvalidAudience)
    ));
    assert!(matches!(
        Audience::many(vec!["a".to_owned(), "".to_owned()]),
        Err(IssuerError::InvalidAudience)
    ));
    let aud = Audience::many(vec!["a".to_owned(), "b".to_owned()]).expect("ok");
    assert!(aud.contains("a"));
    assert!(aud.contains("b"));
    assert!(!aud.contains("c"));
}

// ── RFC 7662 Token Introspection ─────────────────────────────────────────────

fn access_token_claims_for_introspection() -> iam_identity_oidc_issuer_kernel::AccessTokenClaims {
    build_access_token_claims(AccessTokenSpec {
        issuer: issuer(),
        audience: Audience::single("oya-api").expect("ok"),
        subject: Subject::new("usr_abc").expect("ok"),
        tenant_id: "ten_acme".to_owned(),
        scopes: vec!["openid".to_owned(), "email".to_owned()],
        issued_at_epoch_seconds: 1_700_000_000,
        expires_at_epoch_seconds: 1_700_003_600,
        purpose: None,
        data_class: None,
    })
    .expect("ok")
}

#[test]
fn introspection_request_rejects_empty_and_accepts_typed_hints() {
    // Empty token rejected.
    assert!(matches!(
        IntrospectionRequest::validate("", None),
        Err(IssuerError::MalformedIntrospectionRequest(_))
    ));
    // Whitespace-only token rejected.
    assert!(matches!(
        IntrospectionRequest::validate("   ", None),
        Err(IssuerError::MalformedIntrospectionRequest(_))
    ));
    // Valid token without hint accepted.
    let req = IntrospectionRequest::validate("opaque-token", None).expect("ok");
    assert_eq!(req.token, "opaque-token");
    assert_eq!(req.token_type_hint, None);
    // Valid token with access_token hint accepted.
    let req2 = IntrospectionRequest::validate("tok", Some(TokenTypeHint::AccessToken)).expect("ok");
    assert_eq!(req2.token_type_hint, Some(TokenTypeHint::AccessToken));
    // Valid token with refresh_token hint accepted.
    let req3 =
        IntrospectionRequest::validate("tok", Some(TokenTypeHint::RefreshToken)).expect("ok");
    assert_eq!(req3.token_type_hint, Some(TokenTypeHint::RefreshToken));
}

#[test]
fn introspection_response_inactive_discloses_nothing() {
    let resp = IntrospectionResponse::inactive();
    assert!(!resp.active, "inactive response must have active=false");
    assert!(resp.sub.is_none());
    assert!(resp.aud.is_none());
    assert!(resp.exp.is_none());
    assert!(resp.iat.is_none());
    assert!(resp.scope.is_none());
    assert!(resp.client_id.is_none());
    assert!(resp.tenant_id.is_none());
    assert!(resp.token_type.is_none());
}

#[test]
fn build_introspection_response_active_token_discloses_claims() {
    let claims = access_token_claims_for_introspection();
    let now = 1_700_001_800_i64; // well within validity window
    let skew = ClockSkewTolerance::new(60).expect("ok");
    let resp = build_introspection_response(&claims, now, skew).expect("ok");
    assert!(resp.active);
    assert_eq!(resp.sub.as_deref(), Some("usr_abc"));
    assert_eq!(resp.aud.as_deref(), Some(["oya-api".to_owned()].as_slice()));
    assert_eq!(resp.exp, Some(1_700_003_600));
    assert_eq!(resp.iat, Some(1_700_000_000));
    assert_eq!(resp.scope.as_deref(), Some("openid email"));
    assert_eq!(
        resp.client_id, None,
        "kernel has no client_id in AccessTokenClaims"
    );
    assert_eq!(resp.tenant_id.as_deref(), Some("ten_acme"));
    assert_eq!(resp.token_type.as_deref(), Some("at+jwt"));
}

#[test]
fn build_introspection_response_expired_collapses_to_inactive() {
    let claims = access_token_claims_for_introspection();
    // now > exp(1_700_003_600) + skew(60) → expired
    let now = 1_700_003_661_i64;
    let skew = ClockSkewTolerance::new(60).expect("ok");
    let resp = build_introspection_response(&claims, now, skew).expect("ok");
    assert!(!resp.active);
    assert!(resp.sub.is_none());
    assert!(resp.exp.is_none());
}

#[test]
fn build_introspection_response_not_yet_valid_collapses_to_inactive() {
    let claims = access_token_claims_for_introspection();
    // nbf == iat == 1_700_000_000; now + skew(60) < nbf → not yet valid
    let now = 1_699_999_939_i64; // 1_699_999_939 + 60 = 1_699_999_999 < 1_700_000_000
    let skew = ClockSkewTolerance::new(60).expect("ok");
    let resp = build_introspection_response(&claims, now, skew).expect("ok");
    assert!(!resp.active);
    assert!(resp.sub.is_none());
    assert!(resp.exp.is_none());
}

// ── select_verification_key integration tests ─────────────────────────────────

fn active_key(kid: &str, activated_at: i64) -> SigningKey {
    let mut k = SigningKey::provision(kid, Algorithm::Rs256, rsa_components()).expect("ok");
    k.activate(activated_at).expect("ok");
    k
}

fn rotated_key(kid: &str, activated_at: i64) -> SigningKey {
    let mut k = active_key(kid, activated_at);
    k.rotate_out().expect("ok");
    k
}

fn retired_key(kid: &str, activated_at: i64) -> SigningKey {
    let mut k = rotated_key(kid, activated_at);
    k.retire().expect("ok");
    k
}

#[test]
fn verification_key_active_accept() {
    // Active key is accepted unconditionally regardless of how old it is.
    let keys = vec![active_key("k-active", 1_700_000_000)];
    let grace = VerificationGrace::new(3_600).expect("ok");
    let found = select_verification_key(&keys, "k-active", 1_799_999_999, grace);
    assert_eq!(found.map(|k| k.kid()), Some("k-active"));
}

#[test]
fn verification_key_rotated_within_grace_accept() {
    // activated_at=1_000, now=1_000+3_600=4_600, grace=3_600 → age==grace → accept.
    let keys = vec![rotated_key("k-rotated", 1_000)];
    let grace = VerificationGrace::new(3_600).expect("ok");
    let found = select_verification_key(&keys, "k-rotated", 4_600, grace);
    assert_eq!(found.map(|k| k.kid()), Some("k-rotated"));
}

#[test]
fn verification_key_rotated_past_grace_reject() {
    // activated_at=1_000, now=4_601, grace=3_600 → age=3_601 > grace → None.
    let keys = vec![rotated_key("k-rotated", 1_000)];
    let grace = VerificationGrace::new(3_600).expect("ok");
    let found = select_verification_key(&keys, "k-rotated", 4_601, grace);
    assert!(found.is_none());
}

#[test]
fn verification_key_retired_reject() {
    // Retired keys are hard-rejected regardless of grace.
    let keys = vec![retired_key("k-retired", 1_000)];
    let grace = VerificationGrace::new(VERIFICATION_GRACE_SECONDS).expect("ok");
    assert!(select_verification_key(&keys, "k-retired", 2_000, grace).is_none());
}

#[test]
fn verification_key_not_yet_active_reject() {
    // NotYetActive keys are hard-rejected.
    let keys =
        vec![SigningKey::provision("k-pre", Algorithm::Rs256, rsa_components()).expect("ok")];
    let grace = VerificationGrace::new(VERIFICATION_GRACE_SECONDS).expect("ok");
    assert!(select_verification_key(&keys, "k-pre", 2_000, grace).is_none());
}

#[test]
fn verification_key_unknown_kid_none() {
    // kid not present in bundle → None.
    let keys = vec![active_key("k1", 1_000)];
    let grace = VerificationGrace::new(3_600).expect("ok");
    assert!(select_verification_key(&keys, "k-missing", 2_000, grace).is_none());
}

#[test]
fn verification_grace_ceiling_bound_rejection() {
    match VerificationGrace::new(VERIFICATION_GRACE_SECONDS + 1) {
        Err(IssuerError::GracePeriodTooLong {
            requested_seconds,
            ceiling_seconds,
        }) => {
            assert_eq!(requested_seconds, VERIFICATION_GRACE_SECONDS + 1);
            assert_eq!(ceiling_seconds, VERIFICATION_GRACE_SECONDS);
        }
        other => panic!("expected GracePeriodTooLong, got {other:?}"),
    }
}

#[test]
fn verification_grace_at_ceiling_accepted() {
    assert!(
        VerificationGrace::new(VERIFICATION_GRACE_SECONDS).is_ok(),
        "ceiling value itself must be accepted"
    );
}

#[test]
fn verification_grace_negative_rejected() {
    assert_eq!(
        VerificationGrace::new(-1),
        Err(IssuerError::NegativeGracePeriod)
    );
}

#[test]
fn verification_grace_zero_boundary() {
    // grace=0: RotatedOut key accepted only when now == activated_at (age=0).
    let keys = vec![rotated_key("k1", 1_000)];
    let grace = VerificationGrace::new(0).expect("ok");
    // Exact boundary: age=0 ≤ grace(0) → accept.
    assert!(select_verification_key(&keys, "k1", 1_000, grace).is_some());
    // One second past: age=1 > grace(0) → reject.
    assert!(select_verification_key(&keys, "k1", 1_001, grace).is_none());
}
