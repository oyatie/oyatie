/// Versioned publication and maintainer-change delay policy.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyQuarantinePolicyV1 {
    minimum_publication_age_seconds: u64,
    maintainer_change_hold_seconds: u64,
    registry_observation_freshness_seconds: u64,
    policy_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyQuarantinePolicyV1 {
    #[must_use]
    pub fn new(
        minimum_publication_age_seconds: u64,
        maintainer_change_hold_seconds: u64,
        registry_observation_freshness_seconds: u64,
        policy_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.dependency-quarantine-policy.v1\0");
        hash.u64(minimum_publication_age_seconds);
        hash.u64(maintainer_change_hold_seconds);
        hash.u64(registry_observation_freshness_seconds);
        hash.digest(policy_receipt_sha256);
        Self {
            minimum_publication_age_seconds,
            maintainer_change_hold_seconds,
            registry_observation_freshness_seconds,
            policy_receipt_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn minimum_publication_age_seconds(&self) -> u64 {
        self.minimum_publication_age_seconds
    }

    #[must_use]
    pub const fn maintainer_change_hold_seconds(&self) -> u64 {
        self.maintainer_change_hold_seconds
    }

    #[must_use]
    pub const fn registry_observation_freshness_seconds(&self) -> u64 {
        self.registry_observation_freshness_seconds
    }

    #[must_use]
    pub const fn policy_receipt_sha256(&self) -> DigestV1 {
        self.policy_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Provenance coordinates supplied by the Security-owned decision producer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DependencySecurityDecisionEvidenceV1 {
    authority_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    decision_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencySecurityDecisionEvidenceV1 {
    #[must_use]
    pub fn new(
        authority_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        decision_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.dependency-security-decision.v1\0");
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

/// Security-owned, candidate-scoped permission to bypass quarantine delays.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyEmergencySecurityExceptionV1 {
    candidate_identity_sha256: DigestV1,
    policy_identity_sha256: DigestV1,
    advisory_identity_sha256: DigestV1,
    authorized_at: LifecycleTimestampV1,
    expires_at: LifecycleTimestampV1,
    security_decision: DependencySecurityDecisionEvidenceV1,
    identity_sha256: DigestV1,
}

impl DependencyEmergencySecurityExceptionV1 {
    pub fn try_new(
        candidate: &DependencyCandidateV1,
        policy: &DependencyQuarantinePolicyV1,
        advisory_identity_sha256: DigestV1,
        authorized_at: LifecycleTimestampV1,
        expires_at: LifecycleTimestampV1,
        security_decision: DependencySecurityDecisionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let current = candidate.current().evidence().advisories().identities();
        let proposed = candidate.proposed().evidence().advisories().identities();
        if authorized_at > expires_at
            || current.binary_search(&advisory_identity_sha256).is_err()
            || proposed.binary_search(&advisory_identity_sha256).is_ok()
        {
            return Err(invalid_security_exception());
        }
        let candidate_identity_sha256 = candidate.identity_sha256();
        let policy_identity_sha256 = policy.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.dependency-security-exception.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(policy_identity_sha256);
        hash.digest(advisory_identity_sha256);
        hash.u64(authorized_at.unix_seconds());
        hash.u64(expires_at.unix_seconds());
        hash.digest(security_decision.identity_sha256());
        Ok(Self {
            candidate_identity_sha256,
            policy_identity_sha256,
            advisory_identity_sha256,
            authorized_at,
            expires_at,
            security_decision,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn policy_identity_sha256(&self) -> DigestV1 {
        self.policy_identity_sha256
    }

    #[must_use]
    pub const fn advisory_identity_sha256(&self) -> DigestV1 {
        self.advisory_identity_sha256
    }

    #[must_use]
    pub const fn authorized_at(&self) -> LifecycleTimestampV1 {
        self.authorized_at
    }

    #[must_use]
    pub const fn expires_at(&self) -> LifecycleTimestampV1 {
        self.expires_at
    }

    #[must_use]
    pub const fn security_decision(&self) -> DependencySecurityDecisionEvidenceV1 {
        self.security_decision
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

pub(crate) const fn dependency_analysis_mismatch() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::DependencyAnalysisMismatch)
}

const fn invalid_security_exception() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidSecurityException)
}
