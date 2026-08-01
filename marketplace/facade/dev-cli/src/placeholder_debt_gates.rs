use std::fs;
use std::path::{Path, PathBuf};

use check_placeholder_debt::{
    PlaceholderDebtFinding, PlaceholderDebtRecord, PlaceholderDocument, discover_placeholder_debt,
    validate_placeholder_debt,
};

use crate::{path_has_component, slash_path, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaceholderDebtValidateArgs {
    docs_dir: PathBuf,
    registry_path: PathBuf,
    write_registry_path: Option<PathBuf>,
    report_path: Option<PathBuf>,
}

pub(crate) fn parse_placeholder_debt_validate_args(
    args: Vec<String>,
) -> Result<PlaceholderDebtValidateArgs, String> {
    let mut parsed = PlaceholderDebtValidateArgs {
        docs_dir: PathBuf::from("docs"),
        registry_path: PathBuf::from("registry/placeholder-debt/registry.tsv"),
        write_registry_path: None,
        report_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--registry" => parsed.registry_path = PathBuf::from(path),
            "--write-registry" => parsed.write_registry_path = Some(PathBuf::from(path)),
            "--write-report" => parsed.report_path = Some(PathBuf::from(path)),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_placeholder_debt_gate(
    args: PlaceholderDebtValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let documents = read_markdown_placeholder_documents(&args.docs_dir)?;
    let report = if let Some(path) = &args.write_registry_path {
        let report = discover_placeholder_debt(documents)
            .map_err(|error| format!("placeholder debt invalid: {error:?}"))?;
        write_placeholder_debt_registry(path, &report.findings)?;
        report
    } else {
        let records = read_placeholder_debt_registry(&args.registry_path)?;
        validate_placeholder_debt(documents, records)
            .map_err(|error| format!("placeholder debt invalid: {error:?}"))?
    };
    if let Some(path) = &args.report_path {
        write_placeholder_debt_report(path, &report.findings)?;
    }
    Ok((
        report.documents_checked,
        report.open_placeholders,
        report.tracked_records,
    ))
}

fn read_markdown_placeholder_documents(
    docs_dir: &Path,
) -> Result<Vec<PlaceholderDocument>, String> {
    let mut documents = Vec::new();
    collect_markdown_placeholder_documents(docs_dir, docs_dir, &mut documents)?;
    if documents.is_empty() {
        Err(format!(
            "placeholder debt docs directory contains no markdown files: {}",
            docs_dir.display()
        ))
    } else {
        Ok(documents)
    }
}

fn collect_markdown_placeholder_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<PlaceholderDocument>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("placeholder debt docs directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| {
            format!("placeholder debt docs directory entry unreadable: {error}")
        })?;
        let path = entry.path();
        if path_has_component(&path, "raw") || path_has_component(&path, "machine-readable") {
            continue;
        }
        if path.is_dir() {
            collect_markdown_placeholder_documents(root, &path, documents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("placeholder debt doc path not under docs dir: {error}"))?;
        let normalized_path = format!("docs/{}", slash_path(relative));
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("placeholder debt doc unreadable: {error}"))?;
        documents.push(PlaceholderDocument {
            path: normalized_path,
            contents,
        });
    }
    Ok(())
}

fn read_placeholder_debt_registry(path: &Path) -> Result<Vec<PlaceholderDebtRecord>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "placeholder debt registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut records = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let columns = trimmed.split('\t').collect::<Vec<_>>();
        if columns.len() != 5 {
            return Err(format!(
                "placeholder debt registry {}:{} must be '<token>\\t<path>\\t<occurrences>\\t<excerpt>\\t<rationale>'",
                path.display(),
                index + 1
            ));
        }
        let occurrences = columns[2].parse::<usize>().map_err(|error| {
            format!(
                "placeholder debt registry {}:{} has invalid occurrence count: {error}",
                path.display(),
                index + 1
            )
        })?;
        records.push(PlaceholderDebtRecord {
            token: columns[0].trim().to_string(),
            path: columns[1].trim().to_string(),
            occurrences,
            excerpt: columns[3].trim().to_string(),
            rationale: columns[4].trim().to_string(),
        });
    }
    Ok(records)
}

fn write_placeholder_debt_registry(
    path: &Path,
    findings: &[PlaceholderDebtFinding],
) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "placeholder debt registry directory unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let mut findings = findings.to_vec();
    findings.sort_by(|left, right| {
        (&left.token, &left.path, &left.excerpt).cmp(&(&right.token, &right.path, &right.excerpt))
    });
    let mut contents =
        "# Oyatie placeholder debt registry. Format: <token>\\t<path>\\t<occurrences>\\t<excerpt>\\t<rationale>\n".to_string();
    for finding in findings {
        contents.push_str(&format!(
            "{}\t{}\t{}\t{}\towner=council-architecture; issue=PLACEHOLDER-DEBT-AUTO-CAPTURE; captured_at=2026-05-19; action=close-or-archive-before-production-claim\n",
            finding.token, finding.path, finding.occurrences, finding.excerpt
        ));
    }
    fs::write(path, contents).map_err(|error| {
        format!(
            "placeholder debt registry unwritable {}: {error}",
            path.display()
        )
    })
}

fn write_placeholder_debt_report(
    path: &Path,
    findings: &[PlaceholderDebtFinding],
) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "placeholder debt report directory unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let mut findings = findings.to_vec();
    findings.sort_by(|left, right| {
        (&left.token, &left.path, &left.excerpt).cmp(&(&right.token, &right.path, &right.excerpt))
    });
    let mut contents = "# token\tpath\toccurrences\texcerpt\n".to_string();
    for finding in findings {
        contents.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            finding.token, finding.path, finding.occurrences, finding.excerpt
        ));
    }
    fs::write(path, contents).map_err(|error| {
        format!(
            "placeholder debt report unwritable {}: {error}",
            path.display()
        )
    })
}
