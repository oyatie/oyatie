#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_push_api::{Push, PushCounts, PushDevice, PushError, PushNotice, PushPriority};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivered {
    pub event_id: Option<String>,
    pub room_id: Option<String>,
    pub counts: PushCounts,
    pub prio: PushPriority,
    pub app_id: String,
}

#[derive(Default)]
struct Inner {
    rejected: BTreeSet<String>,
    unregistered_at: BTreeMap<String, u64>,
    outage: Option<u64>,
    delivered: Vec<Delivered>,
}

pub struct MemoryPush {
    apps: BTreeSet<String>,
    inner: Mutex<Inner>,
}

impl MemoryPush {
    pub fn new(apps: impl IntoIterator<Item = impl Into<String>>) -> Arc<Self> {
        Arc::new(Self {
            apps: apps.into_iter().map(Into::into).collect(),
            inner: Mutex::new(Inner::default()),
        })
    }

    pub async fn reject_key(&self, pushkey: impl Into<String>) {
        self.inner.lock().await.rejected.insert(pushkey.into());
    }

    pub async fn unregister_key(&self, pushkey: impl Into<String>, at: u64) {
        self.inner
            .lock()
            .await
            .unregistered_at
            .insert(pushkey.into(), at);
    }

    pub async fn set_outage(&self, retry_after: u64) {
        self.inner.lock().await.outage = Some(retry_after.max(1));
    }

    pub async fn delivered(&self) -> Vec<Delivered> {
        self.inner.lock().await.delivered.clone()
    }

    fn deliver(
        apps: &BTreeSet<String>,
        inner: &mut Inner,
        notice: &PushNotice,
        device: &PushDevice,
    ) -> Result<bool, PushError> {
        if !apps.contains(&device.app_id) || inner.rejected.contains(&device.pushkey) {
            return Ok(false);
        }
        if let Some(&when) = inner.unregistered_at.get(&device.pushkey) {
            if device.pushkey_ts.is_some_and(|updated| updated > when) {
                return Err(PushError::Unavailable { retry_after: 1 });
            }
            return Ok(false);
        }
        inner.delivered.push(Delivered {
            event_id: notice.event_id.clone(),
            room_id: notice.room_id.clone(),
            counts: notice.counts.clone(),
            prio: notice.prio,
            app_id: device.app_id.clone(),
        });
        Ok(true)
    }
}

impl Push for MemoryPush {
    async fn notify(&self, notice: &PushNotice) -> Result<Vec<String>, PushError> {
        notice.validate()?;
        let mut inner = self.inner.lock().await;
        if let Some(retry_after) = inner.outage {
            return Err(PushError::Unavailable { retry_after });
        }
        let mut rejected = BTreeSet::new();
        let mut retry = 0;
        for device in &notice.devices {
            match Self::deliver(&self.apps, &mut inner, notice, device) {
                Ok(true) => {}
                Ok(false) => {
                    rejected.insert(device.pushkey.clone());
                }
                Err(PushError::Unavailable { retry_after }) => retry = retry.max(retry_after),
                Err(PushError::Invalid) => return Err(PushError::Invalid),
            }
        }
        if retry > 0 {
            return Err(PushError::Unavailable { retry_after: retry });
        }
        Ok(rejected.into_iter().collect())
    }
}
