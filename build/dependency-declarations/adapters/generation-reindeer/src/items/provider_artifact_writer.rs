fn render_reindeer_artifact_writer_tokens_v1() -> proc_macro2::TokenStream {
    quote::quote! {
        struct BoundedRenderedBytesV1 {
            bytes: Vec<u8>,
            max_bytes: usize,
        }

        impl BoundedRenderedBytesV1 {
            fn new(max_bytes: usize) -> Self {
                Self {
                    bytes: Vec::new(),
                    max_bytes,
                }
            }

            fn len(&self) -> usize {
                self.bytes.len()
            }

            fn into_boxed_slice(self) -> Box<[u8]> {
                self.bytes.into_boxed_slice()
            }
        }

        impl std::io::Write for BoundedRenderedBytesV1 {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                let next = self
                    .bytes
                    .len()
                    .checked_add(bytes.len())
                    .ok_or_else(|| std::io::Error::other("rendered BUCK byte length overflow"))?;
                if next > self.max_bytes {
                    return Err(std::io::Error::other(
                        "rendered BUCK exceeds byte bound",
                    ));
                }
                self.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        #[cfg(test)]
        mod tests {
            use std::io::Write as _;

            use super::BoundedRenderedBytesV1;
            use super::MAX_RULES;
            use super::ReindeerValueSerializerV1;
            use super::validate_rule_count;

            #[test]
            fn artifact_rule_count_refuses_before_graph_normalization() {
                assert!(validate_rule_count(MAX_RULES).is_ok());
                assert!(validate_rule_count(MAX_RULES + 1).is_err());
            }

            #[test]
            fn artifact_rendered_writer_refuses_before_partial_overflow() {
                let mut output = BoundedRenderedBytesV1::new(3);
                output.write_all(b"abc").unwrap();

                assert!(output.write_all(b"d").is_err());
                assert_eq!(&*output.into_boxed_slice(), b"abc");
            }

            #[test]
            fn artifact_semantic_budget_is_shared_and_fail_closed() {
                let serializer = ReindeerValueSerializerV1::with_limit(3);
                let child = serializer.child().unwrap();
                serializer.charge(2).unwrap();
                child.charge(1).unwrap();

                assert!(child.charge(1).is_err());
                serializer.charge(0).unwrap();
            }
        }
    }
}
