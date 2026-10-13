use crate::model_support::*;
use foundry_model_api::conformance::property;

pub(crate) fn maximum_document() -> ModelDocument {
    let mut document = changed();
    let entity_index = document
        .entities
        .iter()
        .position(|entity| entity.id == "ety_record")
        .unwrap();
    let display = ModelDisplay {
        display_name: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        description: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        icon: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        color: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        plural_name: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
    };
    document.entities[entity_index].display = Some(display.clone());
    for property in &mut document.entities[entity_index].properties {
        property.display = Some(display.clone());
    }
    for index in 0..51 {
        let mut row = property(&format!("additional_{index}"), false);
        row.display = Some(display.clone());
        document.entities[entity_index].properties.push(row);
    }
    let mut excess = serde_json::to_vec(&document).unwrap().len() - MAX_MODEL_BYTES;
    for row in &mut document.entities[entity_index].properties {
        let display = row.display.as_mut().unwrap();
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
    document
}
