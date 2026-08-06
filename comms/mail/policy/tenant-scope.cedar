// mail µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/mail/policy/schema.cedarschema
//
// Purpose: enforce per-tenant + per-context (Personal vs Professional) scope on mailbox
//          read/write operations served via REST, JMAP, IMAP, SDK.
// Owner:   ops-security + axis-mail + council-privacy
// Audit:   every deny emits mail_unauthorized_read_attempt_total or mail_cross_context_routing_refused_total
// References: dual-context-isolation.md §"Invariants" + threat-model.md T-I-01, T-I-02, T-E-05
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every read/write is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant employee reads own Professional-context mailbox
// =============================================================================
//
// A principal in TenantEmployee::"<user-id>" with session.context == Professional
// may read mailboxes where:
//   - mailbox.tenant_id == principal.tenant_id  (tenant-scope)
//   - mailbox.context_kind == "Professional"    (context-scope)
//   - mailbox.owner_ref == principal.user_id    (ownership-scope)
// OR principal has the per-tenant "shared_mailbox_read" entitlement.

permit (
  principal in TenantEmployee::?p,
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"list_folders",
    Action::"search"
  ],
  resource in Mailbox::?m
)
when {
  principal has tenant_id &&
  principal has session_context &&
  principal.session_context == "Professional" &&
  resource has tenant_id &&
  resource has context_kind &&
  resource has owner_ref &&
  principal.tenant_id == resource.tenant_id &&
  resource.context_kind == "Professional" &&
  (resource.owner_ref == principal.user_id ||
   (principal has shared_mailbox_entitlements && resource.mailbox_id in principal.shared_mailbox_entitlements))
};

// =============================================================================
// PERMIT 2 — tenant employee writes own Professional-context mailbox
// =============================================================================

permit (
  principal in TenantEmployee::?p,
  action in [
    Action::"send_message",
    Action::"move_message",
    Action::"flag_message",
    Action::"label_message",
    Action::"delete_message"
  ],
  resource in Mailbox::?m
)
when {
  principal has tenant_id &&
  principal has session_context &&
  principal.session_context == "Professional" &&
  resource has tenant_id &&
  resource has context_kind &&
  resource has owner_ref &&
  principal.tenant_id == resource.tenant_id &&
  resource.context_kind == "Professional" &&
  resource.owner_ref == principal.user_id
};

// =============================================================================
// PERMIT 3 — user reads own Personal-context mailbox
// =============================================================================
//
// A principal in TenantEmployee::"<user-id>" OR PersonalUser::"<user-id>" with
// session.context == Personal may read their OWN Personal mailbox.
// Tenant scope does NOT apply to Personal (Personal mailboxes are not tenant-owned;
// they are user-owned).

permit (
  principal in [TenantEmployee::?p, PersonalUser::?p],
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"list_folders",
    Action::"search"
  ],
  resource in Mailbox::?m
)
when {
  principal has user_id &&
  principal has session_context &&
  principal.session_context == "Personal" &&
  resource has context_kind &&
  resource has owner_ref &&
  resource.context_kind == "Personal" &&
  resource.owner_ref == principal.user_id
};

// =============================================================================
// PERMIT 4 — user writes own Personal-context mailbox
// =============================================================================

permit (
  principal in [TenantEmployee::?p, PersonalUser::?p],
  action in [
    Action::"send_message",
    Action::"move_message",
    Action::"flag_message",
    Action::"label_message",
    Action::"delete_message"
  ],
  resource in Mailbox::?m
)
when {
  principal has user_id &&
  principal has session_context &&
  principal.session_context == "Personal" &&
  resource has context_kind &&
  resource has owner_ref &&
  resource.context_kind == "Personal" &&
  resource.owner_ref == principal.user_id
};

// =============================================================================
// PERMIT 5 — tenant mail admin reads tenant's deliverability dashboard
// =============================================================================
//
// Mail admin sees tenant-level aggregates (volume, bounce rate, reputation score).
// Admin does NOT see message content.

permit (
  principal in TenantMailAdmin::?p,
  action in [
    Action::"read_deliverability_dashboard",
    Action::"read_retention_policy",
    Action::"read_dkim_config",
    Action::"read_mta_sts_policy",
    Action::"list_tenant_mailboxes"
  ],
  resource in TenantMailServer::?t
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 6 — tenant compliance officer engages/releases legal hold (Professional only)
// =============================================================================
//
// Compliance officer scope is distinct from mail admin (per threat-model T-E-01).
// Hold engage on personal-pillar is forbidden per dual-context-isolation Invariant DCI-04.
// Four-eyes co-signing required: this Cedar permit covers ONE approver; the engine
// asserts the second approver's signature separately.

permit (
  principal in TenantComplianceOfficer::?p,
  action in [
    Action::"engage_legal_hold",
    Action::"release_legal_hold",
    Action::"list_legal_holds"
  ],
  resource in LegalHoldScope::?s
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource has context_kind &&
  principal.tenant_id == resource.tenant_id &&
  resource.context_kind == "Professional"  // FORBID on Personal
};

// =============================================================================
// PERMIT 7 — tenant compliance officer requests eDiscovery export (Professional only)
// =============================================================================
//
// Same as PERMIT 6 with the four-eyes engine-side enforcement.
// Plaintext disclosure (bundle decryption) requires the second approver.

permit (
  principal in TenantComplianceOfficer::?p,
  action in [
    Action::"request_ediscovery_export",
    Action::"download_ediscovery_export"
  ],
  resource in EDiscoveryExportJob::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource has context_kind &&
  principal.tenant_id == resource.tenant_id &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 8 — workflow operator initiates mail-to-Workflow handoff (Professional only)
// =============================================================================

permit (
  principal in TenantWorkflowOperator::?p,
  action == Action::"create_mail_workflow_handoff",
  resource in MailMessage::?m
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource has context_kind &&
  principal.tenant_id == resource.tenant_id &&
  resource.context_kind == "Professional" &&
  // explicit user action OR tenant policy basis present
  (principal has user_explicit_handoff_session_id ||
   principal has tenant_policy_basis_ref)
};

// =============================================================================
// FORBID — explicit cross-context deny (defence-in-depth)
// =============================================================================
//
// Even if a permit above accidentally matched a cross-context read,
// this forbid blocks it. Cedar's deny-overrides semantics make this safety net.

forbid (
  principal,
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"list_folders",
    Action::"search",
    Action::"send_message",
    Action::"move_message",
    Action::"flag_message",
    Action::"label_message",
    Action::"delete_message"
  ],
  resource in Mailbox::?m
)
unless {
  principal has session_context &&
  resource has context_kind &&
  principal.session_context == resource.context_kind
};

// =============================================================================
// FORBID — Personal mailbox unreachable from any non-self principal
// =============================================================================

forbid (
  principal,
  action,
  resource in Mailbox::?m
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal" &&
  resource has owner_ref &&
  principal has user_id &&
  principal.user_id != resource.owner_ref
};

// =============================================================================
// FORBID — never expose SECRET-class data via any read
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
// FORBID — legal hold + eDiscovery NEVER on Personal-pillar resources
// =============================================================================

forbid (
  principal,
  action in [
    Action::"engage_legal_hold",
    Action::"release_legal_hold",
    Action::"request_ediscovery_export",
    Action::"download_ediscovery_export"
  ],
  resource
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};
