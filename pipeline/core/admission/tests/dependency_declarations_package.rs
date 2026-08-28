//! Exact package-local `PACKAGE` admission for dependency-declarations scanners.

use pipeline_admission::layout_violations;

fn rejected(path: &str) -> bool {
    !layout_violations(&[path.to_owned()]).is_empty()
}

#[test]
fn only_build_script_dependency_declarations_crates_admit_root_package() {
    for path in [
        "build/dependency-declarations/core/reconcile/PACKAGE",
        "build/dependency-declarations/adapters/generation-reindeer/PACKAGE",
        "build/dependency-declarations/adapters/publication-filesystem/PACKAGE",
        "build/dependency-declarations/facade/reconciler-app/PACKAGE",
    ] {
        assert!(!rejected(path), "expected PACKAGE admission: {path}");
    }

    for path in [
        "build/dependency-declarations/ports/generation/PACKAGE",
        "build/dependency-declarations/ports/publication/PACKAGE",
        "network/core/route/PACKAGE",
        "build/dependency-declarations/unknown/reconcile/PACKAGE",
        "build/dependency-declarations/core/unknown/PACKAGE",
        "build/dependency-declarations/core/reconcile/nested/PACKAGE",
        "build/dependency-declarations/core/reconcile/package",
        "build/dependency-declarations/core/reconcile/PACKAGE.txt",
        "build/other/PACKAGE",
    ] {
        assert!(rejected(path), "expected PACKAGE rejection: {path}");
    }
}
