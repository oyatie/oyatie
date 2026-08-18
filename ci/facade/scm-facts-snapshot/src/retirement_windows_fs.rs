//! Windows parent walk and same-directory best-effort replace.
//!
//! Rejects reparse points, `\\?\`, non-disk prefixes, `..`, and NUL. Exclusive
//! temp + `write_all` + `sync_all`, then `remove_file` if present and `rename`.
//! Not `renameat`-atomic and not dirfd / TOCTOU-closed.

use std::io::Write;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::super::NEXT_ATOMIC_WRITE_ID;

/// Walk/create a real, non-reparse parent. Rejects `\\?\`, non-disk prefixes,
/// `..`, and NUL. Not dirfd-bound.
pub(super) fn open_real_windows_parent(
    repo_root: &Path,
    parent_components: &[&str],
    label: &str,
) -> Result<PathBuf, String> {
    reject_windows_path(repo_root, label)?;
    ensure_metadata_is_real_directory(
        &std::fs::symlink_metadata(repo_root)
            .map_err(|error| format!("inspect {label} directory \"<repo-root>\": {error}"))?,
        "<repo-root>",
        label,
    )?;
    let mut directory = repo_root.to_path_buf();
    for component in parent_components {
        reject_windows_component(component, label)?;
        directory.push(component);
        match std::fs::symlink_metadata(&directory) {
            Ok(metadata) => ensure_metadata_is_real_directory(&metadata, component, label)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match std::fs::create_dir(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(format!("create {label} directory {component:?}: {error}"));
                    }
                }
                let metadata = std::fs::symlink_metadata(&directory)
                    .map_err(|error| format!("inspect {label} directory {component:?}: {error}"))?;
                ensure_metadata_is_real_directory(&metadata, component, label)?;
            }
            Err(error) => {
                return Err(format!("inspect {label} directory {component:?}: {error}"));
            }
        }
    }
    Ok(directory)
}

fn reject_windows_path(path: &Path, label: &str) -> Result<(), String> {
    let raw = path.as_os_str().to_string_lossy();
    if raw.contains('\0') {
        return Err(format!("{label} path contains NUL"));
    }
    if raw.contains("\\\\?\\") || raw.contains("//?/") {
        return Err(format!("{label} path must not use a \\\\?\\ prefix"));
    }
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => {
                if !matches!(prefix.kind(), std::path::Prefix::Disk(_)) {
                    return Err(format!(
                        "{label} path must not use a verbatim, UNC, or device prefix"
                    ));
                }
            }
            std::path::Component::ParentDir => {
                return Err(format!("{label} path must not contain .."));
            }
            std::path::Component::Normal(name) => {
                if name.to_string_lossy().contains('\0') {
                    return Err(format!("{label} path contains NUL"));
                }
            }
            std::path::Component::RootDir | std::path::Component::CurDir => {}
        }
    }
    Ok(())
}

fn reject_windows_component(component: &str, label: &str) -> Result<(), String> {
    if component.contains('\0') {
        return Err(format!("{label} directory contains NUL: {component:?}"));
    }
    if component == ".."
        || component.contains('/')
        || component.contains('\\')
        || component.contains(':')
    {
        return Err(format!(
            "{label} directory {component:?} must be a single path component"
        ));
    }
    Ok(())
}

fn ensure_metadata_is_real_directory(
    metadata: &std::fs::Metadata,
    component: &str,
    label: &str,
) -> Result<(), String> {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0
    {
        return Err(format!(
            "{label} directory {component:?} is not a real directory"
        ));
    }
    Ok(())
}

/// Same-directory best-effort replace: exclusive temp, persist, remove+rename.
/// Not `renameat`-atomic and not dirfd / TOCTOU-closed. Unlink temp on error.
pub(super) fn replace_regular_file_best_effort(
    directory: &Path,
    final_name: &str,
    temporary_prefix: &str,
    bytes: &[u8],
) -> Result<(), String> {
    if final_name.contains('\0') || final_name.contains('/') || final_name.contains('\\') {
        return Err(format!(
            "ignored generated basename must be a single path component: {final_name:?}"
        ));
    }
    ensure_windows_regular_or_absent(directory, final_name)?;
    let (temporary_path, mut temporary) =
        create_exclusive_windows_temp(directory, temporary_prefix)?;
    let result = (|| {
        temporary
            .write_all(bytes)
            .map_err(|error| format!("write ignored generated temporary file: {error}"))?;
        temporary
            .sync_all()
            .map_err(|error| format!("sync ignored generated temporary file: {error}"))?;
        drop(temporary);
        let dest = directory.join(final_name);
        match std::fs::symlink_metadata(&dest) {
            Ok(metadata)
                if !metadata.file_type().is_file() || metadata.file_type().is_symlink() =>
            {
                Err("retirement facts output must be a regular file".to_owned())
            }
            Ok(_) => {
                std::fs::remove_file(&dest)
                    .map_err(|error| format!("replace ignored generated output: {error}"))?;
                std::fs::rename(&temporary_path, &dest)
                    .map_err(|error| format!("replace ignored generated output: {error}"))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::rename(&temporary_path, &dest)
                    .map_err(|error| format!("replace ignored generated output: {error}"))
            }
            Err(error) => Err(format!("inspect retirement facts output: {error}")),
        }
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary_path);
    }
    result
}

fn ensure_windows_regular_or_absent(directory: &Path, name: &str) -> Result<(), String> {
    match std::fs::symlink_metadata(directory.join(name)) {
        Ok(metadata) if !metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            Err("retirement facts output must be a regular file".to_owned())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("inspect retirement facts output: {error}")),
    }
}

fn create_exclusive_windows_temp(
    directory: &Path,
    prefix: &str,
) -> Result<(PathBuf, std::fs::File), String> {
    for _ in 0..32 {
        let name = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
        );
        let path = directory.join(&name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
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
