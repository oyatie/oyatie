/// Temporal state of a supplied dependency-currency exception.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DependencyCurrencyExceptionStateV1 {
    Absent = 0,
    NotYetValid = 1,
    Active = 2,
    Expired = 3,
}

/// Exact dependency adoption-lag status at one evaluation time.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyCurrencyStatusV1 {
    WithinTarget {
        due_at: LifecycleTimestampV1,
    },
    Overdue {
        due_at: LifecycleTimestampV1,
    },
    OverdueExcepted {
        due_at: LifecycleTimestampV1,
        exception_identity_sha256: DigestV1,
    },
}

impl DependencyCurrencyStatusV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::WithinTarget { due_at } => {
                hash.tag(0);
                hash.u64(due_at.unix_seconds());
            }
            Self::Overdue { due_at } => {
                hash.tag(1);
                hash.u64(due_at.unix_seconds());
            }
            Self::OverdueExcepted {
                due_at,
                exception_identity_sha256,
            } => {
                hash.tag(2);
                hash.u64(due_at.unix_seconds());
                hash.digest(exception_identity_sha256);
            }
        }
    }
}

/// Mechanical lag assessment; it neither qualifies nor accepts a dependency.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCurrencyAssessmentV1 {
    candidate_identity_sha256: DigestV1,
    policy: DependencyCurrencyPolicyV1,
    evaluated_at: LifecycleTimestampV1,
    lag_seconds: u64,
    status: DependencyCurrencyStatusV1,
    exception: Option<DependencyCurrencyExceptionV1>,
    exception_state: DependencyCurrencyExceptionStateV1,
    identity_sha256: DigestV1,
}

impl DependencyCurrencyAssessmentV1 {
    pub fn try_evaluate(
        candidate: &DependencyCandidateV1,
        policy: &DependencyCurrencyPolicyV1,
        exception: Option<&DependencyCurrencyExceptionV1>,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        for publication in [
            candidate.current().publication(),
            candidate.proposed().publication(),
        ] {
            let fresh_until = checked_lifecycle_timestamp_add(
                publication.observed_at(),
                policy.registry_observation_freshness_seconds(),
            )?;
            if evaluated_at < publication.observed_at() || evaluated_at > fresh_until {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::StaleFact,
                ));
            }
        }
        if exception.is_some_and(|exception| {
            exception.candidate_identity_sha256() != candidate.identity_sha256()
                || exception.policy_identity_sha256() != policy.identity_sha256()
        }) {
            return Err(dependency_analysis_mismatch());
        }

        let publication = candidate.proposed().publication();
        let due_at = checked_lifecycle_timestamp_add(
            publication.published_at(),
            policy.maximum_adoption_lag_seconds(),
        )?;
        let lag_seconds = evaluated_at
            .unix_seconds()
            .checked_sub(publication.published_at().unix_seconds())
            .ok_or_else(lifecycle_bounds)?;
        let exception_state = currency_exception_state(exception, evaluated_at);
        let active_exception = exception
            .filter(|_| exception_state == DependencyCurrencyExceptionStateV1::Active)
            .map(DependencyCurrencyExceptionV1::identity_sha256);
        let status = if evaluated_at <= due_at {
            DependencyCurrencyStatusV1::WithinTarget { due_at }
        } else if let Some(exception_identity_sha256) = active_exception {
            DependencyCurrencyStatusV1::OverdueExcepted {
                due_at,
                exception_identity_sha256,
            }
        } else {
            DependencyCurrencyStatusV1::Overdue { due_at }
        };

        let candidate_identity_sha256 = candidate.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.dependency-currency-assessment.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(policy.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        hash.u64(lag_seconds);
        status.encode(&mut hash);
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
            policy: policy.clone(),
            evaluated_at,
            lag_seconds,
            status,
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
    pub const fn policy(&self) -> &DependencyCurrencyPolicyV1 {
        &self.policy
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> LifecycleTimestampV1 {
        self.evaluated_at
    }

    #[must_use]
    pub const fn lag_seconds(&self) -> u64 {
        self.lag_seconds
    }

    #[must_use]
    pub const fn status(&self) -> DependencyCurrencyStatusV1 {
        self.status
    }

    #[must_use]
    pub const fn exception(&self) -> Option<&DependencyCurrencyExceptionV1> {
        self.exception.as_ref()
    }

    #[must_use]
    pub const fn exception_state(&self) -> DependencyCurrencyExceptionStateV1 {
        self.exception_state
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn currency_exception_state(
    exception: Option<&DependencyCurrencyExceptionV1>,
    evaluated_at: LifecycleTimestampV1,
) -> DependencyCurrencyExceptionStateV1 {
    let Some(exception) = exception else {
        return DependencyCurrencyExceptionStateV1::Absent;
    };
    match lifecycle_window_state(
        exception.authorized_at(),
        exception.expires_at(),
        evaluated_at,
    ) {
        LifecycleWindowStateV1::NotYetValid => {
            DependencyCurrencyExceptionStateV1::NotYetValid
        }
        LifecycleWindowStateV1::Active => DependencyCurrencyExceptionStateV1::Active,
        LifecycleWindowStateV1::Expired => DependencyCurrencyExceptionStateV1::Expired,
    }
}
