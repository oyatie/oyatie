// mail µservice — public-read policy fragment
// Cedar v4 schema
//
// Purpose: define the small, deliberately-bounded surface of mail endpoints
//          available without authentication. These are protocol-required
//          (RFC-compliance) endpoints.
// Owner:   axis-mail + ops-security
// Audit:   public reads NOT individually audited (would generate too much noise),
//          but aggregated counts emitted as mail_public_read_total{endpoint=...}
// References: threat-model.md (no public-read threat above L)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — public DNS-served records (DKIM public keys, SPF, DMARC, MTA-STS, TLS-RPT)
// =============================================================================
//
// These are RFC-mandated to be publicly resolvable; they are published in tenant
// domain DNS and have no authentication concept by design.

permit (
  principal,
  action == Action::"read_dns_public_record",
  resource in TenantDomainPublicRecord::?r
)
when {
  resource has record_type &&
  resource.record_type in [
    "dkim_public_key",
    "spf",
    "dmarc",
    "mta_sts_policy",
    "tls_rpt_endpoint"
  ]
};

// =============================================================================
// PERMIT 2 — public health check endpoints (k8s liveness / readiness)
// =============================================================================

permit (
  principal,
  action in [
    Action::"get_health",
    Action::"get_ready",
    Action::"get_smtp_banner"
  ],
  resource in MailServiceEndpoint::?e
);

// =============================================================================
// PERMIT 3 — abuse-reporting endpoint (RFC 6650 ARF feedback loop intake)
// =============================================================================
//
// Spam complaints + DMARC aggregate reports inbound from ISPs are accepted at a
// well-known public endpoint per RFC 6591 / RFC 6652 / RFC 7489.
// We rate-limit + classify; payload is verified before persisting.

permit (
  principal,
  action == Action::"submit_abuse_report",
  resource in AbuseReportEndpoint::?e
);

permit (
  principal,
  action == Action::"submit_dmarc_aggregate_report",
  resource in DmarcReportEndpoint::?e
);

permit (
  principal,
  action == Action::"submit_tls_rpt_report",
  resource in TlsRptEndpoint::?e
);

// =============================================================================
// PERMIT 4 — STARTTLS handshake on inbound SMTP 25
// =============================================================================
//
// External SMTP senders connect anonymously to port 25 per RFC 5321.
// STARTTLS handshake exposes server cert; this is a public read by design.
// Bound to inbound-smtp receiver only.

permit (
  principal,
  action == Action::"smtp_starttls_handshake",
  resource in MailServiceEndpoint::?e
)
when {
  resource has port &&
  resource.port == 25
};

// =============================================================================
// FORBID — everything else
// =============================================================================
//
// Default forbid above already covers; this is here to make the intent explicit
// for auditors reading this file: no other public-read surface exists.

forbid (
  principal,
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"send_message",
    Action::"read_dkim_config",
    Action::"engage_legal_hold",
    Action::"download_ediscovery_export"
  ],
  resource
);
