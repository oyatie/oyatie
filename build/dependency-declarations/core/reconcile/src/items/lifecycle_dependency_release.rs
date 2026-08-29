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
}

/// One immutable, provenance-bound Cargo dependency release fact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CargoDependencyReleaseV1 {
    package: CargoPackageIdentityV1,
    version: CargoVersionV1,
    axes: DependencyReleaseAxesV1,
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
        let DependencyReleaseCoordinatesV1 {
            source,
            package,
            version,
            checksum_sha256,
            publication,
        } = coordinates;
        validate_dependency_source(&source, qualification)?;
        let axes = DependencyReleaseAxesV1::new(
            source,
            checksum_sha256,
            publication,
            metadata,
            build_surface,
            evidence,
        );
        let mut hash = CanonicalHasherV1::new(b"build.cargo-dependency-release.v1\0");
        hash.digest(package.identity_sha256());
        hash.digest(version.identity_sha256());
        axes.encode(&mut hash);
        qualification.encode(&mut hash);
        Ok(Self {
            package,
            version,
            axes,
            qualification,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        &self.axes.source
    }

    #[must_use]
    pub const fn package(&self) -> &CargoPackageIdentityV1 {
        &self.package
    }

    #[must_use]
    pub const fn version(&self) -> &CargoVersionV1 {
        &self.version
    }

    #[must_use]
    pub const fn checksum_sha256(&self) -> DigestV1 {
        self.axes.checksum_sha256
    }

    #[must_use]
    pub const fn publication(&self) -> DependencyPublicationV1 {
        self.axes.publication
    }

    #[must_use]
    pub const fn metadata(&self) -> &DependencyMetadataV1 {
        &self.axes.metadata
    }

    #[must_use]
    pub const fn build_surface(&self) -> &DependencyBuildSurfaceV1 {
        &self.axes.build_surface
    }

    #[must_use]
    pub const fn evidence(&self) -> &DependencyReleaseEvidenceV1 {
        &self.axes.evidence
    }

    #[must_use]
    pub const fn axes(&self) -> &DependencyReleaseAxesV1 {
        &self.axes
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
