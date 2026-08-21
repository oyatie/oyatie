use std::fs;
use std::path::{Path, PathBuf};

use check_doc_catalog::{DocCatalogRecord, validate_doc_catalog};
use check_documentation_system::{
    DocumentationPipelineRecord, DocumentationPipelineState, DocumentationSystemEvidence,
    validate_documentation_system,
};
use check_readme_coverage::validate_readme_doc_coverage;
use check_gate_catalog_domain::all_canonical_commands_rendered;

use crate::{
    extract_first_backticked_value, extract_json_object_entries, extract_json_object_for_key,
    json_field_has_non_empty_value, next_arg, parse_json_string_array_field,
    parse_json_string_field, slash_path, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadmeDocCoverageValidateArgs {
    docs_dir: PathBuf,
    machine_catalog_path: PathBuf,
}

pub(crate) fn parse_readme_doc_coverage_validate_args(
    args: Vec<String>,
) -> Result<ReadmeDocCoverageValidateArgs, String> {
    let mut parsed = ReadmeDocCoverageValidateArgs {
        docs_dir: PathBuf::from("docs"),
        machine_catalog_path: PathBuf::from("docs/machine-readable/catalog.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--catalog" => parsed.machine_catalog_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_readme_doc_coverage_gate(
    args: ReadmeDocCoverageValidateArgs,
) -> Result<usize, String> {
    let root_doc_paths = list_root_doc_paths(&args.docs_dir)?;
    let catalog_doc_paths = read_doc_catalog_records(&args.machine_catalog_path)?
        .into_iter()
        .map(|record| record.path)
        .collect::<Vec<_>>();
    let readme_linked_doc_paths = read_readme_doc_paths(&args.docs_dir.join("README.md"))?;
    let report =
        validate_readme_doc_coverage(root_doc_paths, catalog_doc_paths, readme_linked_doc_paths)
            .map_err(|error| format!("readme doc coverage invalid: {error:?}"))?;
    Ok(report.documents_checked)
}

fn read_readme_doc_paths(path: &Path) -> Result<Vec<String>, String> {
    let contents =
        fs::read_to_string(path).map_err(|error| format!("README unreadable: {error}"))?;
    let mut paths = Vec::new();
    for target in extract_markdown_link_targets(&contents) {
        if target.ends_with(".md") {
            paths.push(normalize_readme_doc_path(&target));
        }
    }
    Ok(paths)
}

fn extract_markdown_link_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = contents;
    while let Some(open_index) = rest.find("](") {
        let after_open = &rest[open_index + 2..];
        let Some(close_index) = after_open.find(')') else {
            break;
        };
        let target = after_open[..close_index]
            .split('#')
            .next()
            .unwrap_or_default()
            .trim();
        if !target.is_empty() {
            targets.push(target.to_string());
        }
        rest = &after_open[close_index + 1..];
    }
    targets
}

fn normalize_readme_doc_path(target: &str) -> String {
    if target.starts_with("docs/") {
        target.to_string()
    } else {
        format!("docs/{target}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocCatalogValidateArgs {
    docs_dir: PathBuf,
    machine_catalog_path: PathBuf,
}

pub(crate) fn parse_doc_catalog_validate_args(
    args: Vec<String>,
) -> Result<DocCatalogValidateArgs, String> {
    let mut parsed = DocCatalogValidateArgs {
        docs_dir: PathBuf::from("docs"),
        machine_catalog_path: PathBuf::from("docs/machine-readable/catalog.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--catalog" => parsed.machine_catalog_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_doc_catalog_gate(args: DocCatalogValidateArgs) -> Result<usize, String> {
    let records = read_doc_catalog_records(&args.machine_catalog_path)?;
    let existing_doc_paths = list_root_doc_paths(&args.docs_dir)?;
    let markdown_catalog_paths =
        read_markdown_doc_catalog_paths(&args.docs_dir.join("DOC-CATALOG.md"))?;
    let dependency_reference_targets = list_dependency_reference_targets(&args.docs_dir)?;
    let report = validate_doc_catalog(
        &records,
        existing_doc_paths,
        markdown_catalog_paths,
        dependency_reference_targets,
    )
    .map_err(|error| format!("doc catalog coverage invalid: {error:?}"))?;
    Ok(report.documents_checked)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocumentationSystemValidateArgs {
    documentation_path: PathBuf,
    pipeline_path: PathBuf,
    /// Optional test-only override for the wired-commands corpus. When
    /// `None` (production default) the kernel sources its wired-commands
    /// catalog from `oya-governance-gate-catalog-domain` per the .sh-removal
    /// chain IP-C. When `Some(path)`, the CLI reads the path verbatim —
    /// used by the integration-test fixtures in `tests/gate_cli.rs` to
    /// exercise rejection paths.
    check_script_path: Option<PathBuf>,
    wiki_quickref_path: PathBuf,
    repo_root: PathBuf,
}

pub(crate) fn parse_documentation_system_validate_args(
    args: Vec<String>,
) -> Result<DocumentationSystemValidateArgs, String> {
    let mut parsed = DocumentationSystemValidateArgs {
        documentation_path: PathBuf::from("docs/DOCUMENTATION.md"),
        pipeline_path: PathBuf::from("registry/docs/pipeline.tsv"),
        check_script_path: None,
        wiki_quickref_path: PathBuf::from("docs/wiki/quickref/README.md"),
        repo_root: PathBuf::from("."),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let path = next_arg(&mut iter)?;
        match flag.as_str() {
            "--documentation" => parsed.documentation_path = PathBuf::from(path),
            "--pipeline" => parsed.pipeline_path = PathBuf::from(path),
            "--check-script" => parsed.check_script_path = Some(PathBuf::from(path)),
            "--wiki-quickref" => parsed.wiki_quickref_path = PathBuf::from(path),
            "--repo-root" => parsed.repo_root = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_documentation_system_gate(
    args: DocumentationSystemValidateArgs,
) -> Result<check_documentation_system::DocumentationSystemReport, String> {
    let documentation = fs::read_to_string(&args.documentation_path).map_err(|error| {
        format!(
            "documentation system doc unreadable {}: {error}",
            args.documentation_path.display()
        )
    })?;
    // Canonical catalog replaces the legacy `scripts/check.sh` file read
    // (audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
    // row B-1, .sh-removal chain IP-C). The catalog substring-matches the
    // same `cargo run -p oya-dev-cli -- doc <step>` / `catalog validate`
    // patterns the script body historically supplied.
    // Test-only override: `--check-script <path>` swaps the canonical
    // catalog for the file body, so integration-test fixtures can
    // exercise rejection paths.
    let wired_commands = match args.check_script_path.as_ref() {
        Some(path) => fs::read_to_string(path).map_err(|error| {
            format!(
                "documentation system check script unreadable {}: {error}",
                path.display()
            )
        })?,
        None => all_canonical_commands_rendered(),
    };
    let records =
        read_documentation_pipeline_records(&args.pipeline_path, &args.repo_root, &wired_commands)?;
    let evidence = DocumentationSystemEvidence {
        documentation_lane_declared: documentation.contains("oya-governance-docs"),
        wiki_quickref_referenced: documentation.contains("docs/wiki/quickref"),
        wiki_quickref_present: args.wiki_quickref_path.is_file(),
        records,
    };
    validate_documentation_system(evidence)
        .map_err(|error| format!("documentation system invalid: {error:?}"))
}

fn read_documentation_pipeline_records(
    path: &Path,
    repo_root: &Path,
    wired_commands: &str,
) -> Result<Vec<DocumentationPipelineRecord>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "documentation pipeline registry unreadable {}: {error}",
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
        if visible.starts_with("step_id\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}:{} expected documentation pipeline TSV header",
                path.display(),
                line_index + 1
            ));
        }
        records.push(parse_documentation_pipeline_record(
            path,
            line_index + 1,
            row,
            repo_root,
            wired_commands,
        )?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing documentation pipeline TSV header",
            path.display()
        ));
    }
    Ok(records)
}

fn parse_documentation_pipeline_record(
    path: &Path,
    line_number: usize,
    row: &str,
    repo_root: &Path,
    wired_commands: &str,
) -> Result<DocumentationPipelineRecord, String> {
    let cells = row
        .split('\t')
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if cells.len() != 6 {
        return Err(format!(
            "{}:{line_number} documentation pipeline row must have 6 TSV columns",
            path.display()
        ));
    }
    let state = DocumentationPipelineState::parse(&cells[2]).ok_or_else(|| {
        format!(
            "{}:{line_number} unknown documentation pipeline state {}",
            path.display(),
            cells[2]
        )
    })?;
    let check_command = if cells[3].is_empty() {
        None
    } else {
        Some(cells[3].clone())
    };
    Ok(DocumentationPipelineRecord {
        step_id: cells[0].clone(),
        documented_command: cells[1].clone(),
        state,
        check_command_wired: check_command
            .as_deref()
            .is_some_and(|command| wired_commands.contains(command)),
        check_command,
        scope_present: repo_root.join(&cells[4]).exists(),
        scope_path: cells[4].clone(),
        rationale: cells[5].clone(),
    })
}

fn read_doc_catalog_records(path: &Path) -> Result<Vec<DocCatalogRecord>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("machine-readable doc catalog unreadable: {error}"))?;
    let docs_object = extract_json_object_for_key(&contents, "docs")
        .ok_or_else(|| "machine-readable doc catalog missing docs object".to_string())?;
    extract_json_object_entries(docs_object)
        .into_iter()
        .map(|(doc_id, object)| {
            let path = parse_json_string_field(object, "path")
                .ok_or_else(|| format!("{doc_id} missing path"))?;
            let owner_team = parse_json_string_or_array_field(object, "owner_team")
                .ok_or_else(|| format!("{doc_id} missing owner_team"))?;
            let dependent_docs = parse_json_string_array_field(object, "dependent_docs")
                .ok_or_else(|| format!("{doc_id} missing dependent_docs"))?;
            Ok(DocCatalogRecord {
                doc_id,
                path,
                owner_team,
                dependent_docs,
                validation_check_present: json_field_has_non_empty_value(
                    object,
                    "validation_check",
                ),
            })
        })
        .collect()
}

fn parse_json_string_or_array_field(object: &str, key: &str) -> Option<String> {
    if let Some(value) =
        parse_json_string_field(object, key).filter(|value| !value.trim().is_empty())
    {
        return Some(value);
    }
    let values = parse_json_string_array_field(object, key)?
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(","))
}

fn list_root_doc_paths(docs_dir: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(docs_dir)
        .map_err(|error| format!("docs directory unreadable {}: {error}", docs_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("docs directory entry unreadable: {error}"))?;
        let path = entry.path();
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("doc path has invalid file name: {}", path.display()))?;
        paths.push(format!("docs/{file_name}"));
    }
    if paths.is_empty() {
        Err("docs directory contains no root markdown files".to_string())
    } else {
        Ok(paths)
    }
}

fn list_dependency_reference_targets(docs_dir: &Path) -> Result<Vec<String>, String> {
    let mut targets = Vec::new();
    collect_dependency_reference_targets(docs_dir, docs_dir, &mut targets)?;
    let workspace_root = infer_workspace_root_from_docs_dir(docs_dir);
    let specs_dir = workspace_root.join("specs");
    if specs_dir.is_dir() {
        collect_dependency_reference_targets(&workspace_root, &specs_dir, &mut targets)?;
    }
    if workspace_root.join(".github/CODEOWNERS").is_file() {
        targets.push(".github/CODEOWNERS".to_string());
    }
    Ok(targets)
}

fn collect_dependency_reference_targets(
    root: &Path,
    current: &Path,
    targets: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("dependency target directory unreadable: {error}"))?
    {
        let entry = entry
            .map_err(|error| format!("dependency target directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_dependency_reference_targets(root, &path, targets)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("dependency target path not under docs dir: {error}"))?;
        let relative = slash_path(relative);
        targets.push(relative.clone());
        targets.push(format!("docs/{relative}"));
    }
    Ok(())
}

fn infer_workspace_root_from_docs_dir(docs_dir: &Path) -> PathBuf {
    if docs_dir.file_name().and_then(|name| name.to_str()) == Some("docs") {
        return docs_dir
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
    }
    docs_dir.to_path_buf()
}

fn read_markdown_doc_catalog_paths(path: &Path) -> Result<Vec<String>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("DOC-CATALOG unreadable {}: {error}", path.display()))?;
    let mut paths = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("| `doc.") {
            continue;
        }
        let cells = trimmed.split('|').collect::<Vec<_>>();
        let Some(path_cell) = cells.get(2) else {
            continue;
        };
        let Some(path) = extract_first_backticked_value(path_cell) else {
            continue;
        };
        if !path.ends_with(".md") {
            continue;
        }
        if path.starts_with("docs/") {
            paths.push(path);
        } else {
            paths.push(format!("docs/{path}"));
        }
    }
    Ok(paths)
}
