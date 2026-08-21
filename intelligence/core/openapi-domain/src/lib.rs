//! OpenAPI reference source fitness kernel.
//!
//! Public REST contracts become tenant- and ISV-facing promises. This pure
//! kernel validates the minimum OpenAPI 3.2 source shape that `oya doc openapi`
//! may publish: contract paths are versioned, `info.version` agrees with the
//! path suffix, every document declares paths, and every operation carries an
//! operation id plus at least one response. Adapters own filesystem discovery
//! and semver metadata parsing.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::parse_data_class_label;

const OPENAPI_PREFIX: &str = "contracts/openapi/";
const SUPPORTED_OPENAPI_MAJOR_MINOR: &str = "3.2.";
const BEARER_SECURITY_SCHEME: &str = "bearerAuth";
const ADDITIONAL_OPERATIONS_FIELD: &str = "additionalOperations";
const REQUIRED_MUTATING_HEADERS: [&str; 3] = ["X-Request-Id", "X-Tenant-Id", "Idempotency-Key"];
const FIXED_OPERATION_METHODS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenApiSourceReport {
    pub documents_checked: usize,              // data_class: INTERNAL_ONLY
    pub operations_checked: usize,             // data_class: INTERNAL_ONLY
    pub data_class_annotations_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiContractMirrorLocation {
    pub contract_id: String, // data_class: INTERNAL_ONLY
    pub location: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenApiContractMirrorReport {
    pub contracts_checked: usize,         // data_class: INTERNAL_ONLY
    pub spec_references_checked: usize,   // data_class: INTERNAL_ONLY
    pub mirror_references_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiRuntimeBinding {
    pub operation_id: String,                       // data_class: INTERNAL_ONLY
    pub contract_path: String,                      // data_class: INTERNAL_ONLY
    pub runtime_crate: String,                      // data_class: INTERNAL_ONLY
    pub source_path: String,                        // data_class: INTERNAL_ONLY
    pub symbol: String,                             // data_class: INTERNAL_ONLY
    pub status_type: String,                        // data_class: INTERNAL_ONLY
    pub evidence_surface: String,                   // data_class: INTERNAL_ONLY
    pub test_path: String,                          // data_class: INTERNAL_ONLY
    pub response_schemas: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiRuntimeSource {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenApiRuntimeParityReport {
    pub operations_checked: usize,        // data_class: INTERNAL_ONLY
    pub bindings_checked: usize,          // data_class: INTERNAL_ONLY
    pub sources_checked: usize,           // data_class: INTERNAL_ONLY
    pub tests_checked: usize,             // data_class: INTERNAL_ONLY
    pub response_statuses_checked: usize, // data_class: INTERNAL_ONLY
    pub response_schemas_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiSchemaBinding {
    pub schema_name: String,   // data_class: INTERNAL_ONLY
    pub contract_path: String, // data_class: INTERNAL_ONLY
    pub runtime_crate: String, // data_class: INTERNAL_ONLY
    pub source_path: String,   // data_class: INTERNAL_ONLY
    pub rust_struct: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenApiSchemaParityReport {
    pub schemas_checked: usize,  // data_class: INTERNAL_ONLY
    pub bindings_checked: usize, // data_class: INTERNAL_ONLY
    pub sources_checked: usize,  // data_class: INTERNAL_ONLY
    pub fields_checked: usize,   // data_class: INTERNAL_ONLY
    pub types_checked: usize,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenApiSourceError {
    NoDocuments,
    InvalidPath {
        path: String,
        reason: String,
    },
    DuplicateDocument {
        path: String,
    },
    EmptyDocument {
        path: String,
    },
    MissingTopLevelField {
        path: String,
        field: &'static str,
    },
    UnsupportedOpenApiVersion {
        path: String,
        version: String,
    },
    MissingInfoField {
        path: String,
        field: &'static str,
    },
    InvalidInfoVersion {
        path: String,
        version: String,
    },
    VersionSuffixMismatch {
        path: String,
        path_major: u64,
        info_major: u64,
    },
    MissingPathItem {
        path: String,
    },
    MissingOperation {
        path: String,
        api_path: String,
    },
    AdditionalOperationFixedMethodCollision {
        path: String,
        api_path: String,
        method: String,
    },
    MissingOperationId {
        path: String,
        api_path: String,
        method: String,
    },
    MissingResponses {
        path: String,
        api_path: String,
        method: String,
    },
    MissingSpecMirror {
        path: String,
    },
    MissingMachineMirror {
        path: String,
    },
    StaleSpecMirror {
        path: String,
    },
    StaleMachineMirror {
        contract_id: String,
        path: String,
    },
    MissingDataClassAnnotation {
        path: String,
        location: String,
    },
    InvalidDataClassAnnotation {
        path: String,
        location: String,
        data_class: String,
    },
    MissingBearerSecurityScheme {
        path: String,
    },
    InvalidBearerSecurityScheme {
        path: String,
        reason: String,
    },
    MissingOperationSecurity {
        path: String,
        api_path: String,
        method: String,
        scheme: String,
    },
    ForbiddenAuthorizationParameter {
        path: String,
        api_path: String,
        method: String,
    },
    MissingRequiredHeaderParameter {
        path: String,
        api_path: String,
        method: String,
        header: String,
    },
    InvalidHeaderParameter {
        path: String,
        api_path: String,
        method: String,
        header: String,
        reason: String,
    },
    MissingPathTemplateParameter {
        path: String,
        api_path: String,
        method: String,
        parameter: String,
    },
    InvalidPathTemplateParameter {
        path: String,
        api_path: String,
        method: String,
        parameter: String,
        reason: String,
    },
    DuplicateOperationId {
        operation_id: String,
        first_path: String,
        second_path: String,
    },
    MissingRuntimeBinding {
        operation_id: String,
        contract_path: String,
    },
    DuplicateRuntimeBinding {
        operation_id: String,
    },
    StaleRuntimeBinding {
        operation_id: String,
        contract_path: String,
    },
    NonExplicitRuntimeResponseKey {
        operation_id: String,
        contract_path: String,
        response_key: String,
    },
    InvalidRuntimeBinding {
        operation_id: String,
        field: &'static str,
        reason: String,
    },
    DuplicateRuntimeSource {
        path: String,
    },
    DuplicateRuntimeTest {
        path: String,
    },
    MissingRuntimeSource {
        operation_id: String,
        path: String,
    },
    MissingRuntimeTest {
        operation_id: String,
        path: String,
    },
    MissingRuntimeSymbol {
        operation_id: String,
        path: String,
        symbol: String,
    },
    MissingRuntimeEvidenceSurface {
        operation_id: String,
        path: String,
        evidence_surface: String,
    },
    MissingRuntimeStatusType {
        operation_id: String,
        path: String,
        status_type: String,
    },
    InvalidRuntimeStatusType {
        operation_id: String,
        path: String,
        status_type: String,
        reason: String,
    },
    MissingRuntimeTestCoverage {
        operation_id: String,
        test_path: String,
        symbol: String,
        evidence_surface: String,
    },
    MissingRuntimeResponseStatus {
        operation_id: String,
        path: String,
        status_type: String,
        status: String,
    },
    UndocumentedRuntimeResponseStatus {
        operation_id: String,
        path: String,
        status_type: String,
        status: String,
    },
    MissingRuntimeTestResponseStatus {
        operation_id: String,
        test_path: String,
        status_type: String,
        status: String,
    },
    MissingRuntimeResponseSchema {
        operation_id: String,
        contract_path: String,
        status: String,
    },
    RuntimeResponseSchemaMismatch {
        operation_id: String,
        contract_path: String,
        status: String,
        expected_schema: String,
        actual_schema: String,
    },
    MissingSchemaBinding {
        schema_name: String,
        contract_path: String,
    },
    DuplicateSchemaBinding {
        schema_name: String,
        contract_path: String,
    },
    StaleSchemaBinding {
        schema_name: String,
        contract_path: String,
    },
    InvalidSchemaBinding {
        schema_name: String,
        field: &'static str,
        reason: String,
    },
    MissingSchemaRuntimeSource {
        schema_name: String,
        path: String,
    },
    MissingRuntimeStruct {
        schema_name: String,
        path: String,
        rust_struct: String,
    },
    InvalidRuntimeStruct {
        schema_name: String,
        path: String,
        rust_struct: String,
        reason: String,
    },
    SchemaFieldMismatch {
        schema_name: String,
        contract_path: String,
        missing_properties: Vec<String>,
        extra_properties: Vec<String>,
    },
    SchemaRequiredMismatch {
        schema_name: String,
        contract_path: String,
        missing_required: Vec<String>,
        extra_required: Vec<String>,
    },
    SchemaTypeMismatch {
        schema_name: String,
        contract_path: String,
        mismatches: Vec<String>,
    },
}

pub fn validate_openapi_documents<I>(
    documents: I,
) -> Result<OpenApiSourceReport, OpenApiSourceError>
where
    I: IntoIterator<Item = OpenApiDocument>,
{
    let documents = document_map(documents)?;
    let mut operations_checked = 0usize;
    let mut data_class_annotations_checked = 0usize;
    let mut operation_ids = BTreeMap::<String, String>::new();
    for (path, contents) in &documents {
        let report = validate_document(path, contents)?;
        for operation in collect_document_operations(path, contents)? {
            if let Some(first_path) = operation_ids.insert(
                operation.operation_id.clone(),
                operation.contract_path.clone(),
            ) {
                return Err(OpenApiSourceError::DuplicateOperationId {
                    operation_id: operation.operation_id,
                    first_path,
                    second_path: operation.contract_path,
                });
            }
        }
        operations_checked += report.operations_checked;
        data_class_annotations_checked += report.data_class_annotations_checked;
    }
    Ok(OpenApiSourceReport {
        documents_checked: documents.len(),
        operations_checked,
        data_class_annotations_checked,
    })
}

pub fn validate_openapi_contract_mirror<I, L>(
    openapi_paths: I,
    spec_contents: &str,
    mirror_locations: L,
) -> Result<OpenApiContractMirrorReport, OpenApiSourceError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
    L: IntoIterator<Item = OpenApiContractMirrorLocation>,
{
    let openapi_paths = openapi_paths
        .into_iter()
        .map(|path| {
            let path = path.as_ref().to_string();
            validate_document_path(&path)?;
            Ok(path)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if openapi_paths.is_empty() {
        return Err(OpenApiSourceError::NoDocuments);
    }

    let spec_paths = exact_openapi_paths_in_text(spec_contents);
    for path in &openapi_paths {
        if !spec_paths.contains(path) {
            return Err(OpenApiSourceError::MissingSpecMirror { path: path.clone() });
        }
    }
    if let Some(path) = spec_paths
        .iter()
        .find(|path| !openapi_paths.contains(*path))
    {
        return Err(OpenApiSourceError::StaleSpecMirror { path: path.clone() });
    }

    let locations = mirror_locations.into_iter().collect::<Vec<_>>();
    let mut exact_mirror_paths = BTreeSet::new();
    let mut mirror_references_checked = 0usize;
    for location in &locations {
        for reference in openapi_location_references(&location.location) {
            if is_openapi_glob_reference(&reference) {
                continue;
            }
            mirror_references_checked += 1;
            if !openapi_paths.contains(&reference) {
                return Err(OpenApiSourceError::StaleMachineMirror {
                    contract_id: location.contract_id.clone(),
                    path: reference,
                });
            }
            exact_mirror_paths.insert(reference);
        }
    }
    for path in &openapi_paths {
        if !exact_mirror_paths.contains(path) {
            return Err(OpenApiSourceError::MissingMachineMirror { path: path.clone() });
        }
    }

    Ok(OpenApiContractMirrorReport {
        contracts_checked: openapi_paths.len(),
        spec_references_checked: spec_paths.len(),
        mirror_references_checked,
    })
}

pub fn validate_openapi_runtime_parity<D, B, S, T>(
    documents: D,
    bindings: B,
    runtime_sources: S,
    runtime_tests: T,
) -> Result<OpenApiRuntimeParityReport, OpenApiSourceError>
where
    D: IntoIterator<Item = OpenApiDocument>,
    B: IntoIterator<Item = OpenApiRuntimeBinding>,
    S: IntoIterator<Item = OpenApiRuntimeSource>,
    T: IntoIterator<Item = OpenApiRuntimeSource>,
{
    let documents = document_map(documents)?;
    let mut operations = BTreeMap::new();
    for (path, contents) in &documents {
        validate_document(path, contents)?;
        for operation in collect_document_operations(path, contents)? {
            if let Some(first) =
                operations.insert(operation.operation_id.clone(), operation.clone())
            {
                return Err(OpenApiSourceError::DuplicateOperationId {
                    operation_id: operation.operation_id,
                    first_path: first.contract_path,
                    second_path: operation.contract_path,
                });
            }
        }
    }

    let bindings = runtime_binding_map(bindings)?;
    let sources = runtime_source_map(runtime_sources, true)?;
    let tests = runtime_source_map(runtime_tests, false)?;

    for operation in operations.values() {
        if !bindings.contains_key(&operation.operation_id) {
            return Err(OpenApiSourceError::MissingRuntimeBinding {
                operation_id: operation.operation_id.clone(),
                contract_path: operation.contract_path.clone(),
            });
        }
    }

    let mut response_statuses_checked = 0usize;
    let mut response_schemas_checked = 0usize;
    for binding in bindings.values() {
        let Some(operation) = operations.get(&binding.operation_id) else {
            return Err(OpenApiSourceError::StaleRuntimeBinding {
                operation_id: binding.operation_id.clone(),
                contract_path: binding.contract_path.clone(),
            });
        };
        if operation.contract_path != binding.contract_path {
            return Err(OpenApiSourceError::StaleRuntimeBinding {
                operation_id: binding.operation_id.clone(),
                contract_path: binding.contract_path.clone(),
            });
        }
        let report = validate_runtime_binding_source(binding, operation, &sources, &tests)?;
        response_statuses_checked += report.response_statuses_checked;
        response_schemas_checked += report.response_schemas_checked;
    }

    Ok(OpenApiRuntimeParityReport {
        operations_checked: operations.len(),
        bindings_checked: bindings.len(),
        sources_checked: sources.len(),
        tests_checked: tests.len(),
        response_statuses_checked,
        response_schemas_checked,
    })
}

pub fn validate_openapi_schema_parity<D, B, S>(
    documents: D,
    bindings: B,
    runtime_sources: S,
) -> Result<OpenApiSchemaParityReport, OpenApiSourceError>
where
    D: IntoIterator<Item = OpenApiDocument>,
    B: IntoIterator<Item = OpenApiSchemaBinding>,
    S: IntoIterator<Item = OpenApiRuntimeSource>,
{
    let documents = document_map(documents)?;
    let mut schemas = BTreeMap::new();
    for (path, contents) in &documents {
        validate_document(path, contents)?;
        for schema in collect_component_schemas(path, contents)? {
            schemas.insert(
                (schema.contract_path.clone(), schema.schema_name.clone()),
                schema,
            );
        }
    }

    let bindings = schema_binding_map(bindings)?;
    let sources = runtime_source_map(runtime_sources, true)?;
    for schema in schemas.values() {
        let key = (schema.contract_path.clone(), schema.schema_name.clone());
        if !bindings.contains_key(&key) {
            return Err(OpenApiSourceError::MissingSchemaBinding {
                schema_name: schema.schema_name.clone(),
                contract_path: schema.contract_path.clone(),
            });
        }
    }

    let mut fields_checked = 0usize;
    let mut types_checked = 0usize;
    for binding in bindings.values() {
        let key = (binding.contract_path.clone(), binding.schema_name.clone());
        let Some(schema) = schemas.get(&key) else {
            return Err(OpenApiSourceError::StaleSchemaBinding {
                schema_name: binding.schema_name.clone(),
                contract_path: binding.contract_path.clone(),
            });
        };
        let source = sources.get(&binding.source_path).ok_or_else(|| {
            OpenApiSourceError::MissingSchemaRuntimeSource {
                schema_name: binding.schema_name.clone(),
                path: binding.source_path.clone(),
            }
        })?;
        let runtime = parse_rust_struct_fields(source, &binding.rust_struct).map_err(|reason| {
            OpenApiSourceError::InvalidRuntimeStruct {
                schema_name: binding.schema_name.clone(),
                path: binding.source_path.clone(),
                rust_struct: binding.rust_struct.clone(),
                reason,
            }
        })?;
        let runtime = runtime.ok_or_else(|| OpenApiSourceError::MissingRuntimeStruct {
            schema_name: binding.schema_name.clone(),
            path: binding.source_path.clone(),
            rust_struct: binding.rust_struct.clone(),
        })?;
        validate_schema_fields_match(schema, &runtime)?;
        fields_checked += schema.properties.len();
        validate_schema_types_match(schema, &runtime)?;
        types_checked += schema.properties.len();
    }

    Ok(OpenApiSchemaParityReport {
        schemas_checked: schemas.len(),
        bindings_checked: bindings.len(),
        sources_checked: sources.len(),
        fields_checked,
        types_checked,
    })
}

fn document_map<I>(documents: I) -> Result<BTreeMap<String, String>, OpenApiSourceError>
where
    I: IntoIterator<Item = OpenApiDocument>,
{
    let mut map = BTreeMap::new();
    for document in documents {
        validate_document_path(&document.path)?;
        if document.contents.trim().is_empty() {
            return Err(OpenApiSourceError::EmptyDocument {
                path: document.path,
            });
        }
        if map
            .insert(document.path.clone(), document.contents)
            .is_some()
        {
            return Err(OpenApiSourceError::DuplicateDocument {
                path: document.path,
            });
        }
    }
    if map.is_empty() {
        return Err(OpenApiSourceError::NoDocuments);
    }
    Ok(map)
}

fn validate_document_path(path: &str) -> Result<(), OpenApiSourceError> {
    if path.trim().is_empty() {
        return invalid_path(path, "path must be non-empty");
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return invalid_path(path, "path must be a relative slash path");
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return invalid_path(
            path,
            "path must not contain empty, dot, or parent components",
        );
    }
    if !path.starts_with(OPENAPI_PREFIX) {
        return invalid_path(path, "OpenAPI documents must live under contracts/openapi/");
    }
    if !(path.ends_with(".yaml") || path.ends_with(".yml")) {
        return invalid_path(path, "OpenAPI documents must be .yaml or .yml files");
    }
    if path.ends_with(".meta.yaml") || path.ends_with(".meta.yml") {
        return invalid_path(path, "OpenAPI metadata files are not source documents");
    }
    if version_suffix_major(path).is_none() {
        return invalid_path(path, "OpenAPI document filenames must end in -v<major>");
    }
    Ok(())
}

fn runtime_binding_map<I>(
    bindings: I,
) -> Result<BTreeMap<String, OpenApiRuntimeBinding>, OpenApiSourceError>
where
    I: IntoIterator<Item = OpenApiRuntimeBinding>,
{
    let mut map = BTreeMap::new();
    for binding in bindings {
        validate_runtime_binding_shape(&binding)?;
        let operation_id = binding.operation_id.clone();
        if map.insert(operation_id.clone(), binding).is_some() {
            return Err(OpenApiSourceError::DuplicateRuntimeBinding { operation_id });
        }
    }
    Ok(map)
}

fn schema_binding_map<I>(
    bindings: I,
) -> Result<BTreeMap<(String, String), OpenApiSchemaBinding>, OpenApiSourceError>
where
    I: IntoIterator<Item = OpenApiSchemaBinding>,
{
    let mut map = BTreeMap::new();
    for binding in bindings {
        validate_schema_binding_shape(&binding)?;
        let key = (binding.contract_path.clone(), binding.schema_name.clone());
        if map.insert(key.clone(), binding).is_some() {
            return Err(OpenApiSourceError::DuplicateSchemaBinding {
                contract_path: key.0,
                schema_name: key.1,
            });
        }
    }
    Ok(map)
}

fn runtime_source_map<I>(
    sources: I,
    source_file: bool,
) -> Result<BTreeMap<String, String>, OpenApiSourceError>
where
    I: IntoIterator<Item = OpenApiRuntimeSource>,
{
    let mut map = BTreeMap::new();
    for source in sources {
        validate_relative_runtime_path(&source.path, "runtime artifact path", "<runtime-source>")?;
        if map.insert(source.path.clone(), source.contents).is_some() {
            if source_file {
                return Err(OpenApiSourceError::DuplicateRuntimeSource { path: source.path });
            }
            return Err(OpenApiSourceError::DuplicateRuntimeTest { path: source.path });
        }
    }
    Ok(map)
}

/// Whether a binding `source_path`/`test_path` lives in the crate directory of
/// `runtime_crate`. Two layouts are accepted:
/// - the legacy flat layout `crates/<crate>/...`;
/// - the canonical capability-face layout `<capability>/<face>/<crate>/...`
///   (ADR-0562), where the package name is `<capability>-<crate>` — e.g.
///   `intelligence/core/api` hosts the `intelligence-api` package.
///
/// The canonical check is shape-only (path component arithmetic); it does not
/// read the crate manifest, keeping this kernel filesystem-free.
fn path_lives_in_crate(path: &str, runtime_crate: &str) -> bool {
    if let Some(rest) = path.strip_prefix("crates/") {
        return rest.starts_with(&format!("{runtime_crate}/"));
    }
    // Canonical: <capability>/<face>/<crate>/... where `<capability>-<crate>`
    // is the package name. The crate dir is the third path component.
    let mut parts = path.split('/');
    let (Some(capability), Some(_face), Some(crate_dir)) =
        (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    format!("{capability}-{crate_dir}") == runtime_crate
}

fn validate_schema_binding_shape(binding: &OpenApiSchemaBinding) -> Result<(), OpenApiSourceError> {
    require_schema_non_empty(&binding.schema_name, "schema_name", &binding.schema_name)?;
    validate_document_path(&binding.contract_path)?;
    require_schema_non_empty(
        &binding.runtime_crate,
        "runtime_crate",
        &binding.schema_name,
    )?;
    validate_relative_schema_path(&binding.source_path, "source_path", &binding.schema_name)?;
    require_schema_non_empty(&binding.rust_struct, "rust_struct", &binding.schema_name)?;

    if !path_lives_in_crate(&binding.source_path, &binding.runtime_crate) {
        return Err(OpenApiSourceError::InvalidSchemaBinding {
            schema_name: binding.schema_name.clone(),
            field: "source_path",
            reason: format!(
                "source path must live in the crate dir of `{}` (legacy `crates/<crate>/` or canonical `<capability>/<face>/<crate>/` layout)",
                binding.runtime_crate
            ),
        });
    }
    Ok(())
}

fn require_schema_non_empty(
    value: &str,
    field: &'static str,
    schema_name: &str,
) -> Result<(), OpenApiSourceError> {
    if value.trim().is_empty() {
        return Err(OpenApiSourceError::InvalidSchemaBinding {
            schema_name: schema_name.into(),
            field,
            reason: "field must be non-empty".into(),
        });
    }
    Ok(())
}

fn validate_relative_schema_path(
    path: &str,
    field: &'static str,
    schema_name: &str,
) -> Result<(), OpenApiSourceError> {
    if path.trim().is_empty() {
        return Err(OpenApiSourceError::InvalidSchemaBinding {
            schema_name: schema_name.into(),
            field,
            reason: "path must be non-empty".into(),
        });
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return Err(OpenApiSourceError::InvalidSchemaBinding {
            schema_name: schema_name.into(),
            field,
            reason: "path must be a relative slash path".into(),
        });
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(OpenApiSourceError::InvalidSchemaBinding {
            schema_name: schema_name.into(),
            field,
            reason: "path must not contain empty, dot, or parent components".into(),
        });
    }
    Ok(())
}

fn validate_runtime_binding_shape(
    binding: &OpenApiRuntimeBinding,
) -> Result<(), OpenApiSourceError> {
    require_non_empty(&binding.operation_id, "operation_id", &binding.operation_id)?;
    validate_document_path(&binding.contract_path)?;
    require_non_empty(
        &binding.runtime_crate,
        "runtime_crate",
        &binding.operation_id,
    )?;
    validate_relative_runtime_path(&binding.source_path, "source_path", &binding.operation_id)?;
    validate_relative_runtime_path(&binding.test_path, "test_path", &binding.operation_id)?;
    require_non_empty(&binding.symbol, "symbol", &binding.operation_id)?;
    require_non_empty(&binding.status_type, "status_type", &binding.operation_id)?;
    require_non_empty(
        &binding.evidence_surface,
        "evidence_surface",
        &binding.operation_id,
    )?;
    if binding.response_schemas.is_empty() {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: binding.operation_id.clone(),
            field: "response_schemas",
            reason: "field must contain status=schema pairs".into(),
        });
    }
    for (status, schema_name) in &binding.response_schemas {
        if !valid_numeric_response_status(status) {
            return Err(OpenApiSourceError::InvalidRuntimeBinding {
                operation_id: binding.operation_id.clone(),
                field: "response_schemas",
                reason: format!("response status {status} must be an explicit numeric status"),
            });
        }
        require_non_empty(schema_name, "response_schemas", &binding.operation_id)?;
    }

    if !path_lives_in_crate(&binding.source_path, &binding.runtime_crate) {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: binding.operation_id.clone(),
            field: "source_path",
            reason: format!(
                "source path must live in the crate dir of `{}` (legacy `crates/<crate>/` or canonical `<capability>/<face>/<crate>/` layout)",
                binding.runtime_crate
            ),
        });
    }
    if !path_lives_in_crate(&binding.test_path, &binding.runtime_crate) {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: binding.operation_id.clone(),
            field: "test_path",
            reason: format!(
                "test path must live in the crate dir of `{}` (legacy `crates/<crate>/` or canonical `<capability>/<face>/<crate>/` layout)",
                binding.runtime_crate
            ),
        });
    }
    Ok(())
}

fn require_non_empty(
    value: &str,
    field: &'static str,
    operation_id: &str,
) -> Result<(), OpenApiSourceError> {
    if value.trim().is_empty() {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: operation_id.into(),
            field,
            reason: "field must be non-empty".into(),
        });
    }
    Ok(())
}

fn validate_relative_runtime_path(
    path: &str,
    field: &'static str,
    operation_id: &str,
) -> Result<(), OpenApiSourceError> {
    if path.trim().is_empty() {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: operation_id.into(),
            field,
            reason: "path must be non-empty".into(),
        });
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: operation_id.into(),
            field,
            reason: "path must be a relative slash path".into(),
        });
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(OpenApiSourceError::InvalidRuntimeBinding {
            operation_id: operation_id.into(),
            field,
            reason: "path must not contain empty, dot, or parent components".into(),
        });
    }
    Ok(())
}

fn rust_code_contains_identifier(source: &str, identifier: &str) -> bool {
    if !valid_rust_identifier(identifier) {
        return false;
    }
    let code = rust_code_without_comments_and_literals(source);
    find_rust_identifier(&code, identifier, 0).is_some()
}

fn rust_code_contains_public_function(source: &str, function_name: &str) -> bool {
    if !valid_rust_identifier(function_name) {
        return false;
    }
    let code = rust_code_without_comments_and_literals(source);
    let mut search_start = 0usize;
    while let Some((public_index, mut cursor)) = find_top_level_bare_pub_item(&code, search_start) {
        cursor = skip_rust_function_qualifiers(&code, cursor);
        if rust_identifier_at(&code, cursor, "fn") {
            cursor = skip_whitespace(&code, cursor + "fn".len());
            if rust_identifier_at(&code, cursor, function_name) {
                let after_name = skip_whitespace(&code, cursor + function_name.len());
                if code[after_name..].starts_with('(') {
                    return true;
                }
            }
        }
        search_start = public_index + "pub".len();
    }
    false
}

fn rust_code_contains_public_string_const(source: &str, expected: &str) -> bool {
    let code = rust_code_without_comments_and_literals(source);
    let mut search_start = 0usize;
    while let Some((public_index, mut cursor)) = find_top_level_bare_pub_item(&code, search_start) {
        if rust_identifier_at(&code, cursor, "const") {
            cursor = skip_whitespace(&code, cursor + "const".len());
            if let Some(name_end) = rust_identifier_end_at(&code, cursor) {
                let declaration_end = code[name_end..]
                    .find(';')
                    .map_or(code.len(), |relative| name_end + relative);
                if let Some(relative_equals) = code[name_end..declaration_end].find('=') {
                    let equals_index = name_end + relative_equals;
                    let value_index = skip_whitespace(source, equals_index + 1);
                    if string_literal_equals_at(source, value_index, expected) {
                        return true;
                    }
                }
            }
        }
        search_start = public_index + "pub".len();
    }
    false
}

fn find_public_rust_enum(code: &str, enum_name: &str) -> Option<usize> {
    if !valid_rust_identifier(enum_name) {
        return None;
    }
    let mut search_start = 0usize;
    while let Some((public_index, mut cursor)) = find_top_level_bare_pub_item(code, search_start) {
        if rust_identifier_at(code, cursor, "enum") {
            let enum_index = cursor;
            cursor = skip_whitespace(code, cursor + "enum".len());
            if rust_identifier_at(code, cursor, enum_name) {
                let after_name = skip_whitespace(code, cursor + enum_name.len());
                if code[after_name..].starts_with('{') {
                    return Some(enum_index);
                }
            }
        }
        search_start = public_index + "pub".len();
    }
    None
}

fn find_public_rust_struct(code: &str, struct_name: &str) -> Option<usize> {
    if !valid_rust_identifier(struct_name) {
        return None;
    }
    let mut search_start = 0usize;
    while let Some((public_index, mut cursor)) = find_top_level_bare_pub_item(code, search_start) {
        if rust_identifier_at(code, cursor, "struct") {
            let struct_index = cursor;
            cursor = skip_whitespace(code, cursor + "struct".len());
            if rust_identifier_at(code, cursor, struct_name) {
                let after_name = skip_whitespace(code, cursor + struct_name.len());
                if code[after_name..].starts_with('{') {
                    return Some(struct_index);
                }
            }
        }
        search_start = public_index + "pub".len();
    }
    None
}

fn find_top_level_bare_pub_item(code: &str, start: usize) -> Option<(usize, usize)> {
    let mut search_start = start;
    while let Some(public_index) = find_rust_identifier(code, "pub", search_start) {
        let item_start = skip_whitespace(code, public_index + "pub".len());
        if brace_depth_before(code, public_index) == 0 && !code[item_start..].starts_with('(') {
            return Some((public_index, item_start));
        }
        search_start = public_index + "pub".len();
    }
    None
}

fn skip_rust_function_qualifiers(code: &str, mut cursor: usize) -> usize {
    loop {
        let next = ["async", "unsafe", "extern", "const"]
            .iter()
            .find(|qualifier| rust_identifier_at(code, cursor, qualifier));
        let Some(qualifier) = next else {
            return cursor;
        };
        cursor = skip_whitespace(code, cursor + qualifier.len());
    }
}

fn rust_identifier_at(code: &str, index: usize, identifier: &str) -> bool {
    code.get(index..)
        .is_some_and(|remaining| remaining.starts_with(identifier))
        && (index == 0
            || code[..index]
                .chars()
                .next_back()
                .is_none_or(|character| !is_rust_identifier_character(character)))
        && (index + identifier.len() == code.len()
            || code[index + identifier.len()..]
                .chars()
                .next()
                .is_none_or(|character| !is_rust_identifier_character(character)))
}

fn rust_identifier_end_at(code: &str, index: usize) -> Option<usize> {
    let mut chars = code[index..].char_indices();
    let (_, first) = chars.next()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }
    let mut end = index + first.len_utf8();
    for (relative_index, character) in chars {
        if is_rust_identifier_character(character) {
            end = index + relative_index + character.len_utf8();
        } else {
            break;
        }
    }
    Some(end)
}

fn brace_depth_before(code: &str, index: usize) -> usize {
    let mut depth = 0usize;
    for character in code[..index].chars() {
        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn rust_string_literal_equals(source: &str, expected: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if starts_with_bytes(bytes, index, b"//") {
            index = consume_line_comment(bytes, index + 2);
            continue;
        }
        if starts_with_bytes(bytes, index, b"/*") {
            index = consume_block_comment(bytes, index + 2);
            continue;
        }
        if let Some((next_index, value)) = raw_string_literal_value(source, index) {
            if value == expected {
                return true;
            }
            index = next_index;
            continue;
        }
        if bytes[index] == b'"' {
            let (next_index, value) = quoted_string_literal_value(bytes, index + 1);
            if value == expected {
                return true;
            }
            index = next_index;
            continue;
        }
        index += 1;
    }
    false
}

fn string_literal_equals_at(source: &str, index: usize, expected: &str) -> bool {
    if let Some((_, value)) = raw_string_literal_value(source, index) {
        return value == expected;
    }
    if source.as_bytes().get(index).copied() == Some(b'"') {
        let (_, value) = quoted_string_literal_value(source.as_bytes(), index + 1);
        return value == expected;
    }
    false
}

fn valid_rust_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn rust_code_without_comments_and_literals(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0usize;
    while index < bytes.len() {
        if starts_with_bytes(bytes, index, b"//") {
            let next_index = consume_line_comment(bytes, index + 2);
            mask_non_code_bytes(&mut output, index, next_index);
            index = next_index;
            continue;
        }
        if starts_with_bytes(bytes, index, b"/*") {
            let next_index = consume_block_comment(bytes, index + 2);
            mask_non_code_bytes(&mut output, index, next_index);
            index = next_index;
            continue;
        }
        if let Some((next_index, _)) = raw_string_literal_value(source, index) {
            mask_non_code_bytes(&mut output, index, next_index);
            index = next_index;
            continue;
        }
        if bytes[index] == b'"' {
            let (next_index, _) = quoted_string_literal_value(bytes, index + 1);
            mask_non_code_bytes(&mut output, index, next_index);
            index = next_index;
            continue;
        }
        if starts_simple_char_literal(bytes, index) {
            let next_index = consume_quoted_literal(bytes, index + 1, b'\'');
            mask_non_code_bytes(&mut output, index, next_index);
            index = next_index;
            continue;
        }
        index += 1;
    }
    // ADR-0083 Tier 1: `mask_non_code_bytes` overwrites individual bytes
    // inside comment/literal byte ranges, which can split multi-byte UTF-8
    // sequences. Use `from_utf8_lossy` to keep the helper infallible
    // without an `.expect()` and without panicking on inputs that contain
    // non-ASCII characters inside literals or comments.
    String::from_utf8_lossy(&output).into_owned()
}

fn mask_non_code_bytes(output: &mut [u8], start: usize, end: usize) {
    for byte in &mut output[start..end] {
        if *byte != b'\n' && *byte != b'\r' {
            *byte = b' ';
        }
    }
}

fn starts_with_bytes(bytes: &[u8], index: usize, expected: &[u8]) -> bool {
    bytes
        .get(index..)
        .is_some_and(|remaining| remaining.starts_with(expected))
}

fn consume_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    index
}

fn consume_block_comment(bytes: &[u8], mut index: usize) -> usize {
    let mut depth = 1usize;
    while index < bytes.len() {
        if starts_with_bytes(bytes, index, b"/*") {
            depth += 1;
            index += 2;
            continue;
        }
        if starts_with_bytes(bytes, index, b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
            continue;
        }
        index += 1;
    }
    bytes.len()
}

fn consume_quoted_literal(bytes: &[u8], mut index: usize, delimiter: u8) -> usize {
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == delimiter => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn starts_simple_char_literal(bytes: &[u8], index: usize) -> bool {
    if bytes.get(index).copied() != Some(b'\'') {
        return false;
    }
    bytes
        .get(index + 1)
        .copied()
        .is_some_and(|next| next == b'\\' || bytes.get(index + 2).copied() == Some(b'\''))
}

fn quoted_string_literal_value(bytes: &[u8], mut index: usize) -> (usize, String) {
    let mut value = String::new();
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if let Some(escaped) = bytes.get(index + 1).copied() {
                    value.push(match escaped {
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'\\' => '\\',
                        b'"' => '"',
                        other => other as char,
                    });
                }
                index = (index + 2).min(bytes.len());
            }
            b'"' => return (index + 1, value),
            byte => {
                value.push(byte as char);
                index += 1;
            }
        }
    }
    (bytes.len(), value)
}

fn raw_string_literal_value(source: &str, index: usize) -> Option<(usize, &str)> {
    let bytes = source.as_bytes();
    if bytes.get(index).copied()? != b'r' {
        return None;
    }
    if index > 0
        && source[..index]
            .chars()
            .next_back()
            .is_some_and(is_rust_identifier_character)
    {
        return None;
    }
    let mut delimiter_index = index + 1;
    while bytes.get(delimiter_index).copied() == Some(b'#') {
        delimiter_index += 1;
    }
    if bytes.get(delimiter_index).copied() != Some(b'"') {
        return None;
    }
    let hashes = delimiter_index - index - 1;
    let content_start = delimiter_index + 1;
    let mut search_index = content_start;
    while search_index < bytes.len() {
        if bytes[search_index] == b'"'
            && bytes
                .get(search_index + 1..search_index + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            let end_index = search_index + 1 + hashes;
            return Some((end_index, &source[content_start..search_index]));
        }
        search_index += 1;
    }
    Some((bytes.len(), &source[content_start..]))
}

fn find_rust_identifier(source: &str, identifier: &str, start: usize) -> Option<usize> {
    let mut search_start = start;
    while let Some(relative_index) = source[search_start..].find(identifier) {
        let index = search_start + relative_index;
        let before_is_boundary = index == 0
            || source[..index]
                .chars()
                .next_back()
                .is_none_or(|character| !is_rust_identifier_character(character));
        let after_index = index + identifier.len();
        let after_is_boundary = after_index == source.len()
            || source[after_index..]
                .chars()
                .next()
                .is_none_or(|character| !is_rust_identifier_character(character));
        if before_is_boundary && after_is_boundary {
            return Some(index);
        }
        search_start = after_index;
    }
    None
}

fn find_rust_keyword_identifier(
    code: &str,
    keyword: &str,
    identifier: &str,
    start: usize,
) -> Option<usize> {
    let mut search_start = start;
    while let Some(keyword_index) = find_rust_identifier(code, keyword, search_start) {
        let mut cursor = keyword_index + keyword.len();
        let mut saw_whitespace = false;
        while let Some(character) = code[cursor..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            saw_whitespace = true;
            cursor += character.len_utf8();
        }
        if saw_whitespace
            && code[cursor..].starts_with(identifier)
            && code[cursor + identifier.len()..]
                .chars()
                .next()
                .is_none_or(|character| !is_rust_identifier_character(character))
        {
            return Some(keyword_index);
        }
        search_start = keyword_index + keyword.len();
    }
    None
}

fn is_rust_identifier_character(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn validate_runtime_binding_source(
    binding: &OpenApiRuntimeBinding,
    operation: &OpenApiOperation,
    sources: &BTreeMap<String, String>,
    tests: &BTreeMap<String, String>,
) -> Result<RuntimeBindingValidationReport, OpenApiSourceError> {
    let source = sources.get(&binding.source_path).ok_or_else(|| {
        OpenApiSourceError::MissingRuntimeSource {
            operation_id: binding.operation_id.clone(),
            path: binding.source_path.clone(),
        }
    })?;
    if !rust_code_contains_public_function(source, &binding.symbol) {
        return Err(OpenApiSourceError::MissingRuntimeSymbol {
            operation_id: binding.operation_id.clone(),
            path: binding.source_path.clone(),
            symbol: binding.symbol.clone(),
        });
    }
    if !rust_code_contains_public_string_const(source, &binding.evidence_surface) {
        return Err(OpenApiSourceError::MissingRuntimeEvidenceSurface {
            operation_id: binding.operation_id.clone(),
            path: binding.source_path.clone(),
            evidence_surface: binding.evidence_surface.clone(),
        });
    }
    if let Some(response_key) = operation.non_explicit_response_keys.iter().next() {
        return Err(OpenApiSourceError::NonExplicitRuntimeResponseKey {
            operation_id: binding.operation_id.clone(),
            contract_path: operation.contract_path.clone(),
            response_key: response_key.clone(),
        });
    }
    for status in &operation.response_statuses {
        let Some(actual_schema) = operation.response_schema_refs.get(status) else {
            return Err(OpenApiSourceError::MissingRuntimeResponseSchema {
                operation_id: binding.operation_id.clone(),
                contract_path: operation.contract_path.clone(),
                status: status.clone(),
            });
        };
        let Some(expected_schema) = binding.response_schemas.get(status) else {
            return Err(OpenApiSourceError::RuntimeResponseSchemaMismatch {
                operation_id: binding.operation_id.clone(),
                contract_path: operation.contract_path.clone(),
                status: status.clone(),
                expected_schema: "<missing runtime binding response schema>".into(),
                actual_schema: actual_schema.clone(),
            });
        };
        if expected_schema != actual_schema {
            return Err(OpenApiSourceError::RuntimeResponseSchemaMismatch {
                operation_id: binding.operation_id.clone(),
                contract_path: operation.contract_path.clone(),
                status: status.clone(),
                expected_schema: expected_schema.clone(),
                actual_schema: actual_schema.clone(),
            });
        }
    }
    for status in binding.response_schemas.keys() {
        if !operation.response_statuses.contains(status) {
            return Err(OpenApiSourceError::RuntimeResponseSchemaMismatch {
                operation_id: binding.operation_id.clone(),
                contract_path: operation.contract_path.clone(),
                status: status.clone(),
                expected_schema: binding
                    .response_schemas
                    .get(status)
                    .cloned()
                    .unwrap_or_default(),
                actual_schema: "<undocumented response status>".into(),
            });
        }
    }
    let source_statuses = match runtime_status_codes(source, &binding.status_type) {
        Ok(statuses) => statuses,
        Err(RuntimeStatusParseError::MissingStatusType) => {
            return Err(OpenApiSourceError::MissingRuntimeStatusType {
                operation_id: binding.operation_id.clone(),
                path: binding.source_path.clone(),
                status_type: binding.status_type.clone(),
            });
        }
        Err(RuntimeStatusParseError::Invalid(reason)) => {
            return Err(OpenApiSourceError::InvalidRuntimeStatusType {
                operation_id: binding.operation_id.clone(),
                path: binding.source_path.clone(),
                status_type: binding.status_type.clone(),
                reason,
            });
        }
    };
    for status in &operation.response_statuses {
        if !source_statuses.contains(status) {
            return Err(OpenApiSourceError::MissingRuntimeResponseStatus {
                operation_id: binding.operation_id.clone(),
                path: binding.source_path.clone(),
                status_type: binding.status_type.clone(),
                status: status.clone(),
            });
        }
    }
    for status in &source_statuses {
        if !operation.response_statuses.contains(status) {
            return Err(OpenApiSourceError::UndocumentedRuntimeResponseStatus {
                operation_id: binding.operation_id.clone(),
                path: binding.source_path.clone(),
                status_type: binding.status_type.clone(),
                status: status.clone(),
            });
        }
    }

    let test =
        tests
            .get(&binding.test_path)
            .ok_or_else(|| OpenApiSourceError::MissingRuntimeTest {
                operation_id: binding.operation_id.clone(),
                path: binding.test_path.clone(),
            })?;
    if !(rust_code_contains_identifier(test, &binding.symbol)
        && rust_string_literal_equals(test, &binding.evidence_surface))
    {
        return Err(OpenApiSourceError::MissingRuntimeTestCoverage {
            operation_id: binding.operation_id.clone(),
            test_path: binding.test_path.clone(),
            symbol: binding.symbol.clone(),
            evidence_surface: binding.evidence_surface.clone(),
        });
    }
    for status in &operation.response_statuses {
        if !runtime_test_covers_status(test, &binding.status_type, status) {
            return Err(OpenApiSourceError::MissingRuntimeTestResponseStatus {
                operation_id: binding.operation_id.clone(),
                test_path: binding.test_path.clone(),
                status_type: binding.status_type.clone(),
                status: status.clone(),
            });
        }
    }
    Ok(RuntimeBindingValidationReport {
        response_statuses_checked: operation.response_statuses.len(),
        response_schemas_checked: operation.response_schema_refs.len(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeBindingValidationReport {
    response_statuses_checked: usize,
    response_schemas_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RuntimeStatusParseError {
    MissingStatusType,
    Invalid(String),
}

fn runtime_status_codes(
    source: &str,
    status_type: &str,
) -> Result<BTreeSet<String>, RuntimeStatusParseError> {
    let variants = rust_enum_variants(source, status_type)?;
    if variants.is_empty() {
        return Err(RuntimeStatusParseError::Invalid(format!(
            "{status_type} must declare at least one fieldless variant"
        )));
    }

    let mut mappings = BTreeMap::<String, String>::new();
    let mut statuses = BTreeMap::<String, String>::new();
    for block in rust_status_code_match_blocks(source, status_type) {
        for (variant, status) in status_match_mappings(block)? {
            if !variants.contains(&variant) {
                return Err(RuntimeStatusParseError::Invalid(format!(
                    "{status_type} maps undeclared variant {variant}"
                )));
            }
            if let Some(previous_variant) = statuses.insert(status.clone(), variant.clone()) {
                return Err(RuntimeStatusParseError::Invalid(format!(
                    "{status_type} maps status {status} from both {previous_variant} and {variant}"
                )));
            }
            if mappings.insert(variant.clone(), status).is_some() {
                return Err(RuntimeStatusParseError::Invalid(format!(
                    "{status_type} maps variant {variant} more than once"
                )));
            }
        }
    }
    if mappings.is_empty() {
        return Err(RuntimeStatusParseError::Invalid(format!(
            "{status_type} must define code(self) with explicit Self::Variant => status mappings"
        )));
    }
    let mapped_variants = mappings.keys().cloned().collect::<BTreeSet<_>>();
    if mapped_variants != variants {
        let missing = variants
            .difference(&mapped_variants)
            .cloned()
            .collect::<Vec<_>>();
        let extra = mapped_variants
            .difference(&variants)
            .cloned()
            .collect::<Vec<_>>();
        return Err(RuntimeStatusParseError::Invalid(format!(
            "{status_type} code mappings do not cover enum variants: missing={missing:?}, extra={extra:?}"
        )));
    }
    Ok(mappings.into_values().collect())
}

fn rust_enum_variants(
    source: &str,
    status_type: &str,
) -> Result<BTreeSet<String>, RuntimeStatusParseError> {
    let code = rust_code_without_comments_and_literals(source);
    let enum_index = find_public_rust_enum(&code, status_type)
        .ok_or(RuntimeStatusParseError::MissingStatusType)?;
    let brace_index = enum_index
        + code[enum_index..].find('{').ok_or_else(|| {
            RuntimeStatusParseError::Invalid(format!(
                "{status_type} enum declaration must have a body"
            ))
        })?;
    let block_end = balanced_brace_end(&code, brace_index).ok_or_else(|| {
        RuntimeStatusParseError::Invalid(format!(
            "{status_type} enum declaration has unbalanced braces"
        ))
    })?;
    let block = &source[brace_index + 1..block_end];
    let mut variants = BTreeSet::new();
    for segment in block.split(',') {
        let Some(variant) = rust_fieldless_variant_name(status_type, segment)? else {
            continue;
        };
        if !variants.insert(variant.clone()) {
            return Err(RuntimeStatusParseError::Invalid(format!(
                "{status_type} declares variant {variant} more than once"
            )));
        }
    }
    Ok(variants)
}

fn rust_fieldless_variant_name(
    status_type: &str,
    segment: &str,
) -> Result<Option<String>, RuntimeStatusParseError> {
    let cleaned = segment
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("#[")
                && !line.starts_with("//")
                && !line.starts_with("/*")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut chars = trimmed.char_indices().peekable();
    let Some((_, first)) = chars.peek().copied() else {
        return Ok(None);
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(RuntimeStatusParseError::Invalid(format!(
            "{status_type} status enum contains an invalid variant declaration: {trimmed}"
        )));
    }
    let mut name_end = 0usize;
    for (index, character) in chars {
        if character == '_' || character.is_ascii_alphanumeric() {
            name_end = index + character.len_utf8();
        } else {
            break;
        }
    }
    let name = &trimmed[..name_end];
    if name.is_empty() {
        return Ok(None);
    }
    let remainder = trimmed[name_end..].trim();
    if !remainder.is_empty() {
        return Err(RuntimeStatusParseError::Invalid(format!(
            "{status_type} status enum variant {name} must be fieldless"
        )));
    }
    Ok(Some(name.to_string()))
}

fn rust_status_code_match_blocks<'a>(source: &'a str, status_type: &str) -> Vec<&'a str> {
    let mut blocks = Vec::new();
    for impl_block in rust_impl_blocks(source, status_type) {
        let impl_code = rust_code_without_comments_and_literals(impl_block);
        let mut search_start = 0usize;
        while let Some(fn_index) =
            find_rust_keyword_identifier(&impl_code, "fn", "code", search_start)
        {
            let Some(relative_fn_brace_index) = impl_code[fn_index..].find('{') else {
                break;
            };
            let fn_brace_index = fn_index + relative_fn_brace_index;
            let Some(fn_end) = balanced_brace_end(&impl_code, fn_brace_index) else {
                break;
            };
            let fn_block = &impl_block[fn_brace_index..=fn_end];
            if let Some(match_block) = rust_match_self_block(fn_block) {
                blocks.push(match_block);
            }
            search_start = fn_end + 1;
        }
    }
    blocks
}

fn rust_match_self_block(source: &str) -> Option<&str> {
    let code = rust_code_without_comments_and_literals(source);
    let match_index = find_rust_keyword_identifier(&code, "match", "self", 0)?;
    let brace_index = match_index + code[match_index..].find('{')?;
    let block_end = balanced_brace_end(&code, brace_index)?;
    Some(&source[brace_index + 1..block_end])
}

fn rust_impl_blocks<'a>(source: &'a str, status_type: &str) -> Vec<&'a str> {
    let code = rust_code_without_comments_and_literals(source);
    let mut blocks = Vec::new();
    let mut search_start = 0usize;
    while let Some(impl_index) =
        find_rust_keyword_identifier(&code, "impl", status_type, search_start)
    {
        let Some(relative_brace_index) = code[impl_index..].find('{') else {
            break;
        };
        let brace_index = impl_index + relative_brace_index;
        let Some(block_end) = balanced_brace_end(&code, brace_index) else {
            break;
        };
        blocks.push(&source[brace_index..=block_end]);
        search_start = block_end + 1;
    }
    blocks
}

fn balanced_brace_end(source: &str, open_brace_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (relative_index, character) in source[open_brace_index..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open_brace_index + relative_index);
                }
            }
            _ => {}
        }
    }
    None
}

fn status_match_mappings(block: &str) -> Result<Vec<(String, String)>, RuntimeStatusParseError> {
    let mut mappings = Vec::new();
    for arm in block.split(',') {
        let Some((left, right)) = arm.split_once("=>") else {
            continue;
        };
        let Some(status) = explicit_status_literal(right) else {
            return Err(RuntimeStatusParseError::Invalid(
                "status code mappings must return explicit three-digit numeric literals".into(),
            ));
        };
        let mut mapped_any_variant = false;
        for variant_expression in left.split('|') {
            let Some(variant) = self_variant_name(variant_expression) else {
                return Err(RuntimeStatusParseError::Invalid(
                    "status code mappings must use explicit Self::Variant arms".into(),
                ));
            };
            mappings.push((variant, status.clone()));
            mapped_any_variant = true;
        }
        if !mapped_any_variant {
            return Err(RuntimeStatusParseError::Invalid(
                "status code mappings must include at least one Self::Variant arm".into(),
            ));
        }
    }
    Ok(mappings)
}

fn explicit_status_literal(expression: &str) -> Option<String> {
    let trimmed = expression.trim();
    let status = trimmed
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    if !valid_numeric_response_status(&status) {
        return None;
    }
    if !trimmed[status.len()..].trim().is_empty() {
        return None;
    }
    Some(status)
}

fn self_variant_name(expression: &str) -> Option<String> {
    let trimmed = expression.trim();
    let suffix = trimmed.strip_prefix("Self::")?;
    let mut name_end = 0usize;
    for (index, character) in suffix.char_indices() {
        if character == '_' || character.is_ascii_alphanumeric() {
            name_end = index + character.len_utf8();
        } else {
            break;
        }
    }
    let name = &suffix[..name_end];
    if name.is_empty() {
        return None;
    }
    if !suffix[name_end..].trim().is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn runtime_test_covers_status(test: &str, status_type: &str, status: &str) -> bool {
    test.split(';')
        .any(|statement| runtime_test_asserts_status_code(statement, status_type, status))
}

fn runtime_test_asserts_status_code(statement: &str, status_type: &str, status: &str) -> bool {
    let code = rust_code_without_comments_and_literals(statement);
    statement_invokes_macro(&code, "assert_eq")
        && statement_uses_status_type_variant(&code, status_type)
        && statement_calls_code_method(&code)
        && statement_contains_response_status(&code, status)
}

fn statement_invokes_macro(code: &str, macro_name: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(index) = find_rust_identifier(code, macro_name, search_start) {
        let cursor = skip_whitespace(code, index + macro_name.len());
        if code[cursor..].starts_with('!') {
            return true;
        }
        search_start = index + macro_name.len();
    }
    false
}

fn statement_uses_status_type_variant(code: &str, status_type: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(index) = find_rust_identifier(code, status_type, search_start) {
        let cursor = skip_whitespace(code, index + status_type.len());
        if code[cursor..].starts_with("::") {
            return true;
        }
        search_start = index + status_type.len();
    }
    false
}

fn statement_calls_code_method(code: &str) -> bool {
    code.chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .contains(".code()")
}

fn skip_whitespace(source: &str, mut index: usize) -> usize {
    while let Some(character) = source[index..].chars().next() {
        if !character.is_whitespace() {
            break;
        }
        index += character.len_utf8();
    }
    index
}

fn statement_contains_response_status(statement: &str, status: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(relative_index) = statement[search_start..].find(status) {
        let index = search_start + relative_index;
        let before_is_boundary = index == 0
            || statement[..index]
                .chars()
                .next_back()
                .is_none_or(|character| {
                    !is_rust_identifier_character(character) && character != '.'
                });
        let after_index = index + status.len();
        let after_is_boundary = after_index == statement.len()
            || statement[after_index..]
                .chars()
                .next()
                .is_none_or(|character| {
                    !is_rust_identifier_character(character) && character != '.'
                });
        if before_is_boundary && after_is_boundary {
            return true;
        }
        search_start = after_index;
    }
    false
}

fn invalid_path<T>(path: &str, reason: &str) -> Result<T, OpenApiSourceError> {
    Err(OpenApiSourceError::InvalidPath {
        path: path.into(),
        reason: reason.into(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DocumentValidationReport {
    operations_checked: usize,
    data_class_annotations_checked: usize,
}

fn validate_document(
    path: &str,
    contents: &str,
) -> Result<DocumentValidationReport, OpenApiSourceError> {
    let lines = logical_lines(contents);
    let openapi = top_level_value(&lines, "openapi").ok_or_else(|| {
        OpenApiSourceError::MissingTopLevelField {
            path: path.into(),
            field: "openapi",
        }
    })?;
    if !openapi.starts_with(SUPPORTED_OPENAPI_MAJOR_MINOR) {
        return Err(OpenApiSourceError::UnsupportedOpenApiVersion {
            path: path.into(),
            version: openapi,
        });
    }

    let info_range = top_level_block(&lines, "info").ok_or_else(|| {
        OpenApiSourceError::MissingTopLevelField {
            path: path.into(),
            field: "info",
        }
    })?;
    let title = block_scalar_value(&lines, info_range.clone(), 2, "title").ok_or_else(|| {
        OpenApiSourceError::MissingInfoField {
            path: path.into(),
            field: "title",
        }
    })?;
    if title.trim().is_empty() {
        return Err(OpenApiSourceError::MissingInfoField {
            path: path.into(),
            field: "title",
        });
    }
    let info_version = block_scalar_value(&lines, info_range, 2, "version").ok_or_else(|| {
        OpenApiSourceError::MissingInfoField {
            path: path.into(),
            field: "version",
        }
    })?;
    let info_major =
        semver_major(&info_version).ok_or_else(|| OpenApiSourceError::InvalidInfoVersion {
            path: path.into(),
            version: info_version.clone(),
        })?;
    let path_major = version_suffix_major(path).ok_or_else(|| OpenApiSourceError::InvalidPath {
        path: path.into(),
        reason: "OpenAPI document filenames must end in -v<major>".into(),
    })?;
    if info_major != path_major {
        return Err(OpenApiSourceError::VersionSuffixMismatch {
            path: path.into(),
            path_major,
            info_major,
        });
    }

    let paths_range = top_level_block(&lines, "paths").ok_or_else(|| {
        OpenApiSourceError::MissingTopLevelField {
            path: path.into(),
            field: "paths",
        }
    })?;
    validate_bearer_security_scheme(path, &lines)?;
    let operations_checked = validate_paths(path, &lines, paths_range)?;
    let data_class_annotations_checked = validate_data_class_annotations(path, &lines)?;
    Ok(DocumentValidationReport {
        operations_checked,
        data_class_annotations_checked,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpenApiOperation {
    operation_id: String,
    contract_path: String,
    response_statuses: BTreeSet<String>,
    non_explicit_response_keys: BTreeSet<String>,
    response_schema_refs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OpenApiResponseKeys {
    explicit_statuses: BTreeSet<String>,
    non_explicit_keys: BTreeSet<String>,
    schema_refs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathItemOperationBlock {
    method: String,
    range: std::ops::Range<usize>,
}

fn collect_document_operations(
    path: &str,
    contents: &str,
) -> Result<Vec<OpenApiOperation>, OpenApiSourceError> {
    let lines = logical_lines(contents);
    let paths_range = top_level_block(&lines, "paths").ok_or_else(|| {
        OpenApiSourceError::MissingTopLevelField {
            path: path.into(),
            field: "paths",
        }
    })?;
    let mut operations = Vec::new();
    let mut index = paths_range.start;
    while index < paths_range.end {
        let line = &lines[index];
        if line.indent != 2 || !line.text.starts_with('/') {
            index += 1;
            continue;
        }
        let Some(api_path) = yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        let next_path_index = find_next_at_or_above_indent(&lines, index + 1, paths_range.end, 2);
        collect_path_item_operations(
            path,
            &api_path,
            &lines,
            index + 1..next_path_index,
            &mut operations,
        )?;
        index = next_path_index;
    }
    Ok(operations)
}

fn collect_path_item_operations(
    document_path: &str,
    api_path: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    operations: &mut Vec<OpenApiOperation>,
) -> Result<(), OpenApiSourceError> {
    for block in path_item_operation_blocks(document_path, api_path, lines, range)? {
        let operation_id = operation_id_in_range(lines, block.range.clone()).ok_or_else(|| {
            OpenApiSourceError::MissingOperationId {
                path: document_path.into(),
                api_path: api_path.into(),
                method: block.method.clone(),
            }
        })?;
        let response_keys = response_keys_in_range(lines, block.range);
        operations.push(OpenApiOperation {
            operation_id,
            contract_path: document_path.into(),
            response_statuses: response_keys.explicit_statuses,
            non_explicit_response_keys: response_keys.non_explicit_keys,
            response_schema_refs: response_keys.schema_refs,
        });
    }
    Ok(())
}

fn path_item_operation_blocks(
    document_path: &str,
    api_path: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Result<Vec<PathItemOperationBlock>, OpenApiSourceError> {
    let mut blocks = Vec::new();
    let mut index = range.start;
    while index < range.end {
        let line = &lines[index];
        let Some(key) = yaml_key(&line.text) else {
            index += 1;
            continue;
        };
        if line.indent != 4 {
            index += 1;
            continue;
        }
        if is_fixed_operation_method(key) {
            let next_operation_index = find_next_at_or_above_indent(lines, index + 1, range.end, 4);
            blocks.push(PathItemOperationBlock {
                method: key.into(),
                range: index + 1..next_operation_index,
            });
            index = next_operation_index;
            continue;
        }
        if key == ADDITIONAL_OPERATIONS_FIELD {
            let additional_end = find_next_at_or_above_indent(lines, index + 1, range.end, 4);
            collect_additional_operation_blocks(
                document_path,
                api_path,
                lines,
                index + 1..additional_end,
                &mut blocks,
            )?;
            index = additional_end;
            continue;
        }
        index += 1;
    }
    Ok(blocks)
}

fn collect_additional_operation_blocks(
    document_path: &str,
    api_path: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    blocks: &mut Vec<PathItemOperationBlock>,
) -> Result<(), OpenApiSourceError> {
    let mut index = range.start;
    while index < range.end {
        let line = &lines[index];
        if line.indent != 6 {
            index += 1;
            continue;
        }
        let Some(method) = yaml_key(&line.text) else {
            index += 1;
            continue;
        };
        if collides_with_fixed_operation_method(method) {
            return Err(
                OpenApiSourceError::AdditionalOperationFixedMethodCollision {
                    path: document_path.into(),
                    api_path: api_path.into(),
                    method: method.into(),
                },
            );
        }
        let next_operation_index = find_next_at_or_above_indent(lines, index + 1, range.end, 6);
        blocks.push(PathItemOperationBlock {
            method: method.into(),
            range: index + 1..next_operation_index,
        });
        index = next_operation_index;
    }
    Ok(())
}

fn operation_id_in_range(lines: &[LogicalLine], range: std::ops::Range<usize>) -> Option<String> {
    lines[range]
        .iter()
        .find(|line| {
            line.indent > 4 && yaml_key(&line.text).is_some_and(|found| found == "operationId")
        })
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
        .filter(|value| !value.trim().is_empty())
}

fn response_keys_in_range(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> OpenApiResponseKeys {
    let Some(responses_index) = lines[range.clone()].iter().position(|line| {
        line.indent > 4 && yaml_key(&line.text).is_some_and(|found| found == "responses")
    }) else {
        return OpenApiResponseKeys::default();
    };
    let responses_index = range.start + responses_index;
    let responses_indent = lines[responses_index].indent;
    let responses_end =
        find_next_at_or_above_indent(lines, responses_index + 1, range.end, responses_indent);
    let mut response_keys = OpenApiResponseKeys::default();
    let response_indent = responses_indent + 2;
    let mut index = responses_index + 1;
    while index < responses_end {
        let line = &lines[index];
        let Some(key) = yaml_key(&line.text) else {
            index += 1;
            continue;
        };
        if line.indent != response_indent || !valid_response_key(key) {
            index += 1;
            continue;
        }
        let response_end =
            find_next_at_or_above_indent(lines, index + 1, responses_end, line.indent);
        if valid_numeric_response_status(key) {
            response_keys.explicit_statuses.insert(key.to_string());
            if let Some(schema_ref) = response_schema_ref_for(lines, index + 1..response_end) {
                response_keys
                    .schema_refs
                    .insert(key.to_string(), schema_ref);
            }
        } else {
            response_keys.non_explicit_keys.insert(key.to_string());
        }
        index = response_end;
    }
    response_keys
}

fn response_schema_ref_for(
    lines: &[LogicalLine],
    response_range: std::ops::Range<usize>,
) -> Option<String> {
    let content_index = lines[response_range.clone()]
        .iter()
        .position(|line| yaml_key(&line.text).is_some_and(|found| found == "content"))?
        + response_range.start;
    let content_indent = lines[content_index].indent;
    let content_end =
        find_next_at_or_above_indent(lines, content_index + 1, response_range.end, content_indent);
    let json_index = lines[content_index + 1..content_end]
        .iter()
        .position(|line| yaml_key(&line.text).is_some_and(|found| found == "application/json"))?
        + content_index
        + 1;
    let json_indent = lines[json_index].indent;
    let json_end = find_next_at_or_above_indent(lines, json_index + 1, content_end, json_indent);
    let schema_index = lines[json_index + 1..json_end]
        .iter()
        .position(|line| yaml_key(&line.text).is_some_and(|found| found == "schema"))?
        + json_index
        + 1;
    let schema_indent = lines[schema_index].indent;
    let schema_end = find_next_at_or_above_indent(lines, schema_index + 1, json_end, schema_indent);
    lines[schema_index + 1..schema_end]
        .iter()
        .find(|line| yaml_key(&line.text).is_some_and(|found| found == "$ref"))
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
        .and_then(|value| component_schema_ref(&value))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpenApiSchemaShape {
    schema_name: String,
    contract_path: String,
    properties: BTreeMap<String, OpenApiSchemaProperty>,
    required: BTreeSet<String>,
    unsupported_keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RustStructShape {
    fields: BTreeMap<String, RustFieldShape>,
    required_fields: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpenApiSchemaProperty {
    type_name: Option<String>,
    format: Option<String>,
    rust_type: Option<String>,
    ref_schema: Option<String>,
    items_ref_schema: Option<String>,
    items_type_name: Option<String>,
    items_format: Option<String>,
    nullable: bool,
    unsupported_keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RustFieldShape {
    rust_type: String,
    required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedOpenApiScalar {
    type_name: &'static str,
    format: Option<&'static str>,
}

fn collect_component_schemas(
    path: &str,
    contents: &str,
) -> Result<Vec<OpenApiSchemaShape>, OpenApiSourceError> {
    let lines = logical_lines(contents);
    let Some(components_range) = top_level_block(&lines, "components") else {
        return Ok(Vec::new());
    };
    let Some(schemas_index) = lines[components_range.clone()].iter().position(|line| {
        line.indent == 2 && yaml_key(&line.text).is_some_and(|found| found == "schemas")
    }) else {
        return Ok(Vec::new());
    };
    let schemas_index = components_range.start + schemas_index;
    let schemas_indent = lines[schemas_index].indent;
    let schemas_end = find_next_at_or_above_indent(
        &lines,
        schemas_index + 1,
        components_range.end,
        schemas_indent,
    );
    let schema_indent = schemas_indent + 2;
    let mut schemas = Vec::new();
    let mut index = schemas_index + 1;
    while index < schemas_end {
        let line = &lines[index];
        if line.indent != schema_indent {
            index += 1;
            continue;
        }
        let Some(schema_name) = yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        let schema_end =
            find_next_at_or_above_indent(&lines, index + 1, schemas_end, schema_indent);
        let schema_body_range = index + 1..schema_end;
        schemas.push(OpenApiSchemaShape {
            schema_name,
            contract_path: path.into(),
            properties: collect_schema_properties(&lines, schema_body_range.clone()),
            required: collect_schema_required_names(&lines, schema_body_range.clone()),
            unsupported_keywords: unsupported_schema_keywords(
                &lines,
                schema_body_range,
                schema_indent + 2,
            ),
        });
        index = schema_end;
    }
    Ok(schemas)
}

fn collect_schema_properties(
    lines: &[LogicalLine],
    schema_range: std::ops::Range<usize>,
) -> BTreeMap<String, OpenApiSchemaProperty> {
    let Some(properties_index) = lines[schema_range.clone()]
        .iter()
        .position(|line| yaml_key(&line.text).is_some_and(|found| found == "properties"))
    else {
        return BTreeMap::new();
    };
    let properties_index = schema_range.start + properties_index;
    let properties_indent = lines[properties_index].indent;
    let property_indent = properties_indent + 2;
    let properties_end = find_next_at_or_above_indent(
        lines,
        properties_index + 1,
        schema_range.end,
        properties_indent,
    );
    let mut properties = BTreeMap::new();
    let mut index = properties_index + 1;
    while index < properties_end {
        let line = &lines[index];
        if line.indent != property_indent {
            index += 1;
            continue;
        }
        let Some(property_name) = yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        let property_end =
            find_next_at_or_above_indent(lines, index + 1, properties_end, property_indent);
        properties.insert(
            property_name,
            collect_schema_property(lines, index + 1..property_end, property_indent + 2),
        );
        index = property_end;
    }
    properties
}

fn collect_schema_property(
    lines: &[LogicalLine],
    property_range: std::ops::Range<usize>,
    field_indent: usize,
) -> OpenApiSchemaProperty {
    OpenApiSchemaProperty {
        type_name: scalar_value_at_indent(lines, property_range.clone(), field_indent, "type"),
        format: scalar_value_at_indent(lines, property_range.clone(), field_indent, "format"),
        nullable: scalar_value_at_indent(lines, property_range.clone(), field_indent, "nullable")
            .is_some_and(|value| value == "true"),
        ref_schema: scalar_value_at_indent(lines, property_range.clone(), field_indent, "$ref")
            .and_then(|value| component_schema_ref(&value)),
        items_ref_schema: schema_items_ref(lines, property_range.clone(), field_indent),
        items_type_name: schema_items_scalar_value(
            lines,
            property_range.clone(),
            field_indent,
            "type",
        ),
        items_format: schema_items_scalar_value(
            lines,
            property_range.clone(),
            field_indent,
            "format",
        ),
        unsupported_keywords: unsupported_schema_keywords(
            lines,
            property_range.clone(),
            field_indent,
        ),
        rust_type: scalar_value_at_indent(
            lines,
            property_range,
            field_indent,
            "x-oyatie-rust-type",
        )
        .map(|value| canonical_rust_type(&value)),
    }
}

fn schema_items_ref(
    lines: &[LogicalLine],
    property_range: std::ops::Range<usize>,
    field_indent: usize,
) -> Option<String> {
    let items_index = lines[property_range.clone()].iter().position(|line| {
        line.indent == field_indent && yaml_key(&line.text).is_some_and(|found| found == "items")
    })? + property_range.start;
    let items_end = find_next_at_or_above_indent(
        lines,
        items_index + 1,
        property_range.end,
        lines[items_index].indent,
    );
    let item_field_indent = lines[items_index].indent + 2;
    lines[items_index + 1..items_end]
        .iter()
        .find(|line| {
            line.indent == item_field_indent
                && yaml_key(&line.text).is_some_and(|found| found == "$ref")
        })
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
        .and_then(|value| component_schema_ref(&value))
}

fn schema_items_scalar_value(
    lines: &[LogicalLine],
    property_range: std::ops::Range<usize>,
    field_indent: usize,
    key: &str,
) -> Option<String> {
    let items_index = lines[property_range.clone()].iter().position(|line| {
        line.indent == field_indent && yaml_key(&line.text).is_some_and(|found| found == "items")
    })? + property_range.start;
    let items_end = find_next_at_or_above_indent(
        lines,
        items_index + 1,
        property_range.end,
        lines[items_index].indent,
    );
    scalar_value_at_indent(
        lines,
        items_index + 1..items_end,
        lines[items_index].indent + 2,
        key,
    )
}

fn unsupported_schema_keywords(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    indent: usize,
) -> Vec<String> {
    lines[range]
        .iter()
        .filter(|line| line.indent == indent)
        .filter_map(|line| yaml_key(&line.text))
        .filter(|key| unsupported_schema_keyword(key))
        .map(str::to_string)
        .collect()
}

fn unsupported_schema_keyword(key: &str) -> bool {
    matches!(
        key,
        "allOf"
            | "anyOf"
            | "oneOf"
            | "not"
            | "additionalProperties"
            | "patternProperties"
            | "dependentSchemas"
            | "unevaluatedProperties"
            | "if"
            | "then"
            | "else"
            | "contains"
            | "const"
    )
}

fn scalar_value_at_indent(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    indent: usize,
    key: &str,
) -> Option<String> {
    lines[range]
        .iter()
        .find(|line| {
            line.indent == indent && yaml_key(&line.text).is_some_and(|found| found == key)
        })
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
}

fn collect_schema_required_names(
    lines: &[LogicalLine],
    schema_range: std::ops::Range<usize>,
) -> BTreeSet<String> {
    let Some(required_index) = lines[schema_range.clone()]
        .iter()
        .position(|line| yaml_key(&line.text).is_some_and(|found| found == "required"))
    else {
        return BTreeSet::new();
    };
    let required_index = schema_range.start + required_index;
    let required_indent = lines[required_index].indent;
    let required_end =
        find_next_at_or_above_indent(lines, required_index + 1, schema_range.end, required_indent);
    lines[required_index + 1..required_end]
        .iter()
        .filter(|line| line.indent > required_indent)
        .filter_map(|line| list_item_scalar(&line.text))
        .collect()
}

fn parse_rust_struct_fields(
    contents: &str,
    rust_struct: &str,
) -> Result<Option<RustStructShape>, String> {
    let code = rust_code_without_comments_and_literals(contents);
    let Some(struct_index) = find_public_rust_struct(&code, rust_struct) else {
        return Ok(None);
    };
    let Some(relative_brace_index) = code[struct_index..].find('{') else {
        return Err(format!("{rust_struct} struct declaration must have a body"));
    };
    let brace_index = struct_index + relative_brace_index;
    let Some(block_end) = balanced_brace_end(&code, brace_index) else {
        return Err(format!(
            "{rust_struct} struct declaration has unbalanced braces"
        ));
    };
    let mut fields = BTreeMap::new();
    let mut required_fields = BTreeSet::new();
    let rename_all = serde_rename_all_before_item(contents, struct_index, rust_struct)?;
    let source_body = &contents[brace_index + 1..block_end];
    let code_body = &code[brace_index + 1..block_end];
    let mut pending_serde_rename = None::<String>;
    let mut pending_serde_skip = false;
    let mut pending_serde_flatten = false;
    let mut pending_serde_skip_serializing_if = false;
    let mut pending_serde_default = false;
    let mut pending_serde_alias = None::<String>;
    let mut pending_unsupported_serde = None::<String>;
    for (source_line, code_line) in source_body.lines().zip(code_body.lines()) {
        let source_trimmed = source_line.trim();
        let code_trimmed = code_line.trim();
        if source_trimmed.starts_with("#[") {
            if let Some(unsupported) = unsupported_serde_field_attribute(source_trimmed) {
                pending_unsupported_serde.get_or_insert(unsupported);
            }
            if let Some(rename) = serde_rename_attribute(source_trimmed) {
                pending_serde_rename = Some(rename);
            }
            if serde_skip_attribute(source_trimmed) {
                pending_serde_skip = true;
            }
            if serde_flatten_attribute(source_trimmed) {
                pending_serde_flatten = true;
            }
            if serde_skip_serializing_if_attribute(source_trimmed) {
                pending_serde_skip_serializing_if = true;
            }
            if serde_default_attribute(source_trimmed) {
                pending_serde_default = true;
            }
            if let Some(alias) = serde_alias_attribute(source_trimmed) {
                pending_serde_alias = Some(alias);
            }
            continue;
        }
        if let Some((field_name, field)) = parse_rust_public_field(code_trimmed) {
            if let Some(unsupported) = pending_unsupported_serde.as_deref() {
                return Err(format!(
                    "unsupported serde {unsupported} on field {field_name}"
                ));
            }
            if pending_serde_flatten {
                return Err(format!("unsupported serde flatten on field {field_name}"));
            }
            if let Some(alias) = pending_serde_alias.as_deref() {
                return Err(format!(
                    "unsupported serde alias {alias} on field {field_name}"
                ));
            }
            if pending_serde_skip {
                pending_serde_rename = None;
                pending_serde_skip = false;
                pending_serde_flatten = false;
                pending_serde_skip_serializing_if = false;
                pending_serde_default = false;
                pending_serde_alias = None;
                pending_unsupported_serde = None;
                continue;
            }
            let field_name = pending_serde_rename
                .take()
                .or_else(|| {
                    rename_all
                        .as_deref()
                        .map(|style| apply_supported_serde_rename_all(&field_name, style))
                })
                .unwrap_or(field_name);
            let field = if pending_serde_skip_serializing_if || pending_serde_default {
                RustFieldShape {
                    required: false,
                    rust_type: field.rust_type,
                }
            } else {
                field
            };
            if field.required {
                required_fields.insert(field_name.clone());
            }
            fields.insert(field_name, field);
            pending_serde_skip_serializing_if = false;
            pending_serde_default = false;
            pending_serde_alias = None;
            pending_unsupported_serde = None;
        } else if !code_trimmed.is_empty() {
            pending_serde_rename = None;
            pending_serde_skip = false;
            pending_serde_flatten = false;
            pending_serde_skip_serializing_if = false;
            pending_serde_default = false;
            pending_serde_alias = None;
            pending_unsupported_serde = None;
        }
    }
    Ok(Some(RustStructShape {
        fields,
        required_fields,
    }))
}

fn serde_rename_all_before_item(
    source: &str,
    item_index: usize,
    rust_struct: &str,
) -> Result<Option<String>, String> {
    let item_line_start = source[..item_index]
        .rfind('\n')
        .map_or(0usize, |index| index + 1);
    let mut rename_all = None;
    for line in source[..item_line_start].lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("#[") {
            break;
        }
        if let Some(unsupported) = unsupported_serde_struct_attribute(trimmed) {
            return Err(format!(
                "unsupported serde {unsupported} on struct {rust_struct}"
            ));
        }
        if let Some(found) = serde_rename_all_attribute(trimmed) {
            if !supported_serde_rename_all_rule(&found) {
                return Err(format!("unsupported serde rename_all rule {found}"));
            }
            rename_all = Some(found);
        }
    }
    Ok(rename_all)
}

fn serde_rename_attribute(attribute: &str) -> Option<String> {
    serde_attribute_string_value(attribute, "rename")
}

fn serde_alias_attribute(attribute: &str) -> Option<String> {
    serde_attribute_string_value(attribute, "alias")
}

fn serde_rename_all_attribute(attribute: &str) -> Option<String> {
    serde_attribute_string_value(attribute, "rename_all")
}

fn serde_skip_attribute(attribute: &str) -> bool {
    serde_word_attribute(attribute, "skip")
}

fn serde_flatten_attribute(attribute: &str) -> bool {
    serde_word_attribute(attribute, "flatten")
}

fn serde_skip_serializing_if_attribute(attribute: &str) -> bool {
    serde_attribute_string_value(attribute, "skip_serializing_if").is_some()
}

fn serde_default_attribute(attribute: &str) -> bool {
    serde_word_attribute(attribute, "default")
        || serde_attribute_string_value(attribute, "default").is_some()
}

fn serde_word_attribute(attribute: &str, word: &str) -> bool {
    serde_attribute_body(attribute).is_some_and(|body| {
        serde_attribute_clauses(body)
            .into_iter()
            .any(|clause| clause.trim() == word)
    })
}

fn serde_attribute_string_value(attribute: &str, key: &str) -> Option<String> {
    let body = serde_attribute_body(attribute)?;
    serde_attribute_clauses(body)
        .into_iter()
        .find_map(|clause| serde_clause_string_value(clause.trim(), key))
}

fn serde_attribute_body(attribute: &str) -> Option<&str> {
    attribute
        .trim()
        .strip_prefix("#[serde(")?
        .strip_suffix(")]")
}

fn serde_attribute_clauses(body: &str) -> Vec<&str> {
    let bytes = body.as_bytes();
    let mut clauses = Vec::new();
    let mut clause_start = 0usize;
    let mut index = 0usize;
    let mut nesting_depth = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                let (next_index, _) = quoted_string_literal_value(bytes, index + 1);
                index = next_index;
            }
            b'(' | b'[' | b'{' => {
                nesting_depth += 1;
                index += 1;
            }
            b')' | b']' | b'}' => {
                nesting_depth = nesting_depth.saturating_sub(1);
                index += 1;
            }
            b',' if nesting_depth == 0 => {
                clauses.push(&body[clause_start..index]);
                index += 1;
                clause_start = index;
            }
            _ => index += 1,
        }
    }
    clauses.push(&body[clause_start..]);
    clauses
}

fn serde_clause_string_value(clause: &str, key: &str) -> Option<String> {
    let rest = clause.strip_prefix(key)?;
    if rest
        .chars()
        .next()
        .is_some_and(is_rust_identifier_character)
    {
        return None;
    }
    let equals_index = skip_whitespace(clause, key.len());
    if !clause[equals_index..].starts_with('=') {
        return None;
    }
    let value_index = skip_whitespace(clause, equals_index + 1);
    if clause.as_bytes().get(value_index).copied() != Some(b'"') {
        return None;
    }
    let (_, value) = quoted_string_literal_value(clause.as_bytes(), value_index + 1);
    (!value.is_empty()).then_some(value)
}

fn unsupported_serde_field_attribute(attribute: &str) -> Option<String> {
    let body = serde_attribute_body(attribute)?;
    serde_attribute_clauses(body)
        .into_iter()
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .find(|clause| !supported_serde_field_clause(clause))
        .map(serde_clause_name)
}

fn supported_serde_field_clause(clause: &str) -> bool {
    matches!(clause, "skip" | "flatten" | "default")
        || serde_clause_string_value(clause, "rename").is_some()
        || serde_clause_string_value(clause, "alias").is_some()
        || serde_clause_string_value(clause, "skip_serializing_if").is_some()
        || serde_clause_string_value(clause, "default").is_some()
}

fn unsupported_serde_struct_attribute(attribute: &str) -> Option<String> {
    let body = serde_attribute_body(attribute)?;
    serde_attribute_clauses(body)
        .into_iter()
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .find(|clause| !supported_serde_struct_clause(clause))
        .map(serde_clause_name)
}

fn supported_serde_struct_clause(clause: &str) -> bool {
    serde_clause_string_value(clause, "rename_all").is_some()
}

fn serde_clause_name(clause: &str) -> String {
    let mut end = 0usize;
    for (index, character) in clause.char_indices() {
        if index == 0 && !is_rust_identifier_character(character) {
            return clause.to_string();
        }
        if !is_rust_identifier_character(character) {
            break;
        }
        end = index + character.len_utf8();
    }
    if end == 0 {
        clause.to_string()
    } else {
        clause[..end].to_string()
    }
}

fn supported_serde_rename_all_rule(style: &str) -> bool {
    matches!(style, "camelCase" | "snake_case")
}

fn apply_supported_serde_rename_all(field_name: &str, style: &str) -> String {
    match style {
        "camelCase" => snake_to_lower_camel(field_name),
        "snake_case" => field_name.to_string(),
        _ => unreachable!("unsupported serde rename_all rules fail before field mapping"),
    }
}

fn snake_to_lower_camel(field_name: &str) -> String {
    let mut renamed = String::new();
    let mut uppercase_next = false;
    for character in field_name.chars() {
        if character == '_' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            renamed.extend(character.to_uppercase());
            uppercase_next = false;
        } else {
            renamed.push(character);
        }
    }
    renamed
}

fn parse_rust_public_field(line: &str) -> Option<(String, RustFieldShape)> {
    let visible = line.split("//").next()?.trim();
    let field = visible.strip_prefix("pub ")?;
    let (name, field_type) = field.split_once(':')?;
    let name = name.trim();
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        return None;
    }
    let rust_type = canonical_rust_type(field_type.trim().trim_end_matches(',').trim());
    Some((
        name.to_string(),
        RustFieldShape {
            required: strip_option_type(&rust_type).is_none(),
            rust_type,
        },
    ))
}

fn validate_schema_fields_match(
    schema: &OpenApiSchemaShape,
    runtime: &RustStructShape,
) -> Result<(), OpenApiSourceError> {
    let schema_fields = schema.properties.keys().cloned().collect::<BTreeSet<_>>();
    let runtime_fields = runtime.fields.keys().cloned().collect::<BTreeSet<_>>();
    let missing_properties = runtime
        .fields
        .keys()
        .filter(|field| !schema_fields.contains(*field))
        .cloned()
        .collect::<Vec<_>>();
    let extra_properties = schema
        .properties
        .keys()
        .filter(|field| !runtime_fields.contains(*field))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_properties.is_empty() || !extra_properties.is_empty() {
        return Err(OpenApiSourceError::SchemaFieldMismatch {
            schema_name: schema.schema_name.clone(),
            contract_path: schema.contract_path.clone(),
            missing_properties,
            extra_properties,
        });
    }

    let missing_required = runtime
        .required_fields
        .difference(&schema.required)
        .cloned()
        .collect::<Vec<_>>();
    let extra_required = schema
        .required
        .difference(&runtime.required_fields)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_required.is_empty() || !extra_required.is_empty() {
        return Err(OpenApiSourceError::SchemaRequiredMismatch {
            schema_name: schema.schema_name.clone(),
            contract_path: schema.contract_path.clone(),
            missing_required,
            extra_required,
        });
    }
    Ok(())
}

fn validate_schema_types_match(
    schema: &OpenApiSchemaShape,
    runtime: &RustStructShape,
) -> Result<(), OpenApiSourceError> {
    let mut mismatches = schema
        .unsupported_keywords
        .iter()
        .map(|keyword| {
            format!(
                "{}: unsupported OpenAPI schema keyword {keyword}",
                schema.schema_name
            )
        })
        .collect::<Vec<_>>();
    for (field_name, runtime_field) in &runtime.fields {
        let Some(property) = schema.properties.get(field_name) else {
            continue;
        };
        let expected_rust_type = &runtime_field.rust_type;
        match &property.rust_type {
            Some(actual) if actual == expected_rust_type => {}
            Some(actual) => mismatches.push(format!(
                "{field_name}: expected x-oyatie-rust-type {expected_rust_type}, found {actual}"
            )),
            None => mismatches.push(format!(
                "{field_name}: missing x-oyatie-rust-type, expected {expected_rust_type}"
            )),
        }

        validate_schema_property_type(field_name, expected_rust_type, property, &mut mismatches);
    }
    if !mismatches.is_empty() {
        return Err(OpenApiSourceError::SchemaTypeMismatch {
            schema_name: schema.schema_name.clone(),
            contract_path: schema.contract_path.clone(),
            mismatches,
        });
    }
    Ok(())
}

fn validate_schema_property_type(
    field_name: &str,
    expected_rust_type: &str,
    property: &OpenApiSchemaProperty,
    mismatches: &mut Vec<String>,
) {
    for keyword in &property.unsupported_keywords {
        mismatches.push(format!(
            "{field_name}: unsupported OpenAPI schema keyword {keyword}"
        ));
    }

    if property.nullable && strip_option_type(expected_rust_type).is_none() {
        mismatches.push(format!(
            "{field_name}: OpenAPI nullable true requires Rust Option<...>, found {expected_rust_type}"
        ));
    }

    let unwrapped_rust_type = strip_option_type(expected_rust_type).unwrap_or(expected_rust_type);
    if let Some(item_type) = strip_vec_type(unwrapped_rust_type) {
        let actual_type = property.type_name.as_deref().unwrap_or("<missing>");
        if actual_type != "array" {
            mismatches.push(format!(
                "{field_name}: expected type array for Rust {expected_rust_type}, found type {actual_type}"
            ));
        }
        let item_scalar = expected_openapi_scalar(item_type);
        if item_scalar.type_name != "<unsupported-rust-scalar>" {
            if let Some(actual) = property.items_ref_schema.as_deref() {
                mismatches.push(format!(
                    "{field_name}: scalar array items for Rust {expected_rust_type} must not declare items $ref {actual}"
                ));
                return;
            }
            let actual_item_type = property.items_type_name.as_deref().unwrap_or("<missing>");
            let actual_item_format = property.items_format.as_deref().unwrap_or("<none>");
            let expected_item_format = item_scalar.format.unwrap_or("<none>");
            if actual_item_type != item_scalar.type_name
                || actual_item_format != expected_item_format
            {
                mismatches.push(format!(
                    "{field_name}: expected items type {} format {} for Rust {expected_rust_type}, found items type {} format {}",
                    item_scalar.type_name, expected_item_format, actual_item_type, actual_item_format
                ));
            }
            return;
        }
        match property.items_ref_schema.as_deref() {
            Some(actual) if actual == item_type => {}
            Some(actual) => mismatches.push(format!(
                "{field_name}: expected items $ref {item_type} for Rust {expected_rust_type}, found {actual}"
            )),
            None => mismatches.push(format!(
                "{field_name}: missing items $ref, expected {item_type} for Rust {expected_rust_type}"
            )),
        }
        return;
    }

    let scalar = expected_openapi_scalar(unwrapped_rust_type);
    if scalar.type_name != "<unsupported-rust-scalar>" {
        let actual_type = property.type_name.as_deref().unwrap_or("<missing>");
        let actual_format = property.format.as_deref().unwrap_or("<none>");
        let expected_format = scalar.format.unwrap_or("<none>");
        if actual_type != scalar.type_name || actual_format != expected_format {
            mismatches.push(format!(
                "{field_name}: expected type {} format {} for Rust {}, found type {} format {}",
                scalar.type_name, expected_format, expected_rust_type, actual_type, actual_format
            ));
        }
        return;
    }

    match property.ref_schema.as_deref() {
        Some(actual) if actual == unwrapped_rust_type => {
            if let Some(actual_type) = property.type_name.as_deref() {
                mismatches.push(format!(
                    "{field_name}: $ref property for Rust {expected_rust_type} must not declare type {actual_type}"
                ));
            }
            if let Some(actual_format) = property.format.as_deref() {
                mismatches.push(format!(
                    "{field_name}: $ref property for Rust {expected_rust_type} must not declare format {actual_format}"
                ));
            }
        }
        Some(actual) => mismatches.push(format!(
            "{field_name}: expected $ref {unwrapped_rust_type} for Rust {expected_rust_type}, found {actual}"
        )),
        None => mismatches.push(format!(
            "{field_name}: missing $ref, expected {unwrapped_rust_type} for Rust {expected_rust_type}"
        )),
    }
}

fn expected_openapi_scalar(rust_type: &str) -> ExpectedOpenApiScalar {
    let scalar_type = strip_option_type(rust_type).unwrap_or(rust_type);
    match scalar_type {
        "String" | "Purpose" | "SubjectClass" | "BudgetWarning" => ExpectedOpenApiScalar {
            type_name: "string",
            format: None,
        },
        "bool" => ExpectedOpenApiScalar {
            type_name: "boolean",
            format: None,
        },
        "u8" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("uint8"),
        },
        "u16" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("uint16"),
        },
        "u32" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("uint32"),
        },
        "u64" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("uint64"),
        },
        "i8" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("int8"),
        },
        "i16" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("int16"),
        },
        "i32" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("int32"),
        },
        "i64" => ExpectedOpenApiScalar {
            type_name: "integer",
            format: Some("int64"),
        },
        "f32" => ExpectedOpenApiScalar {
            type_name: "number",
            format: Some("float"),
        },
        "f64" => ExpectedOpenApiScalar {
            type_name: "number",
            format: Some("double"),
        },
        _ => ExpectedOpenApiScalar {
            type_name: "<unsupported-rust-scalar>",
            format: None,
        },
    }
}

fn strip_option_type(rust_type: &str) -> Option<&str> {
    rust_type
        .strip_prefix("Option<")
        .and_then(|inner| inner.strip_suffix('>'))
}

fn strip_vec_type(rust_type: &str) -> Option<&str> {
    rust_type
        .strip_prefix("Vec<")
        .and_then(|inner| inner.strip_suffix('>'))
}

fn canonical_rust_type(rust_type: &str) -> String {
    rust_type
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LogicalLine {
    indent: usize,
    text: String,
}

fn logical_lines(contents: &str) -> Vec<LogicalLine> {
    contents
        .lines()
        .filter_map(|raw| {
            let without_comment = strip_yaml_comment(raw);
            let trimmed = without_comment.trim_end();
            if trimmed.trim().is_empty() {
                return None;
            }
            let indent = trimmed
                .chars()
                .take_while(|character| *character == ' ')
                .count();
            Some(LogicalLine {
                indent,
                text: trimmed[indent..].trim().to_string(),
            })
        })
        .collect()
}

fn strip_yaml_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut previous = '\0';
    for (index, character) in line.char_indices() {
        match character {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single && previous != '\\' => in_double = !in_double,
            '#' if !in_single && !in_double => return line[..index].to_string(),
            _ => {}
        }
        previous = character;
    }
    line.to_string()
}

fn top_level_value(lines: &[LogicalLine], key: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| line.indent == 0 && yaml_key(&line.text).is_some_and(|found| found == key))
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
}

fn top_level_block(lines: &[LogicalLine], key: &str) -> Option<std::ops::Range<usize>> {
    let start = lines.iter().position(|line| {
        line.indent == 0 && yaml_key(&line.text).is_some_and(|found| found == key)
    })?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.indent == 0)
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    Some(start + 1..end)
}

fn block_scalar_value(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    indent: usize,
    key: &str,
) -> Option<String> {
    lines[range]
        .iter()
        .find(|line| {
            line.indent == indent && yaml_key(&line.text).is_some_and(|found| found == key)
        })
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
}

fn validate_paths(
    document_path: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Result<usize, OpenApiSourceError> {
    let mut operations_checked = 0usize;
    let mut path_item_seen = false;
    let mut index = range.start;
    while index < range.end {
        let line = &lines[index];
        if line.indent != 2 || !line.text.starts_with('/') {
            index += 1;
            continue;
        }
        let Some(api_path) = yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        path_item_seen = true;
        let next_path_index = find_next_at_or_above_indent(lines, index + 1, range.end, 2);
        let operation_count =
            validate_path_item(document_path, &api_path, lines, index + 1..next_path_index)?;
        operations_checked += operation_count;
        index = next_path_index;
    }

    if !path_item_seen {
        return Err(OpenApiSourceError::MissingPathItem {
            path: document_path.into(),
        });
    }
    Ok(operations_checked)
}

fn validate_path_item(
    document_path: &str,
    api_path: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Result<usize, OpenApiSourceError> {
    let operation_blocks =
        path_item_operation_blocks(document_path, api_path, lines, range.clone())?;
    let inherited_parameters = path_item_parameters(lines, range.clone());
    for block in &operation_blocks {
        validate_operation(
            document_path,
            api_path,
            &block.method,
            lines,
            block.range.clone(),
            &inherited_parameters,
        )?;
    }

    if operation_blocks.is_empty() {
        return Err(OpenApiSourceError::MissingOperation {
            path: document_path.into(),
            api_path: api_path.into(),
        });
    }
    Ok(operation_blocks.len())
}

fn validate_operation(
    document_path: &str,
    api_path: &str,
    method: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    inherited_parameters: &[OperationParameter],
) -> Result<(), OpenApiSourceError> {
    let has_operation_id = lines[range.clone()].iter().any(|line| {
        line.indent > 4
            && yaml_key(&line.text).is_some_and(|found| found == "operationId")
            && yaml_value(&line.text)
                .map(clean_yaml_scalar)
                .is_some_and(|value| !value.trim().is_empty())
    });
    if !has_operation_id {
        return Err(OpenApiSourceError::MissingOperationId {
            path: document_path.into(),
            api_path: api_path.into(),
            method: method.into(),
        });
    }

    let Some(responses_index) = lines[range.clone()].iter().position(|line| {
        line.indent > 4 && yaml_key(&line.text).is_some_and(|found| found == "responses")
    }) else {
        return Err(OpenApiSourceError::MissingResponses {
            path: document_path.into(),
            api_path: api_path.into(),
            method: method.into(),
        });
    };
    let responses_index = range.start + responses_index;
    let responses_end = find_next_at_or_above_indent(
        lines,
        responses_index + 1,
        range.end,
        lines[responses_index].indent,
    );
    let has_response = lines[responses_index + 1..responses_end]
        .iter()
        .any(|line| {
            line.indent > lines[responses_index].indent
                && yaml_key(&line.text).is_some_and(valid_response_key)
        });
    if !has_response {
        return Err(OpenApiSourceError::MissingResponses {
            path: document_path.into(),
            api_path: api_path.into(),
            method: method.into(),
        });
    }
    let parameters = merged_operation_parameters(
        inherited_parameters,
        operation_parameters(lines, range.clone()),
    );
    validate_path_template_parameters(document_path, api_path, method, &parameters)?;
    validate_mutating_operation_ingress(
        document_path,
        api_path,
        method,
        lines,
        range,
        &parameters,
    )?;
    Ok(())
}

fn validate_bearer_security_scheme(
    document_path: &str,
    lines: &[LogicalLine],
) -> Result<(), OpenApiSourceError> {
    let components_range = top_level_block(lines, "components").ok_or_else(|| {
        OpenApiSourceError::MissingBearerSecurityScheme {
            path: document_path.into(),
        }
    })?;
    let security_schemes_index = lines[components_range.clone()]
        .iter()
        .position(|line| {
            line.indent == 2 && yaml_key(&line.text).is_some_and(|key| key == "securitySchemes")
        })
        .map(|offset| components_range.start + offset)
        .ok_or_else(|| OpenApiSourceError::MissingBearerSecurityScheme {
            path: document_path.into(),
        })?;
    let security_schemes_end = find_next_at_or_above_indent(
        lines,
        security_schemes_index + 1,
        components_range.end,
        lines[security_schemes_index].indent,
    );
    let scheme_index = lines[security_schemes_index + 1..security_schemes_end]
        .iter()
        .position(|line| {
            line.indent == 4
                && yaml_key(&line.text).is_some_and(|key| key == BEARER_SECURITY_SCHEME)
        })
        .map(|offset| security_schemes_index + 1 + offset)
        .ok_or_else(|| OpenApiSourceError::MissingBearerSecurityScheme {
            path: document_path.into(),
        })?;
    let scheme_end = find_next_at_or_above_indent(lines, scheme_index + 1, security_schemes_end, 4);
    let type_value = scalar_value_at_indent(lines, scheme_index + 1..scheme_end, 6, "type");
    if type_value.as_deref() != Some("http") {
        return Err(OpenApiSourceError::InvalidBearerSecurityScheme {
            path: document_path.into(),
            reason: "bearerAuth type must be http".into(),
        });
    }
    let scheme_value = scalar_value_at_indent(lines, scheme_index + 1..scheme_end, 6, "scheme");
    if scheme_value.as_deref() != Some("bearer") {
        return Err(OpenApiSourceError::InvalidBearerSecurityScheme {
            path: document_path.into(),
            reason: "bearerAuth scheme must be bearer".into(),
        });
    }
    Ok(())
}

fn validate_mutating_operation_ingress(
    document_path: &str,
    api_path: &str,
    method: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    parameters: &[OperationParameter],
) -> Result<(), OpenApiSourceError> {
    if !operation_requires_mutating_ingress(method) {
        return Ok(());
    }
    if !operation_security_requires(lines, range.clone(), BEARER_SECURITY_SCHEME) {
        return Err(OpenApiSourceError::MissingOperationSecurity {
            path: document_path.into(),
            api_path: api_path.into(),
            method: method.into(),
            scheme: BEARER_SECURITY_SCHEME.into(),
        });
    }

    if parameters.iter().any(|parameter| {
        parameter.name == "Authorization" && parameter.location.as_deref() == Some("header")
    }) {
        return Err(OpenApiSourceError::ForbiddenAuthorizationParameter {
            path: document_path.into(),
            api_path: api_path.into(),
            method: method.into(),
        });
    }

    for header in REQUIRED_MUTATING_HEADERS {
        let Some(parameter) = parameters.iter().find(|parameter| {
            parameter.name == header && parameter.location.as_deref() == Some("header")
        }) else {
            return Err(OpenApiSourceError::MissingRequiredHeaderParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                header: header.into(),
            });
        };
        if parameter.required.as_deref() != Some("true") {
            return Err(OpenApiSourceError::InvalidHeaderParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                header: header.into(),
                reason: "header parameter must be required: true".into(),
            });
        }
        if parameter.schema_type.as_deref() != Some("string") {
            return Err(OpenApiSourceError::InvalidHeaderParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                header: header.into(),
                reason: "header parameter schema type must be string".into(),
            });
        }
        if parameter.min_length.as_deref() != Some("1") {
            return Err(OpenApiSourceError::InvalidHeaderParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                header: header.into(),
                reason: "header parameter schema must declare minLength: 1".into(),
            });
        }
    }
    Ok(())
}

fn validate_path_template_parameters(
    document_path: &str,
    api_path: &str,
    method: &str,
    parameters: &[OperationParameter],
) -> Result<(), OpenApiSourceError> {
    let template_parameters = path_template_parameters(api_path);
    for parameter_name in &template_parameters {
        let Some(parameter) = parameters.iter().find(|parameter| {
            &parameter.name == parameter_name && parameter.location.as_deref() == Some("path")
        }) else {
            return Err(OpenApiSourceError::MissingPathTemplateParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                parameter: parameter_name.clone(),
            });
        };
        if parameter.required.as_deref() != Some("true") {
            return Err(OpenApiSourceError::InvalidPathTemplateParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                parameter: parameter.name.clone(),
                reason: "path template parameter must be required: true".into(),
            });
        }
        if parameter.schema_type.as_deref() != Some("string") {
            return Err(OpenApiSourceError::InvalidPathTemplateParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                parameter: parameter.name.clone(),
                reason: "path template parameter schema type must be string".into(),
            });
        }
    }

    for parameter in parameters
        .iter()
        .filter(|parameter| parameter.location.as_deref() == Some("path"))
    {
        if !template_parameters.contains(&parameter.name) {
            return Err(OpenApiSourceError::InvalidPathTemplateParameter {
                path: document_path.into(),
                api_path: api_path.into(),
                method: method.into(),
                parameter: parameter.name.clone(),
                reason: "path parameter is not present in the path template".into(),
            });
        }
    }
    Ok(())
}

fn operation_requires_mutating_ingress(method: &str) -> bool {
    // OpenAPI 3.2 `additionalOperations` custom methods are treated as
    // tenant-mutating until they become a fixed safe method in this validator.
    // This keeps custom verbs such as COPY on the same bearer/idempotency
    // contract as POST/PUT/PATCH/DELETE instead of accepting unaudited ingress.
    matches!(method, "post" | "put" | "patch" | "delete") || !is_fixed_operation_method(method)
}

fn operation_security_requires(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    scheme: &str,
) -> bool {
    let Some(security_index) = lines[range.clone()].iter().position(|line| {
        line.indent > 4 && yaml_key(&line.text).is_some_and(|key| key == "security")
    }) else {
        return false;
    };
    let security_index = range.start + security_index;
    let security_end = find_next_at_or_above_indent(
        lines,
        security_index + 1,
        range.end,
        lines[security_index].indent,
    );
    lines[security_index + 1..security_end].iter().any(|line| {
        line.text.starts_with("- ") && list_item_yaml_value(&line.text, scheme).is_some()
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OperationParameter {
    name: String,
    location: Option<String>,
    required: Option<String>,
    schema_type: Option<String>,
    min_length: Option<String>,
}

fn operation_parameters(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Vec<OperationParameter> {
    let Some(parameters_index) = lines[range.clone()].iter().position(|line| {
        line.indent > 4 && yaml_key(&line.text).is_some_and(|key| key == "parameters")
    }) else {
        return Vec::new();
    };
    let parameters_index = range.start + parameters_index;
    parameters_at(lines, parameters_index, range.end)
}

fn path_item_parameters(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Vec<OperationParameter> {
    let Some(parameters_index) = lines[range.clone()].iter().position(|line| {
        line.indent == 4 && yaml_key(&line.text).is_some_and(|key| key == "parameters")
    }) else {
        return Vec::new();
    };
    let parameters_index = range.start + parameters_index;
    parameters_at(lines, parameters_index, range.end)
}

fn parameters_at(
    lines: &[LogicalLine],
    parameters_index: usize,
    range_end: usize,
) -> Vec<OperationParameter> {
    let parameters_end = find_next_at_or_above_indent(
        lines,
        parameters_index + 1,
        range_end,
        lines[parameters_index].indent,
    );
    let mut parameters = Vec::new();
    let mut index = parameters_index + 1;
    while index < parameters_end {
        let line = &lines[index];
        if !line.text.starts_with("- ") {
            index += 1;
            continue;
        }
        let parameter_end =
            find_next_at_or_above_indent(lines, index + 1, parameters_end, line.indent);
        if let Some(name) = parameter_name(lines, index, parameter_end) {
            parameters.push(OperationParameter {
                name,
                location: scalar_value_at_any_indent(lines, index..parameter_end, "in"),
                required: scalar_value_at_any_indent(lines, index..parameter_end, "required"),
                schema_type: scalar_value_at_any_indent(lines, index..parameter_end, "type"),
                min_length: scalar_value_at_any_indent(lines, index..parameter_end, "minLength"),
            });
        }
        index = parameter_end;
    }
    parameters
}

fn merged_operation_parameters(
    inherited_parameters: &[OperationParameter],
    operation_parameters: Vec<OperationParameter>,
) -> Vec<OperationParameter> {
    let mut merged = inherited_parameters.to_vec();
    for parameter in operation_parameters {
        if let Some(location) = &parameter.location {
            merged.retain(|inherited| {
                !(inherited.name == parameter.name
                    && inherited.location.as_deref() == Some(location.as_str()))
            });
        }
        merged.push(parameter);
    }
    merged
}

fn scalar_value_at_any_indent(
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
    key: &str,
) -> Option<String> {
    lines[range]
        .iter()
        .find(|line| yaml_key(&line.text).is_some_and(|found| found == key))
        .and_then(|line| yaml_value(&line.text))
        .map(clean_yaml_scalar)
}

fn validate_data_class_annotations(
    document_path: &str,
    lines: &[LogicalLine],
) -> Result<usize, OpenApiSourceError> {
    let mut annotations_checked = 0usize;
    for (index, line) in lines.iter().enumerate() {
        if yaml_key(&line.text).is_some_and(|key| key == "properties") {
            annotations_checked +=
                validate_schema_properties_data_class(document_path, lines, index)?;
        }
        if yaml_key(&line.text).is_some_and(|key| key == "parameters") {
            annotations_checked += validate_parameters_data_class(document_path, lines, index)?;
        }
    }
    Ok(annotations_checked)
}

fn validate_schema_properties_data_class(
    document_path: &str,
    lines: &[LogicalLine],
    properties_index: usize,
) -> Result<usize, OpenApiSourceError> {
    let properties_indent = lines[properties_index].indent;
    let property_indent = properties_indent + 2;
    let end =
        find_next_at_or_above_indent(lines, properties_index + 1, lines.len(), properties_indent);
    let schema_name = parent_schema_name(lines, properties_index, properties_indent);
    let mut annotations_checked = 0usize;
    let mut index = properties_index + 1;
    while index < end {
        let line = &lines[index];
        if line.indent != property_indent {
            index += 1;
            continue;
        }
        let Some(property_name) = yaml_key(&line.text).map(str::to_string) else {
            index += 1;
            continue;
        };
        let property_end = find_next_at_or_above_indent(lines, index + 1, end, property_indent);
        let location = match &schema_name {
            Some(schema_name) => format!("schema {schema_name}.{property_name}"),
            None => format!("schema property {property_name}"),
        };
        annotations_checked += validate_data_class_annotation(
            document_path,
            &location,
            lines,
            index + 1..property_end,
        )?;
        index = property_end;
    }
    Ok(annotations_checked)
}

fn validate_parameters_data_class(
    document_path: &str,
    lines: &[LogicalLine],
    parameters_index: usize,
) -> Result<usize, OpenApiSourceError> {
    let parameters_indent = lines[parameters_index].indent;
    let end =
        find_next_at_or_above_indent(lines, parameters_index + 1, lines.len(), parameters_indent);
    let mut annotations_checked = 0usize;
    let mut index = parameters_index + 1;
    while index < end {
        let line = &lines[index];
        if !line.text.starts_with("- ") {
            index += 1;
            continue;
        }
        let parameter_end = find_next_at_or_above_indent(lines, index + 1, end, line.indent);
        if let Some(parameter_name) = parameter_name(lines, index, parameter_end) {
            let location = format!("parameter {parameter_name}");
            annotations_checked += validate_data_class_annotation(
                document_path,
                &location,
                lines,
                index..parameter_end,
            )?;
        }
        index = parameter_end;
    }
    Ok(annotations_checked)
}

fn validate_data_class_annotation(
    document_path: &str,
    location: &str,
    lines: &[LogicalLine],
    range: std::ops::Range<usize>,
) -> Result<usize, OpenApiSourceError> {
    let value = lines[range].iter().find_map(|line| {
        if yaml_key(&line.text).is_some_and(|key| key == "x-oyatie-data-class") {
            yaml_value(&line.text).map(clean_yaml_scalar)
        } else {
            None
        }
    });
    let Some(data_class) = value else {
        return Err(OpenApiSourceError::MissingDataClassAnnotation {
            path: document_path.into(),
            location: location.into(),
        });
    };
    if !valid_data_class(&data_class) {
        return Err(OpenApiSourceError::InvalidDataClassAnnotation {
            path: document_path.into(),
            location: location.into(),
            data_class,
        });
    }
    Ok(1)
}

fn parent_schema_name(
    lines: &[LogicalLine],
    properties_index: usize,
    properties_indent: usize,
) -> Option<String> {
    if properties_indent < 2 {
        return None;
    }
    lines[..properties_index]
        .iter()
        .rev()
        .find(|line| line.indent + 2 == properties_indent)
        .and_then(|line| yaml_key(&line.text))
        .map(str::to_string)
}

fn parameter_name(lines: &[LogicalLine], item_index: usize, item_end: usize) -> Option<String> {
    if let Some(value) = list_item_yaml_value(&lines[item_index].text, "name") {
        return Some(value);
    }
    lines[item_index + 1..item_end].iter().find_map(|line| {
        if yaml_key(&line.text).is_some_and(|key| key == "name") {
            yaml_value(&line.text).map(clean_yaml_scalar)
        } else {
            None
        }
    })
}

fn list_item_yaml_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix("- ")?;
    let (found_key, value) = rest.split_once(':')?;
    if clean_yaml_key(found_key.trim()) == key && !value.trim().is_empty() {
        Some(clean_yaml_scalar(value.trim()))
    } else {
        None
    }
}

fn list_item_scalar(line: &str) -> Option<String> {
    line.strip_prefix("- ")
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.contains(':'))
        .map(clean_yaml_scalar)
}

fn valid_data_class(value: &str) -> bool {
    parse_data_class_label(value).is_some()
}

fn is_fixed_operation_method(method: &str) -> bool {
    FIXED_OPERATION_METHODS.contains(&method)
}

fn collides_with_fixed_operation_method(method: &str) -> bool {
    FIXED_OPERATION_METHODS
        .iter()
        .any(|fixed| fixed.eq_ignore_ascii_case(method))
}

fn find_next_at_or_above_indent(
    lines: &[LogicalLine],
    start: usize,
    end: usize,
    indent: usize,
) -> usize {
    lines[start..end]
        .iter()
        .position(|line| line.indent <= indent)
        .map(|offset| start + offset)
        .unwrap_or(end)
}

fn yaml_key(line: &str) -> Option<&str> {
    line.split_once(':')
        .map(|(key, _)| clean_yaml_key(key.trim()))
        .filter(|key| !key.is_empty())
}

fn yaml_value(line: &str) -> Option<&str> {
    line.split_once(':')
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
}

fn clean_yaml_key(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|inner| inner.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn clean_yaml_scalar(value: &str) -> String {
    clean_yaml_key(value.trim()).to_string()
}

fn component_schema_ref(value: &str) -> Option<String> {
    value
        .strip_prefix("#/components/schemas/")
        .map(str::to_string)
        .filter(|schema_name| !schema_name.trim().is_empty())
}

fn valid_response_key(key: &str) -> bool {
    key == "default" || valid_numeric_response_status(key) || valid_response_range_key(key)
}

fn valid_numeric_response_status(key: &str) -> bool {
    key.len() == 3 && key.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_response_range_key(key: &str) -> bool {
    matches!(key.as_bytes(), [b'1'..=b'5', b'X', b'X'])
}

fn path_template_parameters(api_path: &str) -> BTreeSet<String> {
    let mut parameters = BTreeSet::new();
    let mut remainder = api_path;
    while let Some(open_index) = remainder.find('{') {
        let after_open = &remainder[open_index + 1..];
        let Some(close_index) = after_open.find('}') else {
            break;
        };
        let parameter = after_open[..close_index].trim();
        if !parameter.is_empty() {
            parameters.insert(parameter.to_string());
        }
        remainder = &after_open[close_index + 1..];
    }
    parameters
}

fn version_suffix_major(path: &str) -> Option<u64> {
    let file_name = path.rsplit('/').next()?;
    let stem = file_name.split('.').next()?;
    let (_, suffix) = stem.rsplit_once("-v")?;
    suffix.parse::<u64>().ok()
}

fn semver_major(version: &str) -> Option<u64> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let _ = (minor, patch);
    Some(major)
}

fn exact_openapi_paths_in_text(contents: &str) -> BTreeSet<String> {
    contents
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '`' | '(' | ')' | '[' | ']' | ',' | ';' | '"' | '\''
                )
        })
        .filter_map(|token| {
            let token =
                token.trim_matches(|character: char| matches!(character, '.' | ':' | '!' | '?'));
            if is_exact_openapi_path(token) {
                Some(token.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn openapi_location_references(location: &str) -> Vec<String> {
    location
        .split(|character: char| character.is_whitespace() || matches!(character, '+' | ','))
        .filter_map(|token| {
            let token = token.trim_matches(|character: char| {
                matches!(character, '`' | '(' | ')' | '[' | ']' | '"' | '\'')
            });
            if token.starts_with("contracts/")
                && (token.contains(".yaml") || token.contains(".yml"))
            {
                Some(token.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn is_openapi_glob_reference(path: &str) -> bool {
    path.contains('*') || path.contains('<') || path.contains('>')
}

fn is_exact_openapi_path(path: &str) -> bool {
    path.starts_with(OPENAPI_PREFIX)
        && (path.ends_with(".yaml") || path.ends_with(".yml"))
        && !is_openapi_glob_reference(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_versioned_openapi_32_document_with_operation_responses() {
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                VALID
            )]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 33,
            })
        );
    }

    #[test]
    fn accepts_openapi_32_range_and_default_response_keys_as_source_shape() {
        let with_range = VALID.replace("        '202':", "        '2XX':");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &with_range
            )]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 33,
            })
        );

        let with_default = VALID.replace("        '403':", "        default:");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &with_default
            )]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 33,
            })
        );
    }

    #[test]
    fn accepts_openapi_32_query_fixed_operation() {
        let query = query_operation_document();
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/query-v1.yaml",
                &query,
            )]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 1,
            })
        );
    }

    #[test]
    fn rejects_duplicate_operation_ids_during_source_validation() {
        let duplicate = query_operation_document().replace(
            "components:\n",
            "  /v1/capability-queries/duplicate:\n    query:\n      operationId: queryCapability\n      responses:\n        '200':\n          description: Query completed.\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/QueryResponse'\ncomponents:\n",
        );

        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/query-v1.yaml",
                &duplicate,
            )]),
            Err(OpenApiSourceError::DuplicateOperationId {
                operation_id: "queryCapability".into(),
                first_path: "contracts/openapi/foundry/query-v1.yaml".into(),
                second_path: "contracts/openapi/foundry/query-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn accepts_openapi_32_additional_operations_custom_method() {
        let copy = copy_operation_document();
        assert_eq!(
            validate_openapi_documents([
                document("contracts/openapi/foundry/copy-v1.yaml", &copy,)
            ]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 5,
            })
        );
    }

    #[test]
    fn accepts_path_item_parameters_for_template_and_mutating_ingress_checks() {
        let inherited_parameters = r#"openapi: 3.2.0
info:
  title: Oyatie Copy API
  version: 1.0.0
paths:
  /v1/resources/{resource_id}:
    parameters:
      - name: resource_id
        in: path
        required: true
        schema:
          type: string
        x-oyatie-data-class: INTERNAL_ONLY
      - name: X-Request-Id
        in: header
        required: true
        schema:
          type: string
          minLength: 1
        x-oyatie-rust-type: String
        x-oyatie-data-class: INTERNAL_ONLY
      - name: X-Tenant-Id
        in: header
        required: true
        schema:
          type: string
          minLength: 1
        x-oyatie-rust-type: String
        x-oyatie-data-class: INTERNAL_ONLY
      - name: Idempotency-Key
        in: header
        required: true
        schema:
          type: string
          minLength: 1
        x-oyatie-rust-type: String
        x-oyatie-data-class: INTERNAL_ONLY
    additionalOperations:
      COPY:
        operationId: copyResource
        security:
          - bearerAuth: []
        responses:
          '200':
            description: Copy completed.
            content:
              application/json:
                schema:
                  $ref: '#/components/schemas/CopyResponse'
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
  schemas:
    CopyResponse:
      type: object
      required:
        - data
      properties:
        data:
          type: string
          x-oyatie-rust-type: String
          x-oyatie-data-class: INTERNAL_ONLY
"#;

        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/copy-v1.yaml",
                inherited_parameters,
            )]),
            Ok(OpenApiSourceReport {
                documents_checked: 1,
                operations_checked: 1,
                data_class_annotations_checked: 5,
            })
        );
    }

    #[test]
    fn rejects_openapi_32_additional_operation_without_mutating_ingress_controls() {
        let missing_security = copy_operation_document()
            .replace("        security:\n          - bearerAuth: []\n", "");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/copy-v1.yaml",
                &missing_security,
            )]),
            Err(OpenApiSourceError::MissingOperationSecurity {
                path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                api_path: "/v1/resources/{resource_id}".into(),
                method: "COPY".into(),
                scheme: "bearerAuth".into(),
            })
        );

        let missing_idempotency = copy_operation_document().replace(
            "          - name: Idempotency-Key\n            in: header\n            required: true\n            schema:\n              type: string\n              minLength: 1\n            x-oyatie-rust-type: String\n            x-oyatie-data-class: INTERNAL_ONLY\n",
            "",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/copy-v1.yaml",
                &missing_idempotency,
            )]),
            Err(OpenApiSourceError::MissingRequiredHeaderParameter {
                path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                api_path: "/v1/resources/{resource_id}".into(),
                method: "COPY".into(),
                header: "Idempotency-Key".into(),
            })
        );
    }

    #[test]
    fn rejects_openapi_32_additional_operations_fixed_method_collisions() {
        for method in ["POST", "QUERY"] {
            let invalid =
                copy_operation_document().replace("      COPY:", &format!("      {method}:"));
            assert_eq!(
                validate_openapi_documents([document(
                    "contracts/openapi/foundry/copy-v1.yaml",
                    &invalid,
                )]),
                Err(
                    OpenApiSourceError::AdditionalOperationFixedMethodCollision {
                        path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                        api_path: "/v1/resources/{resource_id}".into(),
                        method: method.into(),
                    }
                )
            );
        }
    }

    #[test]
    fn rejects_openapi_32_additional_operation_missing_operation_id_or_responses() {
        let missing_operation_id =
            copy_operation_document().replace("        operationId: copyResource\n", "");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/copy-v1.yaml",
                &missing_operation_id,
            )]),
            Err(OpenApiSourceError::MissingOperationId {
                path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                api_path: "/v1/resources/{resource_id}".into(),
                method: "COPY".into(),
            })
        );

        let missing_responses = copy_operation_document().replace(
            "        responses:\n          '200':\n            description: Copy completed.\n            content:\n              application/json:\n                schema:\n                  $ref: '#/components/schemas/CopyResponse'\n",
            "        responses:\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/copy-v1.yaml",
                &missing_responses,
            )]),
            Err(OpenApiSourceError::MissingResponses {
                path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                api_path: "/v1/resources/{resource_id}".into(),
                method: "COPY".into(),
            })
        );
    }

    #[test]
    fn runtime_parity_collects_query_and_additional_operations_as_bindable_operations() {
        let query = query_operation_document();
        assert_eq!(
            validate_openapi_runtime_parity(
                [document("contracts/openapi/foundry/query-v1.yaml", &query)],
                [],
                [],
                [],
            ),
            Err(OpenApiSourceError::MissingRuntimeBinding {
                operation_id: "queryCapability".into(),
                contract_path: "contracts/openapi/foundry/query-v1.yaml".into(),
            })
        );

        let copy = copy_operation_document();
        assert_eq!(
            validate_openapi_runtime_parity(
                [document("contracts/openapi/foundry/copy-v1.yaml", &copy)],
                [],
                [],
                [],
            ),
            Err(OpenApiSourceError::MissingRuntimeBinding {
                operation_id: "copyResource".into(),
                contract_path: "contracts/openapi/foundry/copy-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn additional_operation_runtime_parity_keeps_response_key_and_schema_guards() {
        let with_range = copy_operation_document().replace("          '200':", "          '2XX':");
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/copy-v1.yaml",
                    &with_range
                )],
                [copy_runtime_binding("CopyResponse")],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    COPY_RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/copy_api.rs",
                    COPY_RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::NonExplicitRuntimeResponseKey {
                operation_id: "copyResource".into(),
                contract_path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                response_key: "2XX".into(),
            })
        );

        let missing_schema = copy_operation_document().replace(
            "            content:\n              application/json:\n                schema:\n                  $ref: '#/components/schemas/CopyResponse'\n",
            "",
        );
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/copy-v1.yaml",
                    &missing_schema
                )],
                [copy_runtime_binding("CopyResponse")],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    COPY_RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/copy_api.rs",
                    COPY_RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeResponseSchema {
                operation_id: "copyResource".into(),
                contract_path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                status: "200".into(),
            })
        );

        let copy = copy_operation_document();
        assert_eq!(
            validate_openapi_runtime_parity(
                [document("contracts/openapi/foundry/copy-v1.yaml", &copy)],
                [copy_runtime_binding("OtherResponse")],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    COPY_RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/copy_api.rs",
                    COPY_RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::RuntimeResponseSchemaMismatch {
                operation_id: "copyResource".into(),
                contract_path: "contracts/openapi/foundry/copy-v1.yaml".into(),
                status: "200".into(),
                expected_schema: "OtherResponse".into(),
                actual_schema: "CopyResponse".into(),
            })
        );
    }

    #[test]
    fn rejects_openapi_31_contracts_after_32_pivot() {
        let invalid = VALID.replace("openapi: 3.2.0", "openapi: 3.1.0");

        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid
            )]),
            Err(OpenApiSourceError::UnsupportedOpenApiVersion {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                version: "3.1.0".into(),
            })
        );
    }

    #[test]
    fn rejects_empty_or_non_contract_documents() {
        assert_eq!(
            validate_openapi_documents([]),
            Err(OpenApiSourceError::NoDocuments)
        );
        assert_eq!(
            validate_openapi_documents([document("docs/openapi.yaml", VALID)]),
            Err(OpenApiSourceError::InvalidPath {
                path: "docs/openapi.yaml".into(),
                reason: "OpenAPI documents must live under contracts/openapi/".into(),
            })
        );
    }

    #[test]
    fn rejects_version_drift_between_filename_and_info_version() {
        let invalid = VALID.replace("version: 1.0.0", "version: 2.0.0");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::VersionSuffixMismatch {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                path_major: 1,
                info_major: 2,
            })
        );
    }

    #[test]
    fn rejects_missing_operation_id() {
        let invalid = VALID.replace("      operationId: invokeCapability\n", "");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingOperationId {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
            })
        );
    }

    #[test]
    fn rejects_operations_without_response_statuses() {
        let invalid = replace_response_block("      responses:\n");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingResponses {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
            })
        );
    }

    #[test]
    fn rejects_lowercase_openapi_response_range_keys() {
        let invalid = replace_response_block(
            "      responses:\n        '2xx':\n          description: Lowercase response range is not an OpenAPI 3.2 response key.\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingResponses {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
            })
        );
    }

    #[test]
    fn rejects_missing_bearer_security_scheme_for_mutating_operation() {
        let invalid = VALID.replace(
            "  securitySchemes:\n    bearerAuth:\n      type: http\n      scheme: bearer\n      bearerFormat: STS\n",
            "",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingBearerSecurityScheme {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn rejects_missing_operation_bearer_security_for_mutating_operation() {
        let invalid = VALID.replace("      security:\n        - bearerAuth: []\n", "");
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingOperationSecurity {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
                scheme: "bearerAuth".into(),
            })
        );
    }

    #[test]
    fn rejects_authorization_header_parameter_because_bearer_auth_uses_security_scheme() {
        let invalid = VALID.replace(
            "      parameters:\n",
            "      parameters:\n        - name: Authorization\n          in: header\n          required: true\n          schema:\n            type: string\n          x-oyatie-data-class: SECRET\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::ForbiddenAuthorizationParameter {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
            })
        );
    }

    #[test]
    fn rejects_mutating_operation_missing_required_ingress_header() {
        let invalid = VALID.replace(
            "        - name: Idempotency-Key\n          in: header\n          required: true\n          schema:\n            type: string\n            minLength: 1\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingRequiredHeaderParameter {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
                header: "Idempotency-Key".into(),
            })
        );
    }

    #[test]
    fn rejects_mutating_operation_ingress_header_without_min_length() {
        let invalid = VALID.replace(
            "            minLength: 1\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n        - name: X-Tenant-Id\n",
            "          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n        - name: X-Tenant-Id\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::InvalidHeaderParameter {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
                header: "X-Request-Id".into(),
                reason: "header parameter schema must declare minLength: 1".into(),
            })
        );
    }

    #[test]
    fn rejects_path_template_parameter_without_explicit_operation_parameter() {
        let missing = VALID.replace(
            "        - name: capability_id\n          in: path\n          required: true\n          schema:\n            type: string\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &missing,
            )]),
            Err(OpenApiSourceError::MissingPathTemplateParameter {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
                parameter: "capability_id".into(),
            })
        );

        let optional = VALID.replace(
            "        - name: capability_id\n          in: path\n          required: true\n          schema:\n            type: string\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        - name: capability_id\n          in: path\n          required: false\n          schema:\n            type: string\n          x-oyatie-data-class: INTERNAL_ONLY\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &optional,
            )]),
            Err(OpenApiSourceError::InvalidPathTemplateParameter {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                api_path: "/v1/capabilities/{capability_id}/invoke".into(),
                method: "post".into(),
                parameter: "capability_id".into(),
                reason: "path template parameter must be required: true".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_property_without_data_class_annotation() {
        let invalid = VALID.replace(
            "        tenant_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        tenant_id:\n          type: string\n          x-oyatie-rust-type: String\n",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::MissingDataClassAnnotation {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                location: "schema CapabilityInvocationRequest.tenant_id".into(),
            })
        );
    }

    #[test]
    fn rejects_parameter_with_invalid_data_class_annotation() {
        let invalid = VALID.replace(
            "          x-oyatie-data-class: INTERNAL_ONLY\n      requestBody:",
            "          x-oyatie-data-class: UNKNOWN\n      requestBody:",
        );
        assert_eq!(
            validate_openapi_documents([document(
                "contracts/openapi/foundry/capability-v1.yaml",
                &invalid,
            )]),
            Err(OpenApiSourceError::InvalidDataClassAnnotation {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                location: "parameter Idempotency-Key".into(),
                data_class: "UNKNOWN".into(),
            })
        );
    }

    #[test]
    fn accepts_contract_mirror_when_spec_and_machine_reference_source() {
        assert_eq!(
            validate_openapi_contract_mirror(
                ["contracts/openapi/foundry/capability-v1.yaml"],
                "Reference: contracts/openapi/foundry/capability-v1.yaml",
                [mirror_location(
                    "CAPABILITY_INVOCATION",
                    "crates/oya-foundation-app + contracts/openapi/foundry/capability-v1.yaml",
                )],
            ),
            Ok(OpenApiContractMirrorReport {
                contracts_checked: 1,
                spec_references_checked: 1,
                mirror_references_checked: 1,
            })
        );
    }

    #[test]
    fn rejects_contract_mirror_missing_spec_or_machine_reference() {
        assert_eq!(
            validate_openapi_contract_mirror(
                ["contracts/openapi/foundry/capability-v1.yaml"],
                "No exact contract path here.",
                [mirror_location(
                    "CAPABILITY_INVOCATION",
                    "contracts/openapi/foundry/capability-v1.yaml",
                )],
            ),
            Err(OpenApiSourceError::MissingSpecMirror {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            })
        );

        assert_eq!(
            validate_openapi_contract_mirror(
                ["contracts/openapi/foundry/capability-v1.yaml"],
                "contracts/openapi/foundry/capability-v1.yaml",
                [mirror_location(
                    "CAPABILITY_INVOCATION",
                    "crates/oya-foundation-app"
                )],
            ),
            Err(OpenApiSourceError::MissingMachineMirror {
                path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn rejects_stale_exact_spec_or_machine_contract_paths() {
        assert_eq!(
            validate_openapi_contract_mirror(
                ["contracts/openapi/foundry/capability-v1.yaml"],
                "contracts/openapi/foundry/capability-v1.yaml contracts/openapi/foundry/missing-v1.yaml",
                [mirror_location(
                    "CAPABILITY_INVOCATION",
                    "contracts/openapi/foundry/capability-v1.yaml",
                )],
            ),
            Err(OpenApiSourceError::StaleSpecMirror {
                path: "contracts/openapi/foundry/missing-v1.yaml".into(),
            })
        );

        assert_eq!(
            validate_openapi_contract_mirror(
                ["contracts/openapi/foundry/capability-v1.yaml"],
                "contracts/openapi/foundry/capability-v1.yaml",
                [
                    mirror_location(
                        "CAPABILITY_INVOCATION",
                        "contracts/openapi/foundry/capability-v1.yaml",
                    ),
                    mirror_location("STALE", "contracts/openapi/foundry/missing-v1.yaml"),
                ],
            ),
            Err(OpenApiSourceError::StaleMachineMirror {
                contract_id: "STALE".into(),
                path: "contracts/openapi/foundry/missing-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn accepts_openapi_runtime_binding_when_source_and_test_cover_operation() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Ok(OpenApiRuntimeParityReport {
                operations_checked: 1,
                bindings_checked: 1,
                sources_checked: 1,
                tests_checked: 1,
                response_statuses_checked: 3,
                response_schemas_checked: 3,
            })
        );
    }

    #[test]
    fn accepts_openapi_runtime_binding_at_canonical_capability_face_paths() {
        // ADR-0562 canonical capability-face layout: `<capability>/<face>/<crate>`
        // with the package named `<capability>-<crate>` (intelligence/core/api
        // hosts intelligence-api). The shape validator must accept these paths
        // just like the legacy `crates/<crate>/` layout.
        let canonical = OpenApiRuntimeBinding {
            runtime_crate: "intelligence-api".into(),
            source_path: "intelligence/core/api/src/lib.rs".into(),
            test_path: "intelligence/core/api/tests/capability_invoke_api.rs".into(),
            ..runtime_binding()
        };
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [canonical],
                [runtime_source(
                    "intelligence/core/api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "intelligence/core/api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Ok(OpenApiRuntimeParityReport {
                operations_checked: 1,
                bindings_checked: 1,
                sources_checked: 1,
                tests_checked: 1,
                response_statuses_checked: 3,
                response_schemas_checked: 3,
            })
        );
    }

    #[test]
    fn rejects_non_explicit_openapi_responses_for_typed_runtime_parity() {
        let with_range = VALID.replace("        '202':", "        '2XX':");
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &with_range,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::NonExplicitRuntimeResponseKey {
                operation_id: "invokeCapability".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                response_key: "2XX".into(),
            })
        );

        let with_default = VALID.replace("        '403':", "        default:");
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &with_default,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::NonExplicitRuntimeResponseKey {
                operation_id: "invokeCapability".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                response_key: "default".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_response_status_without_schema_ref() {
        let invalid = VALID.replacen(
            "          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/CapabilityInvokeApiErrorResponse'\n",
            "",
            1,
        );
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeResponseSchema {
                operation_id: "invokeCapability".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                status: "400".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_response_status_with_unexpected_schema_ref() {
        let invalid = VALID.replace(
            "                $ref: '#/components/schemas/CapabilityInvokeApiSuccessResponse'\n",
            "                $ref: '#/components/schemas/CapabilityInvokeApiErrorResponse'\n",
        );
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::RuntimeResponseSchemaMismatch {
                operation_id: "invokeCapability".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                status: "202".into(),
                expected_schema: "CapabilityInvokeApiSuccessResponse".into(),
                actual_schema: "CapabilityInvokeApiErrorResponse".into(),
            })
        );
    }

    #[test]
    fn rejects_openapi_operation_without_runtime_binding() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [],
                [],
                [],
            ),
            Err(OpenApiSourceError::MissingRuntimeBinding {
                operation_id: "invokeCapability".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_without_source_symbol_or_test_coverage() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "pub fn other_symbol() { let _surface = \"foundry.capability.invoke\"; let _status = 202 + 400 + 403; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeSymbol {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                symbol: "invoke_capability_from_api".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_when_evidence_surface_is_not_public_constant() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeEvidenceSurface {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                evidence_surface: "foundry.capability.invoke".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_when_source_or_test_only_contain_substring_markers() {
        let shadow_source = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
        pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
        impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
        pub fn invoke_capability_from_api_shadow() { let _surface = CAPABILITY_INVOKE_SURFACE; }";

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    shadow_source,
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeSymbol {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                symbol: "invoke_capability_from_api".into(),
            })
        );

        let shadow_test = "invoke_capability_from_api_shadow(foundation, request);\n\
            assert_eq!(CAPABILITY_INVOKE_SURFACE_SHADOW, \"foundry.capability.invoke.shadow\");\n\
            assert_eq!(CapabilityInvokeApiStatusShadow::Accepted.code(), 202);\n\
            assert_eq!(CapabilityInvokeApiStatusShadow::BadRequest.code(), 400);\n\
            assert_eq!(CapabilityInvokeApiStatusShadow::Forbidden.code(), 403);";

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    shadow_test,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeTestCoverage {
                operation_id: "invokeCapability".into(),
                test_path: "crates/oya-intelligence-api/tests/capability_invoke_api.rs".into(),
                symbol: "invoke_capability_from_api".into(),
                evidence_surface: "foundry.capability.invoke".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_when_symbol_is_not_public_api_function() {
        let private_source = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
        pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
        impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
        fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    private_source,
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeSymbol {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                symbol: "invoke_capability_from_api".into(),
            })
        );

        let restricted_source = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
        pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
        impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
        pub(crate) fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    restricted_source,
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeSymbol {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                symbol: "invoke_capability_from_api".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_status_test_coverage_without_assertion_shape() {
        let weak_status_test = r#"
    invoke_capability_from_api(foundation, request);
    assert_eq!(CAPABILITY_INVOKE_SURFACE, "foundry.capability.invoke");
    let _accepted = (CapabilityInvokeApiStatus, 202);
    let _bad_request = (CapabilityInvokeApiStatus, 400);
    let _forbidden = (CapabilityInvokeApiStatus, 403);
    "#;

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    weak_status_test,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeTestResponseStatus {
                operation_id: "invokeCapability".into(),
                test_path: "crates/oya-intelligence-api/tests/capability_invoke_api.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
                status: "202".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_status_test_coverage_with_symbolic_status_constants() {
        let symbolic_status_test = r#"
    invoke_capability_from_api(foundation, request);
    assert_eq!(CAPABILITY_INVOKE_SURFACE, "foundry.capability.invoke");
    assert_eq!(CapabilityInvokeApiStatus::Accepted.code(), HTTP_202_ACCEPTED);
    assert_eq!(CapabilityInvokeApiStatus::BadRequest.code(), HTTP_400_BAD_REQUEST);
    assert_eq!(CapabilityInvokeApiStatus::Forbidden.code(), HTTP_403_FORBIDDEN);
    "#;

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    symbolic_status_test,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeTestResponseStatus {
                operation_id: "invokeCapability".into(),
                test_path: "crates/oya-intelligence-api/tests/capability_invoke_api.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
                status: "202".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_without_response_status_coverage() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeResponseStatus {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
                status: "403".into(),
            })
        );

        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    RUNTIME_API
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    "invoke_capability_from_api(foundation, request); assert_eq!(surface, \"foundry.capability.invoke\"); assert_eq!(CapabilityInvokeApiStatus::Accepted.code(), 202); assert_eq!(CapabilityInvokeApiStatus::BadRequest.code(), 400);",
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeTestResponseStatus {
                operation_id: "invokeCapability".into(),
                test_path: "crates/oya-intelligence-api/tests/capability_invoke_api.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
                status: "403".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_without_typed_status_source() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub fn invoke_capability_from_api() { let _accepted = 202; let _bad_request = 400; let _forbidden = 403; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeStatusType {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_binding_when_status_type_is_not_public_api_enum() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::MissingRuntimeStatusType {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
            })
        );
    }

    #[test]
    fn rejects_runtime_status_codes_not_documented_by_openapi() {
        assert_eq!(
            validate_openapi_runtime_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [runtime_binding()],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
	pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden, NotFound }\n\
	impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403, Self::NotFound => 404 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }",
                )],
                [runtime_source(
                    "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                    RUNTIME_API_TEST,
                )],
            ),
            Err(OpenApiSourceError::UndocumentedRuntimeResponseStatus {
                operation_id: "invokeCapability".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                status_type: "CapabilityInvokeApiStatus".into(),
                status: "404".into(),
            })
        );
    }

    #[test]
    fn rejects_invalid_runtime_status_type_mappings() {
        let undeclared_variant = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Teapot => 418 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";
        assert_eq!(
            invalid_runtime_status_type_reason(undeclared_variant),
            "CapabilityInvokeApiStatus maps undeclared variant Teapot"
        );

        let missing_variant = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden, NotFound }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";
        assert_eq!(
            invalid_runtime_status_type_reason(missing_variant),
            "CapabilityInvokeApiStatus code mappings do not cover enum variants: missing=[\"NotFound\"], extra=[]"
        );

        let wildcard_arm = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, _ => 403 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";
        assert_eq!(
            invalid_runtime_status_type_reason(wildcard_arm),
            "status code mappings must use explicit Self::Variant arms"
        );

        let non_literal_status = "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => FORBIDDEN } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }";
        assert_eq!(
            invalid_runtime_status_type_reason(non_literal_status),
            "status code mappings must return explicit three-digit numeric literals"
        );
    }

    #[test]
    fn accepts_schema_parity_when_openapi_fields_match_runtime_structs() {
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Ok(OpenApiSchemaParityReport {
                schemas_checked: 7,
                bindings_checked: 7,
                sources_checked: 2,
                fields_checked: 29,
                types_checked: 29,
            })
        );
    }

    #[test]
    fn accepts_schema_parity_for_vec_string_scalar_items() {
        let contract = r#"openapi: 3.2.0
info:
  title: Oyatie Tags API
  version: 1.0.0
paths:
  /v1/tags:
    get:
      operationId: listTags
      security:
        - bearerAuth: []
      responses:
        '200':
          description: Tags listed.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TagResponse'
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
  schemas:
    TagResponse:
      type: object
      required:
        - tags
      properties:
        tags:
          type: array
          items:
            type: string
          x-oyatie-rust-type: Vec<String>
          x-oyatie-data-class: INTERNAL_ONLY
"#;
        let runtime = r#"
pub struct TagResponse {
    pub tags: Vec<String>, // data_class: INTERNAL_ONLY
}
"#;

        assert_eq!(
            validate_openapi_schema_parity(
                [document("contracts/openapi/foundry/tags-v1.yaml", contract)],
                [OpenApiSchemaBinding {
                    schema_name: "TagResponse".into(),
                    contract_path: "contracts/openapi/foundry/tags-v1.yaml".into(),
                    runtime_crate: "oya-intelligence-api".into(),
                    source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                    rust_struct: "TagResponse".into(),
                }],
                [runtime_source(
                    "crates/oya-intelligence-api/src/lib.rs",
                    runtime
                )],
            ),
            Ok(OpenApiSchemaParityReport {
                schemas_checked: 1,
                bindings_checked: 1,
                sources_checked: 1,
                fields_checked: 1,
                types_checked: 1,
            })
        );
    }

    #[test]
    fn rejects_component_schema_without_runtime_shape_binding() {
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                [],
                [],
            ),
            Err(OpenApiSourceError::MissingSchemaBinding {
                schema_name: "CapabilityInvocationReceipt".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_properties_that_drift_from_runtime_struct_fields() {
        let invalid = VALID.replacen(
            "        tenant_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n        user_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n        capability_id:",
            "        tenant_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n        capability_id:",
            1,
        );
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaFieldMismatch {
                schema_name: "CapabilityInvocationRequest".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_properties: vec!["user_id".into()],
                extra_properties: vec![],
            })
        );
    }

    #[test]
    fn rejects_schema_binding_when_runtime_struct_is_not_public_api_type() {
        let private_success_response = RUNTIME_API.replacen(
            "pub struct CapabilityInvokeApiSuccessResponse",
            "struct CapabilityInvokeApiSuccessResponse",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source(
                        "crates/oya-intelligence-api/src/lib.rs",
                        &private_success_response,
                    ),
                ],
            ),
            Err(OpenApiSourceError::MissingRuntimeStruct {
                schema_name: "CapabilityInvokeApiSuccessResponse".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiSuccessResponse".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_types_that_drift_from_runtime_struct_fields() {
        let invalid = VALID.replacen(
            "        projected_cost_micros:\n          type: integer\n          format: uint64\n          minimum: 0\n          x-oyatie-rust-type: u64\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        projected_cost_micros:\n          type: integer\n          format: int64\n          minimum: 0\n          x-oyatie-rust-type: u64\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            1,
        );
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvocationRequest".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec![
                    "projected_cost_micros: expected type integer format uint64 for Rust u64, found type integer format int64"
                        .into()
                ],
            })
        );
    }

    #[test]
    fn rejects_schema_array_items_with_only_nested_spoofed_ref() {
        let invalid = VALID.replace(
            "        details:\n          type: array\n          items:\n            $ref: '#/components/schemas/CapabilityInvokeApiErrorDetail'\n          x-oyatie-rust-type: Vec<CapabilityInvokeApiErrorDetail>\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        details:\n          type: array\n          items:\n            type: object\n            properties:\n              spoof:\n                $ref: '#/components/schemas/CapabilityInvokeApiErrorDetail'\n                x-oyatie-data-class: INTERNAL_ONLY\n          x-oyatie-rust-type: Vec<CapabilityInvokeApiErrorDetail>\n          x-oyatie-data-class: INTERNAL_ONLY\n",
        );
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvokeApiErrorBody".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec![
                    "details: missing items $ref, expected CapabilityInvokeApiErrorDetail for Rust Vec<CapabilityInvokeApiErrorDetail>"
                        .into()
                ],
            })
        );
    }

    #[test]
    fn rejects_schema_ref_property_with_conflicting_scalar_type() {
        let invalid = VALID.replace(
            "        data:\n          $ref: '#/components/schemas/CapabilityInvocationReceipt'\n          x-oyatie-rust-type: CapabilityInvocationReceipt\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        data:\n          type: string\n          $ref: '#/components/schemas/CapabilityInvocationReceipt'\n          x-oyatie-rust-type: CapabilityInvocationReceipt\n          x-oyatie-data-class: INTERNAL_ONLY\n",
        );
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvokeApiSuccessResponse".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec![
                    "data: $ref property for Rust CapabilityInvocationReceipt must not declare type string"
                        .into()
                ],
            })
        );
    }

    #[test]
    fn rejects_schema_nullable_property_when_runtime_field_is_not_option() {
        let invalid = VALID.replacen(
            "        request_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            "        request_id:\n          type: string\n          nullable: true\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec![
                    "request_id: OpenAPI nullable true requires Rust Option<...>, found String"
                        .into(),
                ],
            })
        );
    }

    #[test]
    fn rejects_schema_property_with_unsupported_composition_keyword() {
        for (keyword, unsupported_block) in [
            ("allOf", "          allOf:\n            - type: string\n"),
            ("anyOf", "          anyOf:\n            - type: string\n"),
            ("oneOf", "          oneOf:\n            - type: string\n"),
            ("not", "          not:\n            type: integer\n"),
            (
                "additionalProperties",
                "          additionalProperties: false\n",
            ),
        ] {
            let invalid = VALID.replacen(
                "        request_id:\n          type: string\n          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n",
                &format!(
                    "        request_id:\n          type: string\n{unsupported_block}          x-oyatie-rust-type: String\n          x-oyatie-data-class: INTERNAL_ONLY\n"
                ),
                1,
            );

            assert_eq!(
                validate_openapi_schema_parity(
                    [document(
                        "contracts/openapi/foundry/capability-v1.yaml",
                        &invalid,
                    )],
                    schema_bindings(),
                    schema_runtime_sources(),
                ),
                Err(OpenApiSourceError::SchemaTypeMismatch {
                    schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                    contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                    mismatches: vec![format!(
                        "request_id: unsupported OpenAPI schema keyword {keyword}"
                    )],
                }),
                "keyword {keyword} must fail closed"
            );
        }
    }

    #[test]
    fn rejects_component_schema_with_unsupported_composition_keyword() {
        let invalid = VALID.replacen(
            "    CapabilityInvokeApiResponseMetadata:\n      type: object\n",
            "    CapabilityInvokeApiResponseMetadata:\n      type: object\n      additionalProperties: true\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec![
                    "CapabilityInvokeApiResponseMetadata: unsupported OpenAPI schema keyword additionalProperties"
                        .into(),
                ],
            })
        );
    }

    #[test]
    fn rejects_schema_property_when_runtime_field_has_serde_rename_drift() {
        let renamed_runtime = RUNTIME_API.replacen(
            "    pub data: CapabilityInvocationReceipt, // data_class: INTERNAL_ONLY\n",
            "    #[serde(rename = \"payload\")]\n    pub data: CapabilityInvocationReceipt, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &renamed_runtime),
                ],
            ),
            Err(OpenApiSourceError::SchemaFieldMismatch {
                schema_name: "CapabilityInvokeApiSuccessResponse".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_properties: vec!["payload".into()],
                extra_properties: vec!["data".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_property_when_runtime_struct_has_serde_rename_all_drift() {
        let renamed_runtime = RUNTIME_API.replacen(
            "pub struct CapabilityInvokeApiResponseMetadata",
            "#[serde(rename_all = \"camelCase\")]\npub struct CapabilityInvokeApiResponseMetadata",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &renamed_runtime),
                ],
            ),
            Err(OpenApiSourceError::SchemaFieldMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_properties: vec!["requestId".into()],
                extra_properties: vec!["request_id".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_binding_with_unsupported_serde_rename_all_rule() {
        let renamed_runtime = RUNTIME_API.replacen(
            "pub struct CapabilityInvokeApiResponseMetadata",
            "#[serde(rename_all = \"Train-Case\")]\npub struct CapabilityInvokeApiResponseMetadata",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &renamed_runtime),
                ],
            ),
            Err(OpenApiSourceError::InvalidRuntimeStruct {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiResponseMetadata".into(),
                reason: "unsupported serde rename_all rule Train-Case".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_binding_with_unmodeled_serde_struct_clauses() {
        for (attribute, unsupported) in [
            ("#[serde(transparent)]", "transparent"),
            ("#[serde(untagged)]", "untagged"),
            ("#[serde(tag = \"kind\")]", "tag"),
            ("#[serde(deny_unknown_fields)]", "deny_unknown_fields"),
            (
                "#[serde(rename_all(serialize = \"camelCase\"))]",
                "rename_all",
            ),
        ] {
            let runtime = RUNTIME_API.replacen(
                "pub struct CapabilityInvokeApiResponseMetadata",
                &format!("{attribute}\npub struct CapabilityInvokeApiResponseMetadata"),
                1,
            );

            assert_eq!(
                validate_openapi_schema_parity(
                    [document(
                        "contracts/openapi/foundry/capability-v1.yaml",
                        VALID,
                    )],
                    schema_bindings(),
                    [
                        runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                        runtime_source("crates/oya-intelligence-api/src/lib.rs", &runtime),
                    ],
                ),
                Err(OpenApiSourceError::InvalidRuntimeStruct {
                    schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                    path: "crates/oya-intelligence-api/src/lib.rs".into(),
                    rust_struct: "CapabilityInvokeApiResponseMetadata".into(),
                    reason: format!(
                        "unsupported serde {unsupported} on struct CapabilityInvokeApiResponseMetadata"
                    ),
                }),
                "attribute {attribute} must fail closed"
            );
        }
    }

    #[test]
    fn rejects_schema_property_when_runtime_field_is_serde_skipped() {
        let skipped_runtime = RUNTIME_API.replacen(
            "    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            "    #[serde(skip)]\n    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &skipped_runtime),
                ],
            ),
            Err(OpenApiSourceError::SchemaFieldMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_properties: vec![],
                extra_properties: vec!["request_id".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_binding_with_unsupported_serde_flatten_field() {
        let flattened_runtime = RUNTIME_API.replacen(
            "    pub metadata: CapabilityInvokeApiResponseMetadata, // data_class: INTERNAL_ONLY\n",
            "    #[serde(flatten)]\n    pub metadata: CapabilityInvokeApiResponseMetadata, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &flattened_runtime),
                ],
            ),
            Err(OpenApiSourceError::InvalidRuntimeStruct {
                schema_name: "CapabilityInvokeApiSuccessResponse".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiSuccessResponse".into(),
                reason: "unsupported serde flatten on field metadata".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_required_when_runtime_field_is_conditionally_serialized() {
        let conditional_runtime = RUNTIME_API.replacen(
            "    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            "    #[serde(skip_serializing_if = \"String::is_empty\")]\n    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source(
                        "crates/oya-intelligence-api/src/lib.rs",
                        &conditional_runtime
                    ),
                ],
            ),
            Err(OpenApiSourceError::SchemaRequiredMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_required: vec![],
                extra_required: vec!["request_id".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_required_when_runtime_field_has_serde_default() {
        let defaulted_runtime = RUNTIME_API.replacen(
            "    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            "    #[serde(default)]\n    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &defaulted_runtime),
                ],
            ),
            Err(OpenApiSourceError::SchemaRequiredMismatch {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_required: vec![],
                extra_required: vec!["request_id".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_binding_with_unsupported_serde_alias_field() {
        let aliased_runtime = RUNTIME_API.replacen(
            "    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            "    #[serde(alias = \"requestId\")]\n    pub request_id: String, // data_class: INTERNAL_ONLY\n",
            1,
        );

        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    VALID,
                )],
                schema_bindings(),
                [
                    runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                    runtime_source("crates/oya-intelligence-api/src/lib.rs", &aliased_runtime),
                ],
            ),
            Err(OpenApiSourceError::InvalidRuntimeStruct {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiResponseMetadata".into(),
                reason: "unsupported serde alias requestId on field request_id".into(),
            })
        );
    }

    #[test]
    fn rejects_schema_binding_with_unmodeled_serde_field_clauses() {
        for (attribute, unsupported) in [
            (
                "#[serde(serialize_with = \"serialize_request_id\")]",
                "serialize_with",
            ),
            (
                "#[serde(deserialize_with = \"deserialize_request_id\")]",
                "deserialize_with",
            ),
            ("#[serde(with = \"request_id_codec\")]", "with"),
            ("#[serde(skip_deserializing)]", "skip_deserializing"),
            ("#[serde(rename(serialize = \"requestId\"))]", "rename"),
        ] {
            let serialized_runtime = RUNTIME_API.replacen(
                "    pub request_id: String, // data_class: INTERNAL_ONLY\n",
                &format!(
                    "    {attribute}\n    pub request_id: String, // data_class: INTERNAL_ONLY\n"
                ),
                1,
            );

            assert_eq!(
                validate_openapi_schema_parity(
                    [document(
                        "contracts/openapi/foundry/capability-v1.yaml",
                        VALID,
                    )],
                    schema_bindings(),
                    [
                        runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
                        runtime_source(
                            "crates/oya-intelligence-api/src/lib.rs",
                            &serialized_runtime
                        ),
                    ],
                ),
                Err(OpenApiSourceError::InvalidRuntimeStruct {
                    schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                    path: "crates/oya-intelligence-api/src/lib.rs".into(),
                    rust_struct: "CapabilityInvokeApiResponseMetadata".into(),
                    reason: format!("unsupported serde {unsupported} on field request_id"),
                }),
                "attribute {attribute} must fail closed"
            );
        }
    }

    #[test]
    fn rejects_schema_rust_type_annotations_that_drift_from_runtime_struct_fields() {
        let invalid = VALID.replacen("          x-oyatie-rust-type: Purpose\n", "", 1);
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaTypeMismatch {
                schema_name: "CapabilityInvocationRequest".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                mismatches: vec!["purpose: missing x-oyatie-rust-type, expected Purpose".into()],
            })
        );
    }

    #[test]
    fn rejects_schema_required_fields_that_drift_from_runtime_optionality() {
        let invalid = VALID.replace(
            "      required:\n        - tenant_id\n        - user_id\n        - capability_id\n        - evidence_event_hash\n",
            "      required:\n        - tenant_id\n        - user_id\n        - capability_id\n        - evidence_event_hash\n        - run_id\n",
        );
        assert_eq!(
            validate_openapi_schema_parity(
                [document(
                    "contracts/openapi/foundry/capability-v1.yaml",
                    &invalid,
                )],
                schema_bindings(),
                schema_runtime_sources(),
            ),
            Err(OpenApiSourceError::SchemaRequiredMismatch {
                schema_name: "CapabilityInvocationReceipt".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                missing_required: vec![],
                extra_required: vec!["run_id".into()],
            })
        );
    }

    fn document(path: &str, contents: &str) -> OpenApiDocument {
        OpenApiDocument {
            path: path.into(),
            contents: contents.into(),
        }
    }

    fn mirror_location(contract_id: &str, location: &str) -> OpenApiContractMirrorLocation {
        OpenApiContractMirrorLocation {
            contract_id: contract_id.into(),
            location: location.into(),
        }
    }

    fn runtime_binding() -> OpenApiRuntimeBinding {
        OpenApiRuntimeBinding {
            operation_id: "invokeCapability".into(),
            contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
            runtime_crate: "oya-intelligence-api".into(),
            source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
            symbol: "invoke_capability_from_api".into(),
            status_type: "CapabilityInvokeApiStatus".into(),
            evidence_surface: "foundry.capability.invoke".into(),
            test_path: "crates/oya-intelligence-api/tests/capability_invoke_api.rs".into(),
            response_schemas: response_schemas(),
        }
    }

    fn response_schemas() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("202".into(), "CapabilityInvokeApiSuccessResponse".into()),
            ("400".into(), "CapabilityInvokeApiErrorResponse".into()),
            ("403".into(), "CapabilityInvokeApiErrorResponse".into()),
        ])
    }

    fn copy_runtime_binding(response_schema: &str) -> OpenApiRuntimeBinding {
        OpenApiRuntimeBinding {
            operation_id: "copyResource".into(),
            contract_path: "contracts/openapi/foundry/copy-v1.yaml".into(),
            runtime_crate: "oya-intelligence-api".into(),
            source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
            symbol: "copy_resource_from_api".into(),
            status_type: "CopyApiStatus".into(),
            evidence_surface: "foundry.copy".into(),
            test_path: "crates/oya-intelligence-api/tests/copy_api.rs".into(),
            response_schemas: BTreeMap::from([("200".into(), response_schema.into())]),
        }
    }

    fn schema_bindings() -> Vec<OpenApiSchemaBinding> {
        vec![
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvocationRequest".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-foundation-app".into(),
                source_path: "crates/oya-foundation-app/src/lib.rs".into(),
                rust_struct: "CapabilityInvocationRequest".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvocationReceipt".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-foundation-app".into(),
                source_path: "crates/oya-foundation-app/src/lib.rs".into(),
                rust_struct: "InvocationReceipt".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvokeApiSuccessResponse".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-intelligence-api".into(),
                source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiSuccessResponse".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvokeApiResponseMetadata".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-intelligence-api".into(),
                source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiResponseMetadata".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvokeApiErrorResponse".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-intelligence-api".into(),
                source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiErrorResponse".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvokeApiErrorBody".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-intelligence-api".into(),
                source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiErrorBody".into(),
            },
            OpenApiSchemaBinding {
                schema_name: "CapabilityInvokeApiErrorDetail".into(),
                contract_path: "contracts/openapi/foundry/capability-v1.yaml".into(),
                runtime_crate: "oya-intelligence-api".into(),
                source_path: "crates/oya-intelligence-api/src/lib.rs".into(),
                rust_struct: "CapabilityInvokeApiErrorDetail".into(),
            },
        ]
    }

    fn runtime_source(path: &str, contents: &str) -> OpenApiRuntimeSource {
        OpenApiRuntimeSource {
            path: path.into(),
            contents: contents.into(),
        }
    }

    fn schema_runtime_sources() -> Vec<OpenApiRuntimeSource> {
        vec![
            runtime_source("crates/oya-foundation-app/src/lib.rs", RUNTIME_STRUCTS),
            runtime_source("crates/oya-intelligence-api/src/lib.rs", RUNTIME_API),
        ]
    }

    fn invalid_runtime_status_type_reason(source: &str) -> String {
        match validate_openapi_runtime_parity(
            [document(
                "contracts/openapi/foundry/capability-v1.yaml",
                VALID,
            )],
            [runtime_binding()],
            [runtime_source(
                "crates/oya-intelligence-api/src/lib.rs",
                source,
            )],
            [runtime_source(
                "crates/oya-intelligence-api/tests/capability_invoke_api.rs",
                RUNTIME_API_TEST,
            )],
        ) {
            Err(OpenApiSourceError::InvalidRuntimeStatusType { reason, .. }) => reason,
            other => panic!("expected InvalidRuntimeStatusType, got {other:?}"),
        }
    }

    fn replace_response_block(replacement: &str) -> String {
        let start = VALID
            .find("      responses:\n")
            .expect("fixture has responses block");
        let end = VALID
            .find("components:\n")
            .expect("fixture has components block");
        format!("{}{}{}", &VALID[..start], replacement, &VALID[end..])
    }

    const VALID: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/openapi/foundry/capability-v1.yaml"
    ));

    fn query_operation_document() -> String {
        r#"openapi: 3.2.0
info:
  title: Oyatie Query API
  version: 1.0.0
paths:
  /v1/capability-queries:
    query:
      operationId: queryCapability
      responses:
        '200':
          description: Query completed.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/QueryResponse'
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
  schemas:
    QueryResponse:
      type: object
      required:
        - data
      properties:
        data:
          type: string
          x-oyatie-rust-type: String
          x-oyatie-data-class: INTERNAL_ONLY
"#
        .into()
    }

    fn copy_operation_document() -> String {
        r#"openapi: 3.2.0
info:
  title: Oyatie Copy API
  version: 1.0.0
paths:
  /v1/resources/{resource_id}:
    additionalOperations:
      COPY:
        operationId: copyResource
        security:
          - bearerAuth: []
        parameters:
          - name: resource_id
            in: path
            required: true
            schema:
              type: string
            x-oyatie-data-class: INTERNAL_ONLY
          - name: X-Request-Id
            in: header
            required: true
            schema:
              type: string
              minLength: 1
            x-oyatie-rust-type: String
            x-oyatie-data-class: INTERNAL_ONLY
          - name: X-Tenant-Id
            in: header
            required: true
            schema:
              type: string
              minLength: 1
            x-oyatie-rust-type: String
            x-oyatie-data-class: INTERNAL_ONLY
          - name: Idempotency-Key
            in: header
            required: true
            schema:
              type: string
              minLength: 1
            x-oyatie-rust-type: String
            x-oyatie-data-class: INTERNAL_ONLY
        responses:
          '200':
            description: Copy completed.
            content:
              application/json:
                schema:
                  $ref: '#/components/schemas/CopyResponse'
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
  schemas:
    CopyResponse:
      type: object
      required:
        - data
      properties:
        data:
          type: string
          x-oyatie-rust-type: String
          x-oyatie-data-class: INTERNAL_ONLY
"#
        .into()
    }

    const RUNTIME_API: &str = r#"
pub const CAPABILITY_INVOKE_SURFACE: &str = "foundry.capability.invoke";

	pub enum CapabilityInvokeApiStatus {
	    Accepted,
	    BadRequest,
	    Forbidden,
	}

	impl CapabilityInvokeApiStatus {
	    pub const fn code(self) -> u16 {
	        match self {
		            Self::Accepted => 202,
		            Self::BadRequest => 400,
		            Self::Forbidden => 403,
		        }
	    }
	}

	pub struct CapabilityInvokeApiSuccessResponse {
	    pub data: CapabilityInvocationReceipt, // data_class: INTERNAL_ONLY
	    pub metadata: CapabilityInvokeApiResponseMetadata, // data_class: INTERNAL_ONLY
	}

	pub struct CapabilityInvokeApiResponseMetadata {
	    pub request_id: String, // data_class: INTERNAL_ONLY
	}

	pub struct CapabilityInvokeApiErrorResponse {
	    pub error: CapabilityInvokeApiErrorBody, // data_class: INTERNAL_ONLY
	}

	pub struct CapabilityInvokeApiErrorBody {
	    pub code: String, // data_class: INTERNAL_ONLY
	    pub message: String, // data_class: INTERNAL_ONLY
	    pub message_localized: Option<String>, // data_class: INTERNAL_ONLY
	    pub request_id: String, // data_class: INTERNAL_ONLY
	    pub details: Vec<CapabilityInvokeApiErrorDetail>, // data_class: INTERNAL_ONLY
	    pub retry_after_seconds: Option<u64>, // data_class: INTERNAL_ONLY
	}

	pub struct CapabilityInvokeApiErrorDetail {
	    pub field: String, // data_class: INTERNAL_ONLY
	    pub issue: String, // data_class: INTERNAL_ONLY
	}

	pub fn invoke_capability_from_api() {
	    let _surface = CAPABILITY_INVOKE_SURFACE;
	}
"#;

    const RUNTIME_API_TEST: &str = r#"
invoke_capability_from_api(foundation, request);
assert_eq!(CAPABILITY_INVOKE_SURFACE, "foundry.capability.invoke");
assert_eq!(CapabilityInvokeApiStatus::Accepted.code(), 202);
assert_eq!(CapabilityInvokeApiStatus::BadRequest.code(), 400);
assert_eq!(CapabilityInvokeApiStatus::Forbidden.code(), 403);

"#;

    const COPY_RUNTIME_API: &str = r#"
pub const COPY_SURFACE: &str = "foundry.copy";
pub enum CopyApiStatus { Ok }
impl CopyApiStatus { pub const fn code(self) -> u16 { match self { Self::Ok => 200 } } }
pub fn copy_resource_from_api() { let _surface = COPY_SURFACE; }
"#;

    const COPY_RUNTIME_API_TEST: &str = r#"
copy_resource_from_api(foundation, request);
assert_eq!(COPY_SURFACE, "foundry.copy");
assert_eq!(CopyApiStatus::Ok.code(), 200);
"#;

    const RUNTIME_STRUCTS: &str = r#"
pub struct CapabilityInvocationRequest {
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub user_id: String, // data_class: INTERNAL_ONLY
    pub capability_id: String, // data_class: INTERNAL_ONLY
    pub purpose: Purpose, // data_class: INTERNAL_ONLY
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
    pub budget_window_id: String, // data_class: INTERNAL_ONLY
    pub projected_cost_micros: u64, // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

pub struct InvocationReceipt {
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub user_id: String, // data_class: INTERNAL_ONLY
    pub capability_id: String, // data_class: INTERNAL_ONLY
    pub evidence_event_hash: String, // data_class: INTERNAL_ONLY
    pub cost_reservation_id: Option<String>, // data_class: INTERNAL_ONLY
    pub cost_budget_warning: Option<BudgetWarning>, // data_class: INTERNAL_ONLY
    pub run_id: Option<String>, // data_class: INTERNAL_ONLY
    pub foundry_step_id: Option<String>, // data_class: INTERNAL_ONLY
    pub foundry_evidence_id: Option<String>, // data_class: INTERNAL_ONLY
}
"#;
}
