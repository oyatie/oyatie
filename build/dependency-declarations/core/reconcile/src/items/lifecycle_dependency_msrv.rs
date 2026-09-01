/// Declaration-only relation to the consuming workspace's tested Rust floor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyMsrvRelationV1 {
    WithinDeclaredFloor {
        required: RustVersionV1,
        evidence_sha256: DigestV1,
    },
    RequiresHigherFloor {
        required: RustVersionV1,
        evidence_sha256: DigestV1,
    },
    UnprovenAbsent {
        evidence_sha256: DigestV1,
    },
    UnprovenUnknown {
        evidence_sha256: DigestV1,
    },
}

impl DependencyMsrvRelationV1 {
    fn from_declaration(
        declaration: DependencyMsrvDeclarationV1,
        declared_floor: RustVersionV1,
    ) -> Self {
        match declaration {
            DependencyMsrvDeclarationV1::Declared {
                version,
                evidence_sha256,
            } if version <= declared_floor => Self::WithinDeclaredFloor {
                required: version,
                evidence_sha256,
            },
            DependencyMsrvDeclarationV1::Declared {
                version,
                evidence_sha256,
            } => Self::RequiresHigherFloor {
                required: version,
                evidence_sha256,
            },
            DependencyMsrvDeclarationV1::Absent { evidence_sha256 } => {
                Self::UnprovenAbsent { evidence_sha256 }
            }
            DependencyMsrvDeclarationV1::Unknown { evidence_sha256 } => {
                Self::UnprovenUnknown { evidence_sha256 }
            }
        }
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::WithinDeclaredFloor {
                required,
                evidence_sha256,
            } => {
                hash.tag(0);
                required.encode(hash);
                hash.digest(evidence_sha256);
            }
            Self::RequiresHigherFloor {
                required,
                evidence_sha256,
            } => {
                hash.tag(1);
                required.encode(hash);
                hash.digest(evidence_sha256);
            }
            Self::UnprovenAbsent { evidence_sha256 } => {
                hash.tag(2);
                hash.digest(evidence_sha256);
            }
            Self::UnprovenUnknown { evidence_sha256 } => {
                hash.tag(3);
                hash.digest(evidence_sha256);
            }
        }
    }
}

/// MSRV declaration precheck; candidate compilation remains a separate proof.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyMsrvCompatibilityV1 {
    candidate_identity_sha256: DigestV1,
    toolchain_matrix_identity_sha256: DigestV1,
    declared_msrv_profile_identity_sha256: DigestV1,
    stable_profile_identity_sha256: DigestV1,
    declared_msrv_version: RustVersionV1,
    stable_version: RustVersionV1,
    current: DependencyMsrvRelationV1,
    proposed: DependencyMsrvRelationV1,
    identity_sha256: DigestV1,
}

impl DependencyMsrvCompatibilityV1 {
    #[must_use]
    pub fn new(candidate: &DependencyCandidateV1, matrix: &ToolchainMatrixV1) -> Self {
        let declared_msrv_version = matrix.msrv().version();
        let current = DependencyMsrvRelationV1::from_declaration(
            candidate.current().metadata().msrv(),
            declared_msrv_version,
        );
        let proposed = DependencyMsrvRelationV1::from_declaration(
            candidate.proposed().metadata().msrv(),
            declared_msrv_version,
        );
        let candidate_identity_sha256 = candidate.identity_sha256();
        let toolchain_matrix_identity_sha256 = matrix.identity_sha256();
        let declared_msrv_profile_identity_sha256 = matrix.msrv().identity_sha256();
        let stable_profile_identity_sha256 = matrix.stable().identity_sha256();
        let stable_version = matrix.stable().version();
        let mut hash = CanonicalHasherV1::new(b"build.dependency-msrv-compatibility.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(toolchain_matrix_identity_sha256);
        hash.digest(declared_msrv_profile_identity_sha256);
        hash.digest(stable_profile_identity_sha256);
        declared_msrv_version.encode(&mut hash);
        stable_version.encode(&mut hash);
        current.encode(&mut hash);
        proposed.encode(&mut hash);
        Self {
            candidate_identity_sha256,
            toolchain_matrix_identity_sha256,
            declared_msrv_profile_identity_sha256,
            stable_profile_identity_sha256,
            declared_msrv_version,
            stable_version,
            current,
            proposed,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn toolchain_matrix_identity_sha256(&self) -> DigestV1 {
        self.toolchain_matrix_identity_sha256
    }

    #[must_use]
    pub const fn declared_msrv_profile_identity_sha256(&self) -> DigestV1 {
        self.declared_msrv_profile_identity_sha256
    }

    #[must_use]
    pub const fn stable_profile_identity_sha256(&self) -> DigestV1 {
        self.stable_profile_identity_sha256
    }

    #[must_use]
    pub const fn declared_msrv_version(&self) -> RustVersionV1 {
        self.declared_msrv_version
    }

    #[must_use]
    pub const fn stable_version(&self) -> RustVersionV1 {
        self.stable_version
    }

    #[must_use]
    pub const fn current(&self) -> DependencyMsrvRelationV1 {
        self.current
    }

    #[must_use]
    pub const fn proposed(&self) -> DependencyMsrvRelationV1 {
        self.proposed
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
