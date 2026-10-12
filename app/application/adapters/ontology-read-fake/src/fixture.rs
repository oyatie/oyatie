use application_ontology_read::{
    AuditDisposition, AuditRow, EntitySchema, HistoryRow, PinnedObject, Property,
    PropertyDefinition, PropertyTier, UpcastState, ValueTypeDeclaration,
};
use serde_json::json;

pub(super) fn properties(revision: u32) -> Vec<String> {
    let mut names = vec!["name", "active", "score"];
    if revision == 2 {
        names.push("note");
    }
    names.into_iter().map(str::to_owned).collect()
}

pub(super) fn schema() -> EntitySchema {
    let properties = [
        (
            "name",
            "PII_IDENTIFYING",
            true,
            Some(ValueTypeDeclaration::String {}),
        ),
        (
            "active",
            "PUBLIC",
            true,
            Some(ValueTypeDeclaration::Boolean {}),
        ),
        (
            "score",
            "INTERNAL_ONLY",
            true,
            Some(ValueTypeDeclaration::Integer {}),
        ),
        ("note", "INTERNAL_ONLY", false, None),
    ]
    .into_iter()
    .map(
        |(name, data_class, required, value_type)| PropertyDefinition {
            name: name.to_owned(),
            tier: PropertyTier::Scalar,
            data_class: data_class.to_owned(),
            required,
            value_type,
        },
    )
    .collect();
    EntitySchema {
        entity_type: super::FixtureOntologyReadSource::ENTITY_TYPE.to_owned(),
        revision: 2,
        properties,
        primary_key_property: None,
        title_property: Some("name".to_owned()),
    }
}

pub(super) fn objects() -> Vec<PinnedObject> {
    [
        ("alpha", "Fixture Alice", true, 7, 1),
        ("beta", "Fixture Bob", false, 11, 2),
    ]
    .into_iter()
    .map(|(id, name, active, score, written_revision)| {
        let mut properties = std::collections::BTreeMap::from([
            (
                "name".to_owned(),
                property("string", "PII_IDENTIFYING", json!(name)),
            ),
            (
                "active".to_owned(),
                property("boolean", "PUBLIC", json!(active)),
            ),
            (
                "score".to_owned(),
                property("integer", "INTERNAL_ONLY", json!(score)),
            ),
        ]);
        if written_revision == 2 {
            properties.insert(
                "note".to_owned(),
                property("string", "INTERNAL_ONLY", json!("Sample fixture only")),
            );
        }
        PinnedObject {
            object_ref: format!("ent_fixture_{id}"),
            written_revision,
            upcast_state: UpcastState::Current,
            properties,
        }
    })
    .collect()
}

fn property(value_type: &str, data_class: &str, value: serde_json::Value) -> Property {
    Property {
        value_type: value_type.to_owned(),
        data_class: data_class.to_owned(),
        value,
    }
}

pub(super) fn history(object_ref: &str) -> Vec<HistoryRow> {
    let entries = match object_ref {
        "ent_fixture_alpha" => vec![(1, 1, "fixture.create", 1), (2, 2, "fixture.update", 1)],
        "ent_fixture_beta" => vec![(3, 1, "fixture.create", 2)],
        _ => vec![],
    };
    entries
        .into_iter()
        .map(
            |(ordinal, object_sequence, action_type, schema_revision)| HistoryRow {
                ordinal,
                object_sequence,
                principal_id: "prn_fixture_user".to_owned(),
                decision_id: format!("fixture_decision_{ordinal}"),
                action_type: action_type.to_owned(),
                audit_event_type: "fixture.object_written".to_owned(),
                schema_revision,
            },
        )
        .collect()
}

pub(super) fn audit() -> Vec<AuditRow> {
    [(1, "alpha"), (2, "alpha"), (3, "beta"), (4, "missing")]
        .into_iter()
        .map(|(ordinal, id)| AuditRow {
            ordinal,
            object_ref: format!("ent_fixture_{id}"),
            action_type: if ordinal == 2 {
                "fixture.update"
            } else {
                "fixture.create"
            }
            .to_owned(),
            disposition: if ordinal == 4 {
                AuditDisposition::Poisoned
            } else {
                AuditDisposition::Applied
            },
            poison_reason: (ordinal == 4).then(|| "fixture rejected entry".to_owned()),
            principal_id: (ordinal != 4).then(|| "prn_fixture_user".to_owned()),
            decision_id: (ordinal != 4).then(|| format!("fixture_decision_{ordinal}")),
        })
        .collect()
}
