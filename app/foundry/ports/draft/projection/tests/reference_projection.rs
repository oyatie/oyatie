//! The conformance suite executed against the reference in-memory
//! store: the projection port's contract, given executable meaning.

use foundry_projection_draft::conformance::{
    ProjectionFixture, check_a_composite_key_value_is_refused,
    check_a_duplicate_primary_key_is_refused, check_a_later_observation_updates_the_edge,
    check_a_missing_key_property_is_refused, check_a_refused_apply_leaves_state_untouched,
    check_a_refused_apply_writes_no_links, check_an_object_may_keep_its_own_key,
    check_an_undeclared_key_constrains_nothing, check_apply_requires_the_next_dense_ordinal,
    check_cross_kind_comparisons_fail_closed, check_divergent_reapply_is_refused,
    check_durability_across_reopen, check_equals_predicate_matches_exactly,
    check_get_returns_the_projected_object, check_identical_reapply_is_a_deduplicated_noop,
    check_keys_are_scoped_to_their_entity_type_and_tenant, check_links_are_tenant_isolated,
    check_links_round_trip_in_both_directions,
    check_poisoned_entries_advance_the_head_without_objects, check_range_kind_mismatch_is_refused,
    check_range_predicate_is_kind_scoped, check_re_applying_an_entry_does_not_duplicate_links,
    check_reads_are_tenant_isolated, check_two_objects_in_one_entry_cannot_share_a_key,
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

#[test]
fn reference_store_passes_the_primary_key_laws() {
    run(check_a_duplicate_primary_key_is_refused);
    run(check_two_objects_in_one_entry_cannot_share_a_key);
    run(check_an_object_may_keep_its_own_key);
    run(check_keys_are_scoped_to_their_entity_type_and_tenant);
    run(check_an_undeclared_key_constrains_nothing);
    run(check_a_missing_key_property_is_refused);
    run(check_a_composite_key_value_is_refused);
}

#[test]
fn reference_store_passes_the_link_laws() {
    run(check_links_round_trip_in_both_directions);
    run(check_links_are_tenant_isolated);
    run(check_a_refused_apply_writes_no_links);
    run(check_re_applying_an_entry_does_not_duplicate_links);
    run(check_a_later_observation_updates_the_edge);
}
