/// Semantic kind of one normalized dependency-graph unit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyGraphNodeKindV1 {
    CargoPackage = 0,
    CargoTarget = 1,
    BuckTarget = 2,
    CargoFeature = 3,
    NativeArtifact = 4,
    GeneratedArtifact = 5,
}

/// Execution side used by one configured dependency unit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyExecutionDomainV1 {
    Target = 0,
    Host = 1,
    PlatformIndependent = 2,
}

/// Role of a directed dependent-to-dependency edge.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyGraphEdgeKindV1 {
    NormalTarget = 0,
    BuildHost = 1,
    DevTarget = 2,
    ProcMacroHost = 3,
    NativeHost = 4,
    FeatureActivation = 5,
    ConfiguredBuck = 6,
}

/// One exact compilation, package, feature, or build unit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyGraphNodeV1 {
    unit_identity_sha256: DigestV1,
    kind: DependencyGraphNodeKindV1,
    execution_domain: DependencyExecutionDomainV1,
    package_release_identity_sha256: Option<DigestV1>,
    platform_scope_sha256: DigestV1,
    evidence_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyGraphNodeV1 {
    #[must_use]
    pub fn new(
        unit_identity_sha256: DigestV1,
        kind: DependencyGraphNodeKindV1,
        execution_domain: DependencyExecutionDomainV1,
        package_release_identity_sha256: Option<DigestV1>,
        platform_scope_sha256: DigestV1,
        evidence_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.dependency-graph-node.v1\0");
        hash.digest(unit_identity_sha256);
        hash.tag(kind as u8);
        hash.tag(execution_domain as u8);
        match package_release_identity_sha256 {
            Some(identity) => {
                hash.tag(1);
                hash.digest(identity);
            }
            None => hash.tag(0),
        }
        hash.digest(platform_scope_sha256);
        hash.digest(evidence_sha256);
        Self {
            unit_identity_sha256,
            kind,
            execution_domain,
            package_release_identity_sha256,
            platform_scope_sha256,
            evidence_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn unit_identity_sha256(&self) -> DigestV1 {
        self.unit_identity_sha256
    }

    #[must_use]
    pub const fn kind(&self) -> DependencyGraphNodeKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn execution_domain(&self) -> DependencyExecutionDomainV1 {
        self.execution_domain
    }

    #[must_use]
    pub const fn package_release_identity_sha256(&self) -> Option<DigestV1> {
        self.package_release_identity_sha256
    }

    #[must_use]
    pub const fn platform_scope_sha256(&self) -> DigestV1 {
        self.platform_scope_sha256
    }

    #[must_use]
    pub const fn evidence_sha256(&self) -> DigestV1 {
        self.evidence_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DependencyGraphEdgeKeyV1 {
    dependent_unit_sha256: DigestV1,
    dependency_unit_sha256: DigestV1,
    kind: DependencyGraphEdgeKindV1,
    configuration_sha256: DigestV1,
}

/// One normalized edge directed from dependent to dependency.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyGraphEdgeV1 {
    dependent_unit_sha256: DigestV1,
    dependency_unit_sha256: DigestV1,
    kind: DependencyGraphEdgeKindV1,
    configuration_sha256: DigestV1,
    evidence_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyGraphEdgeV1 {
    pub fn new(
        dependent_unit_sha256: DigestV1,
        dependency_unit_sha256: DigestV1,
        kind: DependencyGraphEdgeKindV1,
        configuration_sha256: DigestV1,
        evidence_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if dependent_unit_sha256 == dependency_unit_sha256 {
            return Err(invalid_dependency_graph());
        }
        let mut hash = CanonicalHasherV1::new(b"build.dependency-graph-edge.v1\0");
        hash.digest(dependent_unit_sha256);
        hash.digest(dependency_unit_sha256);
        hash.tag(kind as u8);
        hash.digest(configuration_sha256);
        hash.digest(evidence_sha256);
        Ok(Self {
            dependent_unit_sha256,
            dependency_unit_sha256,
            kind,
            configuration_sha256,
            evidence_sha256,
            identity_sha256: hash.finish(),
        })
    }

    pub(crate) const fn semantic_key(&self) -> DependencyGraphEdgeKeyV1 {
        DependencyGraphEdgeKeyV1 {
            dependent_unit_sha256: self.dependent_unit_sha256,
            dependency_unit_sha256: self.dependency_unit_sha256,
            kind: self.kind,
            configuration_sha256: self.configuration_sha256,
        }
    }

    #[must_use]
    pub const fn dependent_unit_sha256(&self) -> DigestV1 {
        self.dependent_unit_sha256
    }

    #[must_use]
    pub const fn dependency_unit_sha256(&self) -> DigestV1 {
        self.dependency_unit_sha256
    }

    #[must_use]
    pub const fn kind(&self) -> DependencyGraphEdgeKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn configuration_sha256(&self) -> DigestV1 {
        self.configuration_sha256
    }

    #[must_use]
    pub const fn evidence_sha256(&self) -> DigestV1 {
        self.evidence_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

pub(crate) const fn invalid_dependency_graph() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidDependencyGraph)
}
