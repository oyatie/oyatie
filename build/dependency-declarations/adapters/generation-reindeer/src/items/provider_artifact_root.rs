fn render_reindeer_artifact_root_v1(
    schema: &ReindeerProviderSchemaV1,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let source_revision = PINNED_SOURCE_REVISION;
    let adaptation_recipe_id = REINDEER_ADAPTATION_RECIPE_ID_V1;
    let source_sha256 = schema.source_sha256().bytes().to_vec();
    let semantic_schema_sha256 = schema.semantic_schema_sha256().bytes().to_vec();
    let rule_kinds = schema
        .rule_variants()
        .iter()
        .map(|variant| quote::format_ident!("{}", variant.name()))
        .collect::<Vec<_>>();
    let rule_tags = (0..rule_kinds.len())
        .map(|value| {
            u8::try_from(value).map_err(|_| ReindeerProviderAdaptationErrorV1::OutputTooLarge)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transport = render_reindeer_artifact_transport_tokens_v1();
    let writer = render_reindeer_artifact_writer_tokens_v1();
    render_provider_module_v1(quote::quote! {
        mod serializer;
        mod value;

        use anyhow::Context as _;
        use sha2::Digest as _;

        use self::serializer::ReindeerValueSerializerV1;
        use self::value::ReindeerValueV1;
        use crate::buck::Rule;
        use crate::config::BuckConfig;

        const SOURCE_REVISION: &str = #source_revision;
        const ADAPTATION_RECIPE_ID: &str = #adaptation_recipe_id;
        const SOURCE_SHA256: [u8; 32] = [#(#source_sha256),*];
        const SEMANTIC_SCHEMA_SHA256: [u8; 32] = [#(#semantic_schema_sha256),*];
        const GRAPH_DOMAIN: &[u8] = b"reindeer.rule-graph.v1\0";
        const MAX_RULES: usize = 1_000_000;
        const MAX_CONTAINER_ITEMS: usize = 1_000_000;
        const MAX_STRING_BYTES: usize = 16 * 1024 * 1024;
        const MAX_VALUE_DEPTH: usize = 128;
        const MAX_SEMANTIC_BYTES: usize = 512 * 1024 * 1024;
        const MAX_GRAPH_BYTES: usize = 512 * 1024 * 1024;
        const MAX_RENDERED_BYTES: usize = 512 * 1024 * 1024;

        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum ReindeerRuleKindV1 {
            #(#rule_kinds,)*
        }

        impl ReindeerRuleKindV1 {
            const fn of(rule: &Rule) -> Self {
                match rule {
                    #(Rule::#rule_kinds(_) => Self::#rule_kinds,)*
                }
            }

            const fn tag(self) -> u8 {
                match self {
                    #(Self::#rule_kinds => #rule_tags,)*
                }
            }
        }

        #[derive(Debug, Eq, PartialEq)]
        struct ReindeerRuleV1 {
            position: u64,
            kind: ReindeerRuleKindV1,
            semantic: ReindeerValueV1,
            rendered: Box<[u8]>,
        }

        #[derive(Debug, Eq, PartialEq)]
        pub(crate) struct ReindeerRuleGraphV1 {
            prefix: Box<[u8]>,
            rules: Box<[ReindeerRuleV1]>,
        }

        impl ReindeerRuleGraphV1 {
            fn from_rules(config: &BuckConfig, rules: &[Rule]) -> anyhow::Result<Self> {
                anyhow::ensure!(rules.len() <= MAX_RULES, "rule graph exceeds rule bound");
                let prefix = render_prefix(config)?;
                let mut graph_rules = Vec::with_capacity(rules.len());
                let value_serializer = ReindeerValueSerializerV1::root();
                let mut rendered_bytes = prefix.len();
                for (position, rule) in rules.iter().enumerate() {
                    let semantic = rule
                        .serialize_with(config, value_serializer.clone())
                        .context("capture typed Reindeer rule")?;
                    rendered_bytes = rendered_bytes
                        .checked_add(usize::from(position > 0))
                        .context("rendered BUCK byte length overflow")?;
                    anyhow::ensure!(
                        rendered_bytes <= MAX_RENDERED_BYTES,
                        "rendered BUCK exceeds byte bound"
                    );
                    let remaining_bytes = MAX_RENDERED_BYTES - rendered_bytes;
                    let mut rendered = BoundedRenderedBytesV1::new(remaining_bytes);
                    rule.render(config, &mut rendered)
                        .context("render Reindeer rule fragment")?;
                    rendered_bytes = rendered_bytes
                        .checked_add(rendered.len())
                        .context("rendered BUCK byte length overflow")?;
                    graph_rules.push(ReindeerRuleV1 {
                        position: u64::try_from(position)
                            .context("rule position exceeds u64")?,
                        kind: ReindeerRuleKindV1::of(rule),
                        semantic,
                        rendered: rendered.into_boxed_slice(),
                    });
                }
                let graph = Self {
                    prefix: prefix.into_boxed_slice(),
                    rules: graph_rules.into_boxed_slice(),
                };
                Ok(graph)
            }

            fn canonical_bytes(&self) -> anyhow::Result<Vec<u8>> {
                let mut output = Vec::new();
                extend_graph_bytes(&mut output, GRAPH_DOMAIN)?;
                encode_graph_bytes(&mut output, SOURCE_REVISION.as_bytes())?;
                encode_graph_bytes(&mut output, ADAPTATION_RECIPE_ID.as_bytes())?;
                extend_graph_bytes(&mut output, &SOURCE_SHA256)?;
                extend_graph_bytes(&mut output, &SEMANTIC_SCHEMA_SHA256)?;
                encode_graph_bytes(&mut output, &self.prefix)?;
                encode_graph_length(&mut output, self.rules.len())?;
                for rule in &self.rules {
                    extend_graph_bytes(&mut output, &rule.position.to_be_bytes())?;
                    extend_graph_bytes(&mut output, &[rule.kind.tag()])?;
                    rule.semantic.encode(&mut output)?;
                    let rendered_sha256: [u8; 32] =
                        sha2::Sha256::digest(&rule.rendered).into();
                    extend_graph_bytes(&mut output, &rendered_sha256)?;
                }
                Ok(output)
            }

            fn rendered_buck(&self) -> anyhow::Result<Box<[u8]>> {
                let mut output = Vec::new();
                extend_rendered_bytes(&mut output, &self.prefix)?;
                for (position, rule) in self.rules.iter().enumerate() {
                    if position > 0 {
                        extend_rendered_bytes(&mut output, b"\n")?;
                    }
                    extend_rendered_bytes(&mut output, &rule.rendered)?;
                }
                Ok(output.into_boxed_slice())
            }
        }

        #transport

        fn render_prefix(config: &BuckConfig) -> anyhow::Result<Vec<u8>> {
            let mut output = Vec::new();
            extend_rendered_bytes(&mut output, config.generated_file_header.as_bytes())?;
            if !config.generated_file_header.is_empty() {
                extend_rendered_bytes(&mut output, b"\n")?;
            }
            extend_rendered_bytes(&mut output, config.buckfile_imports.as_bytes())?;
            if !config.buckfile_imports.is_empty() {
                extend_rendered_bytes(&mut output, b"\n")?;
            }
            Ok(output)
        }

        fn encode_graph_length(output: &mut Vec<u8>, length: usize) -> anyhow::Result<()> {
            let length = u64::try_from(length).context("graph collection exceeds u64")?;
            extend_graph_bytes(output, &length.to_be_bytes())
        }

        fn encode_graph_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> anyhow::Result<()> {
            encode_graph_length(output, bytes.len())?;
            extend_graph_bytes(output, bytes)
        }

        fn extend_graph_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> anyhow::Result<()> {
            let next = output
                .len()
                .checked_add(bytes.len())
                .context("graph byte length overflow")?;
            anyhow::ensure!(next <= MAX_GRAPH_BYTES, "graph exceeds byte bound");
            output.extend_from_slice(bytes);
            Ok(())
        }

        fn extend_rendered_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> anyhow::Result<()> {
            let next = output
                .len()
                .checked_add(bytes.len())
                .context("rendered BUCK byte length overflow")?;
            anyhow::ensure!(
                next <= MAX_RENDERED_BYTES,
                "rendered BUCK exceeds byte bound"
            );
            output.extend_from_slice(bytes);
            Ok(())
        }

        #writer
    })
}
