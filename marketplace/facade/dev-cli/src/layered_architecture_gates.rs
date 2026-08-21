//! Gate dispatcher wiring for ADR-0148 / ADR-0182 / ADR-0183 / ADR-0184 /
//! ADR-0185 layered-architecture discipline + ADR-0185 client-stack
//! discipline.
//!
//! Two gates land here:
//!
//! 1. `layered-architecture-discipline` — strict; ADR-0148/0182/0183/0184.
//! 2. `client-stack-discipline` — strict; ADR-0185.
//!
//! The runner I/O scans the service roots derived by `crate::service_roots`
//! from the closed capability registry (`<root>/manifest.json` and
//! `<root>/<service>/manifest.json` for the layered gate; recursive
//! `client-manifest.json` discovery under the same roots for client-stack
//! discipline). Arguments are accepted but optional.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_client_stack_discipline as client_check;
use check_layered_architecture_discipline as layered_check;

use crate::service_roots::ServiceSubpath;

const DEFAULT_DEFERRED_VIOLATIONS: &str =
    "registry/layered-architecture-discipline/wave-3-i-deferred-manifest-violations.tsv";

/// Resolve the roots this gate scans: the explicit `--microservices-root`
/// when given, otherwise the shared registry-derived default set. See
/// `crate::service_roots` for why the default set is derived rather than
/// hardcoded, and why an absent root is an error.
fn resolve_roots(args: &[String]) -> Result<Vec<PathBuf>, String> {
    match parse_flag_with_value(args, "--microservices-root") {
        Some(explicit) => Ok(vec![PathBuf::from(explicit)]),
        None => crate::service_roots::default_service_roots(),
    }
}

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

/// Every `manifest.json` under the given roots, in BOTH layout shapes,
/// paired with the microservice name the gate keys on.
///
/// The predecessor walked depth 2 only and then recovered the name by
/// searching the path for a literal `cloud` / `oya` / `microservices`
/// component. Two of those three markers no longer exist in the tree, so
/// every manifest outside `oya/` fell through to `unwrap_or_default()` —
/// the empty string.
fn list_manifests_from_roots(roots: &[PathBuf]) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();
    for root in roots {
        out.extend(crate::service_roots::list_service_files(
            root,
            "manifest.json",
        ));
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
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("layered-architecture-discipline: {error}");
            return ExitCode::FAILURE;
        }
    };
    let deferred_path = PathBuf::from(
        parse_flag_with_value(&args, "--deferred-violations")
            .unwrap_or_else(|| DEFAULT_DEFERRED_VIOLATIONS.to_string()),
    );

    let mut manifests = Vec::new();
    for manifest in list_manifests_from_roots(&roots) {
        let Ok(contents) = fs::read_to_string(&manifest.path) else {
            continue;
        };
        manifests.push(layered_check::ManifestDocument {
            microservice: crate::service_roots::declared_microservice(
                &contents,
                manifest.microservice,
            ),
            path: manifest.path.to_string_lossy().to_string(),
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
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("client-stack-discipline: {error}");
            return ExitCode::FAILURE;
        }
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
