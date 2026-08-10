// Cedar v4.2 LTS — Notes µservice abuse-defence baseline
// Authority: ADR-0297; documentation-rigor.md §3.2.3
// UX floor: ZERO friction on typing / sync / search; bot scoring passive on sync surface.

// ---- Anti-bot (sign-up + abnormal-volume share-link issuance only) ----

forbid (
  principal,
  action in [Action::"NotesAccountSignup", Action::"NotesShareLinkIssue"],
  resource
) when {
  principal.bot_score > 95
};

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // Abnormal-volume share-link issuance (>50 per minute) likely indicates compromised account
  principal has share_link_issue_rate_per_minute
  && principal.share_link_issue_rate_per_minute > 50
};

// ---- Anti-scrape (share-link harvesting + sequential note-id traversal) ----

forbid (
  principal,
  action == Action::"NotesShareLinkRedeem",
  resource
) when {
  principal has fingerprint_age_seconds
  && principal.fingerprint_age_seconds < 30
  && principal has share_link_redeem_rate_per_minute
  && principal.share_link_redeem_rate_per_minute > 20
};

forbid (
  principal,
  action == Action::"NotesNoteGet",
  resource
) when {
  // Breadth-first note-id sweep
  principal has read_pattern_anomaly_score
  && principal.read_pattern_anomaly_score > 80
  && !(principal has tenant && principal.tenant.audience_type == "INTERNAL_SUBSTRATE")
};

// ---- Anti-spoof ----

forbid (
  principal,
  action,
  resource
) when {
  principal has principal_type
  && principal.principal_type == "WORKLOAD"
  && !(principal has spiffe_id)
};

// ---- Default permit (sync surface remains friction-free) ----

permit (
  principal,
  action in [Action::"NotesNoteSync", Action::"NotesNoteGet", Action::"NotesTagSearch", Action::"NotesCollabEdit"],
  resource
) when {
  principal.bot_score <= 95
};
