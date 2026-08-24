use super::*;

#[test]
fn draft_dependencies_cannot_cross_owner_boundaries() {
    let path = "network/core/route/Cargo.toml";
    let workspace = "[workspace.dependencies]\nshared={path='storage/ports/draft/blob'}\n";
    let direct = "[package]\nname='network-route'\n[dependencies]\nblob={path='../../../storage/ports/draft/blob'}\n";
    assert!(!draft_dependency_violations(path, direct, workspace).is_empty());

    let inherited = "[package]\nname='network-route'\n[target.'cfg(unix)'.dev-dependencies]\nshared.workspace=true\n";
    assert!(!draft_dependency_violations(path, inherited, workspace).is_empty());

    let local = "[package]\nname='network-route'\n[dependencies]\nrepo={path='../../ports/draft/repository'}\n";
    assert!(draft_dependency_violations(path, local, workspace).is_empty());

    assert!(!workspace_draft_dependency_violations(workspace).is_empty());
    assert!(
        workspace_draft_dependency_violations(
            "[workspace.dependencies]\nshared={path='storage/ports/blob'}\n"
        )
        .is_empty()
    );
    for override_manifest in [
        "[patch.crates-io]\nshared={path='storage/ports/draft/blob'}\n",
        "[replace]\n'shared:1.0.0'={path='storage/ports/draft/blob'}\n",
    ] {
        assert!(!workspace_draft_dependency_violations(override_manifest).is_empty());
    }
}

#[test]
fn meta_root_manifests_cannot_consume_an_owner_draft() {
    let path = "build/port-engine/core/analysis/Cargo.toml";
    let manifest = "[package]\nname='port-analysis'\n[dependencies]\nrepo={path='../../../../pipeline/ports/draft/repository'}\n";
    let violations = draft_dependency_violations(path, manifest, "");
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("unclassified manifest")),
        "{violations:#?}"
    );
}

#[test]
fn unsafe_dependency_paths_fail_closed() {
    let owner_manifest = "network/core/route/Cargo.toml";
    for dependency_path in [
        "/tmp/storage/ports/draft/blob",
        "C:/workspace/storage/ports/draft/blob",
        r"..\..\..\storage\ports\draft\blob",
        "../../../../storage/ports/draft/blob",
    ] {
        let manifest = format!(
            "[package]\nname='network-route'\n[dependencies]\nblob={{path='{dependency_path}'}}\n"
        );
        assert!(
            !draft_dependency_violations(owner_manifest, &manifest, "").is_empty(),
            "expected invalid dependency path rejection: {dependency_path}"
        );
    }

    for dependency_path in [
        "/tmp/storage/ports/draft/blob",
        "C:/workspace/storage/ports/draft/blob",
        r"storage\ports\draft\blob",
        "../storage/ports/draft/blob",
    ] {
        let workspace = format!("[workspace.dependencies]\nshared={{path='{dependency_path}'}}\n");
        assert!(
            !workspace_draft_dependency_violations(&workspace).is_empty(),
            "expected invalid root dependency path rejection: {dependency_path}"
        );
    }
}
