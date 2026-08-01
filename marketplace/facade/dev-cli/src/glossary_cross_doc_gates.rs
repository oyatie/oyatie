use std::fs;
use std::path::{Path, PathBuf};

use check_glossary_coverage::{GlossaryTerm, validate_glossary_cross_doc_coverage};

use crate::{
    extract_json_array_for_key, extract_json_object_for_key, extract_json_objects,
    parse_json_string_array_field, parse_json_string_field, path_has_component, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlossaryCoverageValidateArgs {
    docs_dir: PathBuf,
    glossary_path: PathBuf,
    machine_glossary_path: PathBuf,
}

pub(crate) fn parse_glossary_coverage_validate_args(
    args: Vec<String>,
) -> Result<GlossaryCoverageValidateArgs, String> {
    let mut parsed = GlossaryCoverageValidateArgs {
        docs_dir: PathBuf::from("docs"),
        glossary_path: PathBuf::from("docs/GLOSSARY.md"),
        machine_glossary_path: PathBuf::from("docs/machine-readable/glossary.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--glossary" => parsed.glossary_path = PathBuf::from(path),
            "--machine" => parsed.machine_glossary_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_glossary_coverage_gate(
    args: GlossaryCoverageValidateArgs,
) -> Result<(usize, usize), String> {
    let terms = read_machine_glossary_terms(&args.machine_glossary_path)?;
    let glossary_contents = fs::read_to_string(&args.glossary_path).map_err(|error| {
        format!(
            "glossary markdown unreadable {}: {error}",
            args.glossary_path.display()
        )
    })?;
    let cross_doc_contents =
        read_markdown_doc_contents_except(&args.docs_dir, &args.glossary_path)?;
    let report =
        validate_glossary_cross_doc_coverage(terms, &glossary_contents, cross_doc_contents)
            .map_err(|error| format!("glossary coverage invalid: {error:?}"))?;
    Ok((report.terms_checked, report.cross_doc_terms_checked))
}

fn read_machine_glossary_terms(path: &Path) -> Result<Vec<GlossaryTerm>, String> {
    const INDUSTRY_CATEGORIES: &[&str] = &[
        "architecture",
        "operations",
        "cloud",
        "auth",
        "data_search_ml",
        "ads",
        "compliance_kr",
        "compliance_global",
    ];

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("machine-readable glossary unreadable: {error}"))?;
    let term_categories = extract_json_object_for_key(&contents, "term_categories")
        .ok_or_else(|| "machine-readable glossary missing term_categories object".to_string())?;
    let industry_standard = extract_json_object_for_key(term_categories, "industry_standard")
        .ok_or_else(|| "machine-readable glossary missing industry_standard object".to_string())?;

    let mut terms = Vec::new();
    for category in INDUSTRY_CATEGORIES {
        let values =
            parse_json_string_array_field(industry_standard, category).ok_or_else(|| {
                format!("machine-readable glossary missing industry category {category}")
            })?;
        for value in values {
            terms.push(GlossaryTerm {
                term: value,
                source: format!("machine:industry_standard.{category}"),
                cross_doc_required: true,
            });
        }
    }

    let oyatie_specific = extract_json_array_for_key(term_categories, "oyatie_specific")
        .ok_or_else(|| "machine-readable glossary missing oyatie_specific array".to_string())?;
    for object in extract_json_objects(oyatie_specific) {
        let term = parse_json_string_field(object, "term").ok_or_else(|| {
            "machine-readable glossary oyatie_specific entry missing term".to_string()
        })?;
        terms.push(GlossaryTerm {
            term,
            source: "machine:oyatie_specific".into(),
            cross_doc_required: true,
        });
    }

    let retired_terms = extract_json_array_for_key(term_categories, "retired_terms")
        .ok_or_else(|| "machine-readable glossary missing retired_terms array".to_string())?;
    for object in extract_json_objects(retired_terms) {
        let old = parse_json_string_field(object, "old").ok_or_else(|| {
            "machine-readable glossary retired_terms entry missing old".to_string()
        })?;
        terms.push(GlossaryTerm {
            term: old,
            source: "machine:retired_terms.old".into(),
            cross_doc_required: false,
        });
    }

    if terms.is_empty() {
        Err("machine-readable glossary contains no terms".to_string())
    } else {
        Ok(terms)
    }
}

fn read_markdown_doc_contents_except(
    docs_dir: &Path,
    excluded_path: &Path,
) -> Result<Vec<String>, String> {
    let excluded_path = excluded_path.canonicalize().map_err(|error| {
        format!(
            "excluded glossary path unreadable {}: {error}",
            excluded_path.display()
        )
    })?;
    let mut contents = Vec::new();
    collect_markdown_doc_contents_except(docs_dir, &excluded_path, &mut contents)?;
    if contents.is_empty() {
        Err(format!(
            "docs directory contains no markdown files besides {}",
            excluded_path.display()
        ))
    } else {
        Ok(contents)
    }
}

fn collect_markdown_doc_contents_except(
    current: &Path,
    excluded_path: &Path,
    contents: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("glossary coverage docs directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| {
            format!("glossary coverage docs directory entry unreadable: {error}")
        })?;
        let path = entry.path();
        if path_has_component(&path, "raw") || path_has_component(&path, "machine-readable") {
            continue;
        }
        if path.is_dir() {
            collect_markdown_doc_contents_except(&path, excluded_path, contents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("glossary coverage doc path unreadable: {error}"))?;
        if canonical == excluded_path {
            continue;
        }
        contents.push(
            fs::read_to_string(&path)
                .map_err(|error| format!("glossary coverage doc unreadable: {error}"))?,
        );
    }
    Ok(())
}
