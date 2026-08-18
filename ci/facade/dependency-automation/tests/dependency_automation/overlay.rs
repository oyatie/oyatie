use std::fs;

use ci_dependency_automation::{
    apply_third_party_buck_overlay, apply_third_party_buck_overlay_file,
};

use crate::helpers::{repo_root, temp_root};

#[test]
fn third_party_overlay_file_boundary_is_idempotent_and_fail_closed() {
    let root = temp_root();
    let buck_file = root.join("BUCK");
    let source = r#"buildscript_run(
    name = "aws-lc-rs-1-build-script-run",
    env = {
        "CARGO_PKG_VERSION_PRE": "",
    },
)
cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = [
        "-DCFG_TARGET_OS_darwin",
        "-DCFG_TARGET_ARCH_aarch64",
        "-DCFG_TARGET_ENV_",
    ],
)
"#;
    fs::write(&buck_file, source).expect("write Reindeer fixture");

    assert_eq!(
        apply_third_party_buck_overlay_file(&buck_file).expect("apply overlay"),
        2
    );
    let patched = fs::read_to_string(&buck_file).expect("read patched fixture");
    assert_eq!(
        apply_third_party_buck_overlay_file(&buck_file).expect("reapply overlay"),
        0
    );
    assert_eq!(
        fs::read_to_string(&buck_file).expect("read idempotent fixture"),
        patched
    );

    let corrupt = patched.replace(
        "\"DEP_AWS_LC_0_41_0_INCLUDE\": \"$(location :aws-lc-sys-0.41-build-script-main-run[out_dir])/include\"",
        "\"DEP_AWS_LC_0_41_0_INCLUDE\": \"wrong\"",
    );
    fs::write(&buck_file, &corrupt).expect("write corrupt fixture");
    assert!(apply_third_party_buck_overlay_file(&buck_file).is_err());
    assert_eq!(
        fs::read_to_string(&buck_file).expect("read rejected fixture"),
        corrupt,
        "a rejected overlay must not mutate the source file"
    );

    let second_transform_failure = source.replace(
        r#"    preprocessor_flags = [
        "-DCFG_TARGET_OS_darwin",
        "-DCFG_TARGET_ARCH_aarch64",
        "-DCFG_TARGET_ENV_",
    ],"#,
        r#"    preprocessor_flags = ["unexpected"],"#,
    );
    fs::write(&buck_file, &second_transform_failure)
        .expect("write second-transform failure fixture");
    assert!(apply_third_party_buck_overlay_file(&buck_file).is_err());
    assert_eq!(
        fs::read_to_string(&buck_file).expect("read second-transform rejection"),
        second_transform_failure,
        "a later transform failure must not persist an earlier in-memory patch"
    );

    assert!(
        apply_third_party_buck_overlay_file(&root.join("missing.BUCK")).is_err(),
        "a missing file must fail closed"
    );
    assert!(
        apply_third_party_buck_overlay_file(&root).is_err(),
        "a non-file path must fail closed"
    );

    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn live_third_party_face_is_an_exact_overlay_noop() {
    let root = repo_root();
    let path = root.join("third-party/BUCK");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let overlay = apply_third_party_buck_overlay(&source).expect("validate live third-party face");
    assert_eq!(overlay.patches_applied, 0);
    assert_eq!(
        overlay.text, source,
        "the committed face must be byte-identical to the exact Rust overlay"
    );
}
