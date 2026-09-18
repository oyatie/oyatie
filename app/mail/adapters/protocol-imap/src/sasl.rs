use base64::{Engine, engine::general_purpose::STANDARD};

pub(crate) fn plain(response: &[u8]) -> Option<(String, String, String)> {
    let decoded = STANDARD.decode(response).ok()?;
    let decoded = std::str::from_utf8(&decoded).ok()?;
    let mut fields = decoded.split('\0');
    let authorization = fields.next()?;
    let username = fields.next()?;
    let token = fields.next()?;
    if fields.next().is_some() || username.is_empty() || token.is_empty() {
        return None;
    }
    Some((authorization.into(), username.into(), token.into()))
}

/// Decodes an OAUTHBEARER client response (RFC 7628 §3.1): a GS2 header
/// optionally carrying the authorization identity, then `\x01`-separated
/// `key=value` pairs where `auth` holds `Bearer <token>`. Returns the bearer
/// token and the authorization identity when present.
pub fn oauth_bearer(response: &[u8]) -> Option<(String, Option<String>)> {
    let text = std::str::from_utf8(response).ok()?;
    let (gs2, rest) = text.split_once(",\x01")?;
    let mut header = gs2.split(',');
    if !matches!(header.next()?, "n" | "y") {
        return None;
    }
    let username = match header.next() {
        Some("") | None => None,
        Some(value) => Some(value.strip_prefix("a=")?.to_owned()),
    };
    let mut token = None;
    for field in rest.trim_end_matches('\x01').split('\x01') {
        let (key, value) = field.split_once('=')?;
        if key.eq_ignore_ascii_case("auth") {
            let bearer = value.strip_prefix("Bearer ")?.trim();
            if bearer.is_empty() {
                return None;
            }
            token = Some(bearer.to_owned());
        }
    }
    Some((token?, username))
}
