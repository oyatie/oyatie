/// Temporal state of a supplied toolchain-currency exception.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ToolchainCurrencyExceptionStateV1 {
    Absent = 0,
    NotYetValid = 1,
    Active = 2,
    Expired = 3,
}

/// Mechanical currency assessment; it is not toolchain qualification.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCurrencyAssessmentV1 {
    candidate_identity_sha256: DigestV1,
    snapshot: ToolchainChannelSnapshotV1,
    policy: ToolchainCurrencyPolicyV1,
    evaluated_at: LifecycleTimestampV1,
    changed_execution_roles: Box<[ToolchainRoleCurrencyAssessmentV1]>,
    exception: Option<ToolchainCurrencyExceptionV1>,
    exception_state: ToolchainCurrencyExceptionStateV1,
    identity_sha256: DigestV1,
}

impl ToolchainCurrencyAssessmentV1 {
    pub fn try_evaluate(
        candidate: &ToolchainCandidateV1,
        snapshot: &ToolchainChannelSnapshotV1,
        policy: &ToolchainCurrencyPolicyV1,
        exception: Option<&ToolchainCurrencyExceptionV1>,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_toolchain_currency_observation(snapshot, policy, evaluated_at)?;
        let candidate_identity_sha256 = candidate.identity_sha256();
        if snapshot.host_triple() != candidate.current().stable().tools().rustc().host_triple()
            || exception.is_some_and(|exception| {
                exception.candidate_identity_sha256() != candidate_identity_sha256
                    || exception.policy_identity_sha256() != policy.identity_sha256()
            })
        {
            return Err(toolchain_analysis_mismatch());
        }
        let exception_state = toolchain_currency_exception_state(exception, evaluated_at);
        let active_exception = exception
            .filter(|_| exception_state == ToolchainCurrencyExceptionStateV1::Active);
        let mut changed_execution_roles = Vec::with_capacity(3);
        for role in candidate.changed_roles().iter().copied() {
            let Some(target_seconds) = policy.target_seconds(role) else {
                continue;
            };
            let current = matrix_profile(candidate.current(), role).ok_or_else(lifecycle_internal)?;
            let proposed =
                matrix_profile(candidate.proposed(), role).ok_or_else(lifecycle_internal)?;
            let head = snapshot.head(role).ok_or_else(lifecycle_internal)?;
            let role_exception = active_exception
                .filter(|exception| exception.covers(role))
                .map(ToolchainCurrencyExceptionV1::identity_sha256);
            changed_execution_roles.push(ToolchainRoleCurrencyAssessmentV1::evaluate(
                role,
                current,
                proposed,
                head,
                target_seconds,
                evaluated_at,
                role_exception,
            )?);
        }
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-currency-assessment.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(snapshot.identity_sha256());
        hash.digest(policy.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        hash.u64(lifecycle_len(changed_execution_roles.len())?);
        for assessment in &changed_execution_roles {
            hash.digest(assessment.identity_sha256());
        }
        hash.tag(exception_state as u8);
        match exception {
            None => hash.tag(0),
            Some(exception) => {
                hash.tag(1);
                hash.digest(exception.identity_sha256());
            }
        }
        Ok(Self {
            candidate_identity_sha256,
            snapshot: snapshot.clone(),
            policy: policy.clone(),
            evaluated_at,
            changed_execution_roles: changed_execution_roles.into_boxed_slice(),
            exception: exception.cloned(),
            exception_state,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn snapshot(&self) -> &ToolchainChannelSnapshotV1 {
        &self.snapshot
    }

    #[must_use]
    pub const fn policy(&self) -> &ToolchainCurrencyPolicyV1 {
        &self.policy
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> LifecycleTimestampV1 {
        self.evaluated_at
    }

    #[must_use]
    pub fn changed_execution_roles(&self) -> &[ToolchainRoleCurrencyAssessmentV1] {
        &self.changed_execution_roles
    }

    #[must_use]
    pub const fn exception(&self) -> Option<&ToolchainCurrencyExceptionV1> {
        self.exception.as_ref()
    }

    #[must_use]
    pub const fn exception_state(&self) -> ToolchainCurrencyExceptionStateV1 {
        self.exception_state
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

pub(crate) fn validate_toolchain_currency_observation(
    snapshot: &ToolchainChannelSnapshotV1,
    policy: &ToolchainCurrencyPolicyV1,
    evaluated_at: LifecycleTimestampV1,
) -> Result<(), LifecycleFailureV1> {
    let fresh_until = checked_lifecycle_timestamp_add(
        snapshot.observed_at(),
        policy.observation_freshness_seconds(),
    )?;
    if evaluated_at < snapshot.observed_at() || evaluated_at > fresh_until {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::StaleFact,
        ));
    }
    Ok(())
}

fn matrix_profile(
    matrix: &ToolchainMatrixV1,
    role: ToolchainRoleV1,
) -> Option<&ToolchainProfileV1> {
    match role {
        ToolchainRoleV1::DeclaredMsrvCompatibility => None,
        ToolchainRoleV1::QualifiedStableExecution => Some(matrix.stable()),
        ToolchainRoleV1::BetaShadow => Some(matrix.beta()),
        ToolchainRoleV1::NightlyShadow => Some(matrix.nightly()),
    }
}

fn toolchain_currency_exception_state(
    exception: Option<&ToolchainCurrencyExceptionV1>,
    evaluated_at: LifecycleTimestampV1,
) -> ToolchainCurrencyExceptionStateV1 {
    let Some(exception) = exception else {
        return ToolchainCurrencyExceptionStateV1::Absent;
    };
    match lifecycle_window_state(exception.authorized_at(), exception.expires_at(), evaluated_at) {
        LifecycleWindowStateV1::NotYetValid => ToolchainCurrencyExceptionStateV1::NotYetValid,
        LifecycleWindowStateV1::Active => ToolchainCurrencyExceptionStateV1::Active,
        LifecycleWindowStateV1::Expired => ToolchainCurrencyExceptionStateV1::Expired,
    }
}

const fn toolchain_analysis_mismatch() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::ToolchainAnalysisMismatch)
}
