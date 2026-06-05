//! Retired grouping-wording gate.
//!
//! Rust/Buck2 replacement for the retired shell+Python scanner. The gate is
//! local/static: it scans checked-in text for retired suite/platform/module/
//! product wrapper wording and never mutates GitHub, Kubernetes, or disk state
//! beyond writing its Buck2 output artifact.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const FAILURE_MESSAGE: &str = "Retired grouping wording found in active files. Use flat service/lib/kernel/application/infrastructure/doc-set/test-set/eval-set naming and tenant/RBAC packaging instead.";
const PASS_MESSAGE: &str = "retired grouping wording gate passed";

const IGNORED_DIR_NAMES: &[&str] = &[
    ".git",
    ".omc",
    "buck-out",
    "node_modules",
    "target",
    "third-party",
    "vendor",
];
const IGNORED_PREFIXES: &[&str] = &[
    "docs/specs/",
    "evidence/",
    "registry/stub-audit/",
    "tasks/",
    "tools/agent-skills/",
    "crates/oya-llm-gateway-",
    "microservices/llm-gateway/",
    "docs/decisions/ADR-0373",
    "docs/decisions/ADR-0384",
];

const ALLOWED_EXACT_PATHS: &[&str] = &[
    "registry/catalog/oya-connector-netsuite-adapter.yaml",
    "scripts/ci/assert-retired-grouping-wording.rs",
    "scripts/tests/retired_grouping_wording_check.rs",
    "specs/products/RETIREMENT.md",
    "libs/oya-check-no-grouping/src/lib.rs",
    "ADR-INVENTORY.tsv",
    "microservices/connector/RETIREMENT-PLAN.md",
    "docs/ADR-LEGACY-REGRESSION-MAPPING.md",
    "docs/plans/rename-plan-v4-clean-arch-2026-05-13.md",
    "registry/milestone-audit/index.json",
    "registry/graph/architecture-map.json",
];

const ALLOWED_PREFIXES: &[&str] = &[
    "crates/oya-connector-netsuite-adapter",
    "docs/decisions/ADR-",
    "registry/stub-audit/",
];

const DEFAULT_ACTIVE_SCAN_PATHS: &[&str] = &[
    ".github/branch-protection.yaml",
    ".github/workflows/github-lane-unlocker-ci-cd.yml",
    "AGENTS.md",
    "BUCK",
    "CLAUDE.md",
    "README.md",
    "docs/AGENTS.md",
    "docs/DOC-CATALOG.md",
    "docs/MASTERPLAN.md",
    "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    "infra/branch-protection/dev.json",
    "libs/oya-check-no-grouping/src/lib.rs",
    "registry/repo-hygiene/python-shell-surface-inventory.json",
    "scripts/ci/assert-repo-hygiene-automation.rs",
    "scripts/ci/assert-retired-grouping-wording.rs",
    "scripts/tests/repo_hygiene_automation_check.rs",
    "scripts/tests/retired_grouping_wording_check.rs",
    "specs/agent-operating-contract.json",
    "specs/bespoke-cloud-toolchain-services.json",
    "specs/buck2-authority-policy.json",
    "specs/canonical-primitives.json",
    "specs/cloud-strangler-migration-target.json",
    "specs/cloud-toolchain-target.json",
    "specs/github-lane-unlocker-bridge.json",
    "specs/gitops-vcs-replacement.json",
    "specs/kubernetes-native-anti-patterns.json",
    "specs/master-plan-sequencing.json",
    "specs/masterplan.json",
    "specs/repo-hygiene-automation.json",
    "specs/root-hub-pointers.json",
    "specs/tenant-rbac-packaging.json",
];

const RETIRED_LITERAL_PATTERNS: &[&str] = &[
    "oya-enterprise-suite",
    "enterprise-suite",
    "connect-suite",
    "enterprise suite",
    "connect suite",
    "Enterprise Suite",
    "Connect Suite",
    "Productivity Suite",
    "Documentation Suite",
    "Doc Suite",
    "DocSuite",
    "doc-suite",
    "docsuite",
    "test-suite",
    "test_suite",
    "TestSuite",
    "eval_suite",
    "EvalSuite",
    "suite_id",
    "suite_boundary",
    "suite_shell",
    "suite_perimeter",
    "suite_activation",
    "suite-governance",
    "suite-storage",
    "suite-workflow",
    "suite_policy",
    "suite policy",
    "suite gateway",
    "suite shell",
    "suite perimeter",
    "oya-enterprise-platform",
    "enterprise-platform",
    "connect-platform",
    "enterprise platform",
    "connect platform",
    "Enterprise Platform",
    "Connect Platform",
    "EnterprisePlatform",
    "ConnectPlatform",
    "oya_enterprise_platform",
    "oya_connect_platform",
    "enterprise_platform",
    "connect_platform",
    "platform_id",
    "platform_boundary",
    "platform_shell",
    "platform_perimeter",
    "platform_activation",
    "platform-storage",
    "platform-workflow",
    "oya-enterprise-module",
    "enterprise-module",
    "connect-module",
    "healthcare-module",
    "enterprise module",
    "connect module",
    "healthcare module",
    "Enterprise Module",
    "Connect Module",
    "Healthcare Module",
    "EnterpriseModule",
    "ConnectModule",
    "HealthcareModule",
    "enterprise_module",
    "connect_module",
    "healthcare_module",
    "module_boundary",
    "module_shell",
    "module_perimeter",
    "module_activation",
    "module-governance",
    "module-storage",
    "module-workflow",
    "connect-product",
    "enterprise-product",
    "healthcare-product",
    "connect_product",
    "enterprise_product",
    "healthcare_product",
    "connect product",
    "enterprise product",
    "healthcare product",
    "Connect Product",
    "Enterprise Product",
    "Healthcare Product",
    "ProductPrd:Connect",
    "ProductPrd:Enterprise",
    "PRD-CONNECT",
    "PRD-ENTERPRISE",
    "specs/products/connect",
    "docs/products/connect",
    "connect.oyatie.dev",
    "connect.oyatie.com",
    "connect.oyatie.app",
    ".connect.oyatie.dev",
    ".connect.oyatie.com",
    ".connect.oyatie.app",
    "ns/connect/sa/connect-",
    "connect.svc.cluster.local",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: &'static str,
    pub root: PathBuf,
    pub matches: Vec<String>,
    pub files_scanned: usize,
}

fn repo_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn relpath(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root).map(repo_string).map_err(|error| {
        format!(
            "strip prefix {} from {}: {error}",
            root.display(),
            path.display()
        )
    })
}

fn is_ignored_prefix(rel: &str) -> bool {
    IGNORED_PREFIXES
        .iter()
        .any(|prefix| rel.starts_with(prefix))
}

fn is_allowed_path(rel: &str) -> bool {
    if ALLOWED_EXACT_PATHS.contains(&rel) {
        return true;
    }
    if ALLOWED_PREFIXES
        .iter()
        .any(|prefix| rel.starts_with(prefix))
    {
        return true;
    }
    if rel.starts_with("microservices/")
        && (rel.ends_with("/migration-from-connect.md") || rel.ends_with("/deprecation-notice.md"))
    {
        return true;
    }
    rel.starts_with("docs/architecture/corpus-rigor-audit-") && rel.ends_with(".md")
}

fn should_skip_dir(root: &Path, path: &Path) -> Result<bool, String> {
    let name_skipped = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| IGNORED_DIR_NAMES.contains(&name))
        .unwrap_or(false);
    if name_skipped {
        return Ok(true);
    }
    let rel = relpath(root, path)?;
    Ok(is_ignored_prefix(&(rel + "/")))
}

fn compact_without_ascii_whitespace(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn contains_json_or_yaml_value(line: &str, key: &str, values: &[&str]) -> bool {
    let compact = compact_without_ascii_whitespace(line);
    values.iter().any(|value| {
        let value = value.to_ascii_lowercase();
        [
            format!("\"{key}\":\"{value}\""),
            format!("{key}:\"{value}\""),
            format!("{key}:{value}"),
        ]
        .iter()
        .any(|needle| compact.contains(needle))
    })
}

fn contains_oya_connect_scalar(line: &str, key: &str) -> bool {
    let Some((_, value)) = line.split_once(key) else {
        return false;
    };
    let value = value.trim_start_matches(|ch: char| {
        ch.is_ascii_whitespace() || ch == ':' || ch == '"' || ch == '\''
    });
    let Some(tail) = value.strip_prefix("oya-connect") else {
        return false;
    };
    !tail.starts_with("or")
        && !tail
            .chars()
            .next()
            .map(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            .unwrap_or(false)
}

fn contains_oya_connect_dash_token(line: &str) -> bool {
    let Some((_, tail)) = line.split_once("oya-connect-") else {
        return false;
    };
    tail.chars().any(|ch| ch == '-')
}

pub fn line_has_retired_grouping_wording(line: &str) -> bool {
    if RETIRED_LITERAL_PATTERNS
        .iter()
        .any(|pattern| line.contains(pattern))
    {
        return true;
    }
    if line.ends_with("microservices/connect") || line.contains("microservices/connect/") {
        return true;
    }
    if contains_json_or_yaml_value(
        line,
        "product_class",
        &["suite", "platform", "platform-app", "module"],
    ) {
        return true;
    }
    if contains_json_or_yaml_value(line, "module_id", &["connect", "enterprise", "healthcare"]) {
        return true;
    }
    if contains_json_or_yaml_value(line, "product", &["connect", "enterprise", "healthcare"]) {
        return true;
    }
    if contains_oya_connect_scalar(line, "name") || contains_oya_connect_scalar(line, "repository")
    {
        return true;
    }
    contains_oya_connect_dash_token(line)
}

fn collect_matches(
    root: &Path,
    dir: &Path,
    matches: &mut Vec<String>,
    files_scanned: &mut usize,
) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| format!("read dir {}: {error}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read dir entry {}: {error}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {error}", path.display()))?;
        if file_type.is_dir() {
            if !should_skip_dir(root, &path)? {
                collect_matches(root, &path, matches, files_scanned)?;
            }
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let rel = relpath(root, &path)?;
        if is_ignored_prefix(&rel) || is_allowed_path(&rel) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        *files_scanned += 1;
        for (index, line) in text.lines().enumerate() {
            if line_has_retired_grouping_wording(line) {
                matches.push(format!("{}:{}:{}", rel, index + 1, line));
            }
        }
    }
    Ok(())
}

fn logical_rel_from_source(root: &Path, path: &Path) -> Result<String, String> {
    let normalized = repo_string(path);
    if let Some(index) = normalized.find("/srcs/") {
        return Ok(normalized[index + "/srcs/".len()..].to_string());
    }
    relpath(root, path)
}

fn collect_file_matches(
    root: &Path,
    path: &Path,
    matches: &mut Vec<String>,
    files_scanned: &mut usize,
) -> Result<(), String> {
    let rel = logical_rel_from_source(root, path)?;
    if is_ignored_prefix(&rel) || is_allowed_path(&rel) {
        return Ok(());
    }
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    *files_scanned += 1;
    for (index, line) in text.lines().enumerate() {
        if line_has_retired_grouping_wording(line) {
            matches.push(format!("{}:{}:{}", rel, index + 1, line));
        }
    }
    Ok(())
}

pub fn evaluate(root: &Path) -> Result<Evaluation, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize root {}: {error}", root.display()))?;
    let mut matches = Vec::new();
    let mut files_scanned = 0;
    collect_matches(&root, &root, &mut matches, &mut files_scanned)?;
    let verdict = if matches.is_empty() { "PASS" } else { "FAIL" };
    Ok(Evaluation {
        verdict,
        root,
        matches,
        files_scanned,
    })
}

pub fn evaluate_source_paths(root: &Path, source_paths: &[PathBuf]) -> Result<Evaluation, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize root {}: {error}", root.display()))?;
    let mut matches = Vec::new();
    let mut files_scanned = 0;
    for path in source_paths {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        if path.is_file() {
            collect_file_matches(&root, &path, &mut matches, &mut files_scanned)?;
        }
    }
    let verdict = if matches.is_empty() { "PASS" } else { "FAIL" };
    Ok(Evaluation {
        verdict,
        root,
        matches,
        files_scanned,
    })
}

pub fn evaluate_default_active_paths(root: &Path) -> Result<Evaluation, String> {
    let paths = DEFAULT_ACTIVE_SCAN_PATHS
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    evaluate_source_paths(root, &paths)
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

fn render_json(evaluation: &Evaluation) -> String {
    let matches = evaluation
        .matches
        .iter()
        .map(|line| format!("\"{}\"", json_escape(line)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"checker\":\"scripts/ci/assert-retired-grouping-wording.rs\",\"root\":\"{}\",\"local_static_only\":true,\"live_mutation_performed\":false,\"files_scanned\":{},\"matches\":[{}]}}",
        evaluation.verdict,
        json_escape(&repo_string(&evaluation.root)),
        evaluation.files_scanned,
        matches
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    root: PathBuf,
    source_env: Option<String>,
    full: bool,
    json: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut root = PathBuf::from(".");
    let mut source_env = None;
    let mut full = false;
    let mut json = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--full" => full = true,
            "--root" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--root requires a path".to_string());
                };
                root = PathBuf::from(value);
            }
            "--source-env" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--source-env requires an environment variable name".to_string());
                };
                source_env = Some(value.to_string());
            }
            "--help" | "-h" => {
                println!(
                    "Usage: assert-retired-grouping-wording [--root PATH] [--source-env NAME] [--full] [--json]"
                );
                process::exit(0);
            }
            value if value.starts_with('-') => return Err(format!("unknown argument: {value}")),
            value => root = PathBuf::from(value),
        }
        index += 1;
    }
    Ok(Args {
        root,
        source_env,
        full,
        json,
    })
}

fn run() -> Result<i32, String> {
    let args = parse_args(&env::args().collect::<Vec<_>>())?;
    let evaluation = if let Some(source_env) = &args.source_env {
        let value = env::var(source_env).map_err(|error| format!("read ${source_env}: {error}"))?;
        let paths = value
            .split_whitespace()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        evaluate_source_paths(&args.root, &paths)?
    } else if args.full {
        evaluate(&args.root)?
    } else {
        evaluate_default_active_paths(&args.root)?
    };
    if args.json {
        println!("{}", render_json(&evaluation));
    } else if evaluation.matches.is_empty() {
        println!("{PASS_MESSAGE}");
    } else {
        eprintln!("{FAILURE_MESSAGE}");
        eprintln!("{}", evaluation.matches.join("\n"));
    }
    Ok(if evaluation.matches.is_empty() { 0 } else { 1 })
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("assert-retired-grouping-wording failed: {error}");
            process::exit(2);
        }
    }
}
