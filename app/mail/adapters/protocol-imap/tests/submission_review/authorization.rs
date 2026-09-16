use super::support::*;
use mail_api::{Action, DeliveryQueue, Policy, Store, SubmissionQueue, SubmissionStore};
use mail_kernel::Error;
use mail_service::OwnerPolicy;
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::test]
async fn rejected_identity_email_and_sender_leave_history_email_and_all_queues_unchanged() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let before = f.db.account("a").unwrap();
    for object in [
        json!({"identityId":"b","emailId":"e1"}),
        json!({"identityId":"missing","emailId":"e1"}),
        json!({"identityId":"a","emailId":"missing"}),
        json!({"identityId":"a","emailId":"e1","envelope":envelope("bob@example.org", &["bob@example.org"])}),
    ] {
        let response = f
            .call(
                "EmailSubmission/set",
                json!({"accountId":"a","create":{"s":object}}),
            )
            .await;
        assert!(
            response["methodResponses"][0][1]["notCreated"]["s"]["type"].is_string(),
            "{response}"
        );
        assert_eq!(f.db.account("a").unwrap(), before);
        assert!(f.db.submissions("a", None).unwrap().records.is_empty());
        assert!(f.db.claim_outbound(100).unwrap().is_empty());
        assert!(f.db.claim(100).unwrap().is_empty());
    }
    let foreign = f
        .call(
            "EmailSubmission/set",
            json!({"accountId":"b","create":{"s":{"identityId":"b","emailId":"e1"}}}),
        )
        .await;
    assert_eq!(foreign["methodResponses"][0][1]["type"], "accountNotFound");
}

#[tokio::test]
async fn malformed_envelopes_refuse_all_recipients_before_acceptance() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    for value in [
        envelope("", &["visible@remote.org"]),
        envelope("alice@example.org", &[]),
        envelope(
            "alice@example.org",
            &["bob@example.org", "x\r\nRCPT TO:victim@remote.org"],
        ),
        json!({"mailFrom":{"email":"alice@example.org","parameters":{"UNKNOWN":"x"}},"rcptTo":[{"email":"visible@remote.org"}]}),
        json!({"mailFrom":{"email":"alice@example.org"},"rcptTo":[{"email":"visible@remote.org","parameters":{"NOTIFY":4}}]}),
        json!({"mailFrom":{"email":"alice@example.org"},"rcptTo":[{"email":"visible@remote.org","unknown":"x"}]}),
    ] {
        let response = f.create(value).await;
        assert!(
            response["methodResponses"][0][1]["notCreated"]["s"]["type"].is_string(),
            "{response}"
        );
        assert!(f.db.submissions("a", None).unwrap().records.is_empty());
        assert!(f.db.claim_outbound(100).unwrap().is_empty());
        assert!(f.db.claim(100).unwrap().is_empty());
    }
}

#[tokio::test]
async fn forged_message_author_is_rejected_even_with_a_valid_explicit_envelope() {
    for raw in [
        b"From: bob@example.org\r\nTo: bob@example.org\r\n\r\nforged\r\n".as_slice(),
        b"From: alice@example.org\r\nSender: bob@example.org\r\nTo: bob@example.org\r\n\r\nforged\r\n".as_slice(),
    ] {
        let f = Fixture::new(":memory:", raw, Arc::new(OwnerPolicy));
        let response = f.create(envelope("alice@example.org", &["bob@example.org"])).await;
        assert!(response["methodResponses"][0][1]["notCreated"]["s"]["type"].is_string(), "{response}");
        assert!(f.db.claim(100).unwrap().is_empty());
        assert!(f.db.submissions("a", None).unwrap().records.is_empty());
    }
}

struct NoSubmit;
impl Policy for NoSubmit {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), Error> {
        OwnerPolicy.authorize(principal, action, account)?;
        if action == Action::Submit {
            Err(Error::Forbidden)
        } else {
            Ok(())
        }
    }
}
#[tokio::test]
async fn mailbox_write_permission_does_not_grant_submission_permission() {
    let f = Fixture::new(":memory:", RAW, Arc::new(NoSubmit));
    let response = f.create(Value::Null).await;
    assert!(
        response["methodResponses"][0][1]["notCreated"]["s"]["type"].is_string(),
        "{response}"
    );
    assert!(f.db.submissions("a", None).unwrap().records.is_empty());
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
}

#[tokio::test]
async fn folded_blind_headers_are_removed_from_every_delivery_but_original_is_preserved() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let response = f.create(Value::Null).await;
    id(&response);
    let leases = f.db.claim_outbound(100).unwrap();
    let recipients: std::collections::BTreeSet<_> =
        leases.iter().map(|l| l.recipient.as_str()).collect();
    assert!(
        recipients.contains("visible@remote.org")
            && recipients.contains("blind@remote.org")
            && recipients.contains("hidden@remote.org"),
        "{recipients:?}"
    );
    for lease in leases {
        let raw = f.db.outbound_message(&lease).unwrap().raw;
        let text = std::str::from_utf8(&raw).unwrap();
        assert!(!text.to_ascii_lowercase().contains("bcc:"), "{text}");
        for hidden in [
            "blind@remote.org",
            "hidden@remote.org",
            "hidden-again@remote.org",
        ] {
            assert!(!text.contains(hidden), "{text}");
        }
    }
    assert_eq!(f.db.blob("a", "e1").unwrap(), RAW);
}
