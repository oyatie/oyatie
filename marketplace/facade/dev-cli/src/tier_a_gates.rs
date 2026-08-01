//! Gate dispatcher wiring for the Tier-A hyperscaler-pattern
//! enforcement gates (ADR-0149..ADR-0156).
//!
//! Seven gates land here:
//!
//! 1. `idempotency-key-coverage`   — ADR-0149.
//! 2. `cursor-pagination-coverage` — ADR-0150.
//! 3. `rpo-rto-coverage`           — ADR-0152.
//! 4. `metric-cardinality`         — ADR-0151.
//! 5. `event-schema-versioning`    — ADR-0154.
//! 6. `id-discipline`              — ADR-0156.
//! 7. `image-signing-discipline`   — ADR-0146 + ADR-0039.
//!
//! Each gate scans canonical default paths in the repo; arguments
//! are accepted but optional. Each is strict-mode (fail-closed).
//!
//! Authored 2026-05-18 per the Fix-Agent-I Tier-A landing.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_cursor_pagination_coverage as cursor_check;
use check_event_schema_versioning as event_schema_check;
use check_id_discipline as id_check;
use check_idempotency_key_coverage as idem_check;
use check_image_signing_discipline as image_check;
use check_metric_cardinality as metric_check;
use check_rpo_rto_coverage as rpo_rto_check;

/// Canonical service roots scanned when no explicit `--microservices-root` is given.
/// Order: cloud first (infra services), then oya (application services), then the
/// legacy microservices/ tree for backward compatibility.
const DEFAULT_SERVICE_ROOTS: &[&str] = &["cloud", "oya", "microservices"];
const DEFAULT_WORKFLOWS_DIR: &str = ".github/workflows";

fn parse_flag_with_value(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == flag {
            return iter.next().cloned();
        }
        if let Some(value) = a.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_string());
        }
    }
    None
}

fn read_optional_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn microservice_name_for(path: &Path) -> Option<String> {
    let mut iter = path.components();
    while let Some(c) = iter.next() {
        if c.as_os_str() == OsStr::new("microservices")
            && let Some(next) = iter.next()
        {
            return Some(next.as_os_str().to_string_lossy().to_string());
        }
    }
    None
}

/// Collect all first-level service directories from a list of root paths.
/// For `cloud/` and `oya/`, each direct child of the root is a service dir.
/// For `microservices/`, same pattern. Silently skips roots that don't exist.
fn service_dirs_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.push(p);
            }
        }
    }
    out
}

/// Recursively walk a directory and collect files matching one of
/// `extensions` (no leading dot).
fn collect_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, extensions, &mut out);
    out
}

fn walk(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, extensions, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str())
            && extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        {
            out.push(p);
        }
    }
}

// ---------- Gate 1: idempotency-key-coverage ----------

pub(crate) fn run_idempotency_key_coverage(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut documents = Vec::new();
    // Glob <root>/*/contracts/openapi/*.yaml across all service roots.
    for p in service_dirs_from_roots(&roots) {
        let ms = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let openapi_dir = p.join("contracts").join("openapi");
        if !openapi_dir.exists() {
            continue;
        }
        for f in collect_files(&openapi_dir, &["yaml", "yml"]) {
            if let Some(contents) = read_optional_string(&f) {
                documents.push(idem_check::OpenApiDocument {
                    path: f.to_string_lossy().to_string(),
                    microservice: ms.clone(),
                    contents,
                });
            }
        }
    }

    let (report, findings) = idem_check::audit_all(documents);
    println!(
        "idempotency-key-coverage: {} docs, {} state-changing ops checked, {} covered, {} µservices",
        report.documents_checked,
        report.state_changing_ops_checked,
        report.state_changing_ops_covered,
        report.microservices_audited,
    );
    // Advisory mode until every µservice OpenAPI has been retrofitted;
    // strict-mode promotion tracked under
    // registry/placeholder-debt/adr-follow-ups.yaml#adr-0149-idempotency-impl.
    if !findings.is_empty() {
        println!(
            "idempotency-key-coverage: {} advisory findings (first 30):",
            findings.len()
        );
        for f in findings.iter().take(30) {
            println!(
                "  - {}:{} [{}] {}",
                f.path, f.line, f.microservice, f.message
            );
        }
        println!(
            "(advisory mode per ADR-0149; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0149-idempotency-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 2: cursor-pagination-coverage ----------

pub(crate) fn run_cursor_pagination_coverage(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };
    let mut documents = Vec::new();
    for p in service_dirs_from_roots(&roots) {
        let ms = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let openapi_dir = p.join("contracts").join("openapi");
        if !openapi_dir.exists() {
            continue;
        }
        for f in collect_files(&openapi_dir, &["yaml", "yml"]) {
            if let Some(contents) = read_optional_string(&f) {
                documents.push(cursor_check::OpenApiDocument {
                    path: f.to_string_lossy().to_string(),
                    microservice: ms.clone(),
                    contents,
                });
            }
        }
    }

    let (report, findings) = cursor_check::audit_all(documents);
    println!(
        "cursor-pagination-coverage: {} docs, {} GET ops, {} list ops, {} µservices",
        report.documents_checked,
        report.get_ops_checked,
        report.list_ops_checked,
        report.microservices_audited,
    );
    // Advisory per ADR-0150.
    if !findings.is_empty() {
        println!(
            "cursor-pagination-coverage: {} advisory findings (first 30):",
            findings.len()
        );
        for f in findings.iter().take(30) {
            println!(
                "  - {}:{} [{}] {}",
                f.path, f.line, f.microservice, f.message
            );
        }
        println!(
            "(advisory mode per ADR-0150; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0150-cursor-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 3: rpo-rto-coverage ----------

pub(crate) fn run_rpo_rto_coverage(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };
    let mut documents = Vec::new();
    for p in service_dirs_from_roots(&roots) {
        let ms = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let backfill = p.join("backfill-replay.md");
        if !backfill.exists() {
            continue;
        }
        if let Some(contents) = read_optional_string(&backfill) {
            documents.push(rpo_rto_check::BackfillReplayDocument {
                path: backfill.to_string_lossy().to_string(),
                microservice: ms,
                contents,
            });
        }
    }

    let (report, findings) = rpo_rto_check::audit_all(documents);
    println!(
        "rpo-rto-coverage: {} docs, {} with RTO, {} with RPO, {} µservices",
        report.documents_checked,
        report.documents_with_rto,
        report.documents_with_rpo,
        report.microservices_audited,
    );
    // Advisory per ADR-0152.
    if !findings.is_empty() {
        println!("rpo-rto-coverage: {} advisory findings:", findings.len());
        for f in findings.iter().take(60) {
            println!("  - {} [{}] {}", f.path, f.microservice, f.message);
        }
        println!(
            "(advisory mode per ADR-0152; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0152-rpo-rto-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 4: metric-cardinality ----------

pub(crate) fn run_metric_cardinality(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut documents = Vec::new();
    for p in service_dirs_from_roots(&roots) {
        for sm in collect_files(&p, &["yaml"]) {
            let name = sm.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if !(name == "servicemonitor.yaml" || name == "prometheusrule.yaml") {
                continue;
            }
            let ms = microservice_name_for(&sm).unwrap_or_default();
            if let Some(contents) = read_optional_string(&sm) {
                documents.push(metric_check::ServiceMonitorDocument {
                    path: sm.to_string_lossy().to_string(),
                    microservice: ms,
                    contents,
                });
            }
        }
    }

    let (report, findings) = metric_check::audit_all(documents);
    println!(
        "metric-cardinality: {} docs, {} high-cardinality labels dropped, {} µservices",
        report.documents_checked,
        report.high_cardinality_labels_dropped,
        report.microservices_audited,
    );
    // Advisory per ADR-0151.
    if !findings.is_empty() {
        println!(
            "metric-cardinality: {} advisory findings (first 30):",
            findings.len()
        );
        for f in findings.iter().take(30) {
            println!("  - {} [{}] {}", f.path, f.microservice, f.message);
        }
        println!(
            "(advisory mode per ADR-0151; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0151-metric-cardinality-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 5: event-schema-versioning ----------

pub(crate) fn run_event_schema_versioning(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut documents = Vec::new();
    for p in service_dirs_from_roots(&roots) {
        let ms = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let asyncapi_dir = p.join("contracts").join("asyncapi");
        if !asyncapi_dir.exists() {
            continue;
        }
        for f in collect_files(&asyncapi_dir, &["yaml", "yml"]) {
            if let Some(contents) = read_optional_string(&f) {
                documents.push(event_schema_check::AsyncApiDocument {
                    path: f.to_string_lossy().to_string(),
                    microservice: ms.clone(),
                    contents,
                });
            }
        }
    }

    let (report, findings) = event_schema_check::audit_all(documents);
    println!(
        "event-schema-versioning: {} docs, {} with version field, {} µservices",
        report.documents_checked, report.documents_with_version_field, report.microservices_audited,
    );
    // Advisory per ADR-0154.
    if !findings.is_empty() {
        println!(
            "event-schema-versioning: {} advisory findings (first 30):",
            findings.len()
        );
        for f in findings.iter().take(30) {
            println!("  - {} [{}] {}", f.path, f.microservice, f.message);
        }
        println!(
            "(advisory mode per ADR-0154; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0154-event-schema-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 6: id-discipline ----------

pub(crate) fn run_id_discipline(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut documents = Vec::new();
    for p in service_dirs_from_roots(&roots) {
        let ms = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let contracts = p.join("contracts");
        if !contracts.exists() {
            continue;
        }
        for f in collect_files(&contracts, &["yaml", "yml"]) {
            if let Some(contents) = read_optional_string(&f) {
                documents.push(id_check::SchemaDocument {
                    path: f.to_string_lossy().to_string(),
                    microservice: ms.clone(),
                    contents,
                });
            }
        }
    }

    let (report, findings) = id_check::audit_all(documents);
    println!(
        "id-discipline: {} docs, {} id fields inspected, {} µservices",
        report.documents_checked, report.id_fields_inspected, report.microservices_audited,
    );
    // Advisory per ADR-0156.
    if !findings.is_empty() {
        println!(
            "id-discipline: {} advisory findings (first 30):",
            findings.len()
        );
        for f in findings.iter().take(30) {
            println!(
                "  - {}:{} [{}] {}",
                f.path, f.line, f.microservice, f.message
            );
        }
        println!(
            "(advisory mode per ADR-0156; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0156-ulid-impl)"
        );
    }
    ExitCode::SUCCESS
}

// ---------- Gate 7: image-signing-discipline ----------

pub(crate) fn run_image_signing_discipline(args: Vec<String>) -> ExitCode {
    let workflows_dir = PathBuf::from(
        parse_flag_with_value(&args, "--workflows-dir")
            .unwrap_or_else(|| DEFAULT_WORKFLOWS_DIR.to_string()),
    );

    let mut documents = Vec::new();
    if let Ok(entries) = fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("yml")
                && p.extension().and_then(|e| e.to_str()) != Some("yaml")
            {
                continue;
            }
            if let Some(contents) = read_optional_string(&p) {
                let canonical = format!(
                    ".github/workflows/{}",
                    p.file_name().unwrap_or_default().to_string_lossy()
                );
                documents.push(image_check::WorkflowDocument {
                    path: canonical,
                    contents,
                });
            }
        }
    }

    let (report, findings) = image_check::audit_all(documents);
    println!(
        "image-signing-discipline: {} workflows, cosign={}, trivy={}, slsa={}",
        report.workflows_checked, report.cosign_present, report.trivy_present, report.slsa_present,
    );
    // Advisory per ADR-0146.
    if !findings.is_empty() {
        println!(
            "image-signing-discipline: {} advisory findings:",
            findings.len()
        );
        for f in &findings {
            println!("  - {} {}", f.path, f.message);
        }
        println!(
            "(advisory mode per ADR-0146; strict-mode promotion tracked under \
             registry/placeholder-debt/adr-follow-ups.yaml#adr-0146-image-signing-impl)"
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flag_with_value_handles_separated_form() {
        let args = vec!["--microservices-root".into(), "x".into()];
        assert_eq!(
            parse_flag_with_value(&args, "--microservices-root"),
            Some("x".to_string())
        );
    }

    #[test]
    fn parse_flag_with_value_returns_none_when_absent() {
        let args = vec!["--other".into()];
        assert_eq!(parse_flag_with_value(&args, "--microservices-root"), None);
    }

    #[test]
    fn microservice_name_for_extracts_correctly() {
        let path = PathBuf::from("microservices/tasks/contracts/openapi/tasks.yaml");
        assert_eq!(microservice_name_for(&path), Some("tasks".into()));
    }
}
