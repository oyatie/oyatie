#[path = "submission/support.rs"]
mod support;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use mail_api::{Event, Events};
use mail_foundry::publish_authorized;
use mail_kernel::Error;
use support::{Destination, Outbox, credentials};

#[tokio::test]
async fn a_missing_middle_tenant_credential_stops_before_later_events_and_replays_safely() {
    let outbox = Outbox::new();
    for account in ["a", "b", "a"] {
        outbox.append(account);
    }
    let destination = Destination::default();
    let only_first = |tenant: &str| {
        if tenant == "ten_acme" {
            credentials(tenant)
        } else {
            Err(Error::Forbidden)
        }
    };
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &only_first,
            "mail",
            "foundry",
            10
        )
        .await,
        Err(Error::Forbidden)
    );
    assert_eq!(destination.entries.lock().unwrap().len(), 1);
    let pending = outbox.store.pending("foundry", 10).unwrap();
    assert_eq!(
        pending
            .iter()
            .map(|e| e.account.as_str())
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &credentials,
            "mail",
            "foundry",
            10
        )
        .await
        .unwrap(),
        2
    );
    assert_eq!(destination.entries.lock().unwrap().len(), 3);
    assert!(outbox.store.pending("foundry", 10).unwrap().is_empty());
}

struct FixedEvents {
    entries: Vec<Event>,
    acknowledgements: AtomicUsize,
    requested: AtomicUsize,
    panic_pending: bool,
    panic_ack: bool,
}
impl FixedEvents {
    fn new(entries: Vec<Event>) -> Arc<Self> {
        Arc::new(Self {
            entries,
            acknowledgements: AtomicUsize::new(0),
            requested: AtomicUsize::new(usize::MAX),
            panic_pending: false,
            panic_ack: false,
        })
    }
}
impl Events for FixedEvents {
    fn pending(&self, _: &str, limit: usize) -> Result<Vec<Event>, Error> {
        assert!(!self.panic_pending, "injected pending worker panic");
        self.requested.store(limit, Ordering::SeqCst);
        Ok(self.entries.clone())
    }
    fn acknowledge(&self, _: &str, _: u64) -> Result<(), Error> {
        assert!(!self.panic_ack, "injected acknowledgement worker panic");
        self.acknowledgements.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
fn event() -> Event {
    Event {
        sequence: 1,
        tenant: "ten_acme".into(),
        account: "a".into(),
        revision: 1,
        observed_at_ms: 1_700_000_000_123,
    }
}

#[tokio::test]
async fn an_outbox_exceeding_the_requested_limit_cannot_trigger_destination_work() {
    for (requested, returned, expected) in [(0, 1, 0), (usize::MAX, 1001, 1000)] {
        let events = FixedEvents::new(vec![event(); returned]);
        let destination = Destination::default();
        assert_eq!(
            publish_authorized(
                events.clone(),
                &destination,
                &credentials,
                "mail",
                "foundry",
                requested
            )
            .await,
            Err(Error::Unavailable)
        );
        assert_eq!(events.requested.load(Ordering::SeqCst), expected);
        assert_eq!(events.acknowledgements.load(Ordering::SeqCst), 0);
        assert!(destination.calls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn malformed_event_provenance_is_refused_before_credentials_or_destination_access() {
    for field in ["sequence", "revision", "time", "account", "tenant"] {
        let mut event = event();
        match field {
            "sequence" => event.sequence = 0,
            "revision" => event.revision = 0,
            "time" => event.observed_at_ms = 0,
            "account" => event.account = "invalid/account".into(),
            _ => event.tenant = "invalid/tenant".into(),
        }
        let events = FixedEvents::new(vec![event]);
        let destination = Destination::default();
        let resolver = |_: &str| -> Result<String, Error> {
            panic!("invalid provenance must precede credential access")
        };
        assert_eq!(
            publish_authorized(
                events.clone(),
                &destination,
                &resolver,
                "mail",
                "foundry",
                1
            )
            .await,
            Err(Error::Unavailable),
            "{field}"
        );
        assert_eq!(events.acknowledgements.load(Ordering::SeqCst), 0);
        assert!(destination.calls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn blocking_worker_panics_fail_without_fabricating_an_acknowledgement() {
    for pending in [true, false] {
        let mut events = FixedEvents::new(vec![event()]);
        let mutable = Arc::get_mut(&mut events).unwrap();
        mutable.panic_pending = pending;
        mutable.panic_ack = !pending;
        let destination = Destination::default();
        assert_eq!(
            publish_authorized(
                events.clone(),
                &destination,
                &credentials,
                "mail",
                "foundry",
                1
            )
            .await,
            Err(Error::Unavailable)
        );
        assert_eq!(events.acknowledgements.load(Ordering::SeqCst), 0);
        assert_eq!(
            destination.entries.lock().unwrap().len(),
            usize::from(!pending)
        );
    }
}

#[tokio::test]
async fn subscriptions_share_source_identity_but_another_source_gets_distinct_records() {
    let outbox = Outbox::new();
    outbox.append("a");
    let destination = Destination::default();
    for (source, consumer, count) in [
        ("mail-one", "foundry", 1),
        ("mail-one", "console", 1),
        ("mail-two", "other-foundry", 2),
    ] {
        assert_eq!(
            publish_authorized(
                outbox.clone(),
                &destination,
                &credentials,
                source,
                consumer,
                1
            )
            .await
            .unwrap(),
            1
        );
        assert_eq!(destination.entries.lock().unwrap().len(), count);
    }
    let entries = destination.entries.lock().unwrap();
    let objects = entries
        .values()
        .map(|(request, _)| &request.object_ref)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(objects.len(), 2);
}
