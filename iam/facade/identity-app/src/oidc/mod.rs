//! OIDC subsystem: static JWKS document loading (RFC 7517 subset).
//!
//! Parses the deployment-mounted JWKS file into the validation-side
//! [`Jwks`] used by the workload OIDC adapter. Offline-first: keys are
//! resolved from this static document only — no network JWKS fetch, no
//! synchronous introspection (ADR-0536 IdP domain: offline credential
//! verification everywhere).
//!
//! Fail-closed parsing: an unknown `kty`/`crv` or a missing coordinate is an
//! error, never a silently-skipped key.

pub mod issuer;

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

use iam_identity_workload_oidc::{Jwk, Jwks};

/// RFC 7517 JWKS document (the subset the workload validator consumes).
#[derive(Debug, Deserialize)]
struct JwksDocument {
    keys: Vec<JwkEntry>,
}

/// One RFC 7517 JWK entry. Unknown fields are ignored per RFC 7517 §4.
#[derive(Debug, Deserialize)]
struct JwkEntry {
    kty: String,
    kid: String,
    #[serde(default)]
    alg: Option<String>,
    #[serde(default)]
    crv: Option<String>,
    // RSA
    #[serde(default)]
    n: Option<String>,
    #[serde(default)]
    e: Option<String>,
    // EC / OKP
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y: Option<String>,
}

/// A JWKS document that could not be loaded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JwksParseError {
    /// The document is not valid JSON of the expected shape.
    Malformed(String),
    /// A key uses an unsupported `kty`/`crv` combination.
    UnsupportedKeyType { kid: String, detail: String },
    /// A key is missing a required parameter for its type.
    MissingParameter {
        kid: String,
        parameter: &'static str,
    },
    /// A present key parameter is malformed for its key type.
    InvalidParameter {
        kid: String,
        parameter: &'static str,
        detail: String,
    },
    /// The document contains duplicate key identifiers. Duplicate `kid`s make
    /// key selection ambiguous, so the loader refuses them fail-closed.
    DuplicateKeyId(String),
    /// The document contains no keys (a fail-closed refusal: a service with
    /// zero verification keys can never validate a token).
    Empty,
}

impl fmt::Display for JwksParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(detail) => write!(f, "malformed JWKS document: {detail}"),
            Self::UnsupportedKeyType { kid, detail } => {
                write!(f, "unsupported key type for kid {kid}: {detail}")
            }
            Self::MissingParameter { kid, parameter } => {
                write!(f, "JWK {kid} is missing required parameter {parameter}")
            }
            Self::InvalidParameter {
                kid,
                parameter,
                detail,
            } => {
                write!(f, "JWK {kid} has invalid parameter {parameter}: {detail}")
            }
            Self::DuplicateKeyId(kid) => write!(f, "JWKS document contains duplicate kid {kid}"),
            Self::Empty => write!(f, "JWKS document contains no keys"),
        }
    }
}

impl std::error::Error for JwksParseError {}

/// Parse an RFC 7517 JWKS JSON document into the adapter [`Jwks`].
///
/// # Errors
/// Returns [`JwksParseError`] on malformed JSON, an empty key set, an
/// unsupported key type, or a missing key parameter.
pub fn jwks_from_json(document: &str) -> Result<Jwks, JwksParseError> {
    let parsed: JwksDocument =
        serde_json::from_str(document).map_err(|e| JwksParseError::Malformed(e.to_string()))?;
    if parsed.keys.is_empty() {
        return Err(JwksParseError::Empty);
    }
    let mut jwks = Jwks::new();
    let mut seen = std::collections::BTreeSet::new();
    for entry in parsed.keys {
        if entry.kid.trim().is_empty() {
            return Err(JwksParseError::InvalidParameter {
                kid: entry.kid,
                parameter: "kid",
                detail: "kid must be non-empty".to_string(),
            });
        }
        if !seen.insert(entry.kid.clone()) {
            return Err(JwksParseError::DuplicateKeyId(entry.kid));
        }
        jwks = jwks.add_key(jwk_from_entry(entry)?);
    }
    Ok(jwks)
}

fn jwk_from_entry(entry: JwkEntry) -> Result<Jwk, JwksParseError> {
    let require = |value: Option<String>, parameter: &'static str, kid: &str| {
        value.ok_or_else(|| JwksParseError::MissingParameter {
            kid: kid.to_string(),
            parameter,
        })
    };
    let kid = entry.kid.clone();
    let jwk = match entry.kty.as_str() {
        "RSA" => {
            let n = require(entry.n, "n", &kid)?;
            let e = require(entry.e, "e", &kid)?;
            validate_b64url_min_len(&kid, "n", &n, 128)?;
            validate_rsa_exponent(&kid, &e)?;
            validate_alg(&kid, entry.alg.as_deref(), &["RS256", "RS384", "RS512"])?;
            Jwk::rsa(&kid, n, e)
        }
        "EC" => match entry.crv.as_deref() {
            Some("P-256") => {
                let x = require(entry.x, "x", &kid)?;
                let y = require(entry.y, "y", &kid)?;
                validate_b64url_exact_len(&kid, "x", &x, 32)?;
                validate_b64url_exact_len(&kid, "y", &y, 32)?;
                validate_alg(&kid, entry.alg.as_deref(), &["ES256"])?;
                Jwk::ec_p256(&kid, x, y)
            }
            other => {
                return Err(JwksParseError::UnsupportedKeyType {
                    kid,
                    detail: format!("EC curve {}", other.unwrap_or("<absent>")),
                });
            }
        },
        "OKP" => match entry.crv.as_deref() {
            Some("Ed25519") => {
                let x = require(entry.x, "x", &kid)?;
                validate_b64url_exact_len(&kid, "x", &x, 32)?;
                validate_alg(&kid, entry.alg.as_deref(), &["EdDSA"])?;
                Jwk::okp_ed25519(&kid, x)
            }
            other => {
                return Err(JwksParseError::UnsupportedKeyType {
                    kid,
                    detail: format!("OKP curve {}", other.unwrap_or("<absent>")),
                });
            }
        },
        other => {
            return Err(JwksParseError::UnsupportedKeyType {
                kid,
                detail: format!("kty {other}"),
            });
        }
    };
    Ok(match entry.alg {
        Some(alg) => jwk.with_alg(alg),
        None => jwk,
    })
}

fn decode_b64url(
    kid: &str,
    parameter: &'static str,
    value: &str,
) -> Result<Vec<u8>, JwksParseError> {
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|e| JwksParseError::InvalidParameter {
            kid: kid.to_string(),
            parameter,
            detail: format!("base64url decode failed: {e}"),
        })
}

fn validate_b64url_exact_len(
    kid: &str,
    parameter: &'static str,
    value: &str,
    expected: usize,
) -> Result<(), JwksParseError> {
    let decoded = decode_b64url(kid, parameter, value)?;
    if decoded.len() == expected {
        Ok(())
    } else {
        Err(JwksParseError::InvalidParameter {
            kid: kid.to_string(),
            parameter,
            detail: format!("expected {expected} bytes, got {}", decoded.len()),
        })
    }
}

fn validate_b64url_min_len(
    kid: &str,
    parameter: &'static str,
    value: &str,
    minimum: usize,
) -> Result<(), JwksParseError> {
    let decoded = decode_b64url(kid, parameter, value)?;
    if decoded.len() >= minimum {
        Ok(())
    } else {
        Err(JwksParseError::InvalidParameter {
            kid: kid.to_string(),
            parameter,
            detail: format!("expected at least {minimum} bytes, got {}", decoded.len()),
        })
    }
}

fn validate_rsa_exponent(kid: &str, value: &str) -> Result<(), JwksParseError> {
    let decoded = decode_b64url(kid, "e", value)?;
    match decoded.as_slice() {
        [3] | [1, 0, 1] => Ok(()),
        _ => Err(JwksParseError::InvalidParameter {
            kid: kid.to_string(),
            parameter: "e",
            detail: "expected exponent 3 or 65537".to_string(),
        }),
    }
}

fn validate_alg(kid: &str, alg: Option<&str>, allowed: &[&str]) -> Result<(), JwksParseError> {
    if let Some(alg) = alg
        && !allowed.iter().any(|allowed| allowed == &alg)
    {
        return Err(JwksParseError::InvalidParameter {
            kid: kid.to_string(),
            parameter: "alg",
            detail: format!("alg {alg} is not valid for this key type"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b64(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    #[test]
    fn parses_ec_p256_key() {
        let x = b64(&[0x11; 32]);
        let y = b64(&[0x22; 32]);
        let document = format!(
            r#"{{"keys":[
            {{"kty":"EC","crv":"P-256","kid":"kid-1","alg":"ES256","x":"{x}","y":"{y}"}}
        ]}}"#
        );
        jwks_from_json(&document).expect("parses");
    }

    #[test]
    fn parses_rsa_and_okp_keys() {
        let n = b64(&[0xA5; 256]);
        let okp_x = b64(&[0x33; 32]);
        let document = format!(
            r#"{{"keys":[
            {{"kty":"RSA","kid":"kid-rsa","alg":"RS256","n":"{n}","e":"AQAB"}},
            {{"kty":"OKP","crv":"Ed25519","kid":"kid-okp","alg":"EdDSA","x":"{okp_x}"}}
        ]}}"#
        );
        jwks_from_json(&document).expect("parses");
    }

    #[test]
    fn refuses_empty_key_set() {
        assert_eq!(jwks_from_json(r#"{"keys":[]}"#), Err(JwksParseError::Empty));
    }

    #[test]
    fn refuses_unknown_kty() {
        let document = r#"{"keys":[{"kty":"oct","kid":"kid-sym","x":"eHg"}]}"#;
        assert!(matches!(
            jwks_from_json(document),
            Err(JwksParseError::UnsupportedKeyType { .. })
        ));
    }

    #[test]
    fn refuses_unsupported_ec_curve() {
        let document =
            r#"{"keys":[{"kty":"EC","crv":"P-384","kid":"kid-384","x":"eHg","y":"eXk"}]}"#;
        assert!(matches!(
            jwks_from_json(document),
            Err(JwksParseError::UnsupportedKeyType { .. })
        ));
    }

    #[test]
    fn refuses_missing_coordinate() {
        let document = r#"{"keys":[{"kty":"EC","crv":"P-256","kid":"kid-1","x":"eHg"}]}"#;
        assert_eq!(
            jwks_from_json(document),
            Err(JwksParseError::MissingParameter {
                kid: "kid-1".into(),
                parameter: "y"
            })
        );
    }

    #[test]
    fn refuses_short_or_malformed_key_material() {
        let short = r#"{"keys":[
            {"kty":"EC","crv":"P-256","kid":"kid-1","alg":"ES256","x":"eHg","y":"eXk"}
        ]}"#;
        assert!(matches!(
            jwks_from_json(short),
            Err(JwksParseError::InvalidParameter { parameter: "x", .. })
        ));

        let malformed = r#"{"keys":[
            {"kty":"OKP","crv":"Ed25519","kid":"kid-okp","alg":"EdDSA","x":"not base64!"}
        ]}"#;
        assert!(matches!(
            jwks_from_json(malformed),
            Err(JwksParseError::InvalidParameter { parameter: "x", .. })
        ));
    }

    #[test]
    fn refuses_alg_mismatch_and_duplicate_kids() {
        let x = b64(&[0x11; 32]);
        let y = b64(&[0x22; 32]);
        let wrong_alg = format!(
            r#"{{"keys":[{{"kty":"EC","crv":"P-256","kid":"kid-1","alg":"RS256","x":"{x}","y":"{y}"}}]}}"#
        );
        assert!(matches!(
            jwks_from_json(&wrong_alg),
            Err(JwksParseError::InvalidParameter {
                parameter: "alg",
                ..
            })
        ));

        let duplicate = format!(
            r#"{{"keys":[
              {{"kty":"EC","crv":"P-256","kid":"kid-1","alg":"ES256","x":"{x}","y":"{y}"}},
              {{"kty":"EC","crv":"P-256","kid":"kid-1","alg":"ES256","x":"{x}","y":"{y}"}}
            ]}}"#
        );
        assert_eq!(
            jwks_from_json(&duplicate),
            Err(JwksParseError::DuplicateKeyId("kid-1".into()))
        );
    }

    #[test]
    fn refuses_malformed_json() {
        assert!(matches!(
            jwks_from_json("not json"),
            Err(JwksParseError::Malformed(_))
        ));
    }
}
