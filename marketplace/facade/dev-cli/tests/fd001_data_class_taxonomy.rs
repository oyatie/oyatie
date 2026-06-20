use std::{fs, path::PathBuf};

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
