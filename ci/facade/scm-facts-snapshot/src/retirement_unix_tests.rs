//! Unix dirfd helper tests.

use rustix::fs::{Mode, OFlags};
use std::sync::atomic::Ordering;

use super::{NEXT_ATOMIC_WRITE_ID, create_temporary_file_with_prefix};

#[test]
fn temporary_file_errors_name_the_requested_prefix() {
    let path = std::env::temp_dir().join(format!(
        "retirement-temporary-file-prefix-{}",
        NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, b"not a directory").expect("write non-directory fixture");
    let file = rustix::fs::openat(
        rustix::fs::CWD,
        &path,
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .expect("open non-directory fixture");

    let error = create_temporary_file_with_prefix(&file, ".epoch-receipt")
        .expect_err("a non-directory fd cannot create a temporary file");
    assert!(
        error.contains("temporary file with prefix \".epoch-receipt\""),
        "unexpected error: {error}"
    );
    assert!(
        !error.contains("retirement facts"),
        "generic helper must not name a different caller: {error}"
    );
    std::fs::remove_file(path).expect("remove non-directory fixture");
}
