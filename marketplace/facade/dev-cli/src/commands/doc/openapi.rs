use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use data_boundary_kernel::parse_data_class_label;
use intelligence_api_semver_domain::validate_api_semver;
use intelligence_openapi_domain::{
    OpenApiContractMirrorLocation, OpenApiContractMirrorReport, OpenApiDocument,
    OpenApiRuntimeBinding, OpenApiRuntimeParityReport, OpenApiRuntimeSource, OpenApiSchemaBinding,
    OpenApiSchemaParityReport, OpenApiSourceReport, validate_openapi_contract_mirror,
    validate_openapi_documents, validate_openapi_runtime_parity, validate_openapi_schema_parity,
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
    let mut source = validate_openapi_documents(documents.clone())
        .map_err(|error| format!("OpenAPI source invalid: {error:?}"))?;
    let boundary_documents = read_openapi_boundary_documents(&args.contracts_dir)?;
    let boundary_data_class_annotations =
        validate_openapi_boundary_data_classes(&boundary_documents)
            .map_err(|error| format!("OpenAPI boundary data-class invalid: {error}"))?;
    source.documents_checked += boundary_documents.len();
    source.data_class_annotations_checked += boundary_data_class_annotations;
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryOpenApiDocument {
    path: String,
    contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryLogicalLine {
    indent: usize,
    text: String,
}

fn read_openapi_boundary_documents(
    contracts_dir: &Path,
) -> Result<Vec<BoundaryOpenApiDocument>, String> {
    if !contracts_dir.exists() {
        return Ok(Vec::new());
    }
    let mut documents = Vec::new();
    let mut entries = fs::read_dir(contracts_dir)
        .map_err(|error| {
            format!(
                "OpenAPI boundary contracts directory unreadable {}: {error}",
                contracts_dir.display()
            )
        })?
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|error| {
                format!("OpenAPI boundary contracts directory entry unreadable: {error}")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for path in entries {
        if !path.is_file() || !is_boundary_openapi_source_artifact(&path) {
            continue;
        }
        let relative = path.strip_prefix(contracts_dir).map_err(|error| {
            format!("OpenAPI boundary source path outside contracts dir: {error}")
        })?;
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "OpenAPI boundary source unreadable {}: {error}",
                path.display()
            )
        })?;
        documents.push(BoundaryOpenApiDocument {
            path: format!("contracts/{}", slash_path(relative)),
            contents,
        });
    }
    Ok(documents)
}

fn is_boundary_openapi_source_artifact(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    (file_name.ends_with(".openapi.yaml") || file_name.ends_with(".openapi.yml"))
        && !is_api_contract_metadata_path(path)
}

fn validate_openapi_boundary_data_classes(
    documents: &[BoundaryOpenApiDocument],
) -> Result<usize, String> {
    let mut annotations_checked = 0usize;
    for document in documents {
        annotations_checked += validate_boundary_document_data_classes(document)?;
    }
    Ok(annotations_checked)
}

fn validate_boundary_document_data_classes(
    document: &BoundaryOpenApiDocument,
) -> Result<usize, String> {
    let lines = boundary_logical_lines(&document.contents);
    let mut annotations_checked = 0usize;
    for (index, line) in lines.iter().enumerate() {
        if boundary_yaml_key(&line.text).is_some_and(|key| key == "properties") {
            annotations_checked +=
                validate_boundary_schema_properties_data_class(document, &lines, index)?;
        }
        if boundary_yaml_key(&line.text).is_some_and(|key| key == "parameters") {
            annotations_checked +=
                validate_boundary_parameters_data_class(document, &lines, index)?;
        }
    }
    Ok(annotations_checked)
}

fn validate_boundary_schema_properties_data_class(
    document: &BoundaryOpenApiDocument,
    lines: &[BoundaryLogicalLine],
    properties_index: usize,
) -> Result<usize, String> {
    let properties_indent = lines[properties_index].indent;
    let property_indent = properties_indent + 2;
    let end = boundary_find_next_at_or_above_indent(
        lines,
        properties_index + 1,
        lines.len(),
        properties_indent,
    );
    let schema_name = boundary_parent_schema_name(lines, properties_index, properties_indent);
    let mut annotations_checked = 0usize;
    let mut index = properties_index + 1;
    while index < end {
        let line = &lines[index];
        if line.indent != property_indent {
            index += 1;
            continue;
        }
        let Some(property_name) = boundary_yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        let property_end =
            boundary_find_next_at_or_above_indent(lines, index + 1, end, property_indent);
        let location = match &schema_name {
            Some(schema_name) => format!("schema {schema_name}.{property_name}"),
            None => format!("schema property {property_name}"),
        };
        annotations_checked += validate_boundary_data_class_annotation(
            document,
            &location,
            lines,
            index + 1..property_end,
        )?;
        index = property_end;
    }
    Ok(annotations_checked)
}

fn validate_boundary_parameters_data_class(
    document: &BoundaryOpenApiDocument,
    lines: &[BoundaryLogicalLine],
    parameters_index: usize,
) -> Result<usize, String> {
    let parameters_indent = lines[parameters_index].indent;
    let end = boundary_find_next_at_or_above_indent(
        lines,
        parameters_index + 1,
        lines.len(),
        parameters_indent,
    );
    let mut annotations_checked = 0usize;
    let mut index = parameters_index + 1;
    while index < end {
        let line = &lines[index];
        if !line.text.starts_with("- ") {
            index += 1;
            continue;
        }
        let parameter_end =
            boundary_find_next_at_or_above_indent(lines, index + 1, end, line.indent);
        if let Some(parameter_name) = boundary_parameter_name(lines, index, parameter_end) {
            annotations_checked += validate_boundary_data_class_annotation(
                document,
                &format!("parameter {parameter_name}"),
                lines,
                index..parameter_end,
            )?;
        }
        index = parameter_end;
    }
    Ok(annotations_checked)
}

fn validate_boundary_data_class_annotation(
    document: &BoundaryOpenApiDocument,
    location: &str,
    lines: &[BoundaryLogicalLine],
    range: std::ops::Range<usize>,
) -> Result<usize, String> {
    let data_class = lines[range].iter().find_map(|line| {
        if boundary_yaml_key(&line.text).is_some_and(|key| key == "x-oyatie-data-class") {
            boundary_yaml_value(&line.text).map(boundary_clean_yaml_scalar)
        } else {
            None
        }
    });
    let Some(data_class) = data_class else {
        return Err(format!(
            "{} missing data class at {location}",
            document.path
        ));
    };
    if parse_data_class_label(&data_class).is_none() {
        return Err(format!(
            "{} invalid data class at {location}: {data_class}",
            document.path
        ));
    }
    Ok(1)
}

fn boundary_logical_lines(contents: &str) -> Vec<BoundaryLogicalLine> {
    contents
        .lines()
        .filter_map(|line| {
            let text = line.trim_end_matches('\r');
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            Some(BoundaryLogicalLine {
                indent: text.len() - text.trim_start().len(),
                text: trimmed.to_string(),
            })
        })
        .collect()
}

fn boundary_parent_schema_name(
    lines: &[BoundaryLogicalLine],
    properties_index: usize,
    properties_indent: usize,
) -> Option<String> {
    let schema_indent = properties_indent.checked_sub(2)?;
    lines[..properties_index]
        .iter()
        .rev()
        .find(|line| line.indent == schema_indent)
        .and_then(|line| boundary_yaml_key(&line.text))
        .map(str::to_string)
}

fn boundary_parameter_name(
    lines: &[BoundaryLogicalLine],
    item_index: usize,
    item_end: usize,
) -> Option<String> {
    lines[item_index..item_end].iter().find_map(|line| {
        if boundary_yaml_key(&line.text).is_some_and(|key| key == "name") {
            boundary_yaml_value(&line.text).map(boundary_clean_yaml_scalar)
        } else {
            None
        }
    })
}

fn boundary_find_next_at_or_above_indent(
    lines: &[BoundaryLogicalLine],
    start: usize,
    end: usize,
    indent: usize,
) -> usize {
    let mut index = start;
    while index < end {
        if lines[index].indent <= indent {
            return index;
        }
        index += 1;
    }
    end
}

fn boundary_yaml_key(line: &str) -> Option<&str> {
    let (key, _) = line.split_once(':')?;
    Some(boundary_clean_yaml_key(key.trim().trim_start_matches("- ")))
}

fn boundary_yaml_value(line: &str) -> Option<&str> {
    let (_, value) = line.split_once(':')?;
    let value = value.trim();
    if value.is_empty() { None } else { Some(value) }
}

fn boundary_clean_yaml_key(value: &str) -> &str {
    value.trim().trim_matches('"').trim_matches('\'').trim()
}

fn boundary_clean_yaml_scalar(value: &str) -> String {
    boundary_clean_yaml_key(value.trim()).to_string()
}

fn is_openapi_source_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(extension, "yaml" | "yml") && !is_api_contract_metadata_path(path)
}
