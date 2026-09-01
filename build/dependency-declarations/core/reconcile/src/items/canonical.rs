/// A validated repository-relative slash-separated path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalPathV1(Box<str>);

impl CanonicalPathV1 {
    /// Validates a path without normalizing caller bytes.
    pub fn try_new(value: impl Into<String>) -> Result<Self, FailureV1> {
        let value = value.into();
        if value.is_empty()
            || value.len() > ValidationBoundsV1::MAX_PATH_BYTES
            || value.contains(['\0', '\\'])
            || value.chars().any(char::is_control)
            || has_windows_drive_prefix(&value)
            || value
                .split('/')
                .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return Err(invalid_request());
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact validated path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.0)
    }
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':')
}

/// Role of a directly declared input file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum InputFileRoleV1 {
    Manifest = 0,
    Lock = 1,
    Config = 2,
    TreeManifest = 3,
}

/// One bounded file whose bytes and declared identity agree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InputFileV1 {
    pub(crate) role: InputFileRoleV1,
    pub(crate) path: CanonicalPathV1,
    pub(crate) length_bytes: u64,
    pub(crate) sha256: DigestV1,
    pub(crate) bytes: Box<[u8]>,
}

impl InputFileV1 {
    /// Creates a file with identities computed from its bytes.
    pub fn try_new(
        role: InputFileRoleV1,
        path: CanonicalPathV1,
        bytes: Vec<u8>,
    ) -> Result<Self, FailureV1> {
        let length_bytes = checked_u64(bytes.len(), invalid_request())?;
        let sha256 = DigestV1::of(&bytes);
        Self::try_from_declared(role, path, length_bytes, sha256, bytes)
    }

    /// Verifies untrusted declared metadata against exact bytes.
    pub fn try_from_declared(
        role: InputFileRoleV1,
        path: CanonicalPathV1,
        length_bytes: u64,
        sha256: DigestV1,
        bytes: Vec<u8>,
    ) -> Result<Self, FailureV1> {
        if bytes.len() > ValidationBoundsV1::MAX_DECLARED_FILE_BYTES
            || checked_u64(bytes.len(), invalid_request())? != length_bytes
            || DigestV1::of(&bytes) != sha256
        {
            return Err(invalid_request());
        }
        Ok(Self {
            role,
            path,
            length_bytes,
            sha256,
            bytes: bytes.into_boxed_slice(),
        })
    }

    /// Returns the verified digest.
    #[must_use]
    pub const fn sha256(&self) -> DigestV1 {
        self.sha256
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.tag(self.role as u8);
        self.path.encode(hash)?;
        hash.u64(self.length_bytes);
        hash.digest(self.sha256);
        hash.bytes(&self.bytes)
    }
}
