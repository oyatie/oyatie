/// Versioned lag, observation, and exception policy for Rust channels.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCurrencyPolicyV1 {
    stable_adoption_target_seconds: u64,
    beta_refresh_target_seconds: u64,
    nightly_refresh_target_seconds: u64,
    observation_freshness_seconds: u64,
    maximum_exception_duration_seconds: u64,
    policy_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainCurrencyPolicyV1 {
    #[must_use]
    pub fn new(
        stable_adoption_target_seconds: u64,
        beta_refresh_target_seconds: u64,
        nightly_refresh_target_seconds: u64,
        observation_freshness_seconds: u64,
        maximum_exception_duration_seconds: u64,
        policy_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-currency-policy.v1\0");
        hash.u64(stable_adoption_target_seconds);
        hash.u64(beta_refresh_target_seconds);
        hash.u64(nightly_refresh_target_seconds);
        hash.u64(observation_freshness_seconds);
        hash.u64(maximum_exception_duration_seconds);
        hash.digest(policy_receipt_sha256);
        Self {
            stable_adoption_target_seconds,
            beta_refresh_target_seconds,
            nightly_refresh_target_seconds,
            observation_freshness_seconds,
            maximum_exception_duration_seconds,
            policy_receipt_sha256,
            identity_sha256: hash.finish(),
        }
    }

    pub(crate) const fn target_seconds(&self, role: ToolchainRoleV1) -> Option<u64> {
        match role {
            ToolchainRoleV1::DeclaredMsrvCompatibility => None,
            ToolchainRoleV1::QualifiedStableExecution => {
                Some(self.stable_adoption_target_seconds)
            }
            ToolchainRoleV1::BetaShadow => Some(self.beta_refresh_target_seconds),
            ToolchainRoleV1::NightlyShadow => Some(self.nightly_refresh_target_seconds),
        }
    }

    #[must_use]
    pub const fn stable_adoption_target_seconds(&self) -> u64 {
        self.stable_adoption_target_seconds
    }

    #[must_use]
    pub const fn beta_refresh_target_seconds(&self) -> u64 {
        self.beta_refresh_target_seconds
    }

    #[must_use]
    pub const fn nightly_refresh_target_seconds(&self) -> u64 {
        self.nightly_refresh_target_seconds
    }

    #[must_use]
    pub const fn observation_freshness_seconds(&self) -> u64 {
        self.observation_freshness_seconds
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

/// Provenance supplied by the toolchain-currency decision owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCurrencyDecisionEvidenceV1 {
    authority_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    decision_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainCurrencyDecisionEvidenceV1 {
    #[must_use]
    pub fn new(
        authority_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        decision_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-currency-decision.v1\0");
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

/// Candidate and role-scoped acknowledgement of a toolchain currency gap.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCurrencyExceptionV1 {
    candidate_identity_sha256: DigestV1,
    policy_identity_sha256: DigestV1,
    roles: Box<[ToolchainRoleV1]>,
    authorized_at: LifecycleTimestampV1,
    expires_at: LifecycleTimestampV1,
    decision: ToolchainCurrencyDecisionEvidenceV1,
    identity_sha256: DigestV1,
}

impl ToolchainCurrencyExceptionV1 {
    pub fn try_new(
        candidate: &ToolchainCandidateV1,
        policy: &ToolchainCurrencyPolicyV1,
        mut roles: Vec<ToolchainRoleV1>,
        authorized_at: LifecycleTimestampV1,
        expires_at: LifecycleTimestampV1,
        decision: ToolchainCurrencyDecisionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let Some(duration_seconds) = expires_at
            .unix_seconds()
            .checked_sub(authorized_at.unix_seconds())
        else {
            return Err(invalid_toolchain_currency_exception());
        };
        roles.sort_by_key(|role| *role as u8);
        if roles.is_empty()
            || roles.len() > 3
            || roles.windows(2).any(|pair| pair[0] == pair[1])
            || roles.iter().any(|role| {
                *role == ToolchainRoleV1::DeclaredMsrvCompatibility
                    || !candidate.changed_roles().contains(role)
            })
            || duration_seconds > policy.maximum_exception_duration_seconds()
        {
            return Err(invalid_toolchain_currency_exception());
        }
        let candidate_identity_sha256 = candidate.identity_sha256();
        let policy_identity_sha256 = policy.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-currency-exception.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(policy_identity_sha256);
        hash.u64(lifecycle_len(roles.len())?);
        for role in &roles {
            hash.tag(*role as u8);
        }
        hash.u64(authorized_at.unix_seconds());
        hash.u64(expires_at.unix_seconds());
        hash.digest(decision.identity_sha256());
        Ok(Self {
            candidate_identity_sha256,
            policy_identity_sha256,
            roles: roles.into_boxed_slice(),
            authorized_at,
            expires_at,
            decision,
            identity_sha256: hash.finish(),
        })
    }

    pub(crate) fn covers(&self, role: ToolchainRoleV1) -> bool {
        self.roles.contains(&role)
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
    pub fn roles(&self) -> &[ToolchainRoleV1] {
        &self.roles
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
    pub const fn decision(&self) -> ToolchainCurrencyDecisionEvidenceV1 {
        self.decision
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

const fn invalid_toolchain_currency_exception() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidToolchainCurrencyException)
}
