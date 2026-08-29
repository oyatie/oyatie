/// Mechanical blocker to running dependency qualification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DependencyQualificationBlockerV1 {
    PublicationAge = 0,
    MaintainerChangeHold = 1,
    MsrvEvidence = 2,
    MsrvFloorDecision = 3,
}

/// Suggested qualification path; neither variant accepts the dependency.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyQualificationModeV1 {
    Standard,
    ExpeditedSecurity {
        exception_identity_sha256: DigestV1,
    },
}

impl DependencyQualificationModeV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Standard => hash.tag(0),
            Self::ExpeditedSecurity {
                exception_identity_sha256,
            } => {
                hash.tag(1);
                hash.digest(exception_identity_sha256);
            }
        }
    }
}

/// Nonbinding next-step recommendation over exact Build facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyQualificationRecommendationV1 {
    candidate_identity_sha256: DigestV1,
    impact_identity_sha256: DigestV1,
    fact_envelope: FactEnvelopeV1,
    msrv: DependencyMsrvCompatibilityV1,
    quarantine: DependencyQuarantineV1,
    blockers: Box<[DependencyQualificationBlockerV1]>,
    mode: Option<DependencyQualificationModeV1>,
    identity_sha256: DigestV1,
}

impl DependencyQualificationRecommendationV1 {
    pub fn try_new(
        candidate: &DependencyCandidateV1,
        impact: &DependencyImpactV1,
        msrv: &DependencyMsrvCompatibilityV1,
        quarantine: &DependencyQuarantineV1,
        evaluated_at: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        impact.fact_envelope().require_safe(evaluated_at)?;
        let candidate_identity_sha256 = candidate.identity_sha256();
        if impact.candidate_identity_sha256() != candidate_identity_sha256
            || impact.current_release_identity_sha256()
                != candidate.current().identity_sha256()
            || msrv.candidate_identity_sha256() != candidate_identity_sha256
            || quarantine.candidate_identity_sha256() != candidate_identity_sha256
            || quarantine.evaluated_at() != evaluated_at
        {
            return Err(dependency_analysis_mismatch());
        }

        let mut blockers = Vec::with_capacity(4);
        record_quarantine_blocker(
            &mut blockers,
            quarantine.publication_age(),
            DependencyQualificationBlockerV1::PublicationAge,
        );
        record_quarantine_blocker(
            &mut blockers,
            quarantine.maintainer_change(),
            DependencyQualificationBlockerV1::MaintainerChangeHold,
        );
        match msrv.proposed() {
            DependencyMsrvRelationV1::WithinDeclaredFloor { .. } => {}
            DependencyMsrvRelationV1::RequiresHigherFloor { .. } => {
                blockers.push(DependencyQualificationBlockerV1::MsrvFloorDecision);
            }
            DependencyMsrvRelationV1::UnprovenAbsent { .. }
            | DependencyMsrvRelationV1::UnprovenUnknown { .. } => {
                blockers.push(DependencyQualificationBlockerV1::MsrvEvidence);
            }
        }
        blockers.sort_unstable();
        blockers.dedup();
        let mode = if blockers.is_empty() {
            Some(
                quarantine
                    .active_security_exception_identity_sha256()
                    .map_or(DependencyQualificationModeV1::Standard, |identity| {
                        DependencyQualificationModeV1::ExpeditedSecurity {
                            exception_identity_sha256: identity,
                        }
                    }),
            )
        } else {
            None
        };

        let impact_identity_sha256 = impact.identity_sha256();
        let fact_envelope = impact.fact_envelope().clone();
        let mut hash =
            CanonicalHasherV1::new(b"build.dependency-qualification-recommendation.v1\0");
        hash.digest(candidate_identity_sha256);
        hash.digest(impact_identity_sha256);
        hash.digest(fact_envelope.identity_sha256());
        hash.digest(msrv.identity_sha256());
        hash.digest(quarantine.identity_sha256());
        hash.u64(lifecycle_len(blockers.len())?);
        for blocker in &blockers {
            hash.tag(*blocker as u8);
        }
        match mode {
            None => hash.tag(0),
            Some(mode) => {
                hash.tag(1);
                mode.encode(&mut hash);
            }
        }
        Ok(Self {
            candidate_identity_sha256,
            impact_identity_sha256,
            fact_envelope,
            msrv: msrv.clone(),
            quarantine: quarantine.clone(),
            blockers: blockers.into_boxed_slice(),
            mode,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn impact_identity_sha256(&self) -> DigestV1 {
        self.impact_identity_sha256
    }

    #[must_use]
    pub const fn fact_envelope(&self) -> &FactEnvelopeV1 {
        &self.fact_envelope
    }

    #[must_use]
    pub const fn msrv(&self) -> &DependencyMsrvCompatibilityV1 {
        &self.msrv
    }

    #[must_use]
    pub const fn quarantine(&self) -> &DependencyQuarantineV1 {
        &self.quarantine
    }

    #[must_use]
    pub fn blockers(&self) -> &[DependencyQualificationBlockerV1] {
        &self.blockers
    }

    #[must_use]
    pub const fn mode(&self) -> Option<DependencyQualificationModeV1> {
        self.mode
    }

    #[must_use]
    pub const fn is_ready_for_qualification(&self) -> bool {
        self.mode.is_some()
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn record_quarantine_blocker(
    blockers: &mut Vec<DependencyQualificationBlockerV1>,
    gate: DependencyQuarantineGateV1,
    blocker: DependencyQualificationBlockerV1,
) {
    if gate.is_held() {
        blockers.push(blocker);
    }
}
