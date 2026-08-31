use std::fmt;
use std::str;

use crate::{DigestBuilder, IdentityError};

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryPath(Box<[u8]>);

impl RepositoryPath {
    pub fn new(value: impl Into<Vec<u8>>) -> Result<Self, IdentityError> {
        let value = value.into();
        validate_path(&value)?;
        Ok(Self(value.into_boxed_slice()))
    }

    pub fn from_utf8(value: impl Into<String>) -> Result<Self, IdentityError> {
        Self::new(value.into().into_bytes())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_utf8(&self) -> Result<&str, IdentityError> {
        str::from_utf8(&self.0)
            .map_err(|_| IdentityError::new("repository path", "is not valid UTF-8"))
    }

    pub fn is_under(&self, directory: &RepositoryPath) -> bool {
        self.0
            .strip_prefix(directory.as_bytes())
            .is_some_and(|suffix| suffix.first() == Some(&b'/'))
    }

    pub(crate) fn digest_into(&self, digest: &mut DigestBuilder) {
        digest.push_bytes(&self.0);
    }
}

impl fmt::Debug for RepositoryPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_utf8() {
            Ok(path) => path.fmt(formatter),
            Err(_) => self.0.fmt(formatter),
        }
    }
}

impl fmt::Display for RepositoryPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_utf8() {
            Ok(path) => formatter.write_str(path),
            Err(_) => write!(formatter, "{:?}", self.0),
        }
    }
}

fn validate_path(path: &[u8]) -> Result<(), IdentityError> {
    if path.is_empty() {
        return Err(IdentityError::new("repository path", "must not be empty"));
    }
    if path.contains(&0) {
        return Err(IdentityError::new(
            "repository path",
            "must not contain NUL",
        ));
    }
    if path.first() == Some(&b'/') || path.last() == Some(&b'/') {
        return Err(IdentityError::new(
            "repository path",
            "must be relative and must not end with a separator",
        ));
    }
    if path
        .split(|byte| *byte == b'/')
        .any(|part| part.is_empty() || part == b"." || part == b"..")
    {
        return Err(IdentityError::new(
            "repository path",
            "must contain only non-empty canonical components",
        ));
    }
    Ok(())
}
