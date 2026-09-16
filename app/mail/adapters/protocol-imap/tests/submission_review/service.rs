use super::support::*;
use mail_api::{DeliveryQueue, SubmissionQueue, SubmissionStore};
use mail_kernel::{EnvelopeAddress, SubmissionEnvelope};
use mail_service::{OwnerPolicy, SubmitEmail};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

#[test]
fn direct_service_rejects_unimplemented_or_invalid_schedule_parameters() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    for (parameters, recipient_parameters) in [
        (vec![("UNKNOWN", Some("x"))], vec![]),
        (vec![("HOLDFOR", None)], vec![]),
        (vec![("HOLDFOR", Some("4294967296"))], vec![]),
        (vec![("HOLDFOR", Some("+30"))], vec![]),
        (vec![("HOLDUNTIL", Some("2079-02-30T00:00:00Z"))], vec![]),
        (
            vec![("HOLDUNTIL", Some("2079-11-20T05:00:00+00:00"))],
            vec![],
        ),
        (
            vec![
                ("HOLDFOR", Some("30")),
                ("HOLDUNTIL", Some("2079-11-20T05:00:00Z")),
            ],
            vec![],
        ),
        (vec![], vec![("NOTIFY", None)]),
    ] {
        let convert = |entries: Vec<(&str, Option<&str>)>| {
            entries
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.map(str::to_owned)))
                .collect::<BTreeMap<_, _>>()
        };
        let request = SubmitEmail {
            identity_id: "a".into(),
            email_id: "e1".into(),
            envelope: Some(SubmissionEnvelope {
                mail_from: EnvelopeAddress {
                    email: "alice@example.org".into(),
                    parameters: convert(parameters),
                },
                rcpt_to: vec![EnvelopeAddress {
                    email: "visible@remote.org".into(),
                    parameters: convert(recipient_parameters),
                }],
            }),
        };
        assert!(f.service.submit_email(TOKEN, "a", 0, request).is_err());
        assert!(f.db.submissions("a", None).unwrap().records.is_empty());
        assert!(f.db.claim_outbound(100).unwrap().is_empty());
        assert!(f.db.claim(100).unwrap().is_empty());
    }
}

#[tokio::test]
async fn session_advertises_submission_account_and_implemented_delay_bounds() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let response = f
        .app
        .clone()
        .oneshot(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let value: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let capability = "urn:ietf:params:jmap:submission";
    assert!(value["capabilities"][capability].is_object());
    assert_eq!(value["primaryAccounts"][capability], "a");
    assert_eq!(
        value["accounts"]["a"]["accountCapabilities"][capability]["maxDelayedSend"],
        json!(u32::MAX)
    );
    assert!(value["accounts"]["a"]["accountCapabilities"][capability]["submissionExtensions"]["FUTURERELEASE"].is_array());
}

#[tokio::test]
async fn local_recipient_quota_failure_rolls_back_remote_jobs_and_submission_history() {
    let path = db_path();
    let f = Fixture::new(&path, RAW, Arc::new(OwnerPolicy));
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE accounts SET state=json_set(state,'$.quota_bytes',0) WHERE id='b'",
            [],
        )
        .unwrap();
    let response = f
        .create(envelope(
            "alice@example.org",
            &["bob@example.org", "visible@remote.org"],
        ))
        .await;
    assert_eq!(
        response["methodResponses"][0][1]["notCreated"]["s"]["type"], "overQuota",
        "{response}"
    );
    assert!(f.db.submissions("a", None).unwrap().records.is_empty());
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
    assert!(f.db.claim(100).unwrap().is_empty());
    let connection = rusqlite::Connection::open(&path).unwrap();
    for table in [
        "submission_versions",
        "submission_schedule",
        "submitted_messages",
        "queued_messages",
    ] {
        let count: i64 = connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "{table}");
    }
    drop(connection);
    drop(f);
    std::fs::remove_file(path).unwrap();
}
