fn workspace_dependency_roots_edit_v1(
    source: &str,
    function: &syn::ItemFn,
) -> Result<ReindeerProviderSourceEditV1, ReindeerProviderAdaptationErrorV1> {
    struct LoopCollectorV1<'ast> {
        matches: Vec<&'ast syn::ExprForLoop>,
    }

    impl<'ast> syn::visit::Visit<'ast> for LoopCollectorV1<'ast> {
        fn visit_expr_for_loop(&mut self, value: &'ast syn::ExprForLoop) {
            if workspace_member_loop_v1(value) {
                self.matches.push(value);
            }
            syn::visit::visit_expr_for_loop(self, value);
        }
    }

    let mut collector = LoopCollectorV1 {
        matches: Vec::new(),
    };
    syn::visit::Visit::visit_block(&mut collector, &function.block);
    let workspace_loop = exactly_one_iterator_v1(collector.matches.into_iter())?;
    let replacement = render_workspace_dependency_roots_v1(workspace_loop)?;
    let span = syn::spanned::Spanned::span(workspace_loop);
    let (start, end) = source_span_range_v1(source, span)?;
    let line_start = start
        .checked_sub(span.start().column)
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    let indentation = &source[line_start..start];
    if !indentation.bytes().all(|byte| byte == b' ') {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let replacement = replacement
        .lines()
        .map(|line| format!("{indentation}{line}\n"))
        .collect::<String>();
    Ok(ReindeerProviderSourceEditV1 {
        start: line_start,
        end,
        replacement: replacement
            .strip_suffix('\n')
            .ok_or(ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)?
            .to_owned(),
    })
}

fn render_workspace_dependency_roots_v1(
    workspace_loop: &syn::ExprForLoop,
) -> Result<String, ReindeerProviderAdaptationErrorV1> {
    let wrapper = render_provider_tokens_text_v1(quote::quote! {
        fn workspace_dependency_roots_v1() {
            if context.config.include_workspace_members || context.config.include_top_level {
                #workspace_loop
            } else {
                generate_dep_rules(
                    context,
                    scope,
                    tx.clone(),
                    context.index.non_workspace_public_targets(),
                );
            }
        }
    })?;
    let body = wrapper
        .strip_prefix("fn workspace_dependency_roots_v1() {\n")
        .and_then(|value| value.strip_suffix("}\n"))
        .ok_or(ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)?;
    body.lines()
        .map(|line| {
            line.strip_prefix("    ")
                .ok_or(ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|lines| lines.join("\n"))
}

fn workspace_member_loop_v1(value: &syn::ExprForLoop) -> bool {
    let syn::Pat::Reference(pattern) = value.pat.as_ref() else {
        return false;
    };
    let syn::Pat::Ident(binding) = pattern.pat.as_ref() else {
        return false;
    };
    let syn::Expr::Reference(iteration) = value.expr.as_ref() else {
        return false;
    };
    let syn::Expr::Field(workspace_members) = iteration.expr.as_ref() else {
        return false;
    };
    binding.ident == "workspace_member"
        && matches!(
            &workspace_members.member,
            syn::Member::Named(name) if name == "workspace_members"
        )
}
