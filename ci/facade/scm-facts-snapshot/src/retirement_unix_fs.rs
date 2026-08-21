//! Unix dirfd helpers: openat / NOFOLLOW / renameat / fsync.

use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{AtFlags, FileType, Mode, OFlags};
use std::sync::atomic::Ordering;

use super::NEXT_ATOMIC_WRITE_ID;

pub(crate) fn open_canonical_retirement_facts_parent(repo_root: &Path) -> Result<OwnedFd, String> {
    let mut directory = rustix::fs::openat(
        rustix::fs::CWD,
        repo_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| format!("open retirement facts repository directory: {error}"))?;
    for component in ["ci", "facade", "scm-facts-snapshot"] {
        directory = open_or_create_directory_at(&directory, component)?;
    }
    Ok(directory)
}

pub(crate) fn open_or_create_directory_at(parent: &OwnedFd, name: &str) -> Result<OwnedFd, String> {
    if name.contains('\0') {
        return Err(format!("retirement facts directory contains NUL: {name:?}"));
    }
    match rustix::fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(directory) => Ok(directory),
        Err(error) if error == rustix::io::Errno::NOENT => {
            match rustix::fs::mkdirat(parent, name, Mode::from_bits_retain(0o755)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(error) => {
                    return Err(format!(
                        "create retirement facts directory {name:?}: {error}"
                    ));
                }
            }
            rustix::fs::openat(
                parent,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| format!("open retirement facts directory {name:?}: {error}"))
        }
        Err(error) if error == rustix::io::Errno::NOTDIR || error == rustix::io::Errno::LOOP => {
            Err(format!(
                "retirement facts directory {name:?} is not a real directory"
            ))
        }
        Err(error) => Err(format!("open retirement facts directory {name:?}: {error}")),
    }
}

pub(crate) fn ensure_regular_or_absent(directory: &OwnedFd, name: &str) -> Result<(), String> {
    match rustix::fs::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) if !FileType::from_raw_mode(stat.st_mode).is_file() => {
            Err("retirement facts output must be a regular file".to_owned())
        }
        Ok(_) | Err(rustix::io::Errno::NOENT) => Ok(()),
        Err(error) => Err(format!("inspect retirement facts output: {error}")),
    }
}

pub(crate) fn create_temporary_file_with_prefix(
    directory: &OwnedFd,
    prefix: &str,
) -> Result<(String, OwnedFd), String> {
    for _ in 0..32 {
        let name = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
        );
        match rustix::fs::openat(
            directory,
            &name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::from_bits_retain(0o600),
        ) {
            Ok(file) => return Ok((name, file)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => {
                return Err(format!(
                    "create temporary file with prefix {prefix:?}: {error}"
                ));
            }
        }
    }
    Err(format!(
        "exhausted temporary file names with prefix {prefix:?}"
    ))
}

pub(crate) fn atomic_replace_ignored_generated_file(
    directory: &OwnedFd,
    final_name: &str,
    temporary_prefix: &str,
    bytes: &[u8],
) -> Result<(), String> {
    ensure_regular_or_absent(directory, final_name)?;
    let (temporary_name, temporary) =
        create_temporary_file_with_prefix(directory, temporary_prefix)?;
    let result = (|| {
        write_all(&temporary, bytes)?;
        rustix::fs::fsync(&temporary)
            .map_err(|error| format!("sync ignored generated temporary file: {error}"))?;
        rustix::fs::renameat(directory, &temporary_name, directory, final_name)
            .map_err(|error| format!("replace ignored generated output: {error}"))?;
        rustix::fs::fsync(directory)
            .map_err(|error| format!("sync ignored generated directory: {error}"))
    })();
    if result.is_err() {
        let _ = rustix::fs::unlinkat(directory, &temporary_name, AtFlags::empty());
    }
    result
}

pub(crate) fn write_all(file: &OwnedFd, mut bytes: &[u8]) -> Result<(), String> {
    while !bytes.is_empty() {
        let written = rustix::io::write(file, bytes)
            .map_err(|error| format!("write retirement facts temporary file: {error}"))?;
        if written == 0 {
            return Err("write retirement facts temporary file made no progress".to_owned());
        }
        bytes = &bytes[written..];
    }
    Ok(())
}
