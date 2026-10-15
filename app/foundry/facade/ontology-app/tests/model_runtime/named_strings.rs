use crate::support::*;
use foundry_model_api::{control::*, *};
use reqwest::{Method, StatusCode};
fn named_model() -> ModelDocument {
    let mut model = Fixture::baseline().document;
    model.entities[0].revision = 2;
    model.named_string_types.push(ModelNamedStringType {
        id: "vty_test_status".into(),
        revision: 1,
        display_name: "Test status".into(),
        members: vec!["TestActive".into(), "TestLeave".into()],
    });
    for name in ["test_status", "other_test_status"] {
        let mut property = conformance::property(name, false);
        property.value_type = Some(ModelValueType::NamedString {
            type_id: "vty_test_status".into(),
            revision: 1,
        });
        model.entities[0].properties.push(property);
    }
    model
}
#[tokio::test]
async fn named_string_publication_hot_activates_pinned_schema_and_refuses_nonmember_actions_before_append()
 {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&[
        "foundry-model-author",
        "foundry-model-publisher",
        "foundry-operator",
    ])
    .await;
    let process = fixture.start(&iam.origin).await;
    let saved = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, "model/drafts")
                .header("content-type", "application/json")
                .body(
                    CreateModelDraft {
                        expected_generation: ModelNumber::new(1).unwrap(),
                        document: named_model(),
                    }
                    .encode()
                    .unwrap(),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::CREATED,
        )
        .await,
    )
    .unwrap();
    let input = PrepareModelDraft {
        expected_version: saved.version,
        expected_base: saved.base_generation,
    }
    .encode()
    .unwrap();
    let published = ModelPublished::decode(
        &bytes(
            process
                .request(
                    Method::POST,
                    &format!("model/drafts/{}/publish", saved.id.get()),
                )
                .header("content-type", "application/json")
                .body(input)
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(published.lifecycle.phase, ModelLifecyclePhase::Current);
    assert_eq!(published.lifecycle.observed_generation.unwrap().get(), 2);
    let schemas: serde_json::Value = serde_json::from_slice(
        &bytes(
            process
                .request(Method::GET, "schemas")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    let properties = schemas[0]["properties"].as_array().unwrap();
    let first = properties
        .iter()
        .find(|p| p["name"] == "test_status")
        .unwrap();
    let second = properties
        .iter()
        .find(|p| p["name"] == "other_test_status")
        .unwrap();
    assert_eq!(first["value_type"], second["value_type"]);
    assert_eq!(first["value_type"]["type_id"], "vty_test_status");
    let submit = |key: &str, status: &str| {
        process.client.post(format!("{}/v1/actions",process.origin)).bearer_auth("runtime-operator").json(&serde_json::json!({"object_ref":"ent_named","action_type":"aty_record_write","idempotency_key":key,"occurred_at_epoch_seconds":1,"properties":{"name":"Test record","test_status":status,"other_test_status":"TestLeave"}}))
    };
    assert_eq!(
        submit("named-valid", "TestActive")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let prior: serde_json::Value = serde_json::from_slice(
        &bytes(
            process
                .request(Method::GET, "objects/ent_named?type=ety_record&revision=2")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    let refused = submit("named-nonmember", "TestUnlisted")
        .send()
        .await
        .unwrap();
    assert!(!refused.status().is_success());
    let after: serde_json::Value = serde_json::from_slice(
        &bytes(
            process
                .request(Method::GET, "objects/ent_named?type=ety_record&revision=2")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(prior, after);
    assert_eq!(after["properties"]["test_status"]["value"], "TestActive");
    let pinned: serde_json::Value = serde_json::from_slice(
        &bytes(
            process
                .request(Method::GET, "objects/ent_named?type=ety_record&revision=1")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert!(pinned["properties"].get("test_status").is_none());
    drop(process);
    use foundry_records_draft::RecordsLog;
    let log =
        foundry_records_sqlite_draft::SqliteRecordsLog::open(&fixture.config.action_log).unwrap();
    assert_eq!(log.head(TENANT).unwrap(), 1);
    drop(log);
    let restarted = fixture.start(&iam.origin).await;
    let workspace = ModelWorkspace::decode(
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
    assert_eq!(workspace.document, named_model());
    let restored: serde_json::Value = serde_json::from_slice(
        &bytes(
            restarted
                .request(Method::GET, "objects/ent_named?type=ety_record&revision=2")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(restored, prior);
}

#[tokio::test]
async fn invalid_named_drafts_reopen_with_precise_native_diagnostics_and_never_publish() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&["foundry-model-author", "foundry-model-publisher"]).await;
    let process = fixture.start(&iam.origin).await;
    for (empty_label, code, path) in [
        (
            false,
            "invalid_named_string_members",
            "named_string_types[0].members",
        ),
        (
            true,
            "empty_display_name",
            "named_string_types[0].display_name",
        ),
    ] {
        let mut document = named_model();
        if empty_label {
            document.named_string_types[0].display_name.clear();
        } else {
            document.named_string_types[0].members.clear();
        }
        let saved = ModelDraftWire::decode(
            &bytes(
                process
                    .request(Method::POST, "model/drafts")
                    .header("content-type", "application/json")
                    .body(
                        CreateModelDraft {
                            expected_generation: ModelNumber::new(1).unwrap(),
                            document: document.clone(),
                        }
                        .encode()
                        .unwrap(),
                    )
                    .send()
                    .await
                    .unwrap(),
                StatusCode::CREATED,
            )
            .await,
        )
        .unwrap();
        let url = format!("model/drafts/{}", saved.id.get());
        let input = PrepareModelDraft {
            expected_version: saved.version,
            expected_base: saved.base_generation,
        }
        .encode()
        .unwrap();
        let checked = ModelValidation::decode(
            &bytes(
                process
                    .request(Method::POST, &format!("{url}/validate"))
                    .header("content-type", "application/json")
                    .body(input.clone())
                    .send()
                    .await
                    .unwrap(),
                StatusCode::OK,
            )
            .await,
        )
        .unwrap();
        assert!(!checked.valid);
        assert_eq!(checked.diagnostics[0].code, code);
        assert_eq!(checked.diagnostics[0].path, path);
        assert_eq!(
            process
                .request(Method::POST, &format!("{url}/publish"))
                .header("content-type", "application/json")
                .body(input)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let reopened = ModelDraftWire::decode(
            &bytes(
                process.request(Method::GET, &url).send().await.unwrap(),
                StatusCode::OK,
            )
            .await,
        )
        .unwrap();
        assert_eq!(reopened.document, document);
        assert!(reopened.publication.is_none());
    }
    let workspace = ModelWorkspace::decode(
        &bytes(
            process.request(Method::GET, "model").send().await.unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(workspace.generation.get(), 1);
}
