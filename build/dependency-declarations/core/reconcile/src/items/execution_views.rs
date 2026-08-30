impl GenerationInputsV1 {
    /// Returns the root lockfile input.
    #[must_use]
    pub const fn lock(&self) -> &InputFileV1 {
        &self.lock
    }

    /// Returns the Reindeer configuration input.
    #[must_use]
    pub const fn config(&self) -> &InputFileV1 {
        &self.config
    }

    /// Returns the complete repository-side declared read set.
    #[must_use]
    pub const fn repository_reads(&self) -> &InputTreeV1 {
        &self.repository_reads
    }

    /// Returns the complete fixup-tree descriptor.
    #[must_use]
    pub const fn fixups(&self) -> &InputTreeV1 {
        &self.fixups
    }

    /// Returns the complete isolated Cargo-home declared read set.
    #[must_use]
    pub const fn cargo_home_reads(&self) -> &InputTreeV1 {
        &self.cargo_home_reads
    }
}

impl GenerationToolsV1 {
    /// Returns the generator identity.
    #[must_use]
    pub const fn generator(&self) -> &GeneratorIdentityV1 {
        &self.generator
    }

    /// Returns the Cargo identity.
    #[must_use]
    pub const fn cargo(&self) -> &ToolIdentityV1 {
        &self.cargo
    }

    /// Returns the rustc identity independently of declared MSRV.
    #[must_use]
    pub const fn rustc(&self) -> &ToolIdentityV1 {
        &self.rustc
    }

    /// Returns the exact sandbox runtime image or root artifact.
    #[must_use]
    pub const fn execution_runtime(&self) -> &ArtifactIdentityV1 {
        &self.execution_runtime
    }

    /// Returns the semantic qualification tuple.
    #[must_use]
    pub const fn qualification(&self) -> &GenerationQualificationV1 {
        &self.qualification
    }
}

impl GenerationExecutionV1 {
    /// Returns the canonical platform set.
    #[must_use]
    pub const fn platforms(&self) -> &PlatformSetV1 {
        &self.platforms
    }

    /// Returns the closed environment profile.
    #[must_use]
    pub const fn environment(&self) -> EnvironmentProfileV1 {
        self.environment
    }

    /// Returns the closed sandbox profile.
    #[must_use]
    pub const fn sandbox(&self) -> SandboxProfileV1 {
        self.sandbox
    }

    /// Returns the semantic validator profile.
    #[must_use]
    pub const fn validator(&self) -> ValidatorProfileV1 {
        self.validator
    }

    /// Returns the frozen v1 bounds profile.
    #[must_use]
    pub const fn bounds(&self) -> ValidationBoundsV1 {
        self.bounds
    }
}

impl GenerationRequestV1 {
    /// Returns the repository correlation.
    #[must_use]
    pub const fn repository(&self) -> &RepositoryCorrelationV1 {
        &self.repository
    }

    /// Returns exact generator and tool identities.
    #[must_use]
    pub const fn tools(&self) -> &GenerationToolsV1 {
        &self.tools
    }

    /// Returns platform and closed execution profiles.
    #[must_use]
    pub const fn execution(&self) -> &GenerationExecutionV1 {
        &self.execution
    }
}

impl RuleV1 {
    /// Returns the zero-based rendered position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Returns the exact closed producer rule kind.
    #[must_use]
    pub const fn kind(&self) -> ReindeerRuleKindV1 {
        self.kind
    }

    /// Returns every typed semantic field.
    #[must_use]
    pub const fn semantic(&self) -> &SemanticValueV1 {
        &self.semantic
    }

    /// Returns the rendered rule-fragment digest.
    #[must_use]
    pub const fn rendered_sha256(&self) -> DigestV1 {
        self.rendered_sha256
    }

    /// Returns the target identity derived from the producer semantic value.
    #[must_use]
    pub fn target_name(&self) -> Option<&str> {
        self.derived_target_name()
    }
}

impl RuleGraphV1 {
    /// Returns the exact generated prefix bytes.
    #[must_use]
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    /// Returns all rules in rendered order.
    #[must_use]
    pub fn rules(&self) -> &[RuleV1] {
        &self.rules
    }
}

impl ValidatedGenerationV1 {
    /// Returns the originating request identity.
    #[must_use]
    pub const fn request_id(&self) -> DigestV1 {
        self.request_id
    }

    /// Returns the output digest.
    #[must_use]
    pub const fn output_sha256(&self) -> DigestV1 {
        self.output_sha256
    }

    /// Returns the output byte length.
    #[must_use]
    pub const fn output_length_bytes(&self) -> u64 {
        self.output_length_bytes
    }

    /// Returns the exact producer graph-transport digest.
    #[must_use]
    pub const fn provider_graph_sha256(&self) -> DigestV1 {
        self.provider_graph_sha256
    }

    /// Returns the producer graph digest.
    #[must_use]
    pub const fn graph_sha256(&self) -> DigestV1 {
        self.graph_sha256
    }

    /// Returns the stable declared-plus-observed execution identity.
    #[must_use]
    pub const fn execution_fingerprint_sha256(&self) -> DigestV1 {
        self.execution_fingerprint_sha256
    }

    /// Returns the validated producer graph.
    #[must_use]
    pub const fn graph(&self) -> &RuleGraphV1 {
        &self.graph
    }

    /// Returns the validator profile.
    #[must_use]
    pub const fn validator(&self) -> ValidatorProfileV1 {
        self.validator
    }

    /// Returns the independent projection receipt.
    #[must_use]
    pub const fn projection_receipt(&self) -> DigestV1 {
        self.projection_receipt
    }
}

impl PublicationIntentV1 {
    /// Returns the exact destination preimage or required absence.
    #[must_use]
    pub const fn expected_preimage(&self) -> Option<DigestV1> {
        self.expected_preimage
    }

    /// Returns the qualified publisher profile.
    #[must_use]
    pub const fn publisher(&self) -> PublisherProfileV1 {
        self.publisher
    }
}

impl PublicationAttemptReceiptV1 {
    /// Returns the destination preimage bound into this attempt.
    #[must_use]
    pub const fn expected_preimage(&self) -> Option<DigestV1> {
        self.expected_preimage
    }

    /// Returns the publisher profile bound into this attempt.
    #[must_use]
    pub const fn publisher(&self) -> PublisherProfileV1 {
        self.publisher
    }
}
