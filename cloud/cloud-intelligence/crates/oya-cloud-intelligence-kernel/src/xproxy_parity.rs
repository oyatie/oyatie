use serde::Deserialize;
use std::collections::BTreeMap;

pub const EXTERNAL_PROXY_REFERENCE_BASELINE_JSON: &str =
    include_str!("../capability-parity/external-proxy-reference-20260610.json");
pub const EXTERNAL_PROXY_REFERENCE_DRAFT_TARGETS_JSON: &str =
    include_str!("../capability-parity/external-proxy-reference-draft-targets-20260610.json");

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CapabilityParityMap {
    pub schema_version: String,
    pub kind: String,
    pub artifact_family: String,
    pub capability_namespace: String,
    pub provenance: ParityProvenance,
    pub architecture: ParityArchitecture,
    pub capabilities: Vec<CapabilityParityRow>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ParityProvenance {
    pub source_repo: String,
    pub package_name: String,
    pub package_version: String,
    pub commit_sha: String,
    pub pinned_tree_url: String,
    pub baseline_captured_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ParityArchitecture {
    pub cloud_native_only: bool,
    pub no_cli_tui_targets: bool,
    pub secret_resolution_boundary: String,
    pub authorization_boundary: String,
    pub adapter_policy: String,
    pub gateway_deployment_target: String,
    pub worker_controller_target: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CapabilityParityRow {
    pub id: String,
    pub status: CapabilityStatus,
    pub source_evidence: Vec<String>,
    pub source_behavior_summary: String,
    pub target_boundary: String,
    pub target_capability: String,
    pub target_tests: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReferenceDraftParityTargets {
    pub schema_version: String,
    pub kind: String,
    pub artifact_family: String,
    pub capability_namespace: String,
    pub scope: ReferenceDraftScope,
    pub source_provenance: Vec<ReferenceSourceProvenance>,
    pub targets: Vec<ReferenceDraftParityTarget>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReferenceDraftScope {
    pub generation_providers: Vec<String>,
    pub routing_advisor_models: Vec<String>,
    pub routing_advisors_may_generate: bool,
    pub translations_owned_by_provider_adapters: bool,
    pub model_execution_boundary: String,
    pub advisor_execution_boundary: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReferenceSourceProvenance {
    pub source_url: String,
    pub source_kind: String,
    pub retrieved_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReferenceDraftParityTarget {
    pub capability_id: String,
    pub status: CapabilityStatus,
    pub extracted_feature_groups: Vec<String>,
    pub target_capability: String,
    pub target_tests: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Planned,
    Implemented,
    Superseded,
    ApprovedOutOfScope,
}

impl CapabilityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Implemented => "implemented",
            Self::Superseded => "superseded",
            Self::ApprovedOutOfScope => "approved_out_of_scope",
        }
    }

    pub const fn all() -> [Self; 4] {
        [
            Self::Planned,
            Self::Implemented,
            Self::Superseded,
            Self::ApprovedOutOfScope,
        ]
    }
}

pub fn render_capability_parity_report(map: &CapabilityParityMap) -> String {
    let mut counts: BTreeMap<&'static str, usize> = CapabilityStatus::all()
        .into_iter()
        .map(|status| (status.as_str(), 0))
        .collect();
    for row in &map.capabilities {
        *counts.entry(row.status.as_str()).or_insert(0) += 1;
    }

    let mut report = String::new();
    report.push_str("capability_parity_report\n");
    report.push_str(&format!("artifact_family={}\n", map.artifact_family));
    report.push_str(&format!(
        "capability_namespace={}\n",
        map.capability_namespace
    ));
    report.push_str(&format!("source_repo={}\n", map.provenance.source_repo));
    report.push_str(&format!(
        "package={}@{}\n",
        map.provenance.package_name, map.provenance.package_version
    ));
    report.push_str(&format!("commit={}\n", map.provenance.commit_sha));
    report.push_str(&format!("pinned_tree={}\n", map.provenance.pinned_tree_url));
    report.push_str("status_counts=");
    let mut first = true;
    for (status, count) in counts {
        if !first {
            report.push(',');
        }
        first = false;
        report.push_str(&format!("{status}:{count}"));
    }
    report.push('\n');

    for row in &map.capabilities {
        report.push_str(&format!(
            "{} status={} target_boundary={} target_tests={}\n",
            row.id,
            row.status.as_str(),
            row.target_boundary,
            row.target_tests.join(";")
        ));
    }

    report
}
