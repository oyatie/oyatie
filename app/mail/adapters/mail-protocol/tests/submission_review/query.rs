use super::support::*;
use mail_service::OwnerPolicy;
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::test]
async fn query_positions_beyond_the_last_submission_return_an_empty_bounded_page() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let created = f.create(Value::Null).await;
    let id = id(&created);
    for (page, expected_position) in [
        (json!({"position":999}), 0),
        (json!({"anchor":id,"anchorOffset":999}), 999),
    ] {
        let mut args = json!({"accountId":"a","calculateTotal":true});
        args.as_object_mut()
            .unwrap()
            .extend(page.as_object().unwrap().clone());
        let response = f.call("EmailSubmission/query", args).await;
        assert_eq!(
            response["methodResponses"][0][0], "EmailSubmission/query",
            "{response}"
        );
        let result = &response["methodResponses"][0][1];
        assert_eq!(result["ids"], json!([]));
        assert_eq!(result["position"], expected_position);
        assert_eq!(result["total"], 1);
    }
}

#[tokio::test]
async fn submission_query_filter_values_cannot_escape_the_authenticated_account() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    id(&f.create(Value::Null).await);
    for filter in [
        json!({"emailIds":["e1' OR 1=1 --"]}),
        json!({"identityIds":["a') OR account='b' --"]}),
        json!({"threadIds":["e1\" UNION SELECT id FROM accounts --"]}),
    ] {
        let response = f
            .call(
                "EmailSubmission/query",
                json!({"accountId":"a","filter":filter}),
            )
            .await;
        assert_eq!(
            response["methodResponses"][0][0], "EmailSubmission/query",
            "{response}"
        );
        assert_eq!(response["methodResponses"][0][1]["ids"], json!([]));
    }
}

#[tokio::test]
async fn retained_floor_preserves_net_changes_after_pruned_destroy_and_transient_creation() {
    let path = db_path();
    let f = Fixture::new(&path, RAW, Arc::new(OwnerPolicy));
    let removed = id(&f.create(Value::Null).await);
    let retained = id(&f.create(Value::Null).await);
    let destroy = f
        .call(
            "EmailSubmission/set",
            json!({"accountId":"a","destroy":[removed]}),
        )
        .await;
    assert_eq!(destroy["methodResponses"][0][1]["newState"], "3");
    let before = f
        .call(
            "EmailSubmission/query",
            json!({"accountId":"a","filter":{"undoStatus":"pending"}}),
        )
        .await;
    assert_eq!(before["methodResponses"][0][1]["ids"], json!([retained]));
    let state = before["methodResponses"][0][1]["queryState"]
        .as_str()
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute_batch("BEGIN IMMEDIATE; UPDATE submission_heads SET floor=3 WHERE account='a'; DELETE FROM submission_versions WHERE account='a' AND (until_revision<=3 OR (state IS NULL AND revision<=3)); COMMIT;").unwrap();
    let ephemeral = id(&f.create(Value::Null).await);
    let changed = f.call("EmailSubmission/set", json!({"accountId":"a","destroy":[ephemeral],"update":{&retained:{"undoStatus":"canceled"}}})).await;
    assert_eq!(changed["methodResponses"][0][1]["newState"], "6");
    let changes = f
        .call(
            "EmailSubmission/changes",
            json!({"accountId":"a","sinceState":"3"}),
        )
        .await;
    assert_eq!(changes["methodResponses"][0][0], "EmailSubmission/changes");
    let result = &changes["methodResponses"][0][1];
    assert_eq!(result["created"], json!([]));
    assert_eq!(result["destroyed"], json!([]));
    assert_eq!(result["updated"], json!([retained]));
    assert_eq!(result["newState"], "6");
    assert_eq!(result["hasMoreChanges"], false);
    let changes = f.call("EmailSubmission/queryChanges", json!({"accountId":"a","filter":{"undoStatus":"pending"},"sinceQueryState":state,"calculateTotal":true})).await;
    assert_eq!(
        changes["methodResponses"][0][0], "EmailSubmission/queryChanges",
        "{changes}"
    );
    let result = &changes["methodResponses"][0][1];
    assert_eq!(result["removed"], json!([retained]));
    assert_eq!(result["added"], json!([]));
    assert_eq!(result["total"], 0);
    let old = f
        .call(
            "EmailSubmission/changes",
            json!({"accountId":"a","sinceState":"2"}),
        )
        .await;
    assert_eq!(
        old["methodResponses"][0][1]["type"],
        "cannotCalculateChanges"
    );
    drop((f, sql));
    std::fs::remove_file(path).unwrap();
}
