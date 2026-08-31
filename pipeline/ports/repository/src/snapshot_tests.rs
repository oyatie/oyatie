use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ContentId, Entry, EntryKind, EntryState, HydratedSnapshot, ObjectId, PreparedSnapshot,
    ProducerId, ProfileId, RepositoryId, RepositoryManifest, RepositoryPath, ResolvedRevision,
    RevisionId, SchemaId, SnapshotFailure, SnapshotLimitSpec, SnapshotLimits, SnapshotProfile,
    SnapshotRequest, ToolId, TreeId,
};

fn limits(max_selected_contents: u64) -> SnapshotLimits {
    SnapshotLimits::new(SnapshotLimitSpec {
        max_entries: 100,
        max_path_bytes: 100,
        max_manifest_bytes: 10_000,
        max_selected_contents,
        max_content_bytes: 1_000,
        max_total_content_bytes: 10_000,
        max_stdout_bytes: 20_000,
        max_stderr_bytes: 1_000,
    })
    .unwrap()
}

fn prepared(tool: &str, limits: SnapshotLimits) -> PreparedSnapshot {
    prepared_with(
        "test-profile-v1",
        "repository-snapshot-v1",
        "test-producer-v1",
        tool,
        limits,
    )
}

fn prepared_with(
    profile: &str,
    schema: &str,
    producer: &str,
    tool: &str,
    limits: SnapshotLimits,
) -> PreparedSnapshot {
    let base = RevisionId::from_hex(&"1".repeat(40)).unwrap();
    let head = RevisionId::from_hex(&"2".repeat(40)).unwrap();
    let base_tree = TreeId::from_hex(&"3".repeat(40)).unwrap();
    let head_tree = TreeId::from_hex(&"4".repeat(40)).unwrap();
    let content = ObjectId::from_hex(&"5".repeat(40)).unwrap();
    let profile = SnapshotProfile::new(
        ProfileId::new(profile).unwrap(),
        SchemaId::new(schema).unwrap(),
        limits,
    );
    let request = SnapshotRequest::new(
        RepositoryId::new("test/repository").unwrap(),
        base,
        head,
        profile,
    )
    .unwrap();
    let resolved_base = ResolvedRevision::new(base, base, base_tree).unwrap();
    let merge = RepositoryManifest::new(resolved_base, Vec::new(), limits).unwrap();
    let resolved_head = ResolvedRevision::new(head, head, head_tree).unwrap();
    let head = RepositoryManifest::new(
        resolved_head,
        vec![Entry::new(
            RepositoryPath::from_utf8("one").unwrap(),
            EntryState::new(EntryKind::Blob, content),
        )],
        limits,
    )
    .unwrap();
    PreparedSnapshot::new(
        request,
        resolved_base,
        merge,
        head,
        ProducerId::new(producer).unwrap(),
        ToolId::new(tool).unwrap(),
    )
    .unwrap()
}

fn completed(prepared: PreparedSnapshot) -> HydratedSnapshot {
    let content = prepared.head().entries()[0].content_id().unwrap();
    let selection = prepared.select_content(BTreeSet::from([content])).unwrap();
    HydratedSnapshot::complete(
        prepared,
        selection,
        BTreeMap::from([(content, b"content".to_vec())]),
    )
    .unwrap()
}

#[test]
fn receipt_digest_binds_producer_tool_profile_schema_limits_and_inputs() {
    let first = completed(prepared("tool-a", limits(2)));
    let changed_tool = completed(prepared("tool-b", limits(2)));
    let changed_limits = completed(prepared("tool-a", limits(3)));
    let changed_profile = completed(prepared_with(
        "test-profile-v2",
        "repository-snapshot-v1",
        "test-producer-v1",
        "tool-a",
        limits(2),
    ));
    let changed_schema = completed(prepared_with(
        "test-profile-v1",
        "repository-snapshot-v2",
        "test-producer-v1",
        "tool-a",
        limits(2),
    ));
    let changed_producer = completed(prepared_with(
        "test-profile-v1",
        "repository-snapshot-v1",
        "test-producer-v2",
        "tool-a",
        limits(2),
    ));

    assert_ne!(first.receipt().digest(), changed_tool.receipt().digest());
    assert_ne!(first.receipt().digest(), changed_limits.receipt().digest());
    assert_ne!(first.receipt().digest(), changed_profile.receipt().digest());
    assert_ne!(first.receipt().digest(), changed_schema.receipt().digest());
    assert_ne!(
        first.receipt().digest(),
        changed_producer.receipt().digest()
    );
    assert_eq!(first.receipt().schema().as_str(), "repository-snapshot-v1");
    assert_eq!(first.receipt().repository().as_str(), "test/repository");
}

#[test]
fn hydration_requires_exactly_the_selected_key_set() {
    let prepared = prepared("tool-a", limits(2));
    let content = prepared.head().entries()[0].content_id().unwrap();
    let selection = prepared.select_content(BTreeSet::from([content])).unwrap();
    assert!(matches!(
        HydratedSnapshot::complete(prepared, selection, BTreeMap::new()),
        Err(SnapshotFailure::MissingContent(_))
    ));
}

#[test]
fn selection_count_is_bounded_before_adapter_work() {
    let prepared = prepared("tool-a", limits(1));
    let available = prepared.head().entries()[0].content_id().unwrap();
    let other = ContentId::from_object_id(ObjectId::from_hex(&"6".repeat(40)).unwrap());
    assert!(matches!(
        prepared.select_content(BTreeSet::from([available, other])),
        Err(SnapshotFailure::LimitExceeded {
            limit: "selected content count",
            ..
        })
    ));
}
