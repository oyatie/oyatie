use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_api_semver_kernel::validate_api_semver;
use oya_foundry_openapi_kernel::{
    validate_openapi_contract_mirror, validate_openapi_documents, validate_openapi_runtime_parity,
    validate_openapi_schema_parity, OpenApiContractMirrorLocation, OpenApiContractMirrorReport,
    OpenApiDocument, OpenApiRuntimeBinding, OpenApiRuntimeParityReport, OpenApiRuntimeSource,
    OpenApiSchemaBinding, OpenApiSchemaParityReport, OpenApiSourceReport,
};

use crate::command_output::OutputFormat as DevCheckOutputFormat;
use crate::{
    is_api_contract_metadata_path, read_api_contract_records, read_cross_axis_contracts, slash_path,
};

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_doc_openapi_args(args, usage) {
        Ok(args) => run_doc_openapi(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocOpenApiArgs {
    contracts_dir: PathBuf,
    spec_path: PathBuf,
    contracts_mirror_path: PathBuf,
    runtime_bindings_path: PathBuf,
    schema_bindings_path: PathBuf,
    runtime_root: PathBuf,
    output_format: DevCheckOutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocOpenApiReport {
    source: OpenApiSourceReport,
    mirror: OpenApiContractMirrorReport,
    runtime: OpenApiRuntimeParityReport,
    schema: OpenApiSchemaParityReport,
    contracts_checked: usize,
    metadata_checked: usize,
}

fn parse_doc_openapi_args(args: Vec<String>, usage: &str) -> Result<DocOpenApiArgs, String> {
    let mut parsed = DocOpenApiArgs {
        contracts_dir: PathBuf::from("contracts"),
        spec_path: PathBuf::from("docs/SPEC.md"),
        contracts_mirror_path: PathBuf::from("docs/machine-readable/contracts.json"),
        runtime_bindings_path: PathBuf::from("registry/openapi/runtime-bindings.tsv"),
        schema_bindings_path: PathBuf::from("registry/openapi/schema-bindings.tsv"),
        runtime_root: PathBuf::from("."),
        output_format: DevCheckOutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--contracts-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.contracts_dir = PathBuf::from(value);
            }
            "--spec" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.spec_path = PathBuf::from(value);
            }
            "--contracts-mirror" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.contracts_mirror_path = PathBuf::from(value);
            }
            "--runtime-bindings" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.runtime_bindings_path = PathBuf::from(value);
            }
            "--schema-bindings" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.schema_bindings_path = PathBuf::from(value);
            }
            "--runtime-root" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.runtime_root = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output_format =
                    DevCheckOutputFormat::parse(&value).ok_or_else(|| usage.to_owned())?;
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn run_doc_openapi(args: DocOpenApiArgs) -> ExitCode {
    match run_doc_openapi_result(&args) {
        Ok(report) => match args.output_format {
            DevCheckOutputFormat::Text => {
                println!(
                    "OpenAPI documentation validation passed: {} documents, {} operations, {} data-class annotations, {} runtime bindings, {} runtime sources, {} runtime tests, {} runtime response statuses, {} runtime response schemas, {} schema bindings, {} schema fields, {} schema types, {} contracts, {} metadata records, {} mirrored contracts",
                    report.source.documents_checked,
                    report.source.operations_checked,
                    report.source.data_class_annotations_checked,
                    report.runtime.bindings_checked,
                    report.runtime.sources_checked,
                    report.runtime.tests_checked,
                    report.runtime.response_statuses_checked,
                    report.runtime.response_schemas_checked,
                    report.schema.bindings_checked,
                    report.schema.fields_checked,
                    report.schema.types_checked,
                    report.contracts_checked,
                    report.metadata_checked,
                    report.mirror.contracts_checked
                );
                ExitCode::SUCCESS
            }
            DevCheckOutputFormat::Json => {
                println!(
                    "{{\"command\":\"oya doc openapi\",\"status\":\"passed\",\"documents\":{},\"operations\":{},\"data_class_annotations\":{},\"runtime_bindings\":{},\"runtime_sources\":{},\"runtime_tests\":{},\"runtime_response_statuses\":{},\"runtime_response_schemas\":{},\"schema_bindings\":{},\"schema_fields\":{},\"schema_types\":{},\"contracts\":{},\"metadata\":{},\"mirrored_contracts\":{}}}",
                    report.source.documents_checked,
                    report.source.operations_checked,
                    report.source.data_class_annotations_checked,
                    report.runtime.bindings_checked,
                    report.runtime.sources_checked,
                    report.runtime.tests_checked,
                    report.runtime.response_statuses_checked,
                    report.runtime.response_schemas_checked,
                    report.schema.bindings_checked,
                    report.schema.fields_checked,
                    report.schema.types_checked,
                    report.contracts_checked,
                    report.metadata_checked,
                    report.mirror.contracts_checked
                );
                ExitCode::SUCCESS
            }
        },
        Err(message) => {
            eprintln!("OpenAPI documentation validation failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_doc_openapi_result(args: &DocOpenApiArgs) -> Result<DocOpenApiReport, String> {
    let documents = read_openapi_documents(&args.contracts_dir)?;
    let source = validate_openapi_documents(documents.clone())
        .map_err(|error| format!("OpenAPI source invalid: {error:?}"))?;
    let spec_contents = fs::read_to_string(&args.spec_path).map_err(|error| {
        format!(
            "SPEC mirror unreadable {}: {error}",
            args.spec_path.display()
        )
    })?;
    let mirror_locations = read_openapi_contract_mirror_locations(&args.contracts_mirror_path)?;
    let mirror = validate_openapi_contract_mirror(
        documents.iter().map(|document| document.path.as_str()),
        &spec_contents,
        mirror_locations,
    )
    .map_err(|error| format!("OpenAPI contract mirror invalid: {error:?}"))?;
    let runtime_bindings = read_openapi_runtime_bindings(&args.runtime_bindings_path)?;
    let runtime_sources = read_openapi_runtime_artifacts(
        &args.runtime_root,
        runtime_bindings
            .iter()
            .map(|binding| binding.source_path.as_str()),
        "OpenAPI runtime source",
    )?;
    let runtime_tests = read_openapi_runtime_artifacts(
        &args.runtime_root,
        runtime_bindings
            .iter()
            .map(|binding| binding.test_path.as_str()),
        "OpenAPI runtime test",
    )?;
    let runtime = validate_openapi_runtime_parity(
        documents.clone(),
        runtime_bindings,
        runtime_sources,
        runtime_tests,
    )
    .map_err(|error| format!("OpenAPI runtime parity invalid: {error:?}"))?;
    let schema_bindings = read_openapi_schema_bindings(&args.schema_bindings_path)?;
    let schema_sources = read_openapi_runtime_artifacts(
        &args.runtime_root,
        schema_bindings
            .iter()
            .map(|binding| binding.source_path.as_str()),
        "OpenAPI schema runtime source",
    )?;
    let schema = validate_openapi_schema_parity(documents.clone(), schema_bindings, schema_sources)
        .map_err(|error| format!("OpenAPI schema parity invalid: {error:?}"))?;
    let records = read_api_contract_records(&args.contracts_dir)?;
    let semver =
        validate_api_semver(records).map_err(|error| format!("API semver invalid: {error:?}"))?;
    Ok(DocOpenApiReport {
        source,
        mirror,
        runtime,
        schema,
        contracts_checked: semver.contracts_checked,
        metadata_checked: semver.metadata_checked,
    })
}

fn read_openapi_contract_mirror_locations(
    path: &Path,
) -> Result<Vec<OpenApiContractMirrorLocation>, String> {
    read_cross_axis_contracts(path).map(|contracts| {
        contracts
            .into_iter()
            .map(|contract| OpenApiContractMirrorLocation {
                contract_id: contract.id,
                location: contract.location,
            })
            .collect()
    })
}

fn read_openapi_runtime_bindings(path: &Path) -> Result<Vec<OpenApiRuntimeBinding>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "OpenAPI runtime bindings registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut records = Vec::new();
    let mut seen_header = false;
    for (line_index, line) in contents.lines().enumerate() {
        let row = line.trim_end_matches('\r');
        let visible = row.trim();
        if visible.is_empty() || visible.starts_with('#') {
            continue;
        }
        if visible.starts_with("operation_id\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}:{} expected OpenAPI runtime bindings TSV header",
                path.display(),
                line_index + 1
            ));
        }
        records.push(parse_openapi_runtime_binding(path, line_index + 1, row)?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing OpenAPI runtime bindings TSV header",
            path.display()
        ));
    }
    Ok(records)
}

fn parse_openapi_runtime_binding(
    path: &Path,
    line_number: usize,
    row: &str,
) -> Result<OpenApiRuntimeBinding, String> {
    let cells = row
        .split('\t')
        .map(|cell| cell.trim().to_string())
        .collect::<Vec<_>>();
    if cells.len() != 9 {
        return Err(format!(
            "{}:{line_number} OpenAPI runtime binding row must have 9 columns",
            path.display()
        ));
    }
    Ok(OpenApiRuntimeBinding {
        operation_id: cells[0].clone(),
        contract_path: cells[1].clone(),
        runtime_crate: cells[2].clone(),
        source_path: cells[3].clone(),
        symbol: cells[4].clone(),
        status_type: cells[5].clone(),
        evidence_surface: cells[6].clone(),
        test_path: cells[7].clone(),
        response_schemas: parse_openapi_response_schemas(path, line_number, &cells[8])?,
    })
}

fn parse_openapi_response_schemas(
    path: &Path,
    line_number: usize,
    value: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut schemas = BTreeMap::new();
    for pair in value.split(';') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let Some((status, schema)) = pair.split_once('=') else {
            return Err(format!(
                "{}:{line_number} OpenAPI response schema mapping must use status=schema pairs",
                path.display()
            ));
        };
        let status = status.trim().to_string();
        let schema = schema.trim().to_string();
        if status.is_empty() || schema.is_empty() {
            return Err(format!(
                "{}:{line_number} OpenAPI response schema mapping must not contain empty status or schema",
                path.display()
            ));
        }
        if schemas.insert(status.clone(), schema).is_some() {
            return Err(format!(
                "{}:{line_number} duplicate OpenAPI response schema mapping for status {status}",
                path.display()
            ));
        }
    }
    if schemas.is_empty() {
        return Err(format!(
            "{}:{line_number} OpenAPI response schema mapping must be non-empty",
            path.display()
        ));
    }
    Ok(schemas)
}

fn read_openapi_schema_bindings(path: &Path) -> Result<Vec<OpenApiSchemaBinding>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "OpenAPI schema bindings registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut records = Vec::new();
    let mut seen_header = false;
    for (line_index, line) in contents.lines().enumerate() {
        let row = line.trim_end_matches('\r');
        let visible = row.trim();
        if visible.is_empty() || visible.starts_with('#') {
            continue;
        }
        if visible.starts_with("schema_name\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}:{} expected OpenAPI schema bindings TSV header",
                path.display(),
                line_index + 1
            ));
        }
        records.push(parse_openapi_schema_binding(path, line_index + 1, row)?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing OpenAPI schema bindings TSV header",
            path.display()
        ));
    }
    Ok(records)
}

fn parse_openapi_schema_binding(
    path: &Path,
    line_number: usize,
    row: &str,
) -> Result<OpenApiSchemaBinding, String> {
    let cells = row
        .split('\t')
        .map(|cell| cell.trim().to_string())
        .collect::<Vec<_>>();
    if cells.len() != 5 {
        return Err(format!(
            "{}:{line_number} OpenAPI schema binding row must have 5 columns",
            path.display()
        ));
    }
    Ok(OpenApiSchemaBinding {
        schema_name: cells[0].clone(),
        contract_path: cells[1].clone(),
        runtime_crate: cells[2].clone(),
        source_path: cells[3].clone(),
        rust_struct: cells[4].clone(),
    })
}

fn read_openapi_runtime_artifacts<'a, I>(
    runtime_root: &Path,
    paths: I,
    label: &str,
) -> Result<Vec<OpenApiRuntimeSource>, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut unique_paths = BTreeSet::new();
    for path in paths {
        unique_paths.insert(path.to_string());
    }
    unique_paths
        .into_iter()
        .map(|path| {
            let contents = fs::read_to_string(runtime_root.join(&path)).map_err(|error| {
                format!(
                    "{label} unreadable {}: {error}",
                    runtime_root.join(&path).display()
                )
            })?;
            Ok(OpenApiRuntimeSource { path, contents })
        })
        .collect()
}

fn read_openapi_documents(contracts_dir: &Path) -> Result<Vec<OpenApiDocument>, String> {
    let openapi_dir = contracts_dir.join("openapi");
    if !openapi_dir.exists() {
        return Ok(Vec::new());
    }
    let mut documents = Vec::new();
    collect_openapi_documents(contracts_dir, &openapi_dir, &mut documents)?;
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn collect_openapi_documents(
    contracts_dir: &Path,
    current: &Path,
    documents: &mut Vec<OpenApiDocument>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| {
            format!(
                "OpenAPI contracts directory unreadable {}: {error}",
                current.display()
            )
        })?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("OpenAPI contracts directory entry unreadable: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_openapi_documents(contracts_dir, &path, documents)?;
            continue;
        }
        if !path.is_file() || !is_openapi_source_artifact(&path) {
            continue;
        }
        let relative = path
            .strip_prefix(contracts_dir)
            .map_err(|error| format!("OpenAPI source path outside contracts dir: {error}"))?;
        let normalized_path = format!("contracts/{}", slash_path(relative));
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("OpenAPI source unreadable {}: {error}", path.display()))?;
        documents.push(OpenApiDocument {
            path: normalized_path,
            contents,
        });
    }
    Ok(())
}

fn is_openapi_source_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(extension, "yaml" | "yml") && !is_api_contract_metadata_path(path)
}
