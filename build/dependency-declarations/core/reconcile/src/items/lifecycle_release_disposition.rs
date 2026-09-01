/// Owner disposition of one release item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ReleaseDecisionV1 {
    Adopt = 0,
    Benchmark = 1,
    Defer = 2,
    Reject = 3,
}

/// Deterministic condition that forces disposition re-evaluation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ReevaluationTriggerV1 {
    OnStableRelease = 0,
    OnConsumerObserved = 1,
    OnUpstreamChange = 2,
    BeforeCampaign = 3,
    OnEvidenceChange = 4,
}

/// Explicit effect of one release item on the declared Rust floor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReleaseMsrvEffectV1 {
    NoChange {
        evidence_sha256: DigestV1,
    },
    RequiresAtLeast {
        version: RustVersionV1,
        evidence_sha256: DigestV1,
    },
    Unknown {
        evidence_sha256: DigestV1,
    },
}

impl ReleaseMsrvEffectV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::NoChange { evidence_sha256 } => {
                hash.tag(0);
                hash.digest(evidence_sha256);
            }
            Self::RequiresAtLeast {
                version,
                evidence_sha256,
            } => {
                hash.tag(1);
                version.encode(hash);
                hash.digest(evidence_sha256);
            }
            Self::Unknown { evidence_sha256 } => {
                hash.tag(2);
                hash.digest(evidence_sha256);
            }
        }
    }
}

/// Bounded canonical summary of units affected by one release item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseAffectedUnitsV1 {
    unit_count: u64,
    encoded_bytes: u64,
    sha256: DigestV1,
}

impl ReleaseAffectedUnitsV1 {
    pub fn try_new(
        unit_count: u64,
        encoded_bytes: u64,
        sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if unit_count > LifecycleBoundsV1::MAX_AFFECTED_UNITS
            || encoded_bytes > LifecycleBoundsV1::MAX_AFFECTED_UNIT_BYTES
            || (unit_count == 0) != (encoded_bytes == 0)
        {
            return Err(lifecycle_bounds());
        }
        Ok(Self {
            unit_count,
            encoded_bytes,
            sha256,
        })
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        hash.u64(self.unit_count);
        hash.u64(self.encoded_bytes);
        hash.digest(self.sha256);
    }

    #[must_use]
    pub const fn unit_count(self) -> u64 {
        self.unit_count
    }

    #[must_use]
    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }

    #[must_use]
    pub const fn sha256(self) -> DigestV1 {
        self.sha256
    }
}

/// Evidence required for an owner disposition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseDispositionEvidenceV1 {
    rationale_sha256: DigestV1,
    affected_units: ReleaseAffectedUnitsV1,
    msrv_effect: ReleaseMsrvEffectV1,
    evidence_sha256: DigestV1,
    trigger: ReevaluationTriggerV1,
}

impl ReleaseDispositionEvidenceV1 {
    #[must_use]
    pub const fn new(
        rationale_sha256: DigestV1,
        affected_units: ReleaseAffectedUnitsV1,
        msrv_effect: ReleaseMsrvEffectV1,
        evidence_sha256: DigestV1,
        trigger: ReevaluationTriggerV1,
    ) -> Self {
        Self {
            rationale_sha256,
            affected_units,
            msrv_effect,
            evidence_sha256,
            trigger,
        }
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        hash.digest(self.rationale_sha256);
        self.affected_units.encode(hash);
        self.msrv_effect.encode(hash);
        hash.digest(self.evidence_sha256);
        hash.tag(self.trigger as u8);
    }

    #[must_use]
    pub const fn msrv_effect(self) -> ReleaseMsrvEffectV1 {
        self.msrv_effect
    }

    #[must_use]
    pub const fn rationale_sha256(self) -> DigestV1 {
        self.rationale_sha256
    }

    #[must_use]
    pub const fn affected_units(self) -> ReleaseAffectedUnitsV1 {
        self.affected_units
    }

    #[must_use]
    pub const fn evidence_sha256(self) -> DigestV1 {
        self.evidence_sha256
    }

    #[must_use]
    pub const fn trigger(self) -> ReevaluationTriggerV1 {
        self.trigger
    }
}

/// One nonbinding, owner-attributed release-item disposition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseDispositionV1 {
    item_identity: DigestV1,
    owner: Box<str>,
    decision: ReleaseDecisionV1,
    evidence: ReleaseDispositionEvidenceV1,
    identity_sha256: DigestV1,
}

impl ReleaseDispositionV1 {
    pub fn try_new(
        item_identity: DigestV1,
        owner: impl Into<String>,
        decision: ReleaseDecisionV1,
        evidence: ReleaseDispositionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if decision == ReleaseDecisionV1::Adopt
            && matches!(evidence.msrv_effect(), ReleaseMsrvEffectV1::Unknown { .. })
        {
            return Err(lifecycle_invalid());
        }
        let mut value = Self {
            item_identity,
            owner: lifecycle_identity(owner.into())?,
            decision,
            evidence,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.release-disposition.v1\0");
        value.encode_fields(&mut hash)?;
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        hash.digest(self.item_identity);
        lifecycle_hash_string(hash, &self.owner)?;
        hash.tag(self.decision as u8);
        self.evidence.encode(hash);
        Ok(())
    }

    #[must_use]
    pub const fn item_identity(&self) -> DigestV1 {
        self.item_identity
    }

    #[must_use]
    pub const fn decision(&self) -> ReleaseDecisionV1 {
        self.decision
    }

    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    pub const fn evidence(&self) -> ReleaseDispositionEvidenceV1 {
        self.evidence
    }
}
