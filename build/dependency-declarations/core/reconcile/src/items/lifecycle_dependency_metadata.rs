/// Registry availability observed for one exact package release.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DependencyPublicationStateV1 {
    Available = 0,
    Yanked = 1,
    Deleted = 2,
}

/// Publication and current registry state at one observation time.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DependencyPublicationV1 {
    published_at: LifecycleTimestampV1,
    observed_at: LifecycleTimestampV1,
    state: DependencyPublicationStateV1,
    evidence_sha256: DigestV1,
}

impl DependencyPublicationV1 {
    pub fn try_new(
        published_at: LifecycleTimestampV1,
        observed_at: LifecycleTimestampV1,
        state: DependencyPublicationStateV1,
        evidence_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if published_at > observed_at {
            return Err(lifecycle_invalid());
        }
        Ok(Self {
            published_at,
            observed_at,
            state,
            evidence_sha256,
        })
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        hash.u64(self.published_at.unix_seconds());
        hash.u64(self.observed_at.unix_seconds());
        hash.tag(self.state as u8);
        hash.digest(self.evidence_sha256);
    }

    #[must_use]
    pub const fn state(self) -> DependencyPublicationStateV1 {
        self.state
    }

    #[must_use]
    pub const fn published_at(self) -> LifecycleTimestampV1 {
        self.published_at
    }

    #[must_use]
    pub const fn observed_at(self) -> LifecycleTimestampV1 {
        self.observed_at
    }

    #[must_use]
    pub const fn evidence_sha256(self) -> DigestV1 {
        self.evidence_sha256
    }
}

/// Exact Cargo `rust-version` declaration state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyMsrvDeclarationV1 {
    Declared {
        version: RustVersionV1,
        evidence_sha256: DigestV1,
    },
    Absent {
        evidence_sha256: DigestV1,
    },
    Unknown {
        evidence_sha256: DigestV1,
    },
}

impl DependencyMsrvDeclarationV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Declared {
                version,
                evidence_sha256,
            } => {
                hash.tag(0);
                version.encode(hash);
                hash.digest(evidence_sha256);
            }
            Self::Absent { evidence_sha256 } => {
                hash.tag(1);
                hash.digest(evidence_sha256);
            }
            Self::Unknown { evidence_sha256 } => {
                hash.tag(2);
                hash.digest(evidence_sha256);
            }
        }
    }
}

/// Build-time capability surface of one exact dependency release.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyBuildSurfaceV1 {
    build_script_sha256: Option<DigestV1>,
    proc_macro: bool,
    native_inputs: DependencyNamedFactSetV1,
}

impl DependencyBuildSurfaceV1 {
    #[must_use]
    pub const fn new(
        build_script_sha256: Option<DigestV1>,
        proc_macro: bool,
        native_inputs: DependencyNamedFactSetV1,
    ) -> Self {
        Self {
            build_script_sha256,
            proc_macro,
            native_inputs,
        }
    }

    #[must_use]
    pub const fn build_script_sha256(&self) -> Option<DigestV1> {
        self.build_script_sha256
    }

    #[must_use]
    pub const fn proc_macro(&self) -> bool {
        self.proc_macro
    }

    #[must_use]
    pub const fn native_inputs(&self) -> &DependencyNamedFactSetV1 {
        &self.native_inputs
    }
}

/// Exact registry metadata relevant to dependency adoption.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyMetadataV1 {
    maintainers: DependencyNamedFactSetV1,
    license: DependencyLicenseV1,
    features: DependencyNamedFactSetV1,
    msrv: DependencyMsrvDeclarationV1,
}

impl DependencyMetadataV1 {
    #[must_use]
    pub const fn new(
        maintainers: DependencyNamedFactSetV1,
        license: DependencyLicenseV1,
        features: DependencyNamedFactSetV1,
        msrv: DependencyMsrvDeclarationV1,
    ) -> Self {
        Self {
            maintainers,
            license,
            features,
            msrv,
        }
    }

    #[must_use]
    pub const fn maintainers(&self) -> &DependencyNamedFactSetV1 {
        &self.maintainers
    }

    #[must_use]
    pub const fn license(&self) -> &DependencyLicenseV1 {
        &self.license
    }

    #[must_use]
    pub const fn features(&self) -> &DependencyNamedFactSetV1 {
        &self.features
    }

    #[must_use]
    pub const fn msrv(&self) -> DependencyMsrvDeclarationV1 {
        self.msrv
    }
}

/// Exact graph, advisory, audit, provenance, and SBOM evidence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyReleaseEvidenceV1 {
    dependency_manifest_sha256: DigestV1,
    advisories: DependencyAdvisorySetV1,
    audit_sha256: DigestV1,
    provenance_sha256: DigestV1,
    sbom_sha256: DigestV1,
}

impl DependencyReleaseEvidenceV1 {
    #[must_use]
    pub const fn new(
        dependency_manifest_sha256: DigestV1,
        advisories: DependencyAdvisorySetV1,
        audit_sha256: DigestV1,
        provenance_sha256: DigestV1,
        sbom_sha256: DigestV1,
    ) -> Self {
        Self {
            dependency_manifest_sha256,
            advisories,
            audit_sha256,
            provenance_sha256,
            sbom_sha256,
        }
    }

    #[must_use]
    pub const fn dependency_manifest_sha256(&self) -> DigestV1 {
        self.dependency_manifest_sha256
    }

    #[must_use]
    pub const fn advisories(&self) -> &DependencyAdvisorySetV1 {
        &self.advisories
    }

    #[must_use]
    pub const fn audit_sha256(&self) -> DigestV1 {
        self.audit_sha256
    }

    #[must_use]
    pub const fn provenance_sha256(&self) -> DigestV1 {
        self.provenance_sha256
    }

    #[must_use]
    pub const fn sbom_sha256(&self) -> DigestV1 {
        self.sbom_sha256
    }
}
