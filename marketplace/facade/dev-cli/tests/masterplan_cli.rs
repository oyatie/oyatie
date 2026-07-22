// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

#[test]
fn gen_masterplan_preserves_exact_legacy_and_amended_live_statuses() {
    let temp = temp_dir("gen-masterplan-status-filter");
    let decisions_dir = temp.join("docs/decisions");
    fs::create_dir_all(&decisions_dir).expect("decisions dir created");

    write_adr(
        &decisions_dir,
        "ADR-1000-live-leptos.md",
        "Accepted",
        "Live Leptos app-shell work",
    );
    write_adr(
        &decisions_dir,
        "ADR-1001-lowercase-accepted.md",
        "accepted",
        "Lowercase accepted planning work",
    );
    write_adr(
        &decisions_dir,
        "ADR-1002-accepted-amendment.md",
        "Accepted (amendment)",
        "Accepted amendment planning work",
    );
    write_adr(
        &decisions_dir,
        "ADR-1003-superseded-solidjs.md",
        "Superseded",
        "Retired SolidJS app-shell work",
    );
    write_adr(
        &decisions_dir,
        "ADR-1004-draft-solidjs.md",
        "Proposed",
        "Draft SolidJS experiment",
    );
    write_adr(
        &decisions_dir,
        "ADR-1005-amended-leptos.md",
        "Amended",
        "Amended Leptos app-shell work remains live",
    );

    let output_path = temp.join("docs/machine-readable/masterplan.generated.json");
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gen",
            "masterplan",
            "--decisions-dir",
            decisions_dir.to_str().expect("utf8 decisions dir"),
            "--output",
            output_path.to_str().expect("utf8 output path"),
            "--write",
        ])
        .output()
        .expect("gen masterplan command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let projection_text = fs::read_to_string(&output_path).expect("projection written");
    let projection: Value = serde_json::from_str(&projection_text).expect("projection json");
    assert_eq!(projection["adr_count"], 4);
    assert_eq!(projection["deliverable_count"], 4);

    let projected_ids: Vec<&str> = projection["milestones"][0]["adrs"]
        .as_array()
        .expect("adrs array")
        .iter()
        .map(|adr| adr["id"].as_str().expect("adr id"))
        .collect();
    assert_eq!(
        projected_ids,
        vec!["ADR-1000", "ADR-1001", "ADR-1002", "ADR-1005"]
    );
    assert!(projection_text.contains("ADR-1001"));
    assert!(projection_text.contains("ADR-1002"));
    assert!(!projection_text.contains("ADR-1003"));
    assert!(!projection_text.contains("ADR-1004"));
    assert!(!projection_text.contains("SolidJS"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn live_masterplan_projection_excludes_superseded_solidjs_adr() {
    let temp = temp_dir("gen-masterplan-live-solidjs-filter");
    let decisions_dir = repo_root().join("docs/decisions");
    let output_path = temp.join("docs/machine-readable/masterplan.generated.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gen",
            "masterplan",
            "--decisions-dir",
            decisions_dir.to_str().expect("utf8 decisions dir"),
            "--output",
            output_path.to_str().expect("utf8 output path"),
            "--write",
        ])
        .output()
        .expect("live gen masterplan command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let projection_text = fs::read_to_string(&output_path).expect("projection written");
    let projection: Value = serde_json::from_str(&projection_text).expect("projection json");
    let amended_ids: Vec<&str> = projection["milestones"]
        .as_array()
        .expect("milestones array")
        .iter()
        .flat_map(|milestone| milestone["adrs"].as_array().expect("adrs array").iter())
        .filter(|adr| adr["status"] == "Amended")
        .filter_map(|adr| adr["id"].as_str())
        .filter(|id| matches!(*id, "ADR-0363" | "ADR-0388" | "ADR-0619"))
        .collect();
    assert_eq!(
        amended_ids,
        vec!["ADR-0363", "ADR-0388", "ADR-0619"],
        "all amended planning ADRs remain in the live projection"
    );
    assert!(
        !projection_text.contains("ADR-0372"),
        "superseded ADR-0372 must not appear in live generated masterplan"
    );
    assert!(
        !projection_text.contains("SolidJS") && !projection_text.contains("SolidStart"),
        "retired SolidJS/SolidStart planning text must not appear in live generated masterplan"
    );

    fs::remove_dir_all(temp).ok();
}

fn write_adr(decisions_dir: &Path, file_name: &str, status: &str, description: &str) {
    let amended_date = if status == "Amended" {
        "amended_date: 2026-07-22\n"
    } else {
        ""
    };
    fs::write(
        decisions_dir.join(file_name),
        format!(
            r#"---
status: {status}
{amended_date}planning_impact: true
milestone: M-FRONTEND
depends_on: []
deliverables:
  - id: {id}-D1
    description: "{description}"
    exit_criteria: "generated projection includes only live accepted work"
    verified_by: "buck2 test //marketplace/facade/dev-cli:marketplace-dev-cli-masterplan-cli"
---
# {id}
"#,
            id = &file_name[..8],
            amended_date = amended_date,
        ),
    )
    .expect("ADR fixture written");
}

fn repo_root() -> PathBuf {
    const ADR_0372: &str =
        "docs/decisions/ADR-0372-frontend-stack-solidjs-ts-with-rust-wasm-compute-modules.md";
    std::env::current_dir()
        .expect("current dir readable")
        .ancestors()
        .find(|dir| dir.join(ADR_0372).exists())
        .expect("could not locate Oyatie repo root from current dir")
        .to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{name}-{nanos}"))
}
