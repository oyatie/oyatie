/// Exact registry coordinates and publication state for one release.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyReleaseCoordinatesV1 {
    source: LifecycleSourceV1,
    package: CargoPackageIdentityV1,
    version: CargoVersionV1,
    checksum_sha256: DigestV1,
    publication: DependencyPublicationV1,
}

impl DependencyReleaseCoordinatesV1 {
    #[must_use]
    pub const fn new(
        source: LifecycleSourceV1,
        package: CargoPackageIdentityV1,
        version: CargoVersionV1,
        checksum_sha256: DigestV1,
        publication: DependencyPublicationV1,
    ) -> Self {
        Self {
            source,
            package,
            version,
            checksum_sha256,
            publication,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) {
        hash.digest(self.source.identity_sha256());
        hash.digest(self.package.identity_sha256());
        hash.digest(self.version.identity_sha256());
        hash.digest(self.checksum_sha256);
        self.publication.encode(hash);
    }
}

/// One immutable, provenance-bound Cargo dependency release fact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CargoDependencyReleaseV1 {
    coordinates: DependencyReleaseCoordinatesV1,
    metadata: DependencyMetadataV1,
    build_surface: DependencyBuildSurfaceV1,
    evidence: DependencyReleaseEvidenceV1,
    qualification: DependencyFactQualificationV1,
    identity_sha256: DigestV1,
}

impl CargoDependencyReleaseV1 {
    pub fn try_new(
        coordinates: DependencyReleaseCoordinatesV1,
        metadata: DependencyMetadataV1,
        build_surface: DependencyBuildSurfaceV1,
        evidence: DependencyReleaseEvidenceV1,
        qualification: DependencyFactQualificationV1,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_dependency_source(&coordinates.source, qualification)?;
        let mut value = Self {
            coordinates,
            metadata,
            build_surface,
            evidence,
            qualification,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.cargo-dependency-release.v1\0");
        value.coordinates.encode(&mut hash);
        value.metadata.encode(&mut hash);
        value.build_surface.encode(&mut hash);
        value.evidence.encode(&mut hash);
        value.qualification.encode(&mut hash);
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        &self.coordinates.source
    }

    #[must_use]
    pub const fn package(&self) -> &CargoPackageIdentityV1 {
        &self.coordinates.package
    }

    #[must_use]
    pub const fn version(&self) -> &CargoVersionV1 {
        &self.coordinates.version
    }

    #[must_use]
    pub const fn checksum_sha256(&self) -> DigestV1 {
        self.coordinates.checksum_sha256
    }

    #[must_use]
    pub const fn publication(&self) -> DependencyPublicationV1 {
        self.coordinates.publication
    }

    #[must_use]
    pub const fn metadata(&self) -> &DependencyMetadataV1 {
        &self.metadata
    }

    #[must_use]
    pub const fn build_surface(&self) -> &DependencyBuildSurfaceV1 {
        &self.build_surface
    }

    #[must_use]
    pub const fn evidence(&self) -> &DependencyReleaseEvidenceV1 {
        &self.evidence
    }

    #[must_use]
    pub const fn qualification(&self) -> DependencyFactQualificationV1 {
        self.qualification
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn validate_dependency_source(
    source: &LifecycleSourceV1,
    qualification: DependencyFactQualificationV1,
) -> Result<(), LifecycleFailureV1> {
    if source.component() != LifecycleComponentV1::DependencyRegistry
        || source.channel() != LifecycleChannelV1::Dependency
        || !matches!(source.descriptor().scope(), LifecycleSourceScopeV1::Global)
    {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::DependencySourceMismatch,
        ));
    }
    if qualification.is_qualified() && source.maturity() != SourceMaturityV1::Released {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::ProvisionalSource,
        ));
    }
    Ok(())
}
