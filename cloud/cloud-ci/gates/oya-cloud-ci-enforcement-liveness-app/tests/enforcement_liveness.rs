// FRIC-012/G011 enforcement-liveness live-corpus gate. Runs the Buck-declared
// accounting-registry producer for the enforcement-liveness face, then asserts today's tracked
// hooks are all either wired in both project wiring files or marked as compatibility stubs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use oya_cloud_ci_enforcement_liveness_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

const PRODUCER_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_PRODUCER";
const SCM_FACTS_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_SCM_FACTS";

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

fn load_produced_face() -> Value {
    let producer = std::env::var(PRODUCER_ENV).unwrap_or_else(|e| {
        panic!("{PRODUCER_ENV} must point at Buck-built accounting-registry producer: {e}")
    });
    let scm_facts = std::env::var(SCM_FACTS_ENV).unwrap_or_else(|e| {
        panic!("{SCM_FACTS_ENV} must point at a Buck-declared scm-facts fixture: {e}")
    });
    let root = repo_root();
    let output = Command::new(&producer)
        .args([
            "--repo-root",
            root.to_str().expect("repo root utf-8"),
            "--scm-facts",
            scm_facts.as_str(),
            "--stdout",
            "--face",
            "enforcement-liveness",
        ])
        .output()
        .unwrap_or_else(|e| panic!("run Buck-built enforcement-liveness producer {producer}: {e}"));
    if !output.status.success() {
        panic!(
            "Buck-built enforcement-liveness producer failed with status {:?}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("parse Buck-produced enforcement-liveness face: {e}"))
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
    let face = load_produced_face();
    let rows = face["rows"].as_array().expect("enforcement-liveness rows");
    let hook_rows = rows.iter().filter(|row| row["row_type"] == "hook").count();
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

    assert_eq!(hook_rows, 12, "tracked tools/hooks/*.sh census changed");
    assert_eq!(
        command_rows, 20,
        "Claude+Codex hook command reference census changed"
    );
    assert_eq!(stub_rows, 2, "compatibility stub count changed");
    assert!(evaluate_keyed(&face).is_empty());
    assert_eq!(evaluate(&face).verdict, Verdict::Green);
}
