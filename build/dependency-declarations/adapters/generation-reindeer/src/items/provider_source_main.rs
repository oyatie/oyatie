struct ReindeerMainSeamVisitorV1<'ast> {
    buckify_patterns: Vec<&'ast syn::PatStruct>,
    buckify_calls: Vec<&'ast syn::ExprCall>,
}

impl<'ast> syn::visit::Visit<'ast> for ReindeerMainSeamVisitorV1<'ast> {
    fn visit_pat_struct(&mut self, pattern: &'ast syn::PatStruct) {
        if pattern.path.segments.len() == 2
            && pattern.path.segments[0].ident == "SubCommand"
            && pattern.path.segments[1].ident == "Buckify"
        {
            self.buckify_patterns.push(pattern);
        }
        syn::visit::visit_pat_struct(self, pattern);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let is_buckify = matches!(
            call.func.as_ref(),
            syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "buckify"
                    && path.path.segments[1].ident == "buckify"
        );
        if is_buckify {
            self.buckify_calls.push(call);
        }
        syn::visit::visit_expr_call(self, call);
    }
}

fn adapt_reindeer_main_v1(
    source: &str,
    syntax: &syn::File,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let buck_module = exactly_one_provider_module_v1(&syntax.items, "buck")?;
    let command = exactly_one_provider_enum_v1(&syntax.items, "SubCommand")?;
    let buckify_variant = exactly_one_provider_variant_v1(command, "Buckify")?;
    let syn::Fields::Named(fields) = &buckify_variant.fields else {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    };
    let fast_field = exactly_one_provider_field_v1(&fields.named, "fast")?;
    if exactly_one_provider_field_v1(&fields.named, "artifact_v1").is_ok() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }

    let try_main = exactly_one_provider_function_v1(&syntax.items, "try_main")?;
    let mut visitor = ReindeerMainSeamVisitorV1 {
        buckify_patterns: Vec::new(),
        buckify_calls: Vec::new(),
    };
    syn::visit::Visit::visit_item_fn(&mut visitor, try_main);
    let pattern = exactly_one_collected_v1(&visitor.buckify_patterns)?;
    let call = exactly_one_collected_v1(&visitor.buckify_calls)?;
    let pattern_fast = exactly_one_pattern_field_v1(&pattern.fields, "fast")?;
    let call_fast = call
        .args
        .iter()
        .find(|argument| {
            matches!(
                argument,
                syn::Expr::Unary(unary)
                    if matches!(unary.op, syn::UnOp::Deref(_))
                        && matches!(
                            unary.expr.as_ref(),
                            syn::Expr::Path(path) if path.path.is_ident("fast")
                        )
            )
        })
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;

    apply_source_edits_v1(
        source,
        vec![
            ReindeerProviderSourceEditV1::insert(
                source,
                syn::spanned::Spanned::span(buck_module).start(),
                "mod artifact;\n".to_owned(),
            )?,
            ReindeerProviderSourceEditV1::insert(
                source,
                syn::spanned::Spanned::span(fast_field).start(),
                concat!(
                    "/// Emit one producer-owned typed artifact for qualification.\n",
                    "        ///\n",
                    "        /// This hidden retirement-marked transport is not a user API.\n",
                    "        #[arg(\n",
                    "            long,\n",
                    "            hide = true,\n",
                    "            value_name = \"INVOCATION_ID\",\n",
                    "            conflicts_with = \"stdout\"\n",
                    "        )]\n",
                    "        artifact_v1: Option<String>,\n",
                    "        ",
                )
                .to_owned(),
            )?,
            ReindeerProviderSourceEditV1::insert(
                source,
                syn::spanned::Spanned::span(pattern_fast).start(),
                "artifact_v1,\n            ".to_owned(),
            )?,
            ReindeerProviderSourceEditV1::insert(
                source,
                syn::spanned::Spanned::span(call_fast).start(),
                "artifact_v1.as_deref(),\n                ".to_owned(),
            )?,
        ],
    )
}

fn exactly_one_provider_module_v1<'a>(
    items: &'a [syn::Item],
    name: &str,
) -> Result<&'a syn::ItemMod, ReindeerProviderAdaptationErrorV1> {
    let matches = items.iter().filter_map(|item| match item {
        syn::Item::Mod(module) if module.ident == name => Some(module),
        _ => None,
    });
    exactly_one_iterator_v1(matches)
}

fn exactly_one_provider_enum_v1<'a>(
    items: &'a [syn::Item],
    name: &str,
) -> Result<&'a syn::ItemEnum, ReindeerProviderAdaptationErrorV1> {
    let matches = items.iter().filter_map(|item| match item {
        syn::Item::Enum(value) if value.ident == name => Some(value),
        _ => None,
    });
    exactly_one_iterator_v1(matches)
}

fn exactly_one_provider_variant_v1<'a>(
    value: &'a syn::ItemEnum,
    name: &str,
) -> Result<&'a syn::Variant, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(
        value
            .variants
            .iter()
            .filter(|variant| variant.ident == name),
    )
}

fn exactly_one_provider_field_v1<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    name: &str,
) -> Result<&'a syn::Field, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(
        fields
            .iter()
            .filter(|field| field.ident.as_ref().is_some_and(|ident| ident == name)),
    )
}

fn exactly_one_pattern_field_v1<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::FieldPat, syn::Token![,]>,
    name: &str,
) -> Result<&'a syn::FieldPat, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(fields.iter().filter(|field| match &field.member {
        syn::Member::Named(ident) => ident == name,
        syn::Member::Unnamed(_) => false,
    }))
}

fn exactly_one_collected_v1<'a, T>(
    values: &'a [&'a T],
) -> Result<&'a T, ReindeerProviderAdaptationErrorV1> {
    if let [value] = values {
        Ok(*value)
    } else {
        Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)
    }
}

fn exactly_one_iterator_v1<T>(
    mut values: impl Iterator<Item = T>,
) -> Result<T, ReindeerProviderAdaptationErrorV1> {
    let value = values
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if values.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(value)
}
