struct ReindeerSerializerSubstitutionV1 {
    substitutions: usize,
}

impl syn::visit_mut::VisitMut for ReindeerSerializerSubstitutionV1 {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        let is_serializer = matches!(
            expression,
            syn::Expr::Path(path)
                if path.path.leading_colon.is_none()
                    && path.path.segments.len() == 1
                    && path.path.is_ident("Serializer")
        );
        if is_serializer {
            *expression = syn::parse_quote!(serializer);
            self.substitutions += 1;
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

fn adapt_reindeer_buck_v1(
    source: &str,
    syntax: &syn::File,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let rule_impl = exactly_one_inherent_rule_impl_v1(&syntax.items)?;
    let renderer = rule_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) if method.sig.ident == "render" => Some(method),
            _ => None,
        })
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    let mut renderer_match = renderer_match_expression_v1(renderer)?;
    let mut substitution = ReindeerSerializerSubstitutionV1 { substitutions: 0 };
    syn::visit_mut::VisitMut::visit_expr_match_mut(&mut substitution, &mut renderer_match);
    if substitution.substitutions != 12 {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let tests = exactly_one_provider_module_v1(&syntax.items, "tests")?;

    let replacement = render_provider_tokens_text_v1(quote::quote! {
        #[derive(Debug)]
        pub(crate) struct DuplicateRuleSortKeyV1;

        impl fmt::Display for DuplicateRuleSortKeyV1 {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("duplicate rule sort key")
            }
        }

        impl std::error::Error for DuplicateRuleSortKeyV1 {}

        pub(crate) fn sort_rules_for_artifact(
            mut rules: Vec<Rule>,
        ) -> Result<Vec<Rule>, DuplicateRuleSortKeyV1> {
            rules.sort();
            if rules.windows(2).any(|pair| pair[0].cmp(&pair[1]).is_eq()) {
                return Err(DuplicateRuleSortKeyV1);
            }
            Ok(rules)
        }

        impl Rule {
            pub(crate) fn serialize_with<S>(
                &self,
                config: &BuckConfig,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                #renderer_match
            }

            pub fn render(
                &self,
                config: &BuckConfig,
                out: &mut impl Write,
            ) -> anyhow::Result<()> {
                let serialized =
                    self.serialize_with(config, serde_starlark::Serializer)?;
                out.write_all(serialized.as_bytes())?;
                Ok(())
            }
        }
    })?;
    let collision_test = render_provider_tokens_text_v1(quote::quote! {
        #[test]
        fn artifact_duplicate_rule_sort_keys_refuse_before_graph_collection() {
            let owner = aws_lc_sys_owner();
            let left = http_archive(owner.clone(), "same");
            let right = http_archive(owner, "same");

            let error = super::sort_rules_for_artifact(vec![left, right]).unwrap_err();
            assert!(error.to_string().contains("duplicate rule sort key"));
        }
    })?;
    let (tests_start, tests_end) =
        source_span_range_v1(source, syn::spanned::Spanned::span(tests))?;
    let closing_brace = tests_end
        .checked_sub(1)
        .filter(|position| *position >= tests_start && source.as_bytes()[*position] == b'}')
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;

    apply_source_edits_v1(
        source,
        vec![
            ReindeerProviderSourceEditV1::replace(
                source,
                syn::spanned::Spanned::span(rule_impl),
                replacement,
            )?,
            ReindeerProviderSourceEditV1 {
                start: closing_brace,
                end: closing_brace,
                replacement: format!("\n{}\n", indent_provider_source_v1(&collision_test)),
            },
        ],
    )
}

fn indent_provider_source_v1(source: &str) -> String {
    source.lines().map(|line| format!("    {line}\n")).collect()
}

fn exactly_one_inherent_rule_impl_v1(
    items: &[syn::Item],
) -> Result<&syn::ItemImpl, ReindeerProviderAdaptationErrorV1> {
    let mut matches = items.iter().filter_map(|item| match item {
        syn::Item::Impl(item)
            if item.trait_.is_none() && self_type_name(item).as_deref() == Some("Rule") =>
        {
            Some(item)
        }
        _ => None,
    });
    let item = matches
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(item)
}

fn renderer_match_expression_v1(
    renderer: &syn::ImplItemFn,
) -> Result<syn::ExprMatch, ReindeerProviderAdaptationErrorV1> {
    let mut matches = renderer.block.stmts.iter().filter_map(|statement| {
        let syn::Stmt::Local(local) = statement else {
            return None;
        };
        let syn::Pat::Ident(binding) = &local.pat else {
            return None;
        };
        if binding.ident != "serialized" {
            return None;
        }
        let expression = local.init.as_ref()?.expr.as_ref();
        let syn::Expr::Try(expression) = expression else {
            return None;
        };
        let syn::Expr::Match(expression) = expression.expr.as_ref() else {
            return None;
        };
        Some(expression.clone())
    });
    let expression = matches
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(expression)
}

fn render_provider_tokens_text_v1(
    tokens: proc_macro2::TokenStream,
) -> Result<String, ReindeerProviderAdaptationErrorV1> {
    String::from_utf8(render_provider_module_v1(tokens)?)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::GeneratedSourceInvalid)
}
