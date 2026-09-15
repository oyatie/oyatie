#[path = "submission/support.rs"]
mod support;

use std::sync::atomic::Ordering;

use foundry_submission_draft::{SubmitError, SubmitResponse};
use mail_api::Events;
use mail_foundry::publish_authorized;
use mail_kernel::Error;
use support::{Destination, Outbox, credentials};

#[tokio::test]
async fn canonical_records_preserve_revisions_tenants_and_source_provenance() {
    let outbox = Outbox::new();
    outbox.append("a");
    outbox.append("a");
    outbox.append("b");
    let pending = outbox.store.pending("foundry", 10).unwrap();
    let destination = Destination::default();
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &credentials,
            "mail-main",
            "foundry",
            10
        )
        .await
        .unwrap(),
        3
    );
    assert!(outbox.store.pending("foundry", 10).unwrap().is_empty());
    let entries = destination.entries.lock().unwrap();
    assert_eq!(entries.len(), 3);
    let objects = entries
        .values()
        .map(|(request, _)| &request.object_ref)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        objects.len(),
        3,
        "create-only records need distinct objects per revision"
    );
    for ((tenant, _), (request, _)) in &*entries {
        let note: serde_json::Value = serde_json::from_str(&request.properties["note"]).unwrap();
        let original = pending
            .iter()
            .find(|event| {
                event.tenant == *tenant && event.revision == note["revision"].as_u64().unwrap()
            })
            .unwrap();
        assert_eq!(note["source"], "mail-main");
        assert_eq!(note["accountId"], original.account);
        assert_eq!(note["observedAtMs"], original.observed_at_ms);
        assert_eq!(
            request.occurred_at_epoch_seconds,
            original.observed_at_ms / 1000
        );
        assert!(!request.properties["note"].contains("secret"));
        assert!(!request.properties["note"].contains("confidential"));
    }
}

#[tokio::test]
async fn lost_ack_retries_identical_payload_without_another_destination_record() {
    let outbox = Outbox::new();
    outbox.append("a");
    outbox.reject_ack.store(true, Ordering::SeqCst);
    let destination = Destination::default();
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &credentials,
            "mail-main",
            "foundry",
            10
        )
        .await,
        Err(Error::Unavailable)
    );
    assert_eq!(destination.entries.lock().unwrap().len(), 1);
    assert_eq!(outbox.store.pending("foundry", 10).unwrap().len(), 1);
    outbox.reject_ack.store(false, Ordering::SeqCst);
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &credentials,
            "mail-main",
            "foundry",
            10
        )
        .await
        .unwrap(),
        1
    );
    assert_eq!(destination.entries.lock().unwrap().len(), 1);
    let calls = destination.calls.lock().unwrap();
    assert_eq!(calls[0], calls[1]);
}

#[tokio::test]
async fn policy_and_tenant_refusals_never_acknowledge_source_events() {
    let outbox = Outbox::new();
    outbox.append("a");
    let destination = Destination::default();
    for error in [
        SubmitError::Authorization,
        SubmitError::Unavailable,
        SubmitError::Conflict,
        SubmitError::Credential,
        SubmitError::UnservedTenant,
    ] {
        *destination.reject.lock().unwrap() = Some(error);
        assert!(
            publish_authorized(
                outbox.clone(),
                &destination,
                &credentials,
                "mail-main",
                "foundry",
                10
            )
            .await
            .is_err()
        );
        assert_eq!(outbox.store.pending("foundry", 10).unwrap().len(), 1);
    }
    *destination.reject.lock().unwrap() = None;
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &|_| Ok("secret-b".into()),
            "mail-main",
            "foundry",
            10
        )
        .await,
        Err(Error::Forbidden)
    );
    assert!(destination.entries.lock().unwrap().is_empty());
    assert_eq!(outbox.store.pending("foundry", 10).unwrap().len(), 1);
}

#[tokio::test]
async fn only_a_valid_applied_receipt_advances_the_subscription() {
    let outbox = Outbox::new();
    outbox.append("a");
    let destination = Destination::default();
    for (outcome, ordinal, poison) in [
        ("poisoned", 1, Some("invalid".into())),
        ("applied", 0, None),
        ("applied", 1, Some("invalid".into())),
        ("unknown", 1, None),
    ] {
        *destination.response.lock().unwrap() = Some(SubmitResponse {
            outcome,
            ordinal,
            poison_reason: poison,
            deduplicated: false,
        });
        assert_eq!(
            publish_authorized(
                outbox.clone(),
                &destination,
                &credentials,
                "mail-main",
                "foundry",
                10
            )
            .await,
            Err(Error::Unavailable)
        );
        assert_eq!(outbox.store.pending("foundry", 10).unwrap().len(), 1);
    }
    *destination.response.lock().unwrap() = None;
    assert_eq!(
        publish_authorized(
            outbox.clone(),
            &destination,
            &credentials,
            "mail-main",
            "foundry",
            10
        )
        .await
        .unwrap(),
        1
    );
}
