/// Duration ceiling for one nonbinding stable-recovery incident.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainStableRecoveryPolicyV1 {
    maximum_incident_duration_seconds: u64,
    policy_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainStableRecoveryPolicyV1 {
    #[must_use]
    pub fn new(
        maximum_incident_duration_seconds: u64,
        policy_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-stable-recovery-policy.v1\0");
        hash.u64(maximum_incident_duration_seconds);
        hash.digest(policy_receipt_sha256);
        Self {
            maximum_incident_duration_seconds,
            policy_receipt_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn maximum_incident_duration_seconds(&self) -> u64 {
        self.maximum_incident_duration_seconds
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

/// Provenance supplied by the incident decision owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainRecoveryDecisionEvidenceV1 {
    authority_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    decision_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainRecoveryDecisionEvidenceV1 {
    #[must_use]
    pub fn new(
        authority_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        decision_receipt_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-recovery-decision.v1\0");
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

/// Exact previously qualified stable profile retained for recovery.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RetainedStableProfileV1 {
    profile: ToolchainProfileV1,
    qualified_at: LifecycleTimestampV1,
    superseded_at: LifecycleTimestampV1,
    qualification_history_receipt_sha256: DigestV1,
    retained_artifact_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl RetainedStableProfileV1 {
    pub fn try_new(
        profile: ToolchainProfileV1,
        qualified_at: LifecycleTimestampV1,
        superseded_at: LifecycleTimestampV1,
        qualification_history_receipt_sha256: DigestV1,
        retained_artifact_receipt_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if profile.role() != ToolchainRoleV1::QualifiedStableExecution
            || !matches!(
                profile.qualification(),
                ToolchainQualificationV1::Production { .. }
            )
            || qualified_at >= superseded_at
        {
            return Err(invalid_toolchain_recovery());
        }
        let mut hash = CanonicalHasherV1::new(b"build.retained-stable-profile.v1\0");
        hash.digest(profile.identity_sha256());
        hash.u64(qualified_at.unix_seconds());
        hash.u64(superseded_at.unix_seconds());
        hash.digest(qualification_history_receipt_sha256);
        hash.digest(retained_artifact_receipt_sha256);
        Ok(Self {
            profile,
            qualified_at,
            superseded_at,
            qualification_history_receipt_sha256,
            retained_artifact_receipt_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &ToolchainProfileV1 {
        &self.profile
    }

    #[must_use]
    pub const fn qualified_at(&self) -> LifecycleTimestampV1 {
        self.qualified_at
    }

    #[must_use]
    pub const fn superseded_at(&self) -> LifecycleTimestampV1 {
        self.superseded_at
    }

    #[must_use]
    pub const fn qualification_history_receipt_sha256(&self) -> DigestV1 {
        self.qualification_history_receipt_sha256
    }

    #[must_use]
    pub const fn retained_artifact_receipt_sha256(&self) -> DigestV1 {
        self.retained_artifact_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Expiring incident bound to one exact blocked posture.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainRecoveryIncidentV1 {
    blocked_posture_identity_sha256: DigestV1,
    policy: ToolchainStableRecoveryPolicyV1,
    opened_at: LifecycleTimestampV1,
    expires_at: LifecycleTimestampV1,
    decision: ToolchainRecoveryDecisionEvidenceV1,
    identity_sha256: DigestV1,
}

impl ToolchainRecoveryIncidentV1 {
    pub fn try_new(
        blocked_posture: &ToolchainSafetyPostureV1,
        policy: &ToolchainStableRecoveryPolicyV1,
        opened_at: LifecycleTimestampV1,
        expires_at: LifecycleTimestampV1,
        decision: ToolchainRecoveryDecisionEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let Some(duration) = expires_at.unix_seconds().checked_sub(opened_at.unix_seconds()) else {
            return Err(invalid_toolchain_recovery());
        };
        if blocked_posture.status() != ToolchainSafetyPostureStatusV1::Blocked
            || opened_at < blocked_posture.evaluated_at()
            || duration == 0
            || duration > policy.maximum_incident_duration_seconds()
        {
            return Err(invalid_toolchain_recovery());
        }
        let blocked_posture_identity_sha256 = blocked_posture.identity_sha256();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-recovery-incident.v1\0");
        hash.digest(blocked_posture_identity_sha256);
        hash.digest(policy.identity_sha256());
        hash.u64(opened_at.unix_seconds());
        hash.u64(expires_at.unix_seconds());
        hash.digest(decision.identity_sha256());
        Ok(Self {
            blocked_posture_identity_sha256,
            policy: policy.clone(),
            opened_at,
            expires_at,
            decision,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn blocked_posture_identity_sha256(&self) -> DigestV1 {
        self.blocked_posture_identity_sha256
    }

    #[must_use]
    pub const fn policy(&self) -> &ToolchainStableRecoveryPolicyV1 {
        &self.policy
    }

    #[must_use]
    pub const fn opened_at(&self) -> LifecycleTimestampV1 {
        self.opened_at
    }

    #[must_use]
    pub const fn expires_at(&self) -> LifecycleTimestampV1 {
        self.expires_at
    }

    #[must_use]
    pub const fn decision(&self) -> ToolchainRecoveryDecisionEvidenceV1 {
        self.decision
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Atomic evidence consumed by one stable-recovery candidate.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainStableRecoveryEvidenceV1 {
    current_safety: ToolchainSafetyPostureV1,
    proposed_safety: ToolchainSafetyPostureV1,
    retained_stable: RetainedStableProfileV1,
    incident: ToolchainRecoveryIncidentV1,
    identity_sha256: DigestV1,
}

impl ToolchainStableRecoveryEvidenceV1 {
    #[must_use]
    pub fn new(
        current_safety: ToolchainSafetyPostureV1,
        proposed_safety: ToolchainSafetyPostureV1,
        retained_stable: RetainedStableProfileV1,
        incident: ToolchainRecoveryIncidentV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-stable-recovery-evidence.v1\0");
        hash.digest(current_safety.identity_sha256());
        hash.digest(proposed_safety.identity_sha256());
        hash.digest(retained_stable.identity_sha256());
        hash.digest(incident.identity_sha256());
        Self {
            current_safety,
            proposed_safety,
            retained_stable,
            incident,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn current_safety(&self) -> &ToolchainSafetyPostureV1 {
        &self.current_safety
    }

    #[must_use]
    pub const fn proposed_safety(&self) -> &ToolchainSafetyPostureV1 {
        &self.proposed_safety
    }

    #[must_use]
    pub const fn retained_stable(&self) -> &RetainedStableProfileV1 {
        &self.retained_stable
    }

    #[must_use]
    pub const fn incident(&self) -> &ToolchainRecoveryIncidentV1 {
        &self.incident
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
