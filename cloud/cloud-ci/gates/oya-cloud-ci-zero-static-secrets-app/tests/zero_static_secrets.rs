// cloud-ci-zero-static-secrets evaluator and live-corpus tests. ADR-0083 Tier-3: integration
// tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_zero_static_secrets_app::{GATE_ID, Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

fn fake_openai_key() -> String {
    format!("{}{}", "sk-", "A".repeat(40))
}

fn fake_github_pat() -> String {
    format!("{}{}", "github_pat_", "B".repeat(82))
}

fn fake_bearer_token() -> String {
    "C".repeat(32)
}

fn policy_without_exceptions() -> Value {
    json!({
        "gate_id": GATE_ID,
        "bootstrap_exceptions": [],
    })
}

fn bootstrap_policy() -> Value {
    serde_json::from_str(include_str!("../zero-static-secrets-policy.json"))
        .expect("policy fixture parses")
}

fn row(path: &str, line: u64, text: String) -> Value {
    json!({
        "path": path,
        "line": line,
        "text": text,
    })
}

fn input(policy: Value, rows: Vec<Value>, scanned_paths: u64) -> Value {
    json!({
        "_provenance": {
            "scanned_paths": scanned_paths,
            "scanner": "tracked-path-candidate-line-scanner"
        },
        "policy": policy,
        "rows": rows,
    })
}

#[test]
fn static_secret_fixture_is_detected_without_policy_exception_and_redacted() {
    let input = input(
        policy_without_exceptions(),
        vec![row(
            "src/service.rs",
            7,
            format!("let token = \"{}\";", fake_openai_key()),
        )],
        1,
    );

    let findings = evaluate_keyed(&input);
    let finding = findings
        .iter()
        .find(|finding| finding.code == "static_secret_detected")
        .expect("static secret finding");
    assert_eq!(finding.key, "src/service.rs:7:openai_or_anthropic_key");
    assert!(
        !finding.detail.contains("sk-"),
        "detail leaked secret-ish text"
    );
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn raw_authorization_bearer_header_is_detected_and_redacted() {
    let input = input(
        policy_without_exceptions(),
        vec![row(
            "src/http.rs",
            9,
            format!("Authorization: Bearer {}", fake_bearer_token()),
        )],
        1,
    );

    let findings = evaluate_keyed(&input);
    let finding = findings
        .iter()
        .find(|finding| finding.code == "static_secret_detected")
        .expect("authorization bearer finding");
    assert_eq!(finding.key, "src/http.rs:9:authorization_bearer_token");
    assert!(
        !finding.detail.contains(fake_bearer_token().as_str()),
        "detail leaked secret-ish text"
    );
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn bootstrap_exception_must_be_declared_in_policy_data() {
    let bootstrap_path = "bootstrap/root-of-trust/sealed-openbao-dev.unseal.env";
    let bootstrap_line = format!(
        "OPENBAO_TRANSITIONAL_UNSEAL_KEY={} # zero-static-secrets: allow",
        fake_openai_key()
    );

    let inline_comment_only = input(
        policy_without_exceptions(),
        vec![row(bootstrap_path, 3, bootstrap_line.clone())],
        1,
    );
    let inline_findings = evaluate_keyed(&inline_comment_only);
    assert!(
        inline_findings.iter().any(|finding| {
            finding.code == "static_secret_detected"
                && finding.key == format!("{bootstrap_path}:3:openai_or_anthropic_key")
        }),
        "inline comments must not grant exceptions: {inline_findings:?}"
    );
    assert_eq!(evaluate(&inline_comment_only).verdict, Verdict::Red);

    let declared_in_policy = input(
        bootstrap_policy(),
        vec![row(bootstrap_path, 3, bootstrap_line)],
        1,
    );

    assert!(evaluate_keyed(&declared_in_policy).is_empty());
    assert_eq!(evaluate(&declared_in_policy).verdict, Verdict::Green);
}

#[test]
fn policy_and_observed_rows_fail_closed_when_contract_is_incomplete() {
    let bad_policy = json!({
        "gate_id": "wrong-gate",
        "bootstrap_exceptions": [
            {"id": "dup", "path": "a", "secret_kind": "*", "line_contains": "TOKEN=", "owner": "sec", "reason": "bootstrap", "replacement_contract": "delete"},
            {"id": "dup", "path": "b"}
        ],
    });
    let input = input(
        bad_policy,
        vec![json!({"path": "src/main.rs", "text": fake_github_pat()})],
        0,
    );
    let codes = evaluate(&input).violations;

    assert!(codes.contains("static_secret_policy_gate_id_mismatch"));
    assert!(codes.contains("static_secret_exception_duplicate"));
    assert!(codes.contains("static_secret_exception_missing_field"));
    assert!(codes.contains("static_secret_observed_row_missing_field"));
    assert!(codes.contains("static_secret_no_scanned_paths"));
}

#[test]
fn explicit_redacted_placeholders_do_not_count_as_live_static_secrets() {
    let redacted_github = format!(
        "secret_pattern_hit(\"token = \\\"«redacted:{}{}»\\\"\")",
        "ghp_",
        "0".repeat(36)
    );
    let redacted_aws = format!(
        "secret_pattern_hit(\"key = \\\"«redacted:{}{}»\\\"\")",
        "AKIA",
        "0".repeat(16)
    );
    let input = input(
        policy_without_exceptions(),
        vec![
            row(
                "oya/developer-sdk/crates/oya-dev-cli/src/commands/lint.rs",
                865,
                redacted_github,
            ),
            row(
                "oya/developer-sdk/crates/oya-dev-cli/src/commands/lint.rs",
                869,
                redacted_aws,
            ),
        ],
        2,
    );

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn evaluate_is_bare_projection_of_evaluate_keyed() {
    let input = input(
        policy_without_exceptions(),
        vec![
            row("src/openai.rs", 1, fake_openai_key()),
            row("src/github.rs", 2, fake_github_pat()),
        ],
        2,
    );
    let projected = evaluate_keyed(&input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    assert_eq!(evaluate(&input).violations, projected);
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root
        .join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json");
    let (mut command, run_description) = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        (Command::new(bin), "run producer binary")
    } else {
        let mut command =
            Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()));
        command
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("oya-cloud-ci-accounting-registry-app")
            .arg("--");
        (command, "cargo run oya-cloud-ci-accounting-registry-app")
    };

    let output = command
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect(run_description);

    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

#[test]
fn zero_static_secrets_face_scans_the_live_tracked_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "zero-static-secrets");
    let scanned_paths = face["_provenance"]["scanned_paths"]
        .as_u64()
        .expect("zero-static face scanned_paths");
    let candidate_rows = face["rows"].as_array().expect("zero-static rows");
    assert!(
        scanned_paths > 1000,
        "the zero-static-secrets face must prove it scanned the live tracked corpus, got {scanned_paths}"
    );

    let findings = evaluate_keyed(&face);
    eprintln!(
        "BORN-BLOCKING zero-static-secrets: scanned_paths={} candidate_rows={} total_findings={} verdict={:?}",
        scanned_paths,
        candidate_rows.len(),
        findings.len(),
        evaluate(&face).verdict,
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "static_secret_no_scanned_paths"),
        "producer must report a non-empty scanned corpus: {findings:?}"
    );
    assert!(
        findings.is_empty(),
        "current tracked corpus must not contain unexceptioned static credential-shaped literals: {findings:?}"
    );
}
