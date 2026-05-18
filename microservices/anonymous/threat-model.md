---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: anonymous
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-anonymous + ops-security
deciders: council-architecture, ops-security, axis-anonymous, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy; especially relevant) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0003, ADR-ANON-0006, ADR-ANON-0007]
related_specs: [/specs/microservices/anonymous.json]
review_cadence: quarterly + on every architecture or substrate change + every legal-process disclosure category change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.11, A.8.20, A.8.23, A.8.24, A.8.25, A.8.26"
  - "ISO 27018:2019 PII-in-cloud — full applicability"
  - "GDPR Arts. 5, 6, 11 (data not requiring identification — pseudonymisation), 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU AI Act 2024/1689 Art. 50 transparency"
  - "EU DSA Arts. 14, 16, 17, 20, 24, 27, 28"
  - "ePrivacy Directive 2002/58 Art. 5(3) — cookieless tracking"
  - "US 18 USC §2258A NCMEC CyberTipline reporting"
  - "US ECPA SCA §2701-2712 + §2705 gag-order doctrine"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/21/22-2/23/24/24-2 (alternative pseudonymous processing)/28/29/29-2", "통신비밀보호법 (Communications Secrecy Protection Act) Arts. 5/9/9-2", "정보통신망법 (Network Act) Arts. 22/28/29", "KR-ISMS-P §2.1-2.12", "청소년 보호법 (Youth Protection Act)"]
  pack-us: ["First Amendment (anonymous-speech doctrine: Talley v. California 1960, McIntyre v. Ohio 1995)", "Section 230 Communications Decency Act", "COPPA 15 USC §6501 + 16 CFR §312 (<13 ban)", "California Anti-Doxxing — Cal. Civ. Code §1708.7", "NY Civil Rights Law §50/§51 (publicity rights)", "IL 720 ILCS 5/26.5 (cyberstalking)", "ECPA SCA + §2705 gag-order"]
  pack-eu: ["GDPR Art. 11 + Recital 26 (pseudonymisation explicit basis)", "EU DSA Arts. 14/16/17/20/24/27/28", "EU AI Act Art. 50", "ePrivacy 2002/58 Art. 5(3)", "NIS2 2022/2555"]
  pack-uk: ["UK Online Safety Act 2023 (Ofcom oversight; illegal-content priority)", "UK Investigatory Powers Act 2016 §57 (legal-process disclosure)", "UK Data Protection Act 2018"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.502 minimum-necessary; pack-us-healthcare not heavily applicable (anonymous tier rarely transmits PHI by design); minor coverage"]
  pack-jp: ["通信の秘密 (Constitutional Art. 21 secrecy of communications)", "APPI Arts. 17/18/20/21/23/24/26-2/27", "青少年保護条例 (per prefecture youth protection ordinances)"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12 (where applicable to tenants in financial sector)"]
  pack-au: ["Privacy Act 1988 APP 1-13", "AU Online Safety Act 2021 + BOSE", "TIA Act (intercept legal-process)"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: anonymous µservice

## Purpose

Identify, classify, and mitigate threats to the anonymous µservice's confidentiality, integrity, availability, and *especially* unlinkability + anonymity properties. The anonymous µservice's primary threat is **de-anonymization** — an attacker (or insider) correlating `post_id → user_id` outside the legal-process workflow. This document is reviewed by SOC 2 examiners, ISO 27001 + 27018 auditors, GDPR DPAs (especially around Art. 11 + Recital 26 pseudonymisation conformance), KR PIPC (especially around PIPA Art. 24-2 alternative pseudonymous processing), and Korea Communications Commission (통신비밀보호법) at first-tenant onboarding per pack.

LINDDUN methodology is given equal weight to STRIDE because the privacy-properties (Linkability, Identifiability, Non-repudiation, Detectability, Disclosure-of-information, Unawareness, Non-compliance) are the *primary* threat surface for this µservice, more so than confidentiality alone.

## Scope

### In-scope

All components introduced by parallel ADR-0135 (Connect dissolution) + ADR-0132 (suite dissolution into anonymous surface) for the anonymous µservice. Deployed in the dedicated anonymous Kubernetes cluster.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (posts + votes + attestation-bindings; blinding-column-isolated schema) | `oya-anonymous-pseudonymous-identity-*` (10 crates) |
| Redis 7.2 (feed cache + vote counter) | `oya-anonymous-affinity-attestation-*` (10 crates) |
| Meilisearch 0.10 (hashtag search; never per-author) | `oya-anonymous-post-thread-*` (9 crates) |
| `ring 0.17` (RSA-PSS-blind) OR `rust-bls` (Schnorr-blind / BBS+) per ADR-ANON-0001 | `oya-anonymous-feed-timeline-*` (9 crates) |
| `oya-bbs-plus` Layer-A wrapper (W3C VC 2.0; pinned per ADR-ANON-0002) | `oya-anonymous-upvote-downvote-*` (9 crates) |
| `oya-mls-rs` Layer-A wrapper (RFC 9420; pinned per ADR-MSGR-0002) | `oya-anonymous-content-moderation-*` (9 crates) |
| OPSWAT MetaDefender 5.x / ClamAV 1.x (T2 attachments only) | `oya-anonymous-legal-process-disclosure-*` (8 crates) |
| Cedar v4.2 policy evaluator | `oya-anonymous-retention-policy-*` (8 crates) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited for `Affinity` reads.
- Network-layer anonymity (Tor / I2P / mixnets) — application-layer anonymity (LINDDUN-L + LINDDUN-I) is the in-scope target; users wanting network anonymity layer their own transport.

## Trust Boundaries

```text
┌─ Internet ──────────────────────────────────────────────────────────────────┐
│                                                                             │
│   End-users (web/desktop/mobile; optionally over Tor)                       │
│         │                                                                   │
│         │ (TLS 1.3, OIDC for affinity-IdP only — NOT for posting auth)      │
│         ▼                                                                   │
│  ┌─ Public ingress (Envoy/Cloudflare) ──────────────────────────────────┐   │
│  │  TLS + WAF + DDoS                                                    │   │
│  │  NB: NO third-party CDN with tracking (Cloudflare in pass-through    │   │
│  │      mode only; no Cloudflare Insights, no Cloudflare Analytics)     │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
└──────────────────────────────│──────────────────────────────────────────────┘
                               ▼
┌─ Dedicated anonymous cluster ───────────────────────────────────────────────┐
│                                                                             │
│  TB1: External → Cluster ingress                                            │
│                                                                             │
│  ┌─ Affinity-attestation BC (BBS+ verify) ────────────────────┐             │
│  │  Verifies attestation; does NOT learn issuer identifier    │             │
│  │  Issues blinded-credential (per session)                   │             │
│  └────────────────────────────────────────────────────────────┘             │
│                                                                             │
│  TB2: Affinity-attestation → Pseudonymous-identity (blind-signature)        │
│                                                                             │
│  ┌─ Pseudonymous-identity BC ─────────────────────────────────┐             │
│  │  Issues blind-signed handle; never sees post body          │             │
│  │  Stores blinded commitment ONLY                            │             │
│  └────────────────────────────────────────────────────────────┘             │
│                                                                             │
│  TB3: Pseudonymous-identity → Post-thread (data-class isolation)            │
│                                                                             │
│  ┌─ Post-thread BC ────────────────────────────────────────────┐            │
│  │  Stores post + comment + blinded-author-commitment          │            │
│  │  NEVER sees user_id; NEVER joins to identity tables         │            │
│  └─────────────────────────────────────────────────────────────┘            │
│                                                                             │
│  TB4: BC services → backing stores                                          │
│                                                                             │
│  ┌─ Postgres (per-tenant RLS + blinding-column-isolated schema) ────────┐   │
│  │  separate tables: anonymous.blinded_credential / anonymous.post      │   │
│  │  database-level GRANT prevents JOIN across the two                   │   │
│  │  except via legal_process_disclosure_view (Cedar-gated)              │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│  ┌─ Redis (vote counter + feed cache; opaque keys only) ────────────────┐   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│  ┌─ Meilisearch (hashtag-corpus only; never per-author) ────────────────┐   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  TB5: Legal-process disclosure boundary (audit-chain + Cedar-gated)         │
│                                                                             │
│  ┌─ Legal-process-disclosure BC ──────────────────────────────┐             │
│  │  Court-order → dual-approve → Cedar gate → legal_process_  │             │
│  │  disclosure_view JOIN executes → audit-chain seal          │             │
│  └────────────────────────────────────────────────────────────┘             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Assets

| Asset | Class | Confidentiality | Integrity | Availability | Unlinkability |
|---|---|---|---|---|---|
| User identity (real-name, email, phone) — only known to affinity-IdP | PII_IDENTIFYING | Critical | Critical | Medium | Critical |
| Affinity attestation (binds user to employer/edu/geo) | PII_IDENTIFYING | High | Critical | Medium | Critical |
| Blinded-credential commitment | INTERNAL_ONLY (cryptographic) | Medium | Critical | High | n/a |
| Post body | BEHAVIORAL_USER_CONTENT | Medium (semi-public within affinity) | Critical | High | Critical (must not link to user) |
| Vote action | BEHAVIORAL_TENANT_PRODUCT | Low | High | High | Critical |
| Comment body | BEHAVIORAL_USER_CONTENT | Medium | High | High | Critical |
| Anonymous-DM ciphertext | BEHAVIORAL_TENANT_PRODUCT (ciphertext) | n/a (server holds ciphertext) | High | High | Critical |
| Hashtag corpus | BEHAVIORAL_TENANT_PRODUCT | Low | Medium | Medium | n/a |
| Audit-chain records | AUDIT | High | Critical | High | n/a (audit IS the linkability under court order) |
| Legal-process disclosure package | AUDIT + PII_IDENTIFYING | Critical | Critical | Medium | n/a (purposeful linkage under court) |
| Blind-signature private key (issuer) | SECRET | Critical | Critical | Critical | n/a |
| BBS+ issuer signing key | SECRET | Critical | Critical | Critical | n/a |

## Threat Catalog

Threats categorised by STRIDE + LINDDUN. T-x = STRIDE; T-L-x = LINDDUN.

### T-S Spoofing

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-S-01 | Attacker forges affinity-attestation (claims "I am a Bominal employee" without being one) | Medium | High (community pollution) | BBS+ cryptographic verification per ADR-ANON-0002; verification fails on forged proof | Mitigated |
| T-S-02 | Attacker forges blind-signature ("creates" a blinded credential without going through issuer) | Low (requires breaking RSA-PSS-blind / Schnorr-blind) | Critical (I1 violated) | NIST SP 800-186 + IRTF CFRG draft conformance; library audited; integration tests with known-answer vectors | Mitigated |
| T-S-03 | Replay attack on blind-signature issuance | Medium | High | Per-issuance nonce (issuer-side) + per-session nonce (client-side); replay rejection in `oya-anonymous-pseudonymous-identity-domain` | Mitigated |
| T-S-04 | Affinity-IdP impersonation (attacker pretends to be the corporate IdP issuing attestation) | Low | High | Pre-registered issuer DID + signature verification chain | Mitigated |

### T-T Tampering

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-T-01 | Tamper with post body in transit | Low | Medium | TLS 1.3 + content_hash (sha256) verified at server | Mitigated |
| T-T-02 | Tamper with vote count via SQL injection / Redis injection | Low | High | Postgres parameterized queries; Redis serialization layer; LEAN lane `oya-check-sql-injection-refused` | Mitigated |
| T-T-03 | Tamper with audit-chain record (rewrite history) | Low | Critical | Merkle + Ed25519 seal per Bominal ADR-0028; chain-of-custody hash linked | Mitigated |
| T-T-04 | Tamper with blinded-credential commitment in Postgres | Low | Critical (I1 violated) | Row-level integrity hash + Ed25519 seal | Mitigated |
| T-T-05 | Tenant-admin tampering with retention worker config to extend retention beyond pack policy | Medium | High (regulatory violation) | Pack-policy enforced as hard ceiling in `policy/data-residency.md`; tenant override forbidden at code level | Mitigated |

### T-R Repudiation

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-R-01 | Tenant disputes legal-process disclosure was authorised | Low | Critical | Dual-control approval persisted with two distinct principal IDs + audit-chain seal + court-order attachment hash | Mitigated |
| T-R-02 | End-user disputes a post was theirs after deletion (no audit-chain proof) | Medium | Low | Tombstone in audit-chain seals the blinded commitment ID at deletion time | Mitigated |
| T-R-03 | Moderator disputes verdict was theirs | Low | Medium | Moderator JWT principal_id + reason captured + audit-chain seal | Mitigated |

### T-I Information disclosure

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-I-01 | **PRIMARY THREAT — de-anonymization via internal DB JOIN** Insider with Postgres `psql` access joins `anonymous.post` to `anonymous.blinded_credential` to internal identity table | Medium (insider risk) | Critical (I1 violated) | Database-level GRANT separation; LEAN lane `oya-check-blinding-column-isolation`; legal_process_disclosure_view is the ONLY surface that joins, and it is Cedar-gated + dual-control | Mitigated |
| T-I-02 | De-anonymization via timing-correlation (post-time matches login-time of one user) | Medium | High | Per-session blinded credential with TTL; client-side timing jitter (k-anonymous-within-tag advice in client SDK); server-side metric coarsening | Mitigated (best-effort) |
| T-I-03 | De-anonymization via writing-style (stylometry) on aggregated posts | Medium | Medium | Per-channel handle rotation (FR-02); user-facing UX advisory in client SDK | Mitigated (warning shown) |
| T-I-04 | Push-notification payload leaks real-name (e.g., "{name} mentioned you") | Medium | High | Opaque-handle-only payload (FR-10); LEAN lane `oya-check-notification-payload-opaque-handle-only` | Mitigated |
| T-I-05 | Search-result endpoint leaks per-author corpus (allows enumeration) | Low | High | Search index never has per-author field; LEAN lane `oya-check-search-index-no-author-column` | Mitigated |
| T-I-06 | Server-side log inadvertently logs `user_id` from affinity-IdP claim | Medium | Critical | Structured logging schema enforces no `user_id` field on `anonymous.*` logs; LEAN lane `oya-check-log-schema-no-user-id` | Mitigated |
| T-I-07 | Server-side metric inadvertently labels by `user_id` | Medium | High | Prometheus label set audited; per-author label REFUSED at metric definition | Mitigated |
| T-I-08 | Backup / snapshot leaks `blinded_credential` ↔ `post` join table | Low | Critical | Backups encrypted at rest (tenant-DEK); restore-test confirms blinding-column separation preserved | Mitigated |
| T-I-09 | Side-channel: HTTP response timing differs based on whether user is real owner of post | Low | High | Constant-time response path for `read_own_post` vs `read_post`; not branchy by ownership | Mitigated |
| T-I-10 | Affinity attestation issuer learns post content (issuer is corporate IdP; could collude) | Low | Critical (I2 violated) | Blind-signature ensures issuer signs over a blinded message; issuer never sees post body | Mitigated |
| T-I-11 | Anonymous-DM server holds plaintext via misconfiguration | Low | Critical (I6 violated) | MLS (RFC 9420) end-to-end; LEAN lane `oya-check-e2e-no-plaintext-server-state`; server only holds ciphertext + Welcome / Commit metadata | Mitigated |
| T-I-12 | Vote-correlation: attacker correlates pattern of votes to deduce voter identity | Medium | Medium | Vote tally aggregated; individual vote-records carry blinded-commitment only; deletion + rotation reduces longevity | Mitigated (best-effort) |
| T-I-13 | Hashtag-search reveals minority-affinity-user (rare hashtag + small affinity → identifies) | Medium | Medium | k-anonymity floor on affinity (k=50 geo / k=20 employer); hashtag corpus per-affinity not per-author | Mitigated |

### T-L LINDDUN (privacy-specific)

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-L-01 | Linkability across channels (handle rotation defeated by metadata) | Medium | High | FR-02 per-channel rotation; channel-binding nonce; client SDK rotates session keys | Mitigated |
| T-L-02 | Identifiability via posting-pattern + IP (post times match user's known schedule) | Medium | Medium | Per-session credential TTL; advisory to use Tor for highest assurance; server-side IP not stored beyond rate-limit window (15 min) | Mitigated |
| T-L-03 | Non-repudiation by accident — audit-chain implies user wrote post | Low | High | Audit-chain seals blinded commitment, NOT user_id; only legal-process view bridges | Mitigated |
| T-L-04 | Detectability — adversary detects "user X is on platform" | Medium | Low | Account presence is intentionally undiscoverable (no search by user_id) | Mitigated |
| T-L-05 | Disclosure-of-information beyond purpose (analytics SDK leaks behavioural data to third party) | Critical (without I4) | Critical | I4: no third-party tracker; LEAN lane | Mitigated |
| T-L-06 | Unawareness — user does not know post is being moderated | Medium | Medium | EU AI Act Art. 50 transparency label "AI-assessed" on every verdict | Mitigated |
| T-L-07 | Non-compliance — pack-specific regulation (KR PIPA Art. 24-2; GDPR Recital 26) violated | Medium | Critical | Per-pack overlay + compliance.md per-pack section | Mitigated |
| T-L-08 | Affinity-de-anonymization via small affinity (employer with 1 employee) | Medium | High | k-floor enforcement (ADR-ANON-0007); employer with <10 employees routes through anonymization fallback | Mitigated |
| T-L-09 | Affinity-revocation attack (issuer revokes attestation → past posts become attributable) | Low | High | Revocation does not retroactively link; past posts remain bound to blinded commitment that has been issued; revocation only prevents new posts | Mitigated |
| T-L-10 | Vote-pattern de-anonymization (Sweeney 2002 k-anonymity attack on voting patterns) | Medium | Medium | k-anonymity floor + vote-time bucketing + advisory to vote at varied times | Mitigated |

### T-D Denial of service

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-D-01 | Mass-post spam (one bad actor floods affinity-community) | High | Medium | Per-blinded-credential rate limit (10 posts / minute); affinity-attestation rate limit at IdP layer | Mitigated |
| T-D-02 | Vote-fraud (mass downvote brigade) | High | Medium | Per-blinded-credential one-vote-per-post bound (cryptographic proof); vote-velocity classifier T2 | Mitigated |
| T-D-03 | BBS+ verify CPU exhaustion (computationally expensive) | Medium | High | Per-IP rate limit 100 verify/min; CPU budget per tenant | Mitigated |
| T-D-04 | Postgres tablespace fill (mass post creation) | Medium | High | Per-tenant storage quota; auto-tier-purge on near-quota | Mitigated |
| T-D-05 | Redis memory exhaustion (vote counter blowup) | Medium | Medium | Per-tenant Redis memory quota; LRU eviction | Mitigated |

### T-E Elevation of privilege

| ID | Threat | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| T-E-01 | Tenant-operator escalates to read Personal-tier content via API misuse | Low | Critical (I1 in spirit) | Cedar `policy/tenant-scope.cedar` forbids tenant-operator reads except for legal-process-Cedar | Mitigated |
| T-E-02 | Moderator escalates to read non-flagged content | Low | High | Cedar PERMIT 5 (moderator only on `resource.flagged == true`) | Mitigated |
| T-E-03 | Single legal-process approver attempts unilateral disclosure (no dual-control) | Low | Critical | Cedar `policy/legal-process-disclosure.cedar` requires distinct `paired_approver_id`; both must hold `legal_process_approver` entitlement | Mitigated |
| T-E-04 | Database direct-access (psql with admin credential) bypasses Cedar | Medium (insider) | Critical | Database GRANT separation (post writer cannot read identity tables, identity reader cannot read post body); only `legal_process_disclosure_view` joins; that view is owned by `legal_process_approver` role only | Mitigated |
| T-E-05 | OPSWAT scan sandbox escape on T2-attachment | Low | High | gVisor `runtimeClassName: gvisor` per Helm `deployment.yaml`; readOnlyRootFilesystem + capabilities drop ALL | Mitigated |

## Defence-in-depth summary

1. **Crypto layer (I1):** blind-signature; commitment-only DB column; library audited.
2. **DB GRANT layer:** post writer ≠ identity reader; only `legal_process_disclosure_view` bridges.
3. **Cedar policy layer:** default-deny; legal-process Cedar gate; dual-control.
4. **LEAN-lane CI layer:** 13 LEAN lanes guard structural invariants from PR-1.
5. **Audit-chain layer:** every state-changing op sealed; chain-of-custody on disclosure.
6. **Build-time layer:** no third-party tracker dependencies accepted.
7. **Transport layer:** TLS 1.3 mTLS; client-side advice to use Tor for highest assurance.
8. **Observability layer:** structured logging schema refuses `user_id` field; metric label whitelist enforced.

## Open Questions

| # | Question | Owner | Resolution |
|---|---|---|---|
| 1 | What is the agreed threshold for k-anonymity floor on a "small employer"? Current ADR-ANON-0007 sets k=10 for small employers — is that aggressive enough? | council-privacy + axis-anonymous | ADR-ANON-0007 successor-IP after first 90 days of production data |
| 2 | Tor-as-default-transport for client SDK: opt-in or default? | axis-anonymous + gtm | Open; scheduled-for-distinct-tracked-work to ADR-ANON successor-IP |
| 3 | NCMEC CyberTipline reporting: who at oyatie holds the chain-of-custody signing key for the per-report Ed25519 seal? | ops-security + legal | runbook resolution pending |
