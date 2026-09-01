fn adapt_reindeer_buckify_v1(
    source: &str,
    syntax: &syn::File,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let do_buckify = exactly_one_provider_function_v1(&syntax.items, "do_buckify")?;
    let buckify = exactly_one_provider_function_v1(&syntax.items, "buckify")?;
    let mut edits = do_buckify_edits_v1(source, do_buckify)?;
    edits.push(workspace_dependency_roots_edit_v1(source, do_buckify)?);
    edits.extend(buckify_artifact_edits_v1(source, buckify)?);
    edits.push(public_rule_name_reservation_edit_v1(source, buckify)?);
    apply_source_edits_v1(source, edits)
}

fn public_rule_name_reservation_edit_v1(
    source: &str,
    function: &syn::ItemFn,
) -> Result<ReindeerProviderSourceEditV1, ReindeerProviderAdaptationErrorV1> {
    struct CallCollectorV1<'ast> {
        matches: Vec<&'ast syn::ExprCall>,
    }

    impl<'ast> syn::visit::Visit<'ast> for CallCollectorV1<'ast> {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            let is_constructor = matches!(
                call.func.as_ref(),
                syn::Expr::Path(path)
                    if path.path.segments.len() == 2
                        && path.path.segments[0].ident == "CollisionInfo"
                        && path.path.segments[1].ident == "new_with_reserved"
            );
            if is_constructor {
                self.matches.push(call);
            }
            syn::visit::visit_expr_call(self, call);
        }
    }

    let mut collector = CallCollectorV1 {
        matches: Vec::new(),
    };
    syn::visit::Visit::visit_block(&mut collector, &function.block);
    let call = exactly_one_iterator_v1(collector.matches.into_iter())?;
    let reserved_names = call
        .args
        .iter()
        .nth(1)
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if call.args.len() != 2 {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    ReindeerProviderSourceEditV1::replace(
        source,
        syn::spanned::Spanned::span(reserved_names),
        "index.public_rule_names()".to_owned(),
    )
}

fn exactly_one_provider_function_v1<'a>(
    items: &'a [syn::Item],
    name: &str,
) -> Result<&'a syn::ItemFn, ReindeerProviderAdaptationErrorV1> {
    let mut matches = items.iter().filter_map(|item| match item {
        syn::Item::Fn(function) if function.sig.ident == name => Some(function),
        _ => None,
    });
    let function = matches
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(function)
}

fn do_buckify_edits_v1(
    source: &str,
    function: &syn::ItemFn,
) -> Result<Vec<ReindeerProviderSourceEditV1>, ReindeerProviderAdaptationErrorV1> {
    let syn::ReturnType::Type(_, return_type) = &function.sig.output else {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    };
    let return_collection = nested_type_named_v1(return_type, "BTreeSet")?;

    let mut rules_types = function.block.stmts.iter().filter_map(|statement| {
        let syn::Stmt::Local(local) = statement else {
            return None;
        };
        let syn::Pat::Type(pattern) = &local.pat else {
            return None;
        };
        let syn::Pat::Ident(binding) = pattern.pat.as_ref() else {
            return None;
        };
        (binding.ident == "rules" && outer_type_name_v1(&pattern.ty).as_deref() == Some("BTreeSet"))
            .then_some(pattern.ty.as_ref())
    });
    let rules_type = rules_types
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if rules_types.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }

    let final_expression = function
        .block
        .stmts
        .last()
        .and_then(|statement| match statement {
            syn::Stmt::Expr(expression, None) => Some(expression),
            _ => None,
        })
        .filter(|expression| expression_is_ok_rules_v1(expression))
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;

    Ok(vec![
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(return_collection),
            "Vec<Rule>".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(rules_type),
            "Vec<_>".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(final_expression),
            concat!(
                "buck::sort_rules_for_artifact(rules)\n",
                "        .context(\"refusing invalid rule graph before graph collection\")",
            )
            .to_owned(),
        )?,
    ])
}

fn buckify_artifact_edits_v1(
    source: &str,
    function: &syn::ItemFn,
) -> Result<Vec<ReindeerProviderSourceEditV1>, ReindeerProviderAdaptationErrorV1> {
    let fast = exactly_one_typed_argument_v1(&function.sig.inputs, "fast")?;
    let unused_check = exactly_one_unused_check_v1(&function.block.stmts)?;
    Ok(vec![
        ReindeerProviderSourceEditV1::insert(
            source,
            syn::spanned::Spanned::span(fast).start(),
            "artifact_v1: Option<&str>,\n    ".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::insert(
            source,
            syn::spanned::Spanned::span(unused_check).end(),
            concat!(
                "\n\n    if let Some(invocation_id) = artifact_v1 {\n",
                "        if config.buck.split {\n",
                "            anyhow::bail!(\"typed artifact v1 does not admit split BUCK output\");\n",
                "        }\n",
                "        let artifact = crate::artifact::ReindeerGeneratedArtifactV1::from_rules(\n",
                "            &config.buck,\n",
                "            invocation_id,\n",
                "            &rules,\n",
                "        )?;\n",
                "        artifact.write_transport(io::stdout().lock())?;\n",
                "        return Ok(());\n",
                "    }",
            )
            .to_owned(),
        )?,
    ])
}

fn exactly_one_typed_argument_v1<'a>(
    arguments: &'a syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    name: &str,
) -> Result<&'a syn::PatType, ReindeerProviderAdaptationErrorV1> {
    let mut matches = arguments.iter().filter_map(|argument| {
        let syn::FnArg::Typed(argument) = argument else {
            return None;
        };
        let syn::Pat::Ident(binding) = argument.pat.as_ref() else {
            return None;
        };
        (binding.ident == name).then_some(argument)
    });
    let argument = matches
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(argument)
}

fn exactly_one_unused_check_v1(
    statements: &[syn::Stmt],
) -> Result<&syn::Stmt, ReindeerProviderAdaptationErrorV1> {
    let mut matches = statements.iter().filter(|statement| {
        let syn::Stmt::Expr(syn::Expr::Try(expression), Some(_)) = statement else {
            return false;
        };
        let syn::Expr::MethodCall(call) = expression.expr.as_ref() else {
            return false;
        };
        call.method == "check"
            && matches!(
                call.receiver.as_ref(),
                syn::Expr::Path(path) if path.path.is_ident("unused")
            )
    });
    let statement = matches
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(statement)
}

fn outer_type_name_v1(value: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = value else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn nested_type_named_v1<'a>(
    value: &'a syn::Type,
    name: &'static str,
) -> Result<&'a syn::Type, ReindeerProviderAdaptationErrorV1> {
    struct TypeCollectorV1<'ast> {
        name: &'static str,
        matches: Vec<&'ast syn::Type>,
    }

    impl<'ast> syn::visit::Visit<'ast> for TypeCollectorV1<'ast> {
        fn visit_type(&mut self, value: &'ast syn::Type) {
            if outer_type_name_v1(value).as_deref() == Some(self.name) {
                self.matches.push(value);
            }
            syn::visit::visit_type(self, value);
        }
    }

    let mut collector = TypeCollectorV1 {
        name,
        matches: Vec::new(),
    };
    syn::visit::Visit::visit_type(&mut collector, value);
    exactly_one_iterator_v1(collector.matches.into_iter())
}

fn expression_is_ok_rules_v1(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(callee) = call.func.as_ref() else {
        return false;
    };
    callee.path.is_ident("Ok")
        && call.args.len() == 1
        && matches!(&call.args[0], syn::Expr::Path(path) if path.path.is_ident("rules"))
}
