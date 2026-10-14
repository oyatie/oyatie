use crate::*;
use serde_json::json;

fn query() -> RelationshipQuery {
    RelationshipQuery::decode(br#"{"idempotency_key":"attempt-1","root":{"entity_type":"ety_record","object_ref":"ent_alpha","revision":1},"link_type":"lty_related","direction":"outbound","observed_at_epoch_seconds":123}"#).unwrap()
}
fn link() -> LinkSchema {
    LinkSchema {
        link_type: "lty_related".into(),
        from_entity_type: "ety_record".into(),
        to_entity_type: "ety_profile".into(),
        cardinality: LinkCardinality::OneToMany,
        allow_cross_tenant: false,
        revision: 1,
    }
}
fn wire() -> serde_json::Value {
    json!({"nodes":[{"entity_id":"ent_alpha","entity_type_id":"ety_record","written_revision":1},{"entity_id":"ent_beta","entity_type_id":"ety_profile","written_revision":7}],"edges":[{"from_entity_id":"ent_alpha","to_entity_id":"ent_beta","edge_type_id":"lty_related"}],"observed_at_epoch_seconds":123,"truncated":false})
}

#[test]
fn bounded_query_refuses_unknown_duplicate_positional_and_untrusted_fields() {
    let request = serde_json::to_value(query()).unwrap();
    let mut invalid = vec![json!([])];
    for (field, value) in [
        ("idempotency_key", json!(" bad")),
        ("idempotency_key", json!("x".repeat(129))),
        ("direction", json!("both")),
        (
            "observed_at_epoch_seconds",
            json!(9_007_199_254_740_992_u64),
        ),
        ("tenant", json!("ten_other")),
        ("consent", json!("unrestricted")),
        ("max_depth", json!(2)),
    ] {
        let mut row = request.clone();
        row[field] = value;
        invalid.push(row);
    }
    for (field, value) in [
        ("entity_type", json!("ety_")),
        ("object_ref", json!("ent_bad\n")),
        ("object_ref", json!(format!("ent_{}", "x".repeat(253)))),
        ("principal", json!("usr_other")),
        ("revision", json!(-1)),
    ] {
        let mut row = request.clone();
        row["root"][field] = value;
        invalid.push(row);
    }
    let mut positional = request.clone();
    positional["root"] = json!(["ety_record", "ent_alpha", 1]);
    invalid.push(positional);
    for row in invalid {
        assert_eq!(
            RelationshipQuery::decode(&serde_json::to_vec(&row).unwrap()),
            Err(OntologyReadError::InvalidQuery)
        );
    }
    let duplicate = serde_json::to_string(&request)
        .unwrap()
        .replace("\"revision\":1", "\"revision\":1,\"revision\":1");
    assert_eq!(
        RelationshipQuery::decode(duplicate.as_bytes()),
        Err(OntologyReadError::InvalidQuery)
    );
    assert_eq!(
        RelationshipQuery::decode(&vec![b' '; MAX_RELATIONSHIP_REQUEST_BYTES + 1]),
        Err(OntologyReadError::InvalidQuery)
    );
    assert_eq!(query().validate_for(&link()), Ok(()));
    let mut wrong = query();
    wrong.direction = RelationshipDirection::Inbound;
    assert_eq!(
        wrong.validate_for(&link()),
        Err(OntologyReadError::InvalidQuery)
    );
}

#[test]
fn graph_preserves_independent_pins_empty_root_and_explicit_truncation() {
    let mut row = wire();
    let graph =
        RelationshipGraph::decode(&serde_json::to_vec(&row).unwrap(), &query(), &link()).unwrap();
    assert_eq!(graph.nodes[1].written_revision, 7);
    row["truncated"] = json!(true);
    assert!(
        RelationshipGraph::decode(&serde_json::to_vec(&row).unwrap(), &query(), &link())
            .unwrap()
            .truncated
    );
    row["nodes"].as_array_mut().unwrap().truncate(1);
    row["edges"] = json!([]);
    assert_eq!(
        RelationshipGraph::decode(&serde_json::to_vec(&row).unwrap(), &query(), &link())
            .unwrap()
            .nodes
            .len(),
        1
    );
    let mut inbound = query();
    inbound.direction = RelationshipDirection::Inbound;
    inbound.root.entity_type = "ety_profile".into();
    inbound.root.object_ref = "ent_beta".into();
    assert!(
        RelationshipGraph::decode(&serde_json::to_vec(&wire()).unwrap(), &inbound, &link()).is_ok()
    );
}

#[test]
fn graphs_refuse_ambiguous_missing_detached_unrooted_and_wrong_endpoint_rows() {
    let mut invalid = vec![json!([])];
    for (field, value) in [
        ("nodes", json!([])),
        ("edges", json!([])),
        ("observed_at_epoch_seconds", json!(124)),
        ("extra", json!(true)),
        ("truncated", json!(null)),
    ] {
        let mut row = wire();
        row[field] = value;
        invalid.push(row);
    }
    for (field, value) in [
        ("entity_id", json!("ent_missing")),
        ("entity_type_id", json!("ety_record")),
        ("written_revision", json!(-1)),
        ("extra", json!(1)),
    ] {
        let mut row = wire();
        row["nodes"][1][field] = value;
        invalid.push(row);
    }
    for (field, value) in [
        ("from_entity_id", json!("ent_beta")),
        ("to_entity_id", json!("ent_missing")),
        ("edge_type_id", json!("lty_other")),
    ] {
        let mut row = wire();
        row["edges"][0][field] = value;
        invalid.push(row);
    }
    let mut duplicate = wire();
    let node = duplicate["nodes"][0].clone();
    duplicate["nodes"].as_array_mut().unwrap().push(node);
    invalid.push(duplicate);
    let mut duplicate = wire();
    let edge = duplicate["edges"][0].clone();
    duplicate["edges"].as_array_mut().unwrap().push(edge);
    invalid.push(duplicate);
    let mut positional = wire();
    positional["nodes"][0] = json!(["ent_alpha", "ety_record", 1]);
    invalid.push(positional);
    let mut too_many = wire();
    too_many["nodes"] = json!(vec![wire()["nodes"][0].clone(); MAX_RELATIONSHIP_NODES + 1]);
    invalid.push(too_many);
    let mut too_many = wire();
    too_many["edges"] = json!(vec![wire()["edges"][0].clone(); MAX_RELATIONSHIP_EDGES + 1]);
    invalid.push(too_many);
    for row in invalid {
        assert_eq!(
            RelationshipGraph::decode(&serde_json::to_vec(&row).unwrap(), &query(), &link()),
            Err(OntologyReadError::SourceUnavailable)
        );
    }
    let duplicate = wire().to_string().replace(
        "\"written_revision\":1",
        "\"written_revision\":1,\"written_revision\":1",
    );
    assert_eq!(
        RelationshipGraph::decode(duplicate.as_bytes(), &query(), &link()),
        Err(OntologyReadError::SourceUnavailable)
    );
}

#[test]
fn relationship_response_bytes_are_bounded_before_decoding() {
    assert_eq!(
        RelationshipGraph::decode(
            &vec![b' '; MAX_RELATIONSHIP_RESPONSE_BYTES + 1],
            &query(),
            &link(),
        ),
        Err(OntologyReadError::SourceUnavailable)
    );
}
