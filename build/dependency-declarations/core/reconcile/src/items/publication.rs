/// Qualified local-filesystem publication profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PublisherProfileV1 {
    LinuxExt4V1 = 0,
    LinuxXfsV1 = 1,
    MacosApfsV1 = 2,
}

/// Optional publication intent excluded from generation identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PublicationIntentV1 {
    pub(crate) expected_preimage: Option<DigestV1>,
    pub(crate) publisher: PublisherProfileV1,
}

impl PublicationIntentV1 {
    /// Creates an explicit destination precondition and profile.
    #[must_use]
    pub const fn new(expected_preimage: Option<DigestV1>, publisher: PublisherProfileV1) -> Self {
        Self {
            expected_preimage,
            publisher,
        }
    }
}

/// Generation request plus optional publication intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconciliationRequestV1 {
    pub(crate) generation: GenerationRequestV1,
    pub(crate) publish: Option<PublicationIntentV1>,
}

impl ReconciliationRequestV1 {
    /// Creates a check-only or publishing transaction request.
    #[must_use]
    pub const fn new(
        generation: GenerationRequestV1,
        publish: Option<PublicationIntentV1>,
    ) -> Self {
        Self {
            generation,
            publish,
        }
    }
}

/// Core-validated value passed to the publication port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationRequestV1 {
    generation: ValidatedGenerationV1,
    intent: PublicationIntentV1,
}

impl PublicationRequestV1 {
    pub(crate) const fn new(
        generation: ValidatedGenerationV1,
        intent: PublicationIntentV1,
    ) -> Self {
        Self { generation, intent }
    }

    /// Returns the validated bytes and identities to publish.
    #[must_use]
    pub const fn generation(&self) -> &ValidatedGenerationV1 {
        &self.generation
    }

    /// Returns the destination precondition and qualified profile.
    #[must_use]
    pub const fn intent(&self) -> &PublicationIntentV1 {
        &self.intent
    }

    pub(crate) fn into_parts(self) -> (ValidatedGenerationV1, PublicationIntentV1) {
        (self.generation, self.intent)
    }
}

/// Whether replacement is proven not to have happened or may have happened.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReplacementStateV1 {
    No,
    Maybe,
}

/// Durability knowledge after an indeterminate publication.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DurabilityStateV1 {
    Unknown,
}

/// Complete outcome of one invoked publication attempt.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PublicationOutcomeV1 {
    Unchanged,
    Replaced,
    Failed {
        failure: FailureV1,
        replacement: ReplacementStateV1,
    },
    Indeterminate {
        failure: FailureV1,
        replacement: ReplacementStateV1,
        durability: DurabilityStateV1,
    },
}

/// Observation returned for every invoked publication attempt.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PublicationObservationV1 {
    pub(crate) outcome: PublicationOutcomeV1,
}

impl PublicationObservationV1 {
    /// Wraps an adapter observation for core validation.
    #[must_use]
    pub const fn new(outcome: PublicationOutcomeV1) -> Self {
        Self { outcome }
    }
}

/// Identity-bearing receipt for one publication attempt.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PublicationAttemptReceiptV1 {
    pub(crate) attempt_id: DigestV1,
    pub(crate) generation_id: DigestV1,
    pub(crate) expected_preimage: Option<DigestV1>,
    pub(crate) publisher: PublisherProfileV1,
    pub(crate) outcome: PublicationOutcomeV1,
}

impl PublicationAttemptReceiptV1 {
    /// Returns the stable identity of this exact attempt and outcome.
    #[must_use]
    pub const fn attempt_id(&self) -> DigestV1 {
        self.attempt_id
    }

    /// Returns the generation this attempt tried to publish.
    #[must_use]
    pub const fn generation_id(&self) -> DigestV1 {
        self.generation_id
    }

    /// Returns the exact observed outcome.
    #[must_use]
    pub const fn outcome(&self) -> &PublicationOutcomeV1 {
        &self.outcome
    }
}

/// Pure transaction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationResultV1 {
    Refused {
        request_id: Option<DigestV1>,
        failure: FailureV1,
    },
    Generated {
        generation: ValidatedGenerationV1,
    },
    Published {
        generation: ValidatedGenerationV1,
        attempt: PublicationAttemptReceiptV1,
    },
}

/// Declarative reconciliation phase.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ReconciliationPhaseV1 {
    Pending = 0,
    Running = 1,
    Succeeded = 2,
    Failed = 3,
    Indeterminate = 4,
}

/// Publication adapters report every invoked outcome instead of losing it as an error.
pub type PublicationPortErrorV1 = std::convert::Infallible;
