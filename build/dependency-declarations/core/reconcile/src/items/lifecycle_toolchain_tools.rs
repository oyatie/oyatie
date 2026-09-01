/// Role-specific evidence for one exact toolchain profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ToolchainQualificationV1 {
    Compatibility {
        qualification_receipt_sha256: DigestV1,
    },
    Production {
        qualification_receipt_sha256: DigestV1,
    },
    Shadow {
        observation_receipt_sha256: DigestV1,
    },
}

impl ToolchainQualificationV1 {
    fn matches_role(self, role: ToolchainRoleV1) -> bool {
        matches!(
            (self, role),
            (
                Self::Compatibility { .. },
                ToolchainRoleV1::DeclaredMsrvCompatibility
            ) | (
                Self::Production { .. },
                ToolchainRoleV1::QualifiedStableExecution
            ) | (
                Self::Shadow { .. },
                ToolchainRoleV1::BetaShadow | ToolchainRoleV1::NightlyShadow
            )
        )
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Compatibility {
                qualification_receipt_sha256,
            } => {
                hash.tag(0);
                hash.digest(qualification_receipt_sha256);
            }
            Self::Production {
                qualification_receipt_sha256,
            } => {
                hash.tag(1);
                hash.digest(qualification_receipt_sha256);
            }
            Self::Shadow {
                observation_receipt_sha256,
            } => {
                hash.tag(2);
                hash.digest(observation_receipt_sha256);
            }
        }
    }
}

/// Exact compiler tools supplied by one Rust distribution.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainToolsV1 {
    rustc: ToolIdentityV1,
    cargo: ToolIdentityV1,
    rustfmt: ToolIdentityV1,
    clippy: ToolIdentityV1,
}

impl ToolchainToolsV1 {
    pub fn try_new(
        rustc: ToolIdentityV1,
        cargo: ToolIdentityV1,
        rustfmt: ToolIdentityV1,
        clippy: ToolIdentityV1,
    ) -> Result<Self, LifecycleFailureV1> {
        for (tool, expected) in [
            (&rustc, "rustc"),
            (&cargo, "cargo"),
            (&rustfmt, "rustfmt"),
            (&clippy, "clippy"),
        ] {
            if tool.name() != expected || tool.host_triple() != rustc.host_triple() {
                return Err(lifecycle_invalid());
            }
        }
        Ok(Self {
            rustc,
            cargo,
            rustfmt,
            clippy,
        })
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        for tool in [&self.rustc, &self.cargo, &self.rustfmt, &self.clippy] {
            tool.encode(hash).map_err(|_| lifecycle_internal())?;
        }
        Ok(())
    }

    #[must_use]
    pub const fn rustc(&self) -> &ToolIdentityV1 {
        &self.rustc
    }

    #[must_use]
    pub const fn cargo(&self) -> &ToolIdentityV1 {
        &self.cargo
    }

    #[must_use]
    pub const fn rustfmt(&self) -> &ToolIdentityV1 {
        &self.rustfmt
    }

    #[must_use]
    pub const fn clippy(&self) -> &ToolIdentityV1 {
        &self.clippy
    }
}
