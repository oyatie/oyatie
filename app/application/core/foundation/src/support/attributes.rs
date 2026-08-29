//! Authorization-attribute and evidence-field assembly.

use crate::*;

pub(crate) fn invocation_authorization_attributes(
    request: &CapabilityInvocationRequest,
    capability: &Capability,
    principal_autonomy_ceiling: AutonomyTier,
    autonomy_decision: &AutonomyDecision,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::from([
        ("tenant_id".to_string(), request.tenant_id.clone()),
        ("purpose".to_string(), format!("{:?}", request.purpose)),
        (
            "subject_class".to_string(),
            format!("{:?}", request.subject_class),
        ),
        (
            "required_tier".to_string(),
            format!("{:?}", capability.required_tier),
        ),
        (
            "principal_autonomy_ceiling".to_string(),
            format!("{principal_autonomy_ceiling:?}"),
        ),
        (
            "tenant_configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.tenant_configured_ceiling),
        ),
        (
            "principal_ceiling".to_string(),
            format!("{:?}", autonomy_decision.principal_ceiling),
        ),
        (
            "capability_required_cap".to_string(),
            format!("{:?}", autonomy_decision.capability_required_cap),
        ),
        (
            "agentic_ads_cap".to_string(),
            format!("{:?}", autonomy_decision.agentic_ads_cap),
        ),
        (
            "vertical_pack_cap".to_string(),
            format!("{:?}", autonomy_decision.vertical_pack_cap),
        ),
        (
            "subject_class_cap".to_string(),
            format!("{:?}", autonomy_decision.subject_class_cap),
        ),
        (
            "denial_threshold".to_string(),
            format!("{:?}", autonomy_decision.denial_threshold),
        ),
        (
            "effective_ceiling".to_string(),
            format!("{:?}", autonomy_decision.effective_ceiling),
        ),
        (
            "autonomy_verdict".to_string(),
            format!("{:?}", autonomy_decision.verdict),
        ),
        (
            "blocking_cap_source".to_string(),
            autonomy_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "blocking_cap_reason".to_string(),
            autonomy_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "lowering_cap_source".to_string(),
            autonomy_decision.lowering_cap_source.as_str().to_string(),
        ),
        (
            "lowering_cap_reason".to_string(),
            autonomy_decision.lowering_cap_reason.as_str().to_string(),
        ),
        (
            "data_classes".to_string(),
            capability_record_data_class_labels(capability),
        ),
    ]);
    append_break_glass_authorization_attributes(
        &mut attributes,
        break_glass,
        pre_break_glass_decision,
    );
    attributes
}

pub(crate) fn append_break_glass_authorization_attributes(
    fields: &mut BTreeMap<String, String>,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
) {
    fields.insert(
        "break_glass_applied".to_string(),
        break_glass.is_some().to_string(),
    );
    if let Some(break_glass) = break_glass {
        fields.insert("break_glass_id".to_string(), break_glass.id.value.clone());
        fields.insert(
            "break_glass_requested_tier".to_string(),
            format!("{:?}", break_glass.requested_tier.value),
        );
        fields.insert(
            "break_glass_permitted_tier".to_string(),
            format!("{:?}", break_glass.permitted_tier.value),
        );
        fields.insert(
            "break_glass_approval_quorum".to_string(),
            format!("{:?}", break_glass.approval_quorum.value),
        );
        fields.insert(
            "break_glass_expires_at_epoch_days".to_string(),
            break_glass.expires_at_epoch_days.value.to_string(),
        );
    }
    if let Some(pre_break_glass_decision) = pre_break_glass_decision {
        fields.insert(
            "pre_break_glass_decision".to_string(),
            autonomy_decision_label(pre_break_glass_decision).to_string(),
        );
        fields.insert(
            "pre_break_glass_effective_ceiling".to_string(),
            format!("{:?}", pre_break_glass_decision.effective_ceiling),
        );
        fields.insert(
            "pre_break_glass_denial_threshold".to_string(),
            format!("{:?}", pre_break_glass_decision.denial_threshold),
        );
        fields.insert(
            "pre_break_glass_blocking_cap_source".to_string(),
            pre_break_glass_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        );
        fields.insert(
            "pre_break_glass_blocking_cap_reason".to_string(),
            pre_break_glass_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        );
    }
}

pub(crate) fn append_break_glass_evidence_fields(
    fields: &mut BTreeMap<String, String>,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
    break_glass_invoke_audit_hash: Option<&str>,
) {
    append_break_glass_authorization_attributes(fields, break_glass, pre_break_glass_decision);
    if let Some(break_glass_invoke_audit_hash) = break_glass_invoke_audit_hash {
        fields.insert(
            "break_glass_invoke_audit_event_hash".to_string(),
            break_glass_invoke_audit_hash.to_string(),
        );
    }
}

pub(crate) fn autonomy_decision_fields(
    autonomy_decision: &AutonomyDecision,
    autonomy_audit_hash: &str,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "audit_event_hash".to_string(),
            autonomy_audit_hash.to_string(),
        ),
        (
            "autonomy_audit_event_hash".to_string(),
            autonomy_audit_hash.to_string(),
        ),
        ("tenant_id".to_string(), autonomy_decision.tenant_id.clone()),
        (
            "capability_id".to_string(),
            autonomy_decision.capability_id.clone(),
        ),
        (
            "configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.configured_ceiling),
        ),
        (
            "tenant_configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.tenant_configured_ceiling),
        ),
        (
            "principal_ceiling".to_string(),
            format!("{:?}", autonomy_decision.principal_ceiling),
        ),
        (
            "capability_required_cap".to_string(),
            format!("{:?}", autonomy_decision.capability_required_cap),
        ),
        (
            "agentic_ads_cap".to_string(),
            format!("{:?}", autonomy_decision.agentic_ads_cap),
        ),
        (
            "vertical_pack_cap".to_string(),
            format!("{:?}", autonomy_decision.vertical_pack_cap),
        ),
        (
            "subject_class".to_string(),
            format!("{:?}", autonomy_decision.subject_class),
        ),
        (
            "subject_class_cap".to_string(),
            format!("{:?}", autonomy_decision.subject_class_cap),
        ),
        (
            "denial_threshold".to_string(),
            format!("{:?}", autonomy_decision.denial_threshold),
        ),
        (
            "effective_ceiling".to_string(),
            format!("{:?}", autonomy_decision.effective_ceiling),
        ),
        (
            "required_tier".to_string(),
            format!("{:?}", autonomy_decision.required_tier),
        ),
        (
            "decision".to_string(),
            autonomy_decision_label(autonomy_decision).to_string(),
        ),
        (
            "verdict".to_string(),
            format!("{:?}", autonomy_decision.verdict),
        ),
        (
            "blocking_cap_source".to_string(),
            autonomy_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "blocking_cap_reason".to_string(),
            autonomy_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "lowering_cap_source".to_string(),
            autonomy_decision.lowering_cap_source.as_str().to_string(),
        ),
        (
            "lowering_cap_reason".to_string(),
            autonomy_decision.lowering_cap_reason.as_str().to_string(),
        ),
    ])
}
