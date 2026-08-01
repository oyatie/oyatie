//! `oya gate validate openapi-rest-route-parity` — runtime for the
//! check-openapi-rest-route-parity kernel. Walks REST crate source files
//! + OpenAPI contracts, builds RouteParityInputs, calls validate().

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use check_openapi_rest_route_parity::{
    RouteParityInputs, ValidationReport, Violation, validate,
};

use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpenapiRestRouteParityValidateArgs {
    crates_dir: PathBuf,
    contracts_dir: PathBuf,
    crate_prefix: String,
    contract_prefix: String,
    emit_evidence_path: Option<PathBuf>,
}

pub(crate) fn parse_openapi_rest_route_parity_validate_args(
    args: Vec<String>,
) -> Result<OpenapiRestRouteParityValidateArgs, String> {
    let mut parsed = OpenapiRestRouteParityValidateArgs {
        crates_dir: PathBuf::from("crates"),
        contracts_dir: PathBuf::from("contracts"),
        // Default scope: ops slice only. Other µservices (audit-chain,
        // eventing, secrets, ...) opt into parity via their own gate
        // invocation with a different prefix until the drift across the
        // parallel session is stabilised.
        crate_prefix: "oya-ops-".to_string(),
        contract_prefix: "ops-".to_string(),
        emit_evidence_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--crates-dir" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.crates_dir = PathBuf::from(path);
            }
            "--contracts-dir" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.contracts_dir = PathBuf::from(path);
            }
            "--crate-prefix" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.crate_prefix = value;
            }
            "--contract-prefix" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.contract_prefix = value;
            }
            "--emit-evidence" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_evidence_path = Some(PathBuf::from(path));
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpenapiRestRouteParityReport {
    pub report: ValidationReport,
    pub validation_duration_ms: u64,
}

pub(crate) fn validate_openapi_rest_route_parity_gate(
    args: OpenapiRestRouteParityValidateArgs,
) -> Result<OpenapiRestRouteParityReport, String> {
    let start = Instant::now();

    let rest_routes = scan_rest_routes(&args.crates_dir, &args.crate_prefix)?;
    let openapi_paths = scan_openapi_paths(&args.contracts_dir, &args.contract_prefix)?;

    let inputs = RouteParityInputs {
        rest_routes,
        openapi_paths,
    };
    let report = validate(&inputs);
    let validation_duration_ms = start.elapsed().as_millis() as u64;
    let wrapped = OpenapiRestRouteParityReport {
        report: report.clone(),
        validation_duration_ms,
    };

    if let Some(evidence_path) = &args.emit_evidence_path {
        write_evidence_bundle(evidence_path, &wrapped)?;
    }

    if !report.is_clean() {
        return Err(format_violations(&report.violations));
    }

    Ok(wrapped)
}

/// Scan `crates_dir` for `*-rest` crates whose name starts with `crate_prefix`,
/// read each `src/lib.rs`, and extract every
/// `pub const *_ROUTE: &str = "..."` value.
fn scan_rest_routes(crates_dir: &Path, crate_prefix: &str) -> Result<BTreeSet<String>, String> {
    let mut routes = BTreeSet::new();
    let entries = match fs::read_dir(crates_dir) {
        Ok(it) => it,
        Err(_) => return Ok(routes),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !name.starts_with(crate_prefix) || !name.ends_with("-rest") {
            continue;
        }
        let lib_rs = path.join("src/lib.rs");
        if !lib_rs.exists() {
            continue;
        }
        let text = fs::read_to_string(&lib_rs).map_err(|error| {
            format!("rest crate lib.rs unreadable {}: {error}", lib_rs.display())
        })?;
        extract_route_constants_from_text(&text, &mut routes);
    }
    Ok(routes)
}

/// Extract every `pub const *_ROUTE: &str = "..."` value from the full text
/// of a Rust source file. Tolerates rustfmt's habit of wrapping long
/// constants onto a second line:
///
/// ```ignore
/// pub const FOO_ROUTE: &str =
///     "/some/very/long/path";
/// ```
fn extract_route_constants_from_text(text: &str, out: &mut BTreeSet<String>) {
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("pub const ") else {
            continue;
        };
        let Some(colon) = rest.find(": &str =") else {
            continue;
        };
        let name = &rest[..colon];
        if !name.ends_with("_ROUTE") {
            continue;
        }
        let after_eq = rest[colon + ": &str =".len()..].trim_start();
        let lookup_in: String = if after_eq.is_empty() {
            // rustfmt wrapped — look on the next non-empty line.
            match lines.get(index + 1) {
                Some(next) => next.trim_start().to_string(),
                None => continue,
            }
        } else {
            after_eq.to_string()
        };
        let Some(after_quote) = lookup_in.strip_prefix('"') else {
            continue;
        };
        let Some(close) = after_quote.find('"') else {
            continue;
        };
        out.insert(after_quote[..close].to_string());
    }
}

/// Single-line variant retained for unit tests.
#[cfg(test)]
fn extract_route_constant(line: &str) -> Option<String> {
    let mut out = BTreeSet::new();
    extract_route_constants_from_text(line, &mut out);
    out.into_iter().next()
}

/// Scan `contracts_dir` for `<contract_prefix>*.openapi.yaml`, parse the
/// top-level `paths:` block, and return every key (route) that appears.
fn scan_openapi_paths(
    contracts_dir: &Path,
    contract_prefix: &str,
) -> Result<BTreeSet<String>, String> {
    let mut paths = BTreeSet::new();
    let entries = match fs::read_dir(contracts_dir) {
        Ok(it) => it,
        Err(_) => return Ok(paths),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !name.starts_with(contract_prefix) || !name.ends_with(".openapi.yaml") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("openapi file unreadable {}: {error}", path.display()))?;
        extract_openapi_path_keys(&text, &mut paths);
    }
    Ok(paths)
}

/// Parse top-level `paths:` map keys. Strategy: find the `paths:` line at
/// column 0, then read subsequent lines until indent drops back to 0 (or
/// EOF). Path keys are at indent == 2 (per YAML idiom + our authored files)
/// and end in `:`. Skip nested lines (indent > 2).
fn extract_openapi_path_keys(text: &str, out: &mut BTreeSet<String>) {
    let mut in_paths = false;
    for line in text.lines() {
        if !in_paths {
            if line.trim_end() == "paths:" {
                in_paths = true;
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            // Left the paths block.
            break;
        }
        if indent != 2 {
            continue;
        }
        let trimmed = line.trim_start();
        if !trimmed.ends_with(':') {
            continue;
        }
        let key = &trimmed[..trimmed.len() - 1];
        if key.starts_with('/') {
            out.insert(key.to_string());
        }
    }
}

fn write_evidence_bundle(
    path: &Path,
    wrapped: &OpenapiRestRouteParityReport,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "evidence bundle dir unwriteable {}: {error}",
                parent.display()
            )
        })?;
    }
    let report = &wrapped.report;
    let outcome = if report.is_clean() {
        "success"
    } else {
        "failure"
    };
    let body = format!(
        "{{\n  \"$schema_ref\": \"/templates/evidence-bundle-template.json\",\n  \"_artifact_id\": \"openapi-rest-route-parity-lane-run\",\n  \"_meta\": {{ \"emitter\": \"oya-dev-cli gate validate openapi-rest-route-parity\" }},\n  \"outcome\": \"{}\",\n  \"rest_route_count\": {},\n  \"openapi_path_count\": {},\n  \"violation_count\": {},\n  \"validation_duration_ms\": {}\n}}\n",
        outcome,
        report.rest_route_count,
        report.openapi_path_count,
        report.violations.len(),
        wrapped.validation_duration_ms,
    );
    fs::write(path, body)
        .map_err(|error| format!("evidence bundle write failed {}: {error}", path.display()))?;
    Ok(())
}

fn format_violations(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(|v| match v {
            Violation::MissingFromOpenapi { route } => {
                format!("REST route `{route}` not in any contracts/*.openapi.yaml")
            }
            Violation::MissingFromRest { path } => {
                format!("OpenAPI path `{path}` not in any rest crate route constant")
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_defaults() {
        let args = parse_openapi_rest_route_parity_validate_args(vec![]).unwrap();
        assert_eq!(args.crates_dir, PathBuf::from("crates"));
        assert_eq!(args.contracts_dir, PathBuf::from("contracts"));
    }

    #[test]
    fn parse_args_unknown_flag_errors() {
        let result = parse_openapi_rest_route_parity_validate_args(vec!["--bogus".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn extract_route_constant_simple() {
        assert_eq!(
            extract_route_constant("pub const FOO_ROUTE: &str = \"/foo\";"),
            Some("/foo".to_string())
        );
    }

    #[test]
    fn extract_route_constant_ignores_non_route_constants() {
        assert!(extract_route_constant("pub const FOO_METHOD: &str = \"GET\";").is_none());
        assert!(extract_route_constant("let x = 1;").is_none());
    }

    #[test]
    fn extract_route_constant_tolerates_leading_whitespace() {
        assert_eq!(
            extract_route_constant("    pub const FOO_ROUTE: &str = \"/foo\";"),
            Some("/foo".to_string())
        );
    }

    #[test]
    fn extract_openapi_path_keys_basic() {
        let yaml = "openapi: 3.2.0\npaths:\n  /workspace:\n    get: {}\n  /workspace/api/v1/health:\n    get: {}\ncomponents:\n  schemas: {}\n";
        let mut out = BTreeSet::new();
        extract_openapi_path_keys(yaml, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains("/workspace"));
        assert!(out.contains("/workspace/api/v1/health"));
    }

    #[test]
    fn extract_openapi_path_keys_stops_at_components() {
        let yaml = "paths:\n  /a:\n    get: {}\ncomponents:\n  schemas:\n    /not-a-path:\n      type: string\n";
        let mut out = BTreeSet::new();
        extract_openapi_path_keys(yaml, &mut out);
        assert_eq!(out.len(), 1);
        assert!(out.contains("/a"));
    }

    #[test]
    fn extract_openapi_path_keys_ignores_nested_keys() {
        let yaml = "paths:\n  /a:\n    get:\n      operationId: doX\n      x-cedar-fragments: [ops-internal-public]\n  /b:\n    get: {}\n";
        let mut out = BTreeSet::new();
        extract_openapi_path_keys(yaml, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains("/a"));
        assert!(out.contains("/b"));
    }
}
