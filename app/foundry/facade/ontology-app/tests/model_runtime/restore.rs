use crate::support::*;
use foundry_model_api::{control::*, *};
use reqwest::{Method, StatusCode};

#[tokio::test]
async fn published_metadata_restores_trusted_actions_and_retained_reader_pins_on_restart() {
    let fixture = Fixture::new();
    fixture.initialize();
    let mut store = fixture.open();
    let mut document = store.head(&Fixture::caller()).unwrap().document;
    document.entities[0].revision = 2;
    document.entities[0]
        .properties
        .push(conformance::property("optional_value", false));
    let draft = store.create_draft(&Fixture::caller(), 1, document).unwrap();
    let prepared = store
        .prepare(
            &Fixture::caller(),
            draft.id,
            draft.version,
            draft.base_generation,
        )
        .unwrap();
    store.publish(&Fixture::caller(), &prepared).unwrap();
    drop(store);
    let iam = Iam::start().await;
    let process = fixture.start(&iam.origin).await;
    let response=process.client.post(format!("{}/v1/actions",process.origin)).bearer_auth("runtime-operator")
        .json(&serde_json::json!({"object_ref":"ent_record","action_type":"aty_record_write","idempotency_key":"runtime-write","occurred_at_epoch_seconds":1,
            "properties":{"name":"Native record","note":"durable"}})).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.json::<serde_json::Value>().await.unwrap()["outcome"],
        "applied"
    );
    drop(process);
    let restarted = fixture.start(&iam.origin).await;
    let published = ModelWorkspace::decode(
        &bytes(
            restarted
                .request(Method::GET, "model")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(published.generation.get(), 2);
    let value: serde_json::Value = serde_json::from_slice(
        &bytes(
            restarted
                .request(Method::GET, "objects/ent_record?type=ety_record&revision=1")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(value["written_revision"], 2);
    assert_eq!(value["properties"]["name"]["value"], "Native record");
    assert!(value["properties"].get("optional_value").is_none());
}
