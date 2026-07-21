//! ADR-0515 Git-history-only corpus guard.
//!
//! The seven superseded CI/CD ADRs are absent from the current source corpus
//! and current-authority projections. Historical prose may still name their
//! identifiers as provenance; only current-authority surfaces are guarded.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

pub const ADR_0515_HISTORY_ONLY_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/adr-0515-history-only";
pub const ADR_0515_HISTORY_ONLY_CODE: &str = "adr_0515_history_only_drift";

const HISTORY_ONLY_ADRS: [&str; 7] = [
    "ADR-0124", "ADR-0349", "ADR-0359", "ADR-0361", "ADR-0511", "ADR-0513", "ADR-0514",
];

/// True when a producer citation edge names one of ADR-0515's seven
/// Git-history-only predecessors. These are classified provenance edges, not
/// phantom decisions; the current-authority surfaces are guarded separately.
pub fn is_adr_0515_history_only_citation(entry: &str) -> bool {
    entry
        .split_once('@')
        .is_some_and(|(id, _source)| HISTORY_ONLY_ADRS.contains(&id))
}

fn drift(key: impl Into<String>) -> Finding {
    let key = key.into();
    Finding::new(ADR_0515_HISTORY_ONLY_CODE, &key)
}

fn history_only_id_in_decision_path(path: &str) -> Option<&'static str> {
    let normalized = path.trim_start_matches("./").trim_start_matches('/');
    if !normalized.starts_with("docs/decisions/") || !normalized.ends_with(".md") {
        return None;
    }
    let file_name = normalized.rsplit('/').next()?;
    HISTORY_ONLY_ADRS
        .iter()
        .copied()
        .find(|id| file_name.starts_with(&format!("{id}-")))
}

fn history_only_id_in_path(path: &str) -> Option<&'static str> {
    history_only_id_in_decision_path(path)
}

pub fn evaluate_adr_0515_history_only(
    decision_source_paths: &BTreeSet<String>,
    adr_index_markdown: &str,
    decisions_json: &Value,
    root_hub: &Value,
    masterplan: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    for path in decision_source_paths {
        if let Some(id) = history_only_id_in_decision_path(path) {
            findings.insert(drift(format!("decision_source:{id}")));
        }
    }

    for id in HISTORY_ONLY_ADRS {
        let row_prefix = format!("| {id} |");
        if adr_index_markdown
            .lines()
            .any(|line| line.trim_start().starts_with(&row_prefix))
        {
            findings.insert(drift(format!("docs/ADR-INDEX.md:{id}")));
        }
    }

    match decisions_json.get("decisions").and_then(Value::as_array) {
        Some(decisions) => {
            for decision in decisions {
                if let Some(id) = decision
                    .get("adr")
                    .and_then(Value::as_str)
                    .filter(|id| HISTORY_ONLY_ADRS.contains(id))
                {
                    findings.insert(drift(format!("docs/machine-readable/decisions.json:{id}")));
                }
            }
        }
        None => {
            findings.insert(drift(
                "docs/machine-readable/decisions.json:<invalid-decisions>",
            ));
        }
    }

    match root_hub.get("entry_points").and_then(Value::as_object) {
        Some(entry_points) => {
            for (name, entry) in entry_points {
                if let Some(id) = entry
                    .get("current_path")
                    .and_then(Value::as_str)
                    .and_then(history_only_id_in_path)
                {
                    findings.insert(drift(format!("specs/root-hub-pointers.json:{name}:{id}")));
                }
            }
        }
        None => {
            findings.insert(drift("specs/root-hub-pointers.json:<invalid-entry_points>"));
        }
    }

    match masterplan
        .get("planning_authority")
        .and_then(|authority| authority.get("bound_adrs"))
        .and_then(Value::as_array)
    {
        Some(bound_adrs) => {
            for id in bound_adrs
                .iter()
                .filter_map(Value::as_str)
                .filter(|id| HISTORY_ONLY_ADRS.contains(id))
            {
                findings.insert(drift(format!(
                    "specs/masterplan.json:planning_authority.bound_adrs:{id}"
                )));
            }
        }
        None => {
            findings.insert(drift(
                "specs/masterplan.json:planning_authority.bound_adrs:<invalid>",
            ));
        }
    }

    if let Some(phases) = masterplan
        .get("ideal_production_roadmap")
        .and_then(|roadmap| roadmap.get("phases"))
        .and_then(Value::as_array)
    {
        for (position, phase) in phases.iter().enumerate() {
            let Some(authoring_adrs) = phase.get("authoring_adrs") else {
                continue;
            };
            let phase_id = match phase.get("id").and_then(Value::as_str) {
                Some(id) => id.to_owned(),
                None => position.to_string(),
            };
            match authoring_adrs.as_array() {
                Some(authoring_adrs) => {
                    for id in authoring_adrs
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|id| HISTORY_ONLY_ADRS.contains(id))
                    {
                        findings.insert(drift(format!(
                            "specs/masterplan.json:ideal_production_roadmap.phases[{phase_id}].authoring_adrs:{id}"
                        )));
                    }
                }
                None => {
                    findings.insert(drift(format!(
                        "specs/masterplan.json:ideal_production_roadmap.phases[{phase_id}].authoring_adrs:<invalid>"
                    )));
                }
            }
        }
    } else {
        findings.insert(drift(
            "specs/masterplan.json:ideal_production_roadmap.phases:<invalid>",
        ));
    }

    findings
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::json;

    use super::*;

    fn clean_sources() -> BTreeSet<String> {
        BTreeSet::from([
            "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md"
                .to_owned(),
        ])
    }

    fn clean_index() -> &'static str {
        "| ADR-0515 | Accepted | Current CI authority |\n"
    }

    fn clean_decisions() -> Value {
        json!({"decisions": [{"adr": "ADR-0515"}]})
    }

    fn clean_root_hub() -> Value {
        json!({
            "entry_points": {
                "adr_0515": {
                    "current_path": "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md"
                }
            }
        })
    }

    fn clean_masterplan() -> Value {
        json!({
            "planning_authority": {"bound_adrs": ["ADR-0515"]},
            "ideal_production_roadmap": {
                "phases": [{"id": "P-TOOLCHAIN", "authoring_adrs": ["ADR-0515"]}]
            }
        })
    }

    fn keys(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|finding| finding.key.clone()).collect()
    }

    #[test]
    fn restored_deleted_adr_source_fails_closed() {
        let mut sources = clean_sources();
        sources.insert(
            "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md".to_owned(),
        );

        let findings = evaluate_adr_0515_history_only(
            &sources,
            clean_index(),
            &clean_decisions(),
            &clean_root_hub(),
            &clean_masterplan(),
        );

        assert_eq!(keys(&findings), vec!["decision_source:ADR-0349".to_owned()]);
    }

    #[test]
    fn adr_index_residue_fails_closed() {
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            "| ADR-0349 | Superseded | Stale generated row |\n",
            &json!({"decisions": [{"adr": "ADR-0349"}]}),
            &clean_root_hub(),
            &clean_masterplan(),
        );

        assert_eq!(
            keys(&findings),
            vec![
                "docs/ADR-INDEX.md:ADR-0349".to_owned(),
                "docs/machine-readable/decisions.json:ADR-0349".to_owned(),
            ]
        );
    }

    #[test]
    fn root_hub_re_exposure_fails_closed() {
        let root_hub = json!({
            "entry_points": {
                "stale": {
                    "current_path": "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md"
                }
            }
        });
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            clean_index(),
            &clean_decisions(),
            &root_hub,
            &clean_masterplan(),
        );

        assert_eq!(
            keys(&findings),
            vec!["specs/root-hub-pointers.json:stale:ADR-0349".to_owned()]
        );
    }

    #[test]
    fn stale_active_masterplan_binding_fails_closed() {
        let masterplan = json!({
            "planning_authority": {"bound_adrs": ["ADR-0349"]},
            "ideal_production_roadmap": {
                "phases": [{"id": "P-TOOLCHAIN", "authoring_adrs": ["ADR-0513"]}]
            }
        });
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            clean_index(),
            &clean_decisions(),
            &clean_root_hub(),
            &masterplan,
        );

        assert_eq!(
            keys(&findings),
            vec![
                "specs/masterplan.json:ideal_production_roadmap.phases[P-TOOLCHAIN].authoring_adrs:ADR-0513".to_owned(),
                "specs/masterplan.json:planning_authority.bound_adrs:ADR-0349".to_owned(),
            ]
        );
    }

    #[test]
    fn history_only_citation_classifier_is_exact() {
        assert!(is_adr_0515_history_only_citation(
            "ADR-0349@docs/decisions/ADR-0515-current.md"
        ));
        assert!(!is_adr_0515_history_only_citation(
            "ADR-0397@specs/master-plan-sequencing.json"
        ));
        assert!(!is_adr_0515_history_only_citation("ADR-0349"));
    }
}
