use axum::http::{HeaderMap, HeaderValue, header};

pub(super) const NAME: &str = "__Host-oya_session";

fn valid_token(token: &str) -> bool {
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

/// Other cookies never cross this boundary; duplicate session cookies refuse.
pub(super) fn session_cookie(headers: &HeaderMap) -> Result<Option<String>, ()> {
    let mut found = None;
    let mut size = 0;
    for value in headers.get_all(header::COOKIE) {
        size += value.as_bytes().len();
        if size > 8192 {
            return Err(());
        }
        for pair in value.to_str().map_err(|_| ())?.split(';') {
            let (name, token) = pair.trim().split_once('=').ok_or(())?;
            if name == NAME {
                if found.is_some() || !valid_token(token) {
                    return Err(());
                }
                found = Some(format!("{NAME}={token}"));
            }
        }
    }
    Ok(found)
}

/// Admit only the exact native host cookie and its required protections.
pub(super) fn checked_set_cookie(headers: &HeaderMap, logout: bool) -> Result<HeaderValue, ()> {
    let mut values = headers.get_all(header::SET_COOKIE).iter();
    let value = values.next().ok_or(())?;
    if values.next().is_some() {
        return Err(());
    }
    let mut parts = value.to_str().map_err(|_| ())?.split(';').map(str::trim);
    let (name, token) = parts.next().and_then(|v| v.split_once('=')).ok_or(())?;
    if name != NAME
        || if logout {
            !token.is_empty()
        } else {
            !valid_token(token)
        }
    {
        return Err(());
    }
    let expected = [
        "path=/",
        "secure",
        "httponly",
        "samesite=strict",
        if logout { "max-age=0" } else { "max-age=3600" },
    ];
    let mut seen = [false; 5];
    for part in parts {
        let lower = part.to_ascii_lowercase();
        let index = expected.iter().position(|v| *v == lower).ok_or(())?;
        if seen[index] {
            return Err(());
        }
        seen[index] = true;
    }
    if !seen.into_iter().all(|v| v) {
        return Err(());
    }
    Ok(value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookies_refuse_duplicates_noncanonical_tokens_and_weakened_protections() {
        let token = format!("v1.{}", "A".repeat(43));
        let good =
            format!("{NAME}={token}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=3600");
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            format!("other=value; {NAME}={token}").parse().unwrap(),
        );
        assert_eq!(session_cookie(&h), Ok(Some(format!("{NAME}={token}"))));
        h.append(header::COOKIE, format!("{NAME}={token}").parse().unwrap());
        assert!(session_cookie(&h).is_err());
        for invalid in [
            good.replace("; Secure", ""),
            good.replace("Path=/", "Path=/api"),
            good.replace("SameSite=Strict", "SameSite=Lax"),
            format!("{good}; Domain=example.com"),
            good.replace(&token, &format!("v1.{}B", "A".repeat(42))),
            format!("{good}; Secure"),
        ] {
            let mut h = HeaderMap::new();
            h.insert(header::SET_COOKIE, invalid.parse().unwrap());
            assert!(checked_set_cookie(&h, false).is_err());
        }
        let mut h = HeaderMap::new();
        h.insert(header::SET_COOKIE, good.parse().unwrap());
        assert!(checked_set_cookie(&h, false).is_ok());
        h.append(header::SET_COOKIE, good.parse().unwrap());
        assert!(checked_set_cookie(&h, false).is_err());
    }
}
