use crate::support::*;
use foundry_edits::decode_action_record;
use foundry_model_api::{control::*, *};
use foundry_records_draft::RecordsLog;
use foundry_records_sqlite_draft::SqliteRecordsLog;
use reqwest::{Method, StatusCode};

#[path = "defaults/structure.rs"]
mod structure;
#[path = "defaults/support.rs"]
mod support;
use support::*;

#[tokio::test]
async fn creation_defaults_survive_native_publication_retries_replay_and_process_restart() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&[
        "foundry-model-author",
        "foundry-model-publisher",
        "foundry-operator",
    ])
    .await;
    let process = fixture.start(&iam.origin).await;
    let input = serde_json::json!({"name":"Durable record"});
    write(
        &process,
        "ent_legacy",
        "legacy",
        input.clone(),
        StatusCode::OK,
    )
    .await;

    let mut document = Fixture::baseline().document;
    document.entities[0].revision = 2;
    document.named_string_types.push(ModelNamedStringType {
        id: "vty_employment_status".into(),
        revision: 1,
        display_name: "EmploymentStatus".into(),
        members: vec!["active".into(), "onLeave".into(), "ended".into()],
    });
    let mut status = conformance::property("employment_status", false);
    status.value_type = Some(ModelValueType::NamedString {
        type_id: "vty_employment_status".into(),
        revision: 1,
    });
    status.default_value = Some(ModelValue::String {
        value: "leave".into(),
    });
    document.entities[0].properties.push(status);
    let invalid = draft(&process, 1, document.clone()).await;
    let validation = ModelValidation::decode(
        &bytes(
            process
                .request(
                    Method::POST,
                    &format!("model/drafts/{}/validate", invalid.id.get()),
                )
                .header("content-type", "application/json")
                .body(
                    PrepareModelDraft {
                        expected_version: invalid.version,
                        expected_base: invalid.base_generation,
                    }
                    .encode()
                    .unwrap(),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert!(!validation.valid);
    assert_eq!(validation.diagnostics[0].code, "invalid_property_default");
    assert_eq!(
        validation.diagnostics[0].path,
        "entities.ety_record.properties.employment_status.default_value"
    );
    assert!(!validation.diagnostics[0].message.contains("leave"));
    drop(process);
    let process = fixture.start(&iam.origin).await;
    let reopened = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::GET, &format!("model/drafts/{}", invalid.id.get()))
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(reopened.document, document);
    document.entities[0]
        .properties
        .last_mut()
        .unwrap()
        .default_value = Some(ModelValue::String {
        value: "active".into(),
    });
    let saved = ModelDraftWire::decode(
        &bytes(
            process
                .request(
                    Method::POST,
                    &format!("model/drafts/{}/save", invalid.id.get()),
                )
                .header("content-type", "application/json")
                .body(
                    SaveModelDraft {
                        expected_version: reopened.version,
                        expected_base: reopened.base_generation,
                        document: document.clone(),
                    }
                    .encode()
                    .unwrap(),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    publish(&process, &saved).await;
    assert!(
        object(&process, "ent_legacy", 2).await["properties"]
            .get("employment_status")
            .is_none()
    );
    let first = write(
        &process,
        "ent_defaulted",
        "defaulted",
        input.clone(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        object(&process, "ent_defaulted", 2).await["properties"]["employment_status"]["value"],
        "active"
    );
    assert_eq!(
        write(
            &process,
            "ent_defaulted",
            "defaulted",
            input.clone(),
            StatusCode::OK
        )
        .await["ordinal"],
        first["ordinal"]
    );
    assert_eq!(
        write(
            &process,
            "ent_defaulted",
            "defaulted",
            serde_json::json!({"name":"Durable record","employment_status":"active"}),
            StatusCode::CONFLICT
        )
        .await["gate"],
        "log"
    );
    write(
        &process,
        "ent_invalid",
        "invalid",
        serde_json::json!({"name":"Durable record","employment_status":"leave"}),
        StatusCode::FORBIDDEN,
    )
    .await;
    write(
        &process,
        "ent_null",
        "null",
        serde_json::json!({"name":"Durable record","employment_status":null}),
        StatusCode::BAD_REQUEST,
    )
    .await;
    write(
        &process,
        "ent_replaced",
        "first-replace",
        input.clone(),
        StatusCode::OK,
    )
    .await;
    write(
        &process,
        "ent_replaced",
        "replacement",
        input.clone(),
        StatusCode::OK,
    )
    .await;
    assert!(
        object(&process, "ent_replaced", 2).await["properties"]
            .get("employment_status")
            .is_none()
    );
    drop(process);
    let before = SqliteRecordsLog::open(&fixture.config.action_log)
        .unwrap()
        .replay(TENANT, 1)
        .unwrap();
    assert_eq!(before.len(), 4);
    let materialized = &before[1].envelope.payload;
    assert_eq!(
        decode_action_record(materialized)
            .unwrap()
            .wire_format_version,
        2
    );
    assert_eq!(
        decode_action_record(&before[0].envelope.payload)
            .unwrap()
            .wire_format_version,
        1
    );

    let mut process = fixture.start(&iam.origin).await;
    for (generation, default) in [(2, Some("onLeave")), (3, None)] {
        document.entities[0].revision += 1;
        document.entities[0]
            .properties
            .last_mut()
            .unwrap()
            .default_value = default.map(|value| ModelValue::String {
            value: value.into(),
        });
        let saved = draft(&process, generation, document.clone()).await;
        publish(&process, &saved).await;
        for (id, key) in [("ent_legacy", "legacy"), ("ent_defaulted", "defaulted")] {
            assert_eq!(
                write(&process, id, key, input.clone(), StatusCode::OK).await["deduplicated"],
                true
            );
        }
        assert!(
            object(&process, "ent_legacy", document.entities[0].revision).await["properties"]
                .get("employment_status")
                .is_none()
        );
        assert_eq!(
            object(&process, "ent_defaulted", document.entities[0].revision).await["properties"]["employment_status"]
                ["value"],
            "active"
        );
        drop(process);
        let after = SqliteRecordsLog::open(&fixture.config.action_log)
            .unwrap()
            .replay(TENANT, 1)
            .unwrap();
        assert_eq!(
            after, before,
            "publication and retries cannot change logged input/effect"
        );
        process = fixture.start(&iam.origin).await;
    }
    assert_eq!(
        write(
            &process,
            "ent_defaulted",
            "defaulted",
            input.clone(),
            StatusCode::OK
        )
        .await["deduplicated"],
        true
    );
    assert_eq!(
        write(&process, "ent_legacy", "legacy", input, StatusCode::OK).await["deduplicated"],
        true
    );
    drop(process);

    support::export_proof(&fixture, &saved, &before);
}
