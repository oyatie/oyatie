use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CargoMessageEvidenceFailureV1 {
    MalformedMessage,
    AmbiguousExecutable,
    MissingExecutable,
    AmbiguousBuildFinish,
    BuildNotSuccessful,
}

pub(super) struct CargoMessageStreamV1 {
    events: Box<[serde_json::Value]>,
}

impl CargoMessageStreamV1 {
    pub(super) fn try_new(messages: &[u8]) -> Result<Self, CargoMessageEvidenceFailureV1> {
        let mut events = Vec::new();
        let mut build_success = None;
        for line in messages
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let event: serde_json::Value = serde_json::from_slice(line)
                .map_err(|_| CargoMessageEvidenceFailureV1::MalformedMessage)?;
            let reason = event["reason"]
                .as_str()
                .ok_or(CargoMessageEvidenceFailureV1::MalformedMessage)?;
            if reason == "build-finished" {
                let success = event["success"]
                    .as_bool()
                    .ok_or(CargoMessageEvidenceFailureV1::MalformedMessage)?;
                if build_success.replace(success).is_some() {
                    return Err(CargoMessageEvidenceFailureV1::AmbiguousBuildFinish);
                }
            }
            events.push(event);
        }
        if build_success != Some(true) {
            return Err(CargoMessageEvidenceFailureV1::BuildNotSuccessful);
        }
        Ok(Self {
            events: events.into_boxed_slice(),
        })
    }

    pub(super) fn events(&self) -> &[serde_json::Value] {
        &self.events
    }
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
        .env("CARGO_TARGET_DIR", qualification_target_dir(source_root))
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

pub(super) fn qualification_target_dir(source_root: &Path) -> PathBuf {
    std::env::var_os("REINDEER_QUALIFICATION_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| source_root.join("target"))
}

fn reindeer_binary_from_cargo_messages(
    messages: &[u8],
) -> Result<PathBuf, CargoMessageEvidenceFailureV1> {
    let messages = CargoMessageStreamV1::try_new(messages)?;
    let mut executable = None;
    for event in messages.events() {
        match event["reason"].as_str() {
            Some("compiler-artifact") if is_reindeer_binary(event) => {
                let path = event["executable"]
                    .as_str()
                    .filter(|path| !path.is_empty())
                    .ok_or(CargoMessageEvidenceFailureV1::MalformedMessage)?;
                if executable.replace(PathBuf::from(path)).is_some() {
                    return Err(CargoMessageEvidenceFailureV1::AmbiguousExecutable);
                }
            }
            _ => {}
        }
    }
    executable.ok_or(CargoMessageEvidenceFailureV1::MissingExecutable)
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
    assert!(matches!(
        reindeer_binary_from_cargo_messages(missing),
        Err(CargoMessageEvidenceFailureV1::MissingExecutable)
    ));

    let ambiguous = concat!(
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/a\"}\n",
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/b\"}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n",
    );
    assert!(matches!(
        reindeer_binary_from_cargo_messages(ambiguous.as_bytes()),
        Err(CargoMessageEvidenceFailureV1::AmbiguousExecutable)
    ));
}

#[test]
fn cargo_messages_refuse_malformed_or_unsuccessful_streams() {
    assert!(matches!(
        reindeer_binary_from_cargo_messages(b"not-json\n"),
        Err(CargoMessageEvidenceFailureV1::MalformedMessage)
    ));
    assert!(matches!(
        reindeer_binary_from_cargo_messages(b"{\"reason\":\"build-finished\",\"success\":false}\n"),
        Err(CargoMessageEvidenceFailureV1::BuildNotSuccessful)
    ));
}

#[test]
fn cargo_message_stream_never_skips_a_malformed_line() {
    let messages = concat!(
        "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"reindeer\",\"kind\":[\"bin\"]},\"executable\":\"/tmp/reindeer\"}\n",
        "not-json\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n",
    );

    assert!(matches!(
        CargoMessageStreamV1::try_new(messages.as_bytes()),
        Err(CargoMessageEvidenceFailureV1::MalformedMessage)
    ));

    let duplicate_finish = concat!(
        "{\"reason\":\"build-finished\",\"success\":true}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n",
    );
    assert!(matches!(
        CargoMessageStreamV1::try_new(duplicate_finish.as_bytes()),
        Err(CargoMessageEvidenceFailureV1::AmbiguousBuildFinish)
    ));
}
