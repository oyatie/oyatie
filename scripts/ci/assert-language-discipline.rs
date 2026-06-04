//! AC-0.4 language-discipline seed gate.
//!
//! This checker is intentionally Rust-first for the new gate surface. It reads a
//! small machine-readable registry plus checked-in GOOD/BAD fixtures, blocks new
//! candidate-authored Python/shell sprawl outside the allowlist, and keeps all
//! live-authority/readiness claims false. It is local/static evidence only.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/language-discipline-registry.json";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-language-discipline/tc-0.4-good-allowlisted-bootstrap-shell-edit.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-python-under-scripts.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-shell-test-sprawl.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-good-non-script-change.json",
];

const BACKLOG_OFFENDERS: &[&str] = &[
    "scripts/tests/cloud_control_plane_operation_contract_check.py",
    "scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py",
    "scripts/tests/cloud_enforceability_facets_check.py",
    "scripts/tests/cloud_hyperscaler_parity_taxonomy_check.py",
    "scripts/tests/cloud_observability_slo_evidence_check.py",
    "scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py",
    "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
];

const ACTIVE_SCRIPT_ROOT: &str = "scripts";

const SELF_EXPLANATORY_SCRIPT_NAME_ROOTS: &[&str] = &["scripts", "tools"];

const RETIRED_CLI_SCRIPT_INVOCATION_TOKENS: &[&str] = &[
    "//oya/developer-sdk/crates/oya-dev-cli:oya",
    "oya-dev-cli",
    "oya gate ",
    "oya verify ",
    "oya vcs ",
    "oya git ",
    "./bin/oya",
];

const ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG: &[&str] = &[
    "scripts/asyncapi-lint.mjs",
    "scripts/proto-lint.mjs",
    "scripts/validate-adr-shape.mjs",
    "scripts/validate-foundry-phase00-evidence.mjs",
    "scripts/branch-protection-apply.sh",
    "scripts/onprem-bring-up.sh",
    "scripts/install-trivy-ci.sh",
    "scripts/validate-release-image-supply-chain.sh",
];

const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub expected: String,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub registry: String,
    pub fixture_count: usize,
    pub backlog_offender_count: usize,
    pub active_retired_cli_script_count: usize,
    pub fixture_results: Vec<FixtureResult>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub registry: String,
    pub json: bool,
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

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn required_bool_token(key: &str, value: bool) -> String {
    format!("\"{}\":{}", key, if value { "true" } else { "false" })
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&required_bool_token(key, value))
}

fn value_after_key(chunk: &str, key: &str) -> Option<String> {
    let key_token = format!("\"{}\"", key);
    let after_key = chunk.split_once(&key_token)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let mut value = String::new();
    let mut escaped = false;
    for ch in after_quote.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            _ => value.push(ch),
        }
    }
    None
}

pub fn changed_files_from_fixture(text: &str) -> Vec<ChangedFile> {
    text.split('{')
        .filter_map(|chunk| {
            if !chunk.contains("\"path\"") || !chunk.contains("\"status\"") {
                return None;
            }
            let path = value_after_key(chunk, "path")?;
            let status = value_after_key(chunk, "status")?;
            Some(ChangedFile { path, status })
        })
        .collect()
}

fn is_new_status(status: &str) -> bool {
    matches!(status, "A" | "AM" | "??")
}

fn is_allowlisted_shell(path: &str) -> bool {
    path.starts_with("tools/hook-bootstrap/") && path.ends_with(".sh")
}

pub fn fixture_policy_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("fixture_forbidden_true_or_missing_claim_{}", claim));
        }
    }

    let files = changed_files_from_fixture(text);
    if files.is_empty() {
        failures.push("fixture_missing_changed_files".to_owned());
    }

    for file in files {
        if !is_new_status(&file.status) {
            continue;
        }
        if file.path.ends_with(".py") {
            failures.push(format!("new_python_outside_allowlist:{}", file.path));
        }
        if file.path.ends_with(".sh") && !is_allowlisted_shell(&file.path) {
            failures.push(format!("new_shell_outside_allowlist:{}", file.path));
        }
    }

    failures
}

pub fn registry_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{}", claim));
        }
    }

    for (key, expected) in [
        ("language_discipline_allowlist_measured", true),
        ("new_python_outside_allowlist_blocked", true),
        ("new_shell_outside_allowlist_blocked", true),
        ("offender_inventory_recorded", true),
        ("active_retired_cli_script_surface_recorded", true),
        ("active_script_names_self_descriptive", true),
    ] {
        if !has_bool(text, key, expected) {
            failures.push(format!("missing_registry_boundary_{}", key));
        }
    }

    for token in [
        "//:language-discipline-check",
        "scripts/ci/assert-language-discipline.rs",
        "scripts/tests/language_discipline_check.rs",
        "Buck2-native LLVM source-based coverage",
        "Tarpaulin is not the canonical coverage surface",
        "dual Cargo.toml + Buck2/Reindeer",
        "trusted cloud-ci/oya-ci",
        "hyperscaler-oriented",
        "\"active_retired_cli_script_backlog\"",
        "\"claim_boundary\": \"registered_compatibility_shim_backlog_only_no_merge_ci_authority\"",
        "\"durable_retired_cli_script_authority_allowed\": false",
        "\"unregistered_retired_cli_script_invocations_allowed\": false",
        "\"replacement_language\": \"Rust\"",
        "\"buck2_owned_replacement_required\": true",
        "\"self_explanatory_script_name_policy\"",
        "\"adr_numbered_active_script_names_allowed\": false",
    ] {
        if !text.contains(token) {
            failures.push(format!("missing_required_registry_anchor:{}", token));
        }
    }

    if !compact_json_text(text).contains("\"new_python_or_shell_gate_surface_added\":false") {
        failures.push("missing_no_new_python_or_shell_gate_surface_anchor".to_owned());
    }

    if !compact_json_text(text).contains("\"t0_4_cloud_check_backlog_count\":7") {
        failures.push("missing_t0_4_cloud_check_backlog_count_7".to_owned());
    }

    for path in BACKLOG_OFFENDERS {
        if !text.contains(path) {
            failures.push(format!("missing_required_backlog_offender:{}", path));
        }
    }

    if !compact_json_text(text).contains("\"detected_script_count\":8") {
        failures.push("missing_active_retired_cli_script_count_8".to_owned());
    }

    for path in ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG {
        if !text.contains(path) {
            failures.push(format!("missing_active_retired_cli_script:{}", path));
        }
    }

    for fixture in FIXTURES {
        if !text.contains(fixture) {
            failures.push(format!("missing_registered_fixture:{}", fixture));
        }
    }

    failures
}

pub fn retired_cli_invocation_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("//")
                || trimmed.starts_with('*')
            {
                return None;
            }
            if RETIRED_CLI_SCRIPT_INVOCATION_TOKENS
                .iter()
                .any(|token| line.contains(token))
            {
                return Some(line.trim().to_owned());
            }
            None
        })
        .collect()
}

fn is_scanned_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("sh" | "mjs")
    )
}

fn is_script_name_policy_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("sh" | "mjs" | "rs")
    )
}

fn collect_script_paths(root: &Path, rel: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let dir = root.join(rel);
    for entry in fs::read_dir(&dir).map_err(|err| format!("read dir {}: {}", dir.display(), err))? {
        let entry = entry.map_err(|err| format!("read dir entry {}: {}", dir.display(), err))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("file type {}: {}", entry.path().display(), err))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_script_paths(root, &entry_rel, output)?;
        } else if file_type.is_file() && is_scanned_script(&entry.path()) {
            output.push(entry_rel);
        }
    }
    Ok(())
}

fn collect_script_name_paths(
    root: &Path,
    rel: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let dir = root.join(rel);
    for entry in fs::read_dir(&dir).map_err(|err| format!("read dir {}: {}", dir.display(), err))? {
        let entry = entry.map_err(|err| format!("read dir entry {}: {}", dir.display(), err))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("file type {}: {}", entry.path().display(), err))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_script_name_paths(root, &entry_rel, output)?;
        } else if file_type.is_file() && is_script_name_policy_file(&entry.path()) {
            output.push(entry_rel);
        }
    }
    Ok(())
}

fn path_to_repo_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn adr_numbered_script_name(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0usize;
    while index + 7 <= bytes.len() {
        if &bytes[index..index + 3] == b"adr" {
            let mut cursor = index + 3;
            while cursor < bytes.len() && matches!(bytes[cursor], b'-' | b'_') {
                cursor += 1;
            }
            if cursor + 4 <= bytes.len()
                && bytes[cursor..cursor + 4]
                    .iter()
                    .all(|byte| byte.is_ascii_digit())
            {
                return true;
            }
        }
        index += 1;
    }
    false
}

pub fn active_script_name_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for root_rel in SELF_EXPLANATORY_SCRIPT_NAME_ROOTS {
        let mut paths = Vec::new();
        if let Err(err) = collect_script_name_paths(root, Path::new(root_rel), &mut paths) {
            failures.push(format!(
                "active_script_name_scan_failed:{}:{}",
                root_rel, err
            ));
            continue;
        }
        for rel_path in paths {
            let repo_path = path_to_repo_string(&rel_path);
            if adr_numbered_script_name(&repo_path) {
                failures.push(format!("adr_numbered_active_script_name:{}", repo_path));
            }
        }
    }
    failures
}

pub fn active_retired_cli_script_failures(root: &Path, registry_text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let mut paths = Vec::new();
    if let Err(err) = collect_script_paths(root, Path::new(ACTIVE_SCRIPT_ROOT), &mut paths) {
        failures.push(format!("active_script_scan_failed:{}", err));
        return failures;
    }

    let mut detected = Vec::new();
    for rel_path in paths {
        let repo_path = path_to_repo_string(&rel_path);
        if repo_path.starts_with("scripts/tests/") {
            continue;
        }
        let text = match fs::read_to_string(root.join(&rel_path)) {
            Ok(text) => text,
            Err(err) => {
                failures.push(format!("active_script_read_failed:{}:{}", repo_path, err));
                continue;
            }
        };
        let invocation_lines = retired_cli_invocation_lines(&text);
        if invocation_lines.is_empty() {
            continue;
        }
        detected.push(repo_path.clone());
        if !registry_text.contains(&repo_path) {
            failures.push(format!("unregistered_retired_cli_script:{}", repo_path));
        }
        for line in invocation_lines {
            if line.contains("gate validate")
                || line.contains("supply-chain")
                || line.contains("ops ")
            {
                if !registry_text.contains("\"active_merge_ci_authority\": false")
                    || !registry_text.contains("\"live_mutation_authority\": false")
                {
                    failures.push(format!(
                        "registered_retired_cli_script_missing_non_authority_boundary:{}",
                        repo_path
                    ));
                }
            }
        }
    }

    for expected in ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG {
        if !detected.iter().any(|path| path == expected) {
            failures.push(format!("missing_detected_retired_cli_script:{}", expected));
        }
    }

    if detected.len() != ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG.len() {
        failures.push(format!(
            "retired_cli_script_count_mismatch:expected_{}_got_{}",
            ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG.len(),
            detected.len()
        ));
    }

    failures
}

pub fn evaluate(root: &Path, registry: &str) -> Result<Evaluation, String> {
    let registry_path = root.join(registry);
    let registry_text = fs::read_to_string(&registry_path)
        .map_err(|err| format!("read registry {}: {}", registry_path.display(), err))?;
    let mut failures = registry_failures(&registry_text);
    failures.extend(active_retired_cli_script_failures(root, &registry_text));
    failures.extend(active_script_name_failures(root));

    let mut fixture_results = Vec::new();
    for fixture in FIXTURES {
        let fixture_path = root.join(fixture);
        let fixture_text = fs::read_to_string(&fixture_path)
            .map_err(|err| format!("read fixture {}: {}", fixture_path.display(), err))?;
        let expected = value_after_key(&fixture_text, "expected_verdict")
            .unwrap_or_else(|| "<missing>".to_owned());
        let fixture_failures = fixture_policy_failures(&fixture_text);

        match expected.as_str() {
            "PASS" if !fixture_failures.is_empty() => failures.push(format!(
                "fixture_expected_pass_failed:{}:{}",
                fixture,
                fixture_failures.join("|")
            )),
            "FAIL" if fixture_failures.is_empty() => {
                failures.push(format!("fixture_expected_fail_passed:{}", fixture))
            }
            "FAIL" => {
                if fixture_text.contains("new_python_outside_allowlist")
                    && !fixture_failures
                        .iter()
                        .any(|failure| failure.starts_with("new_python_outside_allowlist"))
                {
                    failures.push(format!(
                        "fixture_missing_expected_python_failure:{}",
                        fixture
                    ));
                }
                if fixture_text.contains("new_shell_outside_allowlist")
                    && !fixture_failures
                        .iter()
                        .any(|failure| failure.starts_with("new_shell_outside_allowlist"))
                {
                    failures.push(format!(
                        "fixture_missing_expected_shell_failure:{}",
                        fixture
                    ));
                }
            }
            "PASS" => {}
            _ => failures.push(format!(
                "fixture_missing_or_invalid_expected_verdict:{}",
                fixture
            )),
        }

        fixture_results.push(FixtureResult {
            path: (*fixture).to_owned(),
            expected,
            failures: fixture_failures,
        });
    }

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned();
    Ok(Evaluation {
        verdict,
        registry: registry.to_owned(),
        fixture_count: fixture_results.len(),
        backlog_offender_count: BACKLOG_OFFENDERS.len(),
        active_retired_cli_script_count: ACTIVE_RETIRED_CLI_SCRIPT_BACKLOG.len(),
        fixture_results,
        failures,
    })
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut repo_root = PathBuf::from(".");
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;

    let mut iter = args.into_iter();
    let _program = iter.next();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--repo-root requires a value".to_owned())?,
                );
            }
            "--registry" => {
                registry = iter
                    .next()
                    .ok_or_else(|| "--registry requires a value".to_owned())?;
            }
            "--json" => json = true,
            "-h" | "--help" => return Err(
                "usage: assert-language-discipline [--repo-root DIR] [--registry PATH] [--json]"
                    .to_owned(),
            ),
            other => return Err(format!("unknown argument: {}", other)),
        }
    }

    Ok(Config {
        repo_root,
        registry,
        json,
    })
}

pub fn evaluation_json(evaluation: &Evaluation) -> String {
    let fixture_json = evaluation
        .fixture_results
        .iter()
        .map(|fixture| {
            let failures = fixture
                .failures
                .iter()
                .map(|failure| format!("\"{}\"", json_escape(failure)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"expected\":\"{}\",\"failures\":[{}],\"path\":\"{}\"}}",
                json_escape(&fixture.expected),
                failures,
                json_escape(&fixture.path)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let failures_json = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", json_escape(failure)))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"active_retired_cli_script_count\":{},\"backlog_offender_count\":{},\"claim_boundary\":{{\"hyperscaler_grade\":false,\"live_required_context_execution_proven\":false,\"p0_0_green\":false,\"phase0_complete\":false,\"production_ready\":false,\"protected_branch_authority_proven\":false,\"status_mutation_performed\":false}},\"failures\":[{}],\"fixture_count\":{},\"fixture_results\":[{}],\"registry\":\"{}\",\"verdict\":\"{}\"}}",
        evaluation.active_retired_cli_script_count,
        evaluation.backlog_offender_count,
        failures_json,
        evaluation.fixture_count,
        fixture_json,
        json_escape(&evaluation.registry),
        evaluation.verdict
    )
}

fn print_human(evaluation: &Evaluation) {
    println!("language-discipline: {}", evaluation.verdict);
    println!("registry: {}", evaluation.registry);
    println!("fixtures: {}", evaluation.fixture_count);
    println!("backlog_offenders: {}", evaluation.backlog_offender_count);
    println!(
        "active_retired_cli_scripts: {}",
        evaluation.active_retired_cli_script_count
    );
    for fixture in &evaluation.fixture_results {
        println!(
            "fixture {} expected={} failures={}",
            fixture.path,
            fixture.expected,
            fixture.failures.len()
        );
    }
    for failure in &evaluation.failures {
        eprintln!("- {}", failure);
    }
}

fn main() {
    let config = match parse_args(env::args()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(2);
        }
    };

    match evaluate(&config.repo_root, &config.registry) {
        Ok(evaluation) => {
            if config.json {
                println!("{}", evaluation_json(&evaluation));
            } else {
                print_human(&evaluation);
            }
            if evaluation.failures.is_empty() {
                std::process::exit(0);
            }
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("language-discipline: RED");
            eprintln!("- {}", err);
            std::process::exit(1);
        }
    }
}
