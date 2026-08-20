// Cedar v4.2 LTS — tenant-scope policy for slides µservice
// Default-deny baseline. Per ADR-0140 (retired per ADR-0145) + ADR-SLIDES-0007 per-slide ACL granularity.
// Effective: deck access, slide access, named-block access, share-link access, broadcast.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// ============================================================================
// Forbid: default-deny baseline
// ============================================================================

forbid (principal, action, resource);

// ============================================================================
// Tenant resolution invariant — every access requires resolved tenant identity
// ============================================================================

forbid (principal, action, resource)
when {
    principal has tenant_id == "" || !(principal has tenant_id)
};

// ============================================================================
// Cross-tenant refusal: principal.tenant_id must equal resource.tenant_id
// ============================================================================

forbid (principal, action, resource)
when {
    principal has tenant_id &&
    resource has tenant_id &&
    principal.tenant_id != resource.tenant_id
};

// ============================================================================
// Cross-pack refusal: principal.pack must equal resource.pack
// (per ADR-0117 residency overlay; cross-pack collab forbidden)
// ============================================================================

forbid (principal, action, resource)
when {
    principal has pack &&
    resource has pack &&
    principal.pack != resource.pack
};

// ============================================================================
// Deck-level access — owner, editor, commenter, viewer roles
// ============================================================================

permit (
    principal,
    action in [
        Action::"deck:open",
        Action::"deck:read",
        Action::"deck:edit",
        Action::"deck:comment",
        Action::"deck:share",
        Action::"deck:export",
        Action::"deck:present",
        Action::"deck:broadcast"
    ],
    resource is Deck
)
when {
    resource.tenant_id == principal.tenant_id &&
    resource.pack == principal.pack &&
    (
        principal in resource.owners ||
        principal in resource.editors ||
        (action in [Action::"deck:open", Action::"deck:read", Action::"deck:comment"] && principal in resource.commenters) ||
        (action in [Action::"deck:open", Action::"deck:read"] && principal in resource.viewers)
    )
};

// ============================================================================
// Per-slide ACL refinement — slide-level grants override deck-level when present
// (per ADR-SLIDES-0007 named-block granularity)
// ============================================================================

permit (
    principal,
    action in [Action::"slide:read", Action::"slide:edit", Action::"slide:comment"],
    resource is Slide
)
when {
    resource.tenant_id == principal.tenant_id &&
    resource.pack == principal.pack &&
    (
        // Deck-level grant + no per-slide deny
        (
            (principal in resource.deck.owners ||
             principal in resource.deck.editors ||
             (action in [Action::"slide:read", Action::"slide:comment"] && principal in resource.deck.commenters) ||
             (action == Action::"slide:read" && principal in resource.deck.viewers))
            &&
            !(principal in resource.per_slide_deny)
        )
        ||
        // Explicit per-slide grant
        (
            principal in resource.per_slide_editors ||
            (action in [Action::"slide:read", Action::"slide:comment"] && principal in resource.per_slide_commenters) ||
            (action == Action::"slide:read" && principal in resource.per_slide_viewers)
        )
    )
};

// ============================================================================
// Named-block ACL refinement — block-level grants further refine slide-level
// (per ADR-SLIDES-0007)
// ============================================================================

permit (
    principal,
    action in [Action::"named_block:read", Action::"named_block:edit"],
    resource is NamedBlock
)
when {
    resource.tenant_id == principal.tenant_id &&
    resource.pack == principal.pack &&
    (
        // Slide-level grant inherited unless block-level deny present
        (
            // Slide allow + no block deny
            !(principal in resource.block_deny)
        )
        ||
        // Explicit block-level grant
        principal in resource.block_editors ||
        (action == Action::"named_block:read" && principal in resource.block_viewers)
    )
};

// ============================================================================
// Share-link scope — link-anyone-with-link evaluated separately
// ============================================================================

permit (
    principal,
    action == Action::"deck:read",
    resource is Deck
)
when {
    resource.share_link_enabled == true &&
    resource.share_link_scope == "anyone_with_link" &&
    principal.bears_share_link == true &&
    resource.pack == principal.pack
};

// ============================================================================
// Per-seat license refinement (T-D-07 fail-closed when license-gate down)
// ============================================================================

forbid (
    principal,
    action == Action::"deck:open",
    resource is Deck
)
when {
    principal has license_evaluated &&
    principal.license_evaluated == false  // license-gate has not evaluated
};

forbid (
    principal,
    action == Action::"deck:open",
    resource is Deck
)
when {
    principal has license_status &&
    principal.license_status == "exceeded"  // tenant seat cap exceeded
};

// ============================================================================
// Broadcast permissions — presenter vs admin
// ============================================================================

permit (
    principal,
    action == Action::"broadcast:start",
    resource is Deck
)
when {
    resource.tenant_id == principal.tenant_id &&
    resource.pack == principal.pack &&
    (principal in resource.owners || principal in resource.editors) &&
    principal.broadcast_capable == true
};

// ============================================================================
// AI-content-generation T2 — explicit pack opt-in for Annex III high-risk
// ============================================================================

forbid (
    principal,
    action == Action::"ai_content_generation:invoke",
    resource is Deck
)
when {
    resource.ai_risk_class == "high_risk_annex_iii" &&
    !(resource.pack_override_annex_iii == true)
};

// ============================================================================
// Special-category data refinement — eu pack + us-healthcare pack
// ============================================================================

forbid (
    principal,
    action,
    resource is Deck
)
when {
    resource has contains_special_category &&
    resource.contains_special_category == true &&
    !(principal has special_category_consent) &&
    resource.pack in ["eu", "us-healthcare"]
};
