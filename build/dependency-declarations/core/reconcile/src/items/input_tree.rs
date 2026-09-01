const TREE_MANIFEST_HEADER_BYTES: usize = 8;
const TREE_MANIFEST_ENTRY_FIXED_BYTES: usize = 8 + 1 + 8 + 32;

/// Semantic mount occupied by one declared input tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum TreeRoleV1 {
    RepositoryRead = 0,
    Fixups = 1,
    CargoHomeRead = 2,
}

/// Admitted regular-file mode in a declared input tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum TreeFileModeV1 {
    Regular = 0,
    Executable = 1,
}

/// One content-addressed regular file in an input tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TreeEntryV1 {
    path: CanonicalPathV1,
    mode: TreeFileModeV1,
    length_bytes: u64,
    sha256: DigestV1,
}

impl TreeEntryV1 {
    /// Creates one exact file descriptor without loading its content.
    #[must_use]
    pub const fn new(
        path: CanonicalPathV1,
        mode: TreeFileModeV1,
        length_bytes: u64,
        sha256: DigestV1,
    ) -> Self {
        Self {
            path,
            mode,
            length_bytes,
            sha256,
        }
    }

    fn append_manifest(&self, output: &mut Vec<u8>) -> Result<(), FailureV1> {
        append_manifest_bytes(output, self.path.as_str().as_bytes())?;
        append_manifest_raw(output, &[self.mode as u8])?;
        append_manifest_raw(output, &self.length_bytes.to_be_bytes())?;
        append_manifest_raw(output, &self.sha256.bytes())
    }

    fn matches_file(&self, file: &InputFileV1) -> bool {
        self.mode == TreeFileModeV1::Regular
            && self.path == file.path
            && self.length_bytes == file.length_bytes
            && self.sha256 == file.sha256
    }
}

/// A bounded, typed, canonically ordered input-tree descriptor.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InputTreeV1 {
    pub(crate) role: TreeRoleV1,
    pub(crate) manifest: InputFileV1,
    pub(crate) entries: Box<[TreeEntryV1]>,
    pub(crate) root_sha256: DigestV1,
    pub(crate) file_count: u64,
    pub(crate) total_bytes: u64,
}

impl InputTreeV1 {
    /// Canonicalizes and retains exact entries for whole-tree materialization.
    pub fn try_from_entries(
        role: TreeRoleV1,
        manifest_path: CanonicalPathV1,
        mut entries: Vec<TreeEntryV1>,
    ) -> Result<Self, FailureV1> {
        let file_count = checked_u64(entries.len(), invalid_request())?;
        let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
            total
                .checked_add(entry.length_bytes)
                .ok_or_else(invalid_request)
        })?;
        validate_tree_bounds(role, file_count, total_bytes)?;
        let manifest_bytes = checked_tree_manifest_bytes(&entries)?;
        entries.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        if entries.windows(2).any(|pair| pair[0].path == pair[1].path) {
            return Err(invalid_request());
        }

        let mut bytes = Vec::with_capacity(manifest_bytes);
        append_manifest_raw(&mut bytes, &file_count.to_be_bytes())?;
        for entry in &entries {
            entry.append_manifest(&mut bytes)?;
        }
        let manifest = InputFileV1::try_new(InputFileRoleV1::TreeManifest, manifest_path, bytes)?;
        let mut hash = CanonicalHasherV1::new(match role {
            TreeRoleV1::RepositoryRead => b"build.input-tree.repository-read.v1\0",
            TreeRoleV1::Fixups => b"build.input-tree.fixups.v1\0",
            TreeRoleV1::CargoHomeRead => b"build.input-tree.cargo-home-read.v1\0",
        });
        hash.tag(role as u8);
        hash.digest(manifest.sha256);
        hash.u64(file_count);
        hash.u64(total_bytes);
        Ok(Self {
            role,
            manifest,
            entries: entries.into_boxed_slice(),
            root_sha256: hash.finish(),
            file_count,
            total_bytes,
        })
    }

    pub(crate) fn contains_file(&self, file: &InputFileV1) -> bool {
        self.entries
            .binary_search_by(|entry| entry.path.cmp(&file.path))
            .is_ok_and(|index| self.entries[index].matches_file(file))
    }

    pub(crate) fn contains_entry(&self, expected: &TreeEntryV1) -> bool {
        self.entries
            .binary_search_by(|entry| entry.path.cmp(&expected.path))
            .is_ok_and(|index| self.entries[index] == *expected)
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

fn checked_tree_manifest_bytes(entries: &[TreeEntryV1]) -> Result<usize, FailureV1> {
    entries
        .iter()
        .try_fold(TREE_MANIFEST_HEADER_BYTES, |total, entry| {
            checked_tree_manifest_entry_bytes(total, entry.path.as_str().len())
        })
}

fn checked_tree_manifest_entry_bytes(
    total: usize,
    path_bytes: usize,
) -> Result<usize, FailureV1> {
    total
        .checked_add(TREE_MANIFEST_ENTRY_FIXED_BYTES)
        .and_then(|total| total.checked_add(path_bytes))
        .filter(|total| *total <= ValidationBoundsV1::MAX_DECLARED_FILE_BYTES)
        .ok_or_else(invalid_request)
}

fn validate_tree_bounds(
    role: TreeRoleV1,
    file_count: u64,
    total_bytes: u64,
) -> Result<(), FailureV1> {
    let valid = match role {
        TreeRoleV1::RepositoryRead => {
            file_count <= ValidationBoundsV1::MAX_REPOSITORY_READ_FILES
                && total_bytes <= ValidationBoundsV1::MAX_REPOSITORY_READ_BYTES
        }
        TreeRoleV1::Fixups => {
            file_count <= ValidationBoundsV1::MAX_FIXUP_FILES
                && total_bytes <= ValidationBoundsV1::MAX_FIXUP_BYTES
        }
        TreeRoleV1::CargoHomeRead => {
            file_count <= ValidationBoundsV1::MAX_CARGO_HOME_READ_FILES
                && total_bytes <= ValidationBoundsV1::MAX_CARGO_HOME_READ_BYTES
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

#[cfg(test)]
mod input_tree_tests {
    use super::*;

    #[test]
    fn tree_manifest_size_refuses_limit_and_integer_overflow() {
        let maximum = ValidationBoundsV1::MAX_DECLARED_FILE_BYTES;
        let entry_bytes = TREE_MANIFEST_ENTRY_FIXED_BYTES + 1;
        assert_eq!(
            checked_tree_manifest_entry_bytes(maximum - entry_bytes, 1),
            Ok(maximum)
        );
        for total in [maximum - entry_bytes + 1, usize::MAX] {
            assert_eq!(
                checked_tree_manifest_entry_bytes(total, 1)
                    .unwrap_err()
                    .class(),
                FailureClassV1::InvalidRequest
            );
        }
    }
}
