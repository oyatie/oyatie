/// Exact configured Buck2 consumer profile required by one generation profile.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BuckConsumerProfileV1 {
    buck2: ArtifactIdentityV1,
    prelude: ArtifactIdentityV1,
    rules_sha256: DigestV1,
    toolchain_sha256: DigestV1,
    cell_config_sha256: DigestV1,
    buckconfig_sha256: DigestV1,
    qualification_plan_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl BuckConsumerProfileV1 {
    /// Groups the independently versioned configured-consumer inputs.
    pub fn try_new(
        buck2: ArtifactIdentityV1,
        prelude: ArtifactIdentityV1,
        rules_sha256: DigestV1,
        toolchain_sha256: DigestV1,
        cell_config_sha256: DigestV1,
        buckconfig_sha256: DigestV1,
        qualification_plan_sha256: DigestV1,
    ) -> Result<Self, FailureV1> {
        let mut profile = Self {
            buck2,
            prelude,
            rules_sha256,
            toolchain_sha256,
            cell_config_sha256,
            buckconfig_sha256,
            qualification_plan_sha256,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.buck-consumer-profile.v1\0");
        profile.encode(&mut hash)?;
        profile.identity_sha256 = hash.finish();
        Ok(profile)
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.buck2.encode_fields(hash)?;
        self.prelude.encode_fields(hash)?;
        hash.digest(self.rules_sha256);
        hash.digest(self.toolchain_sha256);
        hash.digest(self.cell_config_sha256);
        hash.digest(self.buckconfig_sha256);
        hash.digest(self.qualification_plan_sha256);
        Ok(())
    }

    /// Returns the exact Buck2 source and binary identity.
    #[must_use]
    pub const fn buck2(&self) -> &ArtifactIdentityV1 {
        &self.buck2
    }

    /// Returns the exact Prelude source and artifact identity.
    #[must_use]
    pub const fn prelude(&self) -> &ArtifactIdentityV1 {
        &self.prelude
    }

    /// Returns the owned rule-library digest.
    #[must_use]
    pub const fn rules_sha256(&self) -> DigestV1 {
        self.rules_sha256
    }

    /// Returns the configured Buck toolchain-profile digest.
    #[must_use]
    pub const fn toolchain_sha256(&self) -> DigestV1 {
        self.toolchain_sha256
    }

    /// Returns the complete cell-configuration digest.
    #[must_use]
    pub const fn cell_config_sha256(&self) -> DigestV1 {
        self.cell_config_sha256
    }

    /// Returns the root Buck configuration digest.
    #[must_use]
    pub const fn buckconfig_sha256(&self) -> DigestV1 {
        self.buckconfig_sha256
    }

    /// Returns the configured query and representative-consumption plan identity.
    #[must_use]
    pub const fn qualification_plan_sha256(&self) -> DigestV1 {
        self.qualification_plan_sha256
    }

    /// Returns the configured consumer identity without execution evidence.
    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
