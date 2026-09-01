fn adapt_reindeer_cxx_fixup_schema_v1(
    source: &str,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let preimage = concat!(
        "    #[serde(default)]\n",
        "    pub preprocessor_flags: Vec<String>,\n",
        "    pub header_namespace: Option<String>,",
    );
    let postimage = concat!(
        "    #[serde(default)]\n",
        "    pub preprocessor_flags: Vec<String>,\n",
        "    #[serde(default)]\n",
        "    pub preprocessor_flags_select: Vec<BTreeMap<String, Vec<String>>>,\n",
        "    pub header_namespace: Option<String>,",
    );
    let edits = exact_source_fragment_edits_v1(source, preimage, postimage, 1)?;
    apply_source_edits_v1(source, edits)
}

fn adapt_reindeer_cxx_fixup_application_v1(
    source: &str,
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let mut edits = exact_source_fragment_edits_v1(
        source,
        concat!(
            "                preprocessor_flags,\n",
            "                header_namespace,",
        ),
        concat!(
            "                preprocessor_flags,\n",
            "                preprocessor_flags_select,\n",
            "                header_namespace,",
        ),
        1,
    )?;
    edits.extend(exact_source_fragment_edits_v1(
        source,
        concat!(
            "                    preprocessor_flags: preprocessor_flags.clone(),\n",
            "                    header_namespace: header_namespace.clone(),",
        ),
        concat!(
            "                    preprocessor_flags: preprocessor_flags.clone(),\n",
            "                    preprocessor_flags_select: preprocessor_flags_select.clone(),\n",
            "                    header_namespace: header_namespace.clone(),",
        ),
        2,
    )?);
    apply_source_edits_v1(source, edits)
}

fn adapt_reindeer_cxx_rule_v1(
    source: &[u8],
) -> Result<Vec<u8>, ReindeerProviderAdaptationErrorV1> {
    let source = std::str::from_utf8(source)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::InvalidUtf8)?;
    let syntax = syn::parse_file(source)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::InvalidRust)?;
    let serializer = exactly_one_trait_impl_v1(&syntax.items, "Serialize", "PreprocessorFlags")?;
    let replacement = render_provider_tokens_text_v1(quote::quote! {
        impl Serialize for PreprocessorFlags<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let Self {
                    include_directories,
                    preprocessor_flags,
                    preprocessor_flags_select,
                } = self;
                let capacity = include_directories
                    .iter()
                    .filter(|directory| directory.is_target())
                    .count()
                    .saturating_add(preprocessor_flags.len());
                let mut common = Vec::with_capacity(capacity);
                for directory in *include_directories {
                    match directory {
                        SubtargetOrPath::Subtarget(subtarget) => common.push(format!(
                            "-I$(location :{})/{}",
                            subtarget.target, subtarget.relative,
                        )),
                        SubtargetOrPath::Path(path) if directory.is_target() => {
                            common.push(format!("-I$(location {})", path.0.to_str().unwrap()));
                        }
                        SubtargetOrPath::Path(_) => {}
                    }
                }
                common.extend(preprocessor_flags.iter().cloned());
                if preprocessor_flags_select.is_empty() {
                    common.serialize(serializer)
                } else {
                    Select {
                        common,
                        selects: preprocessor_flags_select.to_vec(),
                    }
                    .serialize(serializer)
                }
            }
        }
    })?;
    let mut edits = vec![ReindeerProviderSourceEditV1::replace(
        source,
        syn::spanned::Spanned::span(serializer),
        replacement,
    )?];
    for (preimage, postimage) in [
        (
            concat!(
                "    pub preprocessor_flags: Vec<String>,\n",
                "    pub header_namespace: Option<String>,",
            ),
            concat!(
                "    pub preprocessor_flags: Vec<String>,\n",
                "    pub preprocessor_flags_select: Vec<BTreeMap<String, Vec<String>>>,\n",
                "    pub header_namespace: Option<String>,",
            ),
        ),
        (
            concat!(
                "            preprocessor_flags,\n",
                "            header_namespace,",
            ),
            concat!(
                "            preprocessor_flags,\n",
                "            preprocessor_flags_select,\n",
                "            header_namespace,",
            ),
        ),
        (
            concat!(
                "        if !preprocessor_flags.is_empty()\n",
                "            || include_directories.iter().any(SubtargetOrPath::is_target)",
            ),
            concat!(
                "        if !preprocessor_flags.is_empty()\n",
                "            || !preprocessor_flags_select.is_empty()\n",
                "            || include_directories.iter().any(SubtargetOrPath::is_target)",
            ),
        ),
        (
            concat!(
                "                    include_directories,\n",
                "                    preprocessor_flags,",
            ),
            concat!(
                "                    include_directories,\n",
                "                    preprocessor_flags,\n",
                "                    preprocessor_flags_select,",
            ),
        ),
        (
            concat!("    preprocessor_flags: &'a [String],\n", "}"),
            concat!(
                "    preprocessor_flags: &'a [String],\n",
                "    preprocessor_flags_select: &'a [BTreeMap<String, Vec<String>>],\n",
                "}",
            ),
        ),
    ] {
        edits.extend(exact_source_fragment_edits_v1(
            source, preimage, postimage, 1,
        )?);
    }
    apply_source_edits_v1(source, edits)
}

fn exactly_one_trait_impl_v1<'a>(
    items: &'a [syn::Item],
    trait_name: &str,
    self_name: &str,
) -> Result<&'a syn::ItemImpl, ReindeerProviderAdaptationErrorV1> {
    let mut matching = items.iter().filter_map(|item| {
        let syn::Item::Impl(item) = item else {
            return None;
        };
        let (_, trait_path, _) = item.trait_.as_ref()?;
        let trait_matches = trait_path.segments.last()?.ident == trait_name;
        (trait_matches && generic_self_type_name_v1(item).as_deref() == Some(self_name))
            .then_some(item)
    });
    let implementation = matching
        .next()
        .ok_or(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape)?;
    if matching.next().is_some() {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape);
    }
    Ok(implementation)
}

fn generic_self_type_name_v1(item: &syn::ItemImpl) -> Option<String> {
    let syn::Type::Path(path) = item.self_ty.as_ref() else {
        return None;
    };
    (path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1)
        .then(|| path.path.segments[0].ident.to_string())
}
