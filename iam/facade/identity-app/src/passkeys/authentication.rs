use super::*;

pub(super) async fn authentication_options(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    Json(input): Json<UserStart>,
) -> Response {
    if let Err(response) = active_account(&state, &tenant, &input.user_id).await {
        return *response;
    }
    let credentials = match state
        .store
        .credentials_for_user(&tenant, &input.user_id)
        .await
    {
        Ok(values) => values
            .into_iter()
            .map(serde_json::from_value::<PasskeyCredential>)
            .collect::<Result<Vec<_>, _>>(),
        Err(error) => return store_error(error),
    };
    let Ok(credentials) = credentials else {
        return store_error(PasskeyStoreError::Corrupt);
    };
    if credentials.is_empty() {
        return reply(StatusCode::UNAUTHORIZED, "account unavailable");
    }
    let (options, ceremony) = state.webauthn.start_authentication_with_creds_for_user(
        &user_handle(&tenant, &input.user_id),
        &credentials,
    );
    let challenge = options.challenge.clone();
    let state_json = match serde_json::to_value(ceremony) {
        Ok(value) => value,
        Err(_) => return store_error(PasskeyStoreError::Corrupt),
    };
    let now = (state.now)();
    if let Err(error) = state
        .store
        .save_challenge(
            &tenant,
            &match challenge_bytes(&challenge) {
                Ok(bytes) => bytes,
                Err(response) => return *response,
            },
            "authenticate",
            Some(&input.user_id),
            &state_json,
            ChallengeWindow {
                issued_at_epoch_seconds: now,
                expires_at_epoch_seconds: now + CHALLENGE_SECONDS,
            },
        )
        .await
    {
        return store_error(error);
    }
    no_store(Json(json!({"challenge": challenge, "publicKey": options})).into_response())
}

pub(super) async fn authenticate(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<Finish<AuthenticationFields>>,
) -> Response {
    if !origin_matches(&headers, &state.origin) {
        return reply(StatusCode::FORBIDDEN, "origin refused");
    }
    let challenge = match challenge_bytes(&input.challenge) {
        Ok(bytes) => bytes,
        Err(response) => return *response,
    };
    let pending = match state
        .store
        .take_challenge(&tenant, &challenge, "authenticate", (state.now)())
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => return reply(StatusCode::UNAUTHORIZED, "challenge expired or used"),
        Err(error) => return store_error(error),
    };
    let Some(user_id) = pending.user_id else {
        return store_error(PasskeyStoreError::Corrupt);
    };
    let id = match checked_id(&input.credential) {
        Ok(id) => id,
        Err(response) => return *response,
    };
    if let Err(response) = checked_client_data(&input.credential.response.client_data_json) {
        return *response;
    }
    let ceremony: AuthenticationState = match serde_json::from_value(pending.state) {
        Ok(value) => value,
        Err(_) => return store_error(PasskeyStoreError::Corrupt),
    };
    if ceremony.challenge.as_bytes() != challenge
        || ceremony.user_handle.as_deref() != Some(user_handle(&tenant, &user_id).as_slice())
    {
        return store_error(PasskeyStoreError::Corrupt);
    }
    let stored = match state.store.credential(&tenant, &user_id, &id).await {
        Ok(Some(value)) => value,
        Ok(None) => return reply(StatusCode::UNAUTHORIZED, "passkey authentication refused"),
        Err(error) => return store_error(error),
    };
    let mut credential: PasskeyCredential =
        match serde_json::from_value::<PasskeyCredential>(stored.document) {
            Ok(value) if value.id.as_bytes() == id && value.counter == stored.sign_count => value,
            _ => return store_error(PasskeyStoreError::Corrupt),
        };
    let response = AuthenticationResponse {
        id: input.credential.id,
        authenticator_data: input.credential.response.authenticator_data,
        signature: input.credential.response.signature,
        client_data_json: input.credential.response.client_data_json,
        user_handle: input.credential.response.user_handle,
    };
    let outcome = match state
        .webauthn
        .finish_authentication(&ceremony, &response, &credential)
    {
        Ok(value) => value,
        Err(_) => return reply(StatusCode::UNAUTHORIZED, "passkey authentication refused"),
    };
    credential.counter = outcome.new_counter;
    let document = match serde_json::to_value(credential) {
        Ok(value) => value,
        Err(_) => return store_error(PasskeyStoreError::Corrupt),
    };
    match state
        .store
        .compare_and_set_counter(
            &tenant,
            &user_id,
            &id,
            &document,
            stored.sign_count,
            outcome.new_counter,
        )
        .await
    {
        Ok(true) => {}
        Ok(false) => return reply(StatusCode::CONFLICT, "credential changed"),
        Err(error) => return store_error(error),
    }
    if let Err(response) = active_account(&state, &tenant, &user_id).await {
        return *response;
    }
    let token = match new_session_token() {
        Some(token) => token,
        None => return store_error(PasskeyStoreError::Unavailable),
    };
    let hash = match session_hash(&token) {
        Some(hash) => hash,
        None => return store_error(PasskeyStoreError::Corrupt),
    };
    let now = (state.now)();
    if let Err(error) = state
        .store
        .create_session(&tenant, &user_id, &hash, now + SESSION_SECONDS)
        .await
    {
        return store_error(error);
    }
    let membership = match state.sessions.authorize(Some(&token), &tenant, now).await {
        Ok(value) => value,
        Err(_) => {
            let _ = state.store.delete_session(&tenant, &hash).await;
            return reply(StatusCode::UNAUTHORIZED, "account unavailable");
        }
    };
    let cookie = format!(
        "{SESSION_COOKIE}={token}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age={SESSION_SECONDS}"
    );
    let mut response = no_store(Json(SessionView::from(&membership)).into_response());
    let Ok(cookie) = header::HeaderValue::from_str(&cookie) else {
        return store_error(PasskeyStoreError::Corrupt);
    };
    response.headers_mut().insert(header::SET_COOKIE, cookie);
    response
}
