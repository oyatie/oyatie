use std::collections::BTreeMap;
use std::sync::Arc;

use messenger_authority_memory::MemoryAuthority;
use tokio::sync::Mutex;

use crate::http::Caller;

pub struct AppState {
    pub(crate) authority: Arc<MemoryAuthority>,
    pub(crate) server_name: String,
    sessions: Mutex<BTreeMap<String, Caller>>,
    next_token: Mutex<u64>,
}

pub fn compose(server_name: impl Into<String>) -> Arc<AppState> {
    let server_name = server_name.into();
    Arc::new(AppState {
        authority: MemoryAuthority::new(server_name.clone()),
        server_name,
        sessions: Mutex::new(BTreeMap::new()),
        next_token: Mutex::new(1),
    })
}

impl AppState {
    pub(crate) async fn issue(&self, user: String, device: String) -> String {
        let mut seq = self.next_token.lock().await;
        let token = format!("s{seq}");
        *seq = seq.saturating_add(1);
        drop(seq);
        self.sessions
            .lock()
            .await
            .insert(token.clone(), Caller { user, device });
        token
    }

    pub(crate) async fn caller(&self, token: &str) -> Option<Caller> {
        self.sessions.lock().await.get(token).cloned()
    }
}
