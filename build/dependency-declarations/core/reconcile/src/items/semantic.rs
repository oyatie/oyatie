#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum SemanticValueKindV1 {
    None,
    Bool(bool),
    Signed(i128),
    Unsigned(u128),
    String(Box<str>),
    Identifier(Box<str>),
    List(Box<[SemanticValueV1]>),
    Tuple(Box<[SemanticValueV1]>),
    Map(Box<[(SemanticValueV1, SemanticValueV1)]>),
    Call {
        callee: Box<str>,
        arguments: CallArgumentsV1,
    },
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum CallArgumentsV1 {
    Positional(Box<[SemanticValueV1]>),
    Named(Box<[(Box<str>, SemanticValueV1)]>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ValueMetricsV1 {
    depth: usize,
    encoded_bytes: usize,
    nodes: usize,
}

/// A bounded typed Starlark value shared by producer and parser projections.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticValueV1 {
    kind: SemanticValueKindV1,
    metrics: ValueMetricsV1,
}

impl SemanticValueV1 {
    /// Creates the Starlark `None` value.
    #[must_use]
    pub const fn none() -> Self {
        Self::primitive(SemanticValueKindV1::None, 1)
    }

    /// Creates a Boolean value.
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::primitive(SemanticValueKindV1::Bool(value), 2)
    }

    /// Creates a signed integer value.
    #[must_use]
    pub const fn signed(value: i128) -> Self {
        Self::primitive(SemanticValueKindV1::Signed(value), 17)
    }

    /// Creates an unsigned integer value.
    #[must_use]
    pub const fn unsigned(value: u128) -> Self {
        Self::primitive(SemanticValueKindV1::Unsigned(value), 17)
    }

    /// Creates a bounded exact string value.
    pub fn string(value: impl Into<String>) -> Result<Self, FailureV1> {
        Self::text(value.into(), false)
    }

    /// Creates a bounded nonempty identifier value.
    pub fn identifier(value: impl Into<String>) -> Result<Self, FailureV1> {
        Self::text(value.into(), true)
    }

    /// Creates a bounded ordered list.
    pub fn list(values: Vec<Self>) -> Result<Self, FailureV1> {
        Self::sequence(values, false)
    }

    /// Creates a bounded ordered tuple.
    pub fn tuple(values: Vec<Self>) -> Result<Self, FailureV1> {
        Self::sequence(values, true)
    }

    /// Creates a canonically ordered map and refuses duplicate keys.
    pub fn map(mut entries: Vec<(Self, Self)>) -> Result<Self, FailureV1> {
        check_container_len(entries.len())?;
        entries.sort();
        if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid_graph());
        }
        let metrics = composite_metrics(entries.iter().flat_map(|(key, value)| [key, value]), 9)?;
        Ok(Self {
            kind: SemanticValueKindV1::Map(entries.into_boxed_slice()),
            metrics,
        })
    }

    /// Creates a bounded call with ordered positional arguments.
    pub fn call_positional(
        callee: impl Into<String>,
        values: Vec<Self>,
    ) -> Result<Self, FailureV1> {
        check_container_len(values.len())?;
        let callee = graph_text(callee.into(), true)?;
        let metrics = call_metrics(&callee, values.iter(), 18)?;
        Ok(Self {
            kind: SemanticValueKindV1::Call {
                callee,
                arguments: CallArgumentsV1::Positional(values.into_boxed_slice()),
            },
            metrics,
        })
    }

    /// Creates a bounded call with ordered, unique named arguments.
    pub fn call_named(
        callee: impl Into<String>,
        fields: Vec<(String, Self)>,
    ) -> Result<Self, FailureV1> {
        if fields.len() > ValidationBoundsV1::MAX_ATTRIBUTES_PER_RULE {
            return Err(invalid_graph());
        }
        let callee = graph_text(callee.into(), true)?;
        let mut normalized = Vec::with_capacity(fields.len());
        let mut field_bytes = 0_usize;
        for (name, value) in fields {
            let name = graph_text(name, true)?;
            field_bytes = checked_add(field_bytes, 8 + name.len())?;
            normalized.push((name, value));
        }
        normalized.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        if normalized.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid_graph());
        }
        let mut metrics = call_metrics(&callee, normalized.iter().map(|(_, value)| value), 18)?;
        metrics.encoded_bytes = checked_add(metrics.encoded_bytes, field_bytes)?;
        check_encoded(metrics.encoded_bytes)?;
        Ok(Self {
            kind: SemanticValueKindV1::Call {
                callee,
                arguments: CallArgumentsV1::Named(normalized.into_boxed_slice()),
            },
            metrics,
        })
    }

    fn text(value: String, identifier: bool) -> Result<Self, FailureV1> {
        let value = graph_text(value, identifier)?;
        let encoded_bytes = checked_add(9, value.len())?;
        Ok(Self {
            kind: if identifier {
                SemanticValueKindV1::Identifier(value)
            } else {
                SemanticValueKindV1::String(value)
            },
            metrics: ValueMetricsV1 {
                depth: 1,
                encoded_bytes,
                nodes: 1,
            },
        })
    }

    const fn primitive(kind: SemanticValueKindV1, encoded_bytes: usize) -> Self {
        Self {
            kind,
            metrics: ValueMetricsV1 {
                depth: 1,
                encoded_bytes,
                nodes: 1,
            },
        }
    }

    fn sequence(values: Vec<Self>, tuple: bool) -> Result<Self, FailureV1> {
        check_container_len(values.len())?;
        let metrics = composite_metrics(values.iter(), 9)?;
        Ok(Self {
            kind: if tuple {
                SemanticValueKindV1::Tuple(values.into_boxed_slice())
            } else {
                SemanticValueKindV1::List(values.into_boxed_slice())
            },
            metrics,
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        encode_semantic_value(&self.kind, hash)
    }

    pub(crate) const fn encoded_bytes(&self) -> usize {
        self.metrics.encoded_bytes
    }

    pub(crate) const fn nodes(&self) -> usize {
        self.metrics.nodes
    }

    pub(crate) fn named_string(&self, field_name: &str) -> Option<&str> {
        let SemanticValueKindV1::Call {
            arguments: CallArgumentsV1::Named(fields),
            ..
        } = &self.kind
        else {
            return None;
        };
        fields
            .iter()
            .find(|(name, _)| name.as_ref() == field_name)
            .and_then(|(_, value)| match &value.kind {
                SemanticValueKindV1::String(value) => Some(value.as_ref()),
                _ => None,
            })
    }
}

fn composite_metrics<'a>(
    values: impl Iterator<Item = &'a SemanticValueV1>,
    base_bytes: usize,
) -> Result<ValueMetricsV1, FailureV1> {
    let mut depth = 1_usize;
    let mut encoded_bytes = base_bytes;
    let mut nodes = 1_usize;
    for value in values {
        depth = depth.max(checked_add(value.metrics.depth, 1)?);
        encoded_bytes = checked_add(encoded_bytes, value.metrics.encoded_bytes)?;
        nodes = checked_add(nodes, value.metrics.nodes)?;
    }
    if depth > ValidationBoundsV1::MAX_VALUE_DEPTH || nodes > ValidationBoundsV1::MAX_SEMANTIC_NODES
    {
        return Err(invalid_graph());
    }
    check_encoded(encoded_bytes)?;
    Ok(ValueMetricsV1 {
        depth,
        encoded_bytes,
        nodes,
    })
}

fn call_metrics<'a>(
    callee: &str,
    values: impl Iterator<Item = &'a SemanticValueV1>,
    base_bytes: usize,
) -> Result<ValueMetricsV1, FailureV1> {
    composite_metrics(values, checked_add(base_bytes, callee.len())?)
}

fn check_container_len(length: usize) -> Result<(), FailureV1> {
    (length <= ValidationBoundsV1::MAX_LIST_ENTRIES)
        .then_some(())
        .ok_or_else(invalid_graph)
}

fn graph_text(value: String, nonempty: bool) -> Result<Box<str>, FailureV1> {
    if (nonempty && (value.is_empty() || value.chars().any(char::is_control)))
        || value.len() > ValidationBoundsV1::MAX_STRING_BYTES
        || (nonempty && !is_ascii_identifier(&value))
    {
        return Err(invalid_graph());
    }
    Ok(value.into_boxed_str())
}

fn is_ascii_identifier(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn checked_add(left: usize, right: usize) -> Result<usize, FailureV1> {
    left.checked_add(right).ok_or_else(invalid_graph)
}

fn check_encoded(bytes: usize) -> Result<(), FailureV1> {
    (bytes <= ValidationBoundsV1::MAX_GRAPH_BYTES)
        .then_some(())
        .ok_or_else(invalid_graph)
}
