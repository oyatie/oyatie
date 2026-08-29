/// Producer-owned mechanical projection of one exact toolchain profile.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainProfileAxesV1 {
    version: RustVersionV1,
    source: LifecycleSourceV1,
    tools: ToolchainToolsV1,
    qualification: ToolchainQualificationV1,
    llvm_version: Box<str>,
    targets: Box<[ToolchainTargetV1]>,
    identities: [DigestV1; ToolchainChangeAxisV1::COUNT],
}

impl ToolchainProfileAxesV1 {
    pub(crate) fn try_new(
        version: RustVersionV1,
        source: LifecycleSourceV1,
        tools: ToolchainToolsV1,
        qualification: ToolchainQualificationV1,
        llvm_version: Box<str>,
        targets: Box<[ToolchainTargetV1]>,
    ) -> Result<Self, LifecycleFailureV1> {
        let identities = [
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
            derive_toolchain_axis(ToolchainChangeAxisV1::Qualification, |hash| {
                qualification.encode(hash);
                Ok(())
            })?,
        ];
        Ok(Self {
            version,
            source,
            tools,
            qualification,
            llvm_version,
            targets,
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
