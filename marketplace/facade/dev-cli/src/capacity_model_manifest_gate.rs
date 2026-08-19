//! ADR-0340 capacity_model manifest gate.
//!
//! Validates the machine-readable `capacity_model` block on workload-producing
//! service manifests. This is local bridge evidence only; protected merge
//! authority remains cloud-ci/oya-ci per ADR-0515.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Map, Value};

const SCALING_DIMENSIONS: &[&str] = &[
    "per_user",
    "per_request",
    "per_capability",
    "per_message",
    "per_query",
    "per_workflow_run",
];
const CELL_PLACEMENT_CLASSES: &[&str] = &["Tier-0", "Tier-1", "Tier-2", "Tier-3", "Tier-4"];
const CONNECTION_KINDS: &[&str] = &["valkey", "postgres", "outbound_http"];
const TENANT_CLASSES: &[&str] = &["demo_trial", "paid"];
const COMPLIANCE_PACKS: &[&str] = &[
    "hipaa",
    "pci-dss",
    "gdpr-strict",
    "soc2",
    "csap",
    "eu-ai-act-annex-iii",
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapacityModelArgs {
    roots: Vec<PathBuf>,
    manifests: Vec<PathBuf>,
    require_tenant_class_deltas: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestDocument {
    microservice: String,
    path: PathBuf,
    contents: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CapacityModelReport {
    manifests_checked: usize,
    capacity_models_checked: usize,
    tenant_class_deltas_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapacityModelFinding {
    microservice: String,
    path: PathBuf,
    pointer: String,
    message: String,
}

pub(crate) fn run_capacity_model_manifest(args: Vec<String>, usage: &str) -> ExitCode {
    let args = match parse_capacity_model_args(args) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n{usage}");
            return ExitCode::from(2);
        }
    };

    let manifest_paths = if args.manifests.is_empty() {
        // Only now, with no explicit manifests, do we need service roots.
        // Resolving lazily keeps an explicit --manifest invocation working
        // outside a repository checkout.
        let roots = if args.roots.is_empty() {
            match crate::service_roots::default_service_roots() {
                Ok(roots) => roots,
                Err(error) => {
                    eprintln!("capacity-model-manifest FAILED: {error}");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            args.roots.clone()
        };
        list_manifest_paths_from_roots(&roots)
    } else {
        args.manifests.clone()
    };

    let documents = match read_manifest_documents(&manifest_paths) {
        Ok(documents) => documents,
        Err(message) => {
            eprintln!("capacity-model-manifest FAILED: {message}");
            return ExitCode::FAILURE;
        }
    };

    let (report, findings) = audit_capacity_models(&documents, args.require_tenant_class_deltas);
    if findings.is_empty() {
        println!(
            "capacity-model-manifest passed: {} manifests, {} capacity_model blocks, {} tenant_class_deltas",
            report.manifests_checked,
            report.capacity_models_checked,
            report.tenant_class_deltas_checked
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "capacity-model-manifest FAILED: {} findings",
            findings.len()
        );
        for finding in &findings {
            eprintln!(
                "  - {} ({}): {} — {}",
                finding.microservice,
                finding.path.display(),
                finding.pointer,
                finding.message
            );
        }
        ExitCode::FAILURE
    }
}

fn parse_capacity_model_args(args: Vec<String>) -> Result<CapacityModelArgs, String> {
    let mut explicit_roots = Vec::new();
    let mut manifests = Vec::new();
    let mut require_tenant_class_deltas = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--microservices-root" => {
                let Some(root) = iter.next() else {
                    return Err("--microservices-root requires a path".to_string());
                };
                explicit_roots.push(PathBuf::from(root));
            }
            "--manifest" => {
                let Some(path) = iter.next() else {
                    return Err("--manifest requires a path".to_string());
                };
                manifests.push(PathBuf::from(path));
            }
            "--require-tenant-class-deltas" => require_tenant_class_deltas = true,
            other => return Err(format!("unknown capacity-model-manifest flag: {other}")),
        }
    }

    // Explicit --microservices-root roots are carried through as given; an
    // empty list means "fall back to the shared, registry-derived default
    // set", resolved at point of use in `run` so that an explicit
    // --manifest invocation never needs a repository checkout.
    let roots = explicit_roots;

    Ok(CapacityModelArgs {
        roots,
        manifests,
        require_tenant_class_deltas,
    })
}

/// Every `manifest.json` under the given roots, in BOTH layout shapes
/// (`<root>/manifest.json` and `<root>/<service>/manifest.json`). The
/// predecessor walked depth 2 only, so depth-1 manifests never reached the
/// gate.
fn list_manifest_paths_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for root in roots {
        out.extend(
            crate::service_roots::list_service_files(root, "manifest.json")
                .into_iter()
                .map(|found| found.path),
        );
    }
    out.sort();
    out.dedup();
    out
}

fn read_manifest_documents(paths: &[PathBuf]) -> Result<Vec<ManifestDocument>, String> {
    if paths.is_empty() {
        return Err("no manifest.json files found".to_string());
    }

    let mut documents = Vec::with_capacity(paths.len());
    for path in paths {
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("{} unreadable: {error}", path.display()))?;
        documents.push(ManifestDocument {
            microservice: microservice_name_for(path),
            path: path.clone(),
            contents,
        });
    }
    Ok(documents)
}

/// The owning microservice for a manifest path.
///
/// The predecessor first searched the path for a literal `cloud` / `oya` /
/// `microservices` component and took the segment AFTER it. Two of those
/// three markers no longer name a root, and the surviving scan could still
/// match a same-named component nested anywhere in the path — naming the
/// wrong service. The enclosing directory, which was only the fallback
/// before, is the answer in every live layout shape.
fn microservice_name_for(path: &Path) -> String {
    crate::service_roots::microservice_name_for(path).unwrap_or_else(|| "unknown".to_string())
}

fn audit_capacity_models(
    documents: &[ManifestDocument],
    require_tenant_class_deltas: bool,
) -> (CapacityModelReport, Vec<CapacityModelFinding>) {
    let mut report = CapacityModelReport {
        manifests_checked: documents.len(),
        ..CapacityModelReport::default()
    };
    let mut findings = Vec::new();

    for document in documents {
        let parsed = match serde_json::from_str::<Value>(&document.contents) {
            Ok(parsed) => parsed,
            Err(error) => {
                findings.push(finding(
                    document,
                    "$",
                    format!("manifest JSON parse failed: {error}"),
                ));
                continue;
            }
        };

        let Some(model) = parsed.get("capacity_model") else {
            continue;
        };

        if is_non_runtime_capacity_placeholder(&parsed, model) {
            continue;
        }

        report.capacity_models_checked += 1;
        validate_capacity_model(
            document,
            "capacity_model",
            model,
            require_tenant_class_deltas,
            true,
            &mut report,
            &mut findings,
        );
    }

    if report.capacity_models_checked == 0 {
        findings.push(CapacityModelFinding {
            microservice: "*".to_string(),
            path: PathBuf::from("<scan>"),
            pointer: "capacity_model".to_string(),
            message: "no capacity_model blocks found in scanned manifests".to_string(),
        });
    }

    (report, findings)
}

fn is_non_runtime_capacity_placeholder(parsed: &Value, model: &Value) -> bool {
    let has_no_runtime_exemption = parsed
        .pointer("/slo_exemption/status")
        .and_then(Value::as_str)
        .is_some_and(|status| status.contains("no_runtime"));
    let scaling_is_non_claim =
        model.get("scaling_dimension").and_then(Value::as_str) == Some("not_claimed_runtime");
    let placement_is_future_intent = model
        .get("cell_placement_class")
        .and_then(Value::as_str)
        .is_some_and(|placement| {
            placement.ends_with("-intended-future") || placement.starts_with("not_claimed")
        });
    let notes_declares_non_claim = model
        .get("notes")
        .and_then(Value::as_str)
        .is_some_and(|notes| notes.contains("non-claim") || notes.contains("non-claims"));

    scaling_is_non_claim
        && (has_no_runtime_exemption || placement_is_future_intent || notes_declares_non_claim)
}

fn validate_capacity_model(
    document: &ManifestDocument,
    pointer: &str,
    value: &Value,
    require_tenant_class_deltas: bool,
    validate_children: bool,
    report: &mut CapacityModelReport,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let Some(object) = value.as_object() else {
        findings.push(finding(
            document,
            pointer,
            "capacity_model must be a JSON object".to_string(),
        ));
        return;
    };

    validate_capacity_model_keys(document, pointer, object, findings);
    validate_number_field(
        document,
        pointer,
        object,
        "baseline_cpu_per_tenant",
        0.001,
        1000.0,
        findings,
    );
    validate_integer_field(
        document,
        pointer,
        object,
        "baseline_ram_per_tenant",
        1,
        1_048_576,
        findings,
    );
    validate_integer_field(
        document,
        pointer,
        object,
        "storage_per_tenant",
        0,
        1_048_576,
        findings,
    );
    validate_connections(document, pointer, object, findings);
    validate_enum_field(
        document,
        pointer,
        object,
        "scaling_dimension",
        SCALING_DIMENSIONS,
        findings,
    );
    validate_enum_field(
        document,
        pointer,
        object,
        "cell_placement_class",
        CELL_PLACEMENT_CLASSES,
        findings,
    );
    validate_notes(document, pointer, object, findings);

    if validate_children {
        validate_tenant_class_deltas(
            document,
            pointer,
            object,
            require_tenant_class_deltas,
            report,
            findings,
        );
        validate_compliance_pack_overrides(document, pointer, object, report, findings);
    }
}

fn validate_capacity_model_keys(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let allowed = [
        "baseline_cpu_per_tenant",
        "baseline_ram_per_tenant",
        "storage_per_tenant",
        "connections_per_tenant",
        "scaling_dimension",
        "cell_placement_class",
        "tenant_class_deltas",
        "compliance_pack_overrides",
        "notes",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    for key in object.keys() {
        if !allowed.contains(key.as_str()) {
            findings.push(finding(
                document,
                &format!("{pointer}.{key}"),
                "unknown capacity_model field; ADR-0340 schema is additionalProperties=false"
                    .to_string(),
            ));
        }
    }
}

fn validate_number_field(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    field: &str,
    min: f64,
    max: f64,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field_pointer = format!("{pointer}.{field}");
    let Some(value) = object.get(field) else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} is required"),
        ));
        return;
    };
    let Some(number) = value.as_f64() else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be a JSON number"),
        ));
        return;
    };
    if !(min..=max).contains(&number) {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be between {min} and {max}"),
        ));
    }
}

fn validate_integer_field(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    field: &str,
    min: i64,
    max: i64,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field_pointer = format!("{pointer}.{field}");
    let Some(value) = object.get(field) else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} is required"),
        ));
        return;
    };
    let Some(integer) = value.as_i64() else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be a JSON integer"),
        ));
        return;
    };
    if !(min..=max).contains(&integer) {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be between {min} and {max}"),
        ));
    }
}

fn validate_connections(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field = "connections_per_tenant";
    let field_pointer = format!("{pointer}.{field}");
    let Some(value) = object.get(field) else {
        findings.push(finding(
            document,
            &field_pointer,
            "connections_per_tenant is required".to_string(),
        ));
        return;
    };
    let Some(connections) = value.as_object() else {
        findings.push(finding(
            document,
            &field_pointer,
            "connections_per_tenant must be a JSON object".to_string(),
        ));
        return;
    };

    let allowed = CONNECTION_KINDS.iter().copied().collect::<BTreeSet<_>>();
    for key in connections.keys() {
        if !allowed.contains(key.as_str()) {
            findings.push(finding(
                document,
                &format!("{field_pointer}.{key}"),
                "unknown connections_per_tenant field; allowed values are valkey, postgres, outbound_http".to_string(),
            ));
        }
    }
    for kind in CONNECTION_KINDS {
        validate_integer_field(
            document,
            &field_pointer,
            connections,
            kind,
            0,
            1024,
            findings,
        );
    }
}

fn validate_enum_field(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    field: &str,
    allowed: &[&str],
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field_pointer = format!("{pointer}.{field}");
    let Some(value) = object.get(field) else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} is required"),
        ));
        return;
    };
    let Some(found) = value.as_str() else {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be a string enum"),
        ));
        return;
    };
    if !allowed.contains(&found) {
        findings.push(finding(
            document,
            &field_pointer,
            format!("{field} must be one of {}", allowed.join(", ")),
        ));
    }
}

fn validate_notes(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    findings: &mut Vec<CapacityModelFinding>,
) {
    if let Some(notes) = object.get("notes")
        && !notes.is_string()
    {
        findings.push(finding(
            document,
            &format!("{pointer}.notes"),
            "notes must be a string".to_string(),
        ));
    }
}

fn validate_tenant_class_deltas(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    require_tenant_class_deltas: bool,
    report: &mut CapacityModelReport,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field_pointer = format!("{pointer}.tenant_class_deltas");
    let Some(value) = object.get("tenant_class_deltas") else {
        if require_tenant_class_deltas {
            findings.push(finding(
                document,
                &field_pointer,
                "capacity_model.tenant_class_deltas is required by this gate invocation"
                    .to_string(),
            ));
        }
        return;
    };
    let Some(deltas) = value.as_object() else {
        findings.push(finding(
            document,
            &field_pointer,
            "tenant_class_deltas must be a JSON object".to_string(),
        ));
        return;
    };

    report.tenant_class_deltas_checked += 1;
    let allowed = TENANT_CLASSES.iter().copied().collect::<BTreeSet<_>>();
    for key in deltas.keys() {
        if !allowed.contains(key.as_str()) {
            findings.push(finding(
                document,
                &format!("{field_pointer}.{key}"),
                "unknown tenant_class_deltas key; allowed values are demo_trial and paid"
                    .to_string(),
            ));
        }
    }
    for tenant_class in TENANT_CLASSES {
        let child_pointer = format!("{field_pointer}.{tenant_class}");
        let Some(delta) = deltas.get(*tenant_class) else {
            findings.push(finding(
                document,
                &child_pointer,
                format!("tenant_class_deltas.{tenant_class} is required when tenant_class_deltas is present"),
            ));
            continue;
        };
        validate_capacity_model(
            document,
            &child_pointer,
            delta,
            false,
            false,
            report,
            findings,
        );
    }
}

fn validate_compliance_pack_overrides(
    document: &ManifestDocument,
    pointer: &str,
    object: &Map<String, Value>,
    report: &mut CapacityModelReport,
    findings: &mut Vec<CapacityModelFinding>,
) {
    let field_pointer = format!("{pointer}.compliance_pack_overrides");
    let Some(value) = object.get("compliance_pack_overrides") else {
        return;
    };
    let Some(overrides) = value.as_object() else {
        findings.push(finding(
            document,
            &field_pointer,
            "compliance_pack_overrides must be a JSON object".to_string(),
        ));
        return;
    };

    let allowed = COMPLIANCE_PACKS.iter().copied().collect::<BTreeSet<_>>();
    for (pack, override_model) in overrides {
        let child_pointer = format!("{field_pointer}.{pack}");
        if !allowed.contains(pack.as_str()) {
            findings.push(finding(
                document,
                &child_pointer,
                "unknown compliance_pack_overrides key".to_string(),
            ));
            continue;
        }
        validate_capacity_model(
            document,
            &child_pointer,
            override_model,
            false,
            false,
            report,
            findings,
        );
    }
}

fn finding(document: &ManifestDocument, pointer: &str, message: String) -> CapacityModelFinding {
    CapacityModelFinding {
        microservice: document.microservice.clone(),
        path: document.path.clone(),
        pointer: pointer.to_string(),
        message,
    }
}
