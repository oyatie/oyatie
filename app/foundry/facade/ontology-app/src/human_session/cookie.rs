use axum::http::{HeaderMap, header::COOKIE};
use iam_identity_user_session::SessionRefusal;

pub(super) const SESSION_COOKIE: &str = "__Host-oya_session";

pub(super) fn valid_token(token: &str) -> bool {
    let Some(encoded) = token.strip_prefix("v1.") else {
        return false;
    };
    encoded.len() == 43
        && encoded
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        && encoded
            .as_bytes()
            .last()
            .is_some_and(|b| b"AEIMQUYcgkosw048".contains(b))
}

pub(crate) fn session_token(headers: &HeaderMap) -> Result<&str, SessionRefusal> {
    let mut found = None;
    let mut bytes = 0usize;
    for header in headers.get_all(COOKIE) {
        bytes += header.as_bytes().len();
        if bytes > 8192 {
            return Err(SessionRefusal::InvalidCredential);
        }
        let raw = header
            .to_str()
            .map_err(|_| SessionRefusal::InvalidCredential)?;
        for pair in raw.split(';') {
            let (name, value) = pair
                .trim()
                .split_once('=')
                .ok_or(SessionRefusal::InvalidCredential)?;
            if name == SESSION_COOKIE {
                if found.is_some() || !valid_token(value) {
                    return Err(SessionRefusal::InvalidCredential);
                }
                found = Some(value);
            }
        }
    }
    found.ok_or(SessionRefusal::Unauthenticated)
}
