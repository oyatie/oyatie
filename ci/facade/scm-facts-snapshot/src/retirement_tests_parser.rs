//! Parser and fail-closed JSON tests.

use super::test_fixtures::*;
use super::*;
use serde_json::Value;

#[test]
fn duplicate_json_keys_fail_closed() {
    let error = parse_closed_json::<Value>(br#"{"a":1,"a":2}"#).unwrap_err();
    assert!(error.contains("duplicate object key"));
}

#[test]
fn executable_symlink_and_submodule_targets_fail_closed() {
    for (mode, kind) in [("100755", "blob"), ("120000", "blob"), ("160000", "commit")] {
        let mut source = fixture();
        source.trees.get_mut(PREDECESSOR).unwrap()[0].mode = mode.to_owned();
        source.trees.get_mut(PREDECESSOR).unwrap()[0].kind = kind.to_owned();
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("100644 blob"));
    }
}

#[test]
fn ls_tree_parser_preserves_mode_kind_oid_and_path() {
    let bytes = format!("100755 blob {}\tbin/tool\0", oid(7));
    assert_eq!(
        parse_ls_tree(bytes.as_bytes()).unwrap(),
        vec![TreeEntry {
            mode: "100755".to_owned(),
            kind: "blob".to_owned(),
            oid: oid(7),
            path: "bin/tool".to_owned()
        }]
    );
}
