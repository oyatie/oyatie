use super::*;

#[test]
fn exact_draft_base_is_closed_and_keeps_decimal_identity() {
    for value in [1, 9_007_199_254_740_993, MAX_MODEL_VERSION] {
        let row = ModelDraftBase {
            draft_id: number(value),
            draft_version: number(value),
            base_generation: number(value),
            document: baseline().document,
        };
        assert_closed(&row);
        let bytes = row.encode().unwrap();
        assert_eq!(ModelDraftBase::decode(&bytes).unwrap(), row);
        let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for field in ["draft_id", "draft_version", "base_generation"] {
            assert_eq!(wire[field], value.to_string());
            let mut invalid = wire.clone();
            invalid[field] = serde_json::json!(value);
            assert!(ModelDraftBase::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
    }
}

#[test]
fn exact_draft_base_admits_full_escaped_document_and_refuses_one_extra_byte() {
    let mut document = maximum_document();
    document.entities[0].display_name = "\0".repeat(MAX_MODEL_TEXT_BYTES);
    let mut excess = serde_json::to_vec(&document).unwrap().len() - MAX_MODEL_BYTES;
    for property in &mut document.entities[0].properties {
        let display = property.display.as_mut().unwrap();
        for field in [
            &mut display.display_name,
            &mut display.description,
            &mut display.icon,
            &mut display.color,
            &mut display.plural_name,
        ] {
            let field = field.as_mut().unwrap();
            let remove = excess.min(field.len());
            field.truncate(field.len() - remove);
            excess -= remove;
        }
    }
    assert_eq!(excess, 0);
    assert_eq!(document.encode().unwrap().len(), MAX_MODEL_BYTES);
    let row = ModelDraftBase {
        draft_id: number(MAX_MODEL_VERSION),
        draft_version: number(MAX_MODEL_VERSION),
        base_generation: number(MAX_MODEL_VERSION),
        document,
    };
    let limit = model_wire_limits().unwrap().draft_base;
    let bytes = row.encode().unwrap();
    assert_eq!(bytes.len(), limit);
    assert_eq!(ModelDraftBase::decode(&bytes).unwrap(), row);
    let mut padded = bytes.clone();
    padded.push(b' ');
    assert!(ModelDraftBase::decode(&padded).is_err());
    let mut oversized = row;
    oversized.document.entities[0].properties[0]
        .display
        .as_mut()
        .unwrap()
        .display_name
        .as_mut()
        .unwrap()
        .push('x');
    assert!(oversized.encode().is_err());
    assert!(ModelDraftBase::decode(&serde_json::to_vec(&oversized).unwrap()).is_err());
}
