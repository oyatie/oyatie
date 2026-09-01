fn workspace_dev_dependency_edits_v1(
    source: &str,
    items: &[syn::Item],
) -> Result<Vec<ReindeerProviderSourceEditV1>, ReindeerProviderAdaptationErrorV1> {
    let resolver = exactly_one_iterator_v1(items.iter().filter_map(|item| match item {
        syn::Item::Struct(value) if value.ident == "FeatureResolver" => Some(value),
        _ => None,
    }))?;
    let syn::Fields::Named(fields) = &resolver.fields else {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    };

    let mut visitor = WorkspaceDevDependencyVisitorV1::default();
    for item in items {
        syn::visit::Visit::visit_item(&mut visitor, item);
    }
    let initializer = exactly_one_iterator_v1(visitor.initializers.into_iter())?;
    if visitor.disabled_dev_arms.len() != 3 {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    let ignored_dev_arm = exactly_one_iterator_v1(visitor.ignored_dev_arms.into_iter())?;

    let mut edits = vec![
        ReindeerProviderSourceEditV1::insert(
            source,
            fields.brace_token.span.close().start(),
            "    workspace_packages: &'a HashSet<PackageId>,\n".to_owned(),
        )?,
        ReindeerProviderSourceEditV1::insert(
            source,
            initializer.brace_token.span.close().start(),
            "                workspace_packages: &index.workspace_packages,\n".to_owned(),
        )?,
    ];
    for arm in visitor.disabled_dev_arms {
        edits.push(ReindeerProviderSourceEditV1::replace(
            source,
            syn::spanned::Spanned::span(arm),
            "DepKind::Dev => self.workspace_packages.contains(&pkgid)".to_owned(),
        )?);
    }
    edits.push(ReindeerProviderSourceEditV1::replace(
        source,
        syn::spanned::Spanned::span(ignored_dev_arm),
        "DepKind::Dev => is_target_dep = true,".to_owned(),
    )?);
    Ok(edits)
}

#[derive(Default)]
struct WorkspaceDevDependencyVisitorV1<'ast> {
    initializers: Vec<&'ast syn::ExprStruct>,
    disabled_dev_arms: Vec<&'ast syn::Arm>,
    ignored_dev_arms: Vec<&'ast syn::Arm>,
}

impl<'ast> syn::visit::Visit<'ast> for WorkspaceDevDependencyVisitorV1<'ast> {
    fn visit_expr_struct(&mut self, value: &'ast syn::ExprStruct) {
        if value.path.is_ident("FeatureResolver") {
            self.initializers.push(value);
        }
        syn::visit::visit_expr_struct(self, value);
    }

    fn visit_arm(&mut self, value: &'ast syn::Arm) {
        if dev_pattern_v1(&value.pat) && disabled_body_v1(&value.body) {
            self.disabled_dev_arms.push(value);
        } else if dev_pattern_v1(&value.pat) && empty_body_v1(&value.body) {
            self.ignored_dev_arms.push(value);
        }
        syn::visit::visit_arm(self, value);
    }
}

fn empty_body_v1(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Block(value) if value.block.stmts.is_empty())
}

fn dev_pattern_v1(pattern: &syn::Pat) -> bool {
    let syn::Pat::Path(path) = pattern else {
        return false;
    };
    path.path.segments.len() == 2
        && path.path.segments[0].ident == "DepKind"
        && path.path.segments[1].ident == "Dev"
}

fn disabled_body_v1(expression: &syn::Expr) -> bool {
    matches!(
        expression,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(value),
            ..
        }) if !value.value
    )
}
