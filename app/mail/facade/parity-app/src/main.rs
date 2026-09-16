#![forbid(unsafe_code)]
mod mapping;
mod upstream;
mod view;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, process::Command};

const PIN: &str = "474dd0229cb20cf513036619781ed97bd8073c3f";

fn git(checkout: &Path, args: &[&str]) -> Result<String, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned());
    }
    String::from_utf8(result.stdout).map_err(|e| e.to_string())
}

fn inspect(checkout: &Path, revision: &str) -> Result<Value, String> {
    let commit = git(
        checkout,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )?
    .trim()
    .to_owned();
    let tree = git(checkout, &["rev-parse", &format!("{commit}^{{tree}}")])?
        .trim()
        .to_owned();
    let files = git(checkout, &["ls-tree", "-r", "--name-only", &commit])?;
    let mut crates = vec![];
    let mut unmapped = vec![];
    let mut seen = BTreeSet::new();
    for path in files.lines().filter(|p| {
        p.ends_with("/Cargo.toml") && (p.starts_with("crates/") || p.starts_with("tests/"))
    }) {
        let dir = path
            .strip_suffix("/Cargo.toml")
            .ok_or("invalid manifest path")?;
        let manifest = git(checkout, &["show", &format!("{commit}:{path}")])?;
        let manifest: toml::Value = toml::from_str(&manifest).map_err(|e| e.to_string())?;
        if manifest.get("package").is_none() {
            continue;
        }
        seen.insert(dir.to_owned());
        let Some(map) = mapping::mapping(dir) else {
            unmapped.push(dir.to_owned());
            continue;
        };
        let dependencies: Vec<_> = manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|t| t.iter())
            .filter_map(|(name, value)| {
                value
                    .get("path")
                    .and_then(toml::Value::as_str)
                    .map(|path| json!({"name":name,"path":path}))
            })
            .collect();
        let features: Vec<_> = manifest
            .get("features")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|t| t.keys())
            .cloned()
            .collect();
        crates.push(json!({"upstream":dir,"package":manifest["package"]["name"].as_str(),"source_tree":git(checkout,&["rev-parse",&format!("{commit}:{dir}")])?.trim(),
            "internal_dependencies":dependencies,"features":features,"manifest_license":manifest["package"].get("license").and_then(toml::Value::as_str),
            "implementation_status":"not-qualified", "mapping":{"core":map.core,"ports":map.ports,"adapters":map.adapters,"facade":map.facade,"owner":map.owner}}));
    }
    let removed: Vec<_> = mapping::MAPPINGS
        .iter()
        .filter(|m| !seen.contains(m.upstream))
        .map(|m| m.upstream)
        .collect();
    let mut tests = vec![];
    let mut enterprise = vec![];
    for path in files
        .lines()
        .filter(|p| p.ends_with(".rs") && (p.starts_with("tests/") || p.starts_with("crates/")))
    {
        if path.starts_with("tests/") {
            tests.push(json!({"path":path,"blob":git(checkout,&["rev-parse",&format!("{commit}:{path}")])?.trim(),"status":"not-qualified"}));
        }
        let source = git(checkout, &["show", &format!("{commit}:{path}")])?;
        if source.contains("feature = \"enterprise\"")
            || source.contains("is_enterprise_edition()")
            || source
                .lines()
                .take(12)
                .any(|l| l.contains("SPDX-License-Identifier: LicenseRef-SEL"))
        {
            enterprise.push(json!({"path":path,"blob":git(checkout,&["rev-parse",&format!("{commit}:{path}")])?.trim(),"status":"not-qualified"}));
        }
    }
    let mut inputs = vec![];
    for path in files.lines().filter(|p| {
        p.starts_with("tests/resources/")
            || p.starts_with("docs/")
            || matches!(*p, "README.md" | "Cargo.toml" | "Cargo.lock")
    }) {
        inputs.push(json!({"path":path,"blob":git(checkout,&["rev-parse",&format!("{commit}:{path}")])?.trim()}));
    }
    Ok(
        json!({"schema":1,"producer":"mail-parity-app","producer_sha256":format!("{:x}",Sha256::digest(std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?)),"repository":"https://github.com/stalwartlabs/stalwart","commit":commit,"tree":tree,"baseline_release":"v0.16.22","pinned_revision":PIN,
        "matches_pin":commit==PIN,"mapping_complete":unmapped.is_empty() && removed.is_empty(),"unmapped":unmapped,"removed":removed,
        "scope":{"community":true,"enterprise":true,"license_gating":false,"maturity":"unproven","ergonomics":"unproven","full_parity":false},
        "crates":crates,"test_files":tests,"enterprise_surfaces":enterprise,"compatibility_inputs":inputs}),
    )
}

fn compare(report: &Value, target: &Value) -> Result<Value, String> {
    let mut changes = vec![];
    for category in [
        "crates",
        "test_files",
        "enterprise_surfaces",
        "compatibility_inputs",
    ] {
        let key = if category == "crates" {
            "upstream"
        } else {
            "path"
        };
        let hash = if category == "crates" {
            "source_tree"
        } else {
            "blob"
        };
        let base = report[category].as_array().ok_or("invalid base report")?;
        let next = target[category].as_array().ok_or("invalid target report")?;
        for entry in next {
            let old = base.iter().find(|old| old[key] == entry[key]);
            if old.is_none_or(|old| old[hash] != entry[hash]) {
                changes.push(json!({"category":category,"path":entry[key],"change":if old.is_some(){"modified"}else{"added"}}));
            }
        }
        for entry in base {
            if !next.iter().any(|next| next[key] == entry[key]) {
                changes.push(json!({"category":category,"path":entry[key],"change":"removed"}));
            }
        }
    }
    Ok(
        json!({"base":report["commit"],"target":target["commit"],"changes":changes,"target_mapping_complete":target["mapping_complete"],"target_unmapped":target["unmapped"],"target_removed":target["removed"],"full_parity":false}),
    )
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3
        || args.len() > 5
        || !matches!(
            args[1].as_str(),
            "inspect" | "check" | "diff" | "refresh" | "map"
        )
    {
        return Err("usage: mail-parity-app inspect|check|map CHECKOUT [REVISION]; diff CHECKOUT BASE TARGET; refresh CHECKOUT".into());
    }
    let checkout = Path::new(&args[2]);
    if args[1] == "refresh" {
        return upstream::refresh(checkout);
    }
    let revision = args.get(3).map_or(PIN, String::as_str);
    let report = inspect(checkout, revision)?;
    if args[1] == "map" {
        return view::render(&report);
    }
    if args[1] == "diff" {
        let target = args.get(4).ok_or("diff requires BASE and TARGET")?;
        let target = inspect(checkout, target)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&compare(&report, &target)?).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
        if report["mapping_complete"] != true {
            return Err("upstream crate mapping changed; explicit classification required".into());
        }
        if args[1] == "check" {
            return Err("full parity not qualified: protocol, enterprise, ergonomics and maturity evidence remains incomplete".into());
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
