const PROVIDER_GRAPH_DOMAIN_V1: &[u8] = b"reindeer.rule-graph.v1\0";
const PROVIDER_RECEIPT_DOMAIN_V1: &[u8] = b"reindeer.generated-artifact.v1\0";
const PROVIDER_TRANSPORT_MAGIC_V1: &[u8] = b"REINDEER_GENERATED_ARTIFACT_V1\0";
const MAX_PROVIDER_INVOCATION_ID_BYTES: usize = 256;

impl ValidationBoundsV1 {
    /// Maximum provider stdout retained by a generation adapter.
    pub const MAX_PROVIDER_TRANSPORT_BYTES: usize = PROVIDER_TRANSPORT_MAGIC_V1.len()
        + 8
        + MAX_PROVIDER_INVOCATION_ID_BYTES
        + 8
        + Self::MAX_GRAPH_BYTES
        + 8
        + Self::MAX_OUTPUT_BYTES
        + 32;
}

pub(crate) struct ProviderArtifactV1<'a> {
    pub(crate) invocation_id: &'a str,
    pub(crate) graph: &'a [u8],
    pub(crate) rendered: &'a [u8],
    pub(crate) receipt_sha256: DigestV1,
}

pub(crate) fn decode_provider_artifact_v1(
    bytes: &[u8],
) -> Result<ProviderArtifactV1<'_>, FailureV1> {
    let mut cursor = ProviderCursorV1::new(bytes);
    cursor.expect(PROVIDER_TRANSPORT_MAGIC_V1)?;
    let invocation_id = cursor.read_text(MAX_PROVIDER_INVOCATION_ID_BYTES)?;
    let graph = cursor.read_slice(ValidationBoundsV1::MAX_GRAPH_BYTES)?;
    let rendered = cursor.read_slice(ValidationBoundsV1::MAX_OUTPUT_BYTES)?;
    let receipt_sha256 = cursor.read_digest()?;
    cursor.finish()?;
    Ok(ProviderArtifactV1 {
        invocation_id,
        graph,
        rendered,
        receipt_sha256,
    })
}

pub(crate) fn decode_provider_graph_v1(
    request: &GenerationRequestV1,
    bytes: &[u8],
) -> Result<RuleGraphV1, FailureV1> {
    if bytes.len() > ValidationBoundsV1::MAX_GRAPH_BYTES {
        return Err(invalid_graph());
    }
    let mut cursor = ProviderCursorV1::new(bytes);
    cursor.expect(PROVIDER_GRAPH_DOMAIN_V1)?;

    let source_revision = cursor.read_text(ValidationBoundsV1::MAX_IDENTITY_BYTES)?;
    let provider = &request.tools.qualification.provider_graph;
    if source_revision != request.tools.generator.source_revision.as_ref()
        || cursor.read_text(ValidationBoundsV1::MAX_IDENTITY_BYTES)?
            != provider.adaptation_recipe_id.as_ref()
        || cursor.read_digest()? != provider.schema_source_sha256
        || cursor.read_digest()? != provider.semantic_schema_sha256
    {
        return Err(invalid_graph());
    }

    let prefix = cursor.read_vec(ValidationBoundsV1::MAX_STRING_BYTES)?;
    let rule_count = cursor.read_len(ValidationBoundsV1::MAX_RULES)?;
    let mut rules = Vec::new();
    let mut remaining_semantic_nodes = ValidationBoundsV1::MAX_SEMANTIC_NODES;
    rules
        .try_reserve_exact(rule_count)
        .map_err(|_| invalid_graph())?;
    for expected_position in 0..rule_count {
        let position = cursor.read_u64()?;
        if position != checked_u64(expected_position, invalid_graph())? {
            return Err(invalid_graph());
        }
        let kind = ReindeerRuleKindV1::try_from_tag(cursor.read_u8()?)?;
        let semantic = decode_provider_value_v1(&mut cursor, 1, &mut remaining_semantic_nodes)?;
        let rendered_sha256 = cursor.read_digest()?;
        rules.push(RuleV1::new(position, kind, semantic, rendered_sha256));
    }
    cursor.finish()?;
    RuleGraphV1::try_new(prefix, rules)
}

pub(crate) fn provider_artifact_receipt_v1(
    invocation_id: &str,
    graph: &[u8],
    rendered: &[u8],
) -> Result<DigestV1, FailureV1> {
    if invocation_id.is_empty()
        || invocation_id.len() > MAX_PROVIDER_INVOCATION_ID_BYTES
        || invocation_id.chars().any(char::is_control)
    {
        return Err(invalid_graph());
    }
    let mut hash = CanonicalHasherV1::new(PROVIDER_RECEIPT_DOMAIN_V1);
    hash.bytes(invocation_id.as_bytes())?;
    hash.bytes(graph)?;
    hash.bytes(rendered)?;
    Ok(hash.finish())
}

pub(crate) struct ProviderCursorV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ProviderCursorV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), FailureV1> {
        (self.take(expected.len())? == expected)
            .then_some(())
            .ok_or_else(invalid_graph)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, FailureV1> {
        self.take(1)?.first().copied().ok_or_else(invalid_graph)
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, FailureV1> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| invalid_graph())?;
        Ok(u64::from_be_bytes(bytes))
    }

    pub(crate) fn read_i128(&mut self) -> Result<i128, FailureV1> {
        let bytes: [u8; 16] = self.take(16)?.try_into().map_err(|_| invalid_graph())?;
        Ok(i128::from_be_bytes(bytes))
    }

    pub(crate) fn read_u128(&mut self) -> Result<u128, FailureV1> {
        let bytes: [u8; 16] = self.take(16)?.try_into().map_err(|_| invalid_graph())?;
        Ok(u128::from_be_bytes(bytes))
    }

    pub(crate) fn read_digest(&mut self) -> Result<DigestV1, FailureV1> {
        let bytes: [u8; 32] = self.take(32)?.try_into().map_err(|_| invalid_graph())?;
        Ok(DigestV1::from_bytes(bytes))
    }

    pub(crate) fn read_len(&mut self, max: usize) -> Result<usize, FailureV1> {
        let value = usize::try_from(self.read_u64()?).map_err(|_| invalid_graph())?;
        (value <= max).then_some(value).ok_or_else(invalid_graph)
    }

    pub(crate) fn read_vec(&mut self, max: usize) -> Result<Vec<u8>, FailureV1> {
        Ok(self.read_slice(max)?.to_vec())
    }

    pub(crate) fn read_text(&mut self, max: usize) -> Result<&'a str, FailureV1> {
        std::str::from_utf8(self.read_slice(max)?).map_err(|_| invalid_graph())
    }

    fn read_slice(&mut self, max: usize) -> Result<&'a [u8], FailureV1> {
        let length = self.read_len(max)?;
        self.take(length)
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], FailureV1> {
        let end = self.offset.checked_add(length).ok_or_else(invalid_graph)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(invalid_graph)?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), FailureV1> {
        (self.offset == self.bytes.len())
            .then_some(())
            .ok_or_else(invalid_graph)
    }
}
