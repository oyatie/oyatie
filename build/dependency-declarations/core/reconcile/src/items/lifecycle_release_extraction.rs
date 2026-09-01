/// Evidence state for one exact release-item extractor profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReleaseExtractionQualificationV1 {
    Candidate {
        observation_sha256: DigestV1,
    },
    Qualified {
        qualification_receipt_sha256: DigestV1,
    },
}

impl ReleaseExtractionQualificationV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Candidate { observation_sha256 } => {
                hash.tag(0);
                hash.digest(observation_sha256);
            }
            Self::Qualified {
                qualification_receipt_sha256,
            } => {
                hash.tag(1);
                hash.digest(qualification_receipt_sha256);
            }
        }
    }

    #[must_use]
    pub const fn is_qualified(self) -> bool {
        matches!(self, Self::Qualified { .. })
    }
}

/// Exact parser, grammar, source, and qualification identity for extraction.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseExtractionProfileV1 {
    extractor: ArtifactIdentityV1,
    source_identity: DigestV1,
    source_schema_sha256: DigestV1,
    grammar_sha256: DigestV1,
    qualification: ReleaseExtractionQualificationV1,
    identity_sha256: DigestV1,
}

impl ReleaseExtractionProfileV1 {
    #[must_use]
    pub fn new(
        source: &LifecycleSourceV1,
        extractor: ArtifactIdentityV1,
        grammar_sha256: DigestV1,
        qualification: ReleaseExtractionQualificationV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.release-extraction-profile.v1\0");
        hash.digest(extractor.identity_sha256());
        hash.digest(source.identity_sha256());
        hash.digest(source.schema_sha256());
        hash.digest(grammar_sha256);
        qualification.encode(&mut hash);
        Self {
            extractor,
            source_identity: source.identity_sha256(),
            source_schema_sha256: source.schema_sha256(),
            grammar_sha256,
            qualification,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn extractor(&self) -> &ArtifactIdentityV1 {
        &self.extractor
    }

    #[must_use]
    pub const fn source_identity(&self) -> DigestV1 {
        self.source_identity
    }

    #[must_use]
    pub const fn source_schema_sha256(&self) -> DigestV1 {
        self.source_schema_sha256
    }

    #[must_use]
    pub const fn grammar_sha256(&self) -> DigestV1 {
        self.grammar_sha256
    }

    #[must_use]
    pub const fn qualification(&self) -> ReleaseExtractionQualificationV1 {
        self.qualification
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// One extractor invocation bound to its exact normalized item set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseExtractionReceiptV1 {
    item_count: u64,
    items_sha256: DigestV1,
    observation_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ReleaseExtractionReceiptV1 {
    pub(crate) fn new(
        source: &LifecycleSourceV1,
        profile: &ReleaseExtractionProfileV1,
        item_count: u64,
        items_sha256: DigestV1,
        observation_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.release-extraction-receipt.v1\0");
        hash.digest(source.identity_sha256());
        hash.digest(profile.identity_sha256());
        hash.u64(item_count);
        hash.digest(items_sha256);
        hash.digest(observation_sha256);
        Self {
            item_count,
            items_sha256,
            observation_sha256,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn item_count(&self) -> u64 {
        self.item_count
    }

    #[must_use]
    pub const fn items_sha256(&self) -> DigestV1 {
        self.items_sha256
    }

    #[must_use]
    pub const fn observation_sha256(&self) -> DigestV1 {
        self.observation_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
