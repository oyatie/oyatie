use super::*;

#[tokio::test]
async fn struct_default_publication_capture_and_restart_keep_exact_document() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&[
        "foundry-model-author",
        "foundry-model-publisher",
        "foundry-operator",
    ])
    .await;
    let process = fixture.start(&iam.origin).await;
    let mut document = Fixture::baseline().document;
    document.entities[0].revision = 2;
    let mut property = conformance::property("configuration", false);
    property.tier = ModelTier::Struct;
    property.value_type = Some(ModelValueType::Struct {
        fields: ["z", "a"]
            .map(|name| ModelStructField {
                name: name.into(),
                required: true,
                value_type: ModelValueType::Scalar {
                    kind: ModelScalar::String,
                },
            })
            .to_vec(),
    });
    property.default_value = Some(ModelValue::Struct {
        fields: ["z", "a"]
            .map(|name| (name.into(), ModelValue::String { value: name.into() }))
            .into(),
    });
    document.entities[0].properties.push(property);
    let mut wire = serde_json::to_value(CreateModelDraft {
        expected_generation: ModelNumber::new(1).unwrap(),
        document: document.clone(),
    })
    .unwrap();
    wire["document"]["entities"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()["default_value"]["fields"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let saved = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, "model/drafts")
                .header("content-type", "application/json")
                .body(serde_json::to_vec(&wire).unwrap())
                .send()
                .await
                .unwrap(),
            StatusCode::CREATED,
        )
        .await,
    )
    .unwrap();
    assert_eq!(saved.document, document);
    publish(&process, &saved).await;
    write(
        &process,
        "ent_struct",
        "struct",
        serde_json::json!({"name":"Struct defaults"}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        object(&process, "ent_struct", 2).await["properties"]["configuration"]["value"],
        serde_json::json!({"a":"a","z":"z"})
    );
    drop(process);
    let store = fixture.open();
    let history = store.history(&Fixture::caller()).unwrap();
    let registry = history.registry().unwrap();
    assert_eq!(
        ModelDocument::capture(&registry, TENANT).unwrap(),
        history.head().unwrap().document
    );
    drop(store);
    let restarted = fixture.start(&iam.origin).await;
    let reopened = ModelWorkspace::decode(
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
    assert_eq!(reopened.document, saved.document);
    assert_eq!(
        object(&restarted, "ent_struct", 2).await["properties"]["configuration"]["value"],
        serde_json::json!({"a":"a","z":"z"})
    );
}
