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

/// Role of a manifest-described input tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum TreeRoleV1 {
    Fixups = 0,
    CargoSource = 1,
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

/// One content-addressed entry in an input-tree manifest.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TreeEntryV1 {
    path: CanonicalPathV1,
    length_bytes: u64,
    sha256: DigestV1,
}

impl TreeEntryV1 {
    /// Creates a bounded tree entry without loading its content.
    #[must_use]
    pub const fn new(path: CanonicalPathV1, length_bytes: u64, sha256: DigestV1) -> Self {
        Self {
            path,
            length_bytes,
            sha256,
        }
    }

    fn append_manifest(&self, output: &mut Vec<u8>) -> Result<(), FailureV1> {
        append_manifest_bytes(output, self.path.as_str().as_bytes())?;
        append_manifest_raw(output, &self.length_bytes.to_be_bytes())?;
        append_manifest_raw(output, &self.sha256.bytes())
    }
}

/// A bounded tree represented only by its canonical entry manifest.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InputTreeV1 {
    pub(crate) role: TreeRoleV1,
    pub(crate) manifest: InputFileV1,
    pub(crate) root_sha256: DigestV1,
    pub(crate) file_count: u64,
    pub(crate) total_bytes: u64,
}

impl InputTreeV1 {
    /// Canonicalizes entries by path and materializes only their metadata manifest.
    pub fn try_from_entries(
        role: TreeRoleV1,
        manifest_path: CanonicalPathV1,
        mut entries: Vec<TreeEntryV1>,
    ) -> Result<Self, FailureV1> {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        if entries.windows(2).any(|pair| pair[0].path == pair[1].path) {
            return Err(invalid_request());
        }
        let file_count = checked_u64(entries.len(), invalid_request())?;
        let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
            total
                .checked_add(entry.length_bytes)
                .ok_or_else(invalid_request)
        })?;
        validate_tree_bounds(role, file_count, total_bytes)?;

        let mut bytes = Vec::new();
        append_manifest_raw(&mut bytes, &file_count.to_be_bytes())?;
        for entry in &entries {
            entry.append_manifest(&mut bytes)?;
        }
        let manifest = InputFileV1::try_new(InputFileRoleV1::TreeManifest, manifest_path, bytes)?;
        let mut hash = CanonicalHasherV1::new(match role {
            TreeRoleV1::Fixups => b"build.input-tree.fixups.v1\0",
            TreeRoleV1::CargoSource => b"build.input-tree.cargo-source.v1\0",
        });
        hash.tag(role as u8);
        hash.digest(manifest.sha256);
        hash.u64(file_count);
        hash.u64(total_bytes);
        Ok(Self {
            role,
            manifest,
            root_sha256: hash.finish(),
            file_count,
            total_bytes,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.tag(self.role as u8);
        self.manifest.encode(hash)?;
        hash.digest(self.root_sha256);
        hash.u64(self.file_count);
        hash.u64(self.total_bytes);
        Ok(())
    }
}

fn validate_tree_bounds(
    role: TreeRoleV1,
    file_count: u64,
    total_bytes: u64,
) -> Result<(), FailureV1> {
    let valid = match role {
        TreeRoleV1::Fixups => {
            file_count <= ValidationBoundsV1::MAX_FIXUP_FILES
                && total_bytes <= ValidationBoundsV1::MAX_FIXUP_BYTES
        }
        TreeRoleV1::CargoSource => {
            file_count <= ValidationBoundsV1::MAX_CARGO_SOURCE_FILES
                && total_bytes <= ValidationBoundsV1::MAX_CARGO_SOURCE_BYTES
        }
    };
    valid.then_some(()).ok_or_else(invalid_request)
}

fn append_manifest_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), FailureV1> {
    append_manifest_raw(
        output,
        &checked_u64(bytes.len(), invalid_request())?.to_be_bytes(),
    )?;
    append_manifest_raw(output, bytes)
}

fn append_manifest_raw(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), FailureV1> {
    let next = output
        .len()
        .checked_add(bytes.len())
        .ok_or_else(invalid_request)?;
    if next > ValidationBoundsV1::MAX_DECLARED_FILE_BYTES {
        return Err(invalid_request());
    }
    output.extend_from_slice(bytes);
    Ok(())
}
