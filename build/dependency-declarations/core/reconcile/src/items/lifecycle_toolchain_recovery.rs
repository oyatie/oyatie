/// Nonbinding stable-only recovery between exact admitted matrices.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainStableRecoveryCandidateV1 {
    current: ToolchainMatrixV1,
    proposed: ToolchainMatrixV1,
    delta: ToolchainCandidateDeltaV1,
    evidence: ToolchainStableRecoveryEvidenceV1,
    evaluated_at: LifecycleTimestampV1,
    discovery_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainStableRecoveryCandidateV1 {
    pub fn try_new(
        current: ToolchainMatrixV1,
        proposed: ToolchainMatrixV1,
        evidence: ToolchainStableRecoveryEvidenceV1,
        evaluated_at: LifecycleTimestampV1,
        discovery_receipt_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if matrix_host(&current) != matrix_host(&proposed) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainTargetMismatch,
            ));
        }
        let delta = ToolchainCandidateDeltaV1::between(&current, &proposed);
        let current_safety = evidence.current_safety();
        let proposed_safety = evidence.proposed_safety();
        let retained_stable = evidence.retained_stable();
        let incident = evidence.incident();
        if delta.changed_roles() != [ToolchainRoleV1::QualifiedStableExecution]
            || proposed.stable().version() >= current.stable().version()
            || current_safety.status() != ToolchainSafetyPostureStatusV1::Blocked
            || proposed_safety.status()
                != ToolchainSafetyPostureStatusV1::NoKnownBlockingDefect
            || retained_stable.superseded_at() > incident.opened_at()
        {
            return Err(invalid_toolchain_recovery());
        }
        if current_safety.profile_identity_sha256() != current.stable().identity_sha256()
            || proposed_safety.profile_identity_sha256() != proposed.stable().identity_sha256()
            || retained_stable.profile().identity_sha256()
                != proposed.stable().identity_sha256()
            || incident.blocked_posture_identity_sha256() != current_safety.identity_sha256()
        {
            return Err(toolchain_analysis_mismatch());
        }
        if proposed_safety.evaluated_at() < incident.opened_at()
            || proposed_safety.evaluated_at() > evaluated_at
        {
            return Err(invalid_toolchain_recovery());
        }
        if evaluated_at < incident.opened_at() || evaluated_at > incident.expires_at() {
            return Err(LifecycleFailureV1::new(LifecycleFailureClassV1::StaleFact));
        }
        current_safety.evidence().require_safe(evaluated_at)?;
        proposed_safety.evidence().require_safe(evaluated_at)?;
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-stable-recovery-candidate.v1\0");
        hash.digest(current.identity_sha256());
        hash.digest(proposed.identity_sha256());
        delta.encode(&mut hash)?;
        hash.digest(evidence.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        hash.digest(discovery_receipt_sha256);
        Ok(Self {
            current,
            proposed,
            delta,
            evidence,
            evaluated_at,
            discovery_receipt_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn current(&self) -> &ToolchainMatrixV1 {
        &self.current
    }

    #[must_use]
    pub const fn proposed(&self) -> &ToolchainMatrixV1 {
        &self.proposed
    }

    #[must_use]
    pub const fn delta(&self) -> &ToolchainCandidateDeltaV1 {
        &self.delta
    }

    #[must_use]
    pub const fn current_safety(&self) -> &ToolchainSafetyPostureV1 {
        self.evidence.current_safety()
    }

    #[must_use]
    pub const fn proposed_safety(&self) -> &ToolchainSafetyPostureV1 {
        self.evidence.proposed_safety()
    }

    #[must_use]
    pub const fn retained_stable(&self) -> &RetainedStableProfileV1 {
        self.evidence.retained_stable()
    }

    #[must_use]
    pub const fn incident(&self) -> &ToolchainRecoveryIncidentV1 {
        self.evidence.incident()
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> LifecycleTimestampV1 {
        self.evaluated_at
    }

    #[must_use]
    pub const fn discovery_receipt_sha256(&self) -> DigestV1 {
        self.discovery_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

const fn invalid_toolchain_recovery() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidToolchainRecovery)
}
