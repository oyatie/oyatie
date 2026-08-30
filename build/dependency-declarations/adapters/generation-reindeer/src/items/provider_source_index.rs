fn adapt_reindeer_index_v1(
    source: &str,
    syntax: &syn::File,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let index = exactly_one_index_struct_v1(&syntax.items)?;
    let implementation = exactly_one_index_impl_v1(&syntax.items)?;
    let constructor = exactly_one_index_method_v1(implementation, "new")?;
    let public_rule_name = exactly_one_index_method_v1(implementation, "public_rule_name")?;
    let initializer = exactly_one_index_initializer_v1(constructor)?;
    let public_targets = exactly_one_public_targets_collection_v1(constructor)?;
    let public_package_loop = exactly_one_public_package_loop_v1(constructor)?;
    let syn::Fields::Named(index_fields) = &index.fields else {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    };

    let mut edits = vec![
        ReindeerProviderSourceEditV1::insert(
            source,
            index_fields.brace_token.span.close().start(),
            "    public_rule_names: BTreeMap<PackageId, Name>,\n".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::insert(
            source,
            initializer.brace_token.span.close().start(),
            "            public_rule_names: BTreeMap::new(),\n".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::insert(
            source,
            syn::spanned::Spanned::span(index).start(),
            "use crate::version_naming::CollisionInfo;\n\n".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::insert(
            source,
            syn::spanned::Spanned::span(implementation).start(),
            render_public_rule_name_resolver_v1()?,
        )?,
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(public_targets),
            wrap_public_targets_collection_v1(source, public_targets)?,
        )?,
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(public_package_loop),
            render_public_package_resolution_v1(),
        )?,
        ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(public_rule_name),
            render_public_rule_name_methods_v1()?,
        )?,
    ];
    edits.push(ReindeerProviderSourceEditV1 {
        start: source.len(),
        end: source.len(),
        replacement: render_public_rule_name_tests_v1()?,
    });
    apply_source_edits_v1(source, edits)
}

fn exactly_one_index_struct_v1(
    items: &[syn::Item],
) -> Result<&syn::ItemStruct, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(items.iter().filter_map(|item| match item {
        syn::Item::Struct(value) if value.ident == "Index" => Some(value),
        _ => None,
    }))
}

fn exactly_one_index_impl_v1(
    items: &[syn::Item],
) -> Result<&syn::ItemImpl, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(items.iter().filter_map(|item| match item {
        syn::Item::Impl(value)
            if value.trait_.is_none()
                && outer_type_name_v1(value.self_ty.as_ref()).as_deref() == Some("Index") =>
        {
            Some(value)
        }
        _ => None,
    }))
}

fn exactly_one_index_method_v1<'a>(
    implementation: &'a syn::ItemImpl,
    name: &str,
) -> Result<&'a syn::ImplItemFn, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(implementation.items.iter().filter_map(|item| match item {
        syn::ImplItem::Fn(value) if value.sig.ident == name => Some(value),
        _ => None,
    }))
}

fn exactly_one_index_initializer_v1(
    constructor: &syn::ImplItemFn,
) -> Result<&syn::ExprStruct, ReindeerProviderAdaptationErrorV1> {
    let initializers = constructor.block.stmts.iter().filter_map(|statement| {
        let syn::Stmt::Local(local) = statement else {
            return None;
        };
        let syn::Pat::Ident(binding) = &local.pat else {
            return None;
        };
        if binding.ident != "index" {
            return None;
        }
        match local.init.as_ref()?.expr.as_ref() {
            syn::Expr::Struct(value) if value.path.is_ident("Index") => Some(value),
            _ => None,
        }
    });
    exactly_one_iterator_v1(initializers)
}

fn exactly_one_public_targets_collection_v1(
    constructor: &syn::ImplItemFn,
) -> Result<&syn::ExprMethodCall, ReindeerProviderAdaptationErrorV1> {
    let collections = constructor.block.stmts.iter().filter_map(|statement| {
        let syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_)) = statement else {
            return None;
        };
        let syn::Expr::Field(field) = assignment.left.as_ref() else {
            return None;
        };
        let syn::Expr::Path(base) = field.base.as_ref() else {
            return None;
        };
        if !base.path.is_ident("index")
            || !matches!(&field.member, syn::Member::Named(name) if name == "public_targets")
        {
            return None;
        }
        match assignment.right.as_ref() {
            syn::Expr::MethodCall(call) if call.method == "collect" && call.args.is_empty() => {
                Some(call)
            }
            _ => None,
        }
    });
    exactly_one_iterator_v1(collections)
}

fn wrap_public_targets_collection_v1(
    source: &str,
    collection: &syn::ExprMethodCall,
) -> Result<String, ReindeerProviderAdaptationErrorV1> {
    let (start, end) = source_span_range_v1(
        source,
        syn::spanned::Spanned::span(collection.receiver.as_ref()),
    )?;
    Ok(format!(
        "collect_public_targets({})?",
        &source[start..end]
    ))
}

fn exactly_one_public_package_loop_v1(
    constructor: &syn::ImplItemFn,
) -> Result<&syn::ExprForLoop, ReindeerProviderAdaptationErrorV1> {
    let loops = constructor.block.stmts.iter().filter_map(|statement| {
        let syn::Stmt::Expr(syn::Expr::ForLoop(value), None) = statement else {
            return None;
        };
        value
            .body
            .stmts
            .iter()
            .any(statement_inserts_public_package_v1)
            .then_some(value)
    });
    exactly_one_iterator_v1(loops)
}

fn statement_inserts_public_package_v1(statement: &syn::Stmt) -> bool {
    let syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) = statement else {
        return false;
    };
    let syn::Expr::Field(field) = call.receiver.as_ref() else {
        return false;
    };
    let syn::Expr::Path(base) = field.base.as_ref() else {
        return false;
    };
    call.method == "insert"
        && base.path.is_ident("index")
        && matches!(&field.member, syn::Member::Named(name) if name == "public_packages")
}
