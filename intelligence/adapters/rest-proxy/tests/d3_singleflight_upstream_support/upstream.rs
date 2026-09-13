use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use scripted_http_server::{RecordedRequest, ScriptedResponse, ScriptedServer};

/// An OAuth token endpoint with SINGLE-USE refresh tokens.
///
/// `rotate(from, access, to)` is the port of a `success_mock`: presenting `from`
/// exchanges it for `access` and rotates it to `to`. `revoke(from)` is the port of
/// deleting that mock and installing a `rejected_replay` in its place: `from` is no
/// longer exchangeable, so presenting it is answered 400 exactly as a real provider
/// answers a replayed single-use token.
#[derive(Clone, Default)]
pub struct OAuthUpstream {
    rotations: Arc<Mutex<HashMap<String, (String, String)>>>,
    forced_failure: Arc<Mutex<Option<(u16, String)>>>,
}

impl OAuthUpstream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rotate(
        &self,
        current_refresh: &str,
        access_token: &str,
        rotated_refresh: &str,
    ) -> &Self {
        self.rotations.lock().unwrap().insert(
            current_refresh.to_string(),
            (access_token.to_string(), rotated_refresh.to_string()),
        );
        self
    }

    /// Retire a refresh token: any later presentation of it is a replay and is refused.
    pub fn revoke(&self, current_refresh: &str) -> &Self {
        self.rotations.lock().unwrap().remove(current_refresh);
        self
    }

    /// Force every exchange to fail, whatever token is presented.
    pub fn fail_with(&self, status: u16, body: &str) -> &Self {
        *self.forced_failure.lock().unwrap() = Some((status, body.to_string()));
        self
    }

    pub fn clear_failure(&self) -> &Self {
        *self.forced_failure.lock().unwrap() = None;
        self
    }

    pub fn serve(&self) -> ScriptedServer {
        let upstream = self.clone();
        ScriptedServer::start_with(move |request| {
            if request.path() != "/v1/oauth/token" || request.method != "POST" {
                return ScriptedResponse::status(404).text("not the token endpoint");
            }
            if let Some((status, body)) = upstream.forced_failure.lock().unwrap().clone() {
                return ScriptedResponse::status(status).text(body);
            }
            let Some(presented) = presented_refresh_token(request) else {
                return ScriptedResponse::status(400).text("no refresh_token in request body");
            };
            match upstream.rotations.lock().unwrap().get(&presented) {
                Some((access_token, rotated_refresh)) => ScriptedResponse::ok()
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{"access_token":"{access_token}","refresh_token":"{rotated_refresh}","expires_in":3600}}"#
                    )),
                // The port of every `rejected_replay` mock in the original.
                None => ScriptedResponse::status(400)
                    .text("single-use refresh token already consumed"),
            }
        })
    }
}

/// The `refresh_token` value an exchange request presented.
pub fn presented_refresh_token(request: &RecordedRequest) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(&request.body)
        .ok()?
        .get("refresh_token")?
        .as_str()
        .map(str::to_owned)
}

/// How many exchanges presented `refresh`, counting only requests recorded at or after
/// `since` (the index captured where the original deleted a mock and installed a new one).
pub fn exchanges_since(server: &ScriptedServer, refresh: &str, since: usize) -> usize {
    server
        .requests()
        .iter()
        .skip(since)
        .filter(|request| presented_refresh_token(request).as_deref() == Some(refresh))
        .count()
}

pub fn exchanges_for(server: &ScriptedServer, refresh: &str) -> usize {
    exchanges_since(server, refresh, 0)
}

/// The port of `rejected_replay.assert_hits(0)`, and strictly stronger than it: the
/// original only proved httpmock never SELECTED the replay mock, whereas this reads the
/// bodies that actually went on the wire.
pub fn assert_no_replay_since(server: &ScriptedServer, refresh: &str, since: usize) {
    let replays = exchanges_since(server, refresh, since);
    assert_eq!(
        replays,
        0,
        "the single-use refresh token '{refresh}' was replayed {replays} time(s) after \
         being retired; bodies seen: {:?}",
        server
            .requests()
            .iter()
            .skip(since)
            .map(presented_refresh_token)
            .collect::<Vec<_>>()
    );
}
