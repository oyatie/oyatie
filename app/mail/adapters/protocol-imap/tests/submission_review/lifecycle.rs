use super::support::*;
use mail_api::{DeliveryOutcome, DeliveryQueue, Store, SubmissionQueue, SubmissionStore};
use mail_service::OwnerPolicy;
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::test]
async fn scheduling_delays_both_local_and_remote_jobs_and_cancel_keeps_original_email() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let mut envelope = envelope(
        "alice@example.org",
        &["bob@example.org", "visible@remote.org"],
    );
    envelope["mailFrom"]["parameters"] = json!({"HOLDUNTIL":"2079-11-20T05:00:00Z"});
    let response = f.create(envelope).await;
    let id = id(&response);
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
    assert!(f.db.claim(100).unwrap().is_empty());
    let response = f
        .call(
            "EmailSubmission/set",
            json!({"accountId":"a","update":{&id:{"undoStatus":"canceled"}}}),
        )
        .await;
    assert!(
        response["methodResponses"][0][1]["updated"]
            .as_object()
            .unwrap()
            .contains_key(&id),
        "{response}"
    );
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
    assert!(f.db.claim(100).unwrap().is_empty());
    assert_eq!(f.db.blob("a", "e1").unwrap(), RAW);
    let response = f
        .call("EmailSubmission/get", json!({"accountId":"a","ids":[id]}))
        .await;
    assert_eq!(
        response["methodResponses"][0][1]["list"][0]["undoStatus"],
        "canceled"
    );
}

#[tokio::test]
async fn active_and_expired_outbound_claims_cannot_be_cancelled_or_unsent() {
    let path = db_path();
    let f = Fixture::new(&path, RAW, Arc::new(OwnerPolicy));
    let response = f
        .create(envelope("alice@example.org", &["visible@remote.org"]))
        .await;
    let id = id(&response);
    let leases = f.db.claim_outbound(100).unwrap();
    assert_eq!(leases.len(), 1);
    for expire in [false, true] {
        if expire {
            rusqlite::Connection::open(&path)
                .unwrap()
                .execute("UPDATE outbound_jobs SET lease_until=0", [])
                .unwrap();
        }
        let response = f
            .call(
                "EmailSubmission/set",
                json!({"accountId":"a","update":{&id:{"undoStatus":"canceled"}}}),
            )
            .await;
        assert_eq!(
            response["methodResponses"][0][1]["notUpdated"][&id]["type"], "cannotUnsend",
            "{response}"
        );
        let count: i64 = rusqlite::Connection::open(&path)
            .unwrap()
            .query_row("SELECT count(*) FROM outbound_jobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
    assert_eq!(f.db.blob("a", "e1").unwrap(), RAW);
    drop(f);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn local_claim_alone_prevents_cancellation_even_after_its_lease_expires() {
    let path = db_path();
    let f = Fixture::new(&path, RAW, Arc::new(OwnerPolicy));
    let response = f
        .create(envelope(
            "alice@example.org",
            &["bob@example.org", "visible@remote.org"],
        ))
        .await;
    let id = id(&response);
    assert_eq!(f.db.claim(100).unwrap().len(), 1);
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute("UPDATE delivery_jobs SET lease_until=0", [])
        .unwrap();
    let response = f
        .call(
            "EmailSubmission/set",
            json!({"accountId":"a","update":{&id:{"undoStatus":"canceled"}}}),
        )
        .await;
    assert_eq!(
        response["methodResponses"][0][1]["notUpdated"][&id]["type"], "cannotUnsend",
        "{response}"
    );
    assert_eq!(f.db.claim_outbound(100).unwrap().len(), 1);
    drop(f);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn destroying_history_does_not_delete_email_or_local_and_remote_jobs() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let response = f
        .create(envelope(
            "alice@example.org",
            &["bob@example.org", "visible@remote.org"],
        ))
        .await;
    let id = id(&response);
    let state = response["methodResponses"][0][1]["newState"].clone();
    let response = f
        .call(
            "EmailSubmission/set",
            json!({"accountId":"a","destroy":[&id]}),
        )
        .await;
    assert_eq!(
        response["methodResponses"][0][1]["destroyed"],
        json!([&id]),
        "{response}"
    );
    assert_eq!(f.service.deliver_pending(100).unwrap(), 1);
    let leases = f.db.claim_outbound(100).unwrap();
    assert_eq!(leases.len(), 1);
    f.db.finish_outbound(&leases[0], DeliveryOutcome::Delivered)
        .unwrap();
    let response = f
        .call("EmailSubmission/get", json!({"accountId":"a","ids":[&id]}))
        .await;
    assert_eq!(response["methodResponses"][0][1]["notFound"], json!([&id]));
    let changes = f
        .call(
            "EmailSubmission/changes",
            json!({"accountId":"a","sinceState":state}),
        )
        .await;
    assert_eq!(changes["methodResponses"][0][1]["destroyed"], json!([&id]));
    assert_eq!(f.db.account("b").unwrap().messages.len(), 1);
    assert_eq!(f.db.blob("a", "e1").unwrap(), RAW);
}

#[tokio::test]
async fn history_survives_queue_cleanup_and_reports_final_status_consistently() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let response = f.create(Value::Null).await;
    let id = id(&response);
    let state = response["methodResponses"][0][1]["newState"].clone();
    let leases = f.db.claim_outbound(100).unwrap();
    assert!(!leases.is_empty());
    for lease in leases {
        f.db.finish_outbound(&lease, DeliveryOutcome::Delivered)
            .unwrap();
    }
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
    let record =
        f.db.submissions("a", Some(std::slice::from_ref(&id)))
            .unwrap();
    assert_eq!(record.records.len(), 1);
    let response = f
        .call("EmailSubmission/get", json!({"accountId":"a","ids":[&id]}))
        .await;
    assert_eq!(
        response["methodResponses"][0][1]["list"][0]["undoStatus"],
        "final"
    );
    let changes = f
        .call(
            "EmailSubmission/changes",
            json!({"accountId":"a","sinceState":state}),
        )
        .await;
    assert_eq!(changes["methodResponses"][0][1]["updated"], json!([&id]));
    let query = f
        .call(
            "EmailSubmission/query",
            json!({"accountId":"a","filter":{"undoStatus":"final"}}),
        )
        .await;
    assert_eq!(query["methodResponses"][0][1]["ids"], json!([&id]));
}
