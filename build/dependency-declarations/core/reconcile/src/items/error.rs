/// Stable reconciliation failure classes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum FailureClassV1 {
    InvalidRequest = 0,
    InputChanged = 1,
    MissingFixup = 2,
    GeneratorUnavailable = 3,
    GeneratorFailed = 4,
    GeneratorTimedOut = 5,
    GeneratorOutputTooLarge = 6,
    NondeterministicOutput = 7,
    InvalidGeneratedGraph = 8,
    UnsupportedPublicationProfile = 9,
    DestinationLeaseUnavailable = 10,
    LeaseLost = 11,
    DestinationConflict = 12,
    StageWriteFailed = 13,
    StageSyncFailed = 14,
    ReplaceFailed = 15,
    DirectorySyncFailed = 16,
    InternalInvariant = 17,
    StageCleanupFailed = 18,
}

impl FailureClassV1 {
    pub(crate) const fn tag(self) -> u8 {
        self as u8
    }

    pub(crate) const fn is_publication_attempt_failure(self) -> bool {
        matches!(
            self,
            Self::DestinationLeaseUnavailable
                | Self::LeaseLost
                | Self::DestinationConflict
                | Self::StageWriteFailed
                | Self::StageSyncFailed
                | Self::ReplaceFailed
                | Self::DirectorySyncFailed
                | Self::StageCleanupFailed
                | Self::InternalInvariant
        )
    }
}

/// A stable typed failure without ambient diagnostic text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FailureV1 {
    class: FailureClassV1,
}

impl FailureV1 {
    /// Creates a failure of the supplied stable class.
    #[must_use]
    pub const fn new(class: FailureClassV1) -> Self {
        Self { class }
    }

    /// Returns the stable class.
    #[must_use]
    pub const fn class(self) -> FailureClassV1 {
        self.class
    }
}

impl std::fmt::Display for FailureV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "declaration reconciliation refused: {:?}",
            self.class
        )
    }
}

impl std::error::Error for FailureV1 {}

pub(crate) const fn invalid_request() -> FailureV1 {
    FailureV1::new(FailureClassV1::InvalidRequest)
}

pub(crate) const fn invalid_graph() -> FailureV1 {
    FailureV1::new(FailureClassV1::InvalidGeneratedGraph)
}

pub(crate) const fn internal_invariant() -> FailureV1 {
    FailureV1::new(FailureClassV1::InternalInvariant)
}

/// Failures returned by the generation effect adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationPortErrorV1 {
    InputChanged,
    MissingFixup,
    GeneratorUnavailable,
    GeneratorFailed,
    GeneratorTimedOut,
    GeneratorOutputTooLarge,
    InternalInvariant,
}

impl GenerationPortErrorV1 {
    pub(crate) const fn failure(self) -> FailureV1 {
        let class = match self {
            Self::InputChanged => FailureClassV1::InputChanged,
            Self::MissingFixup => FailureClassV1::MissingFixup,
            Self::GeneratorUnavailable => FailureClassV1::GeneratorUnavailable,
            Self::GeneratorFailed => FailureClassV1::GeneratorFailed,
            Self::GeneratorTimedOut => FailureClassV1::GeneratorTimedOut,
            Self::GeneratorOutputTooLarge => FailureClassV1::GeneratorOutputTooLarge,
            Self::InternalInvariant => FailureClassV1::InternalInvariant,
        };
        FailureV1::new(class)
    }
}

/// Failures returned by the independent syntax projection adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionPortErrorV1 {
    InvalidSyntax,
    UnsupportedSyntax,
    OutputTooLarge,
    InternalInvariant,
}

impl ProjectionPortErrorV1 {
    pub(crate) const fn failure(self) -> FailureV1 {
        match self {
            Self::InternalInvariant => internal_invariant(),
            Self::InvalidSyntax | Self::UnsupportedSyntax | Self::OutputTooLarge => invalid_graph(),
        }
    }
}
