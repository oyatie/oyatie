// network µservice — tenant-scope Cedar policy fragment
// Cedar v4.2 LTS — default-deny per ADR-0140
//
// Purpose: govern tenant-scoped reads + writes for the `network` Professional
//          µservice. All actions tenant-scoped via tenant_id binding. Cross-
//          tenant access strictly forbidden.
//
//          network is Professional-tier-only (ADR-0135 + ADR-0132). No
//          Personal-tier resources ever appear in this policy.
// Owner:   ops-security + axis-network + council-privacy
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant-scoped reads of Professional resources
// =============================================================================

permit (
  principal in Group::"tenant-operator",
  action in [
    Action::"read_profile",
    Action::"read_connection_graph",
    Action::"read_post",
    Action::"read_comment",
    Action::"read_reaction",
    Action::"read_endorsement",
    Action::"read_recommendation",
    Action::"read_skill_assessment",
    Action::"read_page",
    Action::"read_group",
    Action::"read_event",
    Action::"read_job_posting",
    Action::"render_feed",
    Action::"search_people",
    Action::"search_content",
    Action::"search_skills",
    Action::"search_jobs",
    Action::"search_companies",
    Action::"search_events",
    Action::"get_notifications"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 2 — end-user reads own Professional profile + own connections
// =============================================================================

permit (
  principal in Group::"end-user",
  action in [
    Action::"read_profile",
    Action::"render_feed",
    Action::"get_notifications",
    Action::"read_post",
    Action::"read_comment",
    Action::"read_endorsement",
    Action::"read_recommendation",
    Action::"read_skill_assessment",
    Action::"read_inmail_thread"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 3 — end-user writes own Professional content
// =============================================================================

permit (
  principal in Group::"end-user",
  action in [
    Action::"update_profile",
    Action::"send_connection_request",
    Action::"respond_to_connection_request",
    Action::"follow",
    Action::"unfollow",
    Action::"block",
    Action::"restrict",
    Action::"disconnect",
    Action::"publish_post",
    Action::"edit_post",
    Action::"delete_post",
    Action::"repost",
    Action::"quote_post",
    Action::"comment_on_post",
    Action::"react_to_post",
    Action::"endorse_skill",
    Action::"revoke_endorsement",
    Action::"publish_recommendation",
    Action::"submit_skill_assessment",
    Action::"request_profile_verification",
    Action::"send_inmail",
    Action::"upload_media",
    Action::"upload_document",
    Action::"export_profile_vcard",
    Action::"export_profile_jsonresume",
    Action::"export_profile_gdpr_art20",
    Action::"create_group",
    Action::"join_group",
    Action::"leave_group",
    Action::"create_event",
    Action::"rsvp_event",
    Action::"follow_page",
    Action::"set_automated_decision_preference",
    Action::"open_appeal",
    Action::"report_abuse"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has author_ref &&
  resource.author_ref == principal.user_ref
};

// =============================================================================
// PERMIT 4 — Page admins manage Pages + post newsletters via mail bridge
// =============================================================================

permit (
  principal in Group::"page-admin",
  action in [
    Action::"update_page",
    Action::"publish_page_post",
    Action::"request_page_newsletter_send",
    Action::"read_page_analytics"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has page_id &&
  resource.page_id in principal.administered_pages
};

// =============================================================================
// PERMIT 5 — Group admins moderate group content
// =============================================================================

permit (
  principal in Group::"group-admin",
  action in [
    Action::"approve_group_member",
    Action::"reject_group_member",
    Action::"remove_group_member",
    Action::"moderate_group_post",
    Action::"update_group_visibility"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has group_id &&
  resource.group_id in principal.administered_groups
};

// =============================================================================
// PERMIT 6 — Tenant-admin manages tenant configuration
// =============================================================================

permit (
  principal in Group::"tenant-admin",
  action in [
    Action::"configure_retention",
    Action::"configure_recruiter_stub_activation",
    Action::"configure_pack_overlay",
    Action::"manage_tenant_pages",
    Action::"manage_tenant_groups",
    Action::"verify_profile_or_page",
    Action::"open_ediscovery_hold",
    Action::"manage_inmail_rate_budget"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 7 — Recruiter (when tenant-enabled per ADR-NET-0002) invokes ranker
// =============================================================================

permit (
  principal in Group::"recruiter",
  action == Action::"invoke_recruiter_search",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  context has recruiter_stub_activated &&
  context.recruiter_stub_activated == true &&
  context has eu_ai_act_fria_attested &&
  context.eu_ai_act_fria_attested == true &&
  (
    context.tenant_pack != "us-nyc" ||
    (context has nyc_ll144_bias_audit_within_12mo && context.nyc_ll144_bias_audit_within_12mo == true)
  )
};

// =============================================================================
// PERMIT 8 — ATS µservice (Tier G) receives jobs-handoff contract-versioned events
// =============================================================================

permit (
  principal == Service::"ats",
  action == Action::"receive_jobs_handoff_event",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == context.target_tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  context has contract_version &&
  context.contract_version in ["v1", "v2"]
};

// =============================================================================
// PERMIT 9 — messenger µservice receives InMail-bridge events
// =============================================================================

permit (
  principal == Service::"messenger",
  action == Action::"receive_inmail_bridge_event",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == context.target_tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has inmail_target_channel &&
  resource.inmail_target_channel == "professional"
};

// =============================================================================
// PERMIT 10 — calendar µservice receives events-bridge events
// =============================================================================

permit (
  principal == Service::"calendar",
  action == Action::"receive_event_bridge_event",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == context.target_tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 11 — mail µservice receives page-newsletter-bridge events
// =============================================================================

permit (
  principal == Service::"mail",
  action == Action::"receive_page_newsletter_send_event",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == context.target_tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// FORBID — cross-tenant access
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id != principal.tenant_id
};

// =============================================================================
// FORBID — non-Professional context resources
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has context_kind &&
  resource.context_kind != "Professional"
};

// =============================================================================
// FORBID — recruiter when recruiter-stub not activated for tenant
// =============================================================================

forbid (
  principal,
  action == Action::"invoke_recruiter_search",
  resource
)
when {
  !(context has recruiter_stub_activated) ||
  context.recruiter_stub_activated == false
};

// =============================================================================
// FORBID — recruiter without EU AI Act FRIA attestation (for EU + UK tenants)
// =============================================================================

forbid (
  principal,
  action == Action::"invoke_recruiter_search",
  resource
)
when {
  (context.tenant_pack == "eu" || context.tenant_pack == "uk") &&
  (!(context has eu_ai_act_fria_attested) ||
   context.eu_ai_act_fria_attested == false)
};

// =============================================================================
// FORBID — recruiter without NYC LL144 bias audit (for NYC tenants)
// =============================================================================

forbid (
  principal,
  action == Action::"invoke_recruiter_search",
  resource
)
when {
  context.tenant_pack == "us-nyc" &&
  (!(context has nyc_ll144_bias_audit_within_12mo) ||
   context.nyc_ll144_bias_audit_within_12mo == false)
};

// =============================================================================
// FORBID — salary-insights per-individual disclosure
// =============================================================================

forbid (
  principal,
  action == Action::"read_salary_insight",
  resource
)
when {
  !(resource has aggregation_level) ||
  resource.aggregation_level != "aggregate"
};

// =============================================================================
// FORBID — minor-account ever surfaces in recruiter or salary-insights
// =============================================================================

forbid (
  principal,
  action in [
    Action::"invoke_recruiter_search",
    Action::"read_salary_insight",
    Action::"return_in_recruiter_search_result"
  ],
  resource
)
when {
  resource has minor_protect &&
  resource.minor_protect == true
};

// =============================================================================
// FORBID — minor-account receives InMail from unconnected adult
// =============================================================================

forbid (
  principal,
  action == Action::"send_inmail",
  resource
)
when {
  resource has minor_protect &&
  resource.minor_protect == true &&
  context has sender_connected_to_target &&
  context.sender_connected_to_target == false
};

// =============================================================================
// FORBID — endorsement-chain integrity violation
// =============================================================================

forbid (
  principal,
  action == Action::"endorse_skill",
  resource
)
when {
  !(context has signature_valid) ||
  context.signature_valid == false
};
