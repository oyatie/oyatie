fn render_reindeer_artifact_builders_v1() -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let sequence = render_reindeer_artifact_sequence_tokens_v1();
    render_provider_module_v1(quote::quote! {
        use serde::Serialize;
        use serde::ser::SerializeMap;
        use serde::ser::SerializeSeq;
        use serde::ser::SerializeStruct;
        use serde::ser::SerializeTuple;
        use serde::ser::SerializeTupleStruct;
        use serde_starlark::MULTILINE;

        use super::ReindeerValueErrorV1;
        use super::ReindeerValueSerializerV1;
        use crate::artifact::value::ReindeerCallArgumentsV1;
        use crate::artifact::value::ReindeerValueV1;
        use crate::artifact::MAX_CONTAINER_ITEMS;
        use crate::artifact::MAX_STRING_BYTES;

        #sequence

        pub(in crate::artifact) struct ReindeerMapBuilderV1 {
            element_serializer: ReindeerValueSerializerV1,
            entries: Vec<(ReindeerValueV1, ReindeerValueV1)>,
            next_key: Option<ReindeerValueV1>,
        }

        impl ReindeerMapBuilderV1 {
            pub(super) fn new(
                serializer: ReindeerValueSerializerV1,
                len: Option<usize>,
            ) -> Result<Self, ReindeerValueErrorV1> {
                let capacity = collection_capacity_hint(len)?;
                serializer.charge(1)?;
                Ok(Self {
                    element_serializer: serializer.child()?,
                    entries: Vec::with_capacity(capacity),
                    next_key: None,
                })
            }
        }

        impl SerializeMap for ReindeerMapBuilderV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;

            fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                if self.next_key.is_some() {
                    return Err(ReindeerValueErrorV1::refused("map value is missing"));
                }
                self.next_key = Some(key.serialize(self.element_serializer.clone())?);
                Ok(())
            }

            fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                let key = self
                    .next_key
                    .take()
                    .ok_or_else(|| ReindeerValueErrorV1::refused("map key is missing"))?;
                ensure_collection_bound(self.entries.len().saturating_add(1))?;
                self.element_serializer.charge(2)?;
                self.entries
                    .push((key, value.serialize(self.element_serializer.clone())?));
                Ok(())
            }

            fn end(mut self) -> Result<Self::Ok, Self::Error> {
                if self.next_key.is_some() {
                    return Err(ReindeerValueErrorV1::refused("map value is missing"));
                }
                self.entries.sort();
                if self.entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(ReindeerValueErrorV1::refused("duplicate map key"));
                }
                Ok(ReindeerValueV1::Map(self.entries))
            }
        }

        pub(in crate::artifact) struct ReindeerStructBuilderV1 {
            name: &'static str,
            element_serializer: ReindeerValueSerializerV1,
            fields: Vec<(String, ReindeerValueV1)>,
        }

        impl ReindeerStructBuilderV1 {
            pub(super) fn new(
                serializer: ReindeerValueSerializerV1,
                name: &'static str,
                len: usize,
            ) -> Result<Self, ReindeerValueErrorV1> {
                let capacity = collection_capacity_hint(Some(len))?;
                if name.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "callee exceeds byte bound",
                    ));
                }
                serializer.charge(name.len().saturating_add(2))?;
                Ok(Self {
                    name,
                    element_serializer: serializer.child()?,
                    fields: Vec::with_capacity(capacity),
                })
            }
        }

        impl SerializeStruct for ReindeerStructBuilderV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;

            fn serialize_field<T>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                ensure_collection_bound(self.fields.len().saturating_add(1))?;
                self.element_serializer.charge(1)?;
                if key.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "field name exceeds byte bound",
                    ));
                }
                self.element_serializer
                    .charge(key.len().saturating_add(1))?;
                self.fields.push((
                    key.to_owned(),
                    value.serialize(self.element_serializer.clone())?,
                ));
                Ok(())
            }

            fn end(mut self) -> Result<Self::Ok, Self::Error> {
                let callee = if self.name == "(" {
                    let Some(position) = self.fields.iter().position(|(name, _)| name.is_empty())
                    else {
                        return Err(ReindeerValueErrorV1::refused(
                            "renamed call is missing its callee field",
                        ));
                    };
                    let (_, ReindeerValueV1::String(callee)) = self.fields.remove(position) else {
                        return Err(ReindeerValueErrorV1::refused(
                            "renamed call callee is not a string",
                        ));
                    };
                    callee
                } else {
                    self.name.to_owned()
                };
                if self
                    .fields
                    .iter()
                    .any(|(name, _)| name == "*key" || name == "*value")
                {
                    if !self.fields.len().is_multiple_of(2) {
                        return Err(ReindeerValueErrorV1::refused(
                            "named call has an incomplete key/value pair",
                        ));
                    }
                    let mut named = Vec::with_capacity(self.fields.len() / 2);
                    let mut fields = self.fields.into_iter();
                    while let Some((key_marker, key)) = fields.next() {
                        let Some((value_marker, value)) = fields.next() else {
                            return Err(ReindeerValueErrorV1::refused(
                                "named call has an incomplete key/value pair",
                            ));
                        };
                        let ReindeerValueV1::String(key) = key else {
                            return Err(ReindeerValueErrorV1::refused(
                                "named call key is not a string",
                            ));
                        };
                        if key_marker != "*key" || value_marker != "*value" {
                            return Err(ReindeerValueErrorV1::refused(
                                "named call key/value markers are malformed",
                            ));
                        }
                        named.push((key, value));
                    }
                    self.fields = named;
                }
                self.fields.sort_by(|left, right| left.0.cmp(&right.0));
                if self.fields.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(ReindeerValueErrorV1::refused(
                        "duplicate named call field",
                    ));
                }
                Ok(ReindeerValueV1::Call {
                    callee,
                    arguments: ReindeerCallArgumentsV1::Named(self.fields),
                })
            }
        }

        fn ensure_collection_bound(len: usize) -> Result<(), ReindeerValueErrorV1> {
            if len > MAX_CONTAINER_ITEMS {
                return Err(ReindeerValueErrorV1::refused(
                    "collection exceeds item bound",
                ));
            }
            Ok(())
        }

        fn collection_capacity_hint(
            len: Option<usize>,
        ) -> Result<usize, ReindeerValueErrorV1> {
            match len {
                None | Some(MULTILINE) => Ok(0),
                Some(len) => {
                    ensure_collection_bound(len)?;
                    Ok(len)
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use super::MAX_CONTAINER_ITEMS;
            use super::collection_capacity_hint;

            #[test]
            fn artifact_formatting_sentinel_is_not_an_item_count() {
                assert_eq!(
                    collection_capacity_hint(Some(serde_starlark::MULTILINE)).unwrap(),
                    0
                );
            }

            #[test]
            fn artifact_excessive_capacity_hint_is_refused() {
                assert!(collection_capacity_hint(Some(MAX_CONTAINER_ITEMS + 1)).is_err());
            }
        }
    })
}
