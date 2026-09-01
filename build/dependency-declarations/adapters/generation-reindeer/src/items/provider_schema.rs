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

#[cfg(test)]
fn inspect_reindeer_provider_schema_v1(
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
        schema_source_sha256: ReindeerProviderDigestV1::of(source),
        semantic_schema_sha256,
        rule_variants: variants.into_boxed_slice(),
    })
}

#[cfg(test)]
mod provider_schema_tests_v1 {
    use std::path::PathBuf;

    use super::{
        PINNED_SOURCE_REVISION, ReindeerProviderSchemaErrorV1, inspect_reindeer_provider_schema_v1,
    };

    fn supported_source(field_type: &str) -> Vec<u8> {
        format!(
            r#"
            pub struct PackageVersion {{ pub name: String, pub version: Version }}
            pub struct Alias {{ pub owner: PackageVersion, pub value: {field_type} }}
            pub struct Sources {{ pub owner: PackageVersion, pub value: String }}
            pub struct Filegroup {{ pub owner: PackageVersion, pub value: String }}
            pub struct ExtractArchive {{ pub owner: PackageVersion, pub value: String }}
            pub struct HttpArchive {{ pub owner: PackageVersion, pub value: String }}
            pub struct GitFetch {{ pub value: String }}
            pub struct RustBinary {{ pub owner: PackageVersion, pub value: String }}
            pub struct RustLibrary {{ pub owner: PackageVersion, pub value: String }}
            pub struct BuildscriptGenrule {{ pub owner: PackageVersion, pub value: String }}
            pub struct CxxLibrary {{ pub owner: PackageVersion, pub value: String }}
            pub struct PrebuiltCxxLibrary {{ pub owner: PackageVersion, pub value: String }}

            pub enum Rule {{
                Alias(Alias),
                Sources(Sources),
                Filegroup(Filegroup),
                ExtractArchive(ExtractArchive),
                HttpArchive(HttpArchive),
                GitFetch(GitFetch),
                Binary(RustBinary),
                Library(RustLibrary),
                BuildscriptBinary(RustBinary),
                BuildscriptGenrule(BuildscriptGenrule),
                CxxLibrary(CxxLibrary),
                PrebuiltCxxLibrary(PrebuiltCxxLibrary),
                RootPackage(RustLibrary),
            }}

            impl Serialize for Alias {{ fn serialize(&self) {{}} }}
            impl Serialize for Sources {{ fn serialize(&self) {{}} }}
            impl Serialize for Filegroup {{ fn serialize(&self) {{}} }}
            impl Serialize for ExtractArchive {{ fn serialize(&self) {{}} }}
            impl Serialize for HttpArchive {{ fn serialize(&self) {{}} }}
            impl Serialize for GitFetch {{ fn serialize(&self) {{}} }}
            impl Serialize for RustBinary {{ fn serialize(&self) {{}} }}
            impl Serialize for RustLibrary {{ fn serialize(&self) {{}} }}
            impl Serialize for BuildscriptGenrule {{ fn serialize(&self) {{}} }}
            impl Serialize for CxxLibrary {{ fn serialize(&self) {{}} }}
            impl Serialize for PrebuiltCxxLibrary {{ fn serialize(&self) {{}} }}

            fn rule_sort_key(rule: &Rule) -> usize {{ match rule {{ _ => 0 }} }}
            impl PartialEq for Rule {{ fn eq(&self, other: &Self) -> bool {{ rule_sort_key(self) == rule_sort_key(other) }} }}
            impl Ord for Rule {{ fn cmp(&self, other: &Self) -> Ordering {{ rule_sort_key(self).cmp(&rule_sort_key(other)) }} }}
            impl Rule {{ pub fn render(&self) {{ match self {{ _ => {{}} }} }} }}
            "#
        )
        .into_bytes()
    }

    #[test]
    fn exact_supported_source_yields_one_closed_schema_receipt() {
        let source = supported_source("String");

        let schema = inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &source)
            .expect("supported schema");

        assert_eq!(schema.parsed_source_files(), 1);
        assert_eq!(schema.rule_variants().len(), 13);
        assert_eq!(schema.rule_variants()[0].name(), "Alias");
        assert_eq!(schema.rule_variants()[0].payload(), "Alias");
        assert_eq!(schema.rule_variants()[0].fields()[1].name(), "value");
        assert_eq!(schema.rule_variants()[0].fields()[1].rust_type(), "String");
        assert_ne!(
            schema.schema_source_sha256(),
            schema.semantic_schema_sha256()
        );
    }

    #[test]
    fn formatting_and_comments_do_not_change_semantic_schema_identity() {
        let source = supported_source("String");
        let mut reformatted = b"// upstream comment\n\n".to_vec();
        reformatted.extend_from_slice(&source);

        let first = inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &source)
            .expect("first schema");
        let second = inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &reformatted)
            .expect("reformatted schema");

        assert_ne!(
            first.schema_source_sha256(),
            second.schema_source_sha256()
        );
        assert_eq!(
            first.semantic_schema_sha256(),
            second.semantic_schema_sha256()
        );
    }

    #[test]
    fn field_type_drift_changes_semantic_schema_identity() {
        let first = inspect_reindeer_provider_schema_v1(
            PINNED_SOURCE_REVISION,
            &supported_source("String"),
        )
        .expect("first schema");
        let second =
            inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &supported_source("u64"))
                .expect("second schema");

        assert_ne!(
            first.semantic_schema_sha256(),
            second.semantic_schema_sha256()
        );
    }

    #[test]
    fn new_rule_variant_refuses_instead_of_becoming_an_open_schema() {
        let source = String::from_utf8(supported_source("String"))
            .expect("UTF-8 fixture")
            .replace(
                "RootPackage(RustLibrary),",
                "RootPackage(RustLibrary),\nUnknown(Alias),",
            );

        assert_eq!(
            inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, source.as_bytes()),
            Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleVariant)
        );
    }

    #[test]
    fn missing_payload_serializer_refuses() {
        let source = String::from_utf8(supported_source("String"))
            .expect("UTF-8 fixture")
            .replace("impl Serialize for Alias { fn serialize(&self) {} }", "");

        assert_eq!(
            inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, source.as_bytes()),
            Err(ReindeerProviderSchemaErrorV1::MissingPayloadSerializer)
        );
    }

    #[test]
    fn wrong_revision_and_oversized_source_refuse_before_claiming_a_schema() {
        assert_eq!(
            inspect_reindeer_provider_schema_v1("different", &supported_source("String")),
            Err(ReindeerProviderSchemaErrorV1::UnsupportedSourceRevision)
        );

        let oversized = vec![b' '; 2 * 1024 * 1024 + 1];
        assert_eq!(
            inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &oversized),
            Err(ReindeerProviderSchemaErrorV1::SourceTooLarge)
        );
    }

    #[test]
    #[ignore = "requires the exact upstream Reindeer source snapshot"]
    fn exact_pinned_upstream_buck_schema_is_supported() {
        let source_path = std::env::var_os("REINDEER_PINNED_BUCK_RS")
            .map(PathBuf::from)
            .expect("REINDEER_PINNED_BUCK_RS must name the exact pinned source");
        let source = std::fs::read(source_path).expect("pinned buck.rs must be readable");

        let schema = inspect_reindeer_provider_schema_v1(PINNED_SOURCE_REVISION, &source)
            .expect("pinned schema");

        assert_eq!(schema.parsed_source_files(), 1);
        assert_eq!(schema.rule_variants().len(), 13);
    }
}
