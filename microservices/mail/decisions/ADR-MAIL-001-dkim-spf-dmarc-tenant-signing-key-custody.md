---
id: ADR-MAIL-001
title: DKIM SPF DMARC Enforcement and Per-Tenant Signing Key Custody
status: Accepted
date: 2026-05-20
microservice: mail
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
  - docs/decisions/ADR-0090-hyper-canonical-http-backbone.md
decision_owner: axis-mail
---

# ADR-MAIL-001: DKIM SPF DMARC Enforcement and Per-Tenant Signing Key Custody

## Context

- Mail owns inbound SMTP, outbound SMTP, JMAP, IMAP, mailbox storage, search, retention, legal hold, and eDiscovery workflows.
- The service architecture already names DKIM, SPF, DMARC, ARC, anti-phishing, and JMAP as cold-start entry points.
- Existing runbooks include `dkim-key-rotation.md`, `dmarc-rollout-monitoring.md`, and `account-compromise-recovery.md`.
- Existing policy files include `policy/anti-phishing.cedar`, `policy/abuse-defence.cedar`, `policy/tenant-scope.cedar`, and `policy/phi-dlp.cedar`.
- Named precedent: Gmail and Google Workspace treat SPF, DKIM, DMARC, and ARC as first-class deliverability controls.
- Named precedent: AWS SES domain identities and configuration sets separate tenant domain verification from platform sending infrastructure.
- Named precedent: Microsoft Exchange Online DKIM uses per-domain selectors and rolling selector activation.
- Constraint MAIL-C1: domain ownership must be tenant-bound via ADR-0002, not inferred from DNS alone after initial verification.
- Constraint MAIL-C2: outbound signing, inbound authentication result, and DMARC disposition must be audit events per ADR-0003.
- Constraint MAIL-C3: Cedar must gate tenant domain mutation, selector activation, delegated sender use, and DMARC override per ADR-0007.
- Constraint MAIL-C4: signing private keys must stay in OpenBao / HSM-backed custody per ADR-0043 with per-tenant paths.
- Constraint MAIL-C5: HTTP management APIs use the Hyper-backed service runtime per ADR-0090.
- Constraint MAIL-C6: demo_trial, paid, HIPAA, EU-sovereign, and KR packs need different DMARC rollout ceilings.
- Constraint MAIL-C7: the service must support bring-your-own-domain and platform-managed sender domains.
- Constraint MAIL-C8: tenant admins must be able to rotate selectors without global platform intervention.
- Constraint MAIL-C9: compromised tenant domain credentials must not compromise platform mail domains or other tenants.
- Constraint MAIL-C10: inbound mail cannot trust display names, From domains, Reply-To domains, or ARC chains without explicit authentication state.
- Constraint MAIL-C11: spam and phishing classifiers must see normalized authentication results as typed inputs, not raw header parsing.
- Constraint MAIL-C12: legal-hold and eDiscovery exports must preserve authentication results and signing evidence.
- Constraint MAIL-C13: user notification UX must distinguish unauthenticated mail from policy-quarantined mail.
- The decision must support RFC 7208 SPF, RFC 6376 DKIM, RFC 7489 DMARC, RFC 8617 ARC, RFC 8461 MTA-STS, RFC 8460 TLSRPT, and RFC 8620 JMAP.
- The architecture must treat DNS lookups as untrusted external IO with cache TTL, DNSSEC status, and resolver-cell evidence.
- The architecture must allow staged DMARC policy progression from `none` to `quarantine` to `reject`.
- The architecture must prevent tenant admins from disabling platform baseline anti-spoof checks.
- The architecture must generate verification evidence that deliverability regressions can be diagnosed from logs and metrics.

## Decision

- Enforce SPF, DKIM, DMARC, ARC, MTA-STS, and TLSRPT as typed mail-auth primitives in the mail domain layer.
- Create bounded context `mail-auth-policy` for domain verification, selector lifecycle, outbound signing, and inbound authentication disposition.
- Store every tenant sending domain as `TenantMailDomain`, owned by one tenant and one home cell.
- Require domain verification through DNS TXT challenge before any outbound signing is enabled.
- Require tenant admins to publish at least two DKIM selectors before moving a domain to production send.
- Use selector names `sYYYYMMDDa` and `sYYYYMMDDb` for deterministic rotation and overlap.
- Use Ed25519 DKIM where receiver compatibility allows; default to RSA-2048 for broad receiver compatibility until pack policy upgrades the default.
- Keep DKIM private keys in OpenBao with HSM-backed auto-unseal and per-tenant path isolation.
- Never export DKIM private key material to application memory outside a <=60 second signing lease.
- Use OpenBao transit signing for pack classes that require hardware-backed non-exportability.
- Use sidecar-issued signing handles for high-volume tenants to avoid per-message OpenBao round trips.
- Validate SPF at inbound SMTP time and persist `spf_result`, `spf_domain`, `spf_dnssec_state`, and `spf_expanded_lookup_count`.
- Validate DKIM at inbound SMTP time and persist one row per signature, including selector, domain, canonicalization, body hash result, and key age.
- Validate DMARC after SPF and DKIM alignment, using relaxed alignment by default and strict alignment where tenant policy requires it.
- Validate ARC only as a modifier to forwarding trust; ARC never overrides DMARC `reject` for high-risk packs unless a tenant allowlist grants it.
- Publish DMARC aggregate and forensic report endpoints per tenant with pack-controlled redaction.
- Enforce outbound DMARC alignment before send; messages that cannot align are held in `outbound_auth_hold`.
- Enforce tenant-level policy progression through `none`, `quarantine`, and `reject` with a seven-day soak between levels.
- Emit a typed audit event for every policy change, selector activation, selector retirement, signing failure, inbound reject, and DMARC override.
- Use Cedar to authorize `mail::domain::verify`, `mail::dkim_selector::activate`, `mail::dmarc_policy::promote`, and `mail::auth_override::grant`.
- Use the anti-phishing policy after authentication normalization, not before it.
- Feed normalized authentication features into spam classification and phishing detection.
- Keep platform-owned domains in a separate `platform-mail` tenant, not in a global exception table.
- Make all mail-auth public contracts additive under `/v1/mail/auth/*`.

## Alternatives Considered

### Shared Platform DKIM Key Per Domain Class

- Pros: fewer keys to rotate.
- Pros: lower OpenBao load.
- Pros: simpler outbound signing worker.
- Cons: one key compromise affects many tenants.
- Cons: no tenant-specific selector history for audit and eDiscovery.
- Cons: enterprise tenants cannot prove custody separation.
- Rejected because per-tenant blast radius is mandatory for workspace and healthcare mail.

### Tenant-Uploaded Private DKIM Keys

- Pros: gives tenants full key ownership.
- Pros: resembles some legacy mail-transfer products.
- Pros: migration from existing infrastructure can be quick.
- Cons: key quality and format validation become tenant support burden.
- Cons: imported material cannot always prove HSM custody.
- Cons: upload surfaces create an avoidable exfiltration path.
- Rejected because Oyatie can generate per-tenant keys in controlled custody and expose DNS records without importing secrets.

### DMARC Reporting Only, No Enforcement

- Pros: lowers false-positive risk during onboarding.
- Pros: avoids blocking misconfigured tenant senders.
- Pros: easy to ship quickly.
- Cons: lets spoofed tenant mail reach users after we know it is unauthenticated.
- Cons: fails high-risk enterprise and healthcare pack expectations.
- Cons: does not give anti-phishing dependable typed features.
- Rejected because the service must enforce, not just observe, domain-auth posture.

### Outsource Signing and Deliverability to SES or SendGrid

- Pros: mature deliverability infrastructure.
- Pros: bounce processing and reputation management are available.
- Pros: lower initial operations burden.
- Cons: conflicts with long-term in-house mail substrate ownership.
- Cons: weakens per-tenant key custody evidence.
- Cons: provider lock-in complicates sovereign and healthcare packs.
- Rejected for core mail; third-party relay can remain a tenant-approved egress adapter with the same policy gates.

## Consequences

- Positive: tenant signing key compromise is isolated to one tenant domain and selector epoch.
- Positive: domain-auth results become typed data for abuse, phishing, support, and audit workflows.
- Positive: tenants can roll selectors without platform-wide maintenance windows.
- Positive: DMARC policy promotion is measurable and reversible.
- Positive: inbound rejection decisions can be explained to support and auditors.
- Positive: platform-owned sender domains are treated as tenant-scoped assets.
- Positive: compliance packs can impose stricter alignment without forking mail architecture.
- Positive: eDiscovery exports include delivery trust evidence without requiring raw log reconstruction.
- Negative: OpenBao / HSM availability becomes part of outbound send availability.
- Negative: DNS resolver cache correctness becomes a user-visible deliverability dependency.
- Negative: selector rotation mistakes can temporarily reduce deliverability.
- Negative: strict DMARC policy can reject legitimate forwarded mail without ARC allowlist tuning.
- Negative: high-volume tenants may require signing-handle pooling and careful lease invalidation.
- Neutral: inbound SMTP can accept and quarantine messages even if downstream JMAP is degraded.
- Neutral: demo_trial tenants may remain at DMARC `none` longer, but spoofing signals still influence user warnings.
- Neutral: tenant-managed relay adapters must map their provider evidence back into the same `MailAuthResult`.
- Neutral: key algorithm defaults can change by pack and receiver compatibility without changing public API shape.
- Neutral: platform support can see authentication evidence, but never DKIM private keys.

## Implementation Notes

- Data shape `TenantMailDomain`: `{tenant_id, domain_id, fqdn, verification_state, home_cell, dmarc_policy, alignment_mode, pack_set_hash}`.
- Data shape `DkimSelector`: `{tenant_id, domain_id, selector, algorithm, public_key_dns_value, openbao_key_ref, state, created_at, activates_at, retires_at}`.
- Data shape `MailAuthResult`: `{message_id, tenant_id, from_domain, spf_result, dkim_results[], dmarc_result, arc_result, tls_result, final_disposition}`.
- Data shape `OutboundSigningRequest`: `{tenant_id, domain_id, selector, message_hash, canonicalization, lease_id, idempotency_key}`.
- Data shape `DmarcPromotion`: `{tenant_id, domain_id, from_policy, to_policy, sample_window, failure_rate, approved_by, audit_event_id}`.
- OpenBao path: `secret/<tenant_id>/mail/dkim/<domain_id>/<selector>`.
- OpenBao path: `transit/keys/<tenant_id>-mail-dkim-<selector>` for non-exportable signing.
- REST endpoint `POST /v1/mail/auth/domains` creates a domain challenge.
- REST endpoint `GET /v1/mail/auth/domains/{domain_id}/dns-records` returns SPF, DKIM, DMARC, MTA-STS, and TLSRPT records.
- REST endpoint `POST /v1/mail/auth/domains/{domain_id}/verify` validates DNS and tenant ownership.
- REST endpoint `POST /v1/mail/auth/domains/{domain_id}/dkim/selectors` creates the next selector pair.
- REST endpoint `POST /v1/mail/auth/domains/{domain_id}/dkim/selectors/{selector}/activate` activates a selector.
- REST endpoint `POST /v1/mail/auth/domains/{domain_id}/dmarc/promote` promotes DMARC policy after soak.
- REST endpoint `POST /v1/mail/auth/overrides/arc-forwarder` grants a scoped ARC forwarding allowlist.
- AsyncAPI channel `mail.auth.domain.verified.v1` publishes domain verification.
- AsyncAPI channel `mail.auth.dkim.selector.activated.v1` publishes active selector state.
- AsyncAPI channel `mail.auth.dmarc.disposition.v1` publishes inbound disposition.
- AsyncAPI channel `mail.auth.signing.failure.v1` publishes outbound signing failures.
- Cedar permit `mail::domain::verify` requires tenant admin role and matching `tenant_id`.
- Cedar permit `mail::dkim_selector::activate` requires step-up and domain ownership.
- Cedar permit `mail::dmarc_policy::promote` requires failure rate below pack threshold.
- Cedar forbid `mail::auth_override::grant` when `resource.pack in ["hipaa", "fedramp-high"]` unless council approval exists.
- Audit event `EVT-MAIL-DOMAIN-VERIFIED` includes DNS challenge hash and resolver cell.
- Audit event `EVT-MAIL-DKIM-SELECTOR-ACTIVATED` includes selector, algorithm, and key custody ref.
- Audit event `EVT-MAIL-DMARC-POLICY-PROMOTED` includes from / to policy and sample metrics.
- Audit event `EVT-MAIL-INBOUND-DMARC-REJECTED` includes aligned identifiers and final disposition.
- Metric `mail_auth_spf_dns_lookup_count` watches RFC 7208 lookup budget exhaustion.
- Metric `mail_auth_dkim_sign_latency_ms` watches signing path latency.
- Metric `mail_auth_dmarc_reject_total` counts enforced rejects by tenant policy state.
- Metric `mail_auth_dns_cache_staleness_seconds` watches resolver freshness.
- Capacity math: if a tenant sends 2,000 messages/s and signing p95 is 4 ms through sidecar handles, Little's Law gives 8 in-flight signatures; provision 80 slots for 10x burst.
- Capacity math: DMARC aggregate reports at 10 million messages/day with 1 percent failure produce 100k failure rows; partition by tenant and day.
- Rollback path: selector activation rollback reverts active selector pointer and keeps old DNS record valid for 72 hours.
- Rollback path: DMARC promotion rollback demotes policy and emits a tenant-visible evidence note.
- Multi-region path: outbound signing happens in home cell; remote queueing can hold but not sign with copied keys.
- Sovereign path: KR, EU, and FedRAMP packs prohibit cross-region signing leases.
- Versioning: `/v1/mail/auth/*` is additive; destructive field changes require `/v2`.
- Deprecation: old selector algorithms get 180-day deprecation unless an emergency CVE forces faster rotation.

## Verification

- Unit test `domain_verify_requires_tenant_admin` proves Cedar rejects non-owner domain verification.
- Unit test `dkim_selector_private_key_never_serializes` proves private key material is not returned by domain APIs.
- Unit test `dmarc_promotion_requires_soak_window` proves `none` to `reject` cannot skip stages.
- Unit test `arc_never_overrides_high_risk_dmarc_reject` proves pack-gated ARC behavior.
- Property test `dmarc_alignment_relaxed_and_strict` covers RFC 7489 alignment combinations.
- Property test `spf_lookup_budget_never_exceeds_ten` covers RFC 7208 DNS lookup limits.
- Fuzz test `dkim_header_parser_rejects_malformed_canonicalization` covers hostile mail headers.
- Integration test `outbound_sign_uses_tenant_openbao_path` verifies custody path shape.
- Integration test `inbound_auth_result_feeds_anti_phishing` verifies typed feature handoff.
- Integration test `selector_rotation_overlap_preserves_delivery` verifies old and new selectors during overlap.
- Integration test `tenant_domain_compromise_revokes_only_that_domain` checks blast radius.
- Integration test `ediscovery_export_includes_auth_result` checks export evidence.
- Load test `dkim_sign_2000_messages_per_second` keeps p95 signing below 10 ms.
- Load test `dmarc_report_ingest_10m_messages_per_day` keeps p99 aggregate write below 200 ms.
- Chaos test `openbao_unavailable_holds_outbound_mail` proves no unsigned fallback occurs.
- Chaos test `dns_resolver_partition_uses_cached_policy_until_ttl` proves bounded stale cache behavior.
- Metric SLO: `mail_auth_dkim_sign_latency_ms` p95 below 10 ms per cell.
- Metric SLO: `mail_auth_dns_cache_staleness_seconds` p99 below published TTL plus 30 seconds.
- Metric SLO: false positive DMARC reject review rate below 0.1 percent during promotion windows.
- Audit check: every selector activation has one `EVT-MAIL-DKIM-SELECTOR-ACTIVATED`.
- Audit check: every DMARC override has an approving principal and expiry.
- Static check: no domain record stores DKIM private key bytes.
- Static check: platform domains are represented with `tenant_id=platform-mail`, not null.
- Contract check: OpenAPI exposes no private-key export endpoint.
