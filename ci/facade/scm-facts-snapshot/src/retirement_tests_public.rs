//! Cross-platform writer-type smoke tests.

use std::path::Path;

use super::{
    CanonicalIgnoredGeneratedWriter, write_canonical_ignored_generated_file,
};

/// Windows soft-smoke regression: integration targets import this type on all
/// platforms. The non-unix stub must keep the name public (see GHA E0432 when
/// only `#[cfg(unix)]` existed).
#[test]
fn canonical_ignored_generated_writer_is_public_on_all_platforms() {
    let _name = std::any::type_name::<CanonicalIgnoredGeneratedWriter>();
    assert!(
        _name.contains("CanonicalIgnoredGeneratedWriter"),
        "type must remain public for cross-platform integration imports: {_name}"
    );
}

#[cfg(not(any(unix, windows)))]
#[test]
fn non_unix_canonical_ignored_generated_writer_fails_closed() {
    let err = CanonicalIgnoredGeneratedWriter::open(
        Path::new("."),
        Path::new(
            "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json",
        ),
    )
    .expect_err("non-unix stub must fail closed");
    assert!(
        err.contains("Unix dirfd"),
        "unexpected non-unix stub error: {err}"
    );
    let err = write_canonical_ignored_generated_file(
        Path::new("."),
        Path::new(
            "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json",
        ),
        b"{}",
    )
    .expect_err("non-unix free function must fail closed");
    assert!(
        err.contains("Unix dirfd"),
        "unexpected non-unix free-function error: {err}"
    );
}
