---
doc_class: ThreatModel
microservice: mail
version: 1.0.0
status: Proposed
date: 2026-05-20
owner: axis-mail + council-security
related_oyatie_adrs:
  - ADR-0003
  - ADR-0009
  - ADR-0145
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0297
  - ADR-0313
  - ADR-0319
---

# Mail Security Threat Model

This document covers the mail substrate security posture for inbound delivery,
outbound submission, mailbox access, per-tenant signing-key custody, reputation,
DLP, legal hold, and incident detection. Mail is both a data plane and an abuse
plane: it receives hostile public traffic, emits trusted tenant-branded traffic,
and stores high-value communications that can contain PII, PHI, secrets, legal
material, and payment instructions.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| MAIL-A01 | MailboxMessageBody | MIME body and attachments for professional and personal context mailboxes. | Object store and mailbox metadata DB | Prevent exfiltration and unauthorized disclosure. |
| MAIL-A02 | MailHeaderEnvelope | SMTP envelope, From, To, CC, BCC, Subject, Message-ID, routing metadata. | Postgres mailbox store | Prevent spoofing, tracking, and enumeration. |
| MAIL-A03 | TenantDkimSigningKey | Per-tenant DKIM private key and selector metadata. | OpenBao | Prevent tenant signing-key custody compromise. |
| MAIL-A04 | TenantDmarcPolicy | SPF, DKIM, DMARC, MTA-STS, TLS-RPT, ARC policy and rollout state. | Tenant config and DNS publication pipeline | Prevent inbound phishing bypass and outbound deliverability collapse. |
| MAIL-A05 | OutboundReputationState | Per-tenant IP pool, bounce rate, complaint rate, abuse score, warmup status. | Reputation tracker DB | Prevent spam outbound and tenant reputation poisoning. |
| MAIL-A06 | MailboxSearchIndex | Encrypted or tokenized search terms and mailbox index state. | Tantivy/Meilisearch equivalent | Prevent plaintext content leakage through index. |
| MAIL-A07 | LegalHoldRecord | Hold scope, approval chain, eDiscovery export request, chain-of-custody seal. | Legal-hold store and audit-chain | Prevent evidence tampering and repudiation. |
| MAIL-A08 | DlpQuarantineRecord | DLP verdict, matching rule, release decision, reviewer identity. | DLP store and audit-chain | Prevent PII/PHI leakage through mail. |
| MAIL-A09 | InboundPhishingSignal | SPF/DKIM/DMARC/ARC result, URL verdict, attachment verdict, BEC indicators. | Abuse-defence telemetry and audit-chain | Detect phishing and BEC. |
| MAIL-A10 | SmtpSubmissionCredential | SASL credential, OIDC token, app password, or service credential used to send. | Identity/OpenBao | Prevent account takeover and open relay. |
| MAIL-A11 | MailWorkflowHandoff | Workflow, calendar, messenger, drive attachment, or automation event emitted from mail. | Workflow bus and audit-chain | Prevent cross-service injection. |
| MAIL-A12 | AuditEmissionEnvelope | ADR-0263 envelope with tenant_id, trace_id, span_id, audit_id, schema_version, source_microservice. | audit-chain | Preserve detection and non-repudiation. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| MAIL-I01 | Inbound SMTP | `../contracts/asyncapi/mail-events.yaml` | External sender | Public port 25, STARTTLS opportunistic, hostile by default. |
| MAIL-I02 | SMTP Submission | `../contracts/asyncapi/mail-events.yaml` | Authenticated user or service | Port 587/465, SASL or OIDC-bound, DKIM signing. |
| MAIL-I03 | IMAP/JMAP/REST Read | `../contracts/openapi/mail.yaml` | Authenticated user | Mailbox read/write, personal/work context split. |
| MAIL-I04 | DKIM Key Rotation | `../runbooks/dkim-key-rotation.md` | Tenant admin and key worker | Per-tenant signing-key custody and selector rollover. |
| MAIL-I05 | DMARC Rollout | `../runbooks/dmarc-rollout-monitoring.md` | Tenant admin | Policy hardening from none to quarantine/reject. |
| MAIL-I06 | DLP Quarantine | `../runbooks/dlp-quarantine-release.md` | Security reviewer | Holds suspect message before delivery or release. |
| MAIL-I07 | Legal Hold Export | `../runbooks/mailbox-restore-from-backup.md` and legal-hold worker | Legal/audit role | Produces eDiscovery evidence. |
| MAIL-I08 | Workflow Handoff | `../contracts/asyncapi/mail-events.yaml` | Mail worker | Sends events to workflow-engine, messenger, calendar, drive. |
| MAIL-I09 | Reputation Dashboard | `../dashboards/dmarc-deliverability.json` | Tenant admin and ops | Complaint, bounce, DMARC, and abuse telemetry. |
| MAIL-I10 | Audit Event Bridge | `../contracts/asyncapi/mail-events.yaml` | Mail service | Emits sealed mail events. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| MAIL-D01 | SMTP receiver stack | Inbound receive, queueing, routing | Mail outage or spoof bypass | `../runbooks/smtp-queue-backup.md`. |
| MAIL-D02 | Rspamd or anti-phishing kernel | Spam, phishing, BEC classification | Inbound phishing bypass | `../policy/anti-phishing.cedar`. |
| MAIL-D03 | OpenDKIM/OpenBao | DKIM signing and key custody | Tenant impersonation | `../decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`. |
| MAIL-D04 | Cedar policy-engine | Tenant, dual-context, DLP, abuse gates | Broken access control | `../policy/tenant-scope.cedar`. |
| MAIL-D05 | Object storage | MIME body and attachment bytes | Data loss or exfiltration | `../slos/ediscovery-export-freshness.openslo.yaml`. |
| MAIL-D06 | Postgres mailbox store | Headers, folders, retention, legal hold | Cross-tenant mailbox leakage | RLS and tenant scope policy. |
| MAIL-D07 | Search index | Mailbox search | Plaintext leak or poisoning | Encrypted-token index controls. |
| MAIL-D08 | audit-chain | Sealed evidence | Repudiation and incident gaps | ADR-0003 and ADR-0263. |
| MAIL-D09 | observability | Detection and SLOs | Missed phishing, spam, BEC, or DLP signals | `../dashboards/abuse-defence-outcomes.json`. |
| MAIL-D10 | identity | OIDC, user context, step-up | Mailbox access takeover | Identity threat model and step-up policies. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| MAIL-B01 | Public SMTP inbound boundary | Internet sender | Inbound SMTP receiver | Phishing, spoofing, malware, queue DoS. |
| MAIL-B02 | SMTP submission boundary | Authenticated tenant user/service | Outbound SMTP worker | Spam outbound, stolen credentials, BEC. |
| MAIL-B03 | Mailbox access boundary | User agent, IMAP/JMAP/REST client | Mailbox read/write API | Mailbox exfiltration and context confusion. |
| MAIL-B04 | Tenant boundary | Tenant A mailbox/sending domain | Tenant B mailbox/sending domain | Cross-tenant data or reputation bleed. |
| MAIL-B05 | DKIM custody boundary | Mail worker | OpenBao signing key path | Signing-key theft or unauthorized signing. |
| MAIL-B06 | Deliverability boundary | Tenant mail stream | Shared IP pool and recipient MTAs | Reputation damage and spam amplification. |
| MAIL-B07 | DLP boundary | Message content | DLP scanner and quarantine workflow | PII/PHI leak, false release. |
| MAIL-B08 | Search/index boundary | Mailbox store | Search index | Search poisoning or plaintext leakage. |
| MAIL-B09 | Legal-hold boundary | Mailbox and retention worker | Legal-hold engine | Evidence deletion or over-disclosure. |
| MAIL-B10 | Workflow handoff boundary | Mail event | Workflow/messenger/calendar/drive | Cross-service injection and replay. |
| MAIL-B11 | Audit boundary | Mail state change | audit-chain emission bridge | Missing audit_id or wrong tenant_id. |
| MAIL-B12 | Personal/work boundary | Personal mailbox context | Professional mailbox context | Employer access to personal mail or reverse leakage. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-S01 | InboundPhishingSignal | MAIL-B01 | SPF/DKIM/DMARC bypass allows phish to appear tenant-trusted. | Credential theft or malware delivery. |
| MAIL-S02 | MailHeaderEnvelope | MAIL-B01 | Header-from and envelope-from mismatch hides impersonation. | Business email compromise. |
| MAIL-S03 | TenantDkimSigningKey | MAIL-B05 | Compromised per-tenant signing key signs attacker mail. | Tenant domain impersonation. |
| MAIL-S04 | SmtpSubmissionCredential | MAIL-B02 | Stolen mailbox or app password authenticates spam sender. | Reputation loss and tenant account abuse. |
| MAIL-S05 | MailWorkflowHandoff | MAIL-B10 | Forged mail event triggers workflow automation. | Downstream action spoofing. |
| MAIL-S06 | MailboxMessageBody | MAIL-B03 | Session attacker presents as mailbox owner. | Mailbox exfiltration. |
| MAIL-S07 | LegalHoldRecord | MAIL-B09 | Unauthorized actor claims legal/audit export identity. | Evidence disclosure. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-T01 | MailboxMessageBody | MAIL-B01 | Malware attachment or active content mutates downstream endpoint state. | Tenant compromise. |
| MAIL-T02 | MailHeaderEnvelope | MAIL-B01 | ARC or DKIM result fields are altered before classification. | Phishing bypass. |
| MAIL-T03 | TenantDmarcPolicy | MAIL-I05 | DMARC policy is weakened from reject to none. | Spoofing resurgence. |
| MAIL-T04 | OutboundReputationState | MAIL-B06 | Reputation score is manipulated to avoid throttling. | Spam outbound at scale. |
| MAIL-T05 | DlpQuarantineRecord | MAIL-B07 | Reviewer or attacker changes DLP verdict to release sensitive mail. | PII/PHI disclosure. |
| MAIL-T06 | LegalHoldRecord | MAIL-B09 | Hold-before-purge invariant is bypassed. | Evidence loss. |
| MAIL-T07 | MailboxSearchIndex | MAIL-B08 | Search index poisoning hides or invents messages. | eDiscovery and user search integrity loss. |
| MAIL-T08 | AuditEmissionEnvelope | MAIL-B11 | Mail event emitted without audit_id or with wrong tenant_id. | Repudiation and broken incident joins. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-R01 | MailHeaderEnvelope | MAIL-B02 | Sender denies sending disputed outbound message. | BEC or harassment case lacks proof. |
| MAIL-R02 | TenantDkimSigningKey | MAIL-B05 | Tenant denies key rotation or selector change. | Domain spoof investigation stalls. |
| MAIL-R03 | DlpQuarantineRecord | MAIL-B07 | Reviewer denies releasing quarantined message. | Compliance gap. |
| MAIL-R04 | LegalHoldRecord | MAIL-B09 | Actor denies creating or releasing legal hold. | Evidence chain unreliable. |
| MAIL-R05 | MailWorkflowHandoff | MAIL-B10 | Downstream service denies receiving mail-triggered event. | Automation incident ambiguity. |
| MAIL-R06 | OutboundReputationState | MAIL-B06 | Tenant disputes throttling due to abuse. | Customer trust and support escalation. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-I01 | MailboxMessageBody | MAIL-B03 | Shared mailbox or token grants overbroad read access. | Mailbox exfiltration. |
| MAIL-I02 | MailboxMessageBody | MAIL-B12 | Professional admin reads personal mailbox bytes. | Personal tenant privacy breach. |
| MAIL-I03 | TenantDkimSigningKey | MAIL-B05 | DKIM private key appears in logs, crash dumps, or export. | Tenant domain impersonation. |
| MAIL-I04 | MailboxSearchIndex | MAIL-B08 | Search index stores plaintext tokens or snippets. | Hidden content disclosure. |
| MAIL-I05 | DlpQuarantineRecord | MAIL-B07 | Quarantine review view reveals PHI/PII beyond need-to-know. | Compliance breach. |
| MAIL-I06 | AuditEmissionEnvelope | MAIL-B11 | ADR-0263 telemetry includes raw address, subject, or body. | Observability privacy breach. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-DOS01 | Inbound SMTP queue | MAIL-B01 | Queue flood exhausts receiver, storage, or scanner capacity. | Tenant mail delay. |
| MAIL-DOS02 | OutboundReputationState | MAIL-B06 | Spam campaign forces IP pool blocklist. | Tenant cannot send. |
| MAIL-DOS03 | DlpQuarantineRecord | MAIL-B07 | Attachment flood saturates DLP and malware scanners. | Delivery delay. |
| MAIL-DOS04 | MailboxSearchIndex | MAIL-B08 | Expensive search queries overload index. | Mailbox read degradation. |
| MAIL-DOS05 | TenantDkimSigningKey | MAIL-B05 | OpenBao/signing latency blocks outbound delivery. | Send outage. |
| MAIL-DOS06 | LegalHoldRecord | MAIL-B09 | Massive export request consumes object store and worker capacity. | eDiscovery outage. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| MAIL-E01 | SmtpSubmissionCredential | MAIL-B02 | Stolen user credential sends as executive or shared mailbox. | Business email compromise. |
| MAIL-E02 | TenantDmarcPolicy | MAIL-B04 | Tenant admin changes another tenant domain settings. | Cross-tenant spoofing. |
| MAIL-E03 | LegalHoldRecord | MAIL-B09 | Non-legal role exports privileged mailbox. | Unauthorized evidence disclosure. |
| MAIL-E04 | DlpQuarantineRecord | MAIL-B07 | Reviewer self-approves release without four-eyes. | DLP bypass. |
| MAIL-E05 | MailWorkflowHandoff | MAIL-B10 | Mail event triggers workflow under elevated service identity. | Cross-service privilege escalation. |
| MAIL-E06 | MailboxMessageBody | MAIL-B12 | Context split bypass exposes personal mail to work admin. | Privacy and compliance failure. |

## DREAD Scoring

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | MAIL-S03 | Per-tenant DKIM signing key compromised. | 10 | 8 | 7 | 10 | 8 | 43 |
| 2 | MAIL-E01 | BEC through stolen executive credential. | 10 | 8 | 8 | 8 | 8 | 42 |
| 3 | MAIL-S01 | SPF/DKIM/DMARC bypass phish accepted. | 9 | 9 | 8 | 8 | 7 | 41 |
| 4 | MAIL-I01 | Mailbox exfiltration through overbroad access. | 10 | 8 | 7 | 8 | 7 | 40 |
| 5 | MAIL-DOS02 | Tenant reputation destroyed by spam outbound. | 9 | 8 | 8 | 8 | 6 | 39 |
| 6 | MAIL-T05 | DLP verdict tampered to release PHI/PII. | 9 | 7 | 6 | 8 | 7 | 37 |
| 7 | MAIL-T06 | Legal hold bypass before purge. | 9 | 6 | 6 | 8 | 7 | 36 |
| 8 | MAIL-I03 | DKIM private key leaks in logs. | 9 | 6 | 6 | 8 | 6 | 35 |
| 9 | MAIL-DOS01 | SMTP queue flood. | 8 | 9 | 8 | 7 | 3 | 35 |
| 10 | MAIL-E06 | Personal/work boundary bypass. | 8 | 6 | 6 | 8 | 7 | 35 |
| 11 | MAIL-S05 | Forged mail workflow handoff. | 8 | 7 | 6 | 7 | 6 | 34 |
| 12 | MAIL-T03 | DMARC policy weakened. | 8 | 7 | 6 | 7 | 6 | 34 |
| 13 | MAIL-I04 | Plaintext search index leak. | 8 | 6 | 5 | 8 | 6 | 33 |
| 14 | MAIL-DOS05 | OpenBao/signing latency blocks outbound mail. | 8 | 7 | 5 | 9 | 3 | 32 |
| 15 | MAIL-R03 | Quarantine release lacks non-repudiation. | 7 | 7 | 5 | 7 | 5 | 31 |

## Attack Trees

### Opportunistic Adversary: Inbound Phishing Bypass

- Goal: deliver credential phish to tenant inbox.
  - Path O1: register lookalike sender domain.
  - Path O2: pass SPF with attacker infrastructure.
  - Path O3: craft DKIM alignment that exploits relaxed canonicalization assumptions.
  - Path O4: exploit DMARC policy set to none or monitoring only.
  - Path O5: include URL or attachment that avoids scanner signatures.
- Required break: anti-phishing Cedar and classifier fail to quarantine.
- Required break: DMARC rollout monitoring does not flag bypass pattern.
- Detection pivot: `MessageReceived`, `DlpQuarantined`, `AbuseDefenceSpoofDetected`.

### Targeted Adversary: Business Email Compromise

- Goal: convince finance or HR user to execute fraudulent action.
  - Path T1: compromise executive mailbox or spoof display name.
  - Path T2: observe prior thread and reply with matching tone.
  - Path T3: send payment instruction or credential request.
  - Path T4: bypass DLP and BEC classifier.
  - Path T5: trigger workflow or payment handoff.
- Required break: mailbox access anomaly detection misses unusual login.
- Required break: BEC classifier does not flag payment-instruction pattern.
- Detection pivot: `MessageSent`, `MailWorkflowHandoffCreated`, `AbuseDefenceWatermarkRecovered`.

### Insider Adversary: DKIM Custody Abuse

- Goal: sign unauthorized mail as tenant domain.
  - Path I1: obtain OpenBao policy or worker identity.
  - Path I2: read or invoke per-tenant DKIM signing key.
  - Path I3: send mail through alternate path with valid signature.
  - Path I4: suppress `DkimKeyRotated` or signing anomaly telemetry.
  - Path I5: blame tenant misconfiguration.
- Required break: `../policy/tenant-scope.cedar` and OpenBao path scoping fail.
- Required break: key use lacks audit_id.
- Detection pivot: `DkimKeyRotated`, `MessageSent`, `OfficeBoundaryAttemptEvaluated`.

### Nation-State Adversary: Mailbox Exfiltration at Scale

- Goal: extract sensitive mail across regulated tenant.
  - Path N1: compromise identity session or admin.
  - Path N2: enumerate mailboxes through JMAP/IMAP.
  - Path N3: export bodies and attachments below rate thresholds.
  - Path N4: poison search index to hide queries.
  - Path N5: purge or tamper legal-hold records.
- Required break: context split, tenant scope, and eDiscovery controls all fail.
- Required break: audit-chain correlation misses download/export pattern.
- Detection pivot: `MailContextSwitched`, `EDiscoveryExportSealed`, `AbuseDefenceScrapeBlocked`.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| MAIL-S01 | SPF/DKIM/DMARC/ARC result evaluation with anti-phishing Cedar gate. | ADR-0297 | `../policy/anti-phishing.cedar`; `../runbooks/dmarc-rollout-monitoring.md`. |
| MAIL-S02 | Header alignment checks and display-name anomaly scoring. | ADR-0297 | `../catalog/oya-mail-anti-phishing-kernel.yaml`. |
| MAIL-S03 | Per-tenant DKIM key custody in OpenBao with rotation and audit emission. | ADR-0243, ADR-0263 | `../decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`; `../runbooks/dkim-key-rotation.md`. |
| MAIL-S04 | SMTP submission requires authenticated identity and tenant-bound sender policy. | ADR-0244 | `../policy/abuse-defence.cedar`; `../policy/tenant-scope.cedar`. |
| MAIL-S05 | Workflow events are sealed and consumed through tenant-scoped topics. | ADR-0003 | `../contracts/asyncapi/mail-events.yaml`. |
| MAIL-I01 | Mailbox reads pass tenant, context, and mailbox ownership checks. | ADR-0243 | `../policy/dual-context-isolation.md`; `../policy/tenant-scope.cedar`. |
| MAIL-I03 | DKIM private keys are never logged; signing uses key handles. | ADR-0263 | `../iac/openbao-policy.yaml`; `../iac/secret-bindings.yaml`. |
| MAIL-DOS01 | SMTP queue backup and rate limiting. | ADR-0297 | `../runbooks/smtp-queue-backup.md`. |
| MAIL-DOS02 | Per-tenant reputation tracker and throttle. | ADR-0297 | `../dashboards/dmarc-deliverability.json`. |
| MAIL-T05 | DLP quarantine release requires reviewer role and audit trail. | ADR-0243 | `../policy/phi-dlp.cedar`; `../runbooks/dlp-quarantine-release.md`. |
| MAIL-T06 | Hold-before-purge invariant and sealed legal-hold events. | ADR-0003 | `../contracts/asyncapi/mail-events.yaml`. |
| MAIL-E06 | Personal/work context guard. | ADR-0313, ADR-0319 | `../policy/dual-context-isolation.md`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| MAIL-RR01 | DMARC alignment can be valid for malicious domains. | axis-mail | BEC classifier, sender reputation, and user-report feedback. | BEC incident. |
| MAIL-RR02 | Per-tenant DKIM key compromise can sign believable messages until rotation completes. | axis-mail | Rapid selector rotation and recipient advisory process. | Any signing anomaly. |
| MAIL-RR03 | User endpoint malware can read mailbox after legitimate auth. | ops-security | Step-up, impossible-travel detection, and session revocation. | Mailbox export spike. |
| MAIL-RR04 | DLP may false-negative embedded image text or encrypted archives. | council-privacy | OCR handoff to drive/intelligence and quarantine escalation. | DLP false-negative. |
| MAIL-RR05 | Shared IP pools can suffer collateral reputation damage. | axis-mail | Per-tenant reputation partitioning and pool isolation. | Complaint-rate threshold. |
| MAIL-RR06 | Legal hold exports intentionally expose plaintext to approved roles. | ops-legal | Four-eyes approval and export watermarking. | eDiscovery export. |
| MAIL-RR07 | Search index tokenization can leak term frequency. | axis-mail | Per-tenant encrypted index and access-limited search telemetry. | Search architecture change. |
| MAIL-RR08 | External MTAs do not always honor TLS or DMARC strictly. | axis-mail | MTA-STS/TLS-RPT monitoring and policy hardening. | TLS-RPT degradation. |
| MAIL-RR09 | Workflow consumers can mishandle mail-originated events. | workflow-engine owner | Sealed event contract and tenant-scoped topics. | New consumer added. |
| MAIL-RR10 | High-volume spam defense can delay legitimate mail. | ops-sre-reliability | SLO alerting and manual allowlist protocol. | Delivery latency burn. |

## Specific Telemetry for Detection

ADR-0263 detection telemetry must include `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` for state-changing
mail events. Cedar denials include the policy id and denied reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| MAIL-S01 | DMARC fail, DKIM fail, SPF fail, ARC anomaly, URL verdict. | `MessageReceived`, `AbuseDefenceSpoofDetected` | Inbound phishing or spoof bypass. |
| MAIL-S03 | Signing call volume, key selector drift, OpenBao path access. | `DkimKeyRotated`, `OfficeBoundaryAttemptEvaluated` | Signing-key custody compromise. |
| MAIL-S04 | SMTP submit from unusual ASN, new device, high volume. | `MessageSent`, `AbuseDefenceCredentialStuffing` | Stolen account used for spam. |
| MAIL-E01 | Executive sender, payment instruction, new recipient, urgency language. | `MessageSent`, `AbuseDefenceWatermarkRecovered` | BEC attempt. |
| MAIL-DOS02 | Complaint rate, bounce rate, blocklist hit, IP pool throttle. | `MailDeliverabilityReputationChanged`, `AbuseDefenceQuotaExceeded` | Tenant reputation attack or spam outbound. |
| MAIL-T05 | DLP verdict changed, release requested, reviewer changed. | `DlpQuarantined`, `DlpReleased`, `OfficeBoundaryClearanceApproved` | DLP tamper or inappropriate release. |
| MAIL-T06 | Retention expired while legal hold active. | `LegalHoldEngaged`, `RetentionExpired` | Hold-before-purge invariant violation. |
| MAIL-I01 | High mailbox download volume or atypical search/read pattern. | `AbuseDefenceScrapeBlocked`, mailbox read audit event | Mailbox exfiltration. |
| MAIL-I02 | Cross-context read denied. | `ConglomeratePersonalTenantBoundaryRefused`, `MailContextSwitched` | Personal/work boundary probe. |
| MAIL-DOS01 | SMTP queue depth, deferred delivery age, scanner backlog. | `AbuseDefenceRateLimitHit`, `MessageBounced` | Queue DoS. |
| MAIL-DOS05 | DKIM signing latency, OpenBao error, selector unavailable. | `AbuseDefenceVendorOutage`, `DkimKeyRotated` | Signing dependency outage. |
| MAIL-E05 | Mail workflow event replay or consumer tenant mismatch. | `MailWorkflowHandoffCreated`, `OfficeBoundaryAttemptDenied` | Cross-service privilege escalation. |

## Threat Coverage Ledger

### MAIL-COV01: Inbound phishing coverage

- Threats covered: MAIL-S01, MAIL-S02, MAIL-T02.
- Asset coverage: InboundPhishingSignal and MailHeaderEnvelope.
- Boundary coverage: MAIL-B01.
- Required control evidence: SPF, DKIM, DMARC, ARC evaluation, classifier result, quarantine action.
- Detection evidence: `MessageReceived`, `DlpQuarantined`, and `AbuseDefenceSpoofDetected`.

### MAIL-COV02: Outbound spam coverage

- Threats covered: MAIL-S04, MAIL-DOS02, MAIL-R06.
- Asset coverage: SmtpSubmissionCredential and OutboundReputationState.
- Boundary coverage: MAIL-B02 and MAIL-B06.
- Required control evidence: sender authorization, per-tenant throttle, IP pool isolation, complaint tracking.
- Detection evidence: `MessageSent`, `MailDeliverabilityReputationChanged`, and `AbuseDefenceQuotaExceeded`.

### MAIL-COV03: BEC coverage

- Threats covered: MAIL-E01, MAIL-S02, MAIL-I01.
- Asset coverage: MailboxMessageBody, MailHeaderEnvelope, and SmtpSubmissionCredential.
- Boundary coverage: MAIL-B02 and MAIL-B03.
- Required control evidence: identity session risk, payment-language classifier, executive impersonation rule.
- Detection evidence: `MessageSent`, mailbox access anomaly, and BEC classifier alert.

### MAIL-COV04: DKIM custody coverage

- Threats covered: MAIL-S03, MAIL-I03, MAIL-DOS05.
- Asset coverage: TenantDkimSigningKey.
- Boundary coverage: MAIL-B05.
- Required control evidence: OpenBao policy, selector rotation, key handle use, no raw key logging.
- Detection evidence: `DkimKeyRotated`, OpenBao audit, and signing anomaly dashboard.

### MAIL-COV05: DLP coverage

- Threats covered: MAIL-T05, MAIL-I05, MAIL-E04.
- Asset coverage: DlpQuarantineRecord and MailboxMessageBody.
- Boundary coverage: MAIL-B07.
- Required control evidence: reviewer separation, PHI DLP Cedar policy, quarantine and release audit trail.
- Detection evidence: `DlpQuarantined`, `DlpReleased`, and `OfficeBoundaryClearanceApproved`.

### MAIL-COV06: Legal hold coverage

- Threats covered: MAIL-T06, MAIL-R04, MAIL-E03.
- Asset coverage: LegalHoldRecord.
- Boundary coverage: MAIL-B09.
- Required control evidence: hold-before-purge invariant, four-eyes export, chain-of-custody seal.
- Detection evidence: `LegalHoldEngaged`, `LegalHoldReleased`, and `EDiscoveryExportSealed`.

### MAIL-COV07: Search leakage coverage

- Threats covered: MAIL-T07, MAIL-I04, MAIL-DOS04.
- Asset coverage: MailboxSearchIndex.
- Boundary coverage: MAIL-B08.
- Required control evidence: encrypted tokens, tenant index partitioning, query rate limits.
- Detection evidence: search latency SLO, scrape block, and query anomaly log.

### MAIL-COV08: Workflow handoff coverage

- Threats covered: MAIL-S05, MAIL-R05, MAIL-E05.
- Asset coverage: MailWorkflowHandoff.
- Boundary coverage: MAIL-B10 and MAIL-B11.
- Required control evidence: sealed mail event, tenant-scoped topic, idempotency key, consumer allowlist.
- Detection evidence: `MailWorkflowHandoffCreated`, `OfficeBoundaryAttemptDenied`, and audit_id join.

### MAIL-COV09: Personal/work context coverage

- Threats covered: MAIL-I02, MAIL-E06.
- Asset coverage: MailboxMessageBody and MailHeaderEnvelope.
- Boundary coverage: MAIL-B12 and MAIL-B04.
- Required control evidence: dual-context isolation policy, tenant-scope policy, context switch audit.
- Detection evidence: `MailContextSwitched` and `ConglomeratePersonalTenantBoundaryRefused`.

### MAIL-COV10: Telemetry privacy coverage

- Threats covered: MAIL-I06, MAIL-T08.
- Asset coverage: AuditEmissionEnvelope.
- Boundary coverage: MAIL-B11.
- Required control evidence: ADR-0263 PII scrubbing at emission, audit_id on state changes, schema version.
- Detection evidence: log-schema validation, PII scrubber failure, and audit-chain completeness SLO.

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| Account compromise and mailbox takeover | `../runbooks/account-compromise-recovery.md` |
| DKIM key compromise or rotation | `../runbooks/dkim-key-rotation.md` |
| DMARC rollout or phishing bypass | `../runbooks/dmarc-rollout-monitoring.md` |
| DLP quarantine release | `../runbooks/dlp-quarantine-release.md` |
| PHI leak recovery | `../runbooks/phi-leak-recovery.md` |
| Mail bot or abuse score recalibration | `../runbooks/mail-bot-score-recalibration.md` |
| SMTP queue backup | `../runbooks/smtp-queue-backup.md` |
| Spam rule rollback | `../runbooks/spam-rule-rollback.md` |
| Mailbox restore from backup | `../runbooks/mailbox-restore-from-backup.md` |
| End-to-end encryption recovery | `../runbooks/e2e-encryption-key-recovery.md` |

## Cross-References

- Root service architecture: `../ARCHITECTURE.md`.
- Product requirements: `../PRD.md`.
- Mail events contract: `../contracts/asyncapi/mail-events.yaml`.
- Mail OpenAPI contract: `../contracts/openapi/mail.yaml`.
- DKIM custody decision: `../decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`.
- Anti-phishing kernel catalog: `../catalog/oya-mail-anti-phishing-kernel.yaml`.
- JMAP frontend catalog: `../catalog/oya-mail-jmap-frontend-rest.yaml`.
- PHI DLP adapter catalog: `../catalog/oya-mail-phi-dlp-adapter-kernel.yaml`.
- Abuse-defence policy: `../policy/abuse-defence.cedar`.
- Anti-phishing policy: `../policy/anti-phishing.cedar`.
- PHI DLP policy: `../policy/phi-dlp.cedar`.
- Tenant scope policy: `../policy/tenant-scope.cedar`.
- Dual-context policy: `../policy/dual-context-isolation.md`.
- DMARC dashboard: `../dashboards/dmarc-deliverability.json`.
- Abuse defence dashboard: `../dashboards/abuse-defence-outcomes.json`.
- ADR-0263 observability emission contract: `../../../docs/decisions/ADR-0706-observability-live-apex.md`.
- ADR-0243 Cedar as universal gate: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244 tenant as universal scoping primitive: `../../../docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0297 abuse defence baseline: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0319 front/middle/back-office information barrier: `../../../docs/decisions/ADR-0709-general-live-apex.md`.

## Checkpoint Notes

- This document does not modify mail decisions or runbooks.
- It references existing playbooks for incident handling rather than editing them.
- It assumes DKIM/SPF/DMARC controls remain tenant-scoped and audit-emitted.
- It accepts that phishing and BEC controls are layered detections, not a single deterministic proof.
