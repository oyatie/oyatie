fn adapt_reindeer_version_naming_v1(
    source: &str,
    syntax: &syn::File,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let implementation = exactly_one_collision_info_impl_v1(&syntax.items)?;
    let target_version = exactly_one_collision_info_method_v1(implementation, "target_version")?;
    let tests = exactly_one_provider_module_v1(&syntax.items, "tests")?;
    let tests_insertion = module_closing_brace_v1(source, tests)?;
    let target_version_replacement = render_collision_safe_target_version_v1()?;
    let test_insertion = render_reserved_target_tests_v1()?;

    apply_source_edits_v1(
        source,
        vec![
            ReindeerProviderSourceEditV1::replace(
                source,
                syn::spanned::Spanned::span(target_version),
                target_version_replacement,
            )?,
            ReindeerProviderSourceEditV1 {
                start: tests_insertion,
                end: tests_insertion,
                replacement: format!(
                    "\n{}\n",
                    indent_provider_source_v1(&test_insertion)
                ),
            },
        ],
    )
}

fn exactly_one_collision_info_impl_v1(
    items: &[syn::Item],
) -> Result<&syn::ItemImpl, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(items.iter().filter_map(|item| match item {
        syn::Item::Impl(value)
            if value.trait_.is_none()
                && outer_type_name_v1(value.self_ty.as_ref()).as_deref()
                    == Some("CollisionInfo") =>
        {
            Some(value)
        }
        _ => None,
    }))
}

fn exactly_one_collision_info_method_v1<'a>(
    implementation: &'a syn::ItemImpl,
    name: &str,
) -> Result<&'a syn::ImplItemFn, ReindeerProviderAdaptationErrorV1> {
    exactly_one_iterator_v1(implementation.items.iter().filter_map(|item| match item {
        syn::ImplItem::Fn(value) if value.sig.ident == name => Some(value),
        _ => None,
    }))
}

fn module_closing_brace_v1(
    source: &str,
    module: &syn::ItemMod,
) -> Result<usize, ReindeerProviderAdaptationErrorV1> {
    let (_, items) = module
        .content
        .as_ref()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    let (_, end) = source_span_range_v1(source, syn::spanned::Spanned::span(module))?;
    end.checked_sub(1)
        .filter(|offset| !items.is_empty() && source.as_bytes()[*offset] == b'}')
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)
}

fn render_collision_safe_target_version_v1(
) -> Result<String, ReindeerProviderAdaptationErrorV1> {
    render_provider_tokens_text_v1(quote::quote! {
        /// Returns a collision-safe version component for an internal target.
        pub fn target_version(&self, pkg: &Manifest) -> String {
            let short = short_version(&pkg.version);
            let source_collision = !matches!(pkg.source, Source::CratesIo)
                && self.collisions.contains(&(pkg.name.clone(), short.clone()));
            let candidates = match &pkg.source {
                Source::CratesIo => [
                    short.clone(),
                    pkg.version.to_string(),
                    format!("{}-crates-io", pkg.version),
                ],
                Source::Git { commit_hash, .. } => [
                    short.clone(),
                    format!("{}-{}", short, commit_hash.chars().take(8).collect::<String>()),
                    format!("{}-{}", pkg.version, commit_hash),
                ],
                Source::Local => [
                    short.clone(),
                    format!("{short}-local"),
                    format!("{}-local", pkg.version),
                ],
                Source::Unrecognized(_) => [
                    short.clone(),
                    pkg.version.to_string(),
                    format!("{}-unrecognized", pkg.version),
                ],
            };
            let first_candidate = usize::from(source_collision);
            for candidate in &candidates[first_candidate..] {
                let display = format!("{}-{candidate}", pkg.name);
                if !self.reserved_target_displays.contains(&display) {
                    return candidate.clone();
                }
            }
            candidates[2].clone()
        }
    })
}

fn render_reserved_target_tests_v1() -> Result<String, ReindeerProviderAdaptationErrorV1> {
    render_provider_tokens_text_v1(quote::quote! {
        #[test]
        fn reserved_git_collision_uses_full_source_identity() {
            let registry = make_manifest("fixture", "1.2.0", Source::CratesIo);
            let git = make_manifest(
                "fixture",
                "1.2.1",
                Source::Git {
                    repo: "https://github.com/example/fixture".to_owned(),
                    commit_hash: "abcdef0123456789abcdef0123456789abcdef01".to_owned(),
                },
            );
            let preliminary = CollisionInfo::new(&[&registry, &git]);
            let public_name = preliminary.target_display(&git);
            let final_names = CollisionInfo::new_with_reserved(
                &[&registry, &git],
                [public_name.as_str()],
            );

            assert_eq!(
                final_names.target_version(&git),
                "1.2.1-abcdef0123456789abcdef0123456789abcdef01"
            );
        }

        #[test]
        fn reserved_zero_zero_registry_name_uses_source_suffix() {
            let package = make_manifest("fixture", "0.0.3", Source::CratesIo);
            let names = CollisionInfo::new_with_reserved(&[&package], ["fixture-0.0.3"]);

            assert_eq!(names.target_version(&package), "0.0.3-crates-io");
        }
    })
}
