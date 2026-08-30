/// Build-relevant mode of one file in the exact provider source tree.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReindeerProviderSourceModeV1 {
    Regular = 0,
    Executable = 1,
}

/// One file in the exact provider source tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderSourceFileV1 {
    path: String,
    mode: ReindeerProviderSourceModeV1,
    bytes: Box<[u8]>,
}

impl ReindeerProviderSourceFileV1 {
    /// Records one source file with its exact mode and bytes.
    pub fn new(
        path: impl Into<String>,
        mode: ReindeerProviderSourceModeV1,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            path: path.into(),
            mode,
            bytes: bytes.into_boxed_slice(),
        }
    }

    /// Returns the repository-relative source path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the exact source mode.
    #[must_use]
    pub const fn mode(&self) -> ReindeerProviderSourceModeV1 {
        self.mode
    }

    /// Returns the exact source bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// A canonical, bounded but unadmitted whole-source candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderSourceSnapshotV1 {
    source_revision: String,
    files: Box<[ReindeerProviderSourceFileV1]>,
    source_tree_sha256: ReindeerProviderDigestV1,
    total_bytes: u64,
}

impl ReindeerProviderSourceSnapshotV1 {
    /// Canonicalizes and bounds an untrusted whole-source candidate.
    pub fn try_new(
        source_revision: impl Into<String>,
        files: Vec<ReindeerProviderSourceFileV1>,
    ) -> Result<Self, ReindeerProviderAdaptationErrorV1> {
        canonical_reindeer_provider_source_snapshot_v1(source_revision.into(), files)
    }

    /// Returns the claimed immutable upstream revision.
    #[must_use]
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    /// Returns the canonical complete source tree.
    #[must_use]
    pub fn files(&self) -> &[ReindeerProviderSourceFileV1] {
        &self.files
    }

    /// Returns the SHA-256 identity of every source path, mode, and byte.
    #[must_use]
    pub const fn source_tree_sha256(&self) -> ReindeerProviderDigestV1 {
        self.source_tree_sha256
    }

    /// Returns the aggregate source byte count.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

/// Immutable upstream and recipe identity for provider adaptation v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReindeerProviderAdaptationProfileV1;

impl ReindeerProviderAdaptationProfileV1 {
    /// Returns the sole admitted upstream repository.
    #[must_use]
    pub const fn source_repository(self) -> &'static str {
        REINDEER_SOURCE_REPOSITORY_V1
    }

    /// Returns the sole admitted upstream tag.
    #[must_use]
    pub const fn source_tag(self) -> &'static str {
        REINDEER_SOURCE_TAG_V1
    }

    /// Returns the immutable upstream revision.
    #[must_use]
    pub const fn source_revision(self) -> &'static str {
        PINNED_SOURCE_REVISION
    }

    /// Returns the admitted whole-source tree identity.
    #[must_use]
    pub const fn source_tree_sha256(self) -> ReindeerProviderDigestV1 {
        REINDEER_SOURCE_TREE_SHA256_V1
    }

    /// Returns the exact syntax, formatter, token, and digest recipe identity.
    #[must_use]
    pub const fn recipe_identity(self) -> &'static str {
        REINDEER_ADAPTATION_RECIPE_ID_V1
    }
}

/// One complete preimage-to-postimage result from the source recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderAdaptedFileV1 {
    path: String,
    preimage: Option<Box<[u8]>>,
    preimage_sha256: Option<ReindeerProviderDigestV1>,
    postimage: Box<[u8]>,
    postimage_sha256: ReindeerProviderDigestV1,
}

impl ReindeerProviderAdaptedFileV1 {
    /// Returns the repository-relative path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the exact upstream bytes, or `None` for a generated path.
    #[must_use]
    pub fn preimage(&self) -> Option<&[u8]> {
        self.preimage.as_deref()
    }

    /// Returns the upstream byte identity, or `None` for a generated path.
    #[must_use]
    pub const fn preimage_sha256(&self) -> Option<ReindeerProviderDigestV1> {
        self.preimage_sha256
    }

    /// Returns the exact generated postimage bytes.
    #[must_use]
    pub fn postimage(&self) -> &[u8] {
        &self.postimage
    }

    /// Returns the adapted byte identity.
    #[must_use]
    pub const fn postimage_sha256(&self) -> ReindeerProviderDigestV1 {
        self.postimage_sha256
    }
}

/// One deterministic whole-batch provider adaptation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderSourceAdaptationV1 {
    source_tree_sha256: ReindeerProviderDigestV1,
    adapted_batch_sha256: ReindeerProviderDigestV1,
    schema: ReindeerProviderSchemaV1,
    files: Box<[ReindeerProviderAdaptedFileV1]>,
    receipt_sha256: ReindeerProviderDigestV1,
}

impl ReindeerProviderSourceAdaptationV1 {
    /// Returns the exact provider and source-recipe profile.
    #[must_use]
    pub const fn profile(&self) -> ReindeerProviderAdaptationProfileV1 {
        ReindeerProviderAdaptationProfileV1
    }

    /// Returns how many upstream Rust files were parsed once.
    #[must_use]
    pub const fn parsed_source_files(&self) -> u64 {
        5
    }

    /// Returns the admitted whole-source tree identity.
    #[must_use]
    pub const fn source_tree_sha256(&self) -> ReindeerProviderDigestV1 {
        self.source_tree_sha256
    }

    /// Returns the identity of the complete canonical adaptation batch.
    #[must_use]
    pub const fn adapted_batch_sha256(&self) -> ReindeerProviderDigestV1 {
        self.adapted_batch_sha256
    }

    /// Returns the closed producer schema proved during adaptation.
    #[must_use]
    pub const fn schema(&self) -> &ReindeerProviderSchemaV1 {
        &self.schema
    }

    /// Returns every adapted file in canonical path order.
    #[must_use]
    pub fn files(&self) -> &[ReindeerProviderAdaptedFileV1] {
        &self.files
    }

    /// Returns the whole-adaptation identity.
    #[must_use]
    pub const fn receipt_sha256(&self) -> ReindeerProviderDigestV1 {
        self.receipt_sha256
    }
}

/// A fail-closed source-adaptation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReindeerProviderAdaptationErrorV1 {
    UnsupportedSourceRevision,
    SourceBatchMismatch,
    SourcePresenceMismatch,
    SourceFileDigestMismatch,
    SourceTreeMismatch,
    SourceTooLarge,
    InvalidUtf8,
    InvalidRust,
    UnsupportedSourceShape,
    UnsupportedBuckSourceShape,
    UnsupportedBuckifySourceShape,
    UnsupportedIndexSourceShape,
    UnsupportedMainSourceShape,
    UnsupportedVersionNamingSourceShape,
    OverlappingSourceEdit,
    GeneratedSourceInvalid,
    OutputTooLarge,
    ProviderSchema(ReindeerProviderSchemaErrorV1),
}

impl fmt::Display for ReindeerProviderAdaptationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Reindeer provider adaptation refused: {self:?}")
    }
}

impl Error for ReindeerProviderAdaptationErrorV1 {}

impl From<ReindeerProviderSchemaErrorV1> for ReindeerProviderAdaptationErrorV1 {
    fn from(error: ReindeerProviderSchemaErrorV1) -> Self {
        Self::ProviderSchema(error)
    }
}
