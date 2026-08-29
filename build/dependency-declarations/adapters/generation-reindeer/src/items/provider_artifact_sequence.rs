fn render_reindeer_artifact_sequence_tokens_v1() -> proc_macro2::TokenStream {
    quote::quote! {
        enum ReindeerSequenceKindV1 {
            List,
            Tuple,
            Call(&'static str),
        }

        pub(in crate::artifact) struct ReindeerSequenceBuilderV1 {
            kind: ReindeerSequenceKindV1,
            element_serializer: ReindeerValueSerializerV1,
            values: Vec<ReindeerValueV1>,
        }

        impl ReindeerSequenceBuilderV1 {
            pub(super) fn list(
                serializer: ReindeerValueSerializerV1,
                len: Option<usize>,
            ) -> Result<Self, ReindeerValueErrorV1> {
                Self::new(
                    ReindeerSequenceKindV1::List,
                    serializer,
                    len.unwrap_or_default(),
                )
            }

            pub(super) fn tuple(
                serializer: ReindeerValueSerializerV1,
                len: usize,
            ) -> Result<Self, ReindeerValueErrorV1> {
                Self::new(ReindeerSequenceKindV1::Tuple, serializer, len)
            }

            pub(super) fn call(
                serializer: ReindeerValueSerializerV1,
                name: &'static str,
                len: usize,
            ) -> Result<Self, ReindeerValueErrorV1> {
                if name.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "callee exceeds byte bound",
                    ));
                }
                serializer.charge(name.len().saturating_add(1))?;
                Self::new(ReindeerSequenceKindV1::Call(name), serializer, len)
            }

            fn new(
                kind: ReindeerSequenceKindV1,
                serializer: ReindeerValueSerializerV1,
                len: usize,
            ) -> Result<Self, ReindeerValueErrorV1> {
                ensure_collection_bound(len)?;
                reserve_collection_budget(&serializer, len, 1)?;
                Ok(Self {
                    kind,
                    element_serializer: serializer.child()?,
                    values: Vec::with_capacity(len),
                })
            }

            fn push<T>(&mut self, value: &T) -> Result<(), ReindeerValueErrorV1>
            where
                T: Serialize + ?Sized,
            {
                ensure_collection_bound(self.values.len().saturating_add(1))?;
                self.values
                    .push(value.serialize(self.element_serializer.clone())?);
                Ok(())
            }

            fn finish(mut self) -> Result<ReindeerValueV1, ReindeerValueErrorV1> {
                Ok(match self.kind {
                    ReindeerSequenceKindV1::List => ReindeerValueV1::List(self.values),
                    ReindeerSequenceKindV1::Tuple => ReindeerValueV1::Tuple(self.values),
                    ReindeerSequenceKindV1::Call("(") => {
                        let Some(ReindeerValueV1::String(callee)) = self.values.first() else {
                            return Err(ReindeerValueErrorV1::refused(
                                "renamed call is missing a string callee",
                            ));
                        };
                        let callee = callee.clone();
                        self.values.remove(0);
                        ReindeerValueV1::Call {
                            callee,
                            arguments: ReindeerCallArgumentsV1::Positional(self.values),
                        }
                    }
                    ReindeerSequenceKindV1::Call(callee) => ReindeerValueV1::Call {
                        callee: callee.to_owned(),
                        arguments: ReindeerCallArgumentsV1::Positional(self.values),
                    },
                })
            }
        }

        impl SerializeSeq for ReindeerSequenceBuilderV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;

            fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                self.push(value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.finish()
            }
        }

        impl SerializeTuple for ReindeerSequenceBuilderV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;

            fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                self.push(value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.finish()
            }
        }

        impl SerializeTupleStruct for ReindeerSequenceBuilderV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;

            fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                self.push(value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.finish()
            }
        }
    }
}
