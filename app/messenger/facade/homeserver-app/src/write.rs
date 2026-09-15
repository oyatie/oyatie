use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use messenger_domain::AdmitCommand;
use serde_json::{Value, json};

use crate::http::{caller, map_error, matrix_error};
use crate::state::AppState;

pub async fn send(
    State(state): State<Arc<AppState>>,
    Path((room, event_type, txn)): Path<(String, String, String)>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let content: Value = match serde_json::from_str(&body) {
        Ok(content) => content,
        Err(_) => return matrix_error(StatusCode::BAD_REQUEST, "M_NOT_JSON", "not json"),
    };
    let endpoint = format!("PUT /_matrix/client/v3/rooms/{room}/send/{event_type}");
    match messenger_admission_usecase::admit(
        &*state.authority,
        &AdmitCommand {
            room,
            sender: caller.user,
            device: caller.device,
            event_type,
            state_key: None,
            content,
            txn: Some(txn),
            endpoint,
        },
    )
    .await
    {
        Ok(admission) => (
            StatusCode::OK,
            Json(json!({ "event_id": admission.event.event_id })),
        )
            .into_response(),
        Err(error) => map_error(error),
    }
}

pub async fn join(
    State(state): State<Arc<AppState>>,
    Path(room): Path<String>,
    headers: HeaderMap,
) -> Response {
    membership(&state, headers, room, "join").await
}

pub async fn leave(
    State(state): State<Arc<AppState>>,
    Path(room): Path<String>,
    headers: HeaderMap,
) -> Response {
    membership(&state, headers, room, "leave").await
}

async fn membership(
    state: &AppState,
    headers: HeaderMap,
    room: String,
    membership: &str,
) -> Response {
    let caller = match caller(state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let user = caller.user.clone();
    let endpoint = format!("POST /_matrix/client/v3/rooms/{room}/{membership}");
    match messenger_admission_usecase::admit(
        &*state.authority,
        &AdmitCommand {
            room: room.clone(),
            sender: user.clone(),
            device: caller.device,
            event_type: "m.room.member".into(),
            state_key: Some(user),
            content: json!({ "membership": membership }),
            txn: None,
            endpoint,
        },
    )
    .await
    {
        Ok(_) if membership == "join" => {
            (StatusCode::OK, Json(json!({ "room_id": room }))).into_response()
        }
        Ok(_) => (StatusCode::OK, Json(json!({}))).into_response(),
        Err(error) => map_error(error),
    }
}
