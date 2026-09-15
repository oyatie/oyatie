#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod claim;
mod ops;
mod state;

use messenger_domain::{Delivery, Error, Installation, InstallationSpec};
use messenger_installation_api::{Dispatch, InstallationStore};
use serde_json::Value;
use state::Inner;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub struct MemoryInstallations {
    inner: Mutex<Inner>,
}

impl MemoryInstallations {
    pub fn new(tenant: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                tenant: tenant.into(),
                ..Inner::default()
            }),
        })
    }

    pub async fn elapse(&self, duration: Duration) {
        self.inner.lock().await.elapse(duration);
    }
}

impl InstallationStore for MemoryInstallations {
    async fn get(&self, room: &str, id: &str) -> Result<Option<Installation>, Error> {
        ops::get(&*self.inner.lock().await, room, id)
    }

    async fn list(&self, room: &str, after: Option<&str>) -> Result<Vec<Installation>, Error> {
        ops::list(&*self.inner.lock().await, room, after)
    }

    async fn install(
        &self,
        spec: &InstallationSpec,
        expected: u64,
        command: &str,
    ) -> Result<Installation, Error> {
        ops::install(&mut *self.inner.lock().await, spec, expected, command)
    }

    async fn revoke(
        &self,
        room: &str,
        id: &str,
        expected: u64,
        command: &str,
    ) -> Result<Installation, Error> {
        ops::revoke(&mut *self.inner.lock().await, room, id, expected, command)
    }

    async fn enqueue(&self, room: &str, delivery: &Delivery) -> Result<(), Error> {
        ops::enqueue(&mut *self.inner.lock().await, room, delivery)
    }

    async fn claim(&self, expected: &Installation) -> Result<Option<Dispatch>, Error> {
        claim::claim(&mut *self.inner.lock().await, expected)
    }

    async fn complete(&self, dispatch: &Dispatch, receipt: &Value) -> Result<(), Error> {
        claim::complete(&mut *self.inner.lock().await, dispatch, receipt)
    }

    async fn retry(&self, dispatch: &Dispatch) -> Result<(), Error> {
        claim::retry(&mut *self.inner.lock().await, dispatch)
    }
}
