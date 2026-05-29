//! Workflow Studio policy-preview domain.
//!
//! This crate evaluates a canonical `workflow_spec.v1` document against the
//! metadata Workflow Studio must disclose before save, publish, or activation:
//! Cedar policy-preview references, unsafe capability gates, LLM-authored draft
//! human-review gates, sensitive-data external-output blocking, and blast-radius
//! summary. The domain is pure and metadata-only; it does not resolve secrets,
//! execute workflow nodes, call Cedar, or persist audit rows.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use oya_workflow_studio_dsl_emitter_domain::{
    WorkflowSpec, WorkflowSpecEmitError, WorkflowSpecNodeKind,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAuthoringOrigin {
    HumanAuthored,
    LlmDraft,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPublishTransition {
    SaveDraft,
    Publish,
    Activate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowDataClass {
    Public,
    InternalOnly,
    Pii,
    Regulated,
    Secret,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAutonomyTier {
    T1Suggest,
    T2Draft,
    T3ExecuteWithApproval,
    T4Autonomous,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNodeRisk {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPolicyFindingSeverity {
    Info,
    Warning,
    Blocker,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPolicyFindingKind {
    SpecInvalid,
    MissingNodePolicyBinding,
    DuplicateNodePolicyBinding,
    MissingPolicyReference,
    CapabilityRequiresPolicyPreview,
    UnsafeCapabilityRequiresHumanReview,
    LlmDraftRequiresHumanReview,
    HighRiskActivationRequiresHumanReview,
    IrreversibleActionRequiresHumanReview,
    SensitiveExternalOutputBlocked,
    BlastRadiusDisclosure,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPolicyPreviewDecision {
    AllowPreview,
    RequiresHumanReview,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowNodePolicyBinding {
    pub node_id: String,                     // data_class: INTERNAL_ONLY
    pub data_class: WorkflowDataClass,       // data_class: PUBLIC
    pub autonomy_tier: WorkflowAutonomyTier, // data_class: PUBLIC
    pub risk: WorkflowNodeRisk,              // data_class: PUBLIC
    pub policy_ref: Option<String>, // data_class: INTERNAL_ONLY; Cedar policy metadata ref only
    pub connector_scope_ref: Option<String>, // data_class: INTERNAL_ONLY; credential/scope metadata ref only
    pub external_output: bool,               // data_class: PUBLIC
    pub irreversible: bool,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicyPreviewInput {
    pub spec: WorkflowSpec,                        // data_class: INTERNAL_ONLY
    pub authoring_origin: WorkflowAuthoringOrigin, // data_class: PUBLIC
    pub requested_transition: WorkflowPublishTransition, // data_class: PUBLIC
    pub human_review_completed: bool,              // data_class: PUBLIC
    pub node_bindings: Vec<WorkflowNodePolicyBinding>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                // data_class: INTERNAL_ONLY; metadata refs only
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicyFinding {
    pub kind: WorkflowPolicyFindingKind,         // data_class: PUBLIC
    pub severity: WorkflowPolicyFindingSeverity, // data_class: PUBLIC
    pub node_id: Option<String>,                 // data_class: INTERNAL_ONLY
    pub message: String,                         // data_class: PUBLIC; no raw workflow payloads
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBlastRadiusSummary {
    pub total_nodes: usize,                                  // data_class: PUBLIC
    pub capability_call_nodes: usize,                        // data_class: PUBLIC
    pub high_risk_nodes: usize,                              // data_class: PUBLIC
    pub sensitive_data_nodes: usize,                         // data_class: PUBLIC
    pub external_output_nodes: usize,                        // data_class: PUBLIC
    pub irreversible_nodes: usize,                           // data_class: PUBLIC
    pub autonomy_tier_ceiling: Option<WorkflowAutonomyTier>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicyPreviewReport {
    pub decision: WorkflowPolicyPreviewDecision, // data_class: PUBLIC
    pub findings: Vec<WorkflowPolicyFinding>,    // data_class: PUBLIC
    pub blast_radius: WorkflowBlastRadiusSummary, // data_class: PUBLIC
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY; metadata refs only
}

impl WorkflowNodePolicyBinding {
    pub fn new(
        node_id: impl Into<String>,
        data_class: WorkflowDataClass,
        autonomy_tier: WorkflowAutonomyTier,
        risk: WorkflowNodeRisk,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            data_class,
            autonomy_tier,
            risk,
            policy_ref: None,
            connector_scope_ref: None,
            external_output: false,
            irreversible: false,
        }
    }

    pub fn with_data_class(mut self, data_class: WorkflowDataClass) -> Self {
        self.data_class = data_class;
        self
    }

    pub fn with_autonomy_tier(mut self, autonomy_tier: WorkflowAutonomyTier) -> Self {
        self.autonomy_tier = autonomy_tier;
        self
    }

    pub fn with_risk(mut self, risk: WorkflowNodeRisk) -> Self {
        self.risk = risk;
        self
    }

    pub fn with_policy_ref(mut self, policy_ref: impl Into<String>) -> Self {
        self.policy_ref = Some(policy_ref.into());
        self
    }

    pub fn with_connector_scope_ref(mut self, connector_scope_ref: impl Into<String>) -> Self {
        self.connector_scope_ref = Some(connector_scope_ref.into());
        self
    }

    pub fn with_external_output(mut self, external_output: bool) -> Self {
        self.external_output = external_output;
        self
    }

    pub fn with_irreversible(mut self, irreversible: bool) -> Self {
        self.irreversible = irreversible;
        self
    }
}

impl WorkflowPolicyPreviewInput {
    pub fn new(
        spec: WorkflowSpec,
        authoring_origin: WorkflowAuthoringOrigin,
        requested_transition: WorkflowPublishTransition,
        human_review_completed: bool,
        node_bindings: Vec<WorkflowNodePolicyBinding>,
        evidence_refs: Vec<String>,
    ) -> Self {
        Self {
            spec,
            authoring_origin,
            requested_transition,
            human_review_completed,
            node_bindings,
            evidence_refs,
        }
    }
}

impl WorkflowPolicyPreviewReport {
    pub fn finding_kinds(&self) -> Vec<WorkflowPolicyFindingKind> {
        self.findings.iter().map(|finding| finding.kind).collect()
    }
}

pub fn preview_workflow_policy(input: &WorkflowPolicyPreviewInput) -> WorkflowPolicyPreviewReport {
    let binding_by_node = node_binding_index(&input.node_bindings);
    let mut findings = Vec::new();

    for duplicate_node_id in duplicate_binding_node_ids(&input.node_bindings) {
        findings.push(finding(
            WorkflowPolicyFindingKind::DuplicateNodePolicyBinding,
            WorkflowPolicyFindingSeverity::Blocker,
            Some(duplicate_node_id),
            "node has duplicate policy-preview metadata bindings",
        ));
    }

    if let Err(error) = input.spec.validate() {
        findings.push(finding(
            WorkflowPolicyFindingKind::SpecInvalid,
            WorkflowPolicyFindingSeverity::Blocker,
            None,
            format_spec_error(error),
        ));
    }

    let blast_radius = summarize_blast_radius(input, &binding_by_node);

    for node in &input.spec.nodes {
        let Some(binding) = binding_by_node.get(node.id.as_str()) else {
            findings.push(finding(
                WorkflowPolicyFindingKind::MissingNodePolicyBinding,
                WorkflowPolicyFindingSeverity::Blocker,
                Some(node.id.clone()),
                "node is missing policy-preview metadata binding",
            ));
            continue;
        };

        if node.kind == WorkflowSpecNodeKind::CapabilityCall {
            if has_blank_policy_ref(binding) {
                findings.push(finding(
                    WorkflowPolicyFindingKind::MissingPolicyReference,
                    WorkflowPolicyFindingSeverity::Blocker,
                    Some(node.id.clone()),
                    "capability-call node lacks Cedar policy-preview reference",
                ));
            } else {
                findings.push(finding(
                    WorkflowPolicyFindingKind::CapabilityRequiresPolicyPreview,
                    WorkflowPolicyFindingSeverity::Info,
                    Some(node.id.clone()),
                    "capability-call node has a Cedar policy-preview reference",
                ));
            }

            if is_unsafe_capability_binding(binding) && !input.human_review_completed {
                findings.push(finding(
                    WorkflowPolicyFindingKind::UnsafeCapabilityRequiresHumanReview,
                    WorkflowPolicyFindingSeverity::Warning,
                    Some(node.id.clone()),
                    "unsafe capability requires visible human-review gate before activation",
                ));
            }
        }

        if is_sensitive(binding.data_class) && binding.external_output {
            findings.push(finding(
                WorkflowPolicyFindingKind::SensitiveExternalOutputBlocked,
                WorkflowPolicyFindingSeverity::Blocker,
                Some(node.id.clone()),
                "sensitive data may not be sent to an external output by preview foundation policy",
            ));
        }

        if binding.risk == WorkflowNodeRisk::High && !input.human_review_completed {
            findings.push(finding(
                WorkflowPolicyFindingKind::HighRiskActivationRequiresHumanReview,
                WorkflowPolicyFindingSeverity::Warning,
                Some(node.id.clone()),
                "high-risk node requires human review before activation",
            ));
        }

        if binding.irreversible && !input.human_review_completed {
            findings.push(finding(
                WorkflowPolicyFindingKind::IrreversibleActionRequiresHumanReview,
                WorkflowPolicyFindingSeverity::Warning,
                Some(node.id.clone()),
                "irreversible action requires human review before activation",
            ));
        }
    }

    if input.authoring_origin == WorkflowAuthoringOrigin::LlmDraft && !input.human_review_completed
    {
        findings.push(finding(
            WorkflowPolicyFindingKind::LlmDraftRequiresHumanReview,
            WorkflowPolicyFindingSeverity::Warning,
            None,
            "LLM-authored workflow specs require human review before save, publish, or activation",
        ));
    }

    findings.push(finding(
        WorkflowPolicyFindingKind::BlastRadiusDisclosure,
        WorkflowPolicyFindingSeverity::Info,
        None,
        "blast-radius summary computed for authoring preview",
    ));
    findings.sort_by(|left, right| {
        (
            left.severity,
            left.kind,
            left.node_id.as_deref().unwrap_or_default(),
            left.message.as_str(),
        )
            .cmp(&(
                right.severity,
                right.kind,
                right.node_id.as_deref().unwrap_or_default(),
                right.message.as_str(),
            ))
    });

    WorkflowPolicyPreviewReport {
        decision: decide(&findings),
        findings,
        blast_radius,
        evidence_refs: canonical_evidence_refs(&input.evidence_refs),
    }
}

fn node_binding_index(
    bindings: &[WorkflowNodePolicyBinding],
) -> BTreeMap<&str, &WorkflowNodePolicyBinding> {
    let mut index = BTreeMap::new();
    for binding in bindings {
        index.entry(binding.node_id.as_str()).or_insert(binding);
    }
    index
}

fn duplicate_binding_node_ids(bindings: &[WorkflowNodePolicyBinding]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for binding in bindings {
        if !seen.insert(binding.node_id.as_str()) {
            duplicates.insert(binding.node_id.clone());
        }
    }
    duplicates.into_iter().collect()
}

fn summarize_blast_radius(
    input: &WorkflowPolicyPreviewInput,
    binding_by_node: &BTreeMap<&str, &WorkflowNodePolicyBinding>,
) -> WorkflowBlastRadiusSummary {
    let mut summary = WorkflowBlastRadiusSummary {
        total_nodes: input.spec.nodes.len(),
        ..WorkflowBlastRadiusSummary::default()
    };

    for node in &input.spec.nodes {
        if node.kind == WorkflowSpecNodeKind::CapabilityCall {
            summary.capability_call_nodes += 1;
        }
        if let Some(binding) = binding_by_node.get(node.id.as_str()) {
            if binding.risk == WorkflowNodeRisk::High {
                summary.high_risk_nodes += 1;
            }
            if is_sensitive(binding.data_class) {
                summary.sensitive_data_nodes += 1;
            }
            if binding.external_output {
                summary.external_output_nodes += 1;
            }
            if binding.irreversible {
                summary.irreversible_nodes += 1;
            }
            summary.autonomy_tier_ceiling = Some(match summary.autonomy_tier_ceiling {
                Some(current) => current.max(binding.autonomy_tier),
                None => binding.autonomy_tier,
            });
        }
    }

    summary
}

fn decide(findings: &[WorkflowPolicyFinding]) -> WorkflowPolicyPreviewDecision {
    if findings
        .iter()
        .any(|finding| finding.severity == WorkflowPolicyFindingSeverity::Blocker)
    {
        return WorkflowPolicyPreviewDecision::Blocked;
    }

    if findings
        .iter()
        .any(|finding| finding.severity == WorkflowPolicyFindingSeverity::Warning)
    {
        return WorkflowPolicyPreviewDecision::RequiresHumanReview;
    }

    WorkflowPolicyPreviewDecision::AllowPreview
}

fn finding(
    kind: WorkflowPolicyFindingKind,
    severity: WorkflowPolicyFindingSeverity,
    node_id: Option<String>,
    message: impl Into<String>,
) -> WorkflowPolicyFinding {
    WorkflowPolicyFinding {
        kind,
        severity,
        node_id,
        message: message.into(),
    }
}

fn has_blank_policy_ref(binding: &WorkflowNodePolicyBinding) -> bool {
    binding
        .policy_ref
        .as_deref()
        .is_none_or(|policy_ref| policy_ref.trim().is_empty())
}

fn is_unsafe_capability_binding(binding: &WorkflowNodePolicyBinding) -> bool {
    binding.risk != WorkflowNodeRisk::Low
        || binding.autonomy_tier >= WorkflowAutonomyTier::T3ExecuteWithApproval
        || is_sensitive(binding.data_class)
        || binding.external_output
        || binding.irreversible
}

fn is_sensitive(data_class: WorkflowDataClass) -> bool {
    matches!(
        data_class,
        WorkflowDataClass::Pii | WorkflowDataClass::Regulated | WorkflowDataClass::Secret
    )
}

fn canonical_evidence_refs(refs: &[String]) -> Vec<String> {
    let mut canonical = BTreeSet::new();
    for evidence_ref in refs {
        let trimmed = evidence_ref.trim();
        if !trimmed.is_empty() {
            canonical.insert(trimmed.to_string());
        }
    }
    canonical.into_iter().collect()
}

fn format_spec_error(error: WorkflowSpecEmitError) -> String {
    format!("workflow_spec.v1 validation failed: {error:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_workflow_studio_dsl_emitter_domain::{
        WorkflowSpec, WorkflowSpecEdge, WorkflowSpecNode, WorkflowSpecNodeKind,
    };

    fn benign_spec() -> WorkflowSpec {
        WorkflowSpec::new(
            "ten_acme",
            "wfd_onboarding",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_start", WorkflowSpecNodeKind::Http, "Start"),
                WorkflowSpecNode::new("wfn_prepare", WorkflowSpecNodeKind::Transform, "Prepare"),
                WorkflowSpecNode::new("wfn_review", WorkflowSpecNodeKind::HumanReview, "Review"),
            ],
            vec![
                WorkflowSpecEdge::new("wfn_start", "wfn_prepare", None),
                WorkflowSpecEdge::new("wfn_prepare", "wfn_review", None),
            ],
        )
    }

    fn capability_spec() -> WorkflowSpec {
        WorkflowSpec::new(
            "ten_acme",
            "wfd_vendor_sync",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_start", WorkflowSpecNodeKind::Http, "Start"),
                WorkflowSpecNode::new(
                    "wfn_capability",
                    WorkflowSpecNodeKind::CapabilityCall,
                    "Sync vendor",
                ),
            ],
            vec![WorkflowSpecEdge::new("wfn_start", "wfn_capability", None)],
        )
    }

    fn binding(node_id: &str) -> WorkflowNodePolicyBinding {
        WorkflowNodePolicyBinding::new(
            node_id,
            WorkflowDataClass::InternalOnly,
            WorkflowAutonomyTier::T1Suggest,
            WorkflowNodeRisk::Low,
        )
    }

    #[test]
    fn allows_benign_workflow_with_complete_policy_bindings() {
        let input = WorkflowPolicyPreviewInput::new(
            benign_spec(),
            WorkflowAuthoringOrigin::HumanAuthored,
            WorkflowPublishTransition::SaveDraft,
            false,
            vec![
                binding("wfn_start"),
                binding("wfn_prepare"),
                binding("wfn_review"),
            ],
            vec!["cedar-preview:baseline".to_string()],
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(report.decision, WorkflowPolicyPreviewDecision::AllowPreview);
        assert_eq!(report.blast_radius.total_nodes, 3);
        assert_eq!(report.blast_radius.high_risk_nodes, 0);
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::BlastRadiusDisclosure)
        );
    }

    #[test]
    fn requires_human_review_for_llm_authored_high_risk_capability() {
        let capability = binding("wfn_capability")
            .with_data_class(WorkflowDataClass::Regulated)
            .with_autonomy_tier(WorkflowAutonomyTier::T3ExecuteWithApproval)
            .with_risk(WorkflowNodeRisk::High)
            .with_policy_ref("cedar:workflow-studio:vendor-sync")
            .with_external_output(false)
            .with_irreversible(true);
        let input = WorkflowPolicyPreviewInput::new(
            capability_spec(),
            WorkflowAuthoringOrigin::LlmDraft,
            WorkflowPublishTransition::Activate,
            false,
            vec![binding("wfn_start"), capability],
            vec!["audit:event-2".to_string(), "audit:event-1".to_string()],
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(
            report.decision,
            WorkflowPolicyPreviewDecision::RequiresHumanReview
        );
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::LlmDraftRequiresHumanReview)
        );
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::UnsafeCapabilityRequiresHumanReview)
        );
        assert_eq!(report.blast_radius.high_risk_nodes, 1);
        assert_eq!(report.blast_radius.capability_call_nodes, 1);
    }

    #[test]
    fn blocks_missing_policy_binding_for_node() {
        let input = WorkflowPolicyPreviewInput::new(
            benign_spec(),
            WorkflowAuthoringOrigin::HumanAuthored,
            WorkflowPublishTransition::SaveDraft,
            false,
            vec![binding("wfn_start"), binding("wfn_review")],
            Vec::new(),
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(report.decision, WorkflowPolicyPreviewDecision::Blocked);
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::MissingNodePolicyBinding)
        );
    }

    #[test]
    fn blocks_duplicate_policy_binding_for_node() {
        let input = WorkflowPolicyPreviewInput::new(
            benign_spec(),
            WorkflowAuthoringOrigin::HumanAuthored,
            WorkflowPublishTransition::SaveDraft,
            false,
            vec![
                binding("wfn_start"),
                binding("wfn_prepare"),
                binding("wfn_prepare").with_risk(WorkflowNodeRisk::High),
                binding("wfn_review"),
            ],
            Vec::new(),
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(report.decision, WorkflowPolicyPreviewDecision::Blocked);
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::DuplicateNodePolicyBinding)
        );
    }

    #[test]
    fn blocks_sensitive_data_to_external_output() {
        let capability = binding("wfn_capability")
            .with_data_class(WorkflowDataClass::Pii)
            .with_policy_ref("cedar:workflow-studio:vendor-sync")
            .with_external_output(true);
        let input = WorkflowPolicyPreviewInput::new(
            capability_spec(),
            WorkflowAuthoringOrigin::HumanAuthored,
            WorkflowPublishTransition::Publish,
            true,
            vec![binding("wfn_start"), capability],
            Vec::new(),
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(report.decision, WorkflowPolicyPreviewDecision::Blocked);
        assert!(
            report
                .finding_kinds()
                .contains(&WorkflowPolicyFindingKind::SensitiveExternalOutputBlocked)
        );
        assert_eq!(report.blast_radius.sensitive_data_nodes, 1);
        assert_eq!(report.blast_radius.external_output_nodes, 1);
    }

    #[test]
    fn evidence_refs_are_sorted_and_deduplicated_metadata_refs() {
        let input = WorkflowPolicyPreviewInput::new(
            benign_spec(),
            WorkflowAuthoringOrigin::HumanAuthored,
            WorkflowPublishTransition::SaveDraft,
            false,
            vec![
                binding("wfn_start"),
                binding("wfn_prepare"),
                binding("wfn_review"),
            ],
            vec![
                " audit:event-2 ".to_string(),
                "audit:event-1".to_string(),
                "audit:event-2".to_string(),
                "".to_string(),
            ],
        );

        let report = preview_workflow_policy(&input);

        assert_eq!(
            report.evidence_refs,
            vec!["audit:event-1".to_string(), "audit:event-2".to_string()]
        );
    }
}
