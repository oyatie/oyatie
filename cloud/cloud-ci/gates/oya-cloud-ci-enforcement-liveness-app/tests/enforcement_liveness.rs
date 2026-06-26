// FRIC-012/G011 enforcement-liveness live-corpus gate. Loads the Buck-declared
// materialized enforcement-liveness face, then asserts today's tracked hooks are all either
// wired in both project wiring files or marked as compatibility stubs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use oya_cloud_ci_enforcement_liveness_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

const FACE_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_FACE";

fn load_declared_face() -> Value {
    let env_path = std::env::var(FACE_ENV)
        .unwrap_or_else(|e| panic!("{FACE_ENV} must point at Buck-declared generated face: {e}"));
    let mut path = PathBuf::from(env_path);
    if path.is_dir() {
        path.push("enforcement-liveness.generated.json");
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "read Buck-declared enforcement-liveness face {}: {e}",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!(
            "parse Buck-declared enforcement-liveness face {}: {e}",
            path.display()
        )
    })
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
    let face = load_declared_face();
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
