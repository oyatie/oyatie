// forms µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/forms/policy/schema.cedarschema
//
// Purpose: enforce per-tenant scope on form authoring, response capture, response export,
//          bulk distribution, AI-form-build, and all other Forms operations.
// Owner:   ops-security + axis-forms
// Audit:   every deny emits oya_forms_unauthorized_attempt_total
// References: threat-model.md §"T-I-01" (cross-tenant response leak) + §"T-T-01" (skip-logic bypass) +
//             §"T-E-01" (within-tenant elevation)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action refused unless explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator authors + reads own tenant's forms
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_form_definition",
    Action::"list_forms",
    Action::"read_form_version_history",
    Action::"read_form_response_aggregate",
    Action::"read_form_analytics",
    Action::"read_form_template_catalog"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator creates, edits, publishes, archives forms
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"create_form",
    Action::"edit_form_draft",
    Action::"publish_form",
    Action::"archive_form",
    Action::"unarchive_form",
    Action::"clone_form",
    Action::"install_form_template"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_forms_seat_entitlement &&
  principal.has_forms_seat_entitlement == true
};

// =============================================================================
// PERMIT 3 — tenant operator reads own tenant's responses (PII-redacted by default)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_response_redacted",
    Action::"list_responses",
    Action::"read_response_metadata"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator reads response with PII (entitlement-gated)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"read_response_with_pii",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_pii_read_entitlement &&
  principal.has_pii_read_entitlement == true
};

// =============================================================================
// PERMIT 5 — tenant operator exports own tenant's responses
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"export_responses_csv",
    Action::"export_responses_xlsx",
    Action::"export_responses_json",
    Action::"export_to_sheets_bridge"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_export_entitlement &&
  principal.has_export_entitlement == true
};

// =============================================================================
// PERMIT 6 — tenant operator bulk-distributes own tenant's form
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"bulk_distribute_form_email",
    Action::"bulk_distribute_form_messenger",
    Action::"bulk_distribute_form_sms"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_bulk_distribute_entitlement &&
  principal.has_bulk_distribute_entitlement == true
};

// =============================================================================
// PERMIT 7 — tenant operator invokes AI-form-build (consent + tenant_class-gated)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"invoke_ai_form_build",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has ai_build_consent &&
  principal.ai_build_consent == true &&
  principal has has_ai_form_build_entitlement &&
  principal.has_ai_form_build_entitlement == true
};

// =============================================================================
// PERMIT 8 — anonymous + authenticated submitter submits a response
// =============================================================================

permit (
  principal,
  action in [
    Action::"submit_response",
    Action::"validate_field_live",
    Action::"upload_attachment",
    Action::"esign_response"
  ],
  resource in PublishedForm::?f
)
when {
  resource has is_published &&
  resource.is_published == true &&
  resource has accepting_submissions &&
  resource.accepting_submissions == true
};

// =============================================================================
// PERMIT 9 — tenant operator configures webhook delivery (purpose-bound)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"configure_webhook",
    Action::"replay_webhook_dlq"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has webhook_purpose &&
  principal has declared_purposes &&
  resource.webhook_purpose in principal.declared_purposes
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defense-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_form_definition",
    Action::"list_forms",
    Action::"create_form",
    Action::"edit_form_draft",
    Action::"publish_form",
    Action::"archive_form",
    Action::"unarchive_form",
    Action::"clone_form",
    Action::"read_response_redacted",
    Action::"read_response_with_pii",
    Action::"list_responses",
    Action::"read_response_metadata",
    Action::"export_responses_csv",
    Action::"export_responses_xlsx",
    Action::"export_responses_json",
    Action::"export_to_sheets_bridge",
    Action::"bulk_distribute_form_email",
    Action::"bulk_distribute_form_messenger",
    Action::"bulk_distribute_form_sms",
    Action::"invoke_ai_form_build",
    Action::"configure_webhook",
    Action::"replay_webhook_dlq",
    Action::"read_form_response_aggregate",
    Action::"read_form_analytics"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — never expose SECRET-class data
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — PHI requires HIPAA pack + BAA
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "PHI" &&
  ((!(principal has pack)) || principal.pack != "pack-us-healthcare")
};

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "PHI" &&
  ((!(principal has baa_signed)) || principal.baa_signed != true)
};

// =============================================================================
// FORBID — GDPR Art. 9 special-category fields require explicit consent + DPIA
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SENSITIVE_GDPR_ART9" &&
  ((!(principal has art9_explicit_consent)) || principal.art9_explicit_consent != true)
};

// =============================================================================
// FORBID — production-tier publish requires Cedar policy preview acknowledgment
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action == Action::"publish_form",
  resource in TenantData::?r
)
when {
  resource has environment &&
  resource.environment == "production" &&
  principal has has_policy_preview_acknowledged &&
  principal.has_policy_preview_acknowledged != true
};

// =============================================================================
// FORBID — AI-form-build T2 cross-µservice gate (per ADR-FORMS-0005)
// =============================================================================

forbid (
  principal,
  action == Action::"invoke_ai_form_build_t2_cross_microservice",
  resource
)
unless {
  principal has has_t2_cross_microservice_consent &&
  principal.has_t2_cross_microservice_consent == true &&
  resource has destination_microservice &&
  principal has t2_cross_destinations_allowlist &&
  resource.destination_microservice in principal.t2_cross_destinations_allowlist
};

// =============================================================================
// FORBID — Forms NEVER executes workflows; only triggers via adapter
// =============================================================================

forbid (
  principal,
  action in [
    Action::"execute_workflow_step",
    Action::"start_workflow_run_direct"
  ],
  resource
);

// =============================================================================
// FORBID — reCAPTCHA usage in pack-eu, pack-kr, pack-us-healthcare (privacy posture)
// =============================================================================

forbid (
  principal,
  action == Action::"configure_recaptcha_provider",
  resource
)
when {
  resource has pack &&
  resource.pack in ["pack-eu", "pack-kr", "pack-us-healthcare"]
};
