// Cedar v4.2 LTS — public-read policy for slides µservice
// Public-read decks (link-sharing) — share-link-anyone-with-link scope.
// Default-deny baseline preserved.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (principal, action, resource);

// Public-share-link reader (anonymous or unauthenticated principal bearing valid share-link token)
permit (
    principal is PublicLinkPrincipal,
    action in [Action::"deck:read", Action::"slide:read"],
    resource is Deck
)
when {
    resource.share_link_enabled == true &&
    resource.share_link_scope == "anyone_with_link" &&
    principal.bears_share_link == true &&
    principal.share_link_signature_valid == true &&
    principal.share_link_pack == resource.pack &&
    resource.share_link_expires_at > principal.now &&
    (
        // No expiry OR future expiry
        !(resource has share_link_expires_at) ||
        resource.share_link_expires_at > principal.now
    )
};

// Public-share-link reader may read embedded forms-poll responses only in aggregate (no per-respondent reveal)
permit (
    principal is PublicLinkPrincipal,
    action == Action::"poll:read_aggregate",
    resource is EmbedPoll
)
when {
    resource.public_aggregate_enabled == true &&
    resource.tenant_id == resource.deck.tenant_id
};

// Public-share-link reader may NEVER comment unless deck explicitly allows
forbid (
    principal is PublicLinkPrincipal,
    action == Action::"deck:comment",
    resource is Deck
)
when {
    resource.allow_public_comments != true
};

permit (
    principal is PublicLinkPrincipal,
    action == Action::"deck:comment",
    resource is Deck
)
when {
    resource.allow_public_comments == true &&
    resource.share_link_enabled == true &&
    principal.share_link_signature_valid == true
};

// Public-share-link reader may NEVER edit
forbid (
    principal is PublicLinkPrincipal,
    action in [Action::"deck:edit", Action::"slide:edit", Action::"deck:share", Action::"deck:delete"],
    resource
);

// Public-share-link reader may NEVER access broadcast
forbid (
    principal is PublicLinkPrincipal,
    action in [Action::"broadcast:start", Action::"broadcast:join_as_presenter"],
    resource
);

// Public-share-link reader may join broadcast as audience-only IF deck.broadcast_link_public == true
permit (
    principal is PublicLinkPrincipal,
    action == Action::"broadcast:join_as_audience",
    resource is BroadcastSession
)
when {
    resource.deck.broadcast_link_public == true &&
    resource.pack == principal.share_link_pack &&
    principal.share_link_signature_valid == true
};

// Public-share-link reader may NEVER read special-category content
forbid (
    principal is PublicLinkPrincipal,
    action,
    resource is Deck
)
when {
    resource has contains_special_category &&
    resource.contains_special_category == true
};

// Public-share-link reader may NEVER access per-slide-restricted content via deck-level grant
forbid (
    principal is PublicLinkPrincipal,
    action == Action::"slide:read",
    resource is Slide
)
when {
    resource has per_slide_public_excluded &&
    resource.per_slide_public_excluded == true
};
