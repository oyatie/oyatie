fn render_reindeer_artifact_serializer_v1() -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let serializer_impl = render_reindeer_artifact_serializer_impl_tokens_v1();
    render_provider_module_v1(quote::quote! {
        mod builders;

        use std::cell::Cell;
        use std::fmt;
        use std::rc::Rc;

        use serde::Serialize;
        use serde::ser::Impossible;
        use serde::ser::Serializer;

        use self::builders::ReindeerMapBuilderV1;
        use self::builders::ReindeerSequenceBuilderV1;
        use self::builders::ReindeerStructBuilderV1;
        use super::value::ReindeerCallArgumentsV1;
        use super::value::ReindeerValueV1;
        use super::MAX_SEMANTIC_BYTES;
        use super::MAX_STRING_BYTES;
        use super::MAX_VALUE_DEPTH;

        #[derive(Debug)]
        pub(super) struct ReindeerValueErrorV1(String);

        impl ReindeerValueErrorV1 {
            pub(super) fn refused(message: impl Into<String>) -> Self {
                Self(message.into())
            }
        }

        impl fmt::Display for ReindeerValueErrorV1 {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl std::error::Error for ReindeerValueErrorV1 {}

        impl serde::ser::Error for ReindeerValueErrorV1 {
            fn custom<T: fmt::Display>(message: T) -> Self {
                Self(message.to_string())
            }
        }

        #[derive(Clone)]
        pub(super) struct ReindeerValueSerializerV1 {
            depth: usize,
            semantic_budget: Rc<ReindeerSemanticBudgetV1>,
        }

        struct ReindeerSemanticBudgetV1 {
            used_bytes: Cell<usize>,
            max_bytes: usize,
        }

        impl ReindeerValueSerializerV1 {
            pub(super) fn root() -> Self {
                Self::with_limit(MAX_SEMANTIC_BYTES)
            }

            pub(super) fn with_limit(max_bytes: usize) -> Self {
                Self {
                    depth: 0,
                    semantic_budget: Rc::new(ReindeerSemanticBudgetV1 {
                        used_bytes: Cell::new(0),
                        max_bytes,
                    }),
                }
            }

            pub(super) fn child(&self) -> Result<Self, ReindeerValueErrorV1> {
                let depth = self
                    .depth
                    .checked_add(1)
                    .ok_or_else(|| ReindeerValueErrorV1::refused("value depth overflow"))?;
                if depth > MAX_VALUE_DEPTH {
                    return Err(ReindeerValueErrorV1::refused(
                        "value exceeds nesting-depth bound",
                    ));
                }
                Ok(Self {
                    depth,
                    semantic_budget: Rc::clone(&self.semantic_budget),
                })
            }

            pub(super) fn charge(&self, bytes: usize) -> Result<(), ReindeerValueErrorV1> {
                let next = self
                    .semantic_budget
                    .used_bytes
                    .get()
                    .checked_add(bytes)
                    .ok_or_else(|| ReindeerValueErrorV1::refused("semantic byte overflow"))?;
                if next > self.semantic_budget.max_bytes {
                    return Err(ReindeerValueErrorV1::refused(
                        "rule graph exceeds semantic-byte bound",
                    ));
                }
                self.semantic_budget.used_bytes.set(next);
                Ok(())
            }

            fn string(&self, value: &str) -> Result<ReindeerValueV1, ReindeerValueErrorV1> {
                if value.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "string exceeds byte bound",
                    ));
                }
                self.charge(value.len().saturating_add(1))?;
                Ok(ReindeerValueV1::String(value.to_owned()))
            }
        }

        #serializer_impl
    })
}
