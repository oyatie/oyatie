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

use serde::Deserialize;

use oya_identity_workload_oidc_adapter::{Jwk, Jwks};

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
    MissingParameter { kid: String, parameter: &'static str },
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
    for entry in parsed.keys {
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
        "RSA" => Jwk::rsa(
            &kid,
            require(entry.n, "n", &kid)?,
            require(entry.e, "e", &kid)?,
        ),
        "EC" => match entry.crv.as_deref() {
            Some("P-256") => Jwk::ec_p256(
                &kid,
                require(entry.x, "x", &kid)?,
                require(entry.y, "y", &kid)?,
            ),
            other => {
                return Err(JwksParseError::UnsupportedKeyType {
                    kid,
                    detail: format!("EC curve {}", other.unwrap_or("<absent>")),
                });
            }
        },
        "OKP" => match entry.crv.as_deref() {
            Some("Ed25519") => Jwk::okp_ed25519(&kid, require(entry.x, "x", &kid)?),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ec_p256_key() {
        let document = r#"{"keys":[
            {"kty":"EC","crv":"P-256","kid":"kid-1","alg":"ES256","x":"eHg","y":"eXk"}
        ]}"#;
        jwks_from_json(document).expect("parses");
    }

    #[test]
    fn parses_rsa_and_okp_keys() {
        let document = r#"{"keys":[
            {"kty":"RSA","kid":"kid-rsa","n":"bm4","e":"AQAB"},
            {"kty":"OKP","crv":"Ed25519","kid":"kid-okp","x":"eHg"}
        ]}"#;
        jwks_from_json(document).expect("parses");
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
        let document = r#"{"keys":[{"kty":"EC","crv":"P-384","kid":"kid-384","x":"eHg","y":"eXk"}]}"#;
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
    fn refuses_malformed_json() {
        assert!(matches!(
            jwks_from_json("not json"),
            Err(JwksParseError::Malformed(_))
        ));
    }
}
