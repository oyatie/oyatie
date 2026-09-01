/// Closed producer rule variants in the exact pinned Reindeer declaration order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ReindeerRuleKindV1 {
    Alias = 0,
    Sources = 1,
    Filegroup = 2,
    ExtractArchive = 3,
    HttpArchive = 4,
    GitFetch = 5,
    Binary = 6,
    Library = 7,
    BuildscriptBinary = 8,
    BuildscriptGenrule = 9,
    CxxLibrary = 10,
    PrebuiltCxxLibrary = 11,
    RootPackage = 12,
}

impl ReindeerRuleKindV1 {
    pub(crate) fn try_from_tag(tag: u8) -> Result<Self, FailureV1> {
        match tag {
            0 => Ok(Self::Alias),
            1 => Ok(Self::Sources),
            2 => Ok(Self::Filegroup),
            3 => Ok(Self::ExtractArchive),
            4 => Ok(Self::HttpArchive),
            5 => Ok(Self::GitFetch),
            6 => Ok(Self::Binary),
            7 => Ok(Self::Library),
            8 => Ok(Self::BuildscriptBinary),
            9 => Ok(Self::BuildscriptGenrule),
            10 => Ok(Self::CxxLibrary),
            11 => Ok(Self::PrebuiltCxxLibrary),
            12 => Ok(Self::RootPackage),
            _ => Err(invalid_graph()),
        }
    }

    pub(crate) const fn tag(self) -> u8 {
        self as u8
    }
}

/// One fully projected rule in rendered order.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuleV1 {
    position: u64,
    kind: ReindeerRuleKindV1,
    semantic: SemanticValueV1,
    rendered_sha256: DigestV1,
}

impl RuleV1 {
    /// Creates a full-field rule projection for graph-level admission.
    #[must_use]
    pub const fn new(
        position: u64,
        kind: ReindeerRuleKindV1,
        semantic: SemanticValueV1,
        rendered_sha256: DigestV1,
    ) -> Self {
        Self {
            position,
            kind,
            semantic,
            rendered_sha256,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.u64(self.position);
        hash.tag(self.kind.tag());
        self.semantic.encode(hash)?;
        hash.digest(self.rendered_sha256);
        Ok(())
    }

    fn encoded_bytes(&self) -> Result<usize, FailureV1> {
        [8, 1, self.semantic.encoded_bytes(), 32]
            .into_iter()
            .try_fold(0_usize, |total, bytes| {
                total.checked_add(bytes).ok_or_else(invalid_graph)
            })
    }

    const fn semantic_nodes(&self) -> usize {
        self.semantic.nodes()
    }

    pub(crate) fn derived_target_name(&self) -> Option<&str> {
        self.semantic.named_string("name")
    }
}

/// Complete producer-owned rule graph projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuleGraphV1 {
    prefix: Box<[u8]>,
    rules: Box<[RuleV1]>,
    sha256: DigestV1,
    encoded_length_bytes: u64,
}

impl RuleGraphV1 {
    /// Validates order, target identities, and graph-size bounds.
    pub fn try_new(prefix: Vec<u8>, rules: Vec<RuleV1>) -> Result<Self, FailureV1> {
        if prefix.len() > ValidationBoundsV1::MAX_STRING_BYTES
            || rules.len() > ValidationBoundsV1::MAX_RULES
        {
            return Err(invalid_graph());
        }
        let mut names = std::collections::BTreeSet::new();
        let mut encoded_bytes = 16_usize
            .checked_add(prefix.len())
            .ok_or_else(invalid_graph)?;
        let mut semantic_nodes = 0_usize;
        for (expected_position, rule) in rules.iter().enumerate() {
            if rule.position != checked_u64(expected_position, invalid_graph())?
                || rule
                    .derived_target_name()
                    .is_some_and(|name| !names.insert(name))
            {
                return Err(invalid_graph());
            }
            encoded_bytes = encoded_bytes
                .checked_add(rule.encoded_bytes()?)
                .ok_or_else(invalid_graph)?;
            semantic_nodes = semantic_nodes
                .checked_add(rule.semantic_nodes())
                .ok_or_else(invalid_graph)?;
        }
        if encoded_bytes > ValidationBoundsV1::MAX_GRAPH_BYTES
            || semantic_nodes > ValidationBoundsV1::MAX_SEMANTIC_NODES
        {
            return Err(invalid_graph());
        }
        let mut value = Self {
            prefix: prefix.into_boxed_slice(),
            rules: rules.into_boxed_slice(),
            sha256: DigestV1::from_bytes([0; 32]),
            encoded_length_bytes: checked_u64(encoded_bytes, invalid_graph())?,
        };
        let mut hash = CanonicalHasherV1::new(b"build.declaration-rule-graph.v1\0");
        value.encode_fields(&mut hash)?;
        value.sha256 = hash.finish();
        Ok(value)
    }

    /// Returns the full-field canonical graph identity.
    #[must_use]
    pub const fn sha256(&self) -> DigestV1 {
        self.sha256
    }

    pub(crate) const fn encoded_length_bytes(&self) -> u64 {
        self.encoded_length_bytes
    }

    fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.bytes(&self.prefix)?;
        hash.u64(checked_u64(self.rules.len(), invalid_graph())?);
        for rule in &self.rules {
            rule.encode(hash)?;
        }
        Ok(())
    }
}
