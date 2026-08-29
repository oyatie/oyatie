/// Stable mechanical change axis for dependency candidate review.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyChangeAxisV1 {
    Source = 0,
    Checksum = 1,
    Publication = 2,
    Maintainers = 3,
    License = 4,
    Features = 5,
    Msrv = 6,
    BuildScript = 7,
    ProcMacro = 8,
    NativeInputs = 9,
    DependencyManifest = 10,
    Advisories = 11,
    Audit = 12,
    Provenance = 13,
    Sbom = 14,
}

impl DependencyChangeAxisV1 {
    pub const ALL: [Self; 15] = [
        Self::Source,
        Self::Checksum,
        Self::Publication,
        Self::Maintainers,
        Self::License,
        Self::Features,
        Self::Msrv,
        Self::BuildScript,
        Self::ProcMacro,
        Self::NativeInputs,
        Self::DependencyManifest,
        Self::Advisories,
        Self::Audit,
        Self::Provenance,
        Self::Sbom,
    ];
    pub const COUNT: usize = Self::ALL.len();
}

/// Canonical changed axes between two exact dependency releases.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCandidateDeltaV1 {
    changed: Box<[DependencyChangeAxisV1]>,
}

impl DependencyCandidateDeltaV1 {
    fn between(current: &CargoDependencyReleaseV1, proposed: &CargoDependencyReleaseV1) -> Self {
        let mut changed = Vec::with_capacity(DependencyChangeAxisV1::COUNT);
        for axis in DependencyChangeAxisV1::ALL {
            if current.axes().identity_sha256(axis) != proposed.axes().identity_sha256(axis) {
                changed.push(axis);
            }
        }
        Self {
            changed: changed.into_boxed_slice(),
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        hash.u64(lifecycle_len(self.changed.len())?);
        for axis in &self.changed {
            hash.tag(*axis as u8);
        }
        Ok(())
    }

    #[must_use]
    pub fn changed(&self, axis: DependencyChangeAxisV1) -> bool {
        self.changed.binary_search(&axis).is_ok()
    }

    #[must_use]
    pub fn axes(&self) -> &[DependencyChangeAxisV1] {
        &self.changed
    }
}

/// One comparable upgrade candidate without acceptance, qualification, or campaign state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCandidateV1 {
    current: CargoDependencyReleaseV1,
    proposed: CargoDependencyReleaseV1,
    delta: DependencyCandidateDeltaV1,
    discovery_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyCandidateV1 {
    pub fn try_new(
        current: CargoDependencyReleaseV1,
        proposed: CargoDependencyReleaseV1,
        discovery_receipt_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if current.package() != proposed.package()
            || proposed.version().precedence_cmp(current.version())
                != std::cmp::Ordering::Greater
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::InvalidDependencyCandidate,
            ));
        }
        if !current.qualification().is_qualified() || !proposed.qualification().is_qualified() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnqualifiedExtraction,
            ));
        }
        if proposed.publication().state() != DependencyPublicationStateV1::Available {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnavailableDependencyRelease,
            ));
        }
        let delta = DependencyCandidateDeltaV1::between(&current, &proposed);
        let mut hash = CanonicalHasherV1::new(b"build.dependency-candidate.v1\0");
        hash.digest(current.identity_sha256());
        hash.digest(proposed.identity_sha256());
        delta.encode(&mut hash)?;
        hash.digest(discovery_receipt_sha256);
        Ok(Self {
            current,
            proposed,
            delta,
            discovery_receipt_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn current(&self) -> &CargoDependencyReleaseV1 {
        &self.current
    }

    #[must_use]
    pub const fn proposed(&self) -> &CargoDependencyReleaseV1 {
        &self.proposed
    }

    #[must_use]
    pub const fn delta(&self) -> &DependencyCandidateDeltaV1 {
        &self.delta
    }

    #[must_use]
    pub const fn discovery_receipt_sha256(&self) -> DigestV1 {
        self.discovery_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
