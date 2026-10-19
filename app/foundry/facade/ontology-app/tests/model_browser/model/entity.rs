use foundry_model_api::{ModelDisplay, ModelEntity, ModelProperty, conformance};

pub(super) fn entity(id: &str, name: &str, extra: Vec<ModelProperty>) -> ModelEntity {
    let mut properties = vec![
        conformance::property("id", true),
        conformance::property("name", true),
    ];
    properties.extend(extra);
    ModelEntity {
        id: id.into(),
        display_name: name.into(),
        properties,
        revision: 1,
        pillar: None,
        primary_key_property: Some("id".into()),
        title_property: Some("name".into()),
        display: Some(ModelDisplay {
            display_name: Some(name.into()),
            description: Some(format!("{name} declaration for controlled authoring proof")),
            icon: None,
            color: None,
            plural_name: Some(format!("{name}s")),
        }),
    }
}
