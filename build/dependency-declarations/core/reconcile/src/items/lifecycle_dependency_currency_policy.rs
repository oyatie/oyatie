/// Versioned maximum-lag and observation-freshness policy for dependencies.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCurrencyPolicyV1 {
    maximum_adoption_lag_seconds: u64,
    registry_observation_freshness_seconds: u64,
    maximum_exception_duration_seconds: u64,
    policy_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyCurrencyPolicyV1 {
    #[must_use]
    pub fn new(
        maximum_adoption_lag_seconds: u64,
        registry_observation_freshness_seconds: u64,
        maximum_exception_duration_seconds: u64,
        policy_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.dependency-currency-policy.v1\0");
        hash.u64(maximum_adoption_lag_seconds);
        hash.u64(registry_observation_freshness_seconds);
        hash.u64(maximum_exception_duration_seconds);
        hash.digest(policy_receipt_sha256);
        Self {
            maximum_adoption_lag_seconds,
            registry_observation_freshness_seconds,
            maximum_exception_duration_seconds,
            policy_receipt_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn maximum_adoption_lag_seconds(&self) -> u64 {
        self.maximum_adoption_lag_seconds
    }

    #[must_use]
    pub const fn registry_observation_freshness_seconds(&self) -> u64 {
        self.registry_observation_freshness_seconds
    }

    #[must_use]
    pub const fn maximum_exception_duration_seconds(&self) -> u64 {
        self.maximum_exception_duration_seconds
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

/// Provenance coordinates supplied by the dependency-currency decision owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCurrencyDecisionEvidenceV1 {
    authority_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    decision_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyCurrencyDecisionEvidenceV1 {
    #[must_use]
    pub fn new(
        authority_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        decision_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.dependency-currency-decision.v1\0");
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

/// Candidate-scoped, expiring acknowledgement of dependency adoption lag.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCurrencyExceptionV1 {
    candidate_identity_sha256: DigestV1,
    policy_identity_sha256: DigestV1,
    authorized_at: LifecycleTimestampV1,
    expires_at: LifecycleTimestampV1,
    currency_decision: DependencyCurrencyDecisionEvidenceV1,
    identity_sha256: DigestV1,
}

impl DependencyCurrencyExceptionV1 {
    pub fn try_new(
        candidate: &DependencyCandidateV1,
        policy: &DependencyCurrencyPolicyV1,
        authorized_at: LifecycleTimestampV1,
        expires_at: LifecycleTimestampV1,
        currency_decision: DependencyCurrencyDecisionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let Some(duration_seconds) = expires_at
            .unix_seconds()
            .checked_sub(authorized_at.unix_seconds())
        else {
            return Err(invalid_currency_exception());
        };
        if duration_seconds > policy.maximum_exception_duration_seconds() {
            return Err(invalid_currency_exception());
        }
        let candidate_identity_sha256 = candidate.identity_sha256();
        let policy_identity_sha256 = policy.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.dependency-currency-exception.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(policy_identity_sha256);
        hash.u64(authorized_at.unix_seconds());
        hash.u64(expires_at.unix_seconds());
        hash.digest(currency_decision.identity_sha256());
        Ok(Self {
            candidate_identity_sha256,
            policy_identity_sha256,
            authorized_at,
            expires_at,
            currency_decision,
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
    pub const fn authorized_at(&self) -> LifecycleTimestampV1 {
        self.authorized_at
    }

    #[must_use]
    pub const fn expires_at(&self) -> LifecycleTimestampV1 {
        self.expires_at
    }

    #[must_use]
    pub const fn currency_decision(&self) -> DependencyCurrencyDecisionEvidenceV1 {
        self.currency_decision
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

const fn invalid_currency_exception() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidCurrencyException)
}
