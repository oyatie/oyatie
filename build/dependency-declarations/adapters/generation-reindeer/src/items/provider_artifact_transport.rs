fn render_reindeer_artifact_transport_tokens_v1() -> proc_macro2::TokenStream {
    quote::quote! {
        const RECEIPT_DOMAIN: &[u8] = b"reindeer.generated-artifact.v1\0";
        const TRANSPORT_MAGIC: &[u8] = b"REINDEER_GENERATED_ARTIFACT_V1\0";
        const MAX_INVOCATION_ID_BYTES: usize = 256;

        #[derive(Debug, Eq, PartialEq)]
        pub(crate) struct ReindeerGeneratedArtifactV1 {
            invocation_id: Box<str>,
            graph_bytes: Box<[u8]>,
            rendered_buck: Box<[u8]>,
            receipt_sha256: [u8; 32],
        }

        impl ReindeerGeneratedArtifactV1 {
            pub(crate) fn from_rules(
                config: &BuckConfig,
                invocation_id: &str,
                rules: &[Rule],
            ) -> anyhow::Result<Self> {
                validate_invocation_id(invocation_id)?;
                let graph = ReindeerRuleGraphV1::from_rules(config, rules)?;
                let graph_bytes = graph.canonical_bytes()?.into_boxed_slice();
                let rendered_buck = graph.rendered_buck()?;
                let receipt_sha256 =
                    artifact_receipt(invocation_id, &graph_bytes, &rendered_buck)?;
                Ok(Self {
                    invocation_id: invocation_id.into(),
                    graph_bytes,
                    rendered_buck,
                    receipt_sha256,
                })
            }

            pub(crate) fn write_transport(
                &self,
                mut output: impl std::io::Write,
            ) -> anyhow::Result<()> {
                output.write_all(TRANSPORT_MAGIC)?;
                write_framed_bytes(&mut output, self.invocation_id.as_bytes())?;
                write_framed_bytes(&mut output, &self.graph_bytes)?;
                write_framed_bytes(&mut output, &self.rendered_buck)?;
                output.write_all(&self.receipt_sha256)?;
                Ok(())
            }
        }

        fn validate_invocation_id(value: &str) -> anyhow::Result<()> {
            anyhow::ensure!(!value.is_empty(), "artifact invocation id is empty");
            anyhow::ensure!(
                value.len() <= MAX_INVOCATION_ID_BYTES,
                "artifact invocation id exceeds byte bound"
            );
            anyhow::ensure!(
                !value.chars().any(char::is_control),
                "artifact invocation id contains a control character"
            );
            Ok(())
        }

        fn artifact_receipt(
            invocation_id: &str,
            graph: &[u8],
            rendered: &[u8],
        ) -> anyhow::Result<[u8; 32]> {
            let mut hash = sha2::Sha256::new();
            hash.update(RECEIPT_DOMAIN);
            hash_framed_bytes(&mut hash, invocation_id.as_bytes())?;
            hash_framed_bytes(&mut hash, graph)?;
            hash_framed_bytes(&mut hash, rendered)?;
            Ok(hash.finalize().into())
        }

        fn hash_framed_bytes(hash: &mut sha2::Sha256, bytes: &[u8]) -> anyhow::Result<()> {
            let length = u64::try_from(bytes.len()).context("receipt field exceeds u64")?;
            hash.update(length.to_be_bytes());
            hash.update(bytes);
            Ok(())
        }

        fn write_framed_bytes(
            output: &mut impl std::io::Write,
            bytes: &[u8],
        ) -> anyhow::Result<()> {
            let length = u64::try_from(bytes.len()).context("artifact field exceeds u64")?;
            output.write_all(&length.to_be_bytes())?;
            output.write_all(bytes)?;
            Ok(())
        }
    }
}
