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
//! (`microservices/*/manifest.json` for the layered gate;
//! `microservices/*/clients/*/client-manifest.json` for the client-stack
//! gate). Arguments are accepted but optional.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_check_client_stack_discipline as client_check;
use oya_check_layered_architecture_discipline as layered_check;

const DEFAULT_MICROSERVICES_ROOT: &str = "microservices";

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

fn list_manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
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
    out
}

fn walk_client_manifests(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(ms_entries) = fs::read_dir(root) else {
        return out;
    };
    for ms_entry in ms_entries.flatten() {
        let ms_path = ms_entry.path();
        if !ms_path.is_dir() {
            continue;
        }
        let clients = ms_path.join("clients");
        if !clients.is_dir() {
            continue;
        }
        let Ok(surfaces) = fs::read_dir(&clients) else {
            continue;
        };
        for surface_entry in surfaces.flatten() {
            let surface_path = surface_entry.path();
            if !surface_path.is_dir() {
                continue;
            }
            let manifest = surface_path.join("client-manifest.json");
            if manifest.exists() {
                out.push(manifest);
            }
        }
    }
    out
}

pub(crate) fn run_layered_architecture_discipline(args: Vec<String>) -> ExitCode {
    let root = PathBuf::from(
        parse_flag_with_value(&args, "--microservices-root")
            .unwrap_or_else(|| DEFAULT_MICROSERVICES_ROOT.to_string()),
    );

    let mut manifests = Vec::new();
    for path in list_manifest_paths(&root) {
        let microservice = microservice_name_for(&path, "microservices").unwrap_or_default();
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
    if violations.is_empty() {
        println!(
            "layered-architecture-discipline passed: {} manifests, {} µservices, {} gateway-owners, {} waypoint-enrolled",
            report.manifests_checked,
            report.microservices_audited,
            report.gateway_owners_detected,
            report.waypoint_enrolled_count,
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "layered-architecture-discipline FAILED: {} violations",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {}", v);
        }
        ExitCode::FAILURE
    }
}

pub(crate) fn run_client_stack_discipline(args: Vec<String>) -> ExitCode {
    let root = PathBuf::from(
        parse_flag_with_value(&args, "--microservices-root")
            .unwrap_or_else(|| DEFAULT_MICROSERVICES_ROOT.to_string()),
    );

    let mut manifests = Vec::new();
    for path in walk_client_manifests(&root) {
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
