---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-messenger + ops-security
deciders: council-architecture, ops-security, axis-messenger, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0140]
related_specs: [/specs/microservices/messenger.json]
review_cadence: quarterly + on every architecture or substrate change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316", "HITECH Act breach-notification"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "ePrivacy Directive 2002/58 (communications confidentiality)", "NIS2 2022/2555 (when thresholds engaged)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "TIA Act + Surveillance Devices Act (intercept)"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: messenger µservice

## Purpose

Identify, classify, and mitigate threats to messenger's confidentiality, integrity, availability, and privacy posture. The messenger µservice is the canonical surface for real-time team coordination and direct messaging across personal + professional contexts; a compromise leaks chat history, identity graph, and (for pack-us-healthcare) PHI. This document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, and HIPAA OCR at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by parallel ADR-0135 (Connect dual-context inherited) and ADR-0132 (suite dissolution into messenger surface) for the messenger µservice. Deployed in the dedicated messenger Kubernetes cluster.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres (message + channel + thread store) | `oya-messenger-channel-store-*` (9 crates) |
| Redis (presence + read-receipt) | `oya-messenger-message-stream-*` (11 crates) |
| S3-compatible (attachment blobs) | `oya-messenger-thread-tree-*` (7 crates) |
| Tantivy / Elasticsearch (message search) | `oya-messenger-read-receipt-tracker-*` (7 crates) |
| WebSocket gateway (Envoy + Cloudflare termination) | `oya-messenger-file-attachment-*` (8 crates) |
| OPSWAT MetaDefender / ClamAV (attachment scan) | `oya-messenger-mention-router-*` (7 crates) |
| Cedar policy evaluator | `oya-messenger-presence-*` (8 crates) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited for mention-resolution path.
- Threats to GitHub Actions — owned by `governance`.

## Trust Boundaries

```text
┌─ Internet ──────────────────────────────────────────────────────────────┐
│                                                                         │
│   End-users (web/desktop/mobile)         Workflow Studio shell          │
│         │                                       │                       │
│         │ (TLS, WSS, OIDC, OAuth)               │ (mTLS internal)       │
│         ▼                                       ▼                       │
│  ┌─ Public ingress (Envoy/Cloudflare) ────────────────────────────┐     │
│  │  TLS + WAF + DDoS + WebSocket upgrade                          │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                              │                                          │
└──────────────────────────────│──────────────────────────────────────────┘
                               ▼
┌─ Dedicated messenger cluster ───────────────────────────────────────────┐
│                                                                         │
│  TB1: External → Cluster ingress                                        │
│                                                                         │
│  ┌─ WebSocket gateway (presence + message fanout) ─────────┐            │
│  │  per-tenant connection registry                          │            │
│  │  X-Scope-OrgID enforcement                               │            │
│  └──────────────────────────────────────────────────────────┘            │
│                                                                         │
│  TB2: WebSocket gateway → BC services (mTLS + SPIFFE)                   │
│                                                                         │
│  ┌─ channel-store-rest ──┐ ┌─ message-stream-rest ─┐                    │
│  │ Cedar evaluation      │ │ Cedar evaluation      │                    │
│  └───────────────────────┘ └───────────────────────┘                    │
│                                                                         │
│  TB3: BC services → backing stores                                      │
│                                                                         │
│  ┌─ Postgres (per-tenant RLS) ─┐  ┌─ Redis cluster ─┐                   │
│  │ messages, channels, threads │  │ presence, recv  │                   │
│  └─────────────────────────────┘  └─────────────────┘                   │
│  ┌─ S3 (attachment blobs; KMS) ┐  ┌─ Tantivy/ES ────┐                   │
│  │ per-tenant prefix isolation │  │ per-tenant idx  │                   │
│  └─────────────────────────────┘  └─────────────────┘                   │
│                                                                         │
│  TB4: Personal/Professional context isolation (data-model invariant)    │
│                                                                         │
│  TB5: BC services → audit-chain µservice (Ed25519-signed)               │
│                                                                         │
│  TB6: BC services → ontology µservice (Workflow event)                  │
│                                                                         │
│  TB7: Attachment scan path (OPSWAT/ClamAV; quarantine bucket)           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

Seven trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, WebSocket upgrade).
2. **Gateway → BC services** (mTLS + SPIFFE identity).
3. **BC services → backing stores** (RLS + per-tenant prefix isolation).
4. **Personal/Professional context isolation** (data-model invariant per parallel ADR-0135).
5. **BC services → audit-chain** (Ed25519 seal).
6. **BC services → ontology** (Workflow event bus).
7. **Attachment scan path** (untrusted blob → scanner → quarantine vs production).

## Assets & Data Classification

| Asset | Class | Retention | Authoritative store |
|---|---|---|---|
| Channel messages (professional) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` + occasionally `PHI` | per-pack (30d hot, retention floor per regulator) | Postgres (tenant-DEK encrypted) |
| Channel messages (personal) | `PERSONAL` + E2E ciphertext | per-user policy | Postgres (E2E ciphertext only; no plaintext at rest server-side) |
| Thread replies | inherits parent | inherits | Postgres |
| Read receipts | `BEHAVIORAL_TENANT_PRODUCT` | 90d hot | Redis (persisted to Postgres asynchronously) |
| Presence state | `BEHAVIORAL_TENANT_PRODUCT` | live + 15min trail | Redis |
| File attachments | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` / `PHI` | per-pack | S3 (KMS-encrypted; per-tenant prefix) |
| Attachment metadata (digest, preview, malware-scan verdict) | `INTERNAL_ONLY` + `AUDIT` | append-only | Postgres |
| Channel ACL + member list | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` | append-only history | Postgres |
| Mentions (resolved) | `PII_IDENTIFYING` | parent-message-bound | Postgres |
| Search index | derived from messages | rebuilt from source | Tantivy / ES (per-tenant index) |
| Audit-chain seals (channel-create, member-grant, disclosure) | `AUDIT` | append-only; immutable | audit-chain µservice |
| Per-tenant DEK | `SECRET` | OpenBao 30d rotation; envelope KMS | OpenBao |
| WebSocket gateway session tokens | `SECRET` | ≤ 24h | OpenBao-issued short-lived JWT |
| Personal-DM E2E key material | `SECRET` (client-held; never server-readable) | client lifetime | client device (oyatie sees only public keys) |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| End-user (human) | Untrusted external | OIDC + MFA + WSS bearer | Read/write own DMs + channels they belong to |
| Tenant channel-admin | Semi-trusted internal-to-tenant | OIDC + MFA | Manage channel ACL; can NOT read message bodies (PII) without four-eyes |
| Tenant compliance-officer | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Issue eDiscovery hold; trigger disclosure (requires four-eyes peer) |
| Tenant security-admin | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Configure pack policy; four-eyes pairing peer |
| oyatie ops-security (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin access; no plaintext PII without breakglass + two-person rule |
| Workflow Studio (machine) | Semi-trusted internal | mTLS + SPIFFE | Consume Workflow events; post action-cards via mention-router |
| mail µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Emit `MailActionCardEmitted` event for mention-router |
| ontology µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Serve Person/Team/Channel lookups |
| audit-chain µservice (machine) | Trusted internal | mTLS + SPIFFE | Receive seals from every BC |
| External auditor | Read-only external | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain seals; read policy artifacts |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated supply-chain awareness |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure channel ACL or retention |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality; mitigated by audit-chain + four-eyes |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — User-B impersonates User-A via WebSocket session hijack**
- Asset: WebSocket session
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-connection short-lived JWT bound to device + IP; WSS only; rotation 24h; OIDC re-auth on token expiry; anomaly detection on geo-shift mid-session.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5; GDPR Art. 32(1)(a)(b)

**T-S-02 — Channel-admin impersonates compliance-officer to trigger disclosure single-handed**
- Asset: Four-eyes disclosure path
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires two distinct SPIFFE identities with distinct entitlements + audit-chain seal of both consents; same principal cannot satisfy both halves; replay-resistant nonce.
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.34; GDPR Art. 32

**T-S-03 — Forged @mention from external sender**
- Asset: mention-router
- Likelihood M / Impact M / Risk **M**
- Mitigations: mention-router only resolves mentions in messages signed by an authenticated session token; external-sender events (e.g., from email) carry verified-author signatures; spoofed mentions rejected at ingress.
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15

**T-S-04 — Attacker forges presence as user "online" to deceive recipients**
- Asset: Presence
- Likelihood L / Impact M / Risk **L**
- Mitigations: presence transitions write through gateway → Redis with session-token-bound principal; no client-supplied presence except via authenticated connection.

### Tampering (T)

**T-T-01 — Message tampering at rest (Postgres row mutation)**
- Asset: Message store
- Likelihood L / Impact H / Risk **M**
- Mitigations: every message row carries `content_hash` (sha256(plaintext)) emitted to audit-chain on write; periodic batch verifier compares hashes; mismatch quarantines + alerts.
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)

**T-T-02 — Attachment blob tampering in S3**
- Asset: Attachment blobs
- Likelihood L / Impact H / Risk **M**
- Mitigations: SSE-KMS + S3 Object Lock (WORM); content-digest verified at fetch; tamper triggers quarantine; bucket access via service-account IAM only.

**T-T-03 — Search index poisoning**
- Asset: Tantivy / ES index
- Likelihood L / Impact M / Risk **L**
- Mitigations: only message-stream-worker writes to index; SPIFFE-validated; rebuild from source possible (deterministic).

**T-T-04 — Channel-ACL tampering by privileged insider**
- Asset: Channel ACL rows
- Likelihood L / Impact H / Risk **M**
- Mitigations: every ACL change emits `ChannelMemberGrantedRevoked` event with Ed25519 seal; periodic ACL-drift detection compares Postgres state vs audit-chain authoritative replay; mismatch quarantines channel.

**T-T-05 — Read-receipt tampering**
- Asset: Read-receipt store
- Likelihood L / Impact L / Risk **L**
- Mitigations: read-receipts are best-effort by design; tampering has low impact; rebuild from message-read events possible.

### Repudiation (R)

**T-R-01 — User denies authoring a message**
- Asset: Message authorship
- Likelihood M / Impact M / Risk **M**
- Mitigations: every message carries author SPIFFE identity + session-token nonce + audit-chain seal; client-side device-key signing where available.
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2)

**T-R-02 — Admin denies authorising disclosure**
- Asset: Four-eyes disclosure record
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires both consents in audit-chain with distinct principal IDs + reason code; non-repudiable.

**T-R-03 — Compliance-officer denies eDiscovery hold issuance**
- Asset: EDiscoveryHoldOpened event
- Likelihood L / Impact M / Risk **L**
- Mitigations: hold issuance emits Ed25519-signed audit-chain record with timestamp + scope.

### Information Disclosure (I)

**T-I-01 — Cross-tenant message leak via Postgres RLS misconfiguration**
- Asset: Message store
- Likelihood M / Impact H / Risk **H**
- Mitigations: Postgres Row-Level Security with `tenant_id = current_setting('app.tenant_id')`; gateway sets the GUC per connection; LEAN check `oya-check-postgres-rls-coverage` asserts RLS enabled on every messenger table; pen-test annually.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3; GDPR Art. 32; KR PIPA Art. 29; HIPAA §164.312(a)(1)

**T-I-02 — PHI leak in attachment preview (pack-us-healthcare)**
- Asset: Attachment preview thumbnails
- Likelihood M / Impact H / Risk **H**
- Mitigations: pack-us-healthcare disables auto-preview by default; admin-enabled preview goes through PHI-redactor that scans OCR output; previews stored with same access control as parent.
- Frameworks: HIPAA §164.502; §164.514(b); GDPR Art. 9

**T-I-03 — Personal-DM cipherbody decryption attempt by tenant admin**
- Asset: Personal-DM E2E ciphertext
- Likelihood M / Impact H / Risk **H**
- Mitigations: server stores ciphertext only; key material never sent to server; admin reads return ciphertext (unreadable); attempts emit `oya_personal_dm_admin_decrypt_attempt_total` (target=0).
- Frameworks: GDPR Art. 25 (privacy-by-design); KR PIPA Art. 28

**T-I-04 — Channel name + member-list pivoting (metadata leak)**
- Asset: Channel metadata
- Likelihood M / Impact M / Risk **M**
- Mitigations: channel metadata access requires membership OR Cedar `Action::"read_channel_metadata"`; cross-channel enumeration forbidden by query plan; per-tenant cardinality limits.

**T-I-05 — Search-result leak: returns messages user cannot read**
- Asset: Search results
- Likelihood M / Impact H / Risk **H**
- Mitigations: search post-filters by Cedar evaluation; result set redacted to caller-scope; integration test asserts no over-permitted result.

**T-I-06 — File attachment URL leak via shared-link guess**
- Asset: Attachment URL
- Likelihood M / Impact H / Risk **H**
- Mitigations: attachment URLs are signed short-TTL (≤ 15 min); per-fetch Cedar re-evaluation; no public link unless explicitly externalised + Cedar permits.

**T-I-07 — Cross-context routing of personal-DM into professional channel (the parallel-ADR-0135 invariant violation)**
- Asset: Dual-context isolation
- Likelihood L / Impact H (regulatory + privacy breach) / Risk **H**
- Mitigations: data-model invariant — DirectConversation and Channel are distinct entity types; cross-type write rejected by domain layer; LEAN-lane `oya-check-dual-context-isolation` validates type signatures forbid cross-context flows.
- Frameworks: GDPR Art. 25; KR PIPA Art. 17; parallel ADR-0135

### Denial of Service (D)

**T-D-01 — WebSocket gateway storm: many clients reconnecting after blip**
- Asset: Gateway
- Likelihood H / Impact H / Risk **H**
- Mitigations: per-tenant connection rate limit; jittered exponential backoff in client SDK; gateway HPA on CPU + connection-count; pre-warmed standby pods; runbook `runbooks/websocket-storm.md`.
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-02 — Mention storm: one message @-mentions thousands**
- Asset: mention-router worker queue
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-message mention cap (default 50; tenant-configurable to 500); over-cap mentions truncated + sender warned; runbook `runbooks/mention-storm-throttle.md`.

**T-D-03 — Attachment-store outage (S3 unavailable)**
- Asset: Attachments
- Likelihood L / Impact H / Risk **M**
- Mitigations: queued upload retry; DR-pair failover where pack supports; runbook `runbooks/attachment-restore.md`.

**T-D-04 — Search index lag during ingest spike**
- Asset: Search index
- Likelihood M / Impact M / Risk **M**
- Mitigations: backpressure on indexer; live-fallback to Postgres LIKE-search (slower but correct); runbook `runbooks/search-index-rebuild.md`.

**T-D-05 — Redis presence corruption / eviction storm**
- Asset: Presence store
- Likelihood M / Impact M / Risk **M**
- Mitigations: persistence enabled (AOF + RDB); replicated; rebuild from session connections; runbook `runbooks/presence-rebuild.md`.

**T-D-06 — Postgres ingest spike causes message-send latency breach**
- Asset: Message store
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant ingest rate limit; bulk-write buffering; HPA scale-out; sharding past per-cell capacity threshold.

### Elevation of Privilege (E)

**T-E-01 — Cedar policy bug grants channel-admin to non-member**
- Asset: Cedar evaluator
- Likelihood L / Impact H / Risk **M**
- Mitigations: Cedar v3+; fragment fuzz; integration test asserts no over-permitted action; periodic Cedar-fragment-coverage CI lane.

**T-E-02 — Compromised channel-admin pivots to read all tenant channels**
- Asset: Channel ACL
- Likelihood L / Impact H / Risk **M**
- Mitigations: channel-admin scope bounded per channel (NOT tenant-wide admin); admins of one channel cannot read another channel they are not members of; Cedar enforced.

**T-E-03 — Mention-router pivots to read Ontology entities it shouldn't**
- Asset: Ontology read path
- Likelihood L / Impact M / Risk **L**
- Mitigations: mention-router authenticates as scoped SPIFFE identity; ontology enforces per-caller Cedar; mention-router's queries are constrained to `Person`, `Team`, `Channel` resolution shapes.

**T-E-04 — Attachment scanner bypasses scan path**
- Asset: Quarantine boundary
- Likelihood L / Impact H / Risk **M**
- Mitigations: blob lifecycle: PUT → quarantine bucket → scanner → on-clean copy to production bucket; production bucket write-only by scanner SA; runbook `runbooks/attachment-malware-quarantine.md`.

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Mention graph | Mentions across channels correlate to a single user identity-graph | Per-tenant scope; Cedar evaluation; never cross-tenant linkable | L |
| T-L-02 | Identifiability | User handle | Handle within a channel may identify a user across tenants if reused | Per-tenant handle namespace; cross-tenant correlation forbidden | L |
| T-L-03 | Non-repudiation | Personal-DM authorship | Personal user cannot deny authoring DM since session token signs | Acceptable per GDPR Art. 5(2); explicit in onboarding notice | L |
| T-L-04 | Detectability | Channel activity timing | Channel post times reveal team rhythms | Acceptable; tenant business reality; covered by tenant onboarding consent | M |
| T-L-05 | Disclosure | Compliance hold reveals DM content | Hold + four-eyes disclosure inherently exposes DM bodies to admins | Mitigated to acceptable: four-eyes + audit-chain + reason code + tenant-of-tenant disclosure obligation (joint controllership) | M |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user may not know admin can disclose under four-eyes | Tenant DPA includes disclosure clause; tenant onboarding notice required | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | User requests erasure across all channels they participated in | DSR cascade marks messages tombstoned + redacts identifiers; 30d SLA | M |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS on every messenger table | Preventive | axis-messenger | `oya-check-postgres-rls-coverage` lane |
| Per-connection short-lived JWT bound to device + IP | Preventive | axis-messenger | gateway audit log |
| Four-eyes disclosure with distinct principal IDs | Preventive | axis-messenger + ops-security | integration test |
| Cedar policy on every read/write | Preventive | axis-messenger | LEAN coverage lane |
| Attachment scan + quarantine workflow | Preventive | axis-messenger | end-to-end test |
| Personal-DM ciphertext-only at rest | Preventive | axis-messenger | server-decrypt-attempt audit metric (target=0) |
| Audit-chain Ed25519 seal on every state transition | Detective + Non-repudiation | audit-chain | regression tests |
| Cross-context type-system invariant (DirectConversation ≠ Channel) | Preventive | axis-messenger | `oya-check-dual-context-isolation` lane |
| Per-tenant rate + cardinality limits | Preventive (DoS) | axis-messenger | gateway + Postgres metrics |
| DSR cascade for right-to-erasure | Preventive (compliance) | council-privacy | DSR dashboard SLO |

## Residual Risk Acceptance

| Risk ID | Residual | Why | Re-review |
|---|---|---|---|
| T-I-02 (PHI in previews) | L–M | pack-us-healthcare disables previews by default; admin-enabled goes through redactor | Quarterly |
| T-L-04 (timing detectability) | M | Tenant business reality; consent at onboarding | Annually |
| T-L-05 (hold disclosure inherent) | M | Four-eyes + audit are the load-bearing control; user-side opacity unavoidable | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership clause | Annually |
| T-L-07 (erasure best-effort) | M | Retention bounds + audit immutability tradeoff | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 23 sensitive personal info — sensitive messages (medical, juvenile, biometric) require additional consent at channel-create.
- KR 정보통신망법 §49 (intercept) — server-side admin reads only via four-eyes; covered.
- KR 전자문서법 Art. 5 — audit-chain Ed25519 seal satisfies electronic-document integrity.
- KR-ISMS-P §2.7 — access control via Cedar.

### pack-us-healthcare

- HIPAA §164.312(a)(1) — access control via Cedar + RLS.
- HIPAA §164.312(b) — audit-chain ≥ 6y retention overlay; cost-budget.md reflects.
- HIPAA §164.502(b) — minimum-necessary: attachment-preview redactor + search-redaction.
- HIPAA §164.314 (Business Associate) — per-tenant BAA at `microservices/messenger/legal/baa-template.md`.

### pack-eu

- GDPR Art. 25 — privacy-by-design via cross-context invariant.
- GDPR Art. 32 — every mitigation above contributes.
- GDPR Art. 44-50 — pack-eu messages stay in EU pack; cross-pack federation requires SCC.
- ePrivacy Directive Art. 5(3) — confidentiality of communications; covered by Cedar + RLS + E2E.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlay at `regional-packs/<pack>/messenger-overlay.md`; cross-mapped via compliance.md.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered in `compliance.md` |
| ISO 27001:2022 | A.5–A.8 covered |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35 covered in `dpia.md` + `compliance.md` |

## Re-review Triggers

- Any new BC.
- Any change to dual-context invariant.
- Any new attachment-scanner.
- Any Cedar fragment change.
- Annual scheduled review.
- Post-incident review (any Sev-1 or Sev-2).
- Pen-test or audit finding.

## References

- Parallel ADR-0135 (Connect dual-context inherited).
- Bominal ADR-0028, ADR-0111, ADR-0208, ADR-0215.
- ADR-0008 Data Use Boundary.
- `microservices/messenger/PRD.md`.
- `microservices/messenger/dpia.md`.
- `microservices/messenger/compliance.md`.
- `microservices/messenger/policy/dual-context-isolation.md`.
- OWASP API Top 10 (2023).
- NIST SP 800-154.
