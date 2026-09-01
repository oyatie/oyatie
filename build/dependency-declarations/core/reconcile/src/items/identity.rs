/// Repository correlation that is never used as a content precondition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RepositoryCorrelationV1 {
    repository_id: Box<str>,
    revision: Box<str>,
}

impl RepositoryCorrelationV1 {
    /// Creates a correlation from opaque nonempty identities.
    pub fn try_new(
        repository_id: impl Into<String>,
        revision: impl Into<String>,
    ) -> Result<Self, FailureV1> {
        Ok(Self {
            repository_id: validated_identity(repository_id.into())?,
            revision: validated_identity(revision.into())?,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.repository_id)?;
        hash.string(&self.revision)
    }
}

/// Exact source and built-artifact identity for a qualified component.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ArtifactIdentityV1 {
    name: Box<str>,
    version: Box<str>,
    source_revision: Box<str>,
    source_sha256: DigestV1,
    artifact_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ArtifactIdentityV1 {
    /// Creates a fully bound component identity.
    pub fn try_new(
        name: impl Into<String>,
        version: impl Into<String>,
        source_revision: impl Into<String>,
        source_sha256: DigestV1,
        artifact_sha256: DigestV1,
    ) -> Result<Self, FailureV1> {
        let mut value = Self {
            name: validated_identity(name.into())?,
            version: validated_identity(version.into())?,
            source_revision: validated_identity(source_revision.into())?,
            source_sha256,
            artifact_sha256,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.artifact-identity.v1\0");
        value.encode_fields(&mut hash)?;
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    /// Returns the stable identity of every component field.
    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }

    pub(crate) fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.name)?;
        hash.string(&self.version)?;
        hash.string(&self.source_revision)?;
        hash.digest(self.source_sha256);
        hash.digest(self.artifact_sha256);
        Ok(())
    }
}

/// Exact executable tool identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolIdentityV1 {
    name: Box<str>,
    version: Box<str>,
    commit: Box<str>,
    host_triple: Box<str>,
    binary_sha256: DigestV1,
}

impl ToolIdentityV1 {
    /// Creates a tool identity without conflating version and commit.
    pub fn try_new(
        name: impl Into<String>,
        version: impl Into<String>,
        commit: impl Into<String>,
        host_triple: impl Into<String>,
        binary_sha256: DigestV1,
    ) -> Result<Self, FailureV1> {
        Ok(Self {
            name: validated_identity(name.into())?,
            version: validated_identity(version.into())?,
            commit: validated_identity(commit.into())?,
            host_triple: validated_identity(host_triple.into())?,
            binary_sha256,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.name)?;
        hash.string(&self.version)?;
        hash.string(&self.commit)?;
        hash.string(&self.host_triple)?;
        hash.digest(self.binary_sha256);
        Ok(())
    }
}

/// Provenance mode of the generator binary.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GeneratorBinaryV1 {
    ReproducibleBuild { receipt_sha256: DigestV1 },
    ReleaseAsset { asset_sha256: DigestV1 },
}

/// Exact source and executable identity of the generator.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GeneratorIdentityV1 {
    name: Box<str>,
    version: Box<str>,
    source_revision: Box<str>,
    source_tree_sha256: DigestV1,
    binary_sha256: DigestV1,
    binary: GeneratorBinaryV1,
}

impl GeneratorIdentityV1 {
    /// Creates a generator identity with one explicit binary provenance mode.
    pub fn try_new(
        name: impl Into<String>,
        version: impl Into<String>,
        source_revision: impl Into<String>,
        source_tree_sha256: DigestV1,
        binary_sha256: DigestV1,
        binary: GeneratorBinaryV1,
    ) -> Result<Self, FailureV1> {
        Ok(Self {
            name: validated_identity(name.into())?,
            version: validated_identity(version.into())?,
            source_revision: validated_identity(source_revision.into())?,
            source_tree_sha256,
            binary_sha256,
            binary,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.name)?;
        hash.string(&self.version)?;
        hash.string(&self.source_revision)?;
        hash.digest(self.source_tree_sha256);
        hash.digest(self.binary_sha256);
        match self.binary {
            GeneratorBinaryV1::ReproducibleBuild { receipt_sha256 } => {
                hash.tag(0);
                hash.digest(receipt_sha256);
            }
            GeneratorBinaryV1::ReleaseAsset { asset_sha256 } => {
                hash.tag(1);
                hash.digest(asset_sha256);
            }
        }
        Ok(())
    }
}

/// Exact identity of the producer-owned graph carried by the Reindeer artifact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProviderGraphProfileV1 {
    adaptation_recipe_id: Box<str>,
    schema_source_sha256: DigestV1,
    semantic_schema_sha256: DigestV1,
}

impl ProviderGraphProfileV1 {
    /// Binds the source adaptation and the exact upstream schema it admitted.
    pub fn try_new(
        adaptation_recipe_id: impl Into<String>,
        schema_source_sha256: DigestV1,
        semantic_schema_sha256: DigestV1,
    ) -> Result<Self, FailureV1> {
        Ok(Self {
            adaptation_recipe_id: validated_identity(adaptation_recipe_id.into())?,
            schema_source_sha256,
            semantic_schema_sha256,
        })
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.adaptation_recipe_id)?;
        hash.digest(self.schema_source_sha256);
        hash.digest(self.semantic_schema_sha256);
        Ok(())
    }
}

/// Exact renderer, parser, provider graph, grammar, and Buck consumer profile.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationQualificationV1 {
    pub(crate) renderer: ArtifactIdentityV1,
    pub(crate) parser: ArtifactIdentityV1,
    pub(crate) provider_graph: ProviderGraphProfileV1,
    pub(crate) grammar_sha256: DigestV1,
    pub(crate) buck_consumer: BuckConsumerProfileV1,
}

impl GenerationQualificationV1 {
    /// Creates a qualification tuple whose every component rekeys the request.
    #[must_use]
    pub const fn new(
        renderer: ArtifactIdentityV1,
        parser: ArtifactIdentityV1,
        provider_graph: ProviderGraphProfileV1,
        grammar_sha256: DigestV1,
        buck_consumer: BuckConsumerProfileV1,
    ) -> Self {
        Self {
            renderer,
            parser,
            provider_graph,
            grammar_sha256,
            buck_consumer,
        }
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.renderer.encode_fields(hash)?;
        self.parser.encode_fields(hash)?;
        self.provider_graph.encode(hash)?;
        hash.digest(self.grammar_sha256);
        self.buck_consumer.encode(hash)
    }
}
