/// Stable lifecycle refusal classes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum LifecycleFailureClassV1 {
    InvalidFact = 0,
    BoundsExceeded = 1,
    DuplicateIdentity = 2,
    MissingSource = 3,
    SourceCoverageMismatch = 4,
    MissingDisposition = 5,
    DuplicateDisposition = 6,
    ProvisionalSource = 7,
    ToolchainRoleMismatch = 8,
    UnsupportedVersionRelation = 9,
    ExtractionProfileMismatch = 10,
    UnqualifiedExtraction = 11,
    InternalInvariant = 12,
    InvalidPackageVersion = 13,
    AdvisorySourceMismatch = 14,
    ConflictingAdvisoryRange = 15,
    ConflictingAdvisoryHistory = 16,
    MixedAdvisoryQualification = 17,
    DependencySourceMismatch = 18,
    InvalidDependencyCandidate = 19,
    UnavailableDependencyRelease = 20,
    IncompleteFactCoverage = 21,
    StaleFact = 22,
    InvalidDependencyGraph = 23,
    MissingDependencyRoot = 24,
    UnsupportedFactEvidence = 25,
    DependencyAnalysisMismatch = 26,
    InvalidSecurityException = 27,
    DependencyImpactCancelled = 28,
    DependencyImpactDeadlineExceeded = 29,
    ToolchainTargetMismatch = 30,
    ToolchainVersionMismatch = 31,
    InvalidToolchainCandidate = 32,
    MissingToolchainIntent = 33,
    ToolchainIntentMismatch = 34,
    InvalidCurrencyException = 35,
    ToolchainAnalysisMismatch = 36,
    InvalidToolchainCurrencyException = 37,
}

/// A matchable lifecycle refusal without ambient diagnostic text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LifecycleFailureV1 {
    class: LifecycleFailureClassV1,
}

impl LifecycleFailureV1 {
    #[must_use]
    pub const fn new(class: LifecycleFailureClassV1) -> Self {
        Self { class }
    }

    #[must_use]
    pub const fn class(self) -> LifecycleFailureClassV1 {
        self.class
    }
}

impl std::fmt::Display for LifecycleFailureV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "dependency lifecycle refused: {:?}", self.class)
    }
}

impl std::error::Error for LifecycleFailureV1 {}

pub(crate) fn lifecycle_identity(value: String) -> Result<Box<str>, LifecycleFailureV1> {
    if value.is_empty()
        || value.len() > ValidationBoundsV1::MAX_IDENTITY_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(lifecycle_invalid());
    }
    Ok(value.into_boxed_str())
}

pub(crate) const fn lifecycle_invalid() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidFact)
}

pub(crate) const fn lifecycle_bounds() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::BoundsExceeded)
}

pub(crate) const fn lifecycle_internal() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InternalInvariant)
}

pub(crate) fn lifecycle_hash_string(
    hash: &mut CanonicalHasherV1,
    value: &str,
) -> Result<(), LifecycleFailureV1> {
    hash.string(value).map_err(|_| lifecycle_internal())
}

pub(crate) fn lifecycle_len(value: usize) -> Result<u64, LifecycleFailureV1> {
    u64::try_from(value).map_err(|_| lifecycle_bounds())
}
