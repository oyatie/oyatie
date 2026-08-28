use dependency_declarations_reconcile::{
    GeneratedArtifactObservationV1, GenerationDeterminismErrorV1, ProducerRuleGraphV1,
    compare_generation_runs,
};

#[derive(Debug)]
struct GeneratorOwnedGraph {
    sort_key: &'static str,
    full_field_bytes: Box<[u8]>,
}

impl PartialEq for GeneratorOwnedGraph {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key == other.sort_key
    }
}

impl ProducerRuleGraphV1 for GeneratorOwnedGraph {
    fn canonical_full_field_bytes(&self) -> &[u8] {
        &self.full_field_bytes
    }
}

fn artifact(
    sort_key: &'static str,
    full_field_bytes: &'static [u8],
    rendered_buck: &'static [u8],
) -> GeneratedArtifactObservationV1<GeneratorOwnedGraph> {
    GeneratedArtifactObservationV1::new(
        GeneratorOwnedGraph {
            sort_key,
            full_field_bytes: full_field_bytes.into(),
        },
        rendered_buck,
    )
}

#[test]
fn identical_bytes_and_full_graph_produce_one_two_run_proof() {
    let left = artifact("crate", b"owner=one;attributes=[dep]", b"rust_library()\n");
    let right = artifact("crate", b"owner=one;attributes=[dep]", b"rust_library()\n");

    let proof = compare_generation_runs(&left, &right).expect("identical runs must qualify");

    assert_eq!(proof.rendered_buck_length_bytes(), 15);
    assert_eq!(proof.producer_graph_length_bytes(), 26);
    assert_ne!(proof.rendered_buck_sha256(), proof.producer_graph_sha256());
}

#[test]
fn rendered_byte_difference_refuses_even_when_graphs_are_equal() {
    let left = artifact("crate", b"owner=one", b"rust_library()\n");
    let right = artifact("crate", b"owner=one", b"rust_binary()\n");

    assert_eq!(
        compare_generation_runs(&left, &right),
        Err(GenerationDeterminismErrorV1::RenderedBuckMismatch)
    );
}

#[test]
fn full_graph_difference_refuses_when_sort_key_equality_would_accept() {
    let left = artifact("same-sort-key", b"visibility=private", b"alias()\n");
    let right = artifact("same-sort-key", b"visibility=public", b"alias()\n");
    assert_eq!(
        left.graph(),
        right.graph(),
        "fixture must mimic Rule::PartialEq"
    );

    assert_eq!(
        compare_generation_runs(&left, &right),
        Err(GenerationDeterminismErrorV1::ProducerGraphMismatch)
    );
}
