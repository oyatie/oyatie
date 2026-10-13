use super::maximum::maximum_document;
use crate::model_support::*;

#[tokio::test]
async fn exact_native_document_budget_is_created_and_one_extra_byte_is_refused() {
    let fixture = crate::facade_support::Fixture::new("model-full-envelope");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    let input = CreateModelDraft {
        expected_generation: number(1),
        document: maximum_document(),
    };
    let bytes = input.encode().unwrap();
    assert!(bytes.len() > MAX_MODEL_BYTES);
    let saved = ModelDraftWire::decode(
        &body(
            send(&state.router, request("POST", DRAFTS, bytes)).await,
            StatusCode::CREATED,
        )
        .await,
    )
    .unwrap();
    assert_eq!(saved.document, input.document);
    assert_eq!(saved.document.encode().unwrap().len(), MAX_MODEL_BYTES);
    let mut oversized = input;
    oversized
        .document
        .entities
        .iter_mut()
        .find(|entity| entity.id == "ety_record")
        .unwrap()
        .display_name
        .push('x');
    refusal(
        send(
            &state.router,
            request("POST", DRAFTS, serde_json::to_vec(&oversized).unwrap()),
        )
        .await,
        StatusCode::PAYLOAD_TOO_LARGE,
        ModelRefusalCode::BodyTooLarge,
    )
    .await;
}
