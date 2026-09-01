fn encode_semantic_value(
    value: &SemanticValueKindV1,
    hash: &mut CanonicalHasherV1,
) -> Result<(), FailureV1> {
    match value {
        SemanticValueKindV1::None => hash.tag(0),
        SemanticValueKindV1::Bool(value) => {
            hash.tag(1);
            hash.boolean(*value);
        }
        SemanticValueKindV1::Signed(value) => {
            hash.tag(2);
            hash.i128(*value);
        }
        SemanticValueKindV1::Unsigned(value) => {
            hash.tag(3);
            hash.u128(*value);
        }
        SemanticValueKindV1::String(value) => {
            hash.tag(4);
            hash.string(value)?;
        }
        SemanticValueKindV1::Identifier(value) => {
            hash.tag(5);
            hash.string(value)?;
        }
        SemanticValueKindV1::List(values) | SemanticValueKindV1::Tuple(values) => {
            hash.tag(if matches!(value, SemanticValueKindV1::List(_)) {
                6
            } else {
                7
            });
            encode_semantic_sequence(values, hash)?;
        }
        SemanticValueKindV1::Map(entries) => {
            hash.tag(8);
            hash.u64(checked_u64(entries.len(), invalid_graph())?);
            for (key, value) in entries {
                key.encode(hash)?;
                value.encode(hash)?;
            }
        }
        SemanticValueKindV1::Call { callee, arguments } => {
            hash.tag(9);
            hash.string(callee)?;
            match arguments {
                CallArgumentsV1::Positional(values) => {
                    hash.tag(0);
                    encode_semantic_sequence(values, hash)?;
                }
                CallArgumentsV1::Named(fields) => {
                    hash.tag(1);
                    hash.u64(checked_u64(fields.len(), invalid_graph())?);
                    for (name, value) in fields {
                        hash.string(name)?;
                        value.encode(hash)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn encode_semantic_sequence(
    values: &[SemanticValueV1],
    hash: &mut CanonicalHasherV1,
) -> Result<(), FailureV1> {
    hash.u64(checked_u64(values.len(), invalid_graph())?);
    for value in values {
        value.encode(hash)?;
    }
    Ok(())
}
