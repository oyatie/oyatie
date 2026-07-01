use iam_policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTuple, RebacTuplePage,
    RebacTupleQuery, RebacTupleStore, RebacTupleStoreError, SnapshotToken, UsersetRewrite, Zookie,
};

#[test]
fn tuple_canonical_form_round_trips_object_relation_subject() {
    let tuple = RebacTuple::new(
        RebacObjectRef::new("document", "roadmap-q3").expect("object ref is valid"),
        RebacRelation::new("viewer").expect("relation is valid"),
        RebacSubjectRef::object(
            RebacObjectRef::new("user", "alice").expect("subject object is valid"),
        ),
    );

    assert_eq!(
        tuple.to_canonical_string(),
        "document:roadmap-q3#viewer@user:alice"
    );
    assert_eq!(
        RebacTuple::parse("document:roadmap-q3#viewer@user:alice").expect("canonical tuple parses"),
        tuple
    );
}

#[test]
fn tuple_subject_can_be_a_userset_rewrite_edge() {
    let tuple = RebacTuple::parse("document:roadmap-q3#viewer@group:platform#member")
        .expect("userset tuple parses");

    assert_eq!(tuple.object.object_type(), "document");
    assert_eq!(tuple.relation.as_str(), "viewer");
    assert_eq!(tuple.subject.to_canonical_string(), "group:platform#member");
    assert_eq!(
        tuple.to_canonical_string(),
        "document:roadmap-q3#viewer@group:platform#member"
    );
}

#[test]
fn zookie_and_snapshot_tokens_are_opaque_closed_serde_values() {
    let zookie = Zookie::new("zk-20260630-0001").expect("zookie token is valid");
    let snapshot = SnapshotToken::from_zookie(zookie.clone());

    assert_eq!(zookie.as_str(), "zk-20260630-0001");
    assert_eq!(snapshot.as_str(), "zk-20260630-0001");
    assert_eq!(
        serde_json::to_string(&snapshot).expect("snapshot serializes"),
        "\"zk-20260630-0001\""
    );
    assert_eq!(
        serde_json::from_str::<SnapshotToken>("\"zk-20260630-0001\"")
            .expect("snapshot deserializes"),
        snapshot
    );

    assert!(Zookie::new("contains whitespace").is_err());
    assert!(SnapshotToken::new("").is_err());
}

#[test]
fn userset_rewrite_tree_models_direct_computed_tuple_to_userset_and_difference() {
    let rewrite = UsersetRewrite::union(vec![
        UsersetRewrite::this(),
        UsersetRewrite::computed_userset(
            RebacRelation::new("owner").expect("owner relation is valid"),
        ),
        UsersetRewrite::tuple_to_userset(
            RebacRelation::new("parent").expect("parent relation is valid"),
            RebacRelation::new("viewer").expect("viewer relation is valid"),
        ),
        UsersetRewrite::difference(
            UsersetRewrite::computed_userset(
                RebacRelation::new("member").expect("member relation is valid"),
            ),
            UsersetRewrite::computed_userset(
                RebacRelation::new("suspended").expect("suspended relation is valid"),
            ),
        ),
    ])
    .expect("non-empty union is valid");

    assert!(rewrite.validate().is_ok());
    let json = serde_json::to_value(&rewrite).expect("rewrite serializes");
    assert_eq!(json["kind"], "union");
    assert_eq!(json["children"][0]["kind"], "this");
    assert_eq!(json["children"][2]["tupleset_relation"], "parent");
    assert_eq!(json["children"][2]["computed_userset_relation"], "viewer");

    let roundtrip: UsersetRewrite = serde_json::from_value(json).expect("rewrite deserializes");
    assert_eq!(roundtrip, rewrite);
    assert!(UsersetRewrite::union(Vec::new()).is_err());
}

#[test]
fn tuple_store_port_exposes_write_zookie_and_read_snapshot_contract() {
    #[derive(Default)]
    struct RecordingStore {
        tuples: Vec<RebacTuple>,
    }

    impl RebacTupleStore for RecordingStore {
        fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
            self.tuples.push(tuple);
            Zookie::new(format!("zk-{}", self.tuples.len()))
                .map_err(RebacTupleStoreError::InvalidZookie)
        }

        fn read_tuples(
            &self,
            query: &RebacTupleQuery,
            snapshot: RebacReadSnapshot,
        ) -> Result<RebacTuplePage, RebacTupleStoreError> {
            let tuples = self
                .tuples
                .iter()
                .filter(|tuple| query.matches(tuple))
                .cloned()
                .collect();
            Ok(RebacTuplePage {
                tuples,
                snapshot: snapshot.into_snapshot_token(),
                next_page_token: None,
            })
        }
    }

    let tuple = RebacTuple::parse("document:roadmap-q3#viewer@user:alice").expect("tuple parses");
    let mut store = RecordingStore::default();
    let zookie = store
        .write_tuple(tuple.clone())
        .expect("write returns zookie");
    let page = store
        .read_tuples(
            &RebacTupleQuery::object_relation(tuple.object.clone(), tuple.relation.clone()),
            RebacReadSnapshot::at_zookie(zookie.clone()),
        )
        .expect("read returns page");

    assert_eq!(page.tuples, vec![tuple]);
    assert_eq!(page.snapshot.as_str(), zookie.as_str());
}
