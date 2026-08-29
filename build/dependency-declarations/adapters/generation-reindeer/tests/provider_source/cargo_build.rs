use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CargoBuildEvidenceFailureV1 {
    MalformedMessage,
    AmbiguousExecutable,
    MissingExecutable,
    AmbiguousBuildFinish,
    BuildNotSuccessful,
}

pub(super) fn build_reindeer_binary(cargo: &OsStr, source_root: &Path) -> PathBuf {
    let output = Command::new(cargo)
        .args([
            "build",
            "--locked",
            "--offline",
            "--bin",
            "reindeer",
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(source_root)
        .env("CARGO_TARGET_DIR", source_root.join("target"))
        .output()
        .expect("adapted provider build must run");
    assert!(
        output.status.success(),
        "adapted provider failed to build:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let binary = reindeer_binary_from_cargo_messages(&output.stdout)
        .expect("successful Cargo output must identify exactly one Reindeer binary");
    if binary.is_absolute() {
        binary
    } else {
        source_root.join(binary)
    }
}

fn reindeer_binary_from_cargo_messages(
    messages: &[u8],
) -> Result<PathBuf, CargoBuildEvidenceFailureV1> {
    let mut executable = None;
    let mut build_success = None;
    for line in messages
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let event: serde_json::Value = serde_json::from_slice(line)
            .map_err(|_| CargoBuildEvidenceFailureV1::MalformedMessage)?;
        let reason = event["reason"]
            .as_str()
            .ok_or(CargoBuildEvidenceFailureV1::MalformedMessage)?;
        match reason {
            "compiler-artifact" if is_reindeer_binary(&event) => {
                let path = event["executable"]
                    .as_str()
                    .filter(|path| !path.is_empty())
                    .ok_or(CargoBuildEvidenceFailureV1::MalformedMessage)?;
                if executable.replace(PathBuf::from(path)).is_some() {
                    return Err(CargoBuildEvidenceFailureV1::AmbiguousExecutable);
                }
            }
            "build-finished" => {
                let success = event["success"]
                    .as_bool()
                    .ok_or(CargoBuildEvidenceFailureV1::MalformedMessage)?;
                if build_success.replace(success).is_some() {
                    return Err(CargoBuildEvidenceFailureV1::AmbiguousBuildFinish);
                }
            }
            _ => {}
        }
    }
    if build_success != Some(true) {
        return Err(CargoBuildEvidenceFailureV1::BuildNotSuccessful);
    }
    executable.ok_or(CargoBuildEvidenceFailureV1::MissingExecutable)
}

fn is_reindeer_binary(event: &serde_json::Value) -> bool {
    event["target"]["name"] == "reindeer"
        && event["target"]["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
}

#[test]
fn cargo_messages_select_the_exact_reindeer_binary() {
    let messages = concat!(
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"lib\"]},\"executable\":null}\n",
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/target/reindeer\"}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n",
    );

    assert_eq!(
        reindeer_binary_from_cargo_messages(messages.as_bytes()).unwrap(),
        PathBuf::from("/tmp/target/reindeer")
    );
}

#[test]
fn cargo_messages_refuse_missing_or_ambiguous_executables() {
    let missing = b"{\"reason\":\"build-finished\",\"success\":true}\n";
    assert!(reindeer_binary_from_cargo_messages(missing).is_err());

    let ambiguous = concat!(
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/a\"}\n",
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/b\"}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n",
    );
    assert!(reindeer_binary_from_cargo_messages(ambiguous.as_bytes()).is_err());
}

#[test]
fn cargo_messages_refuse_malformed_or_unsuccessful_streams() {
    assert!(reindeer_binary_from_cargo_messages(b"not-json\n").is_err());
    assert!(
        reindeer_binary_from_cargo_messages(b"{\"reason\":\"build-finished\",\"success\":false}\n")
            .is_err()
    );
}
