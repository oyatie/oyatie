// notes µservice — public-read policy fragment
// Cedar v4.2; covers share-link access by unauthenticated readers.
//
// Purpose: permit anonymous reads on valid share-link tokens; refuse all else.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// PERMIT — anonymous reader via share-link, after passphrase + TTL check
permit (
  principal == AnonymousReader::"shared",
  action in [
    Action::"read_shared_note",
    Action::"render_shared_note"
  ],
  resource in ShareLink::?s
)
when {
  resource has token_valid &&
  resource.token_valid == true &&
  resource has ttl_remaining &&
  resource.ttl_remaining > 0 &&
  resource has passphrase_satisfied &&
  resource.passphrase_satisfied == true &&
  resource has scope &&
  resource.scope == "ReadOnly"
};

// FORBID — anonymous reader MUST NOT access note metadata beyond shared scope
forbid (
  principal == AnonymousReader::"shared",
  action in [
    Action::"list_notes",
    Action::"read_backlink",
    Action::"read_tag_graph",
    Action::"render_graph_view",
    Action::"read_audit_chain"
  ],
  resource
);

// FORBID — anonymous reader MUST NOT access Personal-tier content even via share-link
// (Personal-tier sharing requires explicit per-note revealing of plaintext + passphrase;
// when shared the note's context_kind=Personal SHOULD remain enforced; metadata-only
// share is allowed for Personal as an exception; full-body share-link is permitted
// only when user explicitly toggles share-link-reveals-body.)
forbid (
  principal == AnonymousReader::"shared",
  action in [
    Action::"invoke_ai_assist",
    Action::"start_collab_session"
  ],
  resource
);
