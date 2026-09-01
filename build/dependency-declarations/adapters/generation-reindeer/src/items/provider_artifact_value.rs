fn render_reindeer_artifact_value_v1() -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    render_provider_module_v1(quote::quote! {
        use super::encode_graph_bytes;
        use super::encode_graph_length;
        use super::extend_graph_bytes;

        #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub(super) enum ReindeerCallArgumentsV1 {
            Positional(Vec<ReindeerValueV1>),
            Named(Vec<(String, ReindeerValueV1)>),
        }

        #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub(super) enum ReindeerValueV1 {
            None,
            Bool(bool),
            Signed(i128),
            Unsigned(u128),
            String(String),
            Identifier(String),
            List(Vec<Self>),
            Tuple(Vec<Self>),
            Map(Vec<(Self, Self)>),
            Call {
                callee: String,
                arguments: ReindeerCallArgumentsV1,
            },
        }

        impl ReindeerValueV1 {
            pub(super) fn encode(&self, output: &mut Vec<u8>) -> anyhow::Result<()> {
                match self {
                    Self::None => extend_graph_bytes(output, &[0]),
                    Self::Bool(value) => {
                        extend_graph_bytes(output, &[1, u8::from(*value)])
                    }
                    Self::Signed(value) => {
                        extend_graph_bytes(output, &[2])?;
                        extend_graph_bytes(output, &value.to_be_bytes())
                    }
                    Self::Unsigned(value) => {
                        extend_graph_bytes(output, &[3])?;
                        extend_graph_bytes(output, &value.to_be_bytes())
                    }
                    Self::String(value) => encode_string(output, 4, value),
                    Self::Identifier(value) => encode_string(output, 5, value),
                    Self::List(values) => encode_sequence(output, 6, values),
                    Self::Tuple(values) => encode_sequence(output, 7, values),
                    Self::Map(entries) => {
                        extend_graph_bytes(output, &[8])?;
                        encode_graph_length(output, entries.len())?;
                        for (key, value) in entries {
                            key.encode(output)?;
                            value.encode(output)?;
                        }
                        Ok(())
                    }
                    Self::Call { callee, arguments } => {
                        extend_graph_bytes(output, &[9])?;
                        encode_graph_bytes(output, callee.as_bytes())?;
                        match arguments {
                            ReindeerCallArgumentsV1::Positional(values) => {
                                extend_graph_bytes(output, &[0])?;
                                encode_graph_length(output, values.len())?;
                                for value in values {
                                    value.encode(output)?;
                                }
                            }
                            ReindeerCallArgumentsV1::Named(fields) => {
                                extend_graph_bytes(output, &[1])?;
                                encode_graph_length(output, fields.len())?;
                                for (name, value) in fields {
                                    encode_graph_bytes(output, name.as_bytes())?;
                                    value.encode(output)?;
                                }
                            }
                        }
                        Ok(())
                    }
                }
            }
        }

        fn encode_string(output: &mut Vec<u8>, tag: u8, value: &str) -> anyhow::Result<()> {
            extend_graph_bytes(output, &[tag])?;
            encode_graph_bytes(output, value.as_bytes())
        }

        fn encode_sequence(
            output: &mut Vec<u8>,
            tag: u8,
            values: &[ReindeerValueV1],
        ) -> anyhow::Result<()> {
            extend_graph_bytes(output, &[tag])?;
            encode_graph_length(output, values.len())?;
            for value in values {
                value.encode(output)?;
            }
            Ok(())
        }
    })
}
