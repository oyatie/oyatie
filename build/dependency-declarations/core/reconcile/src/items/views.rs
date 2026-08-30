impl RepositoryCorrelationV1 {
    /// Returns the opaque repository identity.
    #[must_use]
    pub fn repository_id(&self) -> &str {
        &self.repository_id
    }

    /// Returns the immutable revision correlation.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
}

impl InputFileV1 {
    /// Returns this file's semantic role.
    #[must_use]
    pub const fn role(&self) -> InputFileRoleV1 {
        self.role
    }

    /// Returns this file's repository-relative path.
    #[must_use]
    pub const fn path(&self) -> &CanonicalPathV1 {
        &self.path
    }

    /// Returns the verified byte length.
    #[must_use]
    pub const fn length_bytes(&self) -> u64 {
        self.length_bytes
    }

    /// Returns the exact admitted bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl TreeEntryV1 {
    /// Returns the tree-relative path.
    #[must_use]
    pub const fn path(&self) -> &CanonicalPathV1 {
        &self.path
    }

    /// Returns the exact regular-file mode.
    #[must_use]
    pub const fn mode(&self) -> TreeFileModeV1 {
        self.mode
    }

    /// Returns the declared content length.
    #[must_use]
    pub const fn length_bytes(&self) -> u64 {
        self.length_bytes
    }

    /// Returns the declared content digest.
    #[must_use]
    pub const fn sha256(&self) -> DigestV1 {
        self.sha256
    }
}

impl InputTreeV1 {
    /// Returns this tree's semantic role.
    #[must_use]
    pub const fn role(&self) -> TreeRoleV1 {
        self.role
    }

    /// Returns the canonical entry manifest.
    #[must_use]
    pub const fn manifest(&self) -> &InputFileV1 {
        &self.manifest
    }

    /// Returns entries in canonical path order for batch materialization.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntryV1] {
        &self.entries
    }

    /// Returns the tree identity.
    #[must_use]
    pub const fn root_sha256(&self) -> DigestV1 {
        self.root_sha256
    }

    /// Returns the bounded entry count.
    #[must_use]
    pub const fn file_count(&self) -> u64 {
        self.file_count
    }

    /// Returns the declared aggregate byte count.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

impl PlatformIdentityV1 {
    /// Returns the canonical profile name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact Rust target triple.
    #[must_use]
    pub fn target_triple(&self) -> &str {
        &self.target_triple
    }

    /// Returns the Buck select label.
    #[must_use]
    pub fn select_label(&self) -> &str {
        &self.select_label
    }

    /// Returns the Buck platform label.
    #[must_use]
    pub fn platform_label(&self) -> &str {
        &self.platform_label
    }

    /// Reports whether this profile has a qualified execution platform.
    #[must_use]
    pub const fn execution_platform(&self) -> bool {
        self.execution_platform
    }
}

impl PlatformSetV1 {
    /// Returns mappings in canonical name order.
    #[must_use]
    pub fn entries(&self) -> &[PlatformIdentityV1] {
        &self.entries
    }
}

impl ArtifactIdentityV1 {
    /// Returns the component name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the component version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the exact source revision.
    #[must_use]
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    /// Returns the source digest.
    #[must_use]
    pub const fn source_sha256(&self) -> DigestV1 {
        self.source_sha256
    }

    /// Returns the built artifact digest.
    #[must_use]
    pub const fn artifact_sha256(&self) -> DigestV1 {
        self.artifact_sha256
    }
}

impl ToolIdentityV1 {
    /// Returns the tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the tool version independently of its commit.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the exact tool commit.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// Returns the tool's host triple.
    #[must_use]
    pub fn host_triple(&self) -> &str {
        &self.host_triple
    }

    /// Returns the executable digest.
    #[must_use]
    pub const fn binary_sha256(&self) -> DigestV1 {
        self.binary_sha256
    }
}

impl GeneratorIdentityV1 {
    /// Returns the generator name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the generator version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the exact source revision.
    #[must_use]
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    /// Returns the complete source-tree digest.
    #[must_use]
    pub const fn source_tree_sha256(&self) -> DigestV1 {
        self.source_tree_sha256
    }

    /// Returns the executable digest.
    #[must_use]
    pub const fn binary_sha256(&self) -> DigestV1 {
        self.binary_sha256
    }

    /// Returns the binary provenance mode.
    #[must_use]
    pub const fn binary(&self) -> &GeneratorBinaryV1 {
        &self.binary
    }
}

impl GenerationQualificationV1 {
    /// Returns the renderer identity.
    #[must_use]
    pub const fn renderer(&self) -> &ArtifactIdentityV1 {
        &self.renderer
    }

    /// Returns the maintained parser identity.
    #[must_use]
    pub const fn parser(&self) -> &ArtifactIdentityV1 {
        &self.parser
    }

    /// Returns the producer-owned graph profile.
    #[must_use]
    pub const fn provider_graph(&self) -> &ProviderGraphProfileV1 {
        &self.provider_graph
    }

    /// Returns the admitted grammar digest.
    #[must_use]
    pub const fn grammar_sha256(&self) -> DigestV1 {
        self.grammar_sha256
    }

    /// Returns the Buck consumer-profile identity.
    #[must_use]
    pub const fn buck_consumer(&self) -> &BuckConsumerProfileV1 {
        &self.buck_consumer
    }
}

impl ProviderGraphProfileV1 {
    /// Returns the exact source-adaptation recipe identity.
    #[must_use]
    pub fn adaptation_recipe_id(&self) -> &str {
        &self.adaptation_recipe_id
    }

    /// Returns the digest of the upstream source file whose schema was admitted.
    #[must_use]
    pub const fn schema_source_sha256(&self) -> DigestV1 {
        self.schema_source_sha256
    }

    /// Returns the complete producer semantic-schema digest.
    #[must_use]
    pub const fn semantic_schema_sha256(&self) -> DigestV1 {
        self.semantic_schema_sha256
    }
}
