/// Producer-owned mechanical adoption-review projection for one exact release.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyReleaseAxesV1 {
    source: LifecycleSourceV1,
    checksum_sha256: DigestV1,
    publication: DependencyPublicationV1,
    metadata: DependencyMetadataV1,
    build_surface: DependencyBuildSurfaceV1,
    evidence: DependencyReleaseEvidenceV1,
    identities: [DigestV1; DependencyChangeAxisV1::COUNT],
}

impl DependencyReleaseAxesV1 {
    pub(crate) fn new(
        source: LifecycleSourceV1,
        checksum_sha256: DigestV1,
        publication: DependencyPublicationV1,
        metadata: DependencyMetadataV1,
        build_surface: DependencyBuildSurfaceV1,
        evidence: DependencyReleaseEvidenceV1,
    ) -> Self {
        let identities = [
            derive_dependency_axis(DependencyChangeAxisV1::Source, |hash| {
                hash.digest(source.identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Checksum, |hash| {
                hash.digest(checksum_sha256);
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Publication, |hash| {
                publication.encode(hash);
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Maintainers, |hash| {
                hash.digest(metadata.maintainers().identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::License, |hash| {
                hash.digest(metadata.license().identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Features, |hash| {
                hash.digest(metadata.features().identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Msrv, |hash| {
                metadata.msrv().encode(hash);
            }),
            derive_dependency_axis(DependencyChangeAxisV1::BuildScript, |hash| {
                match build_surface.build_script_sha256() {
                    None => hash.tag(0),
                    Some(identity) => {
                        hash.tag(1);
                        hash.digest(identity);
                    }
                }
            }),
            derive_dependency_axis(DependencyChangeAxisV1::ProcMacro, |hash| {
                hash.tag(u8::from(build_surface.proc_macro()));
            }),
            derive_dependency_axis(DependencyChangeAxisV1::NativeInputs, |hash| {
                hash.digest(build_surface.native_inputs().identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::DependencyManifest, |hash| {
                hash.digest(evidence.dependency_manifest_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Advisories, |hash| {
                hash.digest(evidence.advisories().identity_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Audit, |hash| {
                hash.digest(evidence.audit_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Provenance, |hash| {
                hash.digest(evidence.provenance_sha256());
            }),
            derive_dependency_axis(DependencyChangeAxisV1::Sbom, |hash| {
                hash.digest(evidence.sbom_sha256());
            }),
        ];
        Self {
            source,
            checksum_sha256,
            publication,
            metadata,
            build_surface,
            evidence,
            identities,
        }
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) {
        hash.u64(DependencyChangeAxisV1::COUNT as u64);
        for axis in DependencyChangeAxisV1::ALL {
            hash.tag(axis as u8);
            hash.digest(self.identity_sha256(axis));
        }
    }

    #[must_use]
    pub const fn identity_sha256(&self, axis: DependencyChangeAxisV1) -> DigestV1 {
        self.identities[axis as usize]
    }
}

fn derive_dependency_axis(
    axis: DependencyChangeAxisV1,
    encode: impl FnOnce(&mut CanonicalHasherV1),
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.dependency-release-axis.v1\0");
    hash.tag(axis as u8);
    encode(&mut hash);
    hash.finish()
}
