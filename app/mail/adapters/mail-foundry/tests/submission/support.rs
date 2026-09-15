use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use foundry_submission_draft::{
    ActionSubmitter, Submission, SubmitError, SubmitRequest, SubmitResponse,
};
use mail_api::{Event, Events};
use mail_kernel::{Account, Error};
use mail_sqlite::SqliteStore;

pub struct Outbox {
    pub store: SqliteStore,
    pub reject_ack: AtomicBool,
    worker: std::thread::ThreadId,
}

impl Outbox {
    pub fn new() -> Arc<Self> {
        let store = SqliteStore::open(":memory:").unwrap();
        for (account, tenant) in [("a", "ten_acme"), ("b", "ten_other")] {
            store
                .provision(
                    Account::new(account, tenant, account, &format!("{account}@example.org"))
                        .unwrap(),
                    &account.repeat(32),
                )
                .unwrap();
        }
        Arc::new(Self {
            store,
            reject_ack: AtomicBool::new(false),
            worker: std::thread::current().id(),
        })
    }

    pub fn append(&self, account: &str) {
        self.store
            .deliver(
                &[format!("{account}@example.org")],
                b"Subject: confidential\r\n\r\nsecret body",
            )
            .unwrap();
    }
}

impl Events for Outbox {
    fn pending(&self, consumer: &str, limit: usize) -> Result<Vec<Event>, Error> {
        assert_ne!(
            self.worker,
            std::thread::current().id(),
            "SQLite pending must leave the async worker"
        );
        self.store.pending(consumer, limit)
    }
    fn acknowledge(&self, consumer: &str, sequence: u64) -> Result<(), Error> {
        assert_ne!(
            self.worker,
            std::thread::current().id(),
            "SQLite acknowledge must leave the async worker"
        );
        if self.reject_ack.load(Ordering::SeqCst) {
            return Err(Error::Unavailable);
        }
        self.store.acknowledge(consumer, sequence)
    }
}

type Entry = (String, SubmitRequest);

#[derive(Default)]
pub struct Destination {
    pub calls: Mutex<Vec<Entry>>,
    pub entries: Mutex<BTreeMap<(String, String), (SubmitRequest, u64)>>,
    pub reject: Mutex<Option<SubmitError>>,
    pub response: Mutex<Option<SubmitResponse>>,
}

impl ActionSubmitter for Destination {
    fn submit<'a>(&'a self, _: &'a str, _: SubmitRequest) -> Submission<'a> {
        panic!("mail must constrain the destination to the source tenant")
    }
    fn submit_in_tenant<'a>(
        &'a self,
        credential: &'a str,
        tenant: &'a str,
        request: SubmitRequest,
    ) -> Submission<'a> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push((tenant.into(), request.clone()));
            let verified = match credential {
                "secret-a" => "ten_acme",
                "secret-b" => "ten_other",
                _ => return Err(SubmitError::Credential),
            };
            if verified != tenant {
                return Err(SubmitError::TenantMismatch);
            }
            if let Some(error) = self.reject.lock().unwrap().clone() {
                return Err(error);
            }
            if let Some(response) = self.response.lock().unwrap().clone() {
                return Ok(response);
            }
            assert_eq!(request.action_type, "aty_record_write");
            assert!(request.object_ref.starts_with("ent_"));
            assert_eq!(
                request
                    .properties
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["name", "note"]
            );
            let mut entries = self.entries.lock().unwrap();
            let key = (tenant.into(), request.idempotency_key.clone());
            let (ordinal, deduplicated) = match entries.get(&key) {
                Some((old, ordinal)) if old == &request => (*ordinal, true),
                Some(_) => return Err(SubmitError::Conflict),
                None => {
                    let ordinal =
                        entries.keys().filter(|(owner, _)| owner == tenant).count() as u64 + 1;
                    entries.insert(key, (request, ordinal));
                    (ordinal, false)
                }
            };
            Ok(SubmitResponse {
                outcome: "applied",
                ordinal,
                deduplicated,
                poison_reason: None,
            })
        })
    }
}

pub fn credentials(tenant: &str) -> Result<String, Error> {
    match tenant {
        "ten_acme" => Ok("secret-a".into()),
        "ten_other" => Ok("secret-b".into()),
        _ => Err(Error::Forbidden),
    }
}
