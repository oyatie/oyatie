//! Protected workflow binds the native freeze but no migration transport yet.

use pipeline_admission::{changed_layout_violations, git_change_paths_from_name_status_z};

fn read_declared(name: &str, repository_relative: &str) -> String {
    let path = std::env::var_os(name)
        .map(std::path::PathBuf::from)
        .or_else(|| {
            option_env!("CARGO_MANIFEST_DIR").map(|manifest| {
                std::path::Path::new(manifest)
                    .join("../../..")
                    .join(repository_relative)
            })
        })
        .unwrap_or_else(|| panic!("FAIL-CLOSED: declared source binding {name} is unset"));
    let metadata = std::fs::symlink_metadata(&path)
        .unwrap_or_else(|error| panic!("FAIL-CLOSED: inspect {name}={}: {error}", path.display()));
    assert!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "FAIL-CLOSED: {name}={} must be a regular non-symlink file",
        path.display()
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("FAIL-CLOSED: read {name}={}: {error}", path.display()))
}

#[test]
fn protected_layout_uses_fail_closed_owner_prose_default() {
    let change = git_change_paths_from_name_status_z(b"D\0policy/ADR.md\0")
        .expect("one exact deletion record");
    let violations = changed_layout_violations(&change, &["policy".to_owned()].into());
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("frozen non-root Markdown"))
    );
    assert_eq!(change.deleted.len(), 1);
    assert!(change.deleted.contains("policy/ADR.md"));

    let workflow = read_declared(
        "OYATIE_PIPELINE_PRESUBMIT_WORKFLOW",
        ".github/workflows/presubmit.yml",
    );
    assert!(workflow.contains("pipeline-path-layout-app\" \"$base_sha\" \"$head_sha\""));
    assert!(
        !workflow.contains("--owner-prose-view"),
        "no trusted external view transport is claimed live"
    );

    let facade = format!(
        "{}\n{}",
        read_declared(
            "OYATIE_PIPELINE_PATH_LAYOUT_MAIN",
            "pipeline/facade/path-layout-app/src/main.rs",
        ),
        read_declared(
            "OYATIE_PIPELINE_OWNER_PROSE_VIEW_SOURCE",
            "pipeline/facade/path-layout-app/src/owner_prose_view.rs",
        )
    );
    assert!(facade.contains("qualify_owner_prose"));
    assert!(facade.contains("owner prose view Unknown"));
    let port = read_declared(
        "OYATIE_PIPELINE_REPOSITORY_PORT_SOURCE",
        "pipeline/ports/draft/repository/src/lib.rs",
    );
    for operation in ["repository_identity", "resolve_commit", "tree_id"] {
        assert!(port.contains(operation), "{operation}");
    }
}
