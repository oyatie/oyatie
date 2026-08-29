use dependency_declarations_reconcile::*;

mod effects;
mod provider;

pub use effects::{FixedProjection, RecordingPublisher, ScriptedGenerator};
pub use provider::{
    ProviderArtifactFaultV1, raw_provider_artifact, raw_provider_artifact_with_fault,
};

pub fn digest(bytes: &[u8]) -> DigestV1 {
    DigestV1::of(bytes)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BuckConsumerVariation {
    Baseline,
    Buck2,
    Prelude,
    Rules,
    Toolchain,
    CellConfig,
    BuckConfig,
    QualificationReceipt,
}

pub fn graph(target: &str) -> RuleGraphV1 {
    let fragment = format!("rust_library(name = \"{target}\")");
    graph_with_fragment(target, fragment.as_bytes())
}

pub fn graph_with_fragment(target: &str, fragment: &[u8]) -> RuleGraphV1 {
    let semantic = SemanticValueV1::call_named(
        "rust_library",
        vec![
            ("name".to_owned(), SemanticValueV1::string(target).unwrap()),
            (
                "deps".to_owned(),
                SemanticValueV1::list(vec![SemanticValueV1::string(":dep").unwrap()]).unwrap(),
            ),
        ],
    )
    .unwrap();
    let rule = RuleV1::new(0, ReindeerRuleKindV1::Library, semantic, digest(fragment));
    RuleGraphV1::try_new(b"# generated\n".to_vec(), vec![rule]).unwrap()
}

pub fn rendered(target: &str) -> Vec<u8> {
    rendered_fragment(format!("rust_library(name = \"{target}\")").as_bytes())
}

pub fn rendered_fragment(fragment: &[u8]) -> Vec<u8> {
    let mut bytes = b"# generated\n".to_vec();
    bytes.extend_from_slice(fragment);
    bytes
}

pub fn valid_generation_request(platform_order: bool) -> GenerationRequestV1 {
    generation_request(
        platform_order,
        b"[workspace]\n",
        "oyatie.reindeer.source-adaptation.v1",
        b"provider source",
        b"graph schema",
        BuckConsumerVariation::Baseline,
    )
}

pub fn generation_request_with_manifest(manifest: &[u8]) -> GenerationRequestV1 {
    generation_request(
        false,
        manifest,
        "oyatie.reindeer.source-adaptation.v1",
        b"provider source",
        b"graph schema",
        BuckConsumerVariation::Baseline,
    )
}

pub fn generation_request_with_provider_profile(
    recipe: &str,
    source: &[u8],
    schema: &[u8],
) -> GenerationRequestV1 {
    generation_request(
        false,
        b"[workspace]\n",
        recipe,
        source,
        schema,
        BuckConsumerVariation::Baseline,
    )
}

pub fn generation_request_with_buck_consumer_variation(
    variation: BuckConsumerVariation,
) -> GenerationRequestV1 {
    generation_request(
        false,
        b"[workspace]\n",
        "oyatie.reindeer.source-adaptation.v1",
        b"provider source",
        b"graph schema",
        variation,
    )
}

fn generation_request(
    platform_order: bool,
    manifest_bytes: &[u8],
    provider_recipe: &str,
    provider_source: &[u8],
    provider_schema: &[u8],
    buck_consumer_variation: BuckConsumerVariation,
) -> GenerationRequestV1 {
    let manifest = InputFileV1::try_new(
        InputFileRoleV1::Manifest,
        CanonicalPathV1::try_new("Cargo.toml").unwrap(),
        manifest_bytes.to_vec(),
    )
    .unwrap();
    let lock = InputFileV1::try_new(
        InputFileRoleV1::Lock,
        CanonicalPathV1::try_new("Cargo.lock").unwrap(),
        b"version = 4\n".to_vec(),
    )
    .unwrap();
    let config = InputFileV1::try_new(
        InputFileRoleV1::Config,
        CanonicalPathV1::try_new("reindeer.toml").unwrap(),
        b"[buck]\n".to_vec(),
    )
    .unwrap();
    let fixups = tree(
        TreeRoleV1::Fixups,
        "snapshots/fixups.manifest",
        "crate/fixups.toml",
    );
    let sources = tree(
        TreeRoleV1::CargoSource,
        "snapshots/sources.manifest",
        "registry/crate/src/lib.rs",
    );
    let inputs = GenerationInputsV1::try_new(manifest, lock, config, fixups, sources).unwrap();

    let generator = GeneratorIdentityV1::try_new(
        "reindeer",
        "2026.08.10.00",
        "bb681570d2bc47d1446080c12b8681a50a95f628",
        digest(b"generator source tree"),
        digest(b"generator binary"),
        GeneratorBinaryV1::ReproducibleBuild {
            receipt_sha256: digest(b"build receipt"),
        },
    )
    .unwrap();
    let qualification = GenerationQualificationV1::new(
        artifact("serde_starlark", "0.1.19"),
        artifact("starlark_syntax", "0.14.2"),
        ProviderGraphProfileV1::try_new(
            provider_recipe,
            digest(provider_source),
            digest(provider_schema),
        )
        .unwrap(),
        digest(b"grammar"),
        buck_consumer_profile(buck_consumer_variation),
    );
    let tools = GenerationToolsV1::new(
        generator,
        tool("cargo", "1.98.0"),
        tool("rustc", "1.98.0"),
        qualification,
    );

    let mut platforms = vec![
        platform("linux-x86_64", "x86_64-unknown-linux-gnu", true),
        platform("macos-arm64", "aarch64-apple-darwin", true),
    ];
    if platform_order {
        platforms.reverse();
    }
    let execution = GenerationExecutionV1::new(
        PlatformSetV1::try_new(platforms).unwrap(),
        EnvironmentProfileV1::ReindeerHermeticV1,
        SandboxProfileV1::DeclaredReadStageWriteNoNetworkV1,
        ValidatorProfileV1::ReindeerBuckV1,
        ValidationBoundsV1,
    );
    GenerationRequestV1::try_new(
        RepositoryCorrelationV1::try_new("oyatie", "a355428b").unwrap(),
        inputs,
        tools,
        execution,
    )
    .unwrap()
}

fn tree(role: TreeRoleV1, manifest: &str, entry: &str) -> InputTreeV1 {
    let entry = TreeEntryV1::new(
        CanonicalPathV1::try_new(entry).unwrap(),
        7,
        digest(entry.as_bytes()),
    );
    InputTreeV1::try_from_entries(
        role,
        CanonicalPathV1::try_new(manifest).unwrap(),
        vec![entry],
    )
    .unwrap()
}

fn platform(name: &str, triple: &str, execution: bool) -> PlatformIdentityV1 {
    PlatformIdentityV1::try_new(
        name,
        triple,
        format!("//platform:{name}-select"),
        format!("//platform:{name}"),
        execution,
    )
    .unwrap()
}

fn artifact(name: &str, version: &str) -> ArtifactIdentityV1 {
    ArtifactIdentityV1::try_new(
        name,
        version,
        format!("{name}-revision"),
        digest(format!("{name}-source").as_bytes()),
        digest(format!("{name}-artifact").as_bytes()),
    )
    .unwrap()
}

fn buck_consumer_profile(variation: BuckConsumerVariation) -> BuckConsumerProfileV1 {
    let version = |field| {
        if variation == field { "changed" } else { "v1" }
    };
    let bytes = |field, baseline: &'static [u8]| {
        if variation == field {
            b"changed".as_slice()
        } else {
            baseline
        }
    };
    BuckConsumerProfileV1::new(
        artifact("buck2", version(BuckConsumerVariation::Buck2)),
        artifact("buck2-prelude", version(BuckConsumerVariation::Prelude)),
        digest(bytes(BuckConsumerVariation::Rules, b"owned rules")),
        digest(bytes(BuckConsumerVariation::Toolchain, b"buck toolchain")),
        digest(bytes(BuckConsumerVariation::CellConfig, b"cell config")),
        digest(bytes(BuckConsumerVariation::BuckConfig, b"buck config")),
        digest(bytes(
            BuckConsumerVariation::QualificationReceipt,
            b"consumer qualification receipt",
        )),
    )
}

fn tool(name: &str, version: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        version,
        format!("{name}-commit"),
        "aarch64-apple-darwin",
        digest(format!("{name}-binary").as_bytes()),
    )
    .unwrap()
}
