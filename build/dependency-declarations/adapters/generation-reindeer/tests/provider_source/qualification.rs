use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use dependency_declarations_generation::{DeclarationProviderCapabilityPort, GenerationPort};
use dependency_declarations_generation_reindeer::{
    ReindeerProviderSourceAdaptationV1, StarlarkSyntaxProjectionV1,
};
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::*;

#[path = "qualification/consumer.rs"]
mod consumer;
use consumer::{FixtureBuckConsumer, profile as buck_consumer};

pub(super) fn assert_provider_parser_reconciliation(
    binary: &Path,
    adaptation: &ReindeerProviderSourceAdaptationV1,
    first_root: &Path,
    second_root: &Path,
) {
    let request = generation_request(binary, adaptation, first_root);
    let generator = FixtureGenerator::new(binary, first_root, second_root, request.request_id());
    let projector = StarlarkSyntaxProjectionV1::new(request.projection_profile_sha256());
    let consumer = FixtureBuckConsumer;
    let publisher = CheckOnlyPublisher;

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projector,
        &consumer,
        &publisher,
    );

    let ReconciliationResultV1::Generated { generation } = result else {
        panic!("exact provider and maintained parser must reconcile");
    };
    assert_eq!(generator.artifact_count(), 2);
    assert_ne!(generation.attempts()[0], generation.attempts()[1]);
}

struct FixtureGenerator {
    binary: PathBuf,
    roots: [PathBuf; 2],
    request_id: DigestV1,
    artifacts: Mutex<Vec<Vec<u8>>>,
}

impl FixtureGenerator {
    fn new(binary: &Path, first: &Path, second: &Path, request_id: DigestV1) -> Self {
        Self {
            binary: binary.to_owned(),
            roots: [first.to_owned(), second.to_owned()],
            request_id,
            artifacts: Mutex::new(Vec::new()),
        }
    }

    fn artifact_count(&self) -> usize {
        self.artifacts.lock().unwrap().len()
    }
}

impl<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>
    for FixtureGenerator
{
    fn generate(
        &self,
        invocation: &GenerationInvocationV1<'a>,
    ) -> Result<RawGenerationV1, GenerationPortErrorV1> {
        if invocation.request_id() != self.request_id {
            return Err(GenerationPortErrorV1::InputChanged);
        }
        let root = match invocation.attempt() {
            GenerationAttemptV1::First => &self.roots[0],
            GenerationAttemptV1::Second => &self.roots[1],
        };
        let output = Command::new(&self.binary)
            .arg("--cargo-options=--offline")
            .arg("--cargo-options=--locked")
            .arg("-c")
            .arg(root.join("reindeer.toml"))
            .arg("buckify")
            .arg("--artifact-v1")
            .arg(invocation.invocation_id().to_string())
            .current_dir(root)
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .map_err(|_| GenerationPortErrorV1::GeneratorUnavailable)?;
        if !output.status.success() {
            return Err(GenerationPortErrorV1::GeneratorFailed);
        }
        self.artifacts.lock().unwrap().push(output.stdout.clone());
        let execution = GenerationExecutionObservationV1::completed(
            invocation,
            DigestV1::of(b"qualification observed reads"),
            DigestV1::of(b"qualification observed writes"),
            invocation.invocation_id(),
        );
        Ok(RawGenerationV1::unverified_provider_artifact(
            output.stdout,
            output.stderr,
            execution,
        ))
    }
}

impl DeclarationProviderCapabilityPort<GenerationRequestV1> for FixtureGenerator {
    fn supports(&self, request: &GenerationRequestV1) -> bool {
        request.request_id() == self.request_id
    }
}

struct CheckOnlyPublisher;

impl PublicationCapabilityPort<PublisherProfileV1> for CheckOnlyPublisher {
    fn supports(&self, _profile: &PublisherProfileV1) -> bool {
        false
    }
}

impl PublicationPort<PublicationRequestV1, PublicationObservationV1, PublicationPortErrorV1>
    for CheckOnlyPublisher
{
    fn publish(
        &self,
        _request: &PublicationRequestV1,
    ) -> Result<PublicationObservationV1, PublicationPortErrorV1> {
        Ok(PublicationObservationV1::new(
            PublicationOutcomeV1::Unchanged,
        ))
    }
}

fn generation_request(
    binary: &Path,
    adaptation: &ReindeerProviderSourceAdaptationV1,
    root: &Path,
) -> GenerationRequestV1 {
    let manifest = input_file(root, "third-party/Cargo.toml", InputFileRoleV1::Manifest);
    let lock = input_file(root, "third-party/Cargo.lock", InputFileRoleV1::Lock);
    let config = input_file(root, "reindeer.toml", InputFileRoleV1::Config);
    let repository_reads = input_tree(
        TreeRoleV1::RepositoryRead,
        "qualification/repository-reads.manifest",
        repository_entries(root),
    );
    let fixups = input_tree(
        TreeRoleV1::Fixups,
        "qualification/fixups.manifest",
        Vec::new(),
    );
    let cargo_home_reads = input_tree(
        TreeRoleV1::CargoHomeRead,
        "qualification/cargo-home-reads.manifest",
        Vec::new(),
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
    let binary_sha256 = DigestV1::of(&std::fs::read(binary).unwrap());
    let profile = adaptation.profile();
    let generator = GeneratorIdentityV1::try_new(
        "reindeer",
        profile.source_tag(),
        profile.source_revision(),
        DigestV1::from_bytes(adaptation.source_tree_sha256().bytes()),
        binary_sha256,
        GeneratorBinaryV1::ReproducibleBuild {
            receipt_sha256: DigestV1::from_bytes(adaptation.receipt_sha256().bytes()),
        },
    )
    .unwrap();
    let provider_graph = ProviderGraphProfileV1::try_new(
        profile.recipe_identity(),
        DigestV1::from_bytes(adaptation.schema().schema_source_sha256().bytes()),
        DigestV1::from_bytes(adaptation.schema().semantic_schema_sha256().bytes()),
    )
    .unwrap();
    let qualification = GenerationQualificationV1::new(
        artifact("serde_starlark", "0.1.19"),
        artifact("starlark_syntax", "0.14.2"),
        provider_graph,
        DigestV1::of(b"reindeer rendered projection grammar v1"),
        buck_consumer(),
    );
    let tools = GenerationToolsV1::new(
        generator,
        tool("cargo", "qualification"),
        tool("rustc", "qualification"),
        artifact("qualification-runtime", "v1"),
        qualification,
    );
    let execution = GenerationExecutionV1::new(
        PlatformSetV1::try_new(vec![
            PlatformIdentityV1::try_new(
                "qualification-host",
                "aarch64-apple-darwin",
                "//platform:qualification-select",
                "//platform:qualification",
                true,
            )
            .unwrap(),
        ])
        .unwrap(),
        EnvironmentProfileV1::ReindeerHermeticV1,
        SandboxProfileV1::DeclaredReadStageWriteNoNetworkV1,
        ValidatorProfileV1::ReindeerBuckV1,
        ValidationBoundsV1,
    );
    GenerationRequestV1::try_new(
        RepositoryCorrelationV1::try_new("reindeer-qualification", "fixture-v1").unwrap(),
        inputs,
        tools,
        execution,
    )
    .unwrap()
}

fn input_file(root: &Path, relative: &str, role: InputFileRoleV1) -> InputFileV1 {
    InputFileV1::try_new(
        role,
        CanonicalPathV1::try_new(relative).unwrap(),
        std::fs::read(root.join(relative)).unwrap(),
    )
    .unwrap()
}

fn repository_entries(root: &Path) -> Vec<TreeEntryV1> {
    let mut pending = VecDeque::from([root.to_owned()]);
    let mut entries = Vec::new();
    while let Some(directory) = pending.pop_front() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                pending.push_back(entry.path());
                continue;
            }
            assert!(file_type.is_file());
            let relative = entry.path().strip_prefix(root).unwrap().to_owned();
            let relative = relative
                .to_str()
                .unwrap()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let bytes = std::fs::read(entry.path()).unwrap();
            entries.push(TreeEntryV1::new(
                CanonicalPathV1::try_new(relative).unwrap(),
                TreeFileModeV1::Regular,
                u64::try_from(bytes.len()).unwrap(),
                DigestV1::of(&bytes),
            ));
        }
    }
    entries
}

fn input_tree(role: TreeRoleV1, manifest: &str, entries: Vec<TreeEntryV1>) -> InputTreeV1 {
    InputTreeV1::try_from_entries(role, CanonicalPathV1::try_new(manifest).unwrap(), entries)
        .unwrap()
}

fn tool(name: &str, version: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        version,
        "qualification",
        "aarch64-apple-darwin",
        DigestV1::of(name.as_bytes()),
    )
    .unwrap()
}

fn artifact(name: &str, version: &str) -> ArtifactIdentityV1 {
    ArtifactIdentityV1::try_new(
        name,
        version,
        "qualification",
        DigestV1::of(format!("{name}-source").as_bytes()),
        DigestV1::of(format!("{name}-artifact").as_bytes()),
    )
    .unwrap()
}
