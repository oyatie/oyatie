use serde_json::json;

use super::*;

#[test]
fn native_property_shapes_preserve_kind_privacy_and_value_without_coercion() {
    let wire = json!({
        "object_ref": "ent_sample",
        "written_revision": 1,
        "upcast_state": "upcast_pending",
        "properties": {
            "name": {"value_type": "string", "data_class": "PII_IDENTIFYING", "value": "Sample"},
            "count": {"value_type": "integer", "data_class": "INTERNAL_ONLY", "value": -9},
            "ratio": {"value_type": "double", "data_class": "FINANCIAL", "value": 1.25},
            "active": {"value_type": "boolean", "data_class": "PUBLIC", "value": false},
            "date": {"value_type": "date", "data_class": "PII_QUASI_IDENTIFIER", "value": "2026-10-12"},
            "time": {"value_type": "timestamp", "data_class": "AUDIT", "value": 1791792000000_i64},
            "items": {"value_type": "array", "data_class": "SECRET", "value": [1, true, "two"]},
            "record": {"value_type": "struct", "data_class": "PHI", "value": {"nested": [false, 3]}}
        }
    });
    let object: PinnedObject = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(&object).unwrap(), wire);
    let detail = ObjectDetail {
        requested_revision: 2,
        object,
    };
    assert_eq!(detail.requested_revision, 2);
    assert_eq!(detail.object.written_revision, 1);
    assert_eq!(detail.object.upcast_state, UpcastState::UpcastPending);
    assert!(serde_json::from_value::<PinnedObject>(json!({
        "object_ref": "ent_sample", "written_revision": 1, "upcast_state": "guessed", "properties": {}
    })).is_err());
}

#[test]
fn native_schema_and_log_rows_keep_their_distinct_evidence_fields() {
    let types = json!([{"entity_type": "ety_sample", "revision": 2, "properties": ["name"]}]);
    let history = json!([{
        "ordinal": 9, "object_sequence": 2, "principal_id": "prn_sample", "decision_id": "decision_sample",
        "action_type": "sample.update", "audit_event_type": "object_written", "schema_revision": 1
    }]);
    let audit = json!([{
        "ordinal": 10, "object_ref": "ent_rejected", "action_type": "sample.create", "disposition": "poisoned",
        "poison_reason": "sample rejected entry", "principal_id": null, "decision_id": null
    }]);
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Vec<EntityType>>(types.clone()).unwrap())
            .unwrap(),
        types
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Vec<HistoryRow>>(history.clone()).unwrap())
            .unwrap(),
        history
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Vec<AuditRow>>(audit.clone()).unwrap())
            .unwrap(),
        audit
    );
}

#[test]
fn typed_queries_bound_pages_and_refuse_incoherent_ranges() {
    let mut query = ObjectListQuery::first("ety_sample", 1);
    assert_eq!(query.limit, 100);
    assert_eq!(query.validate(), Ok(()));
    for limit in [0, 1001] {
        query.limit = limit;
        assert_eq!(query.validate(), Err(OntologyReadError::InvalidQuery));
    }
    query.limit = 1000;
    query.cursor = Some("ent_last".to_owned());
    assert_eq!(query.validate(), Ok(()));
    query.cursor = Some("arbitrary".to_owned());
    assert_eq!(query.validate(), Err(OntologyReadError::InvalidQuery));
    for filter in [
        PropertyFilter::Equals {
            property: " padded ".to_owned(),
            value: FilterValue::Integer(1),
        },
        PropertyFilter::Range {
            property: "score".to_owned(),
            from: FilterValue::Integer(2),
            to: FilterValue::Integer(1),
        },
        PropertyFilter::Range {
            property: "score".to_owned(),
            from: FilterValue::Integer(1),
            to: FilterValue::Text("2".to_owned()),
        },
    ] {
        assert_eq!(filter.validate(), Err(OntologyReadError::InvalidQuery));
    }
}

#[test]
fn refusals_have_distinct_serializable_names_and_safe_diagnostics() {
    let cases = [
        (OntologyReadError::Unauthenticated, "unauthenticated"),
        (OntologyReadError::Forbidden, "forbidden"),
        (OntologyReadError::InvalidQuery, "invalid_query"),
        (OntologyReadError::AbsentObject, "absent_object"),
        (OntologyReadError::RevisionConflict, "revision_conflict"),
        (OntologyReadError::FilterConflict, "filter_conflict"),
        (OntologyReadError::SourceUnavailable, "source_unavailable"),
    ];
    let mut diagnostics = std::collections::BTreeSet::new();
    for (error, name) in cases {
        assert_eq!(serde_json::to_value(error).unwrap(), name);
        assert_eq!(
            serde_json::from_value::<OntologyReadError>(json!(name)).unwrap(),
            error
        );
        assert!(diagnostics.insert(error.to_string()));
    }
}
