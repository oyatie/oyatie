/// Provenance supplied by the owner deciding that a compiler defect blocks use.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainDefectDecisionEvidenceV1 {
    authority_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    decision_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainDefectDecisionEvidenceV1 {
    #[must_use]
    pub fn new(
        authority_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        decision_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-defect-decision.v1\0");
        hash.digest(authority_identity_sha256);
        hash.digest(schema_identity_sha256);
        hash.digest(decision_receipt_sha256);
        Self {
            authority_identity_sha256,
            schema_identity_sha256,
            decision_receipt_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn authority_identity_sha256(self) -> DigestV1 {
        self.authority_identity_sha256
    }

    #[must_use]
    pub const fn schema_identity_sha256(self) -> DigestV1 {
        self.schema_identity_sha256
    }

    #[must_use]
    pub const fn decision_receipt_sha256(self) -> DigestV1 {
        self.decision_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Owner-decided blocker joined to one exact compiler-material identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainBlockingDefectV1 {
    affected_material_identity_sha256: DigestV1,
    normalized_advisory_identity_sha256: DigestV1,
    applicability_receipt_sha256: DigestV1,
    decision: ToolchainDefectDecisionEvidenceV1,
    identity_sha256: DigestV1,
}

impl ToolchainBlockingDefectV1 {
    pub fn try_new(
        affected: &ToolchainProfileV1,
        advisory: &NormalizedAdvisoryFactV1,
        applicability_receipt_sha256: DigestV1,
        decision: ToolchainDefectDecisionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if advisory.lifecycle() != NormalizedAdvisoryLifecycleV1::Active {
            return Err(lifecycle_invalid());
        }
        let affected_material_identity_sha256 = affected.material_identity_sha256();
        let normalized_advisory_identity_sha256 = advisory.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-blocking-defect.v1\0");
        hash.digest(affected_material_identity_sha256);
        hash.digest(normalized_advisory_identity_sha256);
        hash.digest(applicability_receipt_sha256);
        hash.digest(decision.identity_sha256());
        Ok(Self {
            affected_material_identity_sha256,
            normalized_advisory_identity_sha256,
            applicability_receipt_sha256,
            decision,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn affected_material_identity_sha256(&self) -> DigestV1 {
        self.affected_material_identity_sha256
    }

    #[must_use]
    pub const fn normalized_advisory_identity_sha256(&self) -> DigestV1 {
        self.normalized_advisory_identity_sha256
    }

    #[must_use]
    pub const fn applicability_receipt_sha256(&self) -> DigestV1 {
        self.applicability_receipt_sha256
    }

    #[must_use]
    pub const fn decision(&self) -> ToolchainDefectDecisionEvidenceV1 {
        self.decision
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Bounded known-defect result; the clear state is not a global safety claim.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ToolchainSafetyPostureStatusV1 {
    NoKnownBlockingDefect = 0,
    Blocked = 1,
}

/// Fresh complete blocker posture for one exact qualified profile.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainSafetyPostureV1 {
    profile_identity_sha256: DigestV1,
    profile_material_identity_sha256: DigestV1,
    evidence: FactEnvelopeV1,
    evaluated_at: LifecycleTimestampV1,
    blocking_defects: Box<[ToolchainBlockingDefectV1]>,
    status: ToolchainSafetyPostureStatusV1,
    identity_sha256: DigestV1,
}

impl ToolchainSafetyPostureV1 {
    pub fn try_evaluate(
        profile: &ToolchainProfileV1,
        mut blocking_defects: Vec<ToolchainBlockingDefectV1>,
        evidence: FactEnvelopeV1,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        evidence.require_safe(evaluated_at)?;
        if blocking_defects.len() > LifecycleBoundsV1::MAX_TOOLCHAIN_BLOCKING_DEFECTS {
            return Err(lifecycle_bounds());
        }
        let profile_material_identity_sha256 = profile.material_identity_sha256();
        if evidence.temporal().scope().toolchain_sha256() != profile_material_identity_sha256
            || blocking_defects.iter().any(|defect| {
                defect.affected_material_identity_sha256() != profile_material_identity_sha256
            })
        {
            return Err(toolchain_analysis_mismatch());
        }
        blocking_defects
            .sort_by_key(ToolchainBlockingDefectV1::normalized_advisory_identity_sha256);
        if blocking_defects.windows(2).any(|pair| {
            pair[0].normalized_advisory_identity_sha256()
                == pair[1].normalized_advisory_identity_sha256()
        }) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let status = if blocking_defects.is_empty() {
            ToolchainSafetyPostureStatusV1::NoKnownBlockingDefect
        } else {
            ToolchainSafetyPostureStatusV1::Blocked
        };
        let profile_identity_sha256 = profile.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-safety-posture.v1\0");
        hash.digest(profile_identity_sha256);
        hash.digest(profile_material_identity_sha256);
        hash.digest(evidence.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        hash.tag(status as u8);
        hash.u64(lifecycle_len(blocking_defects.len())?);
        for defect in &blocking_defects {
            hash.digest(defect.identity_sha256());
        }
        Ok(Self {
            profile_identity_sha256,
            profile_material_identity_sha256,
            evidence,
            evaluated_at,
            blocking_defects: blocking_defects.into_boxed_slice(),
            status,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn profile_identity_sha256(&self) -> DigestV1 {
        self.profile_identity_sha256
    }

    #[must_use]
    pub const fn profile_material_identity_sha256(&self) -> DigestV1 {
        self.profile_material_identity_sha256
    }

    #[must_use]
    pub const fn evidence(&self) -> &FactEnvelopeV1 {
        &self.evidence
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> LifecycleTimestampV1 {
        self.evaluated_at
    }

    #[must_use]
    pub fn blocking_defects(&self) -> &[ToolchainBlockingDefectV1] {
        &self.blocking_defects
    }

    #[must_use]
    pub const fn status(&self) -> ToolchainSafetyPostureStatusV1 {
        self.status
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
