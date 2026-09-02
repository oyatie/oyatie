use std::path::PathBuf;

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn exact_pinned_upstream_toolchain_declaration_is_supported() {
    let root = PathBuf::from(
        std::env::var_os("REINDEER_PINNED_SOURCE_ROOT")
            .expect("REINDEER_PINNED_SOURCE_ROOT must name the exact pinned checkout"),
    );
    assert_eq!(
        std::fs::read(root.join("rust-toolchain")).expect("pinned rust-toolchain must be readable"),
        b"[toolchain]\nchannel = \"nightly-2026-05-22\"\n"
    );
}
