impl GenerationRequestV1 {
    /// Returns the complete parser, renderer, schema, grammar, and validator profile identity.
    #[must_use]
    pub const fn projection_profile_sha256(&self) -> DigestV1 {
        self.projection_profile_sha256
    }
}

impl RenderedRuleV1 {
    /// Returns the zero-based rendered position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Returns all syntax-visible semantic fields.
    #[must_use]
    pub const fn semantic(&self) -> &SemanticValueV1 {
        &self.semantic
    }

    /// Returns the exact rendered rule-fragment digest.
    #[must_use]
    pub const fn rendered_sha256(&self) -> DigestV1 {
        self.rendered_sha256
    }
}

impl RenderedRuleGraphV1 {
    /// Returns the exact parsed prefix bytes.
    #[must_use]
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    /// Returns all parser-comparable rules in rendered order.
    #[must_use]
    pub fn rules(&self) -> &[RenderedRuleV1] {
        &self.rules
    }
}

impl ParsedBuckProjectionV1 {
    /// Returns the complete profile admitted by the projection adapter.
    #[must_use]
    pub const fn profile_sha256(&self) -> DigestV1 {
        self.profile_sha256
    }

    /// Returns the independently parsed syntax-visible graph.
    #[must_use]
    pub const fn graph(&self) -> &RenderedRuleGraphV1 {
        &self.graph
    }

    /// Returns the exact rendered output digest.
    #[must_use]
    pub const fn output_sha256(&self) -> DigestV1 {
        self.output_sha256
    }

    /// Returns the projection receipt.
    #[must_use]
    pub const fn receipt_sha256(&self) -> DigestV1 {
        self.receipt_sha256
    }
}
