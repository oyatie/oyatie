#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum NormalizedAffectedStateV1 {
    ReferenceOnly,
    Candidate(AdvisoryAffectedSetV1),
    Qualified(AdvisoryAffectedSetV1),
}

impl NormalizedAffectedStateV1 {
    fn qualification(&self) -> NormalizedAdvisoryAffectedSetQualificationV1 {
        match self {
            Self::ReferenceOnly => NormalizedAdvisoryAffectedSetQualificationV1::ReferenceOnly,
            Self::Candidate(_) => NormalizedAdvisoryAffectedSetQualificationV1::Candidate,
            Self::Qualified(_) => NormalizedAdvisoryAffectedSetQualificationV1::Qualified,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::ReferenceOnly => hash.tag(0),
            Self::Candidate(affected) => {
                hash.tag(1);
                hash.digest(affected.identity_sha256());
            }
            Self::Qualified(affected) => {
                hash.tag(2);
                hash.digest(affected.identity_sha256());
            }
        }
    }
}
