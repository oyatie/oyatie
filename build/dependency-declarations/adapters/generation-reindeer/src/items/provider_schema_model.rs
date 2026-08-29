/// A SHA-256 identity used by the pinned Reindeer provider profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReindeerProviderDigestV1([u8; 32]);

impl ReindeerProviderDigestV1 {
    fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// One producer payload field in declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderFieldV1 {
    name: String,
    rust_type: String,
}

impl ReindeerProviderFieldV1 {
    /// Returns the upstream Rust field name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the normalized upstream Rust field type.
    #[must_use]
    pub fn rust_type(&self) -> &str {
        &self.rust_type
    }
}

/// One supported `Rule` variant and its complete direct payload shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderRuleVariantV1 {
    name: String,
    payload: String,
    fields: Box<[ReindeerProviderFieldV1]>,
    serializer_sha256: ReindeerProviderDigestV1,
}

impl ReindeerProviderRuleVariantV1 {
    /// Returns the exact upstream variant name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact upstream payload type.
    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Returns every direct payload field in declaration order.
    #[must_use]
    pub fn fields(&self) -> &[ReindeerProviderFieldV1] {
        &self.fields
    }

    /// Returns the normalized serializer implementation identity.
    #[must_use]
    pub const fn serializer_sha256(&self) -> ReindeerProviderDigestV1 {
        self.serializer_sha256
    }
}

/// The closed schema receipt for the exact pinned Reindeer source profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReindeerProviderSchemaV1 {
    source_sha256: ReindeerProviderDigestV1,
    semantic_schema_sha256: ReindeerProviderDigestV1,
    rule_variants: Box<[ReindeerProviderRuleVariantV1]>,
}

impl ReindeerProviderSchemaV1 {
    /// Returns the number of whole source files parsed for this receipt.
    #[must_use]
    pub const fn parsed_source_files(&self) -> u64 {
        1
    }

    /// Returns the exact source-byte identity.
    #[must_use]
    pub const fn source_sha256(&self) -> ReindeerProviderDigestV1 {
        self.source_sha256
    }

    /// Returns the formatting-insensitive semantic schema identity.
    #[must_use]
    pub const fn semantic_schema_sha256(&self) -> ReindeerProviderDigestV1 {
        self.semantic_schema_sha256
    }

    /// Returns the supported variants in upstream declaration order.
    #[must_use]
    pub fn rule_variants(&self) -> &[ReindeerProviderRuleVariantV1] {
        &self.rule_variants
    }
}

/// A fail-closed reason why the pinned producer schema was not proved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReindeerProviderSchemaErrorV1 {
    UnsupportedSourceRevision,
    SourceTooLarge,
    InvalidUtf8,
    InvalidRust,
    MissingRuleEnum,
    DuplicateRuleEnum,
    UnsupportedRuleShape,
    UnsupportedRuleVariant,
    MissingPayloadStruct,
    DuplicatePayloadStruct,
    UnsupportedPayloadFields,
    MissingPayloadSerializer,
    DuplicatePayloadSerializer,
    MissingRuleSortKey,
    DuplicateRuleSortKey,
    MissingRulePartialEq,
    DuplicateRulePartialEq,
    MissingRuleOrd,
    DuplicateRuleOrd,
    MissingRuleRenderer,
    DuplicateRuleRenderer,
}

impl fmt::Display for ReindeerProviderSchemaErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Reindeer provider schema refused: {self:?}")
    }
}

impl Error for ReindeerProviderSchemaErrorV1 {}
