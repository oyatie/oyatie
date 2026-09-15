use messenger_admission_usecase::{
    AuthorityEvent, AuthorityRecord, AuthoritySync, Error, RoomAuthority,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Memory {
    server: String,
    inner: Mutex<AuthorityRecord>,
}

impl Memory {
    pub fn new(server: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            server: server.into(),
            inner: Mutex::new(AuthorityRecord::default()),
        })
    }
}

pub struct Contending {
    inner: Arc<Memory>,
    fail_commits: Mutex<u32>,
}

impl Contending {
    pub fn new(inner: Arc<Memory>, fail_commits: u32) -> Arc<Self> {
        Arc::new(Self {
            inner,
            fail_commits: Mutex::new(fail_commits),
        })
    }
}

impl RoomAuthority for Memory {
    async fn create_room(&self, creator: &str, join_rule: &str) -> Result<String, Error> {
        let mut inner = self.inner.lock().await;
        inner.seq = inner.seq.saturating_add(1);
        let room_id = format!("!{}:{}", inner.seq, self.server);
        inner.create_room(room_id, creator, join_rule, 1)
    }

    async fn snapshot(&self) -> Result<AuthorityRecord, Error> {
        Ok(self.inner.lock().await.clone())
    }

    async fn commit(
        &self,
        expected_generation: u64,
        mut record: AuthorityRecord,
    ) -> Result<(), Error> {
        let mut guard = self.inner.lock().await;
        if guard.generation != expected_generation {
            return Err(Error::Unavailable("room contention".into()));
        }
        record.generation = expected_generation.saturating_add(1);
        *guard = record;
        Ok(())
    }

    async fn sync(
        &self,
        user: &str,
        _device: &str,
        since: Option<&str>,
    ) -> Result<AuthoritySync, Error> {
        self.inner.lock().await.sync(user, since)
    }

    async fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> Result<Option<AuthorityEvent>, Error> {
        Ok(self.inner.lock().await.state(room, event_type, state_key))
    }
}

impl RoomAuthority for Contending {
    async fn create_room(&self, creator: &str, join_rule: &str) -> Result<String, Error> {
        self.inner.create_room(creator, join_rule).await
    }

    async fn snapshot(&self) -> Result<AuthorityRecord, Error> {
        self.inner.snapshot().await
    }

    async fn commit(&self, expected_generation: u64, record: AuthorityRecord) -> Result<(), Error> {
        let mut remaining = self.fail_commits.lock().await;
        if *remaining > 0 {
            *remaining -= 1;
            return Err(Error::Unavailable("injected contention".into()));
        }
        drop(remaining);
        self.inner.commit(expected_generation, record).await
    }

    async fn sync(
        &self,
        user: &str,
        device: &str,
        since: Option<&str>,
    ) -> Result<AuthoritySync, Error> {
        self.inner.sync(user, device, since).await
    }

    async fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> Result<Option<AuthorityEvent>, Error> {
        self.inner.state(room, event_type, state_key).await
    }
}
