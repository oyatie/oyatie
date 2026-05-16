// ADR-0083 Tier 3: integration tests use .unwrap() / .expect() / panic! to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// ICM JSON round-trip contract test.
//
// Per Condition 3 of iter-3 final fold: mint a synthetic token via `icm store`,
// recall via `icm recall -f json`, parse the JSON array, and assert equality
// of the stored content field.
//
// This test is gated by the presence of the `icm` binary. If `icm` is not on
// PATH, the test is skipped (not failed) — this allows CI without icm installed
// to still pass the xtask fixture matrix gates.

use std::process::Command;

/// Check if the icm binary is available on PATH.
fn icm_available() -> bool {
    Command::new("icm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn icm_jsonl_round_trip() {
    if !icm_available() {
        eprintln!("SKIP icm_jsonl_round_trip: icm binary not found on PATH");
        return;
    }

    // Use a unique content string to avoid false positives from prior test runs.
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let test_content =
        format!("xtask-metadata-augment JSONL round-trip contract test token {unique_suffix}");
    let test_topic = "context-xtask-icm-roundtrip-test";

    // Store the synthetic token.
    let store_output = Command::new("icm")
        .args(["store", "-t", test_topic, "-c", &test_content, "-i", "low"])
        .output()
        .expect("icm store failed to spawn");

    assert!(
        store_output.status.success(),
        "icm store failed: {}",
        String::from_utf8_lossy(&store_output.stderr)
    );

    let stored_id = String::from_utf8_lossy(&store_output.stdout)
        .split_whitespace()
        .next()
        .map(|s| s.replace("Stored:", "").trim().to_owned())
        .unwrap_or_default();

    // Recall with JSON format (-f json emits a parseable JSON array).
    let recall_output = Command::new("icm")
        .args([
            "recall",
            &test_content,
            "-t",
            test_topic,
            "-f",
            "json",
            "-l",
            "10",
        ])
        .output()
        .expect("icm recall failed to spawn");

    // icm recall may return exit 0 even with no matches; we check stdout.
    let stdout = String::from_utf8_lossy(&recall_output.stdout);

    // Parse JSON array: [{content: "...", ...}, ...]
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        eprintln!("JSON parse error: {e}\nstdout was: {stdout}");
        serde_json::Value::Array(vec![])
    });

    let mut found = false;
    if let Some(arr) = parsed.as_array() {
        for entry in arr {
            // icm recall -f json emits `summary` as the stored content field.
            for key in &["summary", "content"] {
                if let Some(field) = entry.get(key).and_then(|v| v.as_str())
                    && field.contains(&test_content)
                {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
    }

    assert!(
        found,
        "ICM JSON round-trip: stored content not found in recall output.\n\
         stored_id={stored_id}\n\
         content={test_content}\n\
         recall stdout={stdout}"
    );
}
