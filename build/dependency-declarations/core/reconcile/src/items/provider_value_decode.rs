pub(crate) fn decode_provider_value_v1(
    cursor: &mut ProviderCursorV1<'_>,
    depth: usize,
    remaining_nodes: &mut usize,
) -> Result<SemanticValueV1, FailureV1> {
    if depth > ValidationBoundsV1::MAX_VALUE_DEPTH || *remaining_nodes == 0 {
        return Err(invalid_graph());
    }
    *remaining_nodes -= 1;
    match cursor.read_u8()? {
        0 => Ok(SemanticValueV1::none()),
        1 => match cursor.read_u8()? {
            0 => Ok(SemanticValueV1::boolean(false)),
            1 => Ok(SemanticValueV1::boolean(true)),
            _ => Err(invalid_graph()),
        },
        2 => Ok(SemanticValueV1::signed(cursor.read_i128()?)),
        3 => Ok(SemanticValueV1::unsigned(cursor.read_u128()?)),
        4 => SemanticValueV1::string(read_string(cursor)?),
        5 => SemanticValueV1::identifier(read_string(cursor)?),
        6 => SemanticValueV1::list(read_sequence(cursor, depth, remaining_nodes)?),
        7 => SemanticValueV1::tuple(read_sequence(cursor, depth, remaining_nodes)?),
        8 => read_map(cursor, depth, remaining_nodes),
        9 => read_call(cursor, depth, remaining_nodes),
        _ => Err(invalid_graph()),
    }
}

fn read_sequence(
    cursor: &mut ProviderCursorV1<'_>,
    depth: usize,
    remaining_nodes: &mut usize,
) -> Result<Vec<SemanticValueV1>, FailureV1> {
    let length = cursor.read_len(ValidationBoundsV1::MAX_LIST_ENTRIES)?;
    if length > *remaining_nodes {
        return Err(invalid_graph());
    }
    let mut values = Vec::new();
    for _ in 0..length {
        values.push(decode_provider_value_v1(
            cursor,
            next_depth(depth)?,
            remaining_nodes,
        )?);
    }
    Ok(values)
}

fn read_map(
    cursor: &mut ProviderCursorV1<'_>,
    depth: usize,
    remaining_nodes: &mut usize,
) -> Result<SemanticValueV1, FailureV1> {
    let length = cursor.read_len(ValidationBoundsV1::MAX_LIST_ENTRIES)?;
    if length
        .checked_mul(2)
        .is_none_or(|nodes| nodes > *remaining_nodes)
    {
        return Err(invalid_graph());
    }
    let mut entries = Vec::new();
    for _ in 0..length {
        let key = decode_provider_value_v1(cursor, next_depth(depth)?, remaining_nodes)?;
        if entries
            .last()
            .is_some_and(|(previous, _): &(SemanticValueV1, SemanticValueV1)| previous >= &key)
        {
            return Err(invalid_graph());
        }
        let value = decode_provider_value_v1(cursor, next_depth(depth)?, remaining_nodes)?;
        entries.push((key, value));
    }
    SemanticValueV1::map(entries)
}

fn read_call(
    cursor: &mut ProviderCursorV1<'_>,
    depth: usize,
    remaining_nodes: &mut usize,
) -> Result<SemanticValueV1, FailureV1> {
    let callee = read_string(cursor)?;
    match cursor.read_u8()? {
        0 => {
            SemanticValueV1::call_positional(callee, read_sequence(cursor, depth, remaining_nodes)?)
        }
        1 => {
            let length = cursor.read_len(ValidationBoundsV1::MAX_ATTRIBUTES_PER_RULE)?;
            if length > *remaining_nodes {
                return Err(invalid_graph());
            }
            let mut fields = Vec::new();
            for _ in 0..length {
                let name = read_string(cursor)?;
                if fields
                    .last()
                    .is_some_and(|(previous, _): &(String, SemanticValueV1)| previous >= &name)
                {
                    return Err(invalid_graph());
                }
                let value = decode_provider_value_v1(cursor, next_depth(depth)?, remaining_nodes)?;
                fields.push((name, value));
            }
            SemanticValueV1::call_named(callee, fields)
        }
        _ => Err(invalid_graph()),
    }
}

fn read_string(cursor: &mut ProviderCursorV1<'_>) -> Result<String, FailureV1> {
    Ok(cursor
        .read_text(ValidationBoundsV1::MAX_STRING_BYTES)?
        .to_owned())
}

fn next_depth(depth: usize) -> Result<usize, FailureV1> {
    depth.checked_add(1).ok_or_else(invalid_graph)
}

#[cfg(test)]
mod provider_value_decode_tests {
    use super::*;

    #[test]
    fn declared_collection_lengths_cannot_amplify_allocation_or_node_count() {
        let mut truncated = vec![6];
        truncated.extend_from_slice(
            &u64::try_from(ValidationBoundsV1::MAX_LIST_ENTRIES)
                .unwrap()
                .to_be_bytes(),
        );
        let mut remaining = ValidationBoundsV1::MAX_SEMANTIC_NODES;
        assert!(
            decode_provider_value_v1(&mut ProviderCursorV1::new(&truncated), 1, &mut remaining,)
                .is_err()
        );

        let mut three_nodes = vec![6];
        three_nodes.extend_from_slice(&2_u64.to_be_bytes());
        three_nodes.extend_from_slice(&[0, 0]);
        let mut remaining = 2;
        assert!(
            decode_provider_value_v1(&mut ProviderCursorV1::new(&three_nodes), 1, &mut remaining,)
                .is_err()
        );
    }
}
