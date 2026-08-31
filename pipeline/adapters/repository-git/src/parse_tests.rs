use std::collections::BTreeSet;

use pipeline_repository::{
    ContentId, EntryKind, ObjectAlgorithm, ObjectId, SnapshotFailure, SnapshotLimitSpec,
    SnapshotLimits, TreeId,
};
use sha1_checked::{Digest as _, Sha1};

use crate::object::verify_tree_identities;
use crate::parse::{parse_batch_contents, parse_ls_tree, parse_merge_base, parse_resolved_objects};

fn limits() -> SnapshotLimits {
    SnapshotLimits::new(limit_spec()).unwrap()
}

fn limit_spec() -> SnapshotLimitSpec {
    SnapshotLimitSpec {
        max_entries: 100,
        max_path_bytes: 100,
        max_manifest_bytes: 10_000,
        max_selected_contents: 100,
        max_content_bytes: 100,
        max_total_content_bytes: 1_000,
        max_stdout_bytes: 2_000,
        max_stderr_bytes: 100,
    }
}

#[test]
fn tree_parser_preserves_non_utf8_paths_and_modes() {
    let output = b"100755 blob 1111111111111111111111111111111111111111\tbin/hi-\xff\0\
                   120000 blob 2222222222222222222222222222222222222222\tlink\0";
    let entries = parse_ls_tree(output, ObjectAlgorithm::Sha1, limits()).unwrap();

    assert_eq!(entries[0].kind(), EntryKind::ExecutableBlob);
    assert_eq!(entries[0].path().as_bytes(), b"bin/hi-\xff");
    assert_eq!(entries[1].kind(), EntryKind::Symlink);
}

#[test]
fn tree_parser_refuses_truncation_and_mode_confusion() {
    assert!(matches!(
        parse_ls_tree(
            b"100644 blob 1111111111111111111111111111111111111111\tfile",
            ObjectAlgorithm::Sha1,
            limits(),
        ),
        Err(SnapshotFailure::MalformedOutput(_))
    ));
    assert!(matches!(
        parse_ls_tree(
            b"160000 blob 1111111111111111111111111111111111111111\tmodule\0",
            ObjectAlgorithm::Sha1,
            limits(),
        ),
        Err(SnapshotFailure::ObjectMismatch(_))
    ));
}

#[test]
fn merge_base_parser_refuses_missing_ambiguous_and_partial_results() {
    assert!(matches!(
        parse_merge_base(b"", ObjectAlgorithm::Sha1),
        Err(SnapshotFailure::MissingMergeBase)
    ));
    assert!(matches!(
        parse_merge_base(
            b"1111111111111111111111111111111111111111\n2222222222222222222222222222222222222222\n",
            ObjectAlgorithm::Sha1,
        ),
        Err(SnapshotFailure::AmbiguousMergeBase { count: 2 })
    ));
    assert!(matches!(
        parse_merge_base(
            b"1111111111111111111111111111111111111111",
            ObjectAlgorithm::Sha1,
        ),
        Err(SnapshotFailure::MalformedOutput(_))
    ));
}

#[test]
fn content_parser_refuses_trailing_and_partial_batches() {
    let id = ContentId::from_object_id(
        ObjectId::from_hex("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap(),
    );
    let selection = BTreeSet::from([id]);
    let valid = b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 blob 0\n\n";
    assert_eq!(
        parse_batch_contents(valid, &selection, limits())
            .unwrap()
            .get(&id)
            .unwrap(),
        b""
    );
    assert!(
        parse_batch_contents(&[valid.as_slice(), b"extra"].concat(), &selection, limits()).is_err()
    );
    assert!(parse_batch_contents(&valid[..valid.len() - 1], &selection, limits()).is_err());

    let forged = b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 blob 3\nabc\n";
    assert!(matches!(
        parse_batch_contents(forged, &selection, limits()),
        Err(SnapshotFailure::ObjectMismatch(_))
    ));

    let missing = b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 missing\n";
    assert!(matches!(
        parse_batch_contents(missing, &selection, limits()),
        Err(SnapshotFailure::MissingContent(_))
    ));
}

#[test]
fn every_protocol_byte_boundary_refuses_truncation() {
    let merge_base = b"1111111111111111111111111111111111111111\n";
    for end in 0..merge_base.len() {
        assert!(parse_merge_base(&merge_base[..end], ObjectAlgorithm::Sha1).is_err());
    }

    let resolved = b"1111111111111111111111111111111111111111 commit\n\
                     2222222222222222222222222222222222222222 tree\n";
    for end in 0..resolved.len() {
        assert!(parse_resolved_objects(&resolved[..end], ObjectAlgorithm::Sha1, 2).is_err());
    }

    let empty = ObjectId::from_hex("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap();
    let tree_body = [tree_record(b"a", empty), tree_record(b"b", empty)].concat();
    let tree = TreeId::from_hex(&git_object_hex(b"tree", &tree_body)).unwrap();
    let tree_output = b"100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\ta\0\
                        100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\tb\0";
    let entries = parse_ls_tree(tree_output, ObjectAlgorithm::Sha1, limits()).unwrap();
    verify_tree_identities(tree, &entries).unwrap();
    for end in 0..tree_output.len() {
        if let Ok(entries) = parse_ls_tree(&tree_output[..end], ObjectAlgorithm::Sha1, limits()) {
            assert!(verify_tree_identities(tree, &entries).is_err());
        }
    }

    let contents = [
        (empty, b"".as_slice()),
        (git_blob_id(b"x"), b"x".as_slice()),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    let selection = contents
        .keys()
        .copied()
        .map(ContentId::from_object_id)
        .collect();
    let mut batch = Vec::new();
    for (id, bytes) in &contents {
        batch.extend_from_slice(format!("{id} blob {}\n", bytes.len()).as_bytes());
        batch.extend_from_slice(bytes);
        batch.push(b'\n');
    }
    parse_batch_contents(&batch, &selection, limits()).unwrap();
    for end in 0..batch.len() {
        assert!(parse_batch_contents(&batch[..end], &selection, limits()).is_err());
    }
}

fn tree_record(name: &[u8], object: ObjectId) -> Vec<u8> {
    let mut record = b"100644 ".to_vec();
    record.extend_from_slice(name);
    record.push(0);
    record.extend_from_slice(object.as_bytes());
    record
}

fn git_blob_id(bytes: &[u8]) -> ObjectId {
    ObjectId::from_hex(&git_object_hex(b"blob", bytes)).unwrap()
}

fn git_object_hex(kind: &[u8], bytes: &[u8]) -> String {
    let mut digest = Sha1::new();
    digest.update(kind);
    digest.update(b" ");
    digest.update(bytes.len().to_string().as_bytes());
    digest.update(b"\0");
    digest.update(bytes);
    let result = digest.try_finalize();
    assert!(!result.has_collision(), "test fixture must not collide");
    result
        .hash()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn content_parser_refuses_per_object_and_aggregate_byte_overruns() {
    let empty = ContentId::from_object_id(
        ObjectId::from_hex("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap(),
    );
    let selection = BTreeSet::from([empty]);
    let oversized = b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 blob 3\nabc\n";
    let per_object = SnapshotLimits::new(SnapshotLimitSpec {
        max_content_bytes: 2,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        parse_batch_contents(oversized, &selection, per_object),
        Err(SnapshotFailure::LimitExceeded {
            limit: "content bytes",
            ..
        })
    ));

    let first = ContentId::from_object_id(
        ObjectId::from_hex("2e65efe2a145dda7ee51d1741299f848e5bf752e").unwrap(),
    );
    let second = ContentId::from_object_id(
        ObjectId::from_hex("63d8dbd40c23542e740659a7168a0ce3138ea748").unwrap(),
    );
    let selection = BTreeSet::from([first, second]);
    let output = b"2e65efe2a145dda7ee51d1741299f848e5bf752e blob 1\na\n\
                   63d8dbd40c23542e740659a7168a0ce3138ea748 blob 1\nb\n";
    let aggregate = SnapshotLimits::new(SnapshotLimitSpec {
        max_content_bytes: 1,
        max_total_content_bytes: 1,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        parse_batch_contents(output, &selection, aggregate),
        Err(SnapshotFailure::LimitExceeded {
            limit: "total content bytes",
            ..
        })
    ));
}

#[test]
fn tree_parser_refuses_semantic_limits_before_manifest_construction() {
    let invalid_excess = b"100644 blob 1111111111111111111111111111111111111111\ta\0broken\0";
    let one_entry = SnapshotLimits::new(SnapshotLimitSpec {
        max_entries: 1,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        parse_ls_tree(invalid_excess, ObjectAlgorithm::Sha1, one_entry),
        Err(SnapshotFailure::LimitExceeded {
            limit: "entry count",
            maximum: 1,
            observed: 2,
        })
    ));

    let long_path = b"100644 blob 1111111111111111111111111111111111111111\tab\0";
    let one_path_byte = SnapshotLimits::new(SnapshotLimitSpec {
        max_path_bytes: 1,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        parse_ls_tree(long_path, ObjectAlgorithm::Sha1, one_path_byte),
        Err(SnapshotFailure::LimitExceeded {
            limit: "path bytes",
            maximum: 1,
            observed: 2,
        })
    ));

    let one_record = b"100644 blob 1111111111111111111111111111111111111111\ta\0";
    let short_manifest = SnapshotLimits::new(SnapshotLimitSpec {
        max_manifest_bytes: 28,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        parse_ls_tree(one_record, ObjectAlgorithm::Sha1, short_manifest),
        Err(SnapshotFailure::LimitExceeded {
            limit: "manifest bytes",
            maximum: 28,
            observed: 29,
        })
    ));
}
