use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use dependency_declarations_generation_reindeer::{
    ReindeerProviderSourceAdaptationV1, ReindeerProviderSourceFileV1,
    ReindeerProviderSourceSnapshotV1, adapt_reindeer_provider_source_v1,
};

pub(super) const PINNED_REVISION: &str = "bb681570d2bc47d1446080c12b8681a50a95f628";
pub(super) const SOURCE_PATHS: [&str; 7] = [
    "src/artifact.rs",
    "src/artifact/serializer.rs",
    "src/artifact/serializer/builders.rs",
    "src/artifact/value.rs",
    "src/buck.rs",
    "src/buckify.rs",
    "src/main.rs",
];
static NEXT_SOURCE_FIXTURE: AtomicUsize = AtomicUsize::new(0);

pub(super) fn pinned_source_root() -> PathBuf {
    std::env::var_os("REINDEER_PINNED_SOURCE_ROOT")
        .map(PathBuf::from)
        .expect("REINDEER_PINNED_SOURCE_ROOT must name the exact pinned checkout")
}

pub(super) fn source_batch(root: &Path) -> Vec<ReindeerProviderSourceFileV1> {
    SOURCE_PATHS
        .iter()
        .map(|path| {
            let source = root.join(path);
            if source.exists() {
                ReindeerProviderSourceFileV1::present(*path, std::fs::read(source).unwrap())
            } else {
                ReindeerProviderSourceFileV1::absent(*path)
            }
        })
        .collect()
}

pub(super) fn materialized_fixture(
    source_root: &Path,
) -> (ReindeerProviderSourceAdaptationV1, SourceFixture) {
    let snapshot =
        ReindeerProviderSourceSnapshotV1::new(PINNED_REVISION, [7; 32], source_batch(source_root));
    let adaptation = adapt_reindeer_provider_source_v1(&snapshot).unwrap();
    let fixture = SourceFixture::copy_from(source_root);
    for file in adaptation.files() {
        let path = fixture.path().join(file.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, file.postimage()).unwrap();
    }
    (adaptation, fixture)
}

pub(super) fn run_artifact(binary: &Path, root: &Path, invocation_id: &str) -> Vec<u8> {
    let output = Command::new(binary)
        .arg("--cargo-options=--offline")
        .arg("-c")
        .arg(root.join("reindeer.toml"))
        .arg("buckify")
        .arg("--artifact-v1")
        .arg(invocation_id)
        .current_dir(root)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "artifact run failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    output.stdout
}

pub(super) struct ParsedArtifact<'a> {
    pub(super) invocation_id: &'a [u8],
    pub(super) graph: &'a [u8],
    pub(super) rendered_buck: &'a [u8],
    pub(super) receipt_sha256: &'a [u8],
}

pub(super) fn parse_artifact(bytes: &[u8]) -> ParsedArtifact<'_> {
    const MAGIC: &[u8] = b"REINDEER_GENERATED_ARTIFACT_V1\0";
    assert!(bytes.starts_with(MAGIC));
    let mut cursor = &bytes[MAGIC.len()..];
    let invocation_id = take_frame(&mut cursor);
    let graph = take_frame(&mut cursor);
    let rendered_buck = take_frame(&mut cursor);
    assert_eq!(cursor.len(), 32);
    ParsedArtifact {
        invocation_id,
        graph,
        rendered_buck,
        receipt_sha256: cursor,
    }
}

fn take_frame<'a>(cursor: &mut &'a [u8]) -> &'a [u8] {
    let (length, rest) = cursor.split_at(8);
    let length = usize::try_from(u64::from_be_bytes(length.try_into().unwrap())).unwrap();
    let (value, rest) = rest.split_at(length);
    *cursor = rest;
    value
}

pub(super) fn write_qualification_workspace(root: &Path) {
    let third_party = root.join("third-party");
    std::fs::create_dir_all(third_party.join("top")).unwrap();
    std::fs::create_dir_all(third_party.join("local/fixture/src")).unwrap();
    std::fs::write(
        root.join("reindeer.toml"),
        concat!(
            "third_party_dir = \"third-party\"\n",
            "vendor = false\n",
            "include_top_level = true\n",
            "\n",
            "[buck]\n",
            "rust_library = \"cargo.rust_library\"\n",
            "rust_binary = \"cargo.rust_binary\"\n",
            "buckfile_imports = \"load(\\\"@prelude//rust:cargo_package.bzl\\\", ",
            "\\\"cargo\\\")\"\n",
        ),
    )
    .unwrap();
    std::fs::write(
        third_party.join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "exclude = [\"local/fixture\"]\n",
            "\n",
            "[package]\n",
            "name = \"rust-third-party\"\n",
            "version = \"0.0.0\"\n",
            "edition = \"2024\"\n",
            "publish = false\n",
            "\n",
            "[[bin]]\n",
            "name = \"top\"\n",
            "path = \"top/main.rs\"\n",
            "\n",
            "[dependencies]\n",
            "fixture = { path = \"local/fixture\" }\n",
        ),
    )
    .unwrap();
    std::fs::write(
        third_party.join("Cargo.lock"),
        concat!(
            "# This file is automatically @generated by Cargo.\n",
            "# It is not intended for manual editing.\n",
            "version = 4\n",
            "\n",
            "[[package]]\n",
            "name = \"fixture\"\n",
            "version = \"1.0.0\"\n",
            "\n",
            "[[package]]\n",
            "name = \"rust-third-party\"\n",
            "version = \"0.0.0\"\n",
            "dependencies = [\n",
            " \"fixture\",\n",
            "]\n",
        ),
    )
    .unwrap();
    std::fs::write(third_party.join("top/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(
        third_party.join("local/fixture/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"fixture\"\n",
            "version = \"1.0.0\"\n",
            "edition = \"2024\"\n",
            "publish = false\n",
        ),
    )
    .unwrap();
    std::fs::write(
        third_party.join("local/fixture/src/lib.rs"),
        "pub fn fixture() {}\n",
    )
    .unwrap();
}

pub(super) struct SourceFixture {
    root: PathBuf,
}

impl SourceFixture {
    fn copy_from(source: &Path) -> Self {
        let sequence = NEXT_SOURCE_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "reindeer-provider-adaptation-{}-{sequence}",
            std::process::id(),
        ));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        copy_tree(source, &root);
        Self { root }
    }

    pub(super) fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for SourceFixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let source = entry.path();
        let destination = destination.join(name);
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&source, &destination);
        } else {
            std::fs::copy(source, destination).unwrap();
        }
    }
}
