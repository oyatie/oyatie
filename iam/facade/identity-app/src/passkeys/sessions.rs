use super::*;

pub(super) async fn current_session(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
) -> Response {
    let token = match cookie_token(&headers) {
        Some(value) => value,
        None => return reply(StatusCode::UNAUTHORIZED, "session unavailable"),
    };
    match state
        .sessions
        .authorize(Some(&token), &tenant, (state.now)())
        .await
    {
        Ok(membership) => no_store(Json(SessionView::from(&membership)).into_response()),
        Err(SessionRefusal::MembershipUnavailable) => store_error(PasskeyStoreError::Unavailable),
        Err(_) => reply(StatusCode::UNAUTHORIZED, "session unavailable"),
    }
}

pub(super) async fn logout(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
) -> Response {
    if !origin_matches(&headers, &state.origin) {
        return reply(StatusCode::FORBIDDEN, "origin refused");
    }
    if let Some(hash) = cookie_token(&headers).as_deref().and_then(session_hash)
        && let Err(error) = state.store.delete_session(&tenant, &hash).await
    {
        return store_error(error);
    }
    let mut response = no_store(StatusCode::NO_CONTENT.into_response());
    response.headers_mut().insert(
        header::SET_COOKIE,
        header::HeaderValue::from_static(
            "__Host-oya_session=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0",
        ),
    );
    response
}

pub(super) fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut values = cookie
        .split(';')
        .filter_map(|part| part.trim().strip_prefix(&format!("{SESSION_COOKIE}=")));
    let token = values.next()?;
    if values.next().is_some() || session_hash(token).is_none() {
        return None;
    }
    Some(token.to_owned())
}

pub(super) fn new_session_token() -> Option<String> {
    let mut random = [0u8; 32];
    SystemRandom::new().fill(&mut random).ok()?;
    Some(format!("v1.{}", URL_SAFE_NO_PAD.encode(random)))
}

pub(super) fn session_hash(token: &str) -> Option<[u8; 32]> {
    let encoded = token.strip_prefix("v1.")?;
    let secret = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    if secret.len() != 32 || token.len() != 46 {
        return None;
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(digest(&SHA256, token.as_bytes()).as_ref());
    Some(hash)
}
