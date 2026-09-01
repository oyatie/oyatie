/// Complete declared reads plus role-specific generation inputs.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationInputsV1 {
    manifest: InputFileV1,
    lock: InputFileV1,
    config: InputFileV1,
    repository_reads: InputTreeV1,
    fixups: InputTreeV1,
    cargo_home_reads: InputTreeV1,
}

impl GenerationInputsV1 {
    /// Creates a role-correct input tuple with no repeated manifest path.
    pub fn try_new(
        manifest: InputFileV1,
        lock: InputFileV1,
        config: InputFileV1,
        repository_reads: InputTreeV1,
        fixups: InputTreeV1,
        cargo_home_reads: InputTreeV1,
    ) -> Result<Self, FailureV1> {
        if manifest.role != InputFileRoleV1::Manifest
            || lock.role != InputFileRoleV1::Lock
            || config.role != InputFileRoleV1::Config
            || repository_reads.role != TreeRoleV1::RepositoryRead
            || fixups.role != TreeRoleV1::Fixups
            || cargo_home_reads.role != TreeRoleV1::CargoHomeRead
            || !repository_reads.contains_file(&manifest)
            || !repository_reads.contains_file(&lock)
            || !repository_reads.contains_file(&config)
            || !fixups
                .entries
                .iter()
                .all(|entry| repository_reads.contains_entry(entry))
        {
            return Err(invalid_request());
        }
        let paths = [
            manifest.path.as_str(),
            lock.path.as_str(),
            config.path.as_str(),
            repository_reads.manifest.path.as_str(),
            fixups.manifest.path.as_str(),
            cargo_home_reads.manifest.path.as_str(),
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
            repository_reads,
            fixups,
            cargo_home_reads,
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
        self.repository_reads.encode(hash)?;
        self.fixups.encode(hash)?;
        self.cargo_home_reads.encode(hash)
    }
}

/// Exact generator, compiler tools, and semantic qualification tuple.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationToolsV1 {
    generator: GeneratorIdentityV1,
    cargo: ToolIdentityV1,
    rustc: ToolIdentityV1,
    execution_runtime: ArtifactIdentityV1,
    qualification: GenerationQualificationV1,
}

impl GenerationToolsV1 {
    /// Groups already validated tool identities.
    #[must_use]
    pub const fn new(
        generator: GeneratorIdentityV1,
        cargo: ToolIdentityV1,
        rustc: ToolIdentityV1,
        execution_runtime: ArtifactIdentityV1,
        qualification: GenerationQualificationV1,
    ) -> Self {
        Self {
            generator,
            cargo,
            rustc,
            execution_runtime,
            qualification,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        self.generator.encode(hash)?;
        self.cargo.encode(hash)?;
        self.rustc.encode(hash)?;
        self.execution_runtime.encode_fields(hash)?;
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
    projection_profile_sha256: DigestV1,
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
            projection_profile_sha256: DigestV1::from_bytes([0; 32]),
            request_id: DigestV1::from_bytes([0; 32]),
        };
        request.projection_profile_sha256 = projection_profile_identity(&request)?;
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
        hash.digest(self.projection_profile_sha256);
        Ok(())
    }
}

fn projection_profile_identity(request: &GenerationRequestV1) -> Result<DigestV1, FailureV1> {
    let qualification = &request.tools.qualification;
    let mut hash = CanonicalHasherV1::new(b"build.declaration-projection-profile.v1\0");
    qualification.renderer.encode_fields(&mut hash)?;
    qualification.parser.encode_fields(&mut hash)?;
    qualification.provider_graph.encode(&mut hash)?;
    hash.digest(qualification.grammar_sha256);
    hash.tag(match request.execution.validator {
        ValidatorProfileV1::ReindeerBuckV1 => 0,
    });
    hash.tag(request.execution.bounds.tag());
    Ok(hash.finish())
}
