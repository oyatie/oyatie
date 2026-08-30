use std::path::Path;
use std::process::Command;

pub(crate) fn run_artifact(binary: &Path, root: &Path, invocation_id: &str) -> Vec<u8> {
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

pub(crate) struct ParsedArtifact<'a> {
    pub(crate) invocation_id: &'a [u8],
    pub(crate) graph: &'a [u8],
    pub(crate) rendered_buck: &'a [u8],
    pub(crate) receipt_sha256: &'a [u8],
}

pub(crate) fn parse_artifact(bytes: &[u8]) -> ParsedArtifact<'_> {
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
