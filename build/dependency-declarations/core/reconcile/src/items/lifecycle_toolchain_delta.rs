/// Stable mechanical axis changed within one exact toolchain role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ToolchainChangeAxisV1 {
    RustVersion = 0,
    DistributionSource = 1,
    Rustc = 2,
    Cargo = 3,
    Rustfmt = 4,
    Clippy = 5,
    Llvm = 6,
    TargetClosure = 7,
    Qualification = 8,
}

impl ToolchainChangeAxisV1 {
    pub const ALL: [Self; 9] = [
        Self::RustVersion,
        Self::DistributionSource,
        Self::Rustc,
        Self::Cargo,
        Self::Rustfmt,
        Self::Clippy,
        Self::Llvm,
        Self::TargetClosure,
        Self::Qualification,
    ];
    pub const COUNT: usize = Self::ALL.len();
}

/// One role-qualified mechanical toolchain change.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainChangeV1 {
    role: ToolchainRoleV1,
    axis: ToolchainChangeAxisV1,
}

impl ToolchainChangeV1 {
    const fn new(role: ToolchainRoleV1, axis: ToolchainChangeAxisV1) -> Self {
        Self { role, axis }
    }

    #[must_use]
    pub const fn role(self) -> ToolchainRoleV1 {
        self.role
    }

    #[must_use]
    pub const fn axis(self) -> ToolchainChangeAxisV1 {
        self.axis
    }
}

/// Canonical complete delta between two admitted four-role matrices.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCandidateDeltaV1 {
    changed_roles: Box<[ToolchainRoleV1]>,
    changes: Box<[ToolchainChangeV1]>,
}

impl ToolchainCandidateDeltaV1 {
    pub(crate) fn between(current: &ToolchainMatrixV1, proposed: &ToolchainMatrixV1) -> Self {
        let mut changed_roles = Vec::with_capacity(4);
        let mut changes = Vec::with_capacity(4 * ToolchainChangeAxisV1::COUNT);
        for (role, current_profile, proposed_profile) in [
            (
                ToolchainRoleV1::DeclaredMsrvCompatibility,
                current.msrv(),
                proposed.msrv(),
            ),
            (
                ToolchainRoleV1::QualifiedStableExecution,
                current.stable(),
                proposed.stable(),
            ),
            (
                ToolchainRoleV1::BetaShadow,
                current.beta(),
                proposed.beta(),
            ),
            (
                ToolchainRoleV1::NightlyShadow,
                current.nightly(),
                proposed.nightly(),
            ),
        ] {
            let prior_len = changes.len();
            record_profile_changes(&mut changes, role, current_profile, proposed_profile);
            if changes.len() != prior_len {
                changed_roles.push(role);
            }
        }
        Self {
            changed_roles: changed_roles.into_boxed_slice(),
            changes: changes.into_boxed_slice(),
        }
    }

    pub(crate) fn encode(
        &self,
        hash: &mut CanonicalHasherV1,
    ) -> Result<(), LifecycleFailureV1> {
        hash.u64(lifecycle_len(self.changes.len())?);
        for change in &self.changes {
            hash.tag(change.role() as u8);
            hash.tag(change.axis() as u8);
        }
        Ok(())
    }

    #[must_use]
    pub fn changed_roles(&self) -> &[ToolchainRoleV1] {
        &self.changed_roles
    }

    #[must_use]
    pub fn changes(&self) -> &[ToolchainChangeV1] {
        &self.changes
    }

    #[must_use]
    pub fn changed(&self, role: ToolchainRoleV1, axis: ToolchainChangeAxisV1) -> bool {
        self.changes
            .iter()
            .any(|change| change.role() == role && change.axis() == axis)
    }
}

fn record_profile_changes(
    changes: &mut Vec<ToolchainChangeV1>,
    role: ToolchainRoleV1,
    current: &ToolchainProfileV1,
    proposed: &ToolchainProfileV1,
) {
    for axis in ToolchainChangeAxisV1::ALL {
        record_toolchain_change(
            changes,
            role,
            axis,
            current.axes().identity_sha256(axis) != proposed.axes().identity_sha256(axis),
        );
    }
}

fn record_toolchain_change(
    changes: &mut Vec<ToolchainChangeV1>,
    role: ToolchainRoleV1,
    axis: ToolchainChangeAxisV1,
    condition: bool,
) {
    if condition {
        changes.push(ToolchainChangeV1::new(role, axis));
    }
}
