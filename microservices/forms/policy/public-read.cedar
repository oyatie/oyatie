// forms µservice — public-read policy fragment
// Cedar v4 schema
//
// Purpose: define what (if anything) is publicly readable from the forms µservice.
//          Default posture: nothing tenant-specific publicly readable. Published forms
//          are publicly accessible only via the published-form public URL (and only
//          when `accepting_submissions=true`). Form definitions themselves are NOT
//          publicly inspectable; only the rendered runtime is.
// Owner:   ops-security + council-privacy + axis-forms
// References: threat-model.md §"T-I-02" (embed iframe cross-origin leak) +
//             §"T-I-03" (renderer XSS)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — public access refused.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — anonymous fetch of static form-renderer WASM bundle + design-system primitives
// =============================================================================
//
// These assets are tenant-agnostic; required for renderer bootstrap.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"fetch_static_asset",
  resource in StaticAsset::?s
)
when {
  resource has tenant_scoped &&
  resource.tenant_scoped == false
};

// =============================================================================
// PERMIT — anonymous render of a published form (per published URL)
// =============================================================================
//
// Published form runtime renderer fetches a sanitised, pack-routed form spec.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"render_published_form",
  resource in PublishedForm::?f
)
when {
  resource has is_published &&
  resource.is_published == true &&
  resource has accepting_submissions &&
  resource.accepting_submissions == true
};

// =============================================================================
// PERMIT — anonymous submission to a published form
// =============================================================================
//
// Captcha + rate-limit enforced separately at WAF + middleware.

permit (
  principal == AnonymousPrincipal::"anonymous",
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
  resource.accepting_submissions == true &&
  resource has anonymous_submission_allowed &&
  resource.anonymous_submission_allowed == true
};

// =============================================================================
// PERMIT — public status (Forms health, no tenant data)
// =============================================================================

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"read_public_health",
  resource == PublicEndpoint::"forms_health"
);

// =============================================================================
// FORBID — explicitly nothing tenant-specific publicly readable
// =============================================================================

forbid (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"read_form_definition",
    Action::"list_forms",
    Action::"read_response_redacted",
    Action::"read_response_with_pii",
    Action::"list_responses",
    Action::"read_form_analytics",
    Action::"read_form_response_aggregate",
    Action::"export_responses_csv",
    Action::"export_responses_xlsx",
    Action::"export_responses_json",
    Action::"create_form",
    Action::"edit_form_draft",
    Action::"publish_form",
    Action::"bulk_distribute_form_email",
    Action::"invoke_ai_form_build"
  ],
  resource
);

// =============================================================================
// FORBID — per-tenant branding mid-render is forbidden (anti-pattern)
// =============================================================================
//
// Anti-pattern: tenant-authored branding rendered inline in same DOM tree as
// form fields without CSP isolation. Strict CSP + Trusted Types enforced.

forbid (
  principal,
  action == Action::"render_tenant_branding_inline_no_csp",
  resource
);
