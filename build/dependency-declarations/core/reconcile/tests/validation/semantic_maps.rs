use dependency_declarations_reconcile::{FailureClassV1, SemanticValueV1};

#[test]
fn semantic_maps_are_canonical_and_duplicate_free() {
    let entry = |key: &str, value: &str| {
        (
            SemanticValueV1::string(key).unwrap(),
            SemanticValueV1::string(value).unwrap(),
        )
    };
    assert_eq!(
        SemanticValueV1::map(vec![entry("b", "second"), entry("a", "first")]).unwrap(),
        SemanticValueV1::map(vec![entry("a", "first"), entry("b", "second")]).unwrap()
    );
    assert_eq!(
        SemanticValueV1::map(vec![entry("a", "first"), entry("a", "second")])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );
}
