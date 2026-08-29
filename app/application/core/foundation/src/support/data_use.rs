//! Data-use evaluation and denial reporting for invocations.

use crate::*;

pub(crate) fn evaluate_invocation_data_use(
    capability: &Capability,
    request: &CapabilityInvocationRequest,
    consent_scope: Option<&ConsentScope>,
) -> Result<Purpose, InvocationDataUseDenial> {
    let effective_purpose = effective_invocation_purpose(capability, request.purpose)?;
    for privacy_data_class in capability.touched_privacy_data_classes() {
        let data_class = privacy_data_class.data_class();
        if let Err(reason) = evaluate_data_use(DataUseAttributes {
            purpose: effective_purpose,
            data_classification: DataClassification::from(*privacy_data_class),
            subject_class: request.subject_class,
        }) {
            return Err(InvocationDataUseDenial {
                effective_purpose,
                denied_data_class: Some(data_class),
                reason: data_use_denial_reason_label(reason),
            });
        }
        if data_class != DataClass::InternalOnly
            && !consent_scope.is_some_and(|scope| {
                scope.allows_privacy_data_class(effective_purpose, *privacy_data_class)
            })
        {
            return Err(InvocationDataUseDenial {
                effective_purpose,
                denied_data_class: Some(data_class),
                reason: "missing_purpose_bound_data_use_grant",
            });
        }
    }
    Ok(effective_purpose)
}

pub(crate) fn effective_invocation_purpose(
    capability: &Capability,
    requested_purpose: Purpose,
) -> Result<Purpose, InvocationDataUseDenial> {
    if matches!(
        capability.action,
        CapabilityAction::AdsBid | CapabilityAction::AdsBudgetAdjust
    ) {
        if requested_purpose != Purpose::AdsTargeting {
            return Err(InvocationDataUseDenial {
                effective_purpose: Purpose::AdsTargeting,
                denied_data_class: None,
                reason: "underdeclared_ads_purpose",
            });
        }
        return Ok(Purpose::AdsTargeting);
    }
    Ok(requested_purpose)
}

pub(crate) fn data_use_denial_reason_label(reason: DataUseDenialReason) -> &'static str {
    match reason {
        DataUseDenialReason::HardDeniedDataClass => "hard_denied_data_class",
        DataUseDenialReason::MinorSubjectAds => "minor_subject_ads",
    }
}

pub(crate) fn data_use_denial_fields(
    request: &CapabilityInvocationRequest,
    capability: &Capability,
    denial: &InvocationDataUseDenial,
    capability_invoke_audit_hash: String,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "capability_invoke_audit_event_hash".to_string(),
            capability_invoke_audit_hash,
        ),
        (
            "data_use_denial_reason".to_string(),
            denial.reason.to_string(),
        ),
        (
            "consent_result".to_string(),
            if denial.reason == "missing_purpose_bound_data_use_grant" {
                "missing"
            } else {
                "not_evaluated"
            }
            .to_string(),
        ),
        (
            "requested_purpose".to_string(),
            format!("{:?}", request.purpose),
        ),
        (
            "effective_purpose".to_string(),
            format!("{:?}", denial.effective_purpose),
        ),
        (
            "subject_class".to_string(),
            format!("{:?}", request.subject_class),
        ),
        (
            "denied_data_class".to_string(),
            denial
                .denied_data_class
                .map(|data_class| data_class.label().to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
        (
            "data_classes".to_string(),
            capability_record_data_class_labels(capability),
        ),
    ])
}
