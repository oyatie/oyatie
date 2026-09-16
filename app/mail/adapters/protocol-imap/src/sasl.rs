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
