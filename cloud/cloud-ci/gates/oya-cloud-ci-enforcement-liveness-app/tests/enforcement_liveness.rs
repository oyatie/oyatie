// FRIC-012/G011 enforcement-liveness live-corpus gate. Runs the producer
// `--face enforcement-liveness`, then asserts today's tracked hooks are all either wired in
// both project wiring files or marked as compatibility stubs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_enforcement_liveness_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

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
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        Command::new(bin)
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg(face)
            .current_dir(root)
            .output()
            .expect("run producer binary")
    } else {
        Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("oya-cloud-ci-accounting-registry-app")
            .arg("--")
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg(face)
            .current_dir(root)
            .output()
            .expect("cargo run oya-cloud-ci-accounting-registry-app")
    };
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

#[test]
fn green_rows_allow_dual_wired_hooks_and_marked_compatibility_stubs() {
    let input = json!({
        "rows": [
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/no-cargo-enforcer.sh",
                "wired_in_claude": true,
                "wired_in_codex": true,
                "stub_marked": false
            },
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/session-start-context-inject.sh",
                "wired_in_claude": false,
                "wired_in_codex": false,
                "stub_marked": true
            },
            {
                "row_type": "command_reference",
                "wiring_file": ".claude/settings.json",
                "command_path": "tools/hooks/no-cargo-enforcer.sh",
                "target_exists": true
            }
        ]
    });

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn live_hook_missing_dual_wiring_is_red() {
    let input = json!({
        "rows": [{
            "row_type": "hook",
            "hook_path": "tools/hooks/no-cargo-enforcer.sh",
            "wired_in_claude": false,
            "wired_in_codex": true,
            "stub_marked": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "hook_unwired_without_stub_marker"
            && finding.key == "tools/hooks/no-cargo-enforcer.sh"
            && finding.remediation.contains(".claude/settings.json")
            && finding.remediation.contains(".codex/hooks.json")
            && finding.remediation.contains("governance PR")
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn one_sided_live_hook_wiring_is_mirror_drift() {
    let input = json!({
        "rows": [{
            "row_type": "hook",
            "hook_path": "tools/hooks/stale-tool-suggester.sh",
            "wired_in_claude": true,
            "wired_in_codex": false,
            "stub_marked": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "hook_wiring_mirror_drift"
            && finding.key == "tools/hooks/stale-tool-suggester.sh"
            && finding.remediation.contains(".claude/settings.json")
            && finding.remediation.contains(".codex/hooks.json")
            && finding.remediation.contains("governance PR")
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn wired_hook_missing_file_is_red() {
    let input = json!({
        "rows": [{
            "row_type": "command_reference",
            "wiring_file": ".codex/hooks.json",
            "command_path": "tools/hooks/deleted-enforcer.sh",
            "target_exists": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert_eq!(findings.len(), 1);
    let finding = findings.iter().next().unwrap();
    assert_eq!(finding.code, "wired_hook_missing_file");
    assert_eq!(
        finding.key,
        ".codex/hooks.json:tools/hooks/deleted-enforcer.sh"
    );
    assert!(finding.remediation.contains(".codex/hooks.json"));
    assert!(finding.remediation.contains("governance PR"));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn evaluate_is_bare_projection_of_evaluate_keyed() {
    let input = json!({
        "rows": [
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/no-cargo-enforcer.sh",
                "wired_in_claude": false,
                "wired_in_codex": true,
                "stub_marked": false
            },
            {
                "row_type": "command_reference",
                "wiring_file": ".codex/hooks.json",
                "command_path": "tools/hooks/deleted-enforcer.sh",
                "target_exists": false
            }
        ]
    });
    let projected: BTreeSet<String> = evaluate_keyed(&input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    assert_eq!(evaluate(&input).violations, projected);
    for code in [
        "hook_unwired_without_stub_marker",
        "hook_wiring_mirror_drift",
        "wired_hook_missing_file",
    ] {
        assert!(projected.contains(code), "expected {code} in {projected:?}");
    }
}

#[test]
fn enforcement_liveness_face_reports_current_tree_green() {
    let root = repo_root();
    let face = run_producer_face(&root, "enforcement-liveness");
    let rows = face["rows"].as_array().expect("enforcement-liveness rows");
    let hook_rows = rows
        .iter()
        .filter(|row| row["row_type"] == "hook")
        .count();
    let command_rows = rows
        .iter()
        .filter(|row| row["row_type"] == "command_reference")
        .count();
    let stub_rows = rows
        .iter()
        .filter(|row| row["stub_marked"].as_bool() == Some(true))
        .count();

    eprintln!(
        "ENFORCEMENT-LIVENESS live corpus: hooks={hook_rows} command_refs={command_rows} stubs={stub_rows}"
    );

    assert_eq!(hook_rows, 11, "tracked tools/hooks/*.sh census changed");
    assert_eq!(command_rows, 18, "Claude+Codex hook command reference census changed");
    assert_eq!(stub_rows, 2, "compatibility stub count changed");
    assert!(evaluate_keyed(&face).is_empty());
    assert_eq!(evaluate(&face).verdict, Verdict::Green);
}
