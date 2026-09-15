use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use messenger_conversation_api::RoomAuthority;
use serde_json::{Value, json};

use crate::http::{caller, map_error, matrix_error};
use crate::state::AppState;

pub async fn login(State(state): State<Arc<AppState>>, body: String) -> Response {
    let parsed: Value = match serde_json::from_str(&body) {
        Ok(parsed) => parsed,
        Err(_) => {
            return matrix_error(StatusCode::BAD_REQUEST, "M_NOT_JSON", "not json");
        }
    };
    let Some((user, device)) = user_and_device(&parsed) else {
        return matrix_error(StatusCode::BAD_REQUEST, "M_INVALID_PARAM", "user required");
    };
    let token = state.issue(user.clone(), device.clone()).await;
    (
        StatusCode::OK,
        Json(json!({
            "user_id": user,
            "access_token": token,
            "device_id": device,
            "home_server": state.server_name,
        })),
    )
        .into_response()
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let parsed: Value = if body.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&body) {
            Ok(parsed) => parsed,
            Err(_) => {
                return matrix_error(StatusCode::BAD_REQUEST, "M_NOT_JSON", "not json");
            }
        }
    };
    match state
        .authority
        .create_room(&caller.user, join_rule(&parsed))
        .await
    {
        Ok(room) => (StatusCode::OK, Json(json!({ "room_id": room }))).into_response(),
        Err(error) => map_error(error),
    }
}

fn user_and_device(body: &Value) -> Option<(String, String)> {
    let user = body
        .pointer("/identifier/user")
        .or_else(|| body.get("user"))
        .or_else(|| body.get("user_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    let device = body
        .get("device_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("DEVICE");
    Some((user.to_owned(), device.to_owned()))
}

fn join_rule(body: &Value) -> &'static str {
    let preset = body.get("preset").and_then(Value::as_str).unwrap_or("");
    let visibility = body.get("visibility").and_then(Value::as_str).unwrap_or("");
    if preset == "public_chat" || visibility == "public" {
        "public"
    } else {
        "invite"
    }
}
