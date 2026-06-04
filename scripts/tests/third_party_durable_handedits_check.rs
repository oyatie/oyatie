#![allow(dead_code)]

#[path = "../ci/assert-third-party-durable-handedits.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {}: {}", path, error);
    })
}

#[test]
fn checked_in_third_party_durable_env_passes() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
    assert!(!evaluation.would_update);
}

#[test]
fn third_party_env_rejects_missing_musl_select() {
    let text = read_repo_file("third-party/BUCK")
        .replace("\"root//platforms:libc_musl\": {", "\"DEFAULT_ONLY\": {");
    let (failures, would_update) = gate::third_party_failures(&text);
    assert!(would_update);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("selected env drifted")),
        "{:?}",
        failures
    );
}

#[test]
fn normalizer_recreates_selected_env() {
    let current = read_repo_file("third-party/BUCK");
    let old_env = "    env = {\n            \"CARGO_MANIFEST_LINKS\": \"aws_lc_0_41_0\",\n            \"LDFLAGS\": \"-nostartfiles\",\n        },\n";
    let drifted = current.replace(&gate::selected_env(), old_env);
    assert_ne!(drifted, current);
    let normalized = gate::normalize_text(&drifted).expect("normalize drifted env");
    assert_eq!(normalized, current);
}
