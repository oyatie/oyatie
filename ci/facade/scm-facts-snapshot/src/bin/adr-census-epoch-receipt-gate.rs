//! Generic fail-closed gate for the active append-only ADR census epoch.
#![forbid(unsafe_code)]

use std::path::PathBuf;

use ci_scm_facts_snapshot::{
    ADR_CENSUS_EPOCH_RECEIPT_PATH, validate_adr_census_epoch_receipt,
    validate_dormant_p3_epoch_policy,
};

fn main() {
    let repo_root = repo_root_from_current_dir();
    if let Err(error) = validate_adr_census_epoch_receipt(
        &repo_root,
        &repo_root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH),
    ) {
        eprintln!("adr-census-epoch-receipt-gate: {error}");
        std::process::exit(1);
    }
}

fn repo_root_from_current_dir() -> PathBuf {
    let mut directory = std::env::current_dir().unwrap_or_else(|error| {
        panic!("adr-census-epoch-receipt-gate: resolve current directory: {error}")
    });
    for _ in 0..16 {
        if directory.join("specs/root-hub-pointers.json").is_file() {
            return directory;
        }
        if !directory.pop() {
            break;
        }
    }
    panic!("adr-census-epoch-receipt-gate: repository root not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_receipt_matches_exact_head_policy() {
        let root = repo_root_from_current_dir();
        let receipt = root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH);
        validate_adr_census_epoch_receipt(&root, &receipt)
            .unwrap_or_else(|error| panic!("active ADR census epoch receipt is invalid: {error}"));
    }

    #[test]
    fn dormant_p3_policy_is_head_derived_and_deterministic() {
        let root = repo_root_from_current_dir();
        validate_dormant_p3_epoch_policy(&root)
            .unwrap_or_else(|error| panic!("dormant P3 policy is invalid: {error}"));
    }

    #[test]
    fn gate_fails_closed_when_the_active_receipt_is_missing() {
        let root = repo_root_from_current_dir();
        let missing = root.join(".buck-out-test-missing-adr-census-epoch-receipt.json");
        assert!(validate_adr_census_epoch_receipt(&root, &missing).is_err());
    }
}
