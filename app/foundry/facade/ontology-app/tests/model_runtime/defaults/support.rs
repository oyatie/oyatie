use super::*;

pub(super) async fn draft(
    process: &Process,
    generation: u64,
    document: ModelDocument,
) -> ModelDraftWire {
    ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, "model/drafts")
                .header("content-type", "application/json")
                .body(
                    CreateModelDraft {
                        expected_generation: ModelNumber::new(generation).unwrap(),
                        document,
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
    .unwrap()
}

pub(super) async fn publish(process: &Process, saved: &ModelDraftWire) {
    let published = ModelPublished::decode(
        &bytes(
            process
                .request(
                    Method::POST,
                    &format!("model/drafts/{}/publish", saved.id.get()),
                )
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
    assert!(published.lifecycle.ready);
}

pub(super) async fn write(
    process: &Process,
    object: &str,
    key: &str,
    properties: serde_json::Value,
    expected: StatusCode,
) -> serde_json::Value {
    let response = process.client.post(format!("{}/v1/actions", process.origin)).bearer_auth("runtime-operator")
        .json(&serde_json::json!({"object_ref":object,"action_type":"aty_record_write","idempotency_key":key,"occurred_at_epoch_seconds":1,"properties":properties}))
        .send().await.unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(status, expected, "{body}");
    serde_json::from_str(&body).unwrap()
}

pub(super) async fn object(process: &Process, id: &str, revision: u32) -> serde_json::Value {
    serde_json::from_slice(
        &bytes(
            process
                .request(
                    Method::GET,
                    &format!("objects/{id}?type=ety_record&revision={revision}"),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap()
}

pub(super) fn export_proof(
    fixture: &Fixture,
    saved: &ModelDraftWire,
    before: &[foundry_records_draft::SealedEnvelope],
) {
    let materialized = &before[1].envelope.payload;
    if let Ok(directory) = std::env::var("OYATIE_FOUNDRY_DEFAULT_PROOF_DIRECTORY") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("defaulted-action.bin"), materialized).unwrap();
        std::fs::write(
            directory.join("legacy-action.bin"),
            &before[0].envelope.payload,
        )
        .unwrap();
        std::fs::write(
            directory.join("binding.json"),
            serde_json::to_vec(&Fixture::binding()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("baseline.json"),
            serde_json::to_vec(&Fixture::baseline()).unwrap(),
        )
        .unwrap();
        for suffix in ["", "-wal", "-shm"] {
            let source =
                std::path::PathBuf::from(format!("{}{suffix}", fixture.model_path().display()));
            if source.exists() {
                std::fs::copy(source, directory.join(format!("model.sqlite{suffix}"))).unwrap();
            }
        }
        let store = fixture.open();
        let historical = store
            .open_draft(&Fixture::caller(), saved.id.get())
            .unwrap();
        std::fs::write(
            directory.join("defaulted-draft.json"),
            serde_json::to_vec(&historical).unwrap(),
        )
        .unwrap();
        let publication = store.history(&Fixture::caller()).unwrap().publications[1].clone();
        std::fs::write(
            directory.join("defaulted-publication.json"),
            serde_json::to_vec(&publication).unwrap(),
        )
        .unwrap();
    }
}
