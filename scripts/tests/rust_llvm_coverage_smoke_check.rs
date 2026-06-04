#[allow(dead_code)]
#[path = "../ci/run-rust-llvm-coverage-smoke.rs"]
mod smoke;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use smoke::{SmokeConfig, render_json, run_smoke};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn source_fixture() -> PathBuf {
    repo_root().join("specs/fixtures/rust-llvm-coverage-smoke/branchy.rs")
}

fn unique_dir(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "oya-{label}-{}-{nanos}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn checked_in_smoke_passes() {
    let result = run_smoke(&SmokeConfig {
        source: source_fixture(),
        out: None,
        rustc: None,
        llvm_bin: None,
    });
    assert_eq!(result.verdict, "PASS", "{}", render_json(&result));
    assert!(result.failures.is_empty());
    assert!(result.fixture_generated);
    assert_eq!(result.line_percent, Some(100.0));
    assert_eq!(result.region_percent, Some(100.0));
    assert!(result.profraw_count >= 1);
    assert!(result.text_report.contains("TOTAL"));
    let rendered = render_json(&result);
    assert!(rendered.contains("\"ambient_path_llvm_tools_required\":false"));
    assert!(rendered.contains("\"fixture_coverage_smoke_generated\":true"));
    assert!(rendered.contains("\"production_coverage_report_generated\":false"));
    assert!(rendered.contains("\"coverage_budget_enforced\":false"));
    assert!(rendered.contains("\"live_required_context_execution_proven\":false"));
    assert!(rendered.contains("\"p0_0_green\":false"));
    assert!(rendered.contains("\"phase0_complete\":false"));
    assert!(rendered.contains("rustc -C instrument-coverage"));
    assert!(rendered.contains("llvm-profdata merge -sparse"));
    assert!(rendered.contains("export --format=text"));
}

#[test]
fn missing_source_fails() {
    let dir = unique_dir("coverage-missing-source");
    let missing = dir.join("no-such.rs");
    let result = run_smoke(&SmokeConfig {
        source: missing,
        out: None,
        rustc: None,
        llvm_bin: None,
    });
    let _ = fs::remove_dir_all(dir);
    assert_eq!(result.verdict, "FAIL");
    assert!(result.failures.contains(&"missing_source_file".to_owned()));
    let rendered = render_json(&result);
    assert!(rendered.contains("\"production_coverage_report_generated\":false"));
    assert!(rendered.contains("\"p0_0_green\":false"));
    assert!(rendered.contains("\"phase0_complete\":false"));
}

#[test]
fn missing_profdata_fails() {
    let dir = unique_dir("coverage-empty-llvm-bin");
    let result = run_smoke(&SmokeConfig {
        source: source_fixture(),
        out: None,
        rustc: None,
        llvm_bin: Some(dir.clone()),
    });
    let _ = fs::remove_dir_all(dir);
    assert_eq!(result.verdict, "FAIL", "{}", render_json(&result));
    assert!(
        result
            .failures
            .contains(&"missing_llvm_profdata".to_owned())
    );
    let rendered = render_json(&result);
    assert!(rendered.contains("\"production_coverage_report_generated\":false"));
    assert!(rendered.contains("\"p0_0_green\":false"));
    assert!(rendered.contains("\"phase0_complete\":false"));
}
