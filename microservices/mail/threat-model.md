---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-mail + ops-security + council-privacy
deciders: council-architecture, ops-security, axis-mail, council-privacy, ops-legal
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154 + RFC 7208 (SPF) + RFC 6376 (DKIM) + RFC 7489 (DMARC) + RFC 8460 (TLS-RPT) + RFC 8461 (MTA-STS)
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140, ADR-0208, ADR-0215]
related_specs: [/specs/microservices/mail.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Layer-A version change or SMTP/IMAP/JMAP protocol change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12 (보호조치)", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 전자문서법 Arts. 5/6/7", "KR-FSS 전자금융감독규정 (when financial-services tenant)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 (Administrative Safeguards)", "§164.310 (Physical Safeguards)", "§164.312 (Technical Safeguards)", "§164.314 (Organizational Requirements)", "§164.316 (Policies and Procedures)", "§164.502 (Permitted Uses)", "§164.504(e) (Business Associate)", "HITECH §13402 (breach notification)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50 (transfers)", "eIDAS 910/2014 (when audit-chain seals AdES)", "NIS2 2022/2555", "ePrivacy Directive 2002/58/EC (e-mail privacy)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13 (esp. APP 6, 8, 11)", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-10", "RBI Master Direction on Outsourcing of IT Services 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: mail µservice

## Purpose

Identify, classify, and mitigate threats to the mail µservice's confidentiality, integrity, availability, and privacy posture. Mail is one of the most attractive attack surfaces in any enterprise system (SMTP relay abuse, phishing, mailbox-content exfiltration, legal-hold tampering); this document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, HIPAA OCR (when pack-us-healthcare active), and KR-FSS (when KR financial-services tenants present) at first-tenant onboarding.

## Scope

### In-scope

All components introduced by ADR-0135 (Connect dissolution) and ADR-0131 (per-microservice flat layout) for the mail µservice, deployed in a dedicated mail Kubernetes namespace (per `iac/kustomize/base`).

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postfix (SMTP receive + submit) | `oya-mail-mailbox-store-*` (~11 crates) |
| Dovecot (IMAP) — or oyatie Rust replacement | `oya-mail-inbound-smtp-*` (~7 crates) |
| Rspamd (spam/phishing detection) | `oya-mail-outbound-smtp-*` (~7 crates) |
| OpenDKIM (DKIM sign/verify) | `oya-mail-imap-frontend-*` (~8 crates) |
| Postgres (mailbox metadata, RLS per-tenant) | `oya-mail-search-index-*` (~8 crates) |
| S3-compatible (MIME blob CAS) | `oya-mail-legal-hold-*` (~9 crates) |
| Tantivy (or Elasticsearch) — encrypted search index | `oya-mail-retention-policy-*` (~6 crates) |
| KMS (per-tenant DEK envelope encryption) | `oya-mail-dual-context-isolation-*` (~6 crates) |
| Per-tenant DKIM keys (OpenBao-stored) | tenant SMTP IP pool reputation tracker |
| MTA-STS + TLS-RPT publication | mail-to-Workflow handoff orchestrator |

### Out-of-scope

- Threats to underlying Kubernetes / hyperscaler IaaS — owned by `cloud-k8s` threat model.
- Threats to `audit-chain` µservice (mail emits to audit-chain; audit-chain owns its own threats).
- Threats to OpenBao secret-manager (owned by `cloud-secrets`).
- Threats to GitHub Actions (owned by `governance`).
- Threats to messenger / calendar / community sub-products of dissolved Connect (each owns its own threat model).
- Threats to the workflow-engine itself (owned by `workflow-engine`).
- Threats to upstream Postfix / Dovecot / Rspamd CVE pipeline — tracked by `cloud-secrets` supply-chain lane; this document inherits.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   External senders (SMTP 25)        Tenant employees (IMAP/JMAP/REST)      │
│         │                                  │                               │
│         │ STARTTLS opportunistic           │ TLS 1.3 mandatory             │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy/Istio gateway + dedicated SMTP edge LB) ─────┐   │
│  │  - TLS termination on submission ports                              │   │
│  │  - SMTP MTA-STS + TLS-RPT compliance                                │   │
│  │  - WAF on REST surface (rate-limit + OWASP CRS)                     │   │
│  │  - DDOS protection (provider + Cloudflare for REST; IP-pool for SMTP)│  │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Mail µservice namespace (per pack) ───────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: Internet → SMTP receivers (port 25 + 465 + 587)         │
│                                                                            │
│  ┌─ inbound-smtp (Postfix + oya-mail-inbound-smtp-rest sidecar) ────┐      │
│  │  - DKIM/SPF/DMARC verification                                   │      │
│  │  - Rspamd spam/phishing                                          │      │
│  │  - Per-recipient tenant routing                                  │      │
│  │  - Rate-limit per source IP + per sender domain                  │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 2: Authenticated user → SMTP submission (587)              │
│                                                                            │
│  ┌─ outbound-smtp ─────────────────────────────────────────────────┐      │
│  │  - SASL auth required (no anonymous relay)                      │      │
│  │  - DKIM sign with per-tenant key from OpenBao                   │      │
│  │  - Per-tenant IP pool + reputation                              │      │
│  │  - DLP scan + abuse classifier                                  │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 3: Authenticated user → IMAP/JMAP/REST mailbox read        │
│                                                                            │
│  ┌─ imap-frontend ────────────────────────────────────────────────┐       │
│  │  - OIDC + SCRAM-SHA-256 (IMAP) / OIDC bearer (JMAP/REST)       │       │
│  │  - Per-user mailbox scope; cross-mailbox refused                │       │
│  │  - Cedar policy: dual-context boundary enforced                 │       │
│  └────────────────────────────────────────────────────────────────┘       │
│                                                                            │
│  Trust boundary 4: Internal worker → Postgres + S3 + KMS                   │
│                                                                            │
│  ┌─ mailbox-store + search-index + legal-hold + retention-policy ──┐      │
│  │  - mTLS Postgres connection; per-tenant RLS                     │      │
│  │  - S3 SSE-KMS envelope; per-tenant DEK                          │      │
│  │  - Search index: encrypted-token only                           │      │
│  │  - Workers: long-lived service-account; SPIFFE identity         │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 5: Legal-hold engine → mailbox + retention scheduler       │
│                                                                            │
│  ┌─ legal-hold ────────────────────────────────────────────────────┐      │
│  │  - Four-eyes approval for plaintext disclosure                  │      │
│  │  - Hold-before-purge invariant (kernel-enforced)                │      │
│  │  - Audit-chain seal on every hold action                        │      │
│  │  - eDiscovery export: signed bundle + chain-of-custody          │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 6: Personal vs Professional context (kernel layer)         │
│                                                                            │
│  ┌─ dual-context-isolation ────────────────────────────────────────┐      │
│  │  - ContextBoundaryGuard called on every cross-API surface       │      │
│  │  - Personal mailbox never exposed via Professional API          │      │
│  │  - Org admin cannot decrypt Personal-pillar blobs               │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **Internet → SMTP receivers** (TLS/STARTTLS, DKIM/SPF/DMARC, rate-limit, abuse).
2. **Authenticated user → SMTP submission** (SASL auth, DKIM sign, DLP, per-tenant pool).
3. **Authenticated user → IMAP/JMAP/REST mailbox read** (OIDC + Cedar dual-context).
4. **Internal worker → Postgres + S3 + KMS** (mTLS, RLS, SSE-KMS, SPIFFE).
5. **Legal-hold engine → mailbox + retention** (four-eyes, hold-before-purge, audit).
6. **Personal vs Professional context** (kernel-layer ContextBoundaryGuard).

## Assets & Data Classification

Per Bominal ADR-0028 (data-class taxonomy) and `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Mailbox MIME body + attachments (Professional context) | `PII_IDENTIFYING` + sometimes `PHI` (pack-us-healthcare) | High | per retention policy; HIPAA 6y when PHI; KR-FSS 5y when KR financial | S3 + KMS DEK |
| Mailbox MIME body + attachments (Personal context) | `PII_IDENTIFYING` + user-controlled | High | per user; org cannot extend | S3 + user-derived DEK |
| Mailbox headers (Professional) | `PII_IDENTIFYING` (To/From/CC/BCC + Subject) | High | mirrors body | Postgres + KMS DEK |
| Mailbox headers (Personal) | `PII_IDENTIFYING` | High | mirrors body | Postgres + user-derived DEK |
| Mailbox metadata (Mailbox, Folder, Thread structure) | `PII_QUASI_IDENTIFIER` | Medium | per retention | Postgres |
| Search index tokens | `BEHAVIORAL_TENANT_PRODUCT` (encrypted tokens; no plaintext) | Medium | mirrors mailbox | Tantivy/Elasticsearch + per-tenant key |
| Retention policy + class definitions | `INTERNAL_ONLY` | Low | append-only | Postgres |
| Legal hold scope + approvals | `AUDIT` + `PII_IDENTIFYING` (scope queries) | High | indefinite (hold lifecycle) | Postgres + audit-chain |
| eDiscovery export bundle | `AUDIT` + `PII_IDENTIFYING` (plaintext disclosure under four-eyes) | Critical | per legal hold lifecycle | S3 + KMS + Ed25519 seal |
| Chain-of-custody seal | `AUDIT` | High | indefinite | audit-chain |
| DKIM private keys (per-tenant) | `SECRET` | Critical | OpenBao with 90d rotation | OpenBao |
| TLS certificates (SMTP + IMAP + REST edges) | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao + cert-manager |
| Per-tenant DEK (envelope key) | `SECRET` | Critical | KMS-wrapped; rotated on `KmsKeyRotated` event | KMS |
| Per-tenant SMTP IP pool reputation | `BEHAVIORAL_TENANT_PRODUCT` | Medium | rolling 90d window | Postgres |
| Mail-to-Workflow handoff audit records | `AUDIT` | High | indefinite | audit-chain |
| SMTP submission credentials (SASL) | `SECRET` | Critical | OpenBao; rotated 30d | OpenBao |
| Audit-chain seal records | `AUDIT` | High | append-only; immutable | audit-chain µservice |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External sender (Internet SMTP) | Untrusted external | none (SMTP 25 is open by RFC); DKIM/SPF/DMARC verify | Send to tenant mailbox after abuse classification |
| End user (employee) | Semi-trusted external | OIDC + MFA + SCRAM-SHA-256 (IMAP-direct) or OIDC bearer (JMAP/REST) | Read own mailbox(es) (Personal + Professional); send via outbound-smtp; switch persona |
| Mail admin (per tenant) | Trusted internal-to-tenant | OIDC + MFA + tenant admin scope | Configure domains, aliases, retention policies, deliverability dashboard view |
| Compliance officer (per tenant) | Trusted internal-to-tenant | OIDC + MFA + compliance-officer scope + four-eyes peer for plaintext disclosure | Open/release legal holds (within scope); request eDiscovery export |
| Workflow operator (per tenant) | Semi-trusted internal-to-tenant | OIDC + MFA + workflow scope | Trigger mail-to-workflow handoff with explicit policy basis |
| Mail-µservice operator (oyatie SRE) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Layer-A operations; no mailbox-content access without JIT 2-person rule |
| Mail-µservice worker (long-lived service) | Trusted internal | SPIFFE identity | Read Postgres/S3/Tantivy per port traits; emit audit-chain |
| External auditor | Read-only external on time-boxed window | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain + retention ledger; cannot read mail content |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Code-review only; no runtime access |
| Attacker — opportunistic | Untrusted | none | SMTP relay abuse scans; phishing attempts; IMAP brute-force; assume always present |
| Attacker — targeted | Untrusted | none | APT-grade; supply-chain awareness; assume present for prod-tier surfaces |
| Attacker — state-sponsored (regulated tenants) | Untrusted | none | Highest-skill threat actor; assume present for KR-FSS and HIPAA tenants |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure retention/hold/DKIM; mitigated by PR-review + LEAN gates |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality; mitigated by 2-person rule + audit-chain + four-eyes for plaintext |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Attacker spoofs sender identity (envelope-from mismatch)**
- Asset: Inbound SMTP receivers; mailbox integrity
- Likelihood: H (commodity; daily) / Impact: M (phishing reaches end user) / Risk: **H**
- Mitigations:
  - DKIM verification on inbound; failures audit-emitted (`mail_dkim_verify_fail_total`).
  - SPF check on envelope-from; soft-fail records flagged; hard-fail rejected.
  - DMARC policy honoured per sender domain; `p=reject` causes inbound refusal; `p=quarantine` causes spam-folder.
  - Rspamd ARC-Authentication-Results header verification + heuristic scoring.
  - Per-tenant policy: tenant may tighten thresholds (e.g., strict DMARC enforce).
- Owner: axis-mail + ops-security
- Residual: M (DKIM/SPF/DMARC defeats most opportunistic spoofing; targeted attacks via lookalike domains still possible)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.14, A.8.23; GDPR Art. 32(1)(b); KR PIPA Art. 29; RFC 6376/7208/7489

**T-S-02 — Open-relay abuse (attacker uses oyatie outbound-smtp to send unauthorised mail)**
- Asset: Outbound SMTP submission; tenant SMTP IP reputation
- Likelihood: M / Impact: H (reputation collapse + abuse complaints + IP blocklisting) / Risk: **H**
- Mitigations:
  - SMTP submission (port 587) requires SASL authentication; no anonymous relay.
  - Per-mailbox rate limit on outbound submission; abuse heuristics flag burst-from-single-user.
  - DKIM signature on every outbound message; tenant key compromise rotates within OpenBao.
  - LEAN check `oya-check-smtp-no-open-relay` (NEW) validates Postfix config + asserts `smtpd_relay_restrictions` does not include `permit_mynetworks` without `permit_sasl_authenticated` constraint.
  - Reputation score: per-tenant IP pool tracked; abuse triggers automatic IP-pool quarantine.
- Owner: axis-mail + ops-deliverability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3; GDPR Art. 32; RFC 5321

**T-S-03 — Tenant-A submits SMTP claiming envelope-from of Tenant-B**
- Asset: Cross-tenant mailbox boundary; tenant DKIM key
- Likelihood: M / Impact: H (impersonation; reputation cross-pollution) / Risk: **H**
- Mitigations:
  - SASL-authenticated user's tenant_id bound at SMTP submission session; envelope-from MUST match an alias owned by user.tenant_id.
  - DKIM signing uses key bound to user.tenant_id; user cannot specify a different signing key.
  - LEAN check `oya-check-smtp-sasl-tenant-binding` validates Postfix config + asserts sender-permission tables match SASL-tenant-binding.
  - Audit-emit on mismatch (`mail_smtp_envelope_tenant_mismatch_total`).
- Owner: axis-mail + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 32

**T-S-04 — IMAP/SMTP brute-force credential stuffing**
- Asset: Mailbox passwords / OIDC tokens
- Likelihood: H (commodity) / Impact: M-H (per-mailbox compromise) / Risk: **H**
- Mitigations:
  - Per-IP rate-limit (10 failed auth/min → 1h lockout per IP).
  - Per-mailbox failed-auth lockout (10 failures → 15min lock; escalating).
  - CAPTCHA cliff at sustained brute-force pattern.
  - SCRAM-SHA-256 for IMAP-direct (resists password-spray); OIDC bearer with MFA for JMAP/REST.
  - Per-tenant policy: tenant may require MFA always, disable IMAP-direct, force OAuth-only.
  - `mail_imap_auth_fail_total` metric; alert at burst threshold.
- Owner: axis-mail + ops-security
- Residual: M (commodity attack baseline; never zero)
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.17, A.8.5, A.9.4; GDPR Art. 32; OWASP API Top 10 #2

**T-S-05 — Compliance officer impersonation to engage unauthorised hold or export**
- Asset: Legal-hold engine; eDiscovery export
- Likelihood: L / Impact: H (privacy breach via fraudulent disclosure) / Risk: **M**
- Mitigations:
  - Four-eyes rule for plaintext disclosure: two distinct compliance-officer principals must both approve before export bundle decrypts.
  - Approval audit-chained with both signatures (Ed25519 each).
  - Mail admin alone cannot engage hold or export — distinct compliance-officer scope required.
  - Time-boxed approval session (5 min); expires if second approver does not concur.
- Owner: council-privacy + axis-mail + ops-legal
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.5.18, A.8.2; GDPR Arts. 28, 32; KR PIPA Art. 23; HIPAA §164.502(b); ADR-0215

**T-S-06 — Attacker forges chain-of-custody seal**
- Asset: eDiscovery export bundle
- Likelihood: L / Impact: H (export legally unusable; reputation) / Risk: **M**
- Mitigations:
  - Seal is Ed25519 over `(export_id, scope, requested_by, approved_by, blob_digest, expires_at, executed_at)`.
  - Verifier re-derives digest from source blocks; provider-asserted digest mismatched against re-derived value triggers quarantine.
  - Audit-chain Merkle proof in seal anchors to global root.
  - Signing key in OpenBao + HSM-backed where available; rotated 90d.
- Owner: ops-security + ops-legal
- Residual: L
- Frameworks: SOC 2 CC4.1, CC7.4; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2); eIDAS 910/2014; KR 전자문서법 Art. 5

### Tampering (T)

**T-T-01 — In-transit mailbox content tampering (MITM on SMTP/IMAP)**
- Asset: SMTP / IMAP / JMAP / REST in-transit traffic
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - TLS 1.3 mandatory on SMTP submission (587), IMAP (993), JMAP/REST.
  - MTA-STS published per tenant + enforced; TLS-RPT collects reporting.
  - STARTTLS opportunistic on inbound SMTP (25) per RFC 8314; many senders downgrade — accept and tag in DKIM-verify.
  - Certificate pinning for highest-priority tenants.
  - mTLS internally between mail-µservice components.
- Owner: ops-security + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.7; ISO 27001 A.5.14, A.8.24; GDPR Art. 32(1)(b); RFC 8314/8461/8460

**T-T-02 — Object-storage MIME blob tampering (S3-backend)**
- Asset: MIME blob CAS
- Likelihood: L / Impact: H (content integrity loss) / Risk: **M**
- Mitigations:
  - S3 SSE-KMS encryption with per-tenant DEK; bucket policy WORM (S3 Object Lock Compliance mode where supported).
  - MIME blobs are content-addressable: blob_id = sha256(ciphertext); on read, recompute and compare.
  - Bucket access via service-account IAM only; no human direct access without ops-security JIT 2-person rule.
  - Periodic block-validator job: sample-validates blob digests.
- Owner: cloud-secrets + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.8.11, A.8.12, A.8.24; GDPR Art. 32

**T-T-03 — Retention policy tampering (reduce retention to permit deletion)**
- Asset: Retention policy + ledger
- Likelihood: L (insider-malicious) / Impact: H (regulatory retention floor breached) / Risk: **M**
- Mitigations:
  - LEAN check `oya-check-retention-floor-conformance` refuses retention configs below statutory floor (per pack: KR 5y for KR-FSS, HIPAA 6y for audit, etc.).
  - Retention ledger is append-only; modifications produce a new entry, never overwrite.
  - Any retention reduction requires 2-person rule + audit-chain emission.
  - Legal hold blocks retention regardless of policy change (hold-before-purge invariant).
- Owner: axis-mail + council-privacy
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.33, A.8.32; GDPR Art. 5(1)(e); HIPAA §164.530(j); KR 상법; KR PIPA Art. 28

**T-T-04 — Legal-hold record tampering (release hold without authorisation)**
- Asset: Legal hold lifecycle
- Likelihood: L / Impact: H (hold bypass; held material deleted) / Risk: **M**
- Mitigations:
  - Hold engage/release requires distinct compliance-officer scope; four-eyes for sensitive scopes.
  - Hold state stored in Postgres with append-only ledger; release writes a new "released" row, never deletes the "engaged" row.
  - Hold lifecycle audit-chained Ed25519.
  - Retention scheduler re-validates hold state on every sweep (not cached).
- Owner: council-privacy + axis-mail + ops-legal
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Arts. 5(2), 17; eIDAS 910/2014

**T-T-05 — DKIM key tampering (replace tenant DKIM private key with attacker's)**
- Asset: Per-tenant DKIM key in OpenBao
- Likelihood: L / Impact: H (impersonation of tenant outbound mail) / Risk: **M**
- Mitigations:
  - OpenBao key-issuance requires 2-person rule for production tenants.
  - DKIM key rotation cadence 90d; CI lane `oya-check-dkim-key-rotation-conformance` refuses keys older than rotation window.
  - DKIM public-key DNS records monitored externally; unexpected changes alert ops-deliverability.
  - Per-tenant audit-chain seal on every DKIM key rotation.
- Owner: ops-security + ops-deliverability
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.17, A.8.7, A.8.24; GDPR Art. 32; RFC 6376

**T-T-06 — Audit-chain seal tampering on mail events**
- Asset: Audit-chain seals for mail events
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Mail µservice emits seals; audit-chain µservice owns Merkle anchoring.
  - Per Bominal ADR-0028: audit-chain Merkle root sealed at clock + logical-clock cadence.
  - Cross-validation: independent reader recomputes Merkle path.
- Owner: audit-chain µservice
- Residual: L (inherited)
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.28; GDPR Art. 5(2); KR 전자문서법 Art. 5

### Repudiation (R)

**T-R-01 — Sender denies authorship of outbound mail**
- Asset: Outbound mail audit trail
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - SMTP submission session bound to SASL-authenticated user identity (OIDC subject).
  - DKIM signature ties message to tenant domain.
  - Audit-chain emission on `MessageSent` includes principal_id + SPIFFE + workflow context.
- Owner: axis-mail + ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.8.15; GDPR Art. 5(2)

**T-R-02 — Compliance officer denies engaging a hold**
- Asset: Legal hold audit-chain
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Hold engage requires OIDC + MFA + audit-chain Ed25519 signature by principal.
  - Four-eyes co-signer recorded.
  - Audit-chain seal carries principal SPIFFE + wall-clock + logical-clock.
- Owner: ops-legal + axis-mail
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 30

**T-R-03 — Mail-to-Workflow handoff disputed (user claims no consent)**
- Asset: Mail handoff audit chain
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Handoff requires explicit user action (UI button click) OR tenant-declared policy basis (recorded in Tenant DPA).
  - Audit-chain record captures: source message id, principal id, action type ("user-explicit" / "policy-basis"), policy basis reference, workflow item id, signed at clock.
  - Tenant DPA captures the policy basis; user-explicit captures the UI session.
- Owner: axis-mail + axis-workflow + council-privacy
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.28; GDPR Arts. 5(2), 7 (consent), 13/14 (information); KR PIPA Art. 15

### Information Disclosure (I)

**T-I-01 — Cross-tenant mailbox content leak (Postgres RLS bypass / S3 cross-bucket read)**
- Asset: Mailbox content; per-tenant boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Postgres RLS enforced on every mailbox table; policy refuses cross-tenant `tenant_id` reads.
  - S3 IAM policy refuses cross-tenant bucket access; per-tenant DEK ensures even cross-bucket read returns ciphertext only.
  - LEAN check `oya-check-rls-policy-conformance` validates RLS policies on every mail-related table.
  - Annual pen-test against cross-tenant boundary.
  - Weekly threat hunt: `mail_cross_tenant_read_attempt_total` SLO (target = 0).
- Owner: ops-security + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Arts. 23, 29; HIPAA §164.312(a)(1)

**T-I-02 — Cross-pillar mailbox leak (Professional API exposes Personal mailbox)**
- Asset: Personal-pillar mailbox; dual-context boundary
- Likelihood: M (developer-error class) / Impact: H (employer reads employee personal mail = trust + legal disaster) / Risk: **H**
- Mitigations:
  - `dual-context-isolation` BC + ContextBoundaryGuard called at every API surface; refuses cross-context routing.
  - Personal blobs encrypted under user-derived DEK; org admin cannot decrypt.
  - LEAN check `oya-check-dual-context-cross-boundary` greps for any code path reading Personal mailbox in a Professional API context.
  - Annual pen-test against pillar boundary.
  - Tenant-DPA acknowledges pillar invariant; cross-pillar request from tenant is refused per ADR-0215 contract.
- Owner: axis-mail + council-privacy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.15, A.8.12; GDPR Arts. 5, 25, 32; KR PIPA Art. 28; HIPAA §164.502(b); ADR-0215

**T-I-03 — Plaintext search index exposes mailbox content**
- Asset: Search index
- Likelihood: M (anti-pattern; engineers may default to plaintext) / Impact: H / Risk: **H**
- Mitigations:
  - Encrypted-token index design: tokens are HMAC(blinding_key, term); query is HMAC(blinding_key, search_term); index never sees plaintext.
  - Per-tenant blinding key in KMS.
  - LEAN check `oya-check-search-index-no-plaintext` greps Tantivy/Elasticsearch schema for plaintext text fields on mail content.
  - Pen-test: extract from search index files; verify only HMACs.
- Owner: axis-mail + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.11, A.8.12, A.8.24; GDPR Arts. 25, 32

**T-I-04 — eDiscovery export sealed bundle exposed to unauthorised reader**
- Asset: eDiscovery export S3 prefix
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Export bundle encrypted under export-specific DEK; KMS access bound to compliance-officer scope + tenant.
  - Download URL is signed + time-bound (≤24h TTL).
  - Download access audit-emitted.
  - Bundle expires after declared retention; auto-deleted with audit record.
- Owner: ops-legal + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5, A.8.11; GDPR Art. 32; HIPAA §164.502(b)

**T-I-05 — Mailbox content disclosure via subpoena/government-request**
- Asset: Mailbox content; legal-process compliance
- Likelihood: M (regulatory inevitability) / Impact: M (regulatory) / Risk: **M**
- Mitigations:
  - Subpoena response procedure in `runbooks/ediscovery-export.md`; legal team reviews every request.
  - Notice-to-tenant clause in tenant DPA: oyatie commits to notify tenant of disclosure unless legally prohibited (gag order).
  - Disclosure scope minimised: only the specifically-named messages, never bulk dump.
  - Audit-chain emission on every subpoena-driven export.
  - Annual transparency report.
- Owner: ops-legal + council-privacy
- Residual: M (regulatory; can't be zeroed)
- Frameworks: SOC 2 CC2.3; ISO 27001 A.5.31, A.5.34; GDPR Arts. 6(1)(c), 23; CLOUD Act; KR PIPA Art. 18

**T-I-06 — Migration source-mailbox plaintext exposed during import**
- Asset: External-provider mailbox during migration
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Migration adapter encrypts source mail to tenant DEK before persisting; in-memory plaintext window minimised.
  - Source-provider OAuth scope minimised (read-only mail).
  - Migration runs in isolated worker with no audit-bypass capability.
  - Audit-chain emission on every import batch.
- Owner: axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.14, A.8.7; GDPR Art. 32

**T-I-07 — DKIM/SPF/DMARC misconfiguration exposes tenant to spoofing**
- Asset: Inbound mail spoofing defence
- Likelihood: M / Impact: M (tenant phished) / Risk: **M**
- Mitigations:
  - DKIM/SPF/DMARC DNS records published by oyatie tenant-provisioner; tenant cannot misconfigure away from publish.
  - DMARC report ingestion + analysis surfaces anomalies; `mail_dmarc_aggregate_anomaly_total` metric.
  - Annual deliverability audit per pack.
- Owner: ops-deliverability + axis-mail
- Residual: M (DNS-level configuration drift baseline)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.5.14; GDPR Art. 32; RFC 6376/7208/7489

### Denial of Service (D)

**T-D-01 — SMTP receiver overload (mass mail / spam flood)**
- Asset: Inbound SMTP receiver capacity
- Likelihood: H (commodity) / Impact: H (legitimate mail delayed) / Risk: **H**
- Mitigations:
  - Per-source-IP rate limit (10 msg/min default; configurable per tenant).
  - Per-sender-domain rate limit on inbound.
  - Receiver HPA scales on queue depth.
  - Receiver replication factor ≥ 4 across AZs.
  - Backpressure: when receiver queue depth > 30s, attacker IPs greylisted (RFC 6647 SMTP greylisting).
  - DDoS-protection at ingress (Cloudflare or provider-level).
- Owner: axis-mail + ops-sre-reliability
- Residual: M (commodity DoS; never zero)
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Mailbox quota exhaustion (tenant or user-level)**
- Asset: Per-mailbox storage budget
- Likelihood: M / Impact: M (legitimate mail dropped at quota) / Risk: **M**
- Mitigations:
  - Per-mailbox quota with soft-warning at 80% + hard-block at 100%.
  - Quota policy tunable per tenant_scope (production: 50GB default; trial: 5GB).
  - Backup quota: tenant-pool overage cap; alerts ops-sre-reliability at 90% tenant pool.
- Owner: axis-mail + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.14

**T-D-03 — Search index corruption (concurrent write conflict / hardware fault)**
- Asset: Tantivy/Elasticsearch index
- Likelihood: L / Impact: M (search degraded; rebuild required) / Risk: **M**
- Mitigations:
  - Index per-tenant; corruption isolated to one tenant.
  - Replication factor ≥ 2 on search index nodes.
  - Index-rebuild worker: rebuild from mailbox-store in O(N) per tenant.
  - LEAN check `oya-check-search-index-rebuild-restore` validates rebuild logic in unit tests.
- Owner: axis-mail
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14

**T-D-04 — IMAP storm (mobile client misbehaviour: 1000s of concurrent fetches)**
- Asset: IMAP frontend capacity
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-user concurrent-session limit (default 5 concurrent IMAP sessions).
  - Per-user request rate-limit (100 requests/s default).
  - IMAP IDLE-only-on-active-session policy; idle sessions garbage-collected after 30min.
  - Mobile-client signature detection: known-broken clients throttled.
- Owner: axis-mail + ops-sre-reliability
- Residual: M
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-05 — KMS rotation gap (DEK rotation event causes mailbox unreadability window)**
- Asset: Per-tenant DEK envelope
- Likelihood: L / Impact: H (mailbox temporarily unreadable) / Risk: **M**
- Mitigations:
  - Rolling rotation: new DEK lazy-applied to new writes; old DEK retained for decryption of old blobs until re-wrapped.
  - Re-wrap worker: background job re-wraps blobs to new DEK over rotation window.
  - Audit-chain emission on rotation start + complete; mismatch alerts.
- Owner: cloud-secrets + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.2; ISO 27001 A.5.17, A.8.24

**T-D-06 — SMTP IP pool blocklisting (cascading reputation loss)**
- Asset: Per-tenant SMTP IP pool
- Likelihood: M / Impact: H (delivery to major providers fails) / Risk: **H**
- Mitigations:
  - Per-tenant IP pool isolation: one tenant's reputation does not affect others.
  - Reputation tracker: real-time score; auto-quarantine of compromised pool member.
  - Warm pool: standby IPs warmed via gradual ramp.
  - DKIM/SPF/DMARC tight per-tenant configuration.
  - Postmaster relationship management: feedback-loop subscription with major ISPs (Gmail, Outlook, Yahoo).
- Owner: ops-deliverability + axis-mail
- Residual: M (ISP-side decisions baseline)
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

### Elevation of Privilege (E)

**T-E-01 — Compliance-officer scope escalation (mail-admin claims compliance-officer)**
- Asset: Legal-hold + eDiscovery scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Distinct OIDC scopes: `mail.admin` and `mail.compliance.officer`; assignment via OpenBao with 2-person rule.
  - Audit-chain emission on scope assignment.
  - Quarterly access-review: stale compliance-officer assignments revoked.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.3; ISO 27001 A.5.18, A.8.2; GDPR Arts. 25, 32; HIPAA §164.308(a)(4)

**T-E-02 — Worker SPIFFE identity stolen → arbitrary mailbox read**
- Asset: Worker ServiceAccount token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SPIFFE identity bound to pod identity; not exfiltratable outside cluster.
  - Token rotation 24h.
  - Network policy: only mail-µservice worker pods may reach Postgres/S3/KMS.
  - Postgres RLS still applies even with worker SA: cross-tenant reads refused.
- Owner: ops-security + axis-mail
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17, A.8.5

**T-E-03 — Cedar policy escape via crafted JMAP field**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v3+ used (no template-escape vectors known).
  - Cedar fragments fuzzed at CI time (`oya-check-cedar-fragment-coverage`).
  - Field input lengths bounded at REST API; oversized inputs rejected.
- Owner: axis-mail + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-04 — Postgres superuser access used to bypass RLS**
- Asset: Mailbox metadata
- Likelihood: L (insider-malicious) / Impact: H / Risk: **M**
- Mitigations:
  - No long-lived Postgres superuser; JIT via OpenBao with 2-person rule.
  - Postgres audit log (pgaudit) emits every superuser operation.
  - LEAN check `oya-check-postgres-no-long-lived-superuser` validates IAM.
- Owner: ops-security + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.4

**T-E-05 — JMAP/REST API privilege escalation via mass-assignment / IDOR**
- Asset: JMAP/REST endpoints
- Likelihood: M / Impact: H / Risk: **M**
- Mitigations:
  - All JMAP/REST endpoints take typed request types (no dynamic dispatch).
  - IDOR-prevention: every path param + body field validated against principal scope before query.
  - OWASP API Top 10 #1 + #5 (BOLA + BFLA) addressed in REST design.
  - Fuzz-tested with OWASP ZAP API scanner.
- Owner: axis-mail + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.26; GDPR Art. 32; OWASP API Top 10

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Mailbox sender/recipient graph | Aggregate sender/recipient patterns can re-identify org structure. | Aggregations only on `tenant:oya-aggregate` reserved tenant with DP-noise; tenant-internal aggregates require admin scope. | L-M |
| T-L-02 | Identifiability | Hashed user-id in span attributes | Span trace attributes may include user-id hash that re-identifies via auxiliary data. | OTel SDK PII redactor; per-tenant salt rotation; trace sampling 1%. | M |
| T-L-03 | Non-repudiation | User cannot deny sending mail signed under own SASL+DKIM | Strong non-repudiation is feature for compliance, but is privacy-relevant. | Recorded in tenant DPA; user awareness via tenant onboarding notice. | L (by design) |
| T-L-04 | Detectability | Mail sending patterns inferable from outbound queue size | Outbound burst timing correlates with tenant events. | Internal metric; not exposed cross-tenant. | M (inherent) |
| T-L-05 | Disclosure | Compliance-officer-driven eDiscovery exposes employee personal context | Legal-hold scope on Professional context must not capture Personal. | Pillar invariant + four-eyes for Professional-plaintext + Personal-pillar-exclusion check on hold scope. | L |
| T-L-06 | Unawareness | End user (recipient) of mail may not know it's archived for eDiscovery | Recipient is third-party w.r.t. tenant; recipient awareness is tenant's responsibility per DPA. | Recorded in tenant DPA's joint-controllership clause; tenant-of-tenant disclosure. | M |
| T-L-07 | Non-compliance | GDPR Art. 17 erasure on a recipient's mail trail across multiple tenants | Recipient requests erasure of all their mail across tenants; cross-tenant DSR is complex. | DSR cascade per `oya-dsr-cascade-runner`: queries per tenant; per-tenant SLA 30d; cross-tenant scope is best-effort and bounded by retention. | M |
| T-L-08 | Linkability | Mail-to-Workflow handoff creates link between Personal context user and Professional context workflow | Handoff must be explicit + within Professional context only. | Handoff API rejects Personal-context source mail; audit-chain on every handoff. | L |
| T-L-09 | Identifiability | DMARC aggregate report sender-domain identifies senders | DMARC reports per-domain volumes; legitimate aggregation. | DMARC is RFC-prescribed; tenant aware via tenant DPA. | L (by design) |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| DKIM/SPF/DMARC inbound verification | Preventive | axis-mail | `oya-check-dkim-spf-dmarc-conformance` lane |
| Per-tenant SASL binding on SMTP submission | Preventive | axis-mail | `oya-check-smtp-sasl-tenant-binding` lane |
| Postgres RLS per-tenant | Preventive | axis-mail | `oya-check-rls-policy-conformance` lane |
| S3 SSE-KMS with per-tenant DEK | Preventive | cloud-secrets + axis-mail | `oya-check-mail-encryption-tenant-dek` lane |
| ContextBoundaryGuard at kernel | Preventive | axis-mail | `oya-check-dual-context-cross-boundary` lane |
| Encrypted-token search index | Preventive | axis-mail | `oya-check-search-index-no-plaintext` lane |
| Four-eyes for plaintext disclosure | Preventive | council-privacy + ops-legal | Cedar policy + audit-chain assertion |
| Hold-before-purge invariant | Preventive | axis-mail | unit test in `oya-mail-legal-hold-domain` |
| Audit-chain seal on every mail event | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Per-tenant DKIM key rotation 90d | Preventive | ops-deliverability | `oya-check-dkim-key-rotation-conformance` lane |
| Open-relay refused via Postfix config | Preventive | axis-mail | `oya-check-smtp-no-open-relay` lane |
| MTA-STS + TLS-RPT publication | Preventive | ops-deliverability | DNS automation + monitoring |
| Per-source-IP rate limit on inbound SMTP | Preventive (DoS) | axis-mail | Postfix config + metrics |
| Per-mailbox quota | Preventive (DoS) | axis-mail | Postgres constraint + metrics |
| OTel SDK PII redactor in traces | Preventive | axis-mail + workload owners | Synthetic-PII detector CI lane |
| 2-person rule + JIT for sensitive operations | Preventive (insider) | ops-security | OpenBao audit log |
| DSR cascade runner | Preventive (compliance) | council-privacy | DSR queue dashboard SLO |
| Annual cross-tenant + cross-pillar pen-test | Detective | ops-security | pen-test report |
| Soft-delete + 30d recovery for mailbox restore | Detective + Recovery | axis-mail | Mailbox restore runbook |

## Residual Risk Acceptance

Residual risks above L require explicit acceptance signed by `council-architecture` + `ops-security` + `council-privacy` + `ops-legal`:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-S-01 (sender spoofing baseline) | M | DKIM/SPF/DMARC defeats opportunistic; targeted lookalike-domains baseline. | Quarterly |
| T-S-04 (brute-force baseline) | M | Commodity attack; mitigated to acceptable via rate-limits + MFA. | Quarterly |
| T-I-05 (subpoena disclosure) | M | Regulatory inevitability; mitigated via minimisation + notice + transparency report. | Annually |
| T-I-07 (DMARC misconfig drift) | M | DNS-layer drift baseline; mitigated via automated publish + monitor. | Quarterly |
| T-D-01 (SMTP DoS baseline) | M | Commodity DDoS; mitigated to acceptable via rate-limit + DDoS provider. | Quarterly |
| T-D-04 (IMAP storm) | M | Mobile-client misbehaviour baseline; mitigated via per-user limits. | Annually |
| T-D-06 (IP pool blocklisting) | M | ISP-decision-driven; mitigated via per-tenant isolation + feedback loops. | Quarterly |
| T-E-05 (REST API IDOR baseline) | L | Mitigated to L by typed requests + scope checks + fuzz tests; reviewed quarterly. | Quarterly |
| T-L-02 (trace linkability) | M | Inherent to tracing; mitigated via redactor + sampling. | Annually |
| T-L-04 (timing detectability) | M | Inherent to behavioural metric; consent at onboarding covers. | Annually |
| T-L-06 (recipient unawareness) | M | Tenant-of-tenant responsibility per DPA. | Annually |
| T-L-07 (DSR cross-tenant best-effort) | M | Bounded by retention; DSR best-effort. | Annually |

Sign-off (this document is RW until council sign-off captured):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`
- ops-legal: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks engaged: KR-ISMS-P + KR PIPA + KR 전자문서법 + KR-FSS 전자금융감독규정 (when financial-services tenant present).

- **KR PIPA Art. 23 (sensitive personal information)**: mailbox content is `PII_IDENTIFYING`; when content includes 주민등록번호 (RRN) the redactor strips at SDK level. Mailbox-stored RRN is treated as KR PIPA Art. 24 — additional layer (KMS + 2-person rule).
- **KR PIPA Art. 28 (storage limitation)**: retention default 1y for non-essential mail metadata; longer per tenant-DPA.
- **KR PIPA Art. 29 (technical safeguards)**: every T-*-NN mitigation maps to one of the 12 prescribed safeguards.
- **KR PIPA Art. 23-2 (sensitive data cross-border)**: KR-resident mail data stays in pack-kr Postgres + S3.
- **KR 전자문서법 Art. 5**: audit-chain Ed25519 seal satisfies electronic-document integrity for mail-as-document.
- **KR-ISMS-P §2.5 + §2.7**: 2-person rule + JIT mapping direct.
- **KR-FSS guidance**: when KR financial-services tenant onboards, mail retention floor 5y + KMS-in-KR + KR-resident operator access only.

### pack-us-healthcare (HIPAA-scoped)

Compliance frameworks engaged: HIPAA + state-level (CCPA / CMIA / etc.).

- **HIPAA §164.312(a)(1) (access control)**: per-tenant Postgres RLS + S3 IAM + Ed25519 audit-chain satisfy Unique User Identification + Emergency Access + Automatic Logoff + Encryption-and-Decryption.
- **HIPAA §164.312(b) (audit controls)**: audit-chain emission on every PHI-touching operation; retention ≥ 6y (cost-budget reflects).
- **HIPAA §164.502 (minimum necessary)**: mail-to-Workflow extraction redacts PII/PHI per `policy/redaction-phi.md` (pack-us-healthcare overlay).
- **HIPAA §164.308(a)(4)(ii)(B) (access authorization)**: distinct compliance-officer scope; auditor JIT scopes per T-S-05.
- **HIPAA §164.504(e) (BAA)**: BAA required pre-onboarding pack-us-healthcare; BAA template at `legal/baa-template.md`.
- **HITECH §13402 (breach notification)**: 60d + media notification > 1000 affected.

### pack-eu

Compliance frameworks engaged: GDPR + eIDAS + NIS2 + ePrivacy Directive.

- **GDPR Art. 25 (privacy-by-design)**: dual-context isolation + per-tenant DEK + encrypted-token search.
- **GDPR Art. 35 (DPIA)**: this threat model + DPIA at `dpia.md` together fulfil DPIA.
- **GDPR Art. 28 (processor)**: oyatie acts as processor for tenant mail; DPA template.
- **GDPR Art. 32 (security of processing)**: every T-*-NN mitigation contributes.
- **GDPR Arts. 44-50 (transfers)**: pack-eu Postgres + S3 EU-resident; cross-region transfer with SCC only.
- **NIS2**: when oyatie crosses Annex I/II thresholds, 24h/72h/1mo timelines apply.
- **eIDAS 910/2014**: Ed25519 audit-chain seals are AdES.
- **ePrivacy Directive Art. 5**: e-mail privacy; processor stance.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/mail-overlay.md` carry the pack-specific legal-citation depth. Each overlay follows this document's structure with pack-specific overlays for PII law articles + cybersecurity-framework controls.

## Compliance Cross-Mapping (Globally Enforced)

See `compliance.md` (Slice B) for the full control-to-framework mapping.

## Re-review Triggers

This threat model re-reviews on:

- Any change to the trust boundary diagram above.
- Any Layer-A version upgrade (Postfix / Dovecot / Rspamd / Tantivy) with security fixes.
- Any new pack activation (e.g., first pack-us-healthcare HIPAA tenant onboarding triggers HIPAA-specific deep-dive).
- Annual scheduled review (Q2).
- Post-incident review (any Sev-1 / Sev-2).
- Pen-test or audit finding.
- Any change to the dual-context invariant (Personal vs Professional).

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519); inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0135: Connect dissolution + dual-context invariant.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward policy.
- ADR-0140: Cedar policy enforcement.
- Bominal ADR-0208 / 0210 / 0215: inherited.
- `microservices/mail/PRD.md`.
- `microservices/mail/dpia.md`.
- `microservices/mail/policy/{dual-context-isolation, data-residency, tenant-scope, ci-scope, auditor-scope, public-read}.{md,cedar}`.
- `/specs/microservices/mail.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023).
- NIST SP 800-154.
- RFC 5321 (SMTP), 6376 (DKIM), 7208 (SPF), 7489 (DMARC), 8314 (mail TLS), 8460 (TLS-RPT), 8461 (MTA-STS).
- ICO DPIA template; CNIL DPIA methodology.
- KR PIPA + ISMS-P; HIPAA OCR guidance; KR 전자문서법 + KR-FSS regulations.
