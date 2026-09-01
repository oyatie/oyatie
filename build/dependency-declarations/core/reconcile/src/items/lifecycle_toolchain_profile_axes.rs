const TOOLCHAIN_MATERIAL_AXIS_COUNT: usize = ToolchainChangeAxisV1::Qualification as usize;

/// Exact compiler, tool, LLVM, and target material before local qualification.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainProfileMaterialV1 {
    role: ToolchainRoleV1,
    version: RustVersionV1,
    source: LifecycleSourceV1,
    tools: ToolchainToolsV1,
    llvm_version: Box<str>,
    targets: Box<[ToolchainTargetV1]>,
    axis_identities: [DigestV1; TOOLCHAIN_MATERIAL_AXIS_COUNT],
    identity_sha256: DigestV1,
}

impl ToolchainProfileMaterialV1 {
    pub fn try_new(
        role: ToolchainRoleV1,
        version: RustVersionV1,
        source: LifecycleSourceV1,
        tools: ToolchainToolsV1,
        llvm_version: impl Into<String>,
        mut targets: Vec<ToolchainTargetV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_toolchain_role(role, &source)?;
        if !version.matches_rustc_release(role, tools.rustc().version()) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainVersionMismatch,
            ));
        }
        if targets.is_empty() || targets.len() > LifecycleBoundsV1::MAX_TOOLCHAIN_TARGETS {
            return Err(lifecycle_bounds());
        }
        if !targets
            .iter()
            .any(|target| target.target_triple() == tools.rustc().host_triple())
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainTargetMismatch,
            ));
        }
        targets.sort_by(|left, right| {
            left.target_triple
                .as_bytes()
                .cmp(right.target_triple.as_bytes())
        });
        if targets
            .windows(2)
            .any(|pair| pair[0].target_triple == pair[1].target_triple)
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let llvm_version = lifecycle_identity(llvm_version.into())?;
        let targets = targets.into_boxed_slice();
        let axis_identities = [
            derive_toolchain_axis(ToolchainChangeAxisV1::RustVersion, |hash| {
                version.encode(hash);
                Ok(())
            })?,
            derive_toolchain_axis(ToolchainChangeAxisV1::DistributionSource, |hash| {
                hash.digest(source.identity_sha256());
                Ok(())
            })?,
            derive_tool_axis(ToolchainChangeAxisV1::Rustc, tools.rustc())?,
            derive_tool_axis(ToolchainChangeAxisV1::Cargo, tools.cargo())?,
            derive_tool_axis(ToolchainChangeAxisV1::Rustfmt, tools.rustfmt())?,
            derive_tool_axis(ToolchainChangeAxisV1::Clippy, tools.clippy())?,
            derive_toolchain_axis(ToolchainChangeAxisV1::Llvm, |hash| {
                lifecycle_hash_string(hash, &llvm_version)
            })?,
            derive_toolchain_axis(ToolchainChangeAxisV1::TargetClosure, |hash| {
                hash.u64(lifecycle_len(targets.len())?);
                for target in &targets {
                    target.encode(hash)?;
                }
                Ok(())
            })?,
        ];
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-profile-material.v1\0");
        hash.tag(role as u8);
        hash.u64(TOOLCHAIN_MATERIAL_AXIS_COUNT as u64);
        for axis in ToolchainChangeAxisV1::ALL
            .into_iter()
            .take(TOOLCHAIN_MATERIAL_AXIS_COUNT)
        {
            hash.tag(axis as u8);
            hash.digest(axis_identities[axis as usize]);
        }
        Ok(Self {
            role,
            version,
            source,
            tools,
            llvm_version,
            targets,
            axis_identities,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn role(&self) -> ToolchainRoleV1 {
        self.role
    }

    #[must_use]
    pub const fn version(&self) -> RustVersionV1 {
        self.version
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        &self.source
    }

    #[must_use]
    pub const fn tools(&self) -> &ToolchainToolsV1 {
        &self.tools
    }

    #[must_use]
    pub fn llvm_version(&self) -> &str {
        &self.llvm_version
    }

    #[must_use]
    pub fn targets(&self) -> &[ToolchainTargetV1] {
        &self.targets
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Producer-owned mechanical projection of one qualified toolchain profile.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainProfileAxesV1 {
    material: ToolchainProfileMaterialV1,
    qualification: ToolchainQualificationV1,
    identities: [DigestV1; ToolchainChangeAxisV1::COUNT],
}

impl ToolchainProfileAxesV1 {
    pub(crate) fn try_new(
        material: ToolchainProfileMaterialV1,
        qualification: ToolchainQualificationV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if !qualification.matches_role(material.role()) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainRoleMismatch,
            ));
        }
        let qualification_identity =
            derive_toolchain_axis(ToolchainChangeAxisV1::Qualification, |hash| {
                qualification.encode(hash);
                Ok(())
            })?;
        let identities = std::array::from_fn(|index| {
            if index < TOOLCHAIN_MATERIAL_AXIS_COUNT {
                material.axis_identities[index]
            } else {
                qualification_identity
            }
        });
        Ok(Self {
            material,
            qualification,
            identities,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) {
        hash.u64(ToolchainChangeAxisV1::COUNT as u64);
        for axis in ToolchainChangeAxisV1::ALL {
            hash.tag(axis as u8);
            hash.digest(self.identity_sha256(axis));
        }
    }

    #[must_use]
    pub const fn identity_sha256(&self, axis: ToolchainChangeAxisV1) -> DigestV1 {
        self.identities[axis as usize]
    }

    #[must_use]
    pub const fn material(&self) -> &ToolchainProfileMaterialV1 {
        &self.material
    }

    #[must_use]
    pub const fn qualification(&self) -> ToolchainQualificationV1 {
        self.qualification
    }
}

fn derive_tool_axis(
    axis: ToolchainChangeAxisV1,
    tool: &ToolIdentityV1,
) -> Result<DigestV1, LifecycleFailureV1> {
    derive_toolchain_axis(axis, |hash| {
        tool.encode(hash).map_err(|_| lifecycle_internal())
    })
}

fn derive_toolchain_axis(
    axis: ToolchainChangeAxisV1,
    encode: impl FnOnce(&mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1>,
) -> Result<DigestV1, LifecycleFailureV1> {
    let mut hash = CanonicalHasherV1::new(b"build.toolchain-profile-axis.v1\0");
    hash.tag(axis as u8);
    encode(&mut hash)?;
    Ok(hash.finish())
}
