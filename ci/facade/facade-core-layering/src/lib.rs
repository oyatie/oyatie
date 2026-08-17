//! ADR-0562's central layering rule, enforced: a `facade` crate reaches its own capability's
//! `core` only through `ports`.
//!
//! The rule has been stated since ADR-0562 was accepted and gated by nothing, so 35 facade
//! packages violate it. This freezes those 35 and makes a NEW one impossible to ship.
//!
//! Two design choices carry the weight:
//!
//! 1. **Detection parses `BUCK`, not `Cargo.toml`.** A manifest scan does not reproduce the
//!    build graph: `intelligence/facade/worker` carries the edge in `BUCK` with **zero** Cargo
//!    path-deps, so a manifest-keyed gate is blind to it. The static `BUCK` parse here was
//!    verified against the authoritative `buck2 uquery` result — 35 packages, identical
//!    per-capability split — before this gate was written.
//!
//! 2. **Baseline keys are cargo package names, not paths or buck2 target labels.** A buck2 label
//!    embeds the path (`//intelligence/facade/worker:…`), so a future capability move would
//!    invalidate every baselined entry at once and the repair would be indistinguishable from
//!    laundering. Package names survive relocation.
//!
//! Hermetic: filesystem reads only. No subprocess, network, clock, or randomness.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

pub const CODE_DIRECT_DEP: &str = "facade_core_direct_dep";
pub const CODE_NO_PORTS: &str = "facade_core_no_ports_layer";

/// Every code this gate can emit. Pinned by test against the policy's declared set so a code
/// added in code but not declared in data (or vice versa) fails closed rather than going unseen.
pub const DECLARED_CODES: &[&str] = &[CODE_DIRECT_DEP, CODE_NO_PORTS];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

#[derive(Debug)]
pub enum ScanError {
    Io(String),
    Policy(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::Io(m) => write!(f, "io: {m}"),
            ScanError::Policy(m) => write!(f, "policy: {m}"),
        }
    }
}

/// Extract the absolute buck2 labels a build file references, as `(cell_dir, face, package)`.
///
/// Deliberately literal: only absolute labels of the form `//<cap>/<face>/<pkg>:<target>` (with
/// or without a `root` cell prefix) are considered. Relative labels (`:target`) cannot cross a
/// face boundary, so they are irrelevant to this rule, and ignoring them keeps the parse honest
/// rather than clever.
fn absolute_labels(buildfile: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for raw in string_literals(buildfile) {
        let body = raw
            .strip_prefix("root//")
            .or_else(|| raw.strip_prefix("//"));
        let Some(body) = body else { continue };
        let Some((path, _target)) = body.split_once(':') else {
            continue;
        };
        let mut parts = path.split('/');
        let (Some(cap), Some(face), Some(pkg)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if cap.is_empty() || face.is_empty() || pkg.is_empty() {
            continue;
        }
        out.push((cap.to_owned(), face.to_owned(), pkg.to_owned()));
    }
    out
}

/// Double-quoted string literals, skipping `#` comments so a commented-out dep is not read as a
/// live edge. (The same comment-blindness bug was found and fixed in gate-self-conformance.)
fn string_literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let code = match line.find('#') {
            // A `#` inside a string is not a comment; only treat it as one if no unclosed quote
            // precedes it.
            Some(idx) if line[..idx].matches('"').count() % 2 == 0 => &line[..idx],
            _ => line,
        };
        let mut rest = code;
        while let Some(open) = rest.find('"') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('"') else { break };
            out.push(after[..close].to_owned());
            rest = &after[close + 1..];
        }
    }
    out
}

fn cargo_package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if in_package {
            if let Some((k, v)) = t.split_once('=') {
                if k.trim() == "name" {
                    return Some(v.trim().trim_matches('"').to_owned());
                }
            }
        }
    }
    None
}

fn dir_names(path: &Path) -> Result<Vec<String>, ScanError> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(ScanError::Io(format!("read_dir {}: {e}", path.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| ScanError::Io(format!("entry in {}: {e}", path.display())))?;
        if entry.path().is_dir() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// Walk the candidate tree and emit the observed violation set.
///
/// A capability is any top-level directory containing a `facade/` subtree. `capabilities_scanned`
/// is reported so a scan that enumerates nothing is distinguishable from a tree that is clean —
/// the two are otherwise identical at the finding level, and the first is a broken gate.
pub fn collect(repo_root: &Path, policy: &Value) -> Result<Value, ScanError> {
    let faces = policy
        .get("faces")
        .ok_or_else(|| ScanError::Policy("missing `faces`".to_owned()))?;
    let facade = faces
        .get("facade")
        .and_then(Value::as_str)
        .ok_or_else(|| ScanError::Policy("missing `faces.facade`".to_owned()))?;
    let core = faces
        .get("core")
        .and_then(Value::as_str)
        .ok_or_else(|| ScanError::Policy("missing `faces.core`".to_owned()))?;
    let ports = faces
        .get("ports")
        .and_then(Value::as_str)
        .ok_or_else(|| ScanError::Policy("missing `faces.ports`".to_owned()))?;
    let buildfile = policy
        .get("scan")
        .and_then(|s| s.get("buildfile"))
        .and_then(Value::as_str)
        .ok_or_else(|| ScanError::Policy("missing `scan.buildfile`".to_owned()))?;

    let mut violations: Vec<Value> = Vec::new();
    let mut capabilities_scanned = 0usize;
    let mut facade_packages_scanned = 0usize;

    for cap in dir_names(repo_root)? {
        let cap_dir = repo_root.join(&cap);
        let facade_dir = cap_dir.join(facade);
        if !facade_dir.is_dir() {
            continue;
        }
        capabilities_scanned += 1;
        let has_ports = !dir_names(&cap_dir.join(ports))?.is_empty();

        for pkg in dir_names(&facade_dir)? {
            let pkg_dir = facade_dir.join(&pkg);
            let build_path = pkg_dir.join(buildfile);
            let Ok(text) = fs::read_to_string(&build_path) else {
                continue;
            };
            facade_packages_scanned += 1;

            let reaches_core = absolute_labels(&text)
                .into_iter()
                .any(|(c, face, _)| c == cap && face == core);
            if !reaches_core {
                continue;
            }

            // Fail closed: a facade package that violates the rule but whose manifest cannot be
            // read is reported under a synthetic key rather than silently dropped. Dropping it
            // would let a violator hide by being unreadable.
            let key = fs::read_to_string(pkg_dir.join("Cargo.toml"))
                .ok()
                .and_then(|m| cargo_package_name(&m))
                .unwrap_or_else(|| format!("<unresolved-package>:{cap}/{facade}/{pkg}"));

            violations.push(json!({
                "code": if has_ports { CODE_DIRECT_DEP } else { CODE_NO_PORTS },
                "key": key,
                "capability": cap,
            }));
        }
    }

    violations.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
    Ok(json!({
        "violations": violations,
        "capabilities_scanned": capabilities_scanned,
        "facade_packages_scanned": facade_packages_scanned,
    }))
}

fn baseline_set(policy: &Value, code: &str) -> BTreeSet<String> {
    policy
        .get("frozen_baseline")
        .and_then(|b| b.get(code))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Pure verdict: any observed violation NOT in its code's frozen baseline is a regression.
///
/// Shrink-only by construction. A baselined entry that disappears is a repair, never a finding —
/// but a baselined entry that no longer exists in the tree IS reported, because a stale baseline
/// silently widens what the gate tolerates.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let details: BTreeMap<&str, String> = DECLARED_CODES
        .iter()
        .map(|c| {
            let d = policy
                .get("codes")
                .and_then(|codes| codes.get(*c))
                .and_then(|entry| entry.get("detail"))
                .and_then(Value::as_str)
                .unwrap_or("<missing policy detail>")
                .to_owned();
            (*c, d)
        })
        .collect();

    let mut seen: BTreeMap<&str, BTreeSet<String>> = DECLARED_CODES
        .iter()
        .map(|c| (*c, BTreeSet::new()))
        .collect();

    for row in observed
        .get("violations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let (Some(code), Some(key)) = (
            row.get("code").and_then(Value::as_str),
            row.get("key").and_then(Value::as_str),
        ) else {
            continue;
        };
        if let Some(bucket) = seen.get_mut(code) {
            bucket.insert(key.to_owned());
        }
        if !baseline_set(policy, code).contains(key) {
            findings.insert(Finding {
                code: code.to_owned(),
                key: key.to_owned(),
                detail: details.get(code).cloned().unwrap_or_default(),
            });
        }
    }

    for code in DECLARED_CODES {
        let live = seen.get(code).cloned().unwrap_or_default();
        for stale in baseline_set(policy, code).difference(&live) {
            findings.insert(Finding {
                code: (*code).to_owned(),
                key: stale.clone(),
                detail: format!(
                    "baseline entry no longer present in the tree; remove it from \
                     frozen_baseline.{code} so the gate cannot tolerate a re-introduction"
                ),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "faces": {"facade": "facade", "core": "core", "ports": "ports"},
            "scan": {"buildfile": "BUCK"},
            "codes": {
                CODE_DIRECT_DEP: {"detail": "direct"},
                CODE_NO_PORTS: {"detail": "no ports"},
            },
            "frozen_baseline": {CODE_DIRECT_DEP: ["known-violator"], CODE_NO_PORTS: []},
        })
    }

    #[test]
    fn a_new_violation_is_born_blocking() {
        let observed = json!({"violations": [
            {"code": CODE_DIRECT_DEP, "key": "known-violator"},
            {"code": CODE_DIRECT_DEP, "key": "brand-new-violator"},
        ]});
        let f = evaluate_keyed(&policy(), &observed);
        assert_eq!(f.len(), 1, "{f:#?}");
        assert_eq!(f.iter().next().unwrap().key, "brand-new-violator");
    }

    #[test]
    fn a_repaired_baseline_entry_is_reported_so_the_baseline_shrinks() {
        // The baselined violator is gone from the tree. That is a repair, and the gate must ask
        // for the baseline row to be removed — otherwise the slot silently stays open for a
        // re-introduction under the same package name.
        let f = evaluate_keyed(&policy(), &json!({"violations": []}));
        assert_eq!(f.len(), 1);
        let only = f.iter().next().unwrap();
        assert_eq!(only.key, "known-violator");
        assert!(only.detail.contains("no longer present"), "{only:?}");
    }

    #[test]
    fn the_two_codes_do_not_share_a_baseline() {
        // Same key, other code: must NOT be absolved by the first code's baseline.
        let observed = json!({"violations": [{"code": CODE_NO_PORTS, "key": "known-violator"}]});
        let codes: BTreeSet<_> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains(CODE_NO_PORTS), "{codes:?}");
    }

    #[test]
    fn labels_are_parsed_only_when_absolute_and_uncommented() {
        let text = r#"
            rust_library(
                name = "x",
                deps = [
                    "//iam/core/policy-kernel:policy-kernel",
                    "root//iam/core/other:other",
                    ":sibling",
                    # "//iam/core/commented-out:nope",
                    "third-party//:serde_json",
                ],
            )
        "#;
        let labels = absolute_labels(text);
        let cores: Vec<_> = labels
            .iter()
            .filter(|(c, f, _)| c == "iam" && f == "core")
            .map(|(_, _, p)| p.as_str())
            .collect();
        assert_eq!(cores, vec!["policy-kernel", "other"], "{labels:?}");
    }

    #[test]
    fn an_unreadable_manifest_fails_closed_rather_than_dropping_the_violator() {
        let dir = std::env::temp_dir().join("facade-core-layering-no-manifest");
        let pkg = dir.join("iam/facade/ghost");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&pkg).unwrap();
        fs::create_dir_all(dir.join("iam/ports/p")).unwrap();
        fs::write(pkg.join("BUCK"), "deps = [\"//iam/core/k:k\"]").unwrap();
        let observed = collect(&dir, &policy()).unwrap();
        let rows = observed["violations"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{observed:#?}");
        assert!(
            rows[0]["key"]
                .as_str()
                .unwrap()
                .starts_with("<unresolved-package>:"),
            "{rows:#?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
