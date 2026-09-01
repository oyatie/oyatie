fn project_rule_expression_v1(
    expression: &AstExpr,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    let ExprP::Call(callee, arguments) = &expression.node else {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    };
    project_call_v1(callee, arguments, 1)
}

fn project_expression_v1(
    expression: &AstExpr,
    depth: usize,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    if depth > ValidationBoundsV1::MAX_VALUE_DEPTH {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    match &expression.node {
        ExprP::Tuple(values) => SemanticValueV1::tuple(project_sequence_v1(values, depth)?)
            .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax),
        ExprP::Call(callee, arguments) => project_call_v1(callee, arguments, depth),
        ExprP::Identifier(identifier) => project_identifier_v1(&identifier.node.ident),
        ExprP::Literal(literal) => project_literal_v1(literal),
        ExprP::Minus(value) => project_negative_integer_v1(value),
        ExprP::Op(left, BinOp::Add, right) => project_addition_v1(left, right, depth),
        ExprP::List(values) => SemanticValueV1::list(project_sequence_v1(values, depth)?)
            .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax),
        ExprP::Dict(entries) => project_map_v1(entries, depth),
        ExprP::Dot(_, _)
        | ExprP::Index(_)
        | ExprP::Index2(_)
        | ExprP::Slice(_, _, _, _)
        | ExprP::Lambda(_)
        | ExprP::Not(_)
        | ExprP::Plus(_)
        | ExprP::BitNot(_)
        | ExprP::Op(_, _, _)
        | ExprP::If(_)
        | ExprP::ListComprehension(_, _, _)
        | ExprP::DictComprehension(_, _, _)
        | ExprP::FString(_) => Err(ProjectionPortErrorV1::UnsupportedSyntax),
    }
}

fn project_addition_v1(
    left: &AstExpr,
    right: &AstExpr,
    depth: usize,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    let value_depth = next_projection_depth_v1(depth)?;
    let mut pending = vec![right, left];
    let mut values = Vec::new();
    let mut saw_list = false;
    while let Some(expression) = pending.pop() {
        if let ExprP::Op(left, BinOp::Add, right) = &expression.node {
            pending.push(right);
            pending.push(left);
            continue;
        }
        if values.len() >= ValidationBoundsV1::MAX_LIST_ENTRIES {
            return Err(ProjectionPortErrorV1::UnsupportedSyntax);
        }
        match &expression.node {
            ExprP::List(_) if values.is_empty() && !saw_list => saw_list = true,
            ExprP::Call(callee, arguments) if canonical_select_call_v1(callee, arguments) => {}
            _ => return Err(ProjectionPortErrorV1::UnsupportedSyntax),
        }
        values.push(project_expression_v1(expression, value_depth)?);
    }
    if values.len() < 2 {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    SemanticValueV1::select_addition(values)
        .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax)
}

fn canonical_select_call_v1(
    callee: &AstExpr,
    arguments: &starlark_syntax::syntax::ast::CallArgsP<starlark_syntax::syntax::ast::AstNoPayload>,
) -> bool {
    matches!(&callee.node, ExprP::Identifier(identifier) if identifier.node.ident == "select")
        && matches!(
            arguments.args.as_slice(),
            [argument]
                if matches!(
                    &argument.node,
                    ArgumentP::Positional(value) if matches!(value.node, ExprP::Dict(_))
                )
        )
}

fn project_call_v1(
    callee: &AstExpr,
    arguments: &starlark_syntax::syntax::ast::CallArgsP<starlark_syntax::syntax::ast::AstNoPayload>,
    depth: usize,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    if arguments.args.is_empty()
        || arguments.args.len() > ValidationBoundsV1::MAX_LIST_ENTRIES
    {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    let callee = project_callee_v1(callee, depth)?;
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for argument in &arguments.args {
        match &argument.node {
            ArgumentP::Positional(value) if named.is_empty() => {
                positional.push(project_expression_v1(value, next_projection_depth_v1(depth)?)?);
            }
            ArgumentP::Named(name, value) if positional.is_empty() => named.push((
                name.node.clone(),
                project_expression_v1(value, next_projection_depth_v1(depth)?)?,
            )),
            ArgumentP::Positional(_)
            | ArgumentP::Named(_, _)
            | ArgumentP::Args(_)
            | ArgumentP::KwArgs(_) => {
                return Err(ProjectionPortErrorV1::UnsupportedSyntax);
            }
        }
    }
    let semantic = if named.is_empty() {
        SemanticValueV1::call_positional(callee, positional)
    } else {
        SemanticValueV1::call_named(callee, named)
    };
    semantic.map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax)
}

fn project_callee_v1(
    expression: &AstExpr,
    depth: usize,
) -> Result<String, ProjectionPortErrorV1> {
    let mut current = expression;
    let mut current_depth = depth;
    let mut reversed = Vec::new();
    loop {
        if current_depth > ValidationBoundsV1::MAX_VALUE_DEPTH {
            return Err(ProjectionPortErrorV1::UnsupportedSyntax);
        }
        match &current.node {
            ExprP::Identifier(identifier) => {
                reversed.push(identifier.node.ident.as_str());
                break;
            }
            ExprP::Dot(parent, field) => {
                reversed.push(field.node.as_str());
                current = parent;
                current_depth = next_projection_depth_v1(current_depth)?;
            }
            _ => return Err(ProjectionPortErrorV1::UnsupportedSyntax),
        }
    }
    reversed.reverse();
    let bytes = reversed
        .iter()
        .try_fold(reversed.len().saturating_sub(1), |total, component| {
            total
                .checked_add(component.len())
                .ok_or(ProjectionPortErrorV1::UnsupportedSyntax)
        })?;
    if bytes > ValidationBoundsV1::MAX_STRING_BYTES {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    let mut callee = String::with_capacity(bytes);
    for (index, component) in reversed.into_iter().enumerate() {
        if index != 0 {
            callee.push('.');
        }
        callee.push_str(component);
    }
    Ok(callee)
}

fn project_identifier_v1(value: &str) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    match value {
        "None" => Ok(SemanticValueV1::none()),
        "True" => Ok(SemanticValueV1::boolean(true)),
        "False" => Ok(SemanticValueV1::boolean(false)),
        _ => SemanticValueV1::identifier(value)
            .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax),
    }
}

fn project_literal_v1(
    literal: &AstLiteral,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    match literal {
        AstLiteral::Int(value) => value
            .node
            .to_string()
            .parse::<i32>()
            .map(|value| SemanticValueV1::signed(value.into()))
            .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax),
        AstLiteral::String(value) => SemanticValueV1::string(value.node.clone())
            .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax),
        AstLiteral::Float(_) | AstLiteral::Bytes(_) | AstLiteral::Ellipsis => {
            Err(ProjectionPortErrorV1::UnsupportedSyntax)
        }
    }
}

fn project_negative_integer_v1(
    expression: &AstExpr,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    let ExprP::Literal(AstLiteral::Int(value)) = &expression.node else {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    };
    let magnitude = value
        .node
        .to_string()
        .parse::<i64>()
        .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax)?;
    let value = magnitude
        .checked_neg()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(ProjectionPortErrorV1::UnsupportedSyntax)?;
    Ok(SemanticValueV1::signed(value.into()))
}

fn project_sequence_v1(
    values: &[AstExpr],
    depth: usize,
) -> Result<Vec<SemanticValueV1>, ProjectionPortErrorV1> {
    if values.len() > ValidationBoundsV1::MAX_LIST_ENTRIES {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    values
        .iter()
        .map(|value| project_expression_v1(value, next_projection_depth_v1(depth)?))
        .collect()
}

fn project_map_v1(
    entries: &[(AstExpr, AstExpr)],
    depth: usize,
) -> Result<SemanticValueV1, ProjectionPortErrorV1> {
    if entries.len() > ValidationBoundsV1::MAX_LIST_ENTRIES {
        return Err(ProjectionPortErrorV1::UnsupportedSyntax);
    }
    let entries = entries
        .iter()
        .map(|(key, value)| {
            let depth = next_projection_depth_v1(depth)?;
            Ok((
                project_expression_v1(key, depth)?,
                project_expression_v1(value, depth)?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionPortErrorV1>>()?;
    SemanticValueV1::map(entries).map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax)
}

fn next_projection_depth_v1(depth: usize) -> Result<usize, ProjectionPortErrorV1> {
    depth
        .checked_add(1)
        .ok_or(ProjectionPortErrorV1::UnsupportedSyntax)
}
