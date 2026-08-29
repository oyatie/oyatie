use dependency_declarations_generation_reindeer::{
    ReindeerProviderSchemaErrorV1, inspect_reindeer_provider_schema_v1,
};
use std::path::PathBuf;

const PINNED_REVISION: &str = "bb681570d2bc47d1446080c12b8681a50a95f628";

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

    let schema = inspect_reindeer_provider_schema_v1(PINNED_REVISION, &source).unwrap();

    assert_eq!(schema.parsed_source_files(), 1);
    assert_eq!(schema.rule_variants().len(), 13);
    assert_eq!(schema.rule_variants()[0].name(), "Alias");
    assert_eq!(schema.rule_variants()[0].payload(), "Alias");
    assert_eq!(schema.rule_variants()[0].fields()[1].name(), "value");
    assert_eq!(schema.rule_variants()[0].fields()[1].rust_type(), "String");
    assert_ne!(schema.source_sha256(), schema.semantic_schema_sha256());
}

#[test]
fn formatting_and_comments_do_not_change_semantic_schema_identity() {
    let source = supported_source("String");
    let mut reformatted = b"// upstream comment\n\n".to_vec();
    reformatted.extend_from_slice(&source);

    let first = inspect_reindeer_provider_schema_v1(PINNED_REVISION, &source).unwrap();
    let second = inspect_reindeer_provider_schema_v1(PINNED_REVISION, &reformatted).unwrap();

    assert_ne!(first.source_sha256(), second.source_sha256());
    assert_eq!(
        first.semantic_schema_sha256(),
        second.semantic_schema_sha256()
    );
}

#[test]
fn field_type_drift_changes_semantic_schema_identity() {
    let first =
        inspect_reindeer_provider_schema_v1(PINNED_REVISION, &supported_source("String")).unwrap();
    let second =
        inspect_reindeer_provider_schema_v1(PINNED_REVISION, &supported_source("u64")).unwrap();

    assert_ne!(
        first.semantic_schema_sha256(),
        second.semantic_schema_sha256()
    );
}

#[test]
fn new_rule_variant_refuses_instead_of_becoming_an_open_schema() {
    let source = String::from_utf8(supported_source("String"))
        .unwrap()
        .replace(
            "RootPackage(RustLibrary),",
            "RootPackage(RustLibrary),\nUnknown(Alias),",
        );

    assert_eq!(
        inspect_reindeer_provider_schema_v1(PINNED_REVISION, source.as_bytes()),
        Err(ReindeerProviderSchemaErrorV1::UnsupportedRuleVariant)
    );
}

#[test]
fn missing_payload_serializer_refuses() {
    let source = String::from_utf8(supported_source("String"))
        .unwrap()
        .replace("impl Serialize for Alias { fn serialize(&self) {} }", "");

    assert_eq!(
        inspect_reindeer_provider_schema_v1(PINNED_REVISION, source.as_bytes()),
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
        inspect_reindeer_provider_schema_v1(PINNED_REVISION, &oversized),
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

    let schema = inspect_reindeer_provider_schema_v1(PINNED_REVISION, &source).unwrap();

    assert_eq!(schema.parsed_source_files(), 1);
    assert_eq!(schema.rule_variants().len(), 13);
}
