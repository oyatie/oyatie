use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use messenger_conversation_api::RoomAuthority;
use serde_json::json;

use crate::http::{caller, map_error, matrix_error, matrix_event};
use crate::state::AppState;

pub async fn sync(State(state): State<Arc<AppState>>, headers: HeaderMap, uri: Uri) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
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

pub async fn state(
    State(app): State<Arc<AppState>>,
    Path((room, event_type, state_key)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    state_at(&app, headers, room, event_type, state_key).await
}

pub async fn state_empty_key(
    State(app): State<Arc<AppState>>,
    Path((room, event_type)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    state_at(&app, headers, room, event_type, String::new()).await
}

async fn state_at(
    app: &AppState,
    headers: HeaderMap,
    room: String,
    event_type: String,
    state_key: String,
) -> Response {
    if let Err(response) = caller(app, &headers).await {
        return response;
    }
    match app.authority.state(&room, &event_type, &state_key).await {
        Ok(Some(event)) => (StatusCode::OK, Json(matrix_event(&event))).into_response(),
        Ok(None) => matrix_error(StatusCode::NOT_FOUND, "M_NOT_FOUND", "event not found"),
        Err(error) => map_error(error),
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
