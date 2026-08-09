// forms µservice — CI-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce CI runner scope on Forms read operations during gate validation;
//          CI must read form metadata + response aggregates + audit-chain seal status
//          across tenants for governance lanes, but is bounded to read-only metadata
//          and forbidden from tenant draft contents OR PII.
// Owner:   axis-foundry + ops-security
// References: threat-model.md §"T-S-02" (Forms SPIFFE impersonation)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI runner reads Forms metadata for governance lanes
// =============================================================================
//
// Reserved CI tenant: oya-forms-ci.

permit (
  principal == ServiceAccount::"oya-forms-ci",
  action in [
    Action::"read_form_metadata",
    Action::"read_response_aggregate_count",
    Action::"read_audit_chain_seal_status",
    Action::"read_forms_self_slo",
    Action::"read_governance_lane_status",
    Action::"read_form_template_signature_status",
    Action::"read_captcha_provider_health",
    Action::"read_webhook_dlq_depth",
    Action::"read_bulk_distribute_queue_depth"
  ],
  resource
);

// =============================================================================
// FORBID — CI never reads tenant draft contents OR PII
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-forms-ci",
  action in [
    Action::"read_form_definition",
    Action::"read_response_redacted",
    Action::"read_response_with_pii",
    Action::"export_responses_csv",
    Action::"export_responses_xlsx",
    Action::"export_responses_json",
    Action::"invoke_ai_form_build",
    Action::"read_response_metadata"
  ],
  resource
);

// =============================================================================
// FORBID — CI never writes; refuses any create/publish/distribute
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-forms-ci",
  action in [
    Action::"create_form",
    Action::"edit_form_draft",
    Action::"publish_form",
    Action::"archive_form",
    Action::"submit_response",
    Action::"bulk_distribute_form_email",
    Action::"bulk_distribute_form_messenger",
    Action::"bulk_distribute_form_sms",
    Action::"configure_webhook",
    Action::"replay_webhook_dlq"
  ],
  resource
);

// =============================================================================
// FORBID — CI cannot read SECRET-class
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-forms-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
