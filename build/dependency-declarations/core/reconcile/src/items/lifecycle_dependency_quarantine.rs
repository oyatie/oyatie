/// State of one independent quarantine delay.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyQuarantineGateV1 {
    NotRequired,
    Satisfied {
        eligible_at: LifecycleTimestampV1,
    },
    Held {
        eligible_at: LifecycleTimestampV1,
    },
    Bypassed {
        eligible_at: LifecycleTimestampV1,
        exception_identity_sha256: DigestV1,
    },
}

impl DependencyQuarantineGateV1 {
    fn evaluate(
        eligible_at: LifecycleTimestampV1,
        evaluated_at: LifecycleTimestampV1,
        active_exception: Option<DigestV1>,
    ) -> Self {
        if evaluated_at >= eligible_at {
            Self::Satisfied { eligible_at }
        } else if let Some(exception_identity_sha256) = active_exception {
            Self::Bypassed {
                eligible_at,
                exception_identity_sha256,
            }
        } else {
            Self::Held { eligible_at }
        }
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::NotRequired => hash.tag(0),
            Self::Satisfied { eligible_at } => {
                hash.tag(1);
                hash.u64(eligible_at.unix_seconds());
            }
            Self::Held { eligible_at } => {
                hash.tag(2);
                hash.u64(eligible_at.unix_seconds());
            }
            Self::Bypassed {
                eligible_at,
                exception_identity_sha256,
            } => {
                hash.tag(3);
                hash.u64(eligible_at.unix_seconds());
                hash.digest(exception_identity_sha256);
            }
        }
    }

    pub(crate) const fn is_held(self) -> bool {
        matches!(self, Self::Held { .. })
    }
}

/// Temporal state of a supplied Security-owned exception.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DependencySecurityExceptionStateV1 {
    Absent = 0,
    NotYetValid = 1,
    Active = 2,
    Expired = 3,
}

/// Mechanical quarantine evaluation; it is not dependency acceptance.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyQuarantineV1 {
    candidate_identity_sha256: DigestV1,
    policy: DependencyQuarantinePolicyV1,
    evaluated_at: LifecycleTimestampV1,
    publication_age: DependencyQuarantineGateV1,
    maintainer_change: DependencyQuarantineGateV1,
    security_exception: Option<DependencyEmergencySecurityExceptionV1>,
    security_exception_state: DependencySecurityExceptionStateV1,
    identity_sha256: DigestV1,
}

impl DependencyQuarantineV1 {
    pub fn try_evaluate(
        candidate: &DependencyCandidateV1,
        policy: &DependencyQuarantinePolicyV1,
        security_exception: Option<&DependencyEmergencySecurityExceptionV1>,
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
        if security_exception.is_some_and(|exception| {
            exception.candidate_identity_sha256() != candidate.identity_sha256()
                || exception.policy_identity_sha256() != policy.identity_sha256()
        }) {
            return Err(dependency_analysis_mismatch());
        }

        let publication = candidate.proposed().publication();
        let security_exception_state = exception_state(security_exception, evaluated_at);
        let active_exception = security_exception
            .filter(|_| security_exception_state == DependencySecurityExceptionStateV1::Active)
            .map(DependencyEmergencySecurityExceptionV1::identity_sha256);
        let publication_eligible_at = checked_lifecycle_timestamp_add(
            publication.published_at(),
            policy.minimum_publication_age_seconds(),
        )?;
        let publication_age = DependencyQuarantineGateV1::evaluate(
            publication_eligible_at,
            evaluated_at,
            active_exception,
        );
        let maintainer_change = if candidate
            .delta()
            .changed(DependencyChangeAxisV1::Maintainers)
        {
            // The exact observation is a conservative change-time upper bound.
            let eligible_at = checked_lifecycle_timestamp_add(
                publication.observed_at(),
                policy.maintainer_change_hold_seconds(),
            )?;
            DependencyQuarantineGateV1::evaluate(
                eligible_at,
                evaluated_at,
                active_exception,
            )
        } else {
            DependencyQuarantineGateV1::NotRequired
        };

        let candidate_identity_sha256 = candidate.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.dependency-quarantine.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(policy.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        publication_age.encode(&mut hash);
        maintainer_change.encode(&mut hash);
        hash.tag(security_exception_state as u8);
        match security_exception {
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
            publication_age,
            maintainer_change,
            security_exception: security_exception.cloned(),
            security_exception_state,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn policy(&self) -> &DependencyQuarantinePolicyV1 {
        &self.policy
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> LifecycleTimestampV1 {
        self.evaluated_at
    }

    #[must_use]
    pub const fn publication_age(&self) -> DependencyQuarantineGateV1 {
        self.publication_age
    }

    #[must_use]
    pub const fn maintainer_change(&self) -> DependencyQuarantineGateV1 {
        self.maintainer_change
    }

    #[must_use]
    pub const fn security_exception(
        &self,
    ) -> Option<&DependencyEmergencySecurityExceptionV1> {
        self.security_exception.as_ref()
    }

    #[must_use]
    pub const fn security_exception_state(&self) -> DependencySecurityExceptionStateV1 {
        self.security_exception_state
    }

    #[must_use]
    pub fn active_security_exception_identity_sha256(&self) -> Option<DigestV1> {
        self.security_exception
            .as_ref()
            .filter(|_| {
                self.security_exception_state == DependencySecurityExceptionStateV1::Active
            })
            .map(DependencyEmergencySecurityExceptionV1::identity_sha256)
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn exception_state(
    security_exception: Option<&DependencyEmergencySecurityExceptionV1>,
    evaluated_at: LifecycleTimestampV1,
) -> DependencySecurityExceptionStateV1 {
    let Some(exception) = security_exception else {
        return DependencySecurityExceptionStateV1::Absent;
    };
    match lifecycle_window_state(
        exception.authorized_at(),
        exception.expires_at(),
        evaluated_at,
    ) {
        LifecycleWindowStateV1::NotYetValid => {
            DependencySecurityExceptionStateV1::NotYetValid
        }
        LifecycleWindowStateV1::Active => DependencySecurityExceptionStateV1::Active,
        LifecycleWindowStateV1::Expired => DependencySecurityExceptionStateV1::Expired,
    }
}
