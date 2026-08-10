use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use check_data_class::{
    FieldIdentity, KernelField, LegacyUnannotatedField, validate_data_class_fitness,
};

use crate::workspace_manifest::read_package_name;
use crate::{read_workspace_member_paths, usage};

const DEFAULT_CATALOG_DIR: &str = "registry/catalog";

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
    let catalog_dir = workspace_dir.join(DEFAULT_CATALOG_DIR);
    let member_paths = read_workspace_member_paths(workspace_manifest_path)?;
    let kernel_catalog_ids = load_kernel_catalog_ids(&catalog_dir)?;
    let scan_members = select_kernel_scan_members(
        &catalog_dir,
        &kernel_catalog_ids,
        workspace_dir,
        &member_paths,
    )?;
    let mut fields = Vec::new();
    for member_path in scan_members {
        let src_dir = workspace_dir.join(&member_path).join("src");
        collect_kernel_fields(workspace_dir, &src_dir, &mut fields)?;
    }
    Ok(fields)
}

fn select_kernel_scan_members(
    catalog_dir: &Path,
    kernel_catalog_ids: &[String],
    workspace_dir: &Path,
    member_paths: &[String],
) -> Result<Vec<String>, String> {
    let mut scan_members = BTreeSet::new();
    for member_path in member_paths {
        let manifest_path = workspace_dir.join(member_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let package_name = read_package_name(&manifest_path)?;
        if is_kernel_role_catalog(catalog_dir, &package_name)? {
            scan_members.insert(member_path.clone());
        }
    }

    let mut coverage_errors = Vec::new();
    for catalog_id in kernel_catalog_ids {
        let Some(member_path) =
            resolve_catalog_workspace_member(catalog_id, catalog_dir, workspace_dir, member_paths)?
        else {
            continue;
        };
        if scan_members.contains(&member_path) {
            continue;
        }
        coverage_errors.push(format!(
            "kernel catalog {catalog_id} maps to workspace member {member_path} but is absent from data-class scan set (registry/catalog/{catalog_id}.yaml role: kernel)"
        ));
    }
    if !coverage_errors.is_empty() {
        return Err(coverage_errors.join("\n"));
    }

    Ok(scan_members.into_iter().collect())
}

fn is_kernel_role_catalog(catalog_dir: &Path, catalog_id: &str) -> Result<bool, String> {
    let path = catalog_dir.join(format!("{catalog_id}.yaml"));
    if !path.is_file() {
        return Ok(false);
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("catalog record unreadable {}: {error}", path.display()))?;
    Ok(parse_catalog_role(&contents).as_deref() == Some("kernel"))
}

fn load_kernel_catalog_ids(catalog_dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(catalog_dir).map_err(|error| {
        format!(
            "kernel catalog directory unreadable {}: {error}",
            catalog_dir.display()
        )
    })?;
    let mut catalog_ids = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("kernel catalog directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }
        let Some(catalog_id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!("kernel catalog record unreadable {}: {error}", path.display())
        })?;
        if parse_catalog_role(&contents).as_deref() == Some("kernel") {
            catalog_ids.push(catalog_id.to_string());
        }
    }
    catalog_ids.sort();
    Ok(catalog_ids)
}

fn resolve_catalog_workspace_member(
    catalog_id: &str,
    catalog_dir: &Path,
    workspace_dir: &Path,
    member_paths: &[String],
) -> Result<Option<String>, String> {
    if let Some(source_crate) = read_catalog_source_crate(catalog_dir, catalog_id)? {
        let member_path = source_crate
            .strip_suffix("/Cargo.toml")
            .unwrap_or(source_crate.as_str())
            .to_string();
        if member_paths.iter().any(|member| member == &member_path) {
            return Ok(Some(member_path));
        }
    }

    for member_path in member_paths {
        let manifest_path = workspace_dir.join(member_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        if read_package_name(&manifest_path)? == catalog_id {
            return Ok(Some(member_path.clone()));
        }
    }

    Ok(None)
}

fn read_catalog_source_crate(catalog_dir: &Path, catalog_id: &str) -> Result<Option<String>, String> {
    let path = catalog_dir.join(format!("{catalog_id}.yaml"));
    if !path.is_file() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("catalog record unreadable {}: {error}", path.display()))?;
    Ok(parse_catalog_source_crate(&contents))
}

fn parse_catalog_role(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        if key.trim() == "role" {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn parse_catalog_source_crate(contents: &str) -> Option<String> {
    let mut in_traceability = false;
    for line in contents.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        if stripped == "traceability:" {
            in_traceability = true;
            continue;
        }
        if in_traceability {
            if !stripped.starts_with(' ') && stripped.contains(':') {
                break;
            }
            let trimmed = stripped.trim_start();
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            if key.trim() == "source_crate" {
                return Some(value.trim().to_string());
            }
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn parse_catalog_role_reads_kernel_role() {
        let yaml = "context: os\nrole: kernel\nplane: control\n";
        assert_eq!(parse_catalog_role(yaml), Some("kernel".to_string()));
    }

    #[test]
    fn parse_catalog_source_crate_reads_traceability_block() {
        let yaml = "context: os\nrole: kernel\ntraceability:\n  source_crate: os/ports/kernel-abi/Cargo.toml\n";
        assert_eq!(
            parse_catalog_source_crate(yaml),
            Some("os/ports/kernel-abi/Cargo.toml".to_string())
        );
    }

    #[test]
    fn os_kernel_abi_catalog_resolves_to_scan_member_via_package_name() {
        let temp = std::env::temp_dir().join(format!(
            "oya-data-class-os-kernel-abi-{}",
            std::process::id()
        ));
        let workspace_dir = &temp;
        fs::create_dir_all(workspace_dir.join("registry/catalog")).expect("catalog dir");
        fs::create_dir_all(workspace_dir.join("os/ports/kernel-abi/src")).expect("crate src");
        fs::write(
            workspace_dir.join("registry/catalog/os-kernel-abi.yaml"),
            "context: os\nrole: kernel\ncapability: kernel-abi-port\nplane: control\ntraceability:\n  source_crate: os/ports/kernel-abi/Cargo.toml\n",
        )
        .expect("catalog yaml");
        fs::write(
            workspace_dir.join("os/ports/kernel-abi/Cargo.toml"),
            "[package]\nname = \"os-kernel-abi\"\n",
        )
        .expect("crate manifest");

        let catalog_dir = workspace_dir.join("registry/catalog");
        let kernel_catalog_ids = load_kernel_catalog_ids(&catalog_dir).expect("catalog ids");
        assert!(kernel_catalog_ids.contains(&"os-kernel-abi".to_string()));

        let member_paths = vec!["os/ports/kernel-abi".to_string()];
        let scan_members = select_kernel_scan_members(
            &catalog_dir,
            &kernel_catalog_ids,
            workspace_dir,
            &member_paths,
        )
        .expect("scan members");
        assert_eq!(scan_members, vec!["os/ports/kernel-abi".to_string()]);
    }

    #[test]
    fn kernel_catalog_coverage_fails_when_workspace_member_not_scanned() {
        let temp = std::env::temp_dir().join(format!(
            "oya-data-class-coverage-{}",
            std::process::id()
        ));
        let workspace_dir = &temp;
        fs::create_dir_all(workspace_dir.join("registry/catalog")).expect("catalog dir");
        fs::create_dir_all(workspace_dir.join("os/ports/kernel-abi/src")).expect("crate src");
        fs::write(
            workspace_dir.join("registry/catalog/os-kernel-abi.yaml"),
            "context: os\nrole: kernel\ncapability: kernel-abi-port\nplane: control\ntraceability:\n  source_crate: os/ports/kernel-abi/Cargo.toml\n",
        )
        .expect("catalog yaml");
        fs::write(
            workspace_dir.join("registry/catalog/not-a-kernel.yaml"),
            "context: os\nrole: rest\ncapability: x\nplane: control\n",
        )
        .expect("non-kernel catalog");
        fs::write(
            workspace_dir.join("os/ports/kernel-abi/Cargo.toml"),
            "[package]\nname = \"not-matching-package\"\n",
        )
        .expect("crate manifest");

        let catalog_dir = workspace_dir.join("registry/catalog");
        let kernel_catalog_ids = load_kernel_catalog_ids(&catalog_dir).expect("catalog ids");
        let member_paths = vec!["os/ports/kernel-abi".to_string()];
        let error = select_kernel_scan_members(
            &catalog_dir,
            &kernel_catalog_ids,
            workspace_dir,
            &member_paths,
        )
        .expect_err("missing scan coverage");
        assert!(
            error.contains("os-kernel-abi"),
            "expected os-kernel-abi coverage error, got: {error}"
        );
    }
}
