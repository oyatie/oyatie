/// Candidate relation to one exact observed Rust channel head.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ToolchainCurrencyRoleStatusV1 {
    AlreadyOnObservedHead,
    CandidateMatchesObservedHeadWithinTarget,
    CandidateMatchesObservedHeadOverdue,
    CandidateMatchesObservedHeadOverdueExcepted {
        exception_identity_sha256: DigestV1,
    },
    CandidateDoesNotMatchObservedHead,
    CandidateDoesNotMatchObservedHeadExcepted {
        exception_identity_sha256: DigestV1,
    },
}

impl ToolchainCurrencyRoleStatusV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::AlreadyOnObservedHead => hash.tag(0),
            Self::CandidateMatchesObservedHeadWithinTarget => hash.tag(1),
            Self::CandidateMatchesObservedHeadOverdue => hash.tag(2),
            Self::CandidateMatchesObservedHeadOverdueExcepted {
                exception_identity_sha256,
            } => {
                hash.tag(3);
                hash.digest(exception_identity_sha256);
            }
            Self::CandidateDoesNotMatchObservedHead => hash.tag(4),
            Self::CandidateDoesNotMatchObservedHeadExcepted {
                exception_identity_sha256,
            } => {
                hash.tag(5);
                hash.digest(exception_identity_sha256);
            }
        }
    }
}

/// Currency evidence for one changed execution role.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainRoleCurrencyAssessmentV1 {
    role: ToolchainRoleV1,
    current_material_identity_sha256: DigestV1,
    proposed_material_identity_sha256: DigestV1,
    observed_head_identity_sha256: DigestV1,
    observed_head_version: RustVersionV1,
    lag_seconds: u64,
    due_at: LifecycleTimestampV1,
    status: ToolchainCurrencyRoleStatusV1,
    identity_sha256: DigestV1,
}

impl ToolchainRoleCurrencyAssessmentV1 {
    pub(crate) fn evaluate(
        role: ToolchainRoleV1,
        current: &ToolchainProfileV1,
        proposed: &ToolchainProfileV1,
        head: &ToolchainChannelHeadV1,
        target_seconds: u64,
        evaluated_at: LifecycleTimestampV1,
        active_exception: Option<DigestV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        if head.version() < current.version() || head.version() < proposed.version() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnsupportedVersionRelation,
            ));
        }
        let due_at = checked_lifecycle_timestamp_add(head.released_at(), target_seconds)?;
        let lag_seconds = evaluated_at
            .unix_seconds()
            .checked_sub(head.released_at().unix_seconds())
            .ok_or_else(lifecycle_bounds)?;
        let current_material_identity_sha256 = current.material_identity_sha256();
        let proposed_material_identity_sha256 = proposed.material_identity_sha256();
        let observed_material_identity_sha256 = head.material_identity_sha256();
        let status = if current_material_identity_sha256 == observed_material_identity_sha256
            && proposed_material_identity_sha256 == observed_material_identity_sha256
        {
            ToolchainCurrencyRoleStatusV1::AlreadyOnObservedHead
        } else if proposed_material_identity_sha256 == observed_material_identity_sha256 {
            if evaluated_at <= due_at {
                ToolchainCurrencyRoleStatusV1::CandidateMatchesObservedHeadWithinTarget
            } else if let Some(exception_identity_sha256) = active_exception {
                ToolchainCurrencyRoleStatusV1::CandidateMatchesObservedHeadOverdueExcepted {
                    exception_identity_sha256,
                }
            } else {
                ToolchainCurrencyRoleStatusV1::CandidateMatchesObservedHeadOverdue
            }
        } else if let Some(exception_identity_sha256) = active_exception {
            ToolchainCurrencyRoleStatusV1::CandidateDoesNotMatchObservedHeadExcepted {
                exception_identity_sha256,
            }
        } else {
            ToolchainCurrencyRoleStatusV1::CandidateDoesNotMatchObservedHead
        };
        let observed_head_identity_sha256 = head.identity_sha256();
        let observed_head_version = head.version();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-role-currency.v1\0");
        hash.tag(role as u8);
        hash.digest(current_material_identity_sha256);
        hash.digest(proposed_material_identity_sha256);
        hash.digest(observed_head_identity_sha256);
        observed_head_version.encode(&mut hash);
        hash.u64(lag_seconds);
        hash.u64(due_at.unix_seconds());
        status.encode(&mut hash);
        Ok(Self {
            role,
            current_material_identity_sha256,
            proposed_material_identity_sha256,
            observed_head_identity_sha256,
            observed_head_version,
            lag_seconds,
            due_at,
            status,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn role(&self) -> ToolchainRoleV1 {
        self.role
    }

    #[must_use]
    pub const fn current_material_identity_sha256(&self) -> DigestV1 {
        self.current_material_identity_sha256
    }

    #[must_use]
    pub const fn proposed_material_identity_sha256(&self) -> DigestV1 {
        self.proposed_material_identity_sha256
    }

    #[must_use]
    pub const fn observed_head_identity_sha256(&self) -> DigestV1 {
        self.observed_head_identity_sha256
    }

    #[must_use]
    pub const fn observed_head_version(&self) -> RustVersionV1 {
        self.observed_head_version
    }

    #[must_use]
    pub const fn lag_seconds(&self) -> u64 {
        self.lag_seconds
    }

    #[must_use]
    pub const fn due_at(&self) -> LifecycleTimestampV1 {
        self.due_at
    }

    #[must_use]
    pub const fn status(&self) -> ToolchainCurrencyRoleStatusV1 {
        self.status
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
