use std::fs;
use std::path::{Path, PathBuf};

use check_data_class::{
    FieldIdentity, KernelField, LegacyUnannotatedField, validate_data_class_fitness,
};

use crate::{read_workspace_member_paths, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DataClassValidateArgs {
    workspace_manifest_path: PathBuf,
    legacy_allowance_path: PathBuf,
}

pub(crate) fn parse_data_class_validate_args(
    args: Vec<String>,
) -> Result<DataClassValidateArgs, String> {
    let mut parsed = DataClassValidateArgs {
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
        legacy_allowance_path: PathBuf::from("registry/data-class/legacy-unannotated-fields.tsv"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(path),
            "--legacy" => parsed.legacy_allowance_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_data_class_gate(
    args: DataClassValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let fields = read_kernel_fields(&args.workspace_manifest_path)?;
    let allowances = read_legacy_unannotated_fields(&args.legacy_allowance_path)?;
    let report = validate_data_class_fitness(&fields, &allowances)
        .map_err(|error| format!("kernel field annotation invalid: {error:?}"))?;
    Ok((
        report.fields_checked,
        report.annotated_fields,
        report.legacy_unannotated_fields,
    ))
}

fn read_kernel_fields(workspace_manifest_path: &Path) -> Result<Vec<KernelField>, String> {
    let workspace_dir = workspace_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut fields = Vec::new();
    for member_path in read_workspace_member_paths(workspace_manifest_path)? {
        let crate_name = Path::new(&member_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("workspace member has invalid path: {member_path}"))?;
        if !crate_name.ends_with("-kernel") {
            continue;
        }
        let src_dir = workspace_dir.join(&member_path).join("src");
        collect_kernel_fields(workspace_dir, &src_dir, &mut fields)?;
    }
    Ok(fields)
}

fn collect_kernel_fields(
    workspace_dir: &Path,
    dir: &Path,
    fields: &mut Vec<KernelField>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| {
        format!(
            "kernel source directory unreadable {}: {error}",
            dir.display()
        )
    })? {
        let entry =
            entry.map_err(|error| format!("kernel source directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_kernel_fields(workspace_dir, &path, fields)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("kernel source unreadable {}: {error}", path.display()))?;
        let relative_path = path
            .strip_prefix(workspace_dir)
            .map_err(|error| format!("kernel source outside workspace: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        fields.extend(parse_kernel_fields(&relative_path, &contents));
    }
    Ok(())
}

fn parse_kernel_fields(path: &str, contents: &str) -> Vec<KernelField> {
    let mut fields = Vec::new();
    let mut current_struct = None::<String>;
    let mut brace_depth = 0_i32;
    let mut previous_line_has_data_class_annotation = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        // ADR-0083 Tier 1: clone the active struct name out of the loop-mutable
        // `Option<String>` so the field-construction path below has no `.expect`
        // and the loop tail can still re-assign `current_struct = None` without
        // a borrow conflict.
        let active_struct = match current_struct.clone() {
            Some(name) => name,
            None => {
                if let Some(struct_name) = parse_pub_struct_name(trimmed) {
                    current_struct = Some(struct_name);
                    brace_depth = count_brace_delta(line);
                    previous_line_has_data_class_annotation = false;
                    if brace_depth <= 0 {
                        current_struct = None;
                    }
                }
                continue;
            }
        };

        if let Some(field_name) = parse_pub_field_name(trimmed) {
            let has_data_class_annotation = previous_line_has_data_class_annotation
                || trimmed.contains("data_class:")
                || trimmed.contains("Classified<")
                || trimmed.contains("DataClass")
                || field_name == "data_class"
                || field_name == "data_classes_touched";
            fields.push(KernelField {
                identity: FieldIdentity {
                    path: path.to_string(),
                    struct_name: active_struct,
                    field_name,
                },
                has_data_class_annotation,
            });
            previous_line_has_data_class_annotation = false;
        } else if trimmed.starts_with("//") || trimmed.starts_with("///") {
            previous_line_has_data_class_annotation = trimmed.contains("data_class:");
        } else if trimmed.starts_with("#[") || trimmed.is_empty() {
        } else {
            previous_line_has_data_class_annotation = false;
        }

        brace_depth += count_brace_delta(line);
        if brace_depth <= 0 {
            current_struct = None;
            previous_line_has_data_class_annotation = false;
        }
    }

    fields
}

fn parse_pub_struct_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("pub struct ")?;
    let name = rest
        .split(|character: char| character == '<' || character == '{' || character.is_whitespace())
        .next()?
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_pub_field_name(trimmed: &str) -> Option<String> {
    let mut rest = trimmed.strip_prefix("pub")?.trim_start();
    if rest.starts_with('(') {
        let (_, after_visibility) = rest.split_once(')')?;
        rest = after_visibility.trim_start();
    }
    let (field_name, _) = rest.split_once(':')?;
    let field_name = field_name.trim();
    if field_name.is_empty() || field_name.contains(char::is_whitespace) {
        None
    } else {
        Some(field_name.trim_start_matches("r#").to_string())
    }
}

fn count_brace_delta(line: &str) -> i32 {
    line.chars().filter(|character| *character == '{').count() as i32
        - line.chars().filter(|character| *character == '}').count() as i32
}

fn read_legacy_unannotated_fields(path: &Path) -> Result<Vec<LegacyUnannotatedField>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("legacy data-class allowance ledger unreadable: {error}"))?;
    let mut allowances = Vec::new();
    for (line_index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts = line.splitn(4, '\t').collect::<Vec<_>>();
        if parts.len() != 4 {
            return Err(format!(
                "{}:{}: legacy data-class allowance must have four tab-separated fields",
                path.display(),
                line_index + 1
            ));
        }
        allowances.push(LegacyUnannotatedField {
            identity: FieldIdentity {
                path: parts[0].to_string(),
                struct_name: parts[1].to_string(),
                field_name: parts[2].to_string(),
            },
            rationale: parts[3].to_string(),
        });
    }
    Ok(allowances)
}
