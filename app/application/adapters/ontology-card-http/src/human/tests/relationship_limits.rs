use application_ontology_read::{
    LinkSchema, MAX_RELATIONSHIP_NODES, MAX_RELATIONSHIP_RESPONSE_BYTES, OntologyReadSource,
    RelationshipGraph, RelationshipQuery,
};
use serde_json::json;

use super::support::{cookie, source, stub};

fn long_id(prefix: &str, ordinal: usize) -> String {
    let mut value = format!("{prefix}{ordinal:04}_");
    value.extend(std::iter::repeat_n('\\', 256 - value.len()));
    value
}

#[tokio::test]
async fn maximal_one_hop_with_escaped_long_ids_fits_the_relationship_response_bound() {
    let root = long_id("ent_", 0);
    let from = long_id("ety_", 0);
    let to = long_id("ety_", 1);
    let link = long_id("lty_", 0);
    let query = RelationshipQuery::decode(&serde_json::to_vec(&json!({
        "idempotency_key":"maximal-escaped", "root":{"entity_type":from,"object_ref":root,"revision":1},
        "link_type":link, "direction":"outbound", "observed_at_epoch_seconds":123
    })).unwrap()).unwrap();
    let declaration = json!({"link_type":link,"from_entity_type":from,"to_entity_type":to,
        "cardinality":"one_to_many","allow_cross_tenant":false,"revision":1});
    let declared: LinkSchema = serde_json::from_value(declaration.clone()).unwrap();
    let nodes: Vec<_> = (0..MAX_RELATIONSHIP_NODES).map(|ordinal| json!({
        "entity_id":long_id("ent_",ordinal),"entity_type_id":if ordinal==0 {&from} else {&to},
        "written_revision":if ordinal==0 {1} else {7}
    })).collect();
    let edges: Vec<_> = (1..MAX_RELATIONSHIP_NODES)
        .map(|ordinal| {
            json!({
                "from_entity_id":root,"to_entity_id":long_id("ent_",ordinal),"edge_type_id":link
            })
        })
        .collect();
    let graph =
        json!({"nodes":nodes,"edges":edges,"observed_at_epoch_seconds":123,"truncated":false});
    let bytes = serde_json::to_vec(&graph).unwrap();
    assert!(
        bytes.len() > 1024 * 1024,
        "legal escaped output exceeds the old generic read limit"
    );
    assert!(bytes.len() <= MAX_RELATIONSHIP_RESPONSE_BYTES);
    let expected = RelationshipGraph::decode(&bytes, &query, &declared).unwrap();
    let (base, _) = stub(vec![
        (200, json!([declaration]).to_string(), String::new()),
        (
            200,
            String::from_utf8(bytes.clone()).unwrap(),
            String::new(),
        ),
    ]);
    let result = source(&base, &cookie('A'))
        .search_around("ten_acme", &query)
        .await;
    assert!(
        result.is_ok(),
        "legal maximal graph refused: {:?}",
        result.as_ref().err()
    );
    assert_eq!(result.unwrap(), expected);
    println!(
        "maximal_relationship_nodes={} edges={} escaped_json_bytes={}",
        MAX_RELATIONSHIP_NODES,
        MAX_RELATIONSHIP_NODES - 1,
        bytes.len()
    );
}
