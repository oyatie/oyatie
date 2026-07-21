// ADR-0083 Tier 3: this repository conformance test uses `expect` to assert
// immutable authority-boundary fixtures.
#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

const CURRENT_AUTHORITY_BOUNDARY: &str = "plain git + protected GitHub PRs; oya-ci-required is the sole merge-status authority; cloud-ci runtime-promotion claims remain held pending independently verified runtime evidence";

const CONTRACTS: &[&str] = &[
    "specs/test-standard.json",
    "specs/deployment-ops-contract.json",
    "specs/agentic-slo-gated-promotion.json",
    "specs/hyperscaler-gates.json",
];

const RETIRED_AUTHORITY_MARKERS: &[&str] = &[
    "oya vcs",
    "oya git",
    "oya-dev-cli",
    "changebundle",
    "merge queue",
    "merge-queue",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn accepted_pipeline_contracts_expose_only_current_merge_authority() {
    let root = repo_root();

    for relative_path in CONTRACTS {
        let absolute_path = root.join(relative_path);
        let text = fs::read_to_string(&absolute_path).expect("contract readable");
        let document: Value = serde_json::from_str(&text).expect("contract valid JSON");
        let metadata = document
            .get("_meta")
            .or_else(|| document.get("_metadata"))
            .and_then(Value::as_object)
            .expect("contract metadata object");

        assert_eq!(
            metadata
                .get("current_authority_boundary")
                .and_then(Value::as_str),
            Some(CURRENT_AUTHORITY_BOUNDARY),
            "{relative_path} must state the current ADR-0363/ADR-0515 authority boundary"
        );

        let lower = text.to_ascii_lowercase();
        for marker in RETIRED_AUTHORITY_MARKERS {
            assert!(
                !lower.contains(marker),
                "{relative_path} retains retired authority marker {marker:?}; historical provenance must be moved out of this active contract or explicitly typed in a nonauthority artifact"
            );
        }
    }
}
