use super::*;

pub(super) async fn enrollment_options(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<UserStart>,
) -> Response {
    if let Err(response) = (state.admin)(&headers, &tenant) {
        return *response;
    }
    if let Err(response) = active_account(&state, &tenant, &input.user_id).await {
        return *response;
    }
    let existing = match state
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
    let Ok(existing) = existing else {
        return store_error(PasskeyStoreError::Corrupt);
    };
    let (options, ceremony) = state.webauthn.start_registration(
        &user_handle(&tenant, &input.user_id),
        &input.user_id,
        &input.user_id,
        &existing
            .iter()
            .map(|credential| credential.id.clone())
            .collect::<Vec<_>>(),
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
            "enroll",
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

pub(super) async fn enroll(
    State(state): State<Arc<PasskeyState>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<Finish<RegistrationFields>>,
) -> Response {
    // An administrator issues the tenant/user-bound, one-use challenge at
    // enrollment-options. The browser completes that invitation without ever
    // receiving the administrator's workload bearer.
    if !origin_matches(&headers, &state.origin) {
        return reply(StatusCode::FORBIDDEN, "origin refused");
    }
    let challenge = match challenge_bytes(&input.challenge) {
        Ok(bytes) => bytes,
        Err(response) => return *response,
    };
    let pending = match state
        .store
        .take_challenge(&tenant, &challenge, "enroll", (state.now)())
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => return reply(StatusCode::UNAUTHORIZED, "challenge expired or used"),
        Err(error) => return store_error(error),
    };
    let Some(user_id) = pending.user_id else {
        return store_error(PasskeyStoreError::Corrupt);
    };
    if let Err(response) = active_account(&state, &tenant, &user_id).await {
        return *response;
    }
    let id = match checked_id(&input.credential) {
        Ok(id) => id,
        Err(response) => return *response,
    };
    if let Err(response) = checked_client_data(&input.credential.response.client_data_json) {
        return *response;
    }
    if let Err(response) = checked_none_attestation(&input.credential.response.attestation_object) {
        return *response;
    }
    let ceremony: RegistrationState = match serde_json::from_value(pending.state) {
        Ok(value) => value,
        Err(_) => return store_error(PasskeyStoreError::Corrupt),
    };
    if ceremony.challenge.as_bytes() != challenge
        || ceremony.user_id != user_handle(&tenant, &user_id)
    {
        return store_error(PasskeyStoreError::Corrupt);
    }
    let response = RegistrationResponse {
        id: input.credential.id,
        transports: input.credential.response.transports,
        attestation_object: input.credential.response.attestation_object,
        client_data_json: input.credential.response.client_data_json,
    };
    let credential = match state.webauthn.finish_registration(&ceremony, &response) {
        Ok(value) if value.id.as_bytes() == id => value,
        _ => return reply(StatusCode::UNAUTHORIZED, "passkey registration refused"),
    };
    let document = match serde_json::to_value(&credential) {
        Ok(value) => value,
        Err(_) => return store_error(PasskeyStoreError::Corrupt),
    };
    match state
        .store
        .insert_credential(&tenant, &user_id, &id, &document, credential.counter)
        .await
    {
        Ok(()) => no_store(StatusCode::CREATED.into_response()),
        Err(PasskeyStoreError::Conflict) => {
            reply(StatusCode::CONFLICT, "credential already enrolled")
        }
        Err(error) => store_error(error),
    }
}
