use super::*;

pub(super) fn challenge_bytes(value: &str) -> Result<Vec<u8>, Box<Response>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid challenge"))?;
    if bytes.len() != 32 {
        return Err(refused(StatusCode::BAD_REQUEST, "invalid challenge"));
    }
    Ok(bytes)
}

pub(super) fn checked_id<T>(credential: &BrowserCredential<T>) -> Result<Vec<u8>, Box<Response>> {
    if credential.kind != "public-key" {
        return Err(refused(StatusCode::BAD_REQUEST, "invalid credential"));
    }
    let id = URL_SAFE_NO_PAD
        .decode(&credential.id)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid credential"))?;
    let raw = URL_SAFE_NO_PAD
        .decode(&credential.raw_id)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid credential"))?;
    if id.is_empty() || id.len() > 1023 || id != raw {
        return Err(refused(StatusCode::BAD_REQUEST, "invalid credential"));
    }
    Ok(id)
}

pub(super) fn checked_client_data(value: &str) -> Result<(), Box<Response>> {
    let raw = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid client data"))?;
    let data: Value = serde_json::from_slice(&raw)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid client data"))?;
    if !data.is_object()
        || data.get("topOrigin").is_some()
        || !matches!(data.get("crossOrigin"), None | Some(Value::Bool(false)))
    {
        return Err(refused(
            StatusCode::BAD_REQUEST,
            "cross-origin ceremony refused",
        ));
    }
    Ok(())
}

pub(super) fn checked_none_attestation(value: &str) -> Result<(), Box<Response>> {
    let raw = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid attestation"))?;
    let mut remaining = raw.as_slice();
    let envelope: CborValue = ciborium::de::from_reader(&mut remaining)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid attestation"))?;
    let CborValue::Map(fields) = envelope else {
        return Err(refused(StatusCode::BAD_REQUEST, "invalid attestation"));
    };
    if !remaining.is_empty() {
        return Err(refused(StatusCode::BAD_REQUEST, "invalid attestation"));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut format = None;
    let mut statement = None;
    let mut auth_data = None;
    for (key, value) in &fields {
        let CborValue::Text(key) = key else {
            return Err(refused(StatusCode::BAD_REQUEST, "invalid attestation"));
        };
        if !seen.insert(key) {
            return Err(refused(StatusCode::BAD_REQUEST, "invalid attestation"));
        }
        match key.as_str() {
            "fmt" => format = Some(value),
            "attStmt" => statement = Some(value),
            "authData" => auth_data = Some(value),
            _ => {}
        }
    }
    if !matches!(format, Some(CborValue::Text(value)) if value == "none")
        || !matches!(statement, Some(CborValue::Map(values)) if values.is_empty())
        || !matches!(auth_data, Some(CborValue::Bytes(_)))
    {
        return Err(refused(
            StatusCode::BAD_REQUEST,
            "attestation format refused",
        ));
    }
    Ok(())
}

pub(super) fn user_handle(tenant: &str, user_id: &str) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(b"oyatie.webauthn.user-handle.v1");
    for part in [tenant, user_id] {
        context.update(&(part.len() as u64).to_be_bytes());
        context.update(part.as_bytes());
    }
    context.finish().as_ref().to_vec()
}
