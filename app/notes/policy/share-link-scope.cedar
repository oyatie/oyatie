// Cedar v4.2 LTS — Notes share-link scope gates
// Authority: ADR-NOTES-0001 (E2E personal-tier default); ADR-0244 (tenant scoping)

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // Personal-tier E2E notes require MLS group expansion (not bare URL share); soft-share is restricted
  resource.tenant.audience_type == "B2C_PERSONAL_E2E"
  && action has share_audience
  && action.share_audience == "PUBLIC_SHARE_VIEW"
  && !(action has client_acknowledges_decryption_implication
       && action.client_acknowledges_decryption_implication == true)
};

forbid (
  principal,
  action == Action::"NotesShareLinkRedeem",
  resource
) when {
  // Revoked share-token cannot be redeemed
  resource has share_token_status
  && resource.share_token_status in ["REVOKED", "EXPIRED", "TERMINATED"]
};

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // Share-token TTL cap 90 days
  action has requested_ttl_seconds
  && action.requested_ttl_seconds > 7776000
};
