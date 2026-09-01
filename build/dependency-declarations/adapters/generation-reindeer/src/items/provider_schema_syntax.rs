fn exactly_one_rule_enum(items: &[Item]) -> Result<&ItemEnum, ReindeerProviderSchemaErrorV1> {
    let mut matches = items.iter().filter_map(|item| match item {
        Item::Enum(item) if item.ident == "Rule" => Some(item),
        _ => None,
    });
    let item = matches
        .next()
        .ok_or(ReindeerProviderSchemaErrorV1::MissingRuleEnum)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderSchemaErrorV1::DuplicateRuleEnum);
    }
    Ok(item)
}

fn rule_variants(item: &ItemEnum) -> Result<Vec<(String, String)>, ReindeerProviderSchemaErrorV1> {
    item.variants
        .iter()
        .map(|variant| {
            let Fields::Unnamed(fields) = &variant.fields else {
                return Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleShape);
            };
            if fields.unnamed.len() != 1 {
                return Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleShape);
            }
            let Type::Path(payload) = &fields.unnamed[0].ty else {
                return Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleShape);
            };
            let payload = payload
                .path
                .get_ident()
                .ok_or(ReindeerProviderSchemaErrorV1::UnsupportedRuleShape)?;
            Ok((variant.ident.to_string(), payload.to_string()))
        })
        .collect()
}

fn payload_structs<'a>(
    items: &'a [Item],
    names: &BTreeSet<&str>,
) -> Result<BTreeMap<String, &'a ItemStruct>, ReindeerProviderSchemaErrorV1> {
    let mut structs = BTreeMap::new();
    for item in items {
        let Item::Struct(item) = item else { continue };
        let name = item.ident.to_string();
        if names.contains(name.as_str()) && structs.insert(name, item).is_some() {
            return Err(ReindeerProviderSchemaErrorV1::DuplicatePayloadStruct);
        }
    }
    Ok(structs)
}

fn payload_serializers<'a>(
    items: &'a [Item],
    names: &BTreeSet<&str>,
) -> Result<BTreeMap<String, &'a ItemImpl>, ReindeerProviderSchemaErrorV1> {
    let mut serializers = BTreeMap::new();
    for item in items {
        let Item::Impl(item) = item else { continue };
        if trait_name(item).as_deref() != Some("Serialize") {
            continue;
        }
        let Some(name) = self_type_name(item) else {
            continue;
        };
        if names.contains(name.as_str()) && serializers.insert(name, item).is_some() {
            return Err(ReindeerProviderSchemaErrorV1::DuplicatePayloadSerializer);
        }
    }
    Ok(serializers)
}

fn named_fields(
    item: &ItemStruct,
) -> Result<Vec<ReindeerProviderFieldV1>, ReindeerProviderSchemaErrorV1> {
    let Fields::Named(fields) = &item.fields else {
        return Err(ReindeerProviderSchemaErrorV1::UnsupportedPayloadFields);
    };
    fields
        .named
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .ok_or(ReindeerProviderSchemaErrorV1::UnsupportedPayloadFields)?;
            Ok(ReindeerProviderFieldV1 {
                name: name.to_string(),
                rust_type: field.ty.to_token_stream().to_string(),
            })
        })
        .collect()
}

fn exactly_one_function<'a>(
    items: &'a [Item],
    name: &str,
) -> Result<&'a syn::ItemFn, ReindeerProviderSchemaErrorV1> {
    let mut matches = items.iter().filter_map(|item| match item {
        Item::Fn(item) if item.sig.ident == name => Some(item),
        _ => None,
    });
    let item = matches
        .next()
        .ok_or(ReindeerProviderSchemaErrorV1::MissingRuleSortKey)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderSchemaErrorV1::DuplicateRuleSortKey);
    }
    Ok(item)
}

fn exactly_one_trait_impl<'a>(
    items: &'a [Item],
    trait_name_expected: &str,
    self_name: &str,
) -> Result<&'a ItemImpl, ReindeerProviderSchemaErrorV1> {
    let mut matches = items.iter().filter_map(|item| match item {
        Item::Impl(item)
            if trait_name(item).as_deref() == Some(trait_name_expected)
                && self_type_name(item).as_deref() == Some(self_name) =>
        {
            Some(item)
        }
        _ => None,
    });
    let missing = if trait_name_expected == "PartialEq" {
        ReindeerProviderSchemaErrorV1::MissingRulePartialEq
    } else {
        ReindeerProviderSchemaErrorV1::MissingRuleOrd
    };
    let duplicate = if trait_name_expected == "PartialEq" {
        ReindeerProviderSchemaErrorV1::DuplicateRulePartialEq
    } else {
        ReindeerProviderSchemaErrorV1::DuplicateRuleOrd
    };
    let item = matches.next().ok_or(missing)?;
    if matches.next().is_some() {
        return Err(duplicate);
    }
    Ok(item)
}

fn exactly_one_method<'a>(
    items: &'a [Item],
    self_name: &str,
    method_name: &str,
) -> Result<&'a syn::ImplItemFn, ReindeerProviderSchemaErrorV1> {
    let mut matches = items.iter().filter_map(|item| {
        let Item::Impl(item) = item else { return None };
        if item.trait_.is_some() || self_type_name(item).as_deref() != Some(self_name) {
            return None;
        }
        item.items.iter().find_map(|item| match item {
            syn::ImplItem::Fn(method) if method.sig.ident == method_name => Some(method),
            _ => None,
        })
    });
    let item = matches
        .next()
        .ok_or(ReindeerProviderSchemaErrorV1::MissingRuleRenderer)?;
    if matches.next().is_some() {
        return Err(ReindeerProviderSchemaErrorV1::DuplicateRuleRenderer);
    }
    Ok(item)
}

fn trait_name(item: &ItemImpl) -> Option<String> {
    item.trait_
        .as_ref()?
        .1
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn self_type_name(item: &ItemImpl) -> Option<String> {
    let Type::Path(path) = item.self_ty.as_ref() else {
        return None;
    };
    path.path.get_ident().map(|ident| ident.to_string())
}

fn token_digest(value: &impl ToTokens) -> ReindeerProviderDigestV1 {
    ReindeerProviderDigestV1::of(value.to_token_stream().to_string().as_bytes())
}

fn semantic_digest(
    variants: &[ReindeerProviderRuleVariantV1],
    sort_key: &impl ToTokens,
    partial_eq: &impl ToTokens,
    ord: &impl ToTokens,
    renderer: &impl ToTokens,
) -> Result<ReindeerProviderDigestV1, ReindeerProviderSchemaErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"build.reindeer-provider-schema.v1\0");
    for variant in variants {
        hash_string(&mut hash, &variant.name)?;
        hash_string(&mut hash, &variant.payload)?;
        hash.update(variant.serializer_sha256.0);
        let field_count = u64::try_from(variant.fields.len())
            .map_err(|_| ReindeerProviderSchemaErrorV1::SourceTooLarge)?;
        hash.update(field_count.to_be_bytes());
        for field in &variant.fields {
            hash_string(&mut hash, &field.name)?;
            hash_string(&mut hash, &field.rust_type)?;
        }
    }
    for item in [
        token_digest(sort_key),
        token_digest(partial_eq),
        token_digest(ord),
        token_digest(renderer),
    ] {
        hash.update(item.0);
    }
    Ok(ReindeerProviderDigestV1(hash.finalize().into()))
}

fn hash_string(hash: &mut Sha256, value: &str) -> Result<(), ReindeerProviderSchemaErrorV1> {
    let length =
        u64::try_from(value.len()).map_err(|_| ReindeerProviderSchemaErrorV1::SourceTooLarge)?;
    hash.update(length.to_be_bytes());
    hash.update(value.as_bytes());
    Ok(())
}
