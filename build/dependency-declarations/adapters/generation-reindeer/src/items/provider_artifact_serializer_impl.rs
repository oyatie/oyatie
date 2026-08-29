fn render_reindeer_artifact_serializer_impl_tokens_v1() -> proc_macro2::TokenStream {
    quote::quote! {
        impl Serializer for ReindeerValueSerializerV1 {
            type Ok = ReindeerValueV1;
            type Error = ReindeerValueErrorV1;
            type SerializeSeq = ReindeerSequenceBuilderV1;
            type SerializeTuple = ReindeerSequenceBuilderV1;
            type SerializeTupleStruct = ReindeerSequenceBuilderV1;
            type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
            type SerializeMap = ReindeerMapBuilderV1;
            type SerializeStruct = ReindeerStructBuilderV1;
            type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

            fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
                self.charge(2)?;
                Ok(ReindeerValueV1::Bool(value))
            }

            fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
                self.serialize_i128(value.into())
            }

            fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
                self.serialize_i128(value.into())
            }

            fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
                self.serialize_i128(value.into())
            }

            fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
                self.serialize_i128(value.into())
            }

            fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
                self.charge(17)?;
                Ok(ReindeerValueV1::Signed(value))
            }

            fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
                self.serialize_u128(value.into())
            }

            fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
                self.serialize_u128(value.into())
            }

            fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
                self.serialize_u128(value.into())
            }

            fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
                self.serialize_u128(value.into())
            }

            fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
                self.charge(17)?;
                Ok(ReindeerValueV1::Unsigned(value))
            }

            fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
                Err(ReindeerValueErrorV1::refused(
                    "floating point is not admitted",
                ))
            }

            fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
                Err(ReindeerValueErrorV1::refused(
                    "floating point is not admitted",
                ))
            }

            fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
                self.string(&value.to_string())
            }

            fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
                self.string(value)
            }

            fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
                Err(ReindeerValueErrorV1::refused(
                    "byte strings are not admitted",
                ))
            }

            fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
                self.charge(1)?;
                Ok(ReindeerValueV1::None)
            }

            fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: Serialize + ?Sized,
            {
                value.serialize(self.child()?)
            }

            fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
                Err(ReindeerValueErrorV1::refused("unit is not admitted"))
            }

            fn serialize_unit_struct(
                self,
                name: &'static str,
            ) -> Result<Self::Ok, Self::Error> {
                if name.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "identifier exceeds byte bound",
                    ));
                }
                self.charge(name.len().saturating_add(1))?;
                Ok(ReindeerValueV1::Identifier(name.to_owned()))
            }

            fn serialize_unit_variant(
                self,
                _name: &'static str,
                _index: u32,
                variant: &'static str,
            ) -> Result<Self::Ok, Self::Error> {
                self.string(variant)
            }

            fn serialize_newtype_struct<T>(
                self,
                name: &'static str,
                value: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: Serialize + ?Sized,
            {
                if name.len() > MAX_STRING_BYTES {
                    return Err(ReindeerValueErrorV1::refused(
                        "callee exceeds byte bound",
                    ));
                }
                self.charge(name.len().saturating_add(1))?;
                Ok(ReindeerValueV1::Call {
                    callee: name.to_owned(),
                    arguments: ReindeerCallArgumentsV1::Positional(vec![
                        value.serialize(self.child()?)?,
                    ]),
                })
            }

            fn serialize_newtype_variant<T>(
                self,
                _name: &'static str,
                _index: u32,
                _variant: &'static str,
                _value: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: Serialize + ?Sized,
            {
                Err(ReindeerValueErrorV1::refused(
                    "newtype variants are not admitted",
                ))
            }

            fn serialize_seq(
                self,
                len: Option<usize>,
            ) -> Result<Self::SerializeSeq, Self::Error> {
                ReindeerSequenceBuilderV1::list(self, len)
            }

            fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
                ReindeerSequenceBuilderV1::tuple(self, len)
            }

            fn serialize_tuple_struct(
                self,
                name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeTupleStruct, Self::Error> {
                ReindeerSequenceBuilderV1::call(self, name, len)
            }

            fn serialize_tuple_variant(
                self,
                _name: &'static str,
                _index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleVariant, Self::Error> {
                Err(ReindeerValueErrorV1::refused(
                    "tuple variants are not admitted",
                ))
            }

            fn serialize_map(
                self,
                len: Option<usize>,
            ) -> Result<Self::SerializeMap, Self::Error> {
                ReindeerMapBuilderV1::new(self, len)
            }

            fn serialize_struct(
                self,
                name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeStruct, Self::Error> {
                ReindeerStructBuilderV1::new(self, name, len)
            }

            fn serialize_struct_variant(
                self,
                _name: &'static str,
                _index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeStructVariant, Self::Error> {
                Err(ReindeerValueErrorV1::refused(
                    "struct variants are not admitted",
                ))
            }
        }
    }
}
