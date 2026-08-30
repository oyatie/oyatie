/// Numeric Rust compatibility version, independent of an execution toolchain.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RustVersionV1 {
    major: u16,
    minor: u16,
    patch: u16,
}

impl RustVersionV1 {
    pub fn try_new(major: u16, minor: u16, patch: u16) -> Result<Self, LifecycleFailureV1> {
        if major == 0 {
            return Err(lifecycle_invalid());
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        hash.u64(u64::from(self.major));
        hash.u64(u64::from(self.minor));
        hash.u64(u64::from(self.patch));
    }

    fn matches_rustc_release(self, role: ToolchainRoleV1, release: &str) -> bool {
        let base = format!("{}.{}.{}", self.major, self.minor, self.patch);
        let Some(suffix) = release.strip_prefix(&base) else {
            return false;
        };
        match role {
            ToolchainRoleV1::DeclaredMsrvCompatibility
            | ToolchainRoleV1::QualifiedStableExecution => suffix.is_empty(),
            ToolchainRoleV1::BetaShadow => suffix
                .strip_prefix("-beta.")
                .is_some_and(|serial| {
                    !serial.is_empty() && serial.bytes().all(|byte| byte.is_ascii_digit())
                }),
            ToolchainRoleV1::NightlyShadow => suffix == "-nightly",
        }
    }

    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }
}

/// Distinct role played by an exact Rust distribution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ToolchainRoleV1 {
    DeclaredMsrvCompatibility = 0,
    QualifiedStableExecution = 1,
    BetaShadow = 2,
    NightlyShadow = 3,
}

/// One qualified target/component closure for a toolchain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainTargetV1 {
    target_triple: Box<str>,
    standard_library_sha256: DigestV1,
    components_sha256: DigestV1,
}

impl ToolchainTargetV1 {
    pub fn try_new(
        target_triple: impl Into<String>,
        standard_library_sha256: DigestV1,
        components_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            target_triple: lifecycle_identity(target_triple.into())?,
            standard_library_sha256,
            components_sha256,
        })
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        lifecycle_hash_string(hash, &self.target_triple)?;
        hash.digest(self.standard_library_sha256);
        hash.digest(self.components_sha256);
        Ok(())
    }

    #[must_use]
    pub fn target_triple(&self) -> &str {
        &self.target_triple
    }

    #[must_use]
    pub const fn standard_library_sha256(&self) -> DigestV1 {
        self.standard_library_sha256
    }

    #[must_use]
    pub const fn components_sha256(&self) -> DigestV1 {
        self.components_sha256
    }
}

/// Exact compiler/tool/component profile for one role.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainProfileV1 {
    role: ToolchainRoleV1,
    axes: ToolchainProfileAxesV1,
    identity_sha256: DigestV1,
}

impl ToolchainProfileV1 {
    pub fn try_new(
        role: ToolchainRoleV1,
        version: RustVersionV1,
        source: LifecycleSourceV1,
        tools: ToolchainToolsV1,
        qualification: ToolchainQualificationV1,
        llvm_version: impl Into<String>,
        targets: Vec<ToolchainTargetV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        let material = ToolchainProfileMaterialV1::try_new(
            role,
            version,
            source,
            tools,
            llvm_version,
            targets,
        )?;
        Self::try_from_material(material, qualification)
    }

    pub fn try_from_material(
        material: ToolchainProfileMaterialV1,
        qualification: ToolchainQualificationV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let role = material.role();
        let axes = ToolchainProfileAxesV1::try_new(material, qualification)?;
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-profile.v1\0");
        hash.tag(role as u8);
        axes.encode(&mut hash);
        Ok(Self {
            role,
            axes,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn role(&self) -> ToolchainRoleV1 {
        self.role
    }

    #[must_use]
    pub const fn version(&self) -> RustVersionV1 {
        self.axes.material().version()
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }

    #[must_use]
    pub const fn material_identity_sha256(&self) -> DigestV1 {
        self.axes.material().identity_sha256()
    }

    #[must_use]
    pub const fn material(&self) -> &ToolchainProfileMaterialV1 {
        self.axes.material()
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        self.axes.material().source()
    }

    #[must_use]
    pub const fn tools(&self) -> &ToolchainToolsV1 {
        self.axes.material().tools()
    }

    #[must_use]
    pub const fn qualification(&self) -> ToolchainQualificationV1 {
        self.axes.qualification()
    }

    #[must_use]
    pub fn llvm_version(&self) -> &str {
        self.axes.material().llvm_version()
    }

    #[must_use]
    pub fn targets(&self) -> &[ToolchainTargetV1] {
        self.axes.material().targets()
    }

    #[must_use]
    pub const fn axes(&self) -> &ToolchainProfileAxesV1 {
        &self.axes
    }
}

fn validate_toolchain_role(
    role: ToolchainRoleV1,
    source: &LifecycleSourceV1,
) -> Result<(), LifecycleFailureV1> {
    let valid = match role {
        ToolchainRoleV1::DeclaredMsrvCompatibility
        | ToolchainRoleV1::QualifiedStableExecution => {
            source.channel() == LifecycleChannelV1::Stable
                && source.maturity() == SourceMaturityV1::Released
        }
        ToolchainRoleV1::BetaShadow => {
            source.channel() == LifecycleChannelV1::Beta
                && source.maturity() == SourceMaturityV1::Provisional
        }
        ToolchainRoleV1::NightlyShadow => {
            source.channel() == LifecycleChannelV1::Nightly
                && source.maturity() == SourceMaturityV1::Provisional
        }
    };
    if source.component() != LifecycleComponentV1::RustDistribution || !valid {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::ToolchainRoleMismatch,
        ));
    }
    Ok(())
}
