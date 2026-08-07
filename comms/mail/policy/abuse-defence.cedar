// Cedar v4.2 LTS — Mail µservice abuse-defence baseline
// Authority: ADR-0297; documentation-rigor.md §3.2.3
// Soak ≥60s per ADR-0294.
// UX floor: ZERO friction on legitimate JMAP / IMAP / SMTP flows. Bot scoring is passive.

// ---- Anti-bot (sign-up + password-reset surfaces only) ----

forbid (
  principal,
  action in [Action::"MailAccountSignup", Action::"MailPasswordReset"],
  resource
) when {
  principal.bot_score > 95
  && !(principal has tenant && principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

forbid (
  principal,
  action == Action::"MailJmapRequest",
  resource
) when {
  principal has request_rate_per_minute
  && resource has rate_limit_per_minute
  && principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// ---- Anti-spoof (inbound SMTP) ----

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // Reject inbound that fails DKIM AND fails SPF AND fails DMARC alignment
  action has dkim_pass
  && action has spf_pass
  && action has dmarc_pass
  && action.dkim_pass == false
  && action.spf_pass == false
  && action.dmarc_pass == false
};

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // DMARC p=reject from sender domain: block; ARC chain must be intact for forwards
  action has dmarc_policy
  && action.dmarc_policy == "reject"
  && action has dmarc_pass
  && action.dmarc_pass == false
};

// ---- Anti-scrape (JMAP harvesting + IMAP-bombing) ----

forbid (
  principal,
  action == Action::"MailMailboxList",
  resource
) when {
  principal has request_pattern
  && principal.request_pattern == "BREADTH_FIRST_HARVEST"
  && !(principal has tenant && principal.tenant.audience_type in ["INTERNAL_SUBSTRATE", "FRIENDLY_CRAWLER_PARTNER"])
};

// ---- Friendly-crawler bypass for public folders ONLY ----

permit (
  principal,
  action == Action::"MailPublicFolderRead",
  resource
) when {
  principal has tenant
  && principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER"
  && resource.public_listed == true
};

// ---- Default permit when bot-score below threshold ----

permit (
  principal,
  action in [Action::"MailJmapRequest", Action::"MailMailboxFetch", Action::"MailMessageGet"],
  resource
) when {
  principal.bot_score <= 95
  && principal.request_rate_per_minute <= resource.rate_limit_per_minute
};
