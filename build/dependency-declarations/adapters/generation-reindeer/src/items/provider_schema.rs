use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use quote::ToTokens;
use sha2::{Digest as _, Sha256};
use syn::{Fields, Item, ItemEnum, ItemImpl, ItemStruct, Type};

const PINNED_SOURCE_REVISION: &str = "bb681570d2bc47d1446080c12b8681a50a95f628";
const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;
const SUPPORTED_RULES: [(&str, &str); 13] = [
    ("Alias", "Alias"),
    ("Sources", "Sources"),
    ("Filegroup", "Filegroup"),
    ("ExtractArchive", "ExtractArchive"),
    ("HttpArchive", "HttpArchive"),
    ("GitFetch", "GitFetch"),
    ("Binary", "RustBinary"),
    ("Library", "RustLibrary"),
    ("BuildscriptBinary", "RustBinary"),
    ("BuildscriptGenrule", "BuildscriptGenrule"),
    ("CxxLibrary", "CxxLibrary"),
    ("PrebuiltCxxLibrary", "PrebuiltCxxLibrary"),
    ("RootPackage", "RustLibrary"),
];

/// Inspects one bounded upstream `buck.rs` batch under the pinned profile.
pub fn inspect_reindeer_provider_schema_v1(
    source_revision: &str,
    source: &[u8],
) -> Result<ReindeerProviderSchemaV1, ReindeerProviderSchemaErrorV1> {
    if source_revision != PINNED_SOURCE_REVISION {
        return Err(ReindeerProviderSchemaErrorV1::UnsupportedSourceRevision);
    }
    if source.len() > MAX_SOURCE_BYTES {
        return Err(ReindeerProviderSchemaErrorV1::SourceTooLarge);
    }
    let source_text =
        std::str::from_utf8(source).map_err(|_| ReindeerProviderSchemaErrorV1::InvalidUtf8)?;
    let syntax =
        syn::parse_file(source_text).map_err(|_| ReindeerProviderSchemaErrorV1::InvalidRust)?;

    inspect_reindeer_provider_schema_syntax_v1(source, &syntax)
}

fn inspect_reindeer_provider_schema_syntax_v1(
    source: &[u8],
    syntax: &syn::File,
) -> Result<ReindeerProviderSchemaV1, ReindeerProviderSchemaErrorV1> {
    let rule = exactly_one_rule_enum(&syntax.items)?;
    let declared_variants = rule_variants(rule)?;
    if !declared_variants
        .iter()
        .zip(SUPPORTED_RULES)
        .all(|((name, payload), expected)| name == expected.0 && payload == expected.1)
        || declared_variants.len() != SUPPORTED_RULES.len()
    {
        return Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleVariant);
    }

    let payload_names: BTreeSet<&str> = SUPPORTED_RULES
        .iter()
        .map(|(_, payload)| *payload)
        .collect();
    let structs = payload_structs(&syntax.items, &payload_names)?;
    let serializers = payload_serializers(&syntax.items, &payload_names)?;
    let sort_key = exactly_one_function(&syntax.items, "rule_sort_key")?;
    let partial_eq = exactly_one_trait_impl(&syntax.items, "PartialEq", "Rule")?;
    let ord = exactly_one_trait_impl(&syntax.items, "Ord", "Rule")?;
    let renderer = exactly_one_method(&syntax.items, "Rule", "render")?;

    let mut variants = Vec::with_capacity(SUPPORTED_RULES.len());
    for (name, payload) in SUPPORTED_RULES {
        let item = structs
            .get(payload)
            .ok_or(ReindeerProviderSchemaErrorV1::MissingPayloadStruct)?;
        let serializer = serializers
            .get(payload)
            .ok_or(ReindeerProviderSchemaErrorV1::MissingPayloadSerializer)?;
        variants.push(ReindeerProviderRuleVariantV1 {
            name: name.to_owned(),
            payload: payload.to_owned(),
            fields: named_fields(item)?.into_boxed_slice(),
            serializer_sha256: token_digest(*serializer),
        });
    }

    let semantic_schema_sha256 = semantic_digest(&variants, sort_key, partial_eq, ord, renderer)?;
    Ok(ReindeerProviderSchemaV1 {
        source_sha256: ReindeerProviderDigestV1::of(source),
        semantic_schema_sha256,
        rule_variants: variants.into_boxed_slice(),
    })
}
