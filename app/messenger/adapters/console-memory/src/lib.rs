#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_collaboration_api::ConsoleCollaboration;
use messenger_domain::{ConsoleCommand, ConsoleObjectRef, Error};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ObjectKey {
    company: String,
    object_type: String,
    instance: String,
}

impl ObjectKey {
    fn from_object(object: &ConsoleObjectRef) -> Self {
        Self {
            company: object.company.clone(),
            object_type: object.object_type.clone(),
            instance: object.instance.clone(),
        }
    }
}

struct StoredObject {
    revision: u32,
    body: Value,
}

#[derive(Default)]
struct Inner {
    credentials: BTreeSet<String>,
    objects: BTreeMap<ObjectKey, StoredObject>,
    receipts: BTreeMap<String, ConsoleObjectRef>,
}

pub struct MemoryConsole {
    company: String,
    inner: Mutex<Inner>,
}

impl MemoryConsole {
    pub fn new(company: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            company: company.into(),
            inner: Mutex::new(Inner::default()),
        })
    }

    pub async fn grant(&self, credential: &str) -> Result<(), Error> {
        if !credential_shape(credential) {
            return Err(Error::Denied);
        }
        self.inner
            .lock()
            .await
            .credentials
            .insert(credential.to_owned());
        Ok(())
    }

    pub async fn put(&self, object: ConsoleObjectRef, body: Value) -> Result<(), Error> {
        object.validate()?;
        if object.company != self.company {
            return Err(Error::Denied);
        }
        self.inner.lock().await.objects.insert(
            ObjectKey::from_object(&object),
            StoredObject {
                revision: object.revision,
                body,
            },
        );
        Ok(())
    }
}

fn credential_shape(credential: &str) -> bool {
    !credential.is_empty() && credential.len() <= 8192 && !credential.chars().any(char::is_control)
}

fn admit(
    inner: &Inner,
    adapter_company: &str,
    credential: &str,
    company: &str,
) -> Result<(), Error> {
    if !credential_shape(credential)
        || company != adapter_company
        || !inner.credentials.contains(credential)
    {
        return Err(Error::Denied);
    }
    Ok(())
}

fn changed() -> Error {
    Error::Invalid("Console object changed; refresh its reference".into())
}

impl ConsoleCollaboration for MemoryConsole {
    async fn read(&self, credential: &str, object: &ConsoleObjectRef) -> Result<Value, Error> {
        object.validate()?;
        let inner = self.inner.lock().await;
        admit(&inner, &self.company, credential, &object.company)?;
        let stored = inner
            .objects
            .get(&ObjectKey::from_object(object))
            .ok_or(Error::Denied)?;
        if stored.revision != object.revision {
            return Err(changed());
        }
        Ok(stored.body.clone())
    }

    async fn execute(
        &self,
        credential: &str,
        command: &ConsoleCommand,
    ) -> Result<(ConsoleObjectRef, bool), Error> {
        command.validate()?;
        let mut inner = self.inner.lock().await;
        admit(&inner, &self.company, credential, &command.object.company)?;
        if let Some(pin) = inner.receipts.get(&command.command_id) {
            return Ok((pin.clone(), true));
        }
        let key = ObjectKey::from_object(&command.object);
        let stored = inner.objects.get_mut(&key).ok_or(Error::Denied)?;
        if stored.revision != command.object.revision {
            return Err(changed());
        }
        stored.revision = stored.revision.saturating_add(1);
        let mut pin = command.object.clone();
        pin.revision = stored.revision;
        inner
            .receipts
            .insert(command.command_id.clone(), pin.clone());
        Ok((pin, false))
    }
}
