//! Advisory ownership of supported local regular files, held until the last handle drops.
#![deny(unsafe_code)]

mod native;

use foundry_store_access::{StoreAccess, StoreIdentity};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOpen {
    Create,
    Existing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreAccessError {
    Busy,
    Unsupported,
    NotRegular,
    IdentityChanged,
    Aliased,
    WrongOwner,
    UnsupportedPath,
    Io(String),
}
impl std::fmt::Display for StoreAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "store access refused: {self:?}")
    }
}
impl std::error::Error for StoreAccessError {}

#[derive(Debug)]
struct Lease {
    _file: File,
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LocalStoreGuard {
    lease: Arc<Lease>,
    access: StoreAccess,
}

impl LocalStoreGuard {
    pub fn shared(path: &Path, open: StoreOpen) -> Result<Self, StoreAccessError> {
        Self::acquire(path, open, false)
    }
    fn acquire(path: &Path, open: StoreOpen, exclusive: bool) -> Result<Self, StoreAccessError> {
        if path == Path::new(":memory:")
            || path.as_os_str().as_encoded_bytes().starts_with(b"file:")
        {
            return Err(StoreAccessError::UnsupportedPath);
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(open == StoreOpen::Create)
            .truncate(false)
            .open(path)
            .map_err(io)?;
        let id = identity(&file.metadata().map_err(io)?)?;
        native::lock(&file, exclusive)?;
        let guard = Self {
            lease: Arc::new(Lease {
                _file: file,
                path: path.canonicalize().map_err(io)?,
            }),
            access: StoreAccess::new(id, exclusive),
        };
        guard.validate(path)?;
        Ok(guard)
    }
    pub fn path(&self) -> &Path {
        &self.lease.path
    }
    pub fn access(&self) -> &StoreAccess {
        &self.access
    }
    pub fn validate(&self, path: &Path) -> Result<(), StoreAccessError> {
        if path.canonicalize().map_err(io)? != self.lease.path {
            return Err(StoreAccessError::WrongOwner);
        }
        if identity(&path.metadata().map_err(io)?)? != self.access.identity() {
            return Err(StoreAccessError::IdentityChanged);
        }
        Ok(())
    }
}

pub struct LocalStoreSet {
    guards: Vec<LocalStoreGuard>,
}
impl LocalStoreSet {
    pub fn exclusive(paths: &[(PathBuf, StoreOpen)]) -> Result<Self, StoreAccessError> {
        let mut ordered = paths.to_vec();
        ordered.sort_by(|left, right| left.0.cmp(&right.0));
        let mut guards: Vec<LocalStoreGuard> = Vec::new();
        for (path, open) in ordered {
            if let Ok(metadata) = path.metadata() {
                let id = identity(&metadata)?;
                if guards.iter().any(|other| other.access.identity() == id) {
                    return Err(StoreAccessError::Aliased);
                }
            }
            let guard = LocalStoreGuard::acquire(&path, open, true)?;
            if guards
                .iter()
                .any(|other| other.access.identity() == guard.access.identity())
            {
                return Err(StoreAccessError::Aliased);
            }
            guards.push(guard);
        }
        Ok(Self { guards })
    }
    pub fn guard(&self, path: &Path) -> Result<LocalStoreGuard, StoreAccessError> {
        self.guards
            .iter()
            .find(|guard| guard.validate(path).is_ok())
            .cloned()
            .ok_or(StoreAccessError::WrongOwner)
    }
}

fn io(error: std::io::Error) -> StoreAccessError {
    StoreAccessError::Io(error.to_string())
}
#[cfg(unix)]
fn identity(metadata: &std::fs::Metadata) -> Result<StoreIdentity, StoreAccessError> {
    use std::os::unix::fs::MetadataExt;
    if !metadata.is_file() {
        return Err(StoreAccessError::NotRegular);
    }
    Ok(StoreIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}
#[cfg(not(unix))]
fn identity(_: &std::fs::Metadata) -> Result<StoreIdentity, StoreAccessError> {
    Err(StoreAccessError::Unsupported)
}

#[cfg(test)]
mod tests;
