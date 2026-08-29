/// The five immutable declaration inputs.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationInputsV1 {
    manifest: InputFileV1,
    lock: InputFileV1,
    config: InputFileV1,
    fixups: InputTreeV1,
    cargo_sources: InputTreeV1,
}

impl GenerationInputsV1 {
    /// Creates a role-correct input tuple with no repeated manifest path.
    pub fn try_new(
        manifest: InputFileV1,
        lock: InputFileV1,
        config: InputFileV1,
        fixups: InputTreeV1,
        cargo_sources: InputTreeV1,
    ) -> Result<Self, FailureV1> {
        if manifest.role != InputFileRoleV1::Manifest
            || lock.role != InputFileRoleV1::Lock
            || config.role != InputFileRoleV1::Config
            || fixups.role != TreeRoleV1::Fixups
            || cargo_sources.role != TreeRoleV1::CargoSource
        {
            return Err(invalid_request());
        }
        let paths = [
            manifest.path.as_str(),
            lock.path.as_str(),
            config.path.as_str(),
            fixups.manifest.path.as_str(),
            cargo_sources.manifest.path.as_str(),
        ];
        if paths
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != paths.len()
        {
            return Err(invalid_request());
        }
        Ok(Self {
            manifest,
            lock,
            config,
            fixups,
            cargo_sources,
        })
    }

    /// Returns the root Cargo manifest input.
    #[must_use]
    pub const fn manifest(&self) -> &InputFileV1 {
        &self.manifest
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.manifest.encode(hash)?;
        self.lock.encode(hash)?;
        self.config.encode(hash)?;
        self.fixups.encode(hash)?;
        self.cargo_sources.encode(hash)
    }
}

/// Exact generator, compiler tools, and semantic qualification tuple.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationToolsV1 {
    generator: GeneratorIdentityV1,
    cargo: ToolIdentityV1,
    rustc: ToolIdentityV1,
    qualification: GenerationQualificationV1,
}

impl GenerationToolsV1 {
    /// Groups already validated tool identities.
    #[must_use]
    pub const fn new(
        generator: GeneratorIdentityV1,
        cargo: ToolIdentityV1,
        rustc: ToolIdentityV1,
        qualification: GenerationQualificationV1,
    ) -> Self {
        Self {
            generator,
            cargo,
            rustc,
            qualification,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.generator.encode(hash)?;
        self.cargo.encode(hash)?;
        self.rustc.encode(hash)?;
        self.qualification.encode(hash)
    }
}

/// Closed platform and execution profiles for one generation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationExecutionV1 {
    platforms: PlatformSetV1,
    environment: EnvironmentProfileV1,
    sandbox: SandboxProfileV1,
    validator: ValidatorProfileV1,
    bounds: ValidationBoundsV1,
}

impl GenerationExecutionV1 {
    /// Groups the closed execution profiles.
    #[must_use]
    pub const fn new(
        platforms: PlatformSetV1,
        environment: EnvironmentProfileV1,
        sandbox: SandboxProfileV1,
        validator: ValidatorProfileV1,
        bounds: ValidationBoundsV1,
    ) -> Self {
        Self {
            platforms,
            environment,
            sandbox,
            validator,
            bounds,
        }
    }
}

/// Fully admitted immutable input to one deterministic generation transaction.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationRequestV1 {
    repository: RepositoryCorrelationV1,
    inputs: GenerationInputsV1,
    tools: GenerationToolsV1,
    execution: GenerationExecutionV1,
    request_id: DigestV1,
}

impl GenerationRequestV1 {
    /// Admits the complete request and computes its domain-separated identity once.
    pub fn try_new(
        repository: RepositoryCorrelationV1,
        inputs: GenerationInputsV1,
        tools: GenerationToolsV1,
        execution: GenerationExecutionV1,
    ) -> Result<Self, FailureV1> {
        let mut request = Self {
            repository,
            inputs,
            tools,
            execution,
            request_id: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.declaration-request.v1\0");
        request.encode_fields(&mut hash)?;
        request.request_id = hash.finish();
        Ok(request)
    }

    /// Returns the cached identity of all admitted generation values.
    #[must_use]
    pub const fn request_id(&self) -> DigestV1 {
        self.request_id
    }

    /// Returns the immutable declared inputs.
    #[must_use]
    pub const fn inputs(&self) -> &GenerationInputsV1 {
        &self.inputs
    }

    /// Returns the exact parser-profile identity expected from the projection port.
    #[must_use]
    pub const fn parser_identity(&self) -> DigestV1 {
        self.tools.qualification.parser.identity_sha256()
    }

    pub(crate) const fn validator(&self) -> ValidatorProfileV1 {
        self.execution.validator
    }

    fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.repository.encode(hash)?;
        self.inputs.encode(hash)?;
        self.execution.platforms.encode(hash)?;
        self.tools.encode(hash)?;
        hash.tag(match self.execution.environment {
            EnvironmentProfileV1::ReindeerHermeticV1 => 0,
        });
        hash.tag(match self.execution.sandbox {
            SandboxProfileV1::DeclaredReadStageWriteNoNetworkV1 => 0,
        });
        hash.tag(match self.execution.validator {
            ValidatorProfileV1::ReindeerBuckV1 => 0,
        });
        hash.tag(self.execution.bounds.tag());
        Ok(())
    }
}
