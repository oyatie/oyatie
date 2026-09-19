use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::{BlobStore, MetadataStore};
use mail_kernel::{Account, Command};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
async fn call(app: &Router, name: &str, mut args: Value) -> Value {
    if args["accountId"].is_null() {
        args["accountId"] = json!("a");
    }
    let body = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[[name,args,"x"]]});
    let request = Request::post("/jmap")
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    let response: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    response["methodResponses"][0][1].clone()
}

async fn submit(app: &Router, email: &str, day: u8) -> String {
    let response = call(app, "EmailSubmission/set", json!({"create":{"s":{"identityId":"a","emailId":email,"envelope":{"mailFrom":{"email":"alice@example.org","parameters":{"HOLDUNTIL":format!("2079-11-{day:02}T05:00:00Z")}},"rcptTo":[{"email":"remote@example.net"}]}}}})).await;
    response["created"]["s"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("submission creation: {response}"))
        .to_owned()
}

async fn fixture() -> (Arc<SqliteStore>, Router, Vec<String>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    for revision in 0..3 {
        db.execute("a", mail_api::Precondition::Observed(revision), vec![db.append("a", vec!["inbox".into()], b"From: alice@example.org\r\nTo: remote@example.net\r\nSubject: query\r\n\r\nbody\r\n", vec![], 1).unwrap()]).unwrap();
    }
    let service = Arc::new(MailService {
        outbound: Some(db.clone()),
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let app = mail_protocol_imap::jmap_router(service, "https://mail.example.org".into());
    let mut ids = vec![];
    for index in 0..3 {
        ids.push(submit(&app, &format!("e{}", index + 1), 20 + index).await);
    }
    (db, app, ids)
}

#[tokio::test]
async fn get_projects_real_submission_properties_and_refuses_invalid_shapes() {
    let (_, app, ids) = fixture().await;
    let response = call(
        &app,
        "EmailSubmission/get",
        json!({"ids":[ids[1],"absent",ids[0],ids[1]],"properties":["emailId"]}),
    )
    .await;
    assert_eq!(
        response["list"],
        json!([{"id":ids[1],"emailId":"e2"},{"id":ids[0],"emailId":"e1"}])
    );
    assert_eq!(response["notFound"], json!(["absent"]));
    let response = call(&app, "EmailSubmission/get", json!({"ids":[ids[0]]})).await;
    let record = &response["list"][0];
    assert_eq!(record["sendAt"], "2079-11-20T05:00:00Z");
    assert_eq!(record["undoStatus"], "pending");
    assert_eq!(record["envelope"]["mailFrom"]["email"], "alice@example.org");
    assert!(record["envelope"]["rcptTo"][0]["parameters"].is_null());
    let delivery = &record["deliveryStatus"]["remote@example.net"];
    assert_eq!(delivery["delivered"], "queued");
    assert_eq!(delivery["displayed"], "unknown");
    assert_eq!(record["dsnBlobIds"], json!([]));
    assert_eq!(record["mdnBlobIds"], json!([]));
    for args in [
        json!({"ids":[1]}),
        json!({"ids":false}),
        json!({"properties":["secret"]}),
        json!({"properties":{}}),
    ] {
        assert_eq!(
            call(&app, "EmailSubmission/get", args).await["type"],
            "invalidArguments"
        );
    }
    assert_eq!(
        call(&app, "EmailSubmission/get", json!({"ids":vec!["id";257]})).await["type"],
        "tooManyObjects"
    );
    assert_eq!(
        call(
            &app,
            "EmailSubmission/get",
            json!({"accountId":"b","ids":[ids[0]]})
        )
        .await["type"],
        "accountNotFound"
    );
}

#[tokio::test]
async fn query_filters_sorts_and_pages_indexed_results() {
    let (_, app, ids) = fixture().await;
    let response = call(
        &app,
        "EmailSubmission/query",
        json!({"position":-1,"calculateTotal":true}),
    )
    .await;
    assert_eq!(response["ids"], json!([ids[2]]));
    assert_eq!(response["position"], 2);
    assert_eq!(response["total"], 3);
    let response = call(
        &app,
        "EmailSubmission/query",
        json!({"anchor":ids[1],"anchorOffset":-1,"limit":1}),
    )
    .await;
    assert_eq!(response["ids"], json!([ids[0]]));
    assert!(response.get("total").is_none());
    let response = call(&app,"EmailSubmission/query",json!({"filter":{"operator":"AND","conditions":[{"identityIds":["a"]},{"after":"2079-11-20T05:00:00Z"},{"before":"2079-11-22T05:00:00Z"}]}})).await;
    assert_eq!(response["ids"], json!([ids[1]]));
    let response = call(
        &app,
        "EmailSubmission/query",
        json!({"filter":{"operator":"NOT","conditions":[{"emailIds":["e1"]},{"emailIds":["e3"]}]}}),
    )
    .await;
    assert_eq!(response["ids"], json!([ids[1]]));
    let response = call(
        &app,
        "EmailSubmission/query",
        json!({"sort":[{"property":"emailId","isAscending":false}]}),
    )
    .await;
    assert_eq!(response["ids"], json!([ids[2], ids[1], ids[0]]));
    assert_eq!(
        call(&app, "EmailSubmission/query", json!({"anchor":"absent"})).await["type"],
        "anchorNotFound"
    );
    assert_eq!(
        call(&app, "EmailSubmission/query", json!({"limit":0})).await["ids"],
        json!([])
    );
}

#[tokio::test]
async fn changes_use_submission_state_and_complete_revision_pages() {
    let (db, app, ids) = fixture().await;
    let state = call(&app, "EmailSubmission/get", json!({"ids":[]})).await["state"].clone();
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(account.revision),
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
    )
    .unwrap();
    let unchanged = call(&app, "EmailSubmission/changes", json!({"sinceState":state})).await;
    assert_eq!(unchanged["newState"], state);
    assert_eq!(unchanged["updated"], json!([]));
    call(&app, "EmailSubmission/set", json!({"destroy":[ids[0]]})).await;
    call(
        &app,
        "EmailSubmission/set",
        json!({"update":{&ids[1]:{"undoStatus":"canceled"}}}),
    )
    .await;
    let first = call(
        &app,
        "EmailSubmission/changes",
        json!({"sinceState":state,"maxChanges":1}),
    )
    .await;
    assert_eq!(first["destroyed"], json!([ids[0]]));
    assert_eq!(first["hasMoreChanges"], true);
    let next = call(
        &app,
        "EmailSubmission/changes",
        json!({"sinceState":first["newState"],"maxChanges":1}),
    )
    .await;
    assert_eq!(next["updated"], json!([ids[1]]));
    assert_eq!(next["hasMoreChanges"], false);
    assert_eq!(
        call(
            &app,
            "EmailSubmission/changes",
            json!({"sinceState":"18446744073709551615"})
        )
        .await["type"],
        "cannotCalculateChanges"
    );
    assert_eq!(
        call(
            &app,
            "EmailSubmission/changes",
            json!({"sinceState":state,"maxChanges":0})
        )
        .await["type"],
        "invalidArguments"
    );
}

#[tokio::test]
async fn query_changes_reconcile_filter_membership_and_respect_up_to_id() {
    let (_, app, ids) = fixture().await;
    let filter = json!({"undoStatus":"pending"});
    let old = call(&app, "EmailSubmission/query", json!({"filter":filter})).await;
    call(&app, "EmailSubmission/set", json!({"destroy":[ids[0]]})).await;
    call(
        &app,
        "EmailSubmission/set",
        json!({"update":{&ids[1]:{"undoStatus":"canceled"}}}),
    )
    .await;
    let added = submit(&app, "e3", 23).await;
    let args = json!({"sinceQueryState":old["queryState"],"filter":filter,"calculateTotal":true});
    let response = call(&app, "EmailSubmission/queryChanges", args.clone()).await;
    assert_eq!(response["removed"], json!([ids[0], ids[1]]));
    assert_eq!(response["added"], json!([{"id":added,"index":1}]));
    assert_eq!(response["total"], 2);
    let mut limited = args.clone();
    limited["upToId"] = json!(ids[2]);
    limited["maxChanges"] = json!(2);
    let response = call(&app, "EmailSubmission/queryChanges", limited).await;
    assert_eq!(response["added"], json!([]));
    assert_eq!(response["removed"], json!([ids[0], ids[1]]));
    let mut limited = args.clone();
    limited["maxChanges"] = json!(2);
    assert_eq!(
        call(&app, "EmailSubmission/queryChanges", limited).await["type"],
        "tooManyChanges"
    );
    let mut changed = args;
    changed["filter"] = Value::Null;
    assert_eq!(
        call(&app, "EmailSubmission/queryChanges", changed).await["type"],
        "cannotCalculateChanges"
    );
}

#[tokio::test]
async fn malformed_query_arguments_fail_before_reading_results() {
    let (_, app, _) = fixture().await;
    for (args, expected) in [
        (
            json!({"filter":{"undoStatus":"invalid"}}),
            "invalidArguments",
        ),
        (json!({"filter":{"after":"not-a-date"}}), "invalidArguments"),
        (
            json!({"filter":{"operator":"AND","conditions":null}}),
            "invalidArguments",
        ),
        (json!({"filter":{"unexpected":true}}), "unsupportedFilter"),
        (json!({"sort":[{"property":"sendAt"}]}), "unsupportedSort"),
        (
            json!({"sort":[{"property":"sentAt","isAscending":"yes"}]}),
            "invalidArguments",
        ),
        (json!({"position":true}), "invalidArguments"),
        (json!({"anchorOffset":"1"}), "invalidArguments"),
        (json!({"calculateTotal":1}), "invalidArguments"),
        (json!({"limit":-1}), "invalidArguments"),
    ] {
        assert_eq!(
            call(&app, "EmailSubmission/query", args.clone()).await["type"],
            expected,
            "{args}"
        );
    }
    let mut filter = json!({});
    for _ in 0..20 {
        filter = json!({"operator":"NOT","conditions":[filter]});
    }
    assert_eq!(
        call(&app, "EmailSubmission/query", json!({"filter":filter})).await["type"],
        "unsupportedFilter"
    );
}
