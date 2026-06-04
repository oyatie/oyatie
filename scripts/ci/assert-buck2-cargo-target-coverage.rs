//! Buck2/Cargo target-root coverage checker.
//!
//! This Rust checker replaces the retired Python AC-0.13 scanner. It is
//! local/static target-coverage evidence only: it measures Cargo workspace
//! target roots against checked-in Buck2
//! `crate_root` mappings. It does not run Cargo, generate source-line coverage,
//! mutate statuses, or prove protected-branch / Phase-0 authority.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_SPEC: &str = "specs/buck2-cargo-target-coverage.json";
const DEFAULT_CARGO_TOML: &str = "Cargo.toml";
const BUCK2_TARGET: &str = "//:buck2-cargo-target-coverage-check";
const CHECKER_PATH: &str = "scripts/ci/assert-buck2-cargo-target-coverage.rs";

const FALSE_CLAIMS: &[&str] = &[
    "source_line_coverage_generated",
    "mutation_lane_implemented",
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

const REQUIRED_URLS: &[&str] = &[
    "https://doc.rust-lang.org/cargo/reference/workspaces.html",
    "https://doc.rust-lang.org/cargo/reference/cargo-targets.html",
    "https://buck2.build/docs/users/commands/",
    "https://buck2.build/docs/about/bootstrapping/",
    "https://github.com/facebookincubator/reindeer",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoTarget {
    pub member: String,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub cargo_target_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub workspace_manifest: String,
    pub workspace_member_count: usize,
    pub cargo_target_root_count: usize,
    pub buck2_mapped_target_root_count: usize,
    pub buck_file_count: usize,
    pub known_divergence_count: usize,
    pub missing_mappings: Vec<CargoTarget>,
    pub unregistered_missing_target_roots: Vec<String>,
    pub stale_known_divergences: Vec<String>,
    pub sample_mappings: Vec<(String, Vec<String>)>,
}

#[derive(Debug, Default)]
struct Config {
    repo_root: PathBuf,
    cargo_toml: PathBuf,
    spec: PathBuf,
    json: bool,
}

#[derive(Debug, Default)]
struct Manifest {
    package_name: Option<String>,
    package_edition: Option<String>,
    autolib: Option<bool>,
    autobins: Option<bool>,
    has_manual_target: bool,
    lib_path: Option<String>,
    lib_name: Option<String>,
    bins: Vec<NamedPath>,
}

#[derive(Debug, Clone)]
struct NamedPath {
    name: Option<String>,
    path: String,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn quoted_values(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in text.chars() {
        if !in_string {
            if ch == '"' {
                in_string = true;
                current.clear();
            }
            continue;
        }
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => {
                in_string = false;
                values.push(current.clone());
            }
            _ => current.push(ch),
        }
    }
    values
}

fn parse_bool_value(raw: &str) -> Option<bool> {
    let value = raw.trim().trim_end_matches(',').trim();
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let cleaned = strip_comment(line).trim();
    let (key, value) = cleaned.split_once('=')?;
    Some((key.trim(), value.trim()))
}

fn extract_string_value(raw: &str) -> Option<String> {
    quoted_values(raw).into_iter().next()
}

fn read_text(path: &Path, failures: &mut Vec<String>, label: &str) -> String {
    match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{label}: read failed: {error}"));
            String::new()
        }
    }
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn has_string_value(text: &str, key: &str, value: &str) -> bool {
    compact_json_text(text).contains(&format!("\"{}\":\"{}\"", key, json_escape(value)))
}

fn contains_json_string(text: &str, value: &str) -> bool {
    text.contains(&json_string(value))
}

fn retired_checker_path() -> String {
    ["scripts/ci/", "assert-buck2-cargo-target-coverage", ".py"].concat()
}

fn parse_string_array_after_key(text: &str, key: &str) -> Vec<String> {
    let Some(start_key) = text.find(key) else {
        return Vec::new();
    };
    let after_key = &text[start_key + key.len()..];
    let Some(eq_index) = after_key.find('=') else {
        return Vec::new();
    };
    let after_eq = &after_key[eq_index + 1..];
    let Some(open) = after_eq.find('[') else {
        return Vec::new();
    };
    let mut depth = 0i32;
    let mut close_index = None;
    for (index, ch) in after_eq[open..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    close_index = Some(open + index + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    close_index
        .map(|end| quoted_values(&after_eq[open..end]))
        .unwrap_or_default()
}

fn workspace_members(root: &Path, cargo_toml: &Path, failures: &mut Vec<String>) -> Vec<String> {
    let text = read_text(cargo_toml, failures, "workspace manifest");
    if text.is_empty() {
        return Vec::new();
    }
    let raw_members = parse_string_array_after_key(&text, "members");
    let excludes: BTreeSet<PathBuf> = parse_string_array_after_key(&text, "exclude")
        .into_iter()
        .map(|item| root.join(item))
        .collect();
    let mut members = BTreeSet::new();
    for member in raw_members {
        if member.contains('*') || member.contains('?') || member.contains('[') {
            let matches = expand_simple_member_glob(root, &member);
            if matches.is_empty() {
                failures.push(format!("workspace_member_glob_matched_nothing:{member}"));
            }
            for path in matches {
                if !excludes.contains(&path) {
                    members.insert(rel(&path, root));
                }
            }
        } else {
            let path = root.join(&member);
            if !excludes.contains(&path) {
                members.insert(member);
            }
        }
    }
    members.into_iter().collect()
}

fn expand_simple_member_glob(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let prefix = pattern
        .split(['*', '?', '['])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    let start = if prefix.is_empty() {
        root
    } else {
        &root.join(prefix)
    };
    let mut dirs = Vec::new();
    collect_dirs_with_manifest(start, &mut dirs);
    dirs.into_iter()
        .filter(|path| simple_match(&rel(path, root), pattern))
        .collect()
}

fn collect_dirs_with_manifest(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.join("Cargo.toml").is_file() {
        out.push(dir.to_path_buf());
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if matches!(name.as_str(), ".git" | "buck-out" | "target") {
                continue;
            }
            collect_dirs_with_manifest(&path, out);
        }
    }
}

fn simple_match(value: &str, pattern: &str) -> bool {
    if pattern == value {
        return true;
    }
    if !pattern.contains('*') {
        return false;
    }
    let mut rest = value;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            continue;
        }
        if first && !pattern.starts_with('*') {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if let Some(index) = rest.find(part) {
            rest = &rest[index + part.len()..];
        } else {
            return false;
        }
        first = false;
    }
    pattern.ends_with('*') || rest.is_empty()
}

fn parse_manifest(text: &str) -> Manifest {
    let mut manifest = Manifest::default();
    let mut section = String::new();
    let mut current_bin: Option<usize> = None;

    for raw_line in text.lines() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("[[") && line.ends_with("]]") {
            section = line
                .trim_start_matches("[[")
                .trim_end_matches("]]")
                .trim()
                .to_owned();
            if section == "bin" {
                manifest.has_manual_target = true;
                manifest.bins.push(NamedPath {
                    name: None,
                    path: String::new(),
                });
                current_bin = Some(manifest.bins.len() - 1);
            } else {
                current_bin = None;
            }
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_owned();
            current_bin = None;
            if matches!(section.as_str(), "lib" | "example" | "test" | "bench") {
                manifest.has_manual_target = true;
            }
            continue;
        }
        let Some((key, value)) = parse_key_value(line) else {
            continue;
        };
        match section.as_str() {
            "package" => match key {
                "name" => manifest.package_name = extract_string_value(value),
                "edition" => manifest.package_edition = extract_string_value(value),
                "edition.workspace" => manifest.package_edition = Some("workspace".to_owned()),
                "autolib" => manifest.autolib = parse_bool_value(value),
                "autobins" => manifest.autobins = parse_bool_value(value),
                _ => {}
            },
            "lib" => match key {
                "path" => manifest.lib_path = extract_string_value(value),
                "name" => manifest.lib_name = extract_string_value(value),
                _ => {}
            },
            "bin" => {
                if let Some(index) = current_bin {
                    match key {
                        "name" => manifest.bins[index].name = extract_string_value(value),
                        "path" => {
                            manifest.bins[index].path =
                                extract_string_value(value).unwrap_or_default()
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    manifest.bins.retain(|bin| !bin.path.is_empty());
    manifest
}

fn manual_targets_defined(manifest: &Manifest) -> bool {
    manifest.has_manual_target || manifest.lib_path.is_some() || !manifest.bins.is_empty()
}

fn package_edition(manifest: &Manifest) -> &str {
    manifest.package_edition.as_deref().unwrap_or("2015")
}

fn auto_discovery_enabled(manifest: &Manifest, key: &str) -> bool {
    match key {
        "autolib" => {
            if let Some(value) = manifest.autolib {
                return value;
            }
        }
        "autobins" => {
            if let Some(value) = manifest.autobins {
                return value;
            }
        }
        _ => {}
    }
    if package_edition(manifest) == "2015" && manual_targets_defined(manifest) {
        return false;
    }
    true
}

fn discovered_src_bin_targets(member_dir: &Path) -> Vec<NamedPath> {
    let src_bin = member_dir.join("src/bin");
    let mut targets = Vec::new();
    let Ok(entries) = fs::read_dir(&src_bin) else {
        return targets;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let file = path.file_name().unwrap_or_default().to_string_lossy();
            targets.push(NamedPath {
                name: Some(name),
                path: format!("src/bin/{file}"),
            });
        } else if path.is_dir() && path.join("main.rs").is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            targets.push(NamedPath {
                name: Some(name.clone()),
                path: format!("src/bin/{name}/main.rs"),
            });
        }
    }
    targets.sort_by(|left, right| left.path.cmp(&right.path));
    targets
}

fn cargo_target_roots(member: &str, member_dir: &Path, manifest: &Manifest) -> Vec<CargoTarget> {
    let package_name = manifest.package_name.clone().unwrap_or_else(|| {
        member_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    let mut targets: Vec<(String, String, String)> = Vec::new();
    if let Some(path) = &manifest.lib_path {
        targets.push((
            "lib".to_owned(),
            manifest
                .lib_name
                .clone()
                .unwrap_or_else(|| package_name.clone()),
            path.clone(),
        ));
    } else if auto_discovery_enabled(manifest, "autolib") && member_dir.join("src/lib.rs").is_file()
    {
        targets.push((
            "lib".to_owned(),
            package_name.clone(),
            "src/lib.rs".to_owned(),
        ));
    }
    for (index, bin) in manifest.bins.iter().enumerate() {
        targets.push((
            "bin".to_owned(),
            bin.name.clone().unwrap_or_else(|| {
                if package_name.is_empty() {
                    format!("bin-{index}")
                } else {
                    package_name.clone()
                }
            }),
            bin.path.clone(),
        ));
    }
    if auto_discovery_enabled(manifest, "autobins") {
        if member_dir.join("src/main.rs").is_file() {
            targets.push((
                "bin".to_owned(),
                package_name.clone(),
                "src/main.rs".to_owned(),
            ));
        }
        for bin in discovered_src_bin_targets(member_dir) {
            targets.push((
                "bin".to_owned(),
                bin.name.unwrap_or_else(|| package_name.clone()),
                bin.path,
            ));
        }
    }

    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for (kind, name, path) in targets {
        if !seen.insert((kind.clone(), path.clone())) {
            continue;
        }
        let cargo_target_root = format!("{member}/{path}");
        result.push(CargoTarget {
            member: member.to_owned(),
            kind,
            name,
            path,
            cargo_target_root,
        });
    }
    result
}

fn collect_buck_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_buck_files_inner(root, &mut files);
    files.sort();
    files
}

fn collect_buck_files_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if matches!(name.as_str(), ".git" | "buck-out" | "target") {
                continue;
            }
            collect_buck_files_inner(&path, out);
        } else if name == "BUCK" {
            out.push(path);
        }
    }
}

fn buck_crate_roots(
    root: &Path,
    failures: &mut Vec<String>,
) -> (BTreeMap<String, Vec<String>>, usize) {
    let mut mappings: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let buck_files = collect_buck_files(root);
    for buck in &buck_files {
        let text = read_text(buck, failures, &format!("{}", rel(buck, root)));
        for line in text.lines() {
            let Some(index) = line.find("crate_root") else {
                continue;
            };
            let after = &line[index..];
            let Some(value) = extract_string_value(after) else {
                continue;
            };
            let target_root = buck.parent().unwrap_or(root).join(&value);
            let target_root = rel(&target_root, root);
            let mapping = format!("{}:{value}", rel(buck, root));
            mappings.entry(target_root).or_default().push(mapping);
        }
    }
    (mappings, buck_files.len())
}

fn validate_spec(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !has_bool(spec, "target_coverage_measured", true) {
        failures.push("target_coverage_measurement_not_recorded".to_owned());
    }
    for claim in FALSE_CLAIMS {
        if !has_bool(spec, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    if !has_string_value(spec, "buck2_target", BUCK2_TARGET) {
        failures.push("wrong_buck2_target".to_owned());
    }
    if !has_string_value(spec, "checker", CHECKER_PATH) {
        failures.push("wrong_checker_path".to_owned());
    }
    if spec.contains(&retired_checker_path()) {
        failures.push("retired_python_checker_path_present".to_owned());
    }
    if !has_string_value(spec, "workspace_manifest", "Cargo.toml") {
        failures.push("wrong_workspace_manifest".to_owned());
    }
    if !has_bool(spec, "parent_buck_allowed", true) {
        failures.push("parent_buck_mapping_not_allowed".to_owned());
    }
    if !spec.contains("crate_root") {
        failures.push("missing_crate_root_mapping_rule".to_owned());
    }
    if !spec.contains("autobins") || !spec.contains("src/bin") {
        failures.push("missing_cargo_bin_autodiscovery_rule".to_owned());
    }
    if !spec.contains("source-line coverage claims") {
        failures.push("source_line_claim_forbidden_authority_missing".to_owned());
    }
    if !spec.contains("protected branch authority") {
        failures.push("protected_branch_forbidden_authority_missing".to_owned());
    }
    for url in REQUIRED_URLS {
        if !contains_json_string(spec, url) {
            failures.push(format!("missing_official_reference_{url}"));
        }
    }
    if !spec.contains("buck2 build //:buck2-cargo-target-coverage-check") {
        failures.push("missing_buck2_target_in_automated_chain".to_owned());
    }
    failures
}

fn known_divergence_roots(spec: &str) -> (BTreeSet<String>, usize, Vec<String>) {
    let mut failures = Vec::new();
    let Some(index) = spec.find("\"known_divergences\"") else {
        return (BTreeSet::new(), 0, failures);
    };
    let after = &spec[index..];
    let Some(open) = after.find('[') else {
        return (BTreeSet::new(), 0, failures);
    };
    let mut depth = 0i32;
    let mut end = None;
    for (offset, ch) in after[open..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let body = end.map(|end| &after[open..end]).unwrap_or("[]");
    if body.trim() == "[]" {
        return (BTreeSet::new(), 0, failures);
    }
    let mut roots = BTreeSet::new();
    let count = body.matches("cargo_target_root").count();
    for segment in body.split("cargo_target_root") {
        if let Some(value) = extract_string_value(segment) {
            roots.insert(value);
        }
    }
    if count > 0 && (!body.contains("owner") || !body.contains("retirement_phase")) {
        failures.push("known_divergence_missing_owner_or_retirement".to_owned());
    }
    (roots, count, failures)
}

pub fn evaluate(root: &Path, cargo_toml_rel: &str, spec_rel: &str) -> Evaluation {
    let mut failures = Vec::new();
    let cargo_toml = root.join(cargo_toml_rel);
    let spec_path = root.join(spec_rel);
    let workspace_members = if cargo_toml.is_file() {
        workspace_members(root, &cargo_toml, &mut failures)
    } else {
        failures.push("missing_workspace_manifest".to_owned());
        Vec::new()
    };
    if workspace_members.is_empty() {
        failures.push("workspace_members_missing".to_owned());
    }

    let spec = if spec_path.is_file() {
        read_text(&spec_path, &mut failures, "contract spec")
    } else {
        failures.push("missing_contract_spec".to_owned());
        String::new()
    };
    failures.extend(validate_spec(&spec));
    let (known_by_root, known_divergence_count, known_failures) = known_divergence_roots(&spec);
    failures.extend(known_failures);

    let (buck_roots, buck_file_count) = buck_crate_roots(root, &mut failures);
    let mut cargo_targets = Vec::new();
    let mut missing_mappings = Vec::new();
    for member in &workspace_members {
        let member_dir = root.join(member);
        let member_manifest = member_dir.join("Cargo.toml");
        if !member_dir.is_dir() || !member_manifest.is_file() {
            failures.push(format!("workspace_member_path_missing:{member}"));
            continue;
        }
        let manifest_text = read_text(
            &member_manifest,
            &mut failures,
            &format!("{member}/Cargo.toml"),
        );
        let manifest = parse_manifest(&manifest_text);
        for mut target in cargo_target_roots(member, &member_dir, &manifest) {
            target.cargo_target_root = rel(&member_dir.join(&target.path), root);
            if !buck_roots.contains_key(&target.cargo_target_root) {
                missing_mappings.push(target.clone());
            }
            cargo_targets.push(target);
        }
    }

    let actual_missing: BTreeSet<String> = missing_mappings
        .iter()
        .map(|item| item.cargo_target_root.clone())
        .collect();
    let unregistered_missing_target_roots = actual_missing
        .difference(&known_by_root)
        .cloned()
        .collect::<Vec<_>>();
    let stale_known_divergences = known_by_root
        .difference(&actual_missing)
        .cloned()
        .collect::<Vec<_>>();
    if !unregistered_missing_target_roots.is_empty() {
        failures.push("missing_buck2_target_root_mapping".to_owned());
    }
    if !stale_known_divergences.is_empty() {
        failures.push("stale_known_divergence".to_owned());
    }

    let sample_mappings = cargo_targets
        .iter()
        .take(10)
        .map(|target| {
            (
                target.cargo_target_root.clone(),
                buck_roots
                    .get(&target.cargo_target_root)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .take(3)
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    let mapped_count = cargo_targets
        .iter()
        .filter(|target| buck_roots.contains_key(&target.cargo_target_root))
        .map(|target| target.cargo_target_root.clone())
        .collect::<BTreeSet<_>>()
        .len();
    failures.sort();
    failures.dedup();
    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        workspace_manifest: if cargo_toml.exists() {
            rel(&cargo_toml, root)
        } else {
            cargo_toml_rel.to_owned()
        },
        workspace_member_count: workspace_members.len(),
        cargo_target_root_count: cargo_targets.len(),
        buck2_mapped_target_root_count: mapped_count,
        buck_file_count,
        known_divergence_count,
        missing_mappings,
        unregistered_missing_target_roots,
        stale_known_divergences,
        sample_mappings,
    }
}

fn render_target(target: &CargoTarget) -> String {
    format!(
        "{{\"cargo_target_root\": {}, \"kind\": {}, \"member\": {}, \"name\": {}, \"path\": {}}}",
        json_string(&target.cargo_target_root),
        json_string(&target.kind),
        json_string(&target.member),
        json_string(&target.name),
        json_string(&target.path)
    )
}

fn render_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_sample_mappings(sample: &[(String, Vec<String>)]) -> String {
    let rows = sample
        .iter()
        .map(|(root, mappings)| {
            format!(
                "{{\"buck2_mappings\": {}, \"cargo_target_root\": {}}}",
                render_string_array(mappings),
                json_string(root)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", rows.join(", "))
}

fn render_json(evaluation: &Evaluation) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\": \"local/static AC-0.13 target-coverage measurement only; no source-line coverage, status mutation, protected-branch authority, P0.0 green, or Phase-0 completion proven\", ",
            "\"buck2_mapped_target_root_count\": {}, ",
            "\"buck_file_count\": {}, ",
            "\"cargo_target_root_count\": {}, ",
            "\"failures\": {}, ",
            "\"hyperscaler_grade\": false, ",
            "\"known_divergence_count\": {}, ",
            "\"live_required_context_execution_proven\": false, ",
            "\"missing_mappings\": {}, ",
            "\"mutation_lane_implemented\": false, ",
            "\"p0_0_green\": false, ",
            "\"phase0_complete\": false, ",
            "\"production_ready\": false, ",
            "\"protected_branch_authority_proven\": false, ",
            "\"sample_mappings\": {}, ",
            "\"source_line_coverage_generated\": false, ",
            "\"stale_known_divergences\": {}, ",
            "\"status_mutation_performed\": false, ",
            "\"target_coverage_measured\": {}, ",
            "\"unregistered_missing_target_roots\": {}, ",
            "\"verdict\": {}, ",
            "\"workspace_manifest\": {}, ",
            "\"workspace_member_count\": {}",
            "}}"
        ),
        evaluation.buck2_mapped_target_root_count,
        evaluation.buck_file_count,
        evaluation.cargo_target_root_count,
        render_string_array(&evaluation.failures),
        evaluation.known_divergence_count,
        format!(
            "[{}]",
            evaluation
                .missing_mappings
                .iter()
                .map(render_target)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        render_sample_mappings(&evaluation.sample_mappings),
        render_string_array(&evaluation.stale_known_divergences),
        if evaluation.failures.is_empty() {
            "true"
        } else {
            "false"
        },
        render_string_array(&evaluation.unregistered_missing_target_roots),
        json_string(&evaluation.verdict),
        json_string(&evaluation.workspace_manifest),
        evaluation.workspace_member_count,
    )
}

fn config() -> Config {
    let mut config = Config {
        repo_root: env::var_os("OYA_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        cargo_toml: PathBuf::from(DEFAULT_CARGO_TOML),
        spec: PathBuf::from(DEFAULT_SPEC),
        json: false,
    };
    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                config.repo_root = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()))
            }
            "--cargo-toml" => {
                config.cargo_toml =
                    PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_CARGO_TOML.to_owned()))
            }
            "--spec" => {
                config.spec = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_SPEC.to_owned()))
            }
            "--json" => config.json = true,
            unknown => {
                eprintln!("assert-buck2-cargo-target-coverage: unknown argument {unknown}");
                std::process::exit(2);
            }
        }
    }
    config
}

fn main() {
    let config = config();
    let root = config.repo_root;
    let cargo_toml = if config.cargo_toml.is_absolute() {
        rel(&config.cargo_toml, &root)
    } else {
        config.cargo_toml.to_string_lossy().into_owned()
    };
    let spec = if config.spec.is_absolute() {
        rel(&config.spec, &root)
    } else {
        config.spec.to_string_lossy().into_owned()
    };
    let evaluation = evaluate(&root, &cargo_toml, &spec);
    let rendered = render_json(&evaluation);
    if config.json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict != "PASS" {
        std::process::exit(1);
    }
}
