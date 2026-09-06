use dependency_declarations_reconcile::repository_manifests::{
    Limits, ManifestError, ManifestInput, plan,
};
use std::collections::{BTreeMap, BTreeSet};

const LIMITS: Limits = Limits {
    manifests: 32,
    bytes: 32_768,
    edges: 64,
};

fn input<'a>(directory: &'a str, contents: &'a str) -> ManifestInput<'a> {
    ManifestInput {
        directory,
        contents,
    }
}

#[test]
fn derives_every_manifest_dependency_category_without_expected_graph_input() {
    let workspace = "[workspace.dependencies]\nshared = { path = 'lib/shared' }\n";
    let manifests = [
        input(
            "app/main",
            "[dependencies]\nshared.workspace = true\n[dev-dependencies]\ntest = { path = '../../lib/test' }\n[build-dependencies]\nbuild = { path = '../../lib/build' }\n[target.'cfg(windows)'.dependencies]\nos = { path = '../../lib/os' }\n",
        ),
        input(
            "lib/shared",
            "[dependencies]\nleaf = { path = '../leaf' }\n",
        ),
        input("lib/test", ""),
        input("lib/build", ""),
        input("lib/os", ""),
        input("lib/leaf", ""),
    ];
    let actual = plan(workspace, &manifests, &["app/main"], LIMITS).unwrap();
    let expected = BTreeMap::from([
        (
            "app/main".into(),
            BTreeSet::from([
                "lib/shared".into(),
                "lib/test".into(),
                "lib/build".into(),
                "lib/os".into(),
            ]),
        ),
        ("lib/shared".into(), BTreeSet::from(["lib/leaf".into()])),
        ("lib/test".into(), BTreeSet::new()),
        ("lib/build".into(), BTreeSet::new()),
        ("lib/os".into(), BTreeSet::new()),
        ("lib/leaf".into(), BTreeSet::new()),
    ]);
    assert_eq!(actual.dependencies, expected);
}

#[test]
fn missing_manifest_and_repository_escape_are_typed_refusals() {
    let missing = [input(
        "app/main",
        "[dependencies]\nx = { path = '../../lib/missing' }\n",
    )];
    assert_eq!(
        plan("", &missing, &["app/main"], LIMITS),
        Err(ManifestError::MissingManifest("lib/missing".into()))
    );
    let escaping = [input(
        "app/main",
        "[dependencies]\nx = { path = '../../../outside' }\n",
    )];
    assert!(matches!(
        plan("", &escaping, &["app/main"], LIMITS),
        Err(ManifestError::InvalidPath(_))
    ));
}

#[test]
fn duplicate_inputs_are_not_silently_overwritten() {
    let inputs = [input("a", ""), input("a", "")];
    assert_eq!(
        plan("", &inputs, &["a"], LIMITS),
        Err(ManifestError::DuplicateManifest("a".into()))
    );
}

#[test]
fn new_transitive_edges_are_discovered_with_the_same_seeds() {
    let initial = [input("a", ""), input("b", "")];
    assert_eq!(
        plan("", &initial, &["a"], LIMITS)
            .unwrap()
            .dependencies
            .len(),
        1
    );
    let changed = [
        input("a", "[dev-dependencies]\nb = { path = '../b' }"),
        input("b", ""),
    ];
    assert_eq!(
        plan("", &changed, &["a"], LIMITS)
            .unwrap()
            .dependencies
            .len(),
        2
    );
}

#[test]
fn cycles_terminate_and_input_order_does_not_change_the_plan() {
    let first = input("a", "[dependencies]\nb = { path = '../b' }");
    let second = input("b", "[dependencies]\na = { path = '../a' }");
    let left = plan("", &[first, second], &["a", "b"], LIMITS).unwrap();
    let right = plan("", &[second, first], &["b", "a"], LIMITS).unwrap();
    assert_eq!(left, right);
    assert_eq!(left.dependencies.len(), 2);
    assert_ne!(left.input_digest, [0; 32]);
}

#[test]
fn workspace_resolution_and_unsupported_surfaces_fail_closed() {
    let inherited = [input("a", "[dependencies]\nmissing.workspace = true")];
    assert_eq!(
        plan("", &inherited, &["a"], LIMITS),
        Err(ManifestError::MissingWorkspaceDependency("missing".into()))
    );
    for workspace in [
        "[patch.crates-io]\nx = { path = 'a' }",
        "[replace]\n'x:1.0' = { path = 'a' }",
    ] {
        assert!(matches!(
            plan(workspace, &[input("a", "")], &["a"], LIMITS),
            Err(ManifestError::UnsupportedSurface(_))
        ));
    }
}

#[test]
fn manifest_bytes_nodes_and_edges_have_explicit_limits() {
    let inputs = [input("a", "[dependencies]\na = { path = '.' }")];
    for (limits, expected) in [
        (
            Limits {
                manifests: 0,
                ..LIMITS
            },
            "manifests",
        ),
        (Limits { bytes: 0, ..LIMITS }, "bytes"),
        (Limits { edges: 0, ..LIMITS }, "edges"),
    ] {
        assert_eq!(
            plan("", &inputs, &["a"], limits),
            Err(ManifestError::LimitExceeded(expected))
        );
    }
}

#[test]
fn noncanonical_manifest_directories_are_refused() {
    for directory in ["../a", "/a", "a/../b", "a//b", "a\\b", "C:/a"] {
        assert!(matches!(
            plan("", &[input(directory, "")], &[directory], LIMITS),
            Err(ManifestError::InvalidPath(_))
        ));
    }
}

#[test]
fn digest_binds_exact_workspace_bytes_not_only_parsed_values() {
    let inputs = [input("a", "")];
    let original = plan("[workspace]\n", &inputs, &["a"], LIMITS).unwrap();
    let changed = plan("# changed source\n[workspace]\n", &inputs, &["a"], LIMITS).unwrap();
    assert_ne!(original.input_digest, changed.input_digest);
}

#[test]
fn target_specific_build_and_dev_dependencies_are_included() {
    let inputs = [
        input(
            "a",
            "[target.'cfg(windows)'.dev-dependencies]\nb = { path = '../b', optional = true }\n[target.'cfg(unix)'.build-dependencies]\nc = { path = '../c' }",
        ),
        input("b", ""),
        input("c", ""),
    ];
    assert_eq!(
        plan("", &inputs, &["a"], LIMITS).unwrap().dependencies["a"],
        BTreeSet::from(["b".into(), "c".into()])
    );
}

#[test]
fn unsupported_dependency_aliases_and_nested_workspaces_are_refused() {
    for manifest in [
        "[dev_dependencies]\nb = { path = '../b' }",
        "[build_dependencies]\nb = { path = '../b' }",
        "[workspace]\n",
    ] {
        let inputs = [input("a", manifest), input("b", "")];
        assert!(matches!(
            plan("", &inputs, &["a"], LIMITS),
            Err(ManifestError::UnsupportedSurface(_))
        ));
    }
}

#[test]
fn malformed_dependency_values_are_not_treated_as_external_packages() {
    for manifest in [
        "[dependencies]\nb = 4",
        "[dependencies]\nb = { path = 4 }",
        "[target]\nx = 4",
    ] {
        assert!(matches!(
            plan("", &[input("a", manifest)], &["a"], LIMITS),
            Err(ManifestError::InvalidManifest(_))
        ));
    }
}

#[test]
fn explicit_workspace_redirects_cannot_select_unprovided_workspace_facts() {
    let redirected = [input("a", "[package]\nworkspace = '../other'\n")];
    assert!(matches!(
        plan("", &redirected, &["a"], LIMITS),
        Err(ManifestError::UnsupportedSurface(_))
    ));
    let root = [input("a", "[package]\nworkspace = '..'\n")];
    assert!(plan("", &root, &["a"], LIMITS).is_ok());
}
