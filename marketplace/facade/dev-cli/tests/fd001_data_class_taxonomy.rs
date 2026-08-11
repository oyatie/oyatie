use std::{fs, path::PathBuf};

#[test]
fn os_kernel_abi_catalog_is_in_data_class_scan_set_when_present() -> Result<(), String> {
    let repo_root = repo_root()?;
    let catalog_path = repo_root.join("registry/catalog/os-kernel-abi.yaml");
    if !catalog_path.is_file() {
        return Ok(());
    }

    let catalog_contents = fs::read_to_string(&catalog_path)
        .map_err(|error| format!("failed to read {}: {error}", catalog_path.display()))?;
    let role = parse_catalog_scalar(&catalog_contents, "role")
        .ok_or_else(|| "os-kernel-abi.yaml missing role".to_string())?;
    if role != "kernel" {
        return Err(format!(
            "os-kernel-abi.yaml must declare role: kernel, got role: {role}"
        ));
    }

    let source_crate = parse_catalog_source_crate(&catalog_contents).ok_or_else(|| {
        "os-kernel-abi.yaml must declare traceability.source_crate".to_string()
    })?;
    let member_path = source_crate
        .strip_suffix("/Cargo.toml")
        .unwrap_or(source_crate.as_str())
        .to_string();
    let manifest_path = repo_root.join(&member_path).join("Cargo.toml");
    if !manifest_path.is_file() {
        return Ok(());
    }

    let package_name = read_package_name(&manifest_path)?;
    if package_name != "os-kernel-abi" {
        return Err(format!(
            "os-kernel-abi workspace member has unexpected package name: {package_name}"
        ));
    }

    let member_path_buf = PathBuf::from(&member_path);
    let dir_name = member_path_buf
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("workspace member has invalid path: {member_path}"))?;
    if dir_name.ends_with("-kernel") {
        return Err(format!(
            "os-kernel-abi meta-test requires a non -kernel directory suffix; got {dir_name}"
        ));
    }

    Ok(())
}

#[test]
fn retired_financial_data_class_tokens_absent_from_contract_annotations() -> Result<(), String> {
    let repo_root = repo_root()?;

    let annotation_files = [
        "contracts/openapi/cloud/cloud-billing-invoice-v1.yaml",
        "contracts/openapi/cloud/cloud-finops-report-v1.yaml",
        "billing/ports/tax-api/src/lib.rs",
    ];
    let retired_annotations = [
        "x-oyatie-data-class: FINANCIAL_KR_신용정보",
        "data_class: FINANCIAL_CREDIT",
    ];

    let mut violations = Vec::new();
    for relative_path in annotation_files {
        let contents = fs::read_to_string(repo_root.join(relative_path))
            .map_err(|error| format!("failed to read {relative_path}: {error}"))?;
        for (line_index, line) in contents.lines().enumerate() {
            if retired_annotations.iter().any(|token| line.contains(token)) {
                violations.push(format!(
                    "{}:{}:{}",
                    relative_path,
                    line_index + 1,
                    line.trim()
                ));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "retired financial data-class annotations must use FINANCIAL_REGULATED_CREDIT:\n{}",
            violations.join("\n")
        ))
    }
}

fn repo_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .map(PathBuf::from)
        .ok_or_else(|| "dev-cli crate should live under the repo root".to_owned())
}

fn read_package_name(manifest_path: &PathBuf) -> Result<String, String> {
    let manifest = fs::read_to_string(manifest_path)
        .map_err(|error| format!("package manifest unreadable {}: {error}", manifest_path.display()))?;
    let mut in_package_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package_section = trimmed == "[package]";
            continue;
        }
        if !in_package_section {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        let package_name = value.trim().trim_matches('"').to_string();
        if !package_name.is_empty() {
            return Ok(package_name);
        }
    }
    Err(format!(
        "package manifest missing package name: {}",
        manifest_path.display()
    ))
}

fn parse_catalog_scalar(contents: &str, target_key: &str) -> Option<String> {
    for line in contents.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        if key.trim() == target_key {
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
            // Indentation must be read from the raw line: `stripped` is already
            // trim()'d, so `starts_with(' ')` would never hold and the nested
            // `source_crate:` key would be misread as a sibling key.
            let indented = line.starts_with(' ') || line.starts_with('\t');
            if !indented && stripped.contains(':') {
                break;
            }
            let Some((key, value)) = stripped.split_once(':') else {
                continue;
            };
            if key.trim() == "source_crate" {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}
