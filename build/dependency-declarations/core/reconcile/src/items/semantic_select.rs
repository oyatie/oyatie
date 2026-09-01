impl SemanticValueV1 {
    /// Creates the closed list/select addition emitted by the qualified renderer.
    pub fn select_addition(values: Vec<Self>) -> Result<Self, FailureV1> {
        check_container_len(values.len())?;
        let Some((first, remaining)) = values.split_first() else {
            return Err(invalid_graph());
        };
        if remaining.is_empty()
            || (!matches!(first.kind, SemanticValueKindV1::List(_))
                && !is_select_projection(first))
            || remaining.iter().any(|value| !is_select_projection(value))
        {
            return Err(invalid_graph());
        }
        let callee: Box<str> = "+".into();
        let metrics = call_metrics(&callee, values.iter(), 18)?;
        Ok(Self {
            kind: SemanticValueKindV1::Call {
                callee,
                arguments: CallArgumentsV1::Positional(values.into_boxed_slice()),
            },
            metrics,
        })
    }
}

fn is_select_projection(value: &SemanticValueV1) -> bool {
    matches!(
        &value.kind,
        SemanticValueKindV1::Call {
            callee,
            arguments: CallArgumentsV1::Positional(arguments),
        } if callee.as_ref() == "select"
            && matches!(arguments.as_ref(), [argument] if matches!(argument.kind, SemanticValueKindV1::Map(_)))
    )
}
