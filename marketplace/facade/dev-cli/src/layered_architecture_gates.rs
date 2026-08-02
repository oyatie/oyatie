//! Gate dispatcher wiring for ADR-0148 / ADR-0182 / ADR-0183 / ADR-0184 /
//! ADR-0185 layered-architecture discipline + ADR-0185 client-stack
//! discipline.
//!
//! Two gates land here:
//!
//! 1. `layered-architecture-discipline` — strict; ADR-0148/0182/0183/0184.
//! 2. `client-stack-discipline` — strict; ADR-0185.
//!
//! The runner I/O scans the canonical default paths in the repo
//! (`cloud/*/manifest.json`, `oya/*/manifest.json`, and `microservices/*/manifest.json`
//! for the layered gate; recursive `client-manifest.json` discovery under the
//! same roots for client-stack discipline). Arguments are accepted but optional.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_client_stack_discipline as client_check;
use check_layered_architecture_discipline as layered_check;

/// Canonical service roots scanned when no explicit `--microservices-root` is given.
const DEFAULT_SERVICE_ROOTS: &[&str] = &["cloud", "oya", "microservices"];
const DEFAULT_DEFERRED_VIOLATIONS: &str =
    "registry/layered-architecture-discipline/wave-3-i-deferred-manifest-violations.tsv";

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

fn microservice_name_for(path: &Path, marker: &str) -> Option<String> {
    let mut iter = path.components();
    while let Some(c) = iter.next() {
        if c.as_os_str() == std::ffi::OsStr::new(marker)
            && let Some(next) = iter.next()
        {
            return Some(next.as_os_str().to_string_lossy().to_string());
        }
    }
    None
}

fn list_manifest_paths_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest = dir.join("manifest.json");
            if manifest.exists() {
                out.push(manifest);
            }
        }
    }
    out
}

fn walk_client_manifests_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        collect_client_manifests(root, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

fn collect_client_manifests(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_client_manifests(&path, out);
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("client-manifest.json") {
            out.push(path);
        }
    }
}

pub(crate) fn run_layered_architecture_discipline(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };
    let deferred_path = PathBuf::from(
        parse_flag_with_value(&args, "--deferred-violations")
            .unwrap_or_else(|| DEFAULT_DEFERRED_VIOLATIONS.to_string()),
    );

    let mut manifests = Vec::new();
    for path in list_manifest_paths_from_roots(&roots) {
        // Extract service name from any root marker (cloud, oya, microservices).
        let microservice = microservice_name_for(&path, "cloud")
            .or_else(|| microservice_name_for(&path, "oya"))
            .or_else(|| microservice_name_for(&path, "microservices"))
            .unwrap_or_default();
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        manifests.push(layered_check::ManifestDocument {
            microservice,
            path: path.to_string_lossy().to_string(),
            contents,
        });
    }

    let (report, violations) = layered_check::audit_all_violations(manifests);
    let deferred = match read_deferred_layered_violations(&deferred_path) {
        Ok(deferred) => deferred,
        Err(error) => {
            eprintln!("layered-architecture-discipline FAILED: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut deferred_count = 0usize;
    let active_violations = violations
        .into_iter()
        .filter(|violation| {
            let key = (
                violation.microservice.clone(),
                format!("{:?}", violation.kind),
            );
            let is_deferred = deferred.contains(&key);
            if is_deferred {
                deferred_count += 1;
            }
            !is_deferred
        })
        .collect::<Vec<_>>();
    if active_violations.is_empty() {
        println!(
            "layered-architecture-discipline passed: {} manifests, {} µservices, {} gateway-owners, {} waypoint-enrolled, {} deferred Wave-3-I manifest violations",
            report.manifests_checked,
            report.microservices_audited,
            report.gateway_owners_detected,
            report.waypoint_enrolled_count,
            deferred_count,
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "layered-architecture-discipline FAILED: {} violations",
            active_violations.len()
        );
        for v in &active_violations {
            eprintln!("  - {}", v);
        }
        ExitCode::FAILURE
    }
}

fn read_deferred_layered_violations(path: &Path) -> Result<BTreeSet<(String, String)>, String> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("deferral registry unreadable {}: {error}", path.display()))?;
    let mut records = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(format!(
                "{}:{} deferral row must be <microservice><tab><violation_kind><tab><reason>",
                path.display(),
                index + 1
            ));
        }
        let microservice = fields[0].trim();
        let kind = fields[1].trim();
        let reason = fields[2].trim();
        if microservice.is_empty() || kind.is_empty() || reason.is_empty() {
            return Err(format!(
                "{}:{} deferral row fields must be non-empty",
                path.display(),
                index + 1
            ));
        }
        match kind {
            "GatewayAndMeshConflict"
            | "CedarAndKyvernoConflict"
            | "CacheBackendConflict"
            | "MeshTierUnderclaimed"
            | "NorthSouthOnlyMisplaced" => {}
            _ => {
                return Err(format!(
                    "{}:{} unknown layered-architecture violation kind {kind:?}",
                    path.display(),
                    index + 1
                ));
            }
        }
        if !records.insert((microservice.to_owned(), kind.to_owned())) {
            return Err(format!(
                "{}:{} duplicate deferral for {microservice}/{kind}",
                path.display(),
                index + 1
            ));
        }
    }
    Ok(records)
}

pub(crate) fn run_client_stack_discipline(args: Vec<String>) -> ExitCode {
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut manifests = Vec::new();
    for path in walk_client_manifests_from_roots(&roots) {
        // Surface is the parent dir name (e.g. "web-sveltekit", "apple-ios").
        let surface = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        manifests.push(client_check::ClientManifest {
            surface,
            path: path.to_string_lossy().to_string(),
            contents,
        });
    }

    let (report, violations) = client_check::audit_all_violations(manifests);
    if violations.is_empty() {
        println!(
            "client-stack-discipline passed: {} client-manifests, {} surfaces",
            report.manifests_checked, report.surfaces_audited
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "client-stack-discipline FAILED: {} violations",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {}", v);
        }
        ExitCode::FAILURE
    }
}
