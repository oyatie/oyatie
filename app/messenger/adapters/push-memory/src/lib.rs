#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_push_api::{Push, PushCounts, PushError, PushNotice, PushPriority};
use std::{collections::BTreeSet, sync::Arc, sync::Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivered {
    pub event_id: Option<String>,
    pub room_id: Option<String>,
    pub counts: PushCounts,
    pub prio: PushPriority,
    pub app_id: String,
}

pub struct MemoryPush {
    apps: BTreeSet<String>,
    delivered: Mutex<Vec<Delivered>>,
}

impl MemoryPush {
    pub fn new(apps: impl IntoIterator<Item = impl Into<String>>) -> Arc<Self> {
        Arc::new(Self {
            apps: apps.into_iter().map(Into::into).collect(),
            delivered: Mutex::new(Vec::new()),
        })
    }

    pub fn delivered(&self) -> Vec<Delivered> {
        self.delivered
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }
}

impl Push for MemoryPush {
    async fn notify(&self, notice: &PushNotice) -> Result<Vec<String>, PushError> {
        notice.validate()?;
        let mut delivered = self
            .delivered
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut rejected = BTreeSet::new();
        for device in &notice.devices {
            if !self.apps.contains(&device.app_id) {
                rejected.insert(device.pushkey.clone());
                continue;
            }
            delivered.push(Delivered {
                event_id: notice.event_id.clone(),
                room_id: notice.room_id.clone(),
                counts: notice.counts.clone(),
                prio: notice.prio,
                app_id: device.app_id.clone(),
            });
        }
        Ok(rejected.into_iter().collect())
    }
}
