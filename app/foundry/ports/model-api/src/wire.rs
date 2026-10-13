// Serde's default struct decoder also accepts positional arrays.
macro_rules! model_object {
    ($name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
        pub struct $name { $(pub $field: $ty),* }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields { $($(#[$field_meta])* $field: $ty),* }
                struct Object;
                impl<'de> serde::de::Visitor<'de> for Object {
                    type Value = $name;
                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(concat!("a ", stringify!($name), " object"))
                    }
                    fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A)
                        -> Result<Self::Value, A::Error> {
                        let row = <Fields as serde::Deserialize>::deserialize(
                            serde::de::value::MapAccessDeserializer::new(map))?;
                        Ok($name { $($field: row.$field),* })
                    }
                }
                decoder.deserialize_map(Object)
            }
        }
    };
}
