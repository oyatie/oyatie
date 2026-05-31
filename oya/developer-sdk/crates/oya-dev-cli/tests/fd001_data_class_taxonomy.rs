use std::{fs, path::PathBuf};

#[test]
fn retired_financial_data_class_tokens_absent_from_contract_annotations() -> Result<(), String> {
    let repo_root = repo_root()?;

    let annotation_files = [
        "contracts/openapi/cloud/cloud-billing-invoice-v1.yaml",
        "contracts/openapi/cloud/cloud-finops-report-v1.yaml",
        "cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/src/lib.rs",
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
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|path| {
            path.join("contracts/openapi/cloud/cloud-billing-invoice-v1.yaml")
                .exists()
        })
        .map(PathBuf::from)
        .ok_or_else(|| "repository root marker not found for fd001 data-class taxonomy".to_owned())
}
