/// One parser-comparable rule in rendered order.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderedRuleV1 {
    position: u64,
    semantic: SemanticValueV1,
    rendered_sha256: DigestV1,
}

impl RenderedRuleV1 {
    /// Creates one rule from syntax-visible fields only.
    #[must_use]
    pub const fn new(
        position: u64,
        semantic: SemanticValueV1,
        rendered_sha256: DigestV1,
    ) -> Self {
        Self {
            position,
            semantic,
            rendered_sha256,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.u64(self.position);
        self.semantic.encode(hash)?;
        hash.digest(self.rendered_sha256);
        Ok(())
    }

    fn encoded_bytes(&self) -> Result<usize, FailureV1> {
        [8, self.semantic.encoded_bytes(), 32]
            .into_iter()
            .try_fold(0_usize, |total, bytes| {
                total.checked_add(bytes).ok_or_else(invalid_graph)
            })
    }
}

/// Bounded semantic graph that rendered Starlark can independently prove.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderedRuleGraphV1 {
    prefix: Box<[u8]>,
    rules: Box<[RenderedRuleV1]>,
    sha256: DigestV1,
}

impl RenderedRuleGraphV1 {
    /// Validates order, syntax-visible target identities, and graph bounds.
    pub fn try_new(prefix: Vec<u8>, rules: Vec<RenderedRuleV1>) -> Result<Self, FailureV1> {
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
            let Some(name) = rule.semantic.named_string("name") else {
                return Err(invalid_graph());
            };
            if rule.position != checked_u64(expected_position, invalid_graph())?
                || !names.insert(name)
            {
                return Err(invalid_graph());
            }
            encoded_bytes = encoded_bytes
                .checked_add(rule.encoded_bytes()?)
                .ok_or_else(invalid_graph)?;
            semantic_nodes = semantic_nodes
                .checked_add(rule.semantic.nodes())
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
        };
        let mut hash = CanonicalHasherV1::new(b"build.rendered-declaration-graph.v1\0");
        value.encode_fields(&mut hash)?;
        value.sha256 = hash.finish();
        Ok(value)
    }

    /// Returns the complete syntax-visible graph identity.
    #[must_use]
    pub const fn sha256(&self) -> DigestV1 {
        self.sha256
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

impl RuleGraphV1 {
    /// Drops producer-only variant identity and normalizes renderer-equivalent integers.
    pub fn rendered_projection(&self) -> Result<RenderedRuleGraphV1, FailureV1> {
        let mut rules = Vec::with_capacity(self.rules.len());
        for rule in &self.rules {
            rules.push(RenderedRuleV1::new(
                rule.position,
                rule.semantic.rendered_projection()?,
                rule.rendered_sha256,
            ));
        }
        RenderedRuleGraphV1::try_new(self.prefix.to_vec(), rules)
    }
}

impl SemanticValueV1 {
    fn rendered_projection(&self) -> Result<Self, FailureV1> {
        match &self.kind {
            SemanticValueKindV1::None => Ok(Self::none()),
            SemanticValueKindV1::Bool(value) => Ok(Self::boolean(*value)),
            SemanticValueKindV1::Signed(value) => i32::try_from(*value)
                .map(|value| Self::signed(value.into()))
                .map_err(|_| invalid_graph()),
            SemanticValueKindV1::Unsigned(value) => i32::try_from(*value)
                .map(|value| Self::signed(value.into()))
                .map_err(|_| invalid_graph()),
            SemanticValueKindV1::String(value) => Self::string(value.to_string()),
            SemanticValueKindV1::Identifier(value) => Self::identifier(value.to_string()),
            SemanticValueKindV1::List(values) => Self::list(project_values(values)?),
            SemanticValueKindV1::Tuple(values) => Self::tuple(project_values(values)?),
            SemanticValueKindV1::Map(entries) => Self::map(
                entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((key.rendered_projection()?, value.rendered_projection()?))
                    })
                    .collect::<Result<Vec<_>, FailureV1>>()?,
            ),
            SemanticValueKindV1::Call { callee, arguments } => match arguments {
                CallArgumentsV1::Positional(values) if callee.as_ref() == "+" => {
                    Self::select_addition(project_values(values)?)
                }
                CallArgumentsV1::Positional(values) => {
                    Self::call_positional(callee.to_string(), project_values(values)?)
                }
                CallArgumentsV1::Named(fields) => Self::call_named(
                    callee.to_string(),
                    fields
                        .iter()
                        .map(|(name, value)| {
                            Ok((name.to_string(), value.rendered_projection()?))
                        })
                        .collect::<Result<Vec<_>, FailureV1>>()?,
                ),
            },
        }
    }
}

fn project_values(values: &[SemanticValueV1]) -> Result<Vec<SemanticValueV1>, FailureV1> {
    values
        .iter()
        .map(SemanticValueV1::rendered_projection)
        .collect()
}
