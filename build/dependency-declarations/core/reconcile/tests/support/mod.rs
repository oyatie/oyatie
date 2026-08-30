use dependency_declarations_reconcile::*;

mod effects;
mod provider;
mod request_helpers;

pub use effects::{FixedProjection, RecordingPublisher, ScriptedGenerator};
pub use provider::{
    ProviderArtifactFaultV1, raw_provider_artifact, raw_provider_artifact_with_fault,
};
use request_helpers::{
    artifact, buck_consumer_profile, entry_for_file, platform, tool, tree, tree_entry,
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ProjectionProfileVariation {
    Baseline,
    Renderer,
    Parser,
    Grammar,
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
        ProjectionProfileVariation::Baseline,
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
        ProjectionProfileVariation::Baseline,
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
        ProjectionProfileVariation::Baseline,
        BuckConsumerVariation::Baseline,
    )
}

pub fn generation_request_with_projection_variation(
    variation: ProjectionProfileVariation,
) -> GenerationRequestV1 {
    generation_request(
        false,
        b"[workspace]\n",
        "oyatie.reindeer.source-adaptation.v1",
        b"provider source",
        b"graph schema",
        variation,
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
        ProjectionProfileVariation::Baseline,
        variation,
    )
}

fn generation_request(
    platform_order: bool,
    manifest_bytes: &[u8],
    provider_recipe: &str,
    provider_source: &[u8],
    provider_schema: &[u8],
    projection_variation: ProjectionProfileVariation,
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
    let fixup = tree_entry("third-party/fixups/crate/fixups.toml", b"fixup\n");
    let repository_reads = InputTreeV1::try_from_entries(
        TreeRoleV1::RepositoryRead,
        CanonicalPathV1::try_new("snapshots/repository-reads.manifest").unwrap(),
        vec![
            entry_for_file(&manifest),
            entry_for_file(&lock),
            entry_for_file(&config),
            fixup.clone(),
        ],
    )
    .unwrap();
    let fixups = InputTreeV1::try_from_entries(
        TreeRoleV1::Fixups,
        CanonicalPathV1::try_new("snapshots/fixups.manifest").unwrap(),
        vec![fixup],
    )
    .unwrap();
    let cargo_home_reads = tree(
        TreeRoleV1::CargoHomeRead,
        "snapshots/cargo-home-reads.manifest",
        "registry/src/crate/src/lib.rs",
    );
    let inputs = GenerationInputsV1::try_new(
        manifest,
        lock,
        config,
        repository_reads,
        fixups,
        cargo_home_reads,
    )
    .unwrap();

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
    let version = |field| {
        if projection_variation == field {
            "changed"
        } else {
            "baseline"
        }
    };
    let qualification = GenerationQualificationV1::new(
        artifact(
            "serde_starlark",
            version(ProjectionProfileVariation::Renderer),
        ),
        artifact(
            "starlark_syntax",
            version(ProjectionProfileVariation::Parser),
        ),
        ProviderGraphProfileV1::try_new(
            provider_recipe,
            digest(provider_source),
            digest(provider_schema),
        )
        .unwrap(),
        digest(
            if projection_variation == ProjectionProfileVariation::Grammar {
                b"changed grammar"
            } else {
                b"grammar"
            },
        ),
        buck_consumer_profile(buck_consumer_variation),
    );
    let tools = GenerationToolsV1::new(
        generator,
        tool("cargo", "1.98.0"),
        tool("rustc", "1.98.0"),
        artifact("generation-runtime", "v1"),
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
