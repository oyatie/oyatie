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
    pub(crate) test_refs_checked: usize,
    pub(crate) metrics_checked: usize,
    pub(crate) verification_refs_checked: usize,
    pub(crate) planned_feature_refs_checked: usize,
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
    let explicit_product_paths = !args.product_paths.is_empty();
    let product_paths = if explicit_product_paths {
        args.product_paths
            .iter()
            .map(|path| absolutize(&repo_root, path))
            .collect::<Vec<_>>()
    } else {
        // Per ADR-0131 specs/products → specs/microservices flatten (2026-05-18).
        // We retain a fallback scan of specs/products/ for the transition window
        // (the legacy directory now holds only RETIREMENT.md and will be removed
        // in a follow-up cleanup IP); the new canonical home is specs/microservices/.
        collect_product_json_paths(&repo_root.join("specs/microservices"))?
    };

    if product_paths.is_empty() {
        return Err("no product PRD JSON files found under specs/microservices".to_string());
    }

    let root_hub_paths = load_root_hub_product_paths(&repo_root)?;
    let mut errors = Vec::new();
    let mut acceptance_criteria_checked = 0usize;
    let mut test_refs_checked = 0usize;
    let mut metrics_checked = 0usize;
    let mut verification_refs_checked = 0usize;
    let mut planned_feature_refs_checked = 0usize;
    let mut root_hub_links_checked = 0usize;
    let mut products_checked = 0usize;

    for path in &product_paths {
        // Skip non-PRD machine-readable specs that happen to live under
        // specs/microservices/ — e.g., Microservice-Consolidation-Spec
        // (spec_id starts with MSC-) per ADR-0136 foundry consolidation.
        match classify_prd_spec_file(path) {
            Ok(ProductPrdFileKind::ActivePrd) => {}
            Ok(ProductPrdFileKind::NonPrd) if explicit_product_paths => {
                errors.push(format!(
                    "{}: explicit product path must be an active product PRD JSON with _meta.spec_id starting PRD-",
                    path.display()
                ));
                continue;
            }
            Ok(ProductPrdFileKind::NonPrd) => continue,
            Err(error) if explicit_product_paths => {
                errors.push(error);
                continue;
            }
            Err(_) => continue,
        }
        match validate_one_product(path, &repo_root, &root_hub_paths) {
            Ok(report) => {
                products_checked += 1;
                acceptance_criteria_checked += report.acceptance_criteria_checked;
                test_refs_checked += report.test_refs_checked;
                metrics_checked += report.metrics_checked;
                verification_refs_checked += report.verification_refs_checked;
                planned_feature_refs_checked += report.planned_feature_refs_checked;
                root_hub_links_checked += 1;
            }
            Err(mut product_errors) => {
                products_checked += 1;
                errors.append(&mut product_errors);
            }
        }
    }

    if products_checked == 0 {
        errors.push(
            "product-prd-json: zero active product PRD JSON specs checked; provide active PRD specs under specs/microservices or explicit --product paths".to_string(),
        );
    }

    if errors.is_empty() {
        Ok(ProductPrdJsonReport {
            products_checked,
            acceptance_criteria_checked,
            test_refs_checked,
            metrics_checked,
            verification_refs_checked,
            planned_feature_refs_checked,
            root_hub_links_checked,
            validation_duration_ms: started.elapsed().as_millis(),
        })
    } else {
        Err(errors.join("\n"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProductPrdFileKind {
    ActivePrd,
    NonPrd,
}

fn classify_prd_spec_file(path: &Path) -> Result<ProductPrdFileKind, String> {
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "{}: unable to read product PRD JSON: {error}",
            path.display()
        )
    })?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("{}: invalid product PRD JSON: {error}", path.display()))?;

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
        return Ok(ProductPrdFileKind::NonPrd);
    }

    if meta
        .and_then(|m| m.get("spec_id"))
        .and_then(Value::as_str)
        .is_some_and(|sid| sid.starts_with("PRD-"))
    {
        Ok(ProductPrdFileKind::ActivePrd)
    } else {
        Ok(ProductPrdFileKind::NonPrd)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AcceptanceShapeReport {
    criteria_checked: usize,
    test_refs_checked: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MetricShapeReport {
    metrics_checked: usize,
    verification_refs_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProductShapeReport {
    acceptance_criteria_checked: usize,
    test_refs_checked: usize,
    metrics_checked: usize,
    verification_refs_checked: usize,
    planned_feature_refs_checked: usize,
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

    let acceptance_report = validate_acceptance_criteria(object, &display, &mut errors);
    let strict_user_facing = identity
        .and_then(|identity| identity.get("user_facing_surface"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let metric_report = validate_metrics(object, &display, strict_user_facing, &mut errors);
    let planned_feature_refs_checked = validate_planned_feature_refs(&json, &display, &mut errors);
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
            acceptance_criteria_checked: acceptance_report.criteria_checked,
            test_refs_checked: acceptance_report.test_refs_checked,
            metrics_checked: metric_report.metrics_checked,
            verification_refs_checked: metric_report.verification_refs_checked,
            planned_feature_refs_checked,
        })
    } else {
        Err(errors)
    }
}

fn validate_acceptance_criteria(
    object: &serde_json::Map<String, Value>,
    display: &str,
    errors: &mut Vec<String>,
) -> AcceptanceShapeReport {
    let Some(criteria) = object.get("acceptance_criteria").and_then(Value::as_array) else {
        errors.push(format!(
            "{display}: acceptance_criteria must be a non-empty array"
        ));
        return AcceptanceShapeReport::default();
    };
    if criteria.is_empty() {
        errors.push(format!("{display}: acceptance_criteria must be non-empty"));
        return AcceptanceShapeReport::default();
    }
    let mut ids = BTreeSet::new();
    let mut report = AcceptanceShapeReport {
        criteria_checked: criteria.len(),
        test_refs_checked: 0,
    };
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
        if let Some(test_ref) = require_string(
            criterion,
            "test_ref",
            &format!("{display}#acceptance_criteria"),
            errors,
        ) {
            report.test_refs_checked += 1;
            validate_current_prd_reference(
                display,
                "acceptance_criteria.test_ref",
                test_ref,
                errors,
            );
        }
    }
    report
}

fn validate_metrics(
    object: &serde_json::Map<String, Value>,
    display: &str,
    strict_metric_verification: bool,
    errors: &mut Vec<String>,
) -> MetricShapeReport {
    let Some(metrics) = object.get("metrics").and_then(Value::as_array) else {
        errors.push(format!("{display}: metrics must be a non-empty array"));
        return MetricShapeReport::default();
    };
    if metrics.is_empty() {
        errors.push(format!("{display}: metrics must be non-empty"));
        return MetricShapeReport::default();
    }
    let mut report = MetricShapeReport {
        metrics_checked: metrics.len(),
        verification_refs_checked: 0,
    };
    for metric in metrics {
        let Some(metric) = metric.as_object() else {
            errors.push(format!("{display}: each metric must be an object"));
            continue;
        };
        require_string(metric, "name", &format!("{display}#metrics"), errors);
        require_object(metric, "targets", &format!("{display}#metrics"), errors);
        if strict_metric_verification {
            if let Some(verification_ref) = require_string(
                metric,
                "verification_ref",
                &format!("{display}#metrics"),
                errors,
            ) {
                report.verification_refs_checked += 1;
                validate_current_prd_reference(
                    display,
                    "metrics.verification_ref",
                    verification_ref,
                    errors,
                );
            }
            if let Some(lane_ref) =
                require_string(metric, "lane_ref", &format!("{display}#metrics"), errors)
            {
                validate_lane_reference(display, "metrics.lane_ref", lane_ref, errors);
            }
        } else {
            let verification_ref = metric.get("verification_ref").and_then(Value::as_str);
            let lane_ref = metric.get("lane_ref").and_then(Value::as_str);
            let has_verification_ref =
                verification_ref.is_some_and(|value| !value.trim().is_empty());
            let has_lane_ref = lane_ref.is_some_and(|value| !value.trim().is_empty());
            if !has_verification_ref && !has_lane_ref {
                errors.push(format!(
                    "{display}: non-strict metric must still declare verification_ref or lane_ref"
                ));
            }
            if let Some(verification_ref) =
                verification_ref.filter(|value| !value.trim().is_empty())
            {
                report.verification_refs_checked += 1;
                validate_current_prd_reference(
                    display,
                    "metrics.verification_ref",
                    verification_ref,
                    errors,
                );
            }
            if let Some(lane_ref) = lane_ref.filter(|value| !value.trim().is_empty()) {
                validate_lane_reference(display, "metrics.lane_ref", lane_ref, errors);
            }
        }
    }
    report
}

fn validate_planned_feature_refs(value: &Value, display: &str, errors: &mut Vec<String>) -> usize {
    let mut refs_checked = 0usize;
    walk_planned_feature_refs(value, "$", display, errors, &mut refs_checked);
    refs_checked
}

fn walk_planned_feature_refs(
    value: &Value,
    location: &str,
    display: &str,
    errors: &mut Vec<String>,
    refs_checked: &mut usize,
) {
    match value {
        Value::Object(object) => {
            let mut has_planned_verification_ref = false;
            for field in ["planned_verification_ref", "planned_enforcement_ref"] {
                if let Some(raw_ref) = object.get(field) {
                    has_planned_verification_ref |= field == "planned_verification_ref";
                    match raw_ref.as_str().map(str::trim) {
                        Some(reference) if !reference.is_empty() => {
                            *refs_checked += 1;
                            validate_planned_reference(display, field, reference, errors);
                        }
                        _ => errors.push(format!(
                            "{display}#{location}: {field} must be a non-empty string"
                        )),
                    }
                }
            }
            if has_planned_verification_ref && !has_non_empty_string(object, "claim_boundary") {
                errors.push(format!(
                    "{display}#{location}: planned feature verification rows must declare a non-empty claim_boundary"
                ));
            }
            for (key, child) in object {
                let child_location = format!("{location}.{key}");
                walk_planned_feature_refs(child, &child_location, display, errors, refs_checked);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_location = format!("{location}[{index}]");
                walk_planned_feature_refs(child, &child_location, display, errors, refs_checked);
            }
        }
        _ => {}
    }
}

fn validate_current_prd_reference(
    display: &str,
    field: &str,
    reference: &str,
    errors: &mut Vec<String>,
) {
    validate_not_retired_reference(display, field, reference, errors);
    if !is_current_prd_reference_shape(reference) {
        errors.push(format!(
            "{display}: {field} must cite a current test/task/spec artifact or cargo/cloud-ci gate command, got {reference:?}"
        ));
    }
}

fn validate_planned_reference(
    display: &str,
    field: &str,
    reference: &str,
    errors: &mut Vec<String>,
) {
    validate_not_retired_reference(display, field, reference, errors);
    if field == "planned_enforcement_ref" {
        if !is_slug_reference(reference) || !reference.starts_with("oya-governance-") {
            errors.push(format!(
                "{display}: {field} must cite a planned oya-governance-* enforcement id, got {reference:?}"
            ));
        }
    } else if !is_current_prd_reference_shape(reference) {
        errors.push(format!(
            "{display}: {field} must cite a current test/task/spec artifact or cargo/cloud-ci gate command, got {reference:?}"
        ));
    }
}

fn validate_lane_reference(display: &str, field: &str, reference: &str, errors: &mut Vec<String>) {
    validate_not_retired_reference(display, field, reference, errors);
    if !is_slug_reference(reference) && !is_current_prd_reference_shape(reference) {
        errors.push(format!(
            "{display}: {field} must be a lowercase lane id or current command/artifact reference, got {reference:?}"
        ));
    }
}

fn validate_not_retired_reference(
    display: &str,
    field: &str,
    reference: &str,
    errors: &mut Vec<String>,
) {
    let lower = reference.to_ascii_lowercase();
    if lower.contains(".omc/")
        || lower.contains(".omx/")
        || lower.contains(".omc\\")
        || lower.contains(".omx\\")
        || lower.contains("implementation-plan")
        || lower.contains("implementation plan")
    {
        errors.push(format!(
            "{display}: {field} must cite product PRD/task/spec/cloud-ci evidence, not retired .omc/.omx implementation-plan inputs: {reference:?}"
        ));
    }
}

fn is_current_prd_reference_shape(reference: &str) -> bool {
    let trimmed = reference.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("cargo ")
        || lower.contains(" cargo ")
        || lower.starts_with("buck2 ")
        || lower.starts_with("bazel ")
        || lower.starts_with("jq ")
        || lower.starts_with("oya gate validate ")
        || lower.contains("cloud-ci")
        || lower.contains("oya-ci-required")
        || lower.contains("load-test")
        || lower.contains("future oya-")
        || lower.contains("$ref:docs/decisions/")
        || lower.contains("$ref:docs/adr-archive/")
        || [
            "crates/",
            "microservices/",
            "marketplace/",
            "tasks/",
            "specs/",
            "registry/",
            "docs/decisions/",
            "docs/adr-archive/",
        ]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
        || CAPABILITY_ROOT_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
}

/// Top-level directories the ADR-0562 capability registry declares legal, projected here as a
/// constant so reference-shape checking stays a pure function of the string.
///
/// This is a committed projection of `governance/capability-registry.json` (the closed placement
/// authority): every capability `name` plus every `meta_directories[].dir`, each as a path prefix.
/// `capability_root_prefixes_match_the_closed_registry` re-derives it from that file and fails on
/// any drift, so a newly registered capability cannot silently go unaccepted here.
const CAPABILITY_ROOT_PREFIXES: [&str; 31] = [
    "app/",
    "audit/",
    "base/",
    "billing/",
    "build/",
    "cell/",
    "ci/",
    "comms/",
    "compliance/",
    "compute/",
    "console/",
    "data/",
    "flags/",
    "gateway/",
    "governance/",
    "iac/",
    "iam/",
    "intelligence/",
    "k8s/",
    "kernel/",
    "marketplace/",
    "messaging/",
    "network/",
    "observability/",
    "os/",
    "policy/",
    "secrets/",
    "storage/",
    "tenancy/",
    "third-party/",
    "workflow/",
];

fn is_slug_reference(reference: &str) -> bool {
    let trimmed = reference.trim();
    !trimmed.is_empty()
        && trimmed.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && trimmed
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
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

        let report = validate_metrics(object, "test-product", false, &mut errors);

        assert_eq!(
            report,
            MetricShapeReport {
                metrics_checked: 1,
                verification_refs_checked: 0
            }
        );
        assert_eq!(
            errors,
            vec!["test-product: non-strict metric must still declare verification_ref or lane_ref"]
        );
    }

    #[test]
    fn acceptance_test_refs_reject_retired_plan_inputs() {
        let object = serde_json::json!({
            "acceptance_criteria": [
                {
                    "id": "AC-01",
                    "given": "a planned feature",
                    "when": "the product PRD is reviewed",
                    "then": "the test points at a live task/spec/cloud-ci artifact",
                    "test_ref": ".omc/plans/milestones/M02/impl-plan.md"
                }
            ]
        });
        let object = object.as_object().expect("test object");
        let mut errors = Vec::new();

        let report = validate_acceptance_criteria(object, "test-product", &mut errors);

        assert_eq!(
            report,
            AcceptanceShapeReport {
                criteria_checked: 1,
                test_refs_checked: 1
            }
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("retired .omc/.omx implementation-plan inputs")),
            "errors={errors:?}"
        );
    }

    #[test]
    fn planned_feature_refs_require_claim_boundary() {
        let product = serde_json::json!({
            "competitive": [
                {
                    "planned_verification_ref": "tasks/social-feed-ranking-score-plan.md",
                    "enforcement_status": "advisory_until_community_expansion_gates"
                }
            ]
        });
        let mut errors = Vec::new();

        let refs_checked = validate_planned_feature_refs(&product, "test-product", &mut errors);

        assert_eq!(refs_checked, 1);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("planned feature verification rows must declare")),
            "errors={errors:?}"
        );
    }

    #[test]
    fn explicit_product_paths_reject_unreadable_invalid_and_non_prd_inputs() {
        let repo_root = temp_prd_repo("explicit-inputs");
        let microservices = repo_root.join("specs/microservices");
        std::fs::create_dir_all(&microservices).expect("microservices dir");
        std::fs::write(
            microservices.join("not-prd.json"),
            r#"{"_meta":{"spec_id":"MSC-OTHER"}}"#,
        )
        .expect("non-prd fixture");
        std::fs::write(microservices.join("invalid.json"), "{not json").expect("invalid fixture");

        let missing_error = validate_product_prd_json_gate(ProductPrdJsonArgs {
            repo_root: repo_root.clone(),
            product_paths: vec![PathBuf::from("specs/microservices/missing.json")],
        })
        .expect_err("missing explicit input rejected");
        assert!(missing_error.contains("unable to read product PRD JSON"));
        assert!(missing_error.contains("zero active product PRD JSON specs checked"));

        let invalid_error = validate_product_prd_json_gate(ProductPrdJsonArgs {
            repo_root: repo_root.clone(),
            product_paths: vec![PathBuf::from("specs/microservices/invalid.json")],
        })
        .expect_err("invalid explicit input rejected");
        assert!(invalid_error.contains("invalid product PRD JSON"));

        let non_prd_error = validate_product_prd_json_gate(ProductPrdJsonArgs {
            repo_root,
            product_paths: vec![PathBuf::from("specs/microservices/not-prd.json")],
        })
        .expect_err("non-PRD explicit input rejected");
        assert!(non_prd_error.contains("explicit product path must be an active product PRD JSON"));
    }

    /// `CAPABILITY_ROOT_PREFIXES` is a committed projection of the closed capability registry.
    /// Re-derive it from that file so registering a capability without extending the projection
    /// fails here rather than silently rejecting every reference into the new root.
    #[test]
    fn capability_root_prefixes_match_the_closed_registry() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !root.join("governance/capability-registry.json").is_file() {
            assert!(
                root.pop(),
                "governance/capability-registry.json not found above {}",
                env!("CARGO_MANIFEST_DIR")
            );
        }

        let registry: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("governance/capability-registry.json"))
                .expect("read capability registry"),
        )
        .expect("parse capability registry");

        let mut expected = BTreeSet::new();
        for capability in registry["capabilities"]
            .as_array()
            .expect("capabilities is an array")
        {
            let name = capability["name"].as_str().expect("capability name");
            expected.insert(format!("{name}/"));
        }
        for meta in registry["meta_directories"]
            .as_array()
            .expect("meta_directories is an array")
        {
            let dir = meta["dir"].as_str().expect("meta dir");
            expected.insert(if dir.ends_with('/') {
                dir.to_owned()
            } else {
                format!("{dir}/")
            });
        }

        let projected: BTreeSet<String> = CAPABILITY_ROOT_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_owned())
            .collect();

        assert_eq!(
            projected, expected,
            "CAPABILITY_ROOT_PREFIXES has drifted from governance/capability-registry.json"
        );
    }

    #[test]
    fn prd_references_reject_unstructured_text() {
        let mut errors = Vec::new();

        validate_current_prd_reference(
            "test-product",
            "acceptance_criteria.test_ref",
            "trust me bro",
            &mut errors,
        );

        assert!(
            errors
                .iter()
                .any(|error| error.contains("current test/task/spec artifact")),
            "errors={errors:?}"
        );
    }

    fn temp_prd_repo(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "oya-product-prd-json-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("specs/microservices")).expect("temp specs dir");
        std::fs::write(
            root.join("specs/root-hub-pointers.json"),
            r#"{"entry_points":{}}"#,
        )
        .expect("root hub fixture");
        root
    }
    #[test]
    fn parse_rejects_unknown_flag() {
        let error = parse_product_prd_json_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }
}
