/// Consumer-impact class of one upstream release item.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReleaseItemKindV1 {
    Language = 0,
    Compiler = 1,
    StandardLibrary = 2,
    Cargo = 3,
    Rustfmt = 4,
    Clippy = 5,
    Security = 6,
    Target = 7,
    Documentation = 8,
    Internal = 9,
}

/// One normalized item from one exact upstream source object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseItemV1 {
    source_identity: DigestV1,
    stable_key: Box<str>,
    upstream_change: Box<str>,
    kind: ReleaseItemKindV1,
    content_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ReleaseItemV1 {
    pub fn try_new(
        source: &LifecycleSourceV1,
        stable_key: impl Into<String>,
        upstream_change: impl Into<String>,
        kind: ReleaseItemKindV1,
        content_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let mut value = Self {
            source_identity: source.identity_sha256(),
            stable_key: lifecycle_identity(stable_key.into())?,
            upstream_change: lifecycle_identity(upstream_change.into())?,
            kind,
            content_sha256,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.release-item.v1\0");
        value.encode_fields(&mut hash)?;
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        hash.digest(self.source_identity);
        lifecycle_hash_string(hash, &self.stable_key)?;
        lifecycle_hash_string(hash, &self.upstream_change)?;
        hash.tag(self.kind as u8);
        hash.digest(self.content_sha256);
        Ok(())
    }

    #[must_use]
    pub const fn source_identity(&self) -> DigestV1 {
        self.source_identity
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub fn upstream_change(&self) -> &str {
        &self.upstream_change
    }

    #[must_use]
    pub const fn kind(&self) -> ReleaseItemKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn content_sha256(&self) -> DigestV1 {
        self.content_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Bounded extraction result for one exact source object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseSourceBatchV1 {
    source: LifecycleSourceV1,
    extraction: ReleaseExtractionProfileV1,
    receipt: ReleaseExtractionReceiptV1,
}

impl ReleaseSourceBatchV1 {
    pub fn try_from_items(
        source: LifecycleSourceV1,
        extraction: ReleaseExtractionProfileV1,
        items: &[ReleaseItemV1],
        observation_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let source_identity = source.identity_sha256();
        if extraction.source_identity() != source_identity
            || extraction.source_schema_sha256() != source.schema_sha256()
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ExtractionProfileMismatch,
            ));
        }
        if items.len() > LifecycleBoundsV1::MAX_RELEASE_ITEMS {
            return Err(lifecycle_bounds());
        }
        if items
            .iter()
            .any(|item| item.source_identity() != source_identity)
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::SourceCoverageMismatch,
            ));
        }
        let item_count = lifecycle_len(items.len())?;
        let items_sha256 = release_item_set_sha256(items)?;
        let receipt = ReleaseExtractionReceiptV1::new(
            &source,
            &extraction,
            item_count,
            items_sha256,
            observation_sha256,
        );
        Ok(Self {
            source,
            extraction,
            receipt,
        })
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        &self.source
    }

    #[must_use]
    pub const fn extraction(&self) -> &ReleaseExtractionProfileV1 {
        &self.extraction
    }

    #[must_use]
    pub const fn receipt(&self) -> &ReleaseExtractionReceiptV1 {
        &self.receipt
    }

    #[must_use]
    pub const fn item_count(&self) -> u64 {
        self.receipt.item_count()
    }

    #[must_use]
    pub const fn items_sha256(&self) -> DigestV1 {
        self.receipt.items_sha256()
    }
}

/// Canonical identity of a complete item set for one source.
pub fn release_item_set_sha256(
    items: &[ReleaseItemV1],
) -> Result<DigestV1, LifecycleFailureV1> {
    release_identity_set_sha256(items.iter().map(ReleaseItemV1::identity_sha256).collect())
}

pub(crate) fn release_identity_set_sha256(
    mut identities: Vec<DigestV1>,
) -> Result<DigestV1, LifecycleFailureV1> {
    if identities.len() > LifecycleBoundsV1::MAX_RELEASE_ITEMS {
        return Err(lifecycle_bounds());
    }
    identities.sort_unstable();
    if identities.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::DuplicateIdentity,
        ));
    }
    let mut hash = CanonicalHasherV1::new(b"build.release-item-set.v1\0");
    hash.u64(lifecycle_len(identities.len())?);
    for identity in identities {
        hash.digest(identity);
    }
    Ok(hash.finish())
}
