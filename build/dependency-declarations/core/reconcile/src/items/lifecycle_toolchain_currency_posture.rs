/// Current relation between one execution role and its observed channel head.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ToolchainCurrencyPostureStatusV1 {
    OnObservedHead = 0,
    DiffersFromObservedHeadWithinTarget = 1,
    DiffersFromObservedHeadOverdue = 2,
}

/// Currency posture for one of the three execution roles.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainRoleCurrencyPostureV1 {
    role: ToolchainRoleV1,
    current_material_identity_sha256: DigestV1,
    observed_material_identity_sha256: DigestV1,
    observed_head_identity_sha256: DigestV1,
    observed_head_version: RustVersionV1,
    head_age_seconds: u64,
    due_at: LifecycleTimestampV1,
    status: ToolchainCurrencyPostureStatusV1,
    identity_sha256: DigestV1,
}

impl ToolchainRoleCurrencyPostureV1 {
    fn try_evaluate(
        role: ToolchainRoleV1,
        current: &ToolchainProfileV1,
        head: &ToolchainChannelHeadV1,
        target_seconds: u64,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if current.role() != role || head.role() != role {
            return Err(lifecycle_internal());
        }
        if head.version() < current.version() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnsupportedVersionRelation,
            ));
        }
        let due_at = checked_lifecycle_timestamp_add(head.released_at(), target_seconds)?;
        let head_age_seconds = evaluated_at
            .unix_seconds()
            .checked_sub(head.released_at().unix_seconds())
            .ok_or_else(lifecycle_bounds)?;
        let current_material_identity_sha256 = current.material_identity_sha256();
        let observed_material_identity_sha256 = head.material_identity_sha256();
        let status = if current_material_identity_sha256 == observed_material_identity_sha256 {
            ToolchainCurrencyPostureStatusV1::OnObservedHead
        } else if evaluated_at <= due_at {
            ToolchainCurrencyPostureStatusV1::DiffersFromObservedHeadWithinTarget
        } else {
            ToolchainCurrencyPostureStatusV1::DiffersFromObservedHeadOverdue
        };
        let observed_head_identity_sha256 = head.identity_sha256();
        let observed_head_version = head.version();
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-role-currency-posture.v1\0");
        hash.tag(role as u8);
        hash.digest(current_material_identity_sha256);
        hash.digest(observed_material_identity_sha256);
        hash.digest(observed_head_identity_sha256);
        observed_head_version.encode(&mut hash);
        hash.u64(head_age_seconds);
        hash.u64(due_at.unix_seconds());
        hash.tag(status as u8);
        Ok(Self {
            role,
            current_material_identity_sha256,
            observed_material_identity_sha256,
            observed_head_identity_sha256,
            observed_head_version,
            head_age_seconds,
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
    pub const fn observed_material_identity_sha256(&self) -> DigestV1 {
        self.observed_material_identity_sha256
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
    pub const fn head_age_seconds(&self) -> u64 {
        self.head_age_seconds
    }

    #[must_use]
    pub const fn due_at(&self) -> LifecycleTimestampV1 {
        self.due_at
    }

    #[must_use]
    pub const fn status(&self) -> ToolchainCurrencyPostureStatusV1 {
        self.status
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// All execution-channel currency posture from one atomic observation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCurrencyPostureV1 {
    current: ToolchainMatrixV1,
    snapshot: ToolchainChannelSnapshotV1,
    policy: ToolchainCurrencyPolicyV1,
    evaluated_at: LifecycleTimestampV1,
    execution_roles: [ToolchainRoleCurrencyPostureV1; 3],
    identity_sha256: DigestV1,
}

impl ToolchainCurrencyPostureV1 {
    pub fn try_evaluate(
        current: &ToolchainMatrixV1,
        snapshot: &ToolchainChannelSnapshotV1,
        policy: &ToolchainCurrencyPolicyV1,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_toolchain_currency_observation(snapshot, policy, evaluated_at)?;
        if snapshot.host_triple() != current.stable().tools().rustc().host_triple() {
            return Err(toolchain_analysis_mismatch());
        }
        let execution_roles = [
            evaluate_posture_role(
                ToolchainRoleV1::QualifiedStableExecution,
                current,
                snapshot,
                policy,
                evaluated_at,
            )?,
            evaluate_posture_role(
                ToolchainRoleV1::BetaShadow,
                current,
                snapshot,
                policy,
                evaluated_at,
            )?,
            evaluate_posture_role(
                ToolchainRoleV1::NightlyShadow,
                current,
                snapshot,
                policy,
                evaluated_at,
            )?,
        ];
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-currency-posture.v1\0");
        hash.digest(current.identity_sha256());
        hash.digest(snapshot.identity_sha256());
        hash.digest(policy.identity_sha256());
        hash.u64(evaluated_at.unix_seconds());
        for role in &execution_roles {
            hash.digest(role.identity_sha256());
        }
        Ok(Self {
            current: current.clone(),
            snapshot: snapshot.clone(),
            policy: policy.clone(),
            evaluated_at,
            execution_roles,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn current(&self) -> &ToolchainMatrixV1 {
        &self.current
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
    pub const fn execution_roles(&self) -> &[ToolchainRoleCurrencyPostureV1; 3] {
        &self.execution_roles
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn evaluate_posture_role(
    role: ToolchainRoleV1,
    current: &ToolchainMatrixV1,
    snapshot: &ToolchainChannelSnapshotV1,
    policy: &ToolchainCurrencyPolicyV1,
    evaluated_at: LifecycleTimestampV1,
) -> Result<ToolchainRoleCurrencyPostureV1, LifecycleFailureV1> {
    let current = matrix_profile(current, role).ok_or_else(lifecycle_internal)?;
    let head = snapshot.head(role).ok_or_else(lifecycle_internal)?;
    let target_seconds = policy.target_seconds(role).ok_or_else(lifecycle_internal)?;
    ToolchainRoleCurrencyPostureV1::try_evaluate(
        role,
        current,
        head,
        target_seconds,
        evaluated_at,
    )
}
