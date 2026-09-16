use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use messenger_conversation_api::RoomAuthority;
use messenger_domain::AdmitCommand;
use serde_json::{Value, json};

use crate::{AppState, caller, map_error, matrix_error, matrix_event, parse_json};

pub async fn login(State(state): State<Arc<AppState>>, body: String) -> Response {
    let parsed = match parse_json(&body) {
        Ok(parsed) => parsed,
        Err(response) => return *response,
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
        Err(response) => return *response,
    };
    let parsed = if body.trim().is_empty() {
        json!({})
    } else {
        match parse_json(&body) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
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

pub async fn send(
    State(state): State<Arc<AppState>>,
    Path((room, event_type, txn)): Path<(String, String, String)>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let content = match parse_json(&body) {
        Ok(content) => content,
        Err(response) => return *response,
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
        Err(response) => return *response,
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

pub async fn sync(State(state): State<Arc<AppState>>, headers: HeaderMap, uri: Uri) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    match state
        .authority
        .sync(&caller.user, &caller.device, since(&uri).as_deref())
        .await
    {
        Ok(sync) => {
            let mut join = serde_json::Map::new();
            for delta in sync.rooms {
                join.insert(
                    delta.room,
                    json!({
                        "timeline": {
                            "events": delta.events.iter().map(matrix_event).collect::<Vec<_>>(),
                            "limited": false,
                        },
                        "state": { "events": [] },
                    }),
                );
            }
            (
                StatusCode::OK,
                Json(json!({
                    "next_batch": sync.next_batch,
                    "rooms": { "join": join },
                })),
            )
                .into_response()
        }
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

fn since(uri: &Uri) -> Option<String> {
    uri.query().and_then(|query| {
        query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            (key == "since").then(|| value.to_owned())
        })
    })
}
