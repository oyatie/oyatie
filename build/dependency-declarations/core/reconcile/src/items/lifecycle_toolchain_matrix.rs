/// Four independently identified Rust compatibility/execution lanes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainMatrixV1 {
    msrv: ToolchainProfileV1,
    stable: ToolchainProfileV1,
    beta: ToolchainProfileV1,
    nightly: ToolchainProfileV1,
    identity_sha256: DigestV1,
}

impl ToolchainMatrixV1 {
    pub fn try_new(
        msrv: ToolchainProfileV1,
        stable: ToolchainProfileV1,
        beta: ToolchainProfileV1,
        nightly: ToolchainProfileV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if msrv.role() != ToolchainRoleV1::DeclaredMsrvCompatibility
            || stable.role() != ToolchainRoleV1::QualifiedStableExecution
            || beta.role() != ToolchainRoleV1::BetaShadow
            || nightly.role() != ToolchainRoleV1::NightlyShadow
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainRoleMismatch,
            ));
        }
        let host_triple = msrv.tools().rustc().host_triple();
        if [&stable, &beta, &nightly]
            .into_iter()
            .any(|profile| profile.tools().rustc().host_triple() != host_triple)
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainTargetMismatch,
            ));
        }
        if msrv.version() > stable.version()
            || stable.version() >= beta.version()
            || beta.version() >= nightly.version()
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnsupportedVersionRelation,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-matrix.v1\0");
        for profile in [&msrv, &stable, &beta, &nightly] {
            hash.digest(profile.identity_sha256());
        }
        Ok(Self {
            msrv,
            stable,
            beta,
            nightly,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn msrv(&self) -> &ToolchainProfileV1 {
        &self.msrv
    }

    #[must_use]
    pub const fn stable(&self) -> &ToolchainProfileV1 {
        &self.stable
    }

    #[must_use]
    pub const fn beta(&self) -> &ToolchainProfileV1 {
        &self.beta
    }

    #[must_use]
    pub const fn nightly(&self) -> &ToolchainProfileV1 {
        &self.nightly
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
