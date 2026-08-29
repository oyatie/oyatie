/// Stable mechanical change axis for dependency candidate review.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyChangeAxisV1 {
    Source = 0,
    Checksum = 1,
    PublicationState = 2,
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

/// Canonical changed axes between two exact dependency releases.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyCandidateDeltaV1 {
    changed: Box<[DependencyChangeAxisV1]>,
}

impl DependencyCandidateDeltaV1 {
    fn between(current: &CargoDependencyReleaseV1, proposed: &CargoDependencyReleaseV1) -> Self {
        let mut changed = Vec::with_capacity(15);
        record_dependency_change(
            &mut changed,
            current.source().identity_sha256() != proposed.source().identity_sha256(),
            DependencyChangeAxisV1::Source,
        );
        record_dependency_change(
            &mut changed,
            current.checksum_sha256() != proposed.checksum_sha256(),
            DependencyChangeAxisV1::Checksum,
        );
        record_dependency_change(
            &mut changed,
            current.publication().state() != proposed.publication().state(),
            DependencyChangeAxisV1::PublicationState,
        );
        record_dependency_change(
            &mut changed,
            current.metadata().maintainers() != proposed.metadata().maintainers(),
            DependencyChangeAxisV1::Maintainers,
        );
        record_dependency_change(
            &mut changed,
            !current
                .metadata()
                .license()
                .same_declaration(proposed.metadata().license()),
            DependencyChangeAxisV1::License,
        );
        record_dependency_change(
            &mut changed,
            current.metadata().features() != proposed.metadata().features(),
            DependencyChangeAxisV1::Features,
        );
        record_dependency_change(
            &mut changed,
            !current
                .metadata()
                .msrv()
                .same_declaration(proposed.metadata().msrv()),
            DependencyChangeAxisV1::Msrv,
        );
        record_dependency_change(
            &mut changed,
            current.build_surface().build_script_sha256()
                != proposed.build_surface().build_script_sha256(),
            DependencyChangeAxisV1::BuildScript,
        );
        record_dependency_change(
            &mut changed,
            current.build_surface().proc_macro() != proposed.build_surface().proc_macro(),
            DependencyChangeAxisV1::ProcMacro,
        );
        record_dependency_change(
            &mut changed,
            current.build_surface().native_inputs()
                != proposed.build_surface().native_inputs(),
            DependencyChangeAxisV1::NativeInputs,
        );
        record_dependency_change(
            &mut changed,
            current.evidence().dependency_manifest_sha256()
                != proposed.evidence().dependency_manifest_sha256(),
            DependencyChangeAxisV1::DependencyManifest,
        );
        record_dependency_change(
            &mut changed,
            current.evidence().advisories() != proposed.evidence().advisories(),
            DependencyChangeAxisV1::Advisories,
        );
        record_dependency_change(
            &mut changed,
            current.evidence().audit_sha256() != proposed.evidence().audit_sha256(),
            DependencyChangeAxisV1::Audit,
        );
        record_dependency_change(
            &mut changed,
            current.evidence().provenance_sha256()
                != proposed.evidence().provenance_sha256(),
            DependencyChangeAxisV1::Provenance,
        );
        record_dependency_change(
            &mut changed,
            current.evidence().sbom_sha256() != proposed.evidence().sbom_sha256(),
            DependencyChangeAxisV1::Sbom,
        );
        changed.sort_unstable();
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

fn record_dependency_change(
    changed: &mut Vec<DependencyChangeAxisV1>,
    condition: bool,
    axis: DependencyChangeAxisV1,
) {
    if condition {
        changed.push(axis);
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
