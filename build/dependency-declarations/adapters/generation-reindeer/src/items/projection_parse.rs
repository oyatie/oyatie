const REINDEER_BUCK_DIALECT_V1: Dialect = Dialect {
    enable_def: false,
    enable_lambda: false,
    enable_load: true,
    enable_keyword_only_arguments: false,
    enable_positional_only_arguments: false,
    enable_types: DialectTypes::Disable,
    enable_load_reexport: false,
    enable_top_level_stmt: false,
    enable_f_strings: false,
    _non_exhaustive: (),
};

struct ParsedRuleV1 {
    span: Span,
    semantic: dependency_declarations_reconcile::SemanticValueV1,
}

fn project_reindeer_buck_v1(
    rendered: &[u8],
) -> Result<RenderedRuleGraphV1, ProjectionPortErrorV1> {
    if rendered.len() > ValidationBoundsV1::MAX_OUTPUT_BYTES {
        return Err(ProjectionPortErrorV1::OutputTooLarge);
    }
    let source = std::str::from_utf8(rendered)
        .map_err(|_| ProjectionPortErrorV1::InvalidSyntax)?;
    let module = AstModule::parse("BUCK", source.to_owned(), &REINDEER_BUCK_DIALECT_V1)
        .map_err(|_| ProjectionPortErrorV1::InvalidSyntax)?;
    let parsed_rules = collect_top_level_rules_v1(&module)?;
    let first_rule_start = parsed_rules
        .first()
        .map(|rule| span_start_v1(rule.span))
        .transpose()?;
    if let Some(first_rule_start) = first_rule_start {
        for span in module.comments() {
            if span_start_v1(*span)? >= first_rule_start {
                return Err(ProjectionPortErrorV1::UnsupportedSyntax);
            }
        }
    }
    let prefix_end = first_rule_start.unwrap_or(rendered.len());
    let prefix = rendered
        .get(..prefix_end)
        .ok_or(ProjectionPortErrorV1::InternalInvariant)?
        .to_vec();
    let rules = project_rule_fragments_v1(rendered, parsed_rules)?;
    RenderedRuleGraphV1::try_new(prefix, rules)
        .map_err(|_| ProjectionPortErrorV1::UnsupportedSyntax)
}

fn collect_top_level_rules_v1(
    module: &AstModule,
) -> Result<Vec<ParsedRuleV1>, ProjectionPortErrorV1> {
    let statements = match &module.statement().node {
        Stmt::Statements(statements) => statements.as_slice(),
        _ => std::slice::from_ref(module.statement()),
    };
    let mut saw_rule = false;
    let mut rules = Vec::new();
    for statement in statements {
        match &statement.node {
            Stmt::Load(_) if !saw_rule => {}
            Stmt::Expression(expression) => {
                saw_rule = true;
                rules.push(ParsedRuleV1 {
                    span: statement.span,
                    semantic: project_rule_expression_v1(expression)?,
                });
            }
            _ => return Err(ProjectionPortErrorV1::UnsupportedSyntax),
        }
    }
    Ok(rules)
}

fn project_rule_fragments_v1(
    rendered: &[u8],
    parsed: Vec<ParsedRuleV1>,
) -> Result<Vec<RenderedRuleV1>, ProjectionPortErrorV1> {
    let mut rules = Vec::with_capacity(parsed.len());
    for (index, rule) in parsed.iter().enumerate() {
        let start = span_start_v1(rule.span)?;
        let syntax_end = span_end_v1(rule.span)?;
        let fragment_end = if let Some(next) = parsed.get(index + 1) {
            let next_start = span_start_v1(next.span)?;
            if rendered.get(syntax_end..next_start) != Some(b"\n\n") {
                return Err(ProjectionPortErrorV1::UnsupportedSyntax);
            }
            next_start
                .checked_sub(1)
                .ok_or(ProjectionPortErrorV1::InternalInvariant)?
        } else {
            if rendered.get(syntax_end..) != Some(b"\n") {
                return Err(ProjectionPortErrorV1::UnsupportedSyntax);
            }
            rendered.len()
        };
        let fragment = rendered
            .get(start..fragment_end)
            .ok_or(ProjectionPortErrorV1::InternalInvariant)?;
        rules.push(RenderedRuleV1::new(
            u64::try_from(index).map_err(|_| ProjectionPortErrorV1::OutputTooLarge)?,
            rule.semantic.clone(),
            DigestV1::of(fragment),
        ));
    }
    Ok(rules)
}

fn span_start_v1(span: Span) -> Result<usize, ProjectionPortErrorV1> {
    usize::try_from(span.begin().get()).map_err(|_| ProjectionPortErrorV1::OutputTooLarge)
}

fn span_end_v1(span: Span) -> Result<usize, ProjectionPortErrorV1> {
    usize::try_from(span.end().get()).map_err(|_| ProjectionPortErrorV1::OutputTooLarge)
}
