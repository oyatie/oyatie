//! The conformance suite executed against the reference in-memory
//! store: the projection port's contract, given executable meaning.

use foundry_projection_draft::conformance::{
    ProjectionFixture, check_a_refused_apply_leaves_state_untouched,
    check_apply_requires_the_next_dense_ordinal, check_cross_kind_comparisons_fail_closed,
    check_divergent_reapply_is_refused, check_durability_across_reopen,
    check_equals_predicate_matches_exactly, check_get_returns_the_projected_object,
    check_identical_reapply_is_a_deduplicated_noop,
    check_poisoned_entries_advance_the_head_without_objects, check_range_kind_mismatch_is_refused,
    check_range_predicate_is_kind_scoped, check_reads_are_tenant_isolated,
    check_type_scan_pages_partition_deterministically,
};
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore};

#[derive(Default)]
struct MemoryFixture {
    store: MemoryProjectionStore,
}

impl ProjectionFixture for MemoryFixture {
    type Store = MemoryProjectionStore;

    fn store(&mut self) -> &mut Self::Store {
        &mut self.store
    }

    fn reopen(&mut self) -> bool {
        false
    }
}

fn run(check: fn(&mut MemoryFixture) -> Result<(), String>) {
    let mut fixture = MemoryFixture::default();
    if let Err(detail) = check(&mut fixture) {
        panic!("conformance refused: {detail}");
    }
}

#[test]
fn reference_store_passes_the_write_laws() {
    run(check_apply_requires_the_next_dense_ordinal);
    run(check_identical_reapply_is_a_deduplicated_noop);
    run(check_divergent_reapply_is_refused);
    run(check_a_refused_apply_leaves_state_untouched);
    run(check_poisoned_entries_advance_the_head_without_objects);
}

#[test]
fn reference_store_passes_the_read_laws() {
    run(check_get_returns_the_projected_object);
    run(check_reads_are_tenant_isolated);
    run(check_type_scan_pages_partition_deterministically);
}

#[test]
fn reference_store_passes_the_predicate_laws() {
    run(check_equals_predicate_matches_exactly);
    run(check_range_predicate_is_kind_scoped);
    run(check_range_kind_mismatch_is_refused);
    run(check_cross_kind_comparisons_fail_closed);
}

#[test]
fn reference_store_reports_volatile_durability_honestly() {
    run(check_durability_across_reopen);
}

#[test]
fn the_reference_store_is_a_projection_store() {
    fn assert_impl<S: ProjectionStore>() {}
    assert_impl::<MemoryProjectionStore>();
}
