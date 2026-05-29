use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductPrdJsonArgs {
    repo_root: PathBuf,
    product_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductPrdJsonReport {
    pub(crate) products_checked: usize,
    pub(crate) acceptance_criteria_checked: usize,
    pub(crate) metrics_checked: usize,
    pub(crate) root_hub_links_checked: usize,
    pub(crate) validation_duration_ms: u128,
}

pub(crate) fn parse_product_prd_json_validate_args(
    args: Vec<String>,
) -> Result<ProductPrdJsonArgs, String> {
    let mut parsed = ProductPrdJsonArgs {
        repo_root: PathBuf::from("."),
        product_paths: Vec::new(),
    };

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                parsed.repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--repo-root requires a path".to_string())?,
                );
            }
            "--product" | "--path" => {
                parsed
                    .product_paths
                    .push(PathBuf::from(iter.next().ok_or_else(|| {
                        format!("{flag} requires a product JSON path")
                    })?));
            }
            other => {
                return Err(format!(
                    "product-prd-json: unknown flag {other:?}; usage: oya gate validate product-prd-json [--repo-root <.>] [--product <specs/microservices/<id>.json>]..."
                ));
            }
        }
    }

    Ok(parsed)
}

pub(crate) fn validate_product_prd_json_gate(
    args: ProductPrdJsonArgs,
) -> Result<ProductPrdJsonReport, String> {
    let started = Instant::now();
    let repo_root = args.repo_root;
    let product_paths = if args.product_paths.is_empty() {
        // Per ADR-0131 specs/products → specs/microservices flatten (2026-05-18).
        // We retain a fallback scan of specs/products/ for the transition window
        // (the legacy directory now holds only RETIREMENT.md and will be removed
        // in a follow-up cleanup IP); the new canonical home is specs/microservices/.
        collect_product_json_paths(&repo_root.join("specs/microservices"))?
    } else {
        args.product_paths
            .iter()
            .map(|path| absolutize(&repo_root, path))
            .collect::<Vec<_>>()
    };

    if product_paths.is_empty() {
        return Err("no product PRD JSON files found under specs/microservices".to_string());
    }

    let root_hub_paths = load_root_hub_product_paths(&repo_root)?;
    let mut errors = Vec::new();
    let mut acceptance_criteria_checked = 0usize;
    let mut metrics_checked = 0usize;
    let mut root_hub_links_checked = 0usize;
    let mut products_checked = 0usize;

    for path in &product_paths {
        // Skip non-PRD machine-readable specs that happen to live under
        // specs/microservices/ — e.g., Microservice-Consolidation-Spec
        // (spec_id starts with MSC-) per ADR-0136 foundry consolidation.
        if !is_prd_spec_file(path) {
            continue;
        }
        match validate_one_product(path, &repo_root, &root_hub_paths) {
            Ok(report) => {
                products_checked += 1;
                acceptance_criteria_checked += report.acceptance_criteria_checked;
                metrics_checked += report.metrics_checked;
                root_hub_links_checked += 1;
            }
            Err(mut product_errors) => {
                products_checked += 1;
                errors.append(&mut product_errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(ProductPrdJsonReport {
            products_checked,
            acceptance_criteria_checked,
            metrics_checked,
            root_hub_links_checked,
            validation_duration_ms: started.elapsed().as_millis(),
        })
    } else {
        Err(errors.join("\n"))
    }
}

fn is_prd_spec_file(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<Value>(&content) else {
        return false;
    };
    let meta = json.get("_meta");
    // Skip retired specs (e.g., shorts.json absorbed into social per ADR-0334;
    // network.json merged into community per Wave 15K). They carry status=Retired
    // and a tombstone-only shape that omits the canonical PRD sections.
    if meta
        .and_then(|m| m.get("doc_class"))
        .and_then(Value::as_str)
        == Some("RetiredMicroserviceMarker")
        || meta.and_then(|m| m.get("status")).and_then(Value::as_str) == Some("Retired")
    {
        return false;
    }
    meta.and_then(|m| m.get("spec_id"))
        .and_then(Value::as_str)
        .map(|sid| sid.starts_with("PRD-"))
        .unwrap_or(false)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProductShapeReport {
    acceptance_criteria_checked: usize,
    metrics_checked: usize,
}

fn validate_one_product(
    path: &Path,
    repo_root: &Path,
    root_hub_paths: &BTreeSet<String>,
) -> Result<ProductShapeReport, Vec<String>> {
    let mut errors = Vec::new();
    let display = path.display().to_string();
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return Err(vec![format!(
                "{display}: unable to read product PRD JSON: {error}"
            )]);
        }
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(error) => return Err(vec![format!("{display}: invalid JSON: {error}")]),
    };

    let Some(object) = json.as_object() else {
        return Err(vec![format!(
            "{display}: top-level value must be an object"
        )]);
    };

    require_string(object, "$schema", &display, &mut errors);
    require_string(object, "$id", &display, &mut errors);
    require_string(object, "title", &display, &mut errors);
    require_object(object, "inherits", &display, &mut errors);
    require_object(object, "scope", &display, &mut errors);
    require_object(object, "contracts", &display, &mut errors);
    require_object(object, "data_model", &display, &mut errors);
    require_object(object, "deps", &display, &mut errors);
    require_present(object, "regional_packs", &display, &mut errors);
    require_present(object, "decision_log", &display, &mut errors);

    if !object.contains_key("optimization_practices")
        && !object.contains_key("optimization_practices_ref")
    {
        errors.push(format!(
            "{display}: missing optimization_practices or optimization_practices_ref"
        ));
    }

    let meta = object.get("_meta").and_then(Value::as_object);
    match meta {
        Some(meta) => {
            match meta.get("doc_class").and_then(Value::as_str) {
                Some("Machine-Readable-Spec") => {}
                Some(other) => errors.push(format!(
                    "{display}: _meta.doc_class must be Machine-Readable-Spec, got {other:?}"
                )),
                None => errors.push(format!("{display}: missing _meta.doc_class")),
            }
            let spec_id = require_string(meta, "spec_id", &format!("{display}#_meta"), &mut errors);
            if let Some(spec_id) = spec_id
                && !spec_id.starts_with("PRD-")
            {
                errors.push(format!("{display}: _meta.spec_id must start with PRD-"));
            }
            require_string(meta, "version", &format!("{display}#_meta"), &mut errors);
            require_string(meta, "status", &format!("{display}#_meta"), &mut errors);
            require_string(meta, "purpose", &format!("{display}#_meta"), &mut errors);
        }
        None => errors.push(format!("{display}: missing _meta object")),
    }

    let identity = object.get("identity").and_then(Value::as_object);
    if let Some(identity) = identity {
        if let Some(product_id) = require_string(
            identity,
            "product_id",
            &format!("{display}#identity"),
            &mut errors,
        ) {
            let expected = expected_product_id(repo_root, path);
            if let Some(expected) = expected
                && product_id != expected
            {
                errors.push(format!(
                    "{display}: identity.product_id {product_id:?} does not match path-derived {expected:?}"
                ));
            }
        }
        require_string(
            identity,
            "owning_axis",
            &format!("{display}#identity"),
            &mut errors,
        );
    } else {
        errors.push(format!("{display}: missing identity object"));
    }

    let acceptance_criteria_checked = validate_acceptance_criteria(object, &display, &mut errors);
    let strict_user_facing = identity
        .and_then(|identity| identity.get("user_facing_surface"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let metrics_checked = validate_metrics(object, &display, strict_user_facing, &mut errors);
    validate_goals(object, &display, &mut errors);

    require_non_empty_array(object, "target_users", &display, &mut errors);
    require_non_empty_array(object, "risks", &display, &mut errors);
    require_non_empty_array(object, "competitive", &display, &mut errors);
    require_non_empty_array(object, "best_practices", &display, &mut errors);
    require_non_empty_array(object, "patterns", &display, &mut errors);
    require_non_empty_array(object, "anti_patterns", &display, &mut errors);
    require_non_empty_array(object, "sources_scanned", &display, &mut errors);

    if strict_user_facing {
        require_object(object, "user_experience", &display, &mut errors);
        validate_frontend_components(object, repo_root, &display, &mut errors);
    }

    let normalized_path = normalized_repo_path(repo_root, path);
    if !root_hub_paths.contains(&normalized_path) {
        errors.push(format!(
            "{display}: specs/root-hub-pointers.json missing entry with current_path={normalized_path:?}"
        ));
    }

    if errors.is_empty() {
        Ok(ProductShapeReport {
            acceptance_criteria_checked,
            metrics_checked,
        })
    } else {
        Err(errors)
    }
}

fn validate_acceptance_criteria(
    object: &serde_json::Map<String, Value>,
    display: &str,
    errors: &mut Vec<String>,
) -> usize {
    let Some(criteria) = object.get("acceptance_criteria").and_then(Value::as_array) else {
        errors.push(format!(
            "{display}: acceptance_criteria must be a non-empty array"
        ));
        return 0;
    };
    if criteria.is_empty() {
        errors.push(format!("{display}: acceptance_criteria must be non-empty"));
        return 0;
    }
    let mut ids = BTreeSet::new();
    for criterion in criteria {
        let Some(criterion) = criterion.as_object() else {
            errors.push(format!(
                "{display}: each acceptance criterion must be an object"
            ));
            continue;
        };
        let id = require_string(
            criterion,
            "id",
            &format!("{display}#acceptance_criteria"),
            errors,
        );
        if let Some(id) = id {
            if !ids.insert(id.to_string()) {
                errors.push(format!(
                    "{display}: duplicate acceptance criterion id {id:?}"
                ));
            }
            if !id.starts_with("AC-") {
                errors.push(format!(
                    "{display}: acceptance criterion id {id:?} must start with AC-"
                ));
            }
        }
        require_string(
            criterion,
            "given",
            &format!("{display}#acceptance_criteria"),
            errors,
        );
        require_string(
            criterion,
            "when",
            &format!("{display}#acceptance_criteria"),
            errors,
        );
        require_string(
            criterion,
            "then",
            &format!("{display}#acceptance_criteria"),
            errors,
        );
        require_string(
            criterion,
            "test_ref",
            &format!("{display}#acceptance_criteria"),
            errors,
        );
    }
    criteria.len()
}

fn validate_metrics(
    object: &serde_json::Map<String, Value>,
    display: &str,
    strict_metric_verification: bool,
    errors: &mut Vec<String>,
) -> usize {
    let Some(metrics) = object.get("metrics").and_then(Value::as_array) else {
        errors.push(format!("{display}: metrics must be a non-empty array"));
        return 0;
    };
    if metrics.is_empty() {
        errors.push(format!("{display}: metrics must be non-empty"));
        return 0;
    }
    for metric in metrics {
        let Some(metric) = metric.as_object() else {
            errors.push(format!("{display}: each metric must be an object"));
            continue;
        };
        require_string(metric, "name", &format!("{display}#metrics"), errors);
        require_object(metric, "targets", &format!("{display}#metrics"), errors);
        if strict_metric_verification {
            require_string(
                metric,
                "verification_ref",
                &format!("{display}#metrics"),
                errors,
            );
            require_string(metric, "lane_ref", &format!("{display}#metrics"), errors);
        } else if !has_non_empty_string(metric, "verification_ref")
            && !has_non_empty_string(metric, "lane_ref")
        {
            errors.push(format!(
                "{display}: non-strict metric must still declare verification_ref or lane_ref"
            ));
        }
    }
    metrics.len()
}

fn validate_goals(
    object: &serde_json::Map<String, Value>,
    display: &str,
    errors: &mut Vec<String>,
) {
    let Some(goals) = object.get("goals").and_then(Value::as_object) else {
        errors.push(format!("{display}: goals must be an object"));
        return;
    };
    for key in [
        "scalability",
        "security",
        "performance",
        "efficiency",
        "reliability",
    ] {
        if !goals.contains_key(key) {
            errors.push(format!("{display}: goals missing {key}"));
        }
    }
}

fn validate_frontend_components(
    object: &serde_json::Map<String, Value>,
    repo_root: &Path,
    display: &str,
    errors: &mut Vec<String>,
) {
    let Some(components) = object.get("frontend_components").and_then(Value::as_array) else {
        errors.push(format!(
            "{display}: user-facing product must declare frontend_components array"
        ));
        return;
    };
    if components.is_empty() {
        errors.push(format!(
            "{display}: user-facing product frontend_components must be non-empty"
        ));
    }
    for component in components {
        let Some(component) = component.as_object() else {
            errors.push(format!(
                "{display}: each frontend component must be an object"
            ));
            continue;
        };
        require_string(
            component,
            "component",
            &format!("{display}#frontend_components"),
            errors,
        );
        if let Some(design_system_ref) = require_string(
            component,
            "design_system_ref",
            &format!("{display}#frontend_components"),
            errors,
        ) {
            validate_design_system_ref_exists(design_system_ref, repo_root, display, errors);
        }
        require_non_empty_array(
            component,
            "tested_at_breakpoints",
            &format!("{display}#frontend_components"),
            errors,
        );
    }
}

fn validate_design_system_ref_exists(
    design_system_ref: &str,
    repo_root: &Path,
    display: &str,
    errors: &mut Vec<String>,
) {
    let Some(path) = design_system_ref.strip_prefix("$ref:") else {
        errors.push(format!(
            "{display}: design_system_ref {design_system_ref:?} must use $ref:<repo-path>"
        ));
        return;
    };
    let path = path.split('#').next().unwrap_or(path);
    let path = repo_root.join(path.trim_start_matches('/'));
    if !path.exists() {
        errors.push(format!(
            "{display}: design_system_ref {design_system_ref:?} does not resolve to {}",
            path.display()
        ));
    }
}

fn collect_product_json_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    collect_json_recursive(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_json_recursive(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Err(format!("{} does not exist", root.display()));
    }
    for entry in std::fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_recursive(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            // Skip meta-files inside specs/microservices/ that are not PRD specs:
            //   manifest-schema.json  — JSON Schema for per-µservice manifest.json
            //   manifests-index.json  — Per-µservice manifest registry index
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && matches!(file_name, "manifest-schema.json" | "manifests-index.json")
            {
                continue;
            }
            out.push(path);
        }
    }
    Ok(())
}

fn load_root_hub_product_paths(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let path = repo_root.join("specs/root-hub-pointers.json");
    let content = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "{}: unable to read root hub pointers: {error}",
            path.display()
        )
    })?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("{}: invalid JSON: {error}", path.display()))?;
    let Some(entries) = json.get("entry_points").and_then(Value::as_object) else {
        return Err("specs/root-hub-pointers.json missing entry_points object".to_string());
    };
    let mut paths = BTreeSet::new();
    for entry in entries.values() {
        if let Some(current_path) = entry.get("current_path").and_then(Value::as_str)
            && (current_path.starts_with("/specs/products/")
                || current_path.starts_with("/specs/microservices/"))
        {
            paths.insert(current_path.to_string());
        }
    }
    Ok(paths)
}

fn expected_product_id(repo_root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(repo_root).ok().unwrap_or(path);
    let mut components = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    let file = components.pop()?;
    let stem = file.strip_suffix(".json")?;
    for index in 0..components.len().saturating_sub(1) {
        if components[index] == "specs" {
            match components[index + 1] {
                // Per ADR-0131 per-µservice flat layout: under specs/microservices/
                // every spec is a single-concern flat file; the product_id matches
                // the filename stem (platform/bundle nesting is retired by ADR-0132).
                "microservices" => return Some(stem.to_string()),
                "products" => {
                    return match components.get(index + 2..) {
                        Some([]) => Some(stem.to_string()),
                        Some([family]) => Some(format!("{family}-{stem}")),
                        _ => None,
                    };
                }
                _ => {}
            }
        }
    }
    None
}

fn normalized_repo_path(repo_root: &Path, path: &Path) -> String {
    let absolute = absolutize(repo_root, path);
    let relative = absolute.strip_prefix(repo_root).ok().unwrap_or(&absolute);
    format!("/{}", relative.to_string_lossy())
}

fn absolutize(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn require_present(
    object: &serde_json::Map<String, Value>,
    key: &str,
    display: &str,
    errors: &mut Vec<String>,
) {
    if !object.contains_key(key) || object.get(key).is_some_and(Value::is_null) {
        errors.push(format!("{display}: missing {key}"));
    }
}

fn require_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    display: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    let value = object.get(key).and_then(Value::as_str);
    match value {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => {
            errors.push(format!("{display}: missing non-empty string {key}"));
            None
        }
    }
}

fn has_non_empty_string(object: &serde_json::Map<String, Value>, key: &str) -> bool {
    object
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn require_object(
    object: &serde_json::Map<String, Value>,
    key: &str,
    display: &str,
    errors: &mut Vec<String>,
) {
    if !object.get(key).is_some_and(Value::is_object) {
        errors.push(format!("{display}: missing object {key}"));
    }
}

fn require_non_empty_array(
    object: &serde_json::Map<String, Value>,
    key: &str,
    display: &str,
    errors: &mut Vec<String>,
) {
    match object.get(key).and_then(Value::as_array) {
        Some(values) if !values.is_empty() => {}
        _ => errors.push(format!("{display}: missing non-empty array {key}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_product_id_for_top_level_product() {
        assert_eq!(
            expected_product_id(Path::new("."), Path::new("specs/products/workflow.json")),
            Some("workflow".to_string())
        );
    }

    #[test]
    fn path_product_id_for_microservice_product() {
        assert_eq!(
            expected_product_id(Path::new("."), Path::new("specs/microservices/mail.json")),
            Some("mail".to_string())
        );
    }

    #[test]
    fn path_product_id_for_absolute_microservice_product() {
        assert_eq!(
            expected_product_id(
                Path::new("."),
                Path::new("/workspace/oyatie/specs/microservices/mail.json")
            ),
            Some("mail".to_string())
        );
    }

    #[test]
    fn non_strict_metrics_require_usable_verification_pointer() {
        let object = serde_json::json!({
            "metrics": [
                {
                    "name": "Coverage",
                    "targets": {},
                    "verification_ref": "",
                    "lane_ref": null
                }
            ]
        });
        let object = object.as_object().expect("test object");
        let mut errors = Vec::new();

        let count = validate_metrics(object, "test-product", false, &mut errors);

        assert_eq!(count, 1);
        assert_eq!(
            errors,
            vec!["test-product: non-strict metric must still declare verification_ref or lane_ref"]
        );
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let error = parse_product_prd_json_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }
}
