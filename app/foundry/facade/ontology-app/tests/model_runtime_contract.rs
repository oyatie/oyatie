#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use foundry_model_api::{control::*, *};
use reqwest::{Method, StatusCode};
#[path = "model_runtime/hot_publish.rs"]
mod hot_publish;
#[path = "model_runtime/refusals.rs"]
mod refusals;
#[path = "model_runtime/replay.rs"]
mod replay;
#[path = "model_runtime/restore.rs"]
mod restore;
#[path = "model_runtime/support.rs"]
mod support;
use support::*;

#[tokio::test]
async fn standard_process_opens_established_model_through_real_human_routes() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start().await;
    let process = fixture.start(&iam.origin).await;
    let workspace = ModelWorkspace::decode(
        &bytes(
            process.request(Method::GET, "model").send().await.unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(workspace.generation.get(), 1);
    let created = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, "model/drafts")
                .header("content-type", "application/json")
                .body(
                    CreateModelDraft {
                        expected_generation: workspace.generation,
                        document: workspace.document,
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
    let mut document = created.document.clone();
    document.entities[0].display_name = "Work record".into();
    document.entities[0].revision = 2;
    document.entities[0].properties.push(ModelProperty {
        name: "tags".into(),
        tier: ModelTier::Vector,
        data_class: ModelPrivacy::internal_only(),
        required: false,
        display: None,
        value_type: Some(ModelValueType::Array {
            element: Box::new(ModelValueType::Scalar {
                kind: ModelScalar::String,
            }),
        }),
    });
    let path = format!("model/drafts/{}", created.id.get());
    let saved = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, &format!("{path}/save"))
                .header("content-type", "application/json")
                .body(
                    SaveModelDraft {
                        expected_version: created.version,
                        expected_base: created.base_generation,
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
    assert_eq!(saved.version.get(), 2);
    let validated = ModelValidation::decode(
        &bytes(
            process
                .request(Method::POST, &format!("{path}/validate"))
                .header("content-type", "application/json")
                .body(
                    PrepareModelDraft {
                        expected_version: saved.version,
                        expected_base: saved.base_generation,
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
    assert!(validated.valid);
    drop(process);
    let restarted = fixture.start(&iam.origin).await;
    let reopened = ModelDraftWire::decode(
        &bytes(
            restarted.request(Method::GET, &path).send().await.unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(reopened.version, saved.version);
    assert_eq!(reopened.document, document);
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
    assert_eq!(
        published.generation.get(),
        1,
        "saving/validation never publishes"
    );
    assert_eq!(published.document.entities[0].display_name, "Record");
    assert_eq!(
        restarted
            .request(Method::POST, &format!("{path}/publish"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn offline_initialize_has_no_iam_or_listener_or_log_dependency_and_refuses_reinitialization() {
    let fixture = Fixture::new();
    let output = fixture
        .command()
        .args(["initialize-model", TENANT])
        .env(
            "OYATIE_FOUNDRY_IAM_BASE_URL",
            "deliberately-invalid-and-unused",
        )
        .env("OYATIE_FOUNDRY_IAM_ALLOW_LOOPBACK_HTTP", "not-a-boolean")
        .env("OYATIE_FOUNDRY_ONTOLOGY_LISTEN_ADDR", "not-an-address")
        .env_remove("OYATIE_FOUNDRY_ONTOLOGY_OPERATORS")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(fixture.model_path().exists());
    for path in [
        &fixture.config.action_log,
        &fixture.config.denial_log,
        &fixture.config.projection_store,
    ] {
        assert!(
            !path.exists(),
            "offline establishment must not open other stores"
        );
    }
    assert!(
        !fixture
            .command()
            .args(["initialize-model", TENANT])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        fixture.open().head(&Fixture::caller()).unwrap().generation,
        1
    );
}

#[tokio::test]
async fn configured_origin_and_session_are_required_by_standard_model_routes() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start().await;
    let process = fixture.start(&iam.origin).await;
    let path = format!("{}/v1/tenants/{TENANT}/model", process.origin);
    bytes(
        process.client.get(&path).send().await.unwrap(),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let request = CreateModelDraft {
        expected_generation: ModelNumber::new(1).unwrap(),
        document: Fixture::baseline().document,
    }
    .encode()
    .unwrap();
    for origin in [None, Some("http://localhost:33001")] {
        let mut send = process
            .client
            .post(format!("{path}/drafts"))
            .header("cookie", format!("__Host-oya_session={TOKEN}"))
            .header("content-type", "application/json")
            .body(request.clone());
        if let Some(origin) = origin {
            send = send.header("origin", origin);
        }
        bytes(send.send().await.unwrap(), StatusCode::FORBIDDEN).await;
    }
    drop(process);
    assert!(
        fixture
            .open()
            .owned_drafts(
                &Fixture::caller(),
                &DraftPageRequest {
                    after: None,
                    limit: 25
                }
            )
            .unwrap()
            .drafts
            .is_empty()
    );
}
