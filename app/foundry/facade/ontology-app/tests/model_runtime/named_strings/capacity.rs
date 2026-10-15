use super::*;
fn boundary_model() -> ModelDocument {
    let mut model = named_model();
    model.named_string_types[0]
        .members
        .extend((0..128).map(|index| format!("Test{index:03}{}", "x".repeat(4000))));
    model.entities[0]
        .properties
        .push(conformance::property("padding", false));
    let mut store =
        ReferenceModelStore::initialize(Fixture::binding(), Fixture::baseline()).unwrap();
    let draft = store
        .create_draft(&Fixture::caller(), 1, model.clone())
        .unwrap();
    let prepared = store.prepare(&Fixture::caller(), draft.id, 1, 1).unwrap();
    let mut needed = MAX_ONTOLOGY_SCHEMA_BYTES
        - schema_bytes(prepared.model_registry(), TENANT)
            .unwrap()
            .len();
    if needed % 2 == 1 {
        model.entities[0]
            .properties
            .last_mut()
            .unwrap()
            .name
            .push('x');
        needed -= 1;
    }
    for member in model.named_string_types[0].members.iter_mut().skip(2) {
        let extra = (4096 - member.len()).min(needed / 2);
        member.push_str(&"x".repeat(extra));
        needed -= extra * 2;
    }
    assert_eq!(needed, 0);
    model
}
#[tokio::test]
async fn actual_schema_response_admits_exact_read_cap_and_overflow_never_commits_or_activates() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&[
        "foundry-model-author",
        "foundry-model-publisher",
        "foundry-operator",
    ])
    .await;
    let process = fixture.start(&iam.origin).await;
    let exact = boundary_model();
    for overflow in [true, false] {
        let mut document = exact.clone();
        if overflow {
            document.entities[0]
                .properties
                .last_mut()
                .unwrap()
                .name
                .push('x');
        }
        let saved = ModelDraftWire::decode(
            &bytes(
                process
                    .request(Method::POST, "model/drafts")
                    .header("content-type", "application/json")
                    .body(
                        CreateModelDraft {
                            expected_generation: ModelNumber::new(1).unwrap(),
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
        .unwrap();
        let input = PrepareModelDraft {
            expected_version: saved.version,
            expected_base: saved.base_generation,
        }
        .encode()
        .unwrap();
        let path = format!("model/drafts/{}", saved.id.get());
        if overflow {
            let checked = ModelValidation::decode(
                &bytes(
                    process
                        .request(Method::POST, &format!("{path}/validate"))
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
            assert_eq!(checked.diagnostics[0].code, "schema_response_too_large");
            assert_eq!(checked.diagnostics[0].path, "schemas");
            assert_eq!(
                process
                    .request(Method::POST, &format!("{path}/publish"))
                    .header("content-type", "application/json")
                    .body(input)
                    .send()
                    .await
                    .unwrap()
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            let lifecycle = ModelLifecycle::decode(
                &bytes(
                    process
                        .request(Method::GET, "model/lifecycle")
                        .send()
                        .await
                        .unwrap(),
                    StatusCode::OK,
                )
                .await,
            )
            .unwrap();
            assert_eq!(lifecycle.observed_generation.unwrap().get(), 1);
            assert_eq!(lifecycle.desired_generation.get(), 1);
            assert!(lifecycle.ready);
        } else {
            let published = ModelPublished::decode(
                &bytes(
                    process
                        .request(Method::POST, &format!("{path}/publish"))
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
            assert_eq!(published.lifecycle.observed_generation.unwrap().get(), 2);
            assert!(published.lifecycle.ready);
        }
    }
    let wire = bytes(
        process
            .request(Method::GET, "schemas")
            .send()
            .await
            .unwrap(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(wire.len(), MAX_ONTOLOGY_SCHEMA_BYTES);
    let insertion = wire
        .windows(br#""value_type":{"kind":"named_string""#.len())
        .any(|bytes| bytes == br#""value_type":{"kind":"named_string""#);
    let sorted = wire
        .windows(br#""value_type":{"display_name":"Test status""#.len())
        .any(|bytes| bytes == br#""value_type":{"display_name":"Test status""#);
    assert_ne!(insertion, sorted);
    eprintln!(
        "served schema JSON map order: {} ({} bytes)",
        if insertion { "insertion" } else { "sorted" },
        wire.len()
    );
    let decoded: serde_json::Value = serde_json::from_slice(&wire).unwrap();
    assert_eq!(
        decoded[0]["properties"][2]["value_type"]["kind"],
        "named_string"
    );
    drop(process);
    use foundry_records_draft::RecordsLog;
    let log =
        foundry_records_sqlite_draft::SqliteRecordsLog::open(&fixture.config.action_log).unwrap();
    assert_eq!(log.head(TENANT).unwrap(), 0);
}
