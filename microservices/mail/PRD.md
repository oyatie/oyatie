---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-mail
microservice: mail
status: Accepted
sales_segment: shared-substrate-and-product
tier: hero-product
milestone_first_ship: M03-connect-dissolution
bominal_source: [ADR-0208, ADR-0210, ADR-0215]
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/products/connect/mail.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
owner_team: axis-mail + council-privacy
doc_status: published
---

# PRD-mail: Standards-compatible mail with dual-context isolation, legal hold, and eDiscovery

## Purpose

The `mail` µservice is oyatie's corporate-and-personal mail surface. SMTP/IMAP/JMAP standards-compatible at the edge; native dual-context (Personal B2C vs Professional B2B) isolation per ADR-0126; tenant retention + legal hold + chain-of-custody-preserving eDiscovery export; mail-to-Workflow handoff with explicit consent/policy basis; cross-organization mail-server pattern (each tenant operates its own logical mail-server inside the µservice).

Connect dissolved per ADR-0132 + parallel ADR-0126 into mail/messenger/calendar/community/social/shorts/network/anonymous. This document is the canonical PRD for the **mail** sub-product; calendar, messenger, etc., own their own PRDs.

Inherits Bominal ADR-0208 (dual-context unified channel hub), ADR-0210 (M03 KR group-mail launch), ADR-0215 (retention/legal-hold dual-context). The oyatie variant treats mail as a µservice in its own right rather than a "Connect app" — this is the load-bearing structural change of ADR-0132 + ADR-0126.

## Tenant Value

- **Tenant Outcome 1 — Standards-compatible mail without vendor coupling.** Tenants get a mailbox endpoint that speaks SMTP submission + SMTP relay + IMAP + JMAP + REST; no Gmail/Outlook/Exchange API dependency. Migration adapter ingests from any of the above without losing message hash, folder labels, or retention class.
- **Tenant Outcome 2 — Dual-context isolation by construction.** A user's Personal mailbox is invisible to org admins, legal-hold workflows, and eDiscovery export even when held in the same physical cluster. Professional mailboxes are encrypted under tenant DEK and never plaintext-indexed; org admins can hold + export per ADR-0215's four-eyes contract.
- **Tenant Outcome 3 — eDiscovery that survives audit.** Sealed exports with Ed25519 chain-of-custody seal, time-bound expiry, and four-eyes approval for plaintext disclosure. Export verification re-derives the digest from source blocks without trusting the producer.
- **Tenant Outcome 4 — Legal hold that survives retention.** Engaging a hold blocks retention expiry within scope at the kernel layer (hold-before-purge invariant). Releasing a hold is audit-chained. Hold-affecting actions across mail/messenger/calendar/community are coordinated by the legal-hold engine (this µservice owns mail's hold engine and consumes the cross-channel coordinator from `audit-chain`).
- **Tenant Outcome 5 — Mail-to-Workflow native.** A work mail item promotes to a Workflow task only with an explicit user action or a tenant-declared policy basis; never silently mined. Every handoff emits an audit record linking the source message, extracted payload, and workflow item.
- **Internal Outcome 6 — Cross-organization mail-server pattern.** Each tenant's logical mail-server is a partition (per-tenant-DEK, per-tenant SMTP IP reputation, per-tenant retention policy), but mail-server software is shared (oya-mail-* crates). Operations is one µservice; legal stance is per-tenant.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | external sender | to submit a message via SMTP to `mx.<tenant-domain>.oyatie.dev:25` | inbound mail reaches my recipient's mailbox | inbound-smtp | Must |
| FR-02 | internal user | to send a message via SMTP submission on :587 or JMAP | outbound mail is signed (DKIM), authenticated (SPF + DMARC), and delivered | outbound-smtp | Must |
| FR-03 | end user | to read/move/label/search messages via IMAP, JMAP, or REST | I can use Apple Mail, Thunderbird, mobile mail apps, or web | imap-frontend, search-index | Must |
| FR-04 | mail admin | to author per-mailbox or per-tenant retention policy | regulated retention floors and minimums are enforced | retention-policy | Must |
| FR-05 | compliance officer | to scope a legal hold by mailbox/date/query | matching messages survive retention expiry until release | legal-hold | Must |
| FR-06 | compliance officer | to export a held set as a sealed eDiscovery package with chain-of-custody seal | the export is admissible and tamper-evident | legal-hold, search-index | Must |
| FR-07 | end user | to switch persona (Personal ↔ Professional) without leaking inbox state | the Personal mailbox is unreachable from the Professional context APIs and vice versa | dual-context-isolation | Must |
| FR-08 | tenant migrator | to ingest existing mail from Gmail / Exchange / IMAP-source while preserving source hash, folder, and retention class | migration is chain-of-custody-preserving | mailbox-store | Must |
| FR-09 | workflow operator | to convert a mail into a Workflow item via explicit handoff | the action emits a `MailWorkflowHandoffCreated` audit record linking source + payload + policy basis | mailbox-store (via Workflow event) | Must |
| FR-10 | DLP/abuse system | to detect spam/phishing inbound and DLP-affecting outbound | abuse is blocked; tenant SMTP reputation is preserved | inbound-smtp, outbound-smtp | Must |
| FR-11 | end user | to perform encrypted-token search across their mailbox without exposing plaintext index | search remains accurate; index is policy-scoped | search-index | Must |
| FR-12 | tenant operator | to configure custom domain (DKIM + SPF + DMARC + MTA-STS + TLS-RPT) | outbound mail meets modern deliverability standards | outbound-smtp | Must |
| FR-13 | compliance officer | to export a `RetentionLedger` showing all retention-expiry and hold actions across the tenant | audits across mail/messenger/calendar reconcile to a single ledger | retention-policy | Must |
| FR-14 | end user | to receive S/MIME-signed or PGP-signed mail and view signature status | end-to-end signed mail is rendered correctly with verification status | imap-frontend | Should |
| FR-15 | enterprise admin | to view per-tenant deliverability dashboard (bounce rate, spam complaints, deliverability score) | I can manage IP reputation | outbound-smtp | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Inbound mail receive (SMTP DATA → mailbox-persisted) | ≤200ms | ≤1s | ≤3s | per ADR-0210 enterprise inbox target |
| Outbound mail submission (SMTP submission → queued) | ≤50ms | ≤300ms | ≤1s | tenant-perceived UX |
| Outbound mail delivery (queue → recipient MX) | ≤5s | ≤30s | ≤5min | dependent on recipient infrastructure |
| Mailbox search (encrypted-token search on 100k-message mailbox) | ≤100ms | ≤500ms | ≤2s | per Microsoft Exchange parity benchmark |
| IMAP fetch (latest 50 headers) | ≤80ms | ≤300ms | ≤1s | mobile/desktop client UX |
| eDiscovery export (10-year archive of 5GB mailbox) | — | ≤24h | — | sealed bundle including chain-of-custody seal |
| Legal hold engage (scope select → block confirmed) | ≤500ms | ≤2s | ≤5s | hold-before-purge invariant |
| Retention expiry sweep (per-mailbox, p99) | — | ≤30s | — | nightly worker; idempotent |
| Mailbox restore (point-in-time, 5GB mailbox) | — | ≤15min | — | RPO 5min; RTO 15min |
| Compose-to-Workflow handoff | ≤200ms | ≤500ms | ≤2s | tenant-visible UX |

### Security

- All mailbox blobs (MIME bodies + attachments) encrypted at rest under tenant DEK (envelope encryption per Bominal ADR-0111).
- Personal-context blobs encrypted under user-derived DEK (when user opts into E2E per Bominal ADR-0208 personal-pillar policy); org admins cannot decrypt.
- All headers indexed via encrypted-search tokens (Cipher-Match per `oya-mail-search-index`) — never plaintext.
- TLS 1.3 required on SMTP submission + IMAP + JMAP + REST. STARTTLS opportunistic on inbound 25 (per RFC 8314); MTA-STS + TLS-RPT published per tenant.
- DKIM + SPF + DMARC signing/verification on every outbound message.
- IMAP/SMTP brute-force defence: per-IP rate-limit + per-mailbox lockout + CAPTCHA cliff per `threat-model.md` T-S-04.
- SMTP relay abuse refused: outbound submission requires authenticated mailbox user; relaying without authentication is forbidden (open-relay refused at config-level per `policy/data-residency.md`).
- Legal-hold bypass attempts emit `mail_legal_hold_bypass_attempt_total` + Sev-1 page.

### Audit + Compliance

- Every `MessageReceived`, `MessageSent`, `LegalHoldEngaged`, `LegalHoldReleased`, `RetentionExpired`, `EDiscoveryExportSealed`, `MailWorkflowHandoffCreated` emits Ed25519 audit-chain record (per Bominal ADR-0028 + ADR-0008 data-use-boundary).
- eDiscovery export bundles signed; verifier re-derives digest from source blocks; mismatch quarantines bundle.
- Audit-chain seal latency ≤1s per `(tenant, period)`.
- HIPAA-pack: BAA required pre-onboarding (mail may carry PHI); audit-log retention ≥ 6y per §164.316(b)(2).
- KR-pack: 전자문서법 Art. 5 (electronic document integrity) satisfied via audit-chain Ed25519 seals.

### Availability + SLO

- Inbound SMTP availability target: 99.95% monthly (RFC 5321 requires graceful queue/retry on outage).
- Outbound delivery availability target: 99.9% monthly.
- IMAP/JMAP availability target: 99.95% monthly.
- Search availability target: 99.9% monthly.
- eDiscovery export endpoint: 99.5% monthly (lower target — admin tool; non-tenant-blocking).
- RTO: ≤15 min for mailbox restore; ≤5 min for SMTP-frontend HA failover. RPO: ≤5 min (sync-replicated WAL).

### Data residency

- Mailbox blobs and metadata inherit tenant's `jurisdiction_code` per ADR-0117.
- Postgres + S3-compatible per pack region (pack-pinning per `policy/data-residency.md`).
- Cross-pack replication forbidden by default; eDiscovery export across packs requires tenant SCC.
- KR-FSS regulated tenants: KMS-in-KR mandatory; 5y retention floor per KR commercial code.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). The mail µservice has 8 primary BCs reflecting its complex domain.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `mailbox-store` | `oya-mail-mailbox-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,worker,sdk,app}` | Mailbox + thread + message + folder model; CRUD + encryption + retention bookkeeping | `Mailbox`, `Thread`, `MailMessage`, `Folder`, `RetentionClass`, `MimeBlob` |
| `inbound-smtp` | `oya-mail-inbound-smtp-{kernel,domain,usecase,api,adapter,adapter-smtp,worker,app}` | SMTP receiver on :25 + :465 (implicit-TLS); DKIM/SPF/DMARC verify; spam/phishing detection; cross-tenant routing | `IncomingSession`, `RecipientResolution`, `AbuseVerdict`, `DkimResult` |
| `outbound-smtp` | `oya-mail-outbound-smtp-{kernel,domain,usecase,api,adapter,adapter-smtp,worker,app}` | SMTP submission on :587; DKIM sign; deliverability queue; bounce processing; reputation tracking | `OutboundEnvelope`, `DeliveryAttempt`, `BounceClassification`, `ReputationScore` |
| `imap-frontend` | `oya-mail-imap-frontend-{kernel,domain,usecase,api,adapter,rest,worker,app}` | IMAP + JMAP + REST mailbox-read frontends; per-folder pagination; flag synchronization | `ImapSession`, `MailboxView`, `MessageFlags`, `JmapCommandBatch` |
| `search-index` | `oya-mail-search-index-{kernel,domain,usecase,api,adapter,adapter-search-index,worker,sdk,app}` | Encrypted-token search index; per-tenant; policy-scoped; supports header + body tokens | `SearchToken`, `IndexShard`, `EncryptedQuery`, `ResultPage` |
| `legal-hold` | `oya-mail-legal-hold-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Scoped legal hold; hold-before-purge invariant; four-eyes plaintext-disclosure approval; chain-of-custody seal | `LegalHold`, `HoldScope`, `HoldApproval`, `ChainOfCustodySeal`, `EDiscoveryExportJob` |
| `retention-policy` | `oya-mail-retention-policy-{kernel,domain,usecase,api,adapter,worker,app}` | Per-tenant + per-mailbox retention policies; statutory floor enforcement; expiry scheduler | `RetentionPolicy`, `RetentionClass`, `ExpiryBatch`, `RetentionLedgerEntry` |
| `dual-context-isolation` | `oya-mail-dual-context-isolation-{kernel,domain,usecase,api,adapter,app}` | Personal vs Professional context boundary at the kernel; cross-context routing forbidden | `ContextKind`, `OwnershipPillar`, `ContextBoundary`, `ContextSwitch` |

Naming justification — `mailbox-store` (representative; same shape for all 8 BCs):

```
NAME: oya-mail-mailbox-store-<layer>
JUSTIFICATION:
- microservice = mail: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. Connect dissolved per ADR-0126; mail stands alone.
- bc-tokens = mailbox-store: primary BC for mailbox + thread + message storage
  + encryption + retention bookkeeping. ADR-0056 v4.1 BC-optionality rule honoured
  (7 sibling BCs exist; explicit BC token justified).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (Mailbox, MailMessage, Thread,
    Folder, RetentionClass, MimeBlob). Zero I/O. Carries data_class annotations
    per Bominal ADR-0028 + oya-check-data-class lane.
  - domain: pure mailbox-folder-thread math + MIME parsing + envelope encryption
    primitives + RetentionClass arithmetic.
  - usecase (per ADR-0106): orchestrators reading/writing mailbox + emitting events.
  - api: protocol-neutral typed I/O contracts; consumed by rest/sdk; depends on kernel.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified (per ADR-0105 Amendment 3); implements
    MailboxRepository + ThreadRepository against Postgres with RLS per-tenant.
  - adapter-s3: backend-qualified; implements MimeBlobStore against S3-compatible
    object storage with SSE-KMS per-tenant DEK envelope encryption.
  - rest: HTTP handler/route layer for REST mailbox API (JMAP-equivalent).
  - worker: nightly retention sweep + restore drill + reputation cron.
  - sdk: client library for tenant-side mailbox automation.
  - app: composition root; wires worker + rest + adapter clients.
- exemptions claimed: none. -adapter-postgres + -adapter-s3 use canonical
  *-adapter-<backend> pattern; no exception required.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-X | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|
| `mailbox-store` | x | x | x | x | x | postgres, s3 | x | x | x | x |
| `inbound-smtp` | x | x | x | x | x | smtp | — | x | — | x |
| `outbound-smtp` | x | x | x | x | x | smtp | — | x | — | x |
| `imap-frontend` | x | x | x | x | x | — | x | x | — | x |
| `search-index` | x | x | x | x | x | search-index | — | x | x | x |
| `legal-hold` | x | x | x | x | x | — | x | x | x | x |
| `retention-policy` | x | x | x | x | x | — | — | x | — | x |
| `dual-context-isolation` | x | x | x | x | x | — | — | — | — | x |

Total crates: ~62 (sizes vary; minimal BCs trim to ~6 layers; full BCs hit 11).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `MailboxRepository` | `oya-mail-mailbox-store-kernel` | `-adapter-postgres` | `PII_IDENTIFYING` (sender/recipient) + `BEHAVIORAL_TENANT_PRODUCT` |
| `MimeBlobStore` | `oya-mail-mailbox-store-kernel` | `-adapter-s3` | `PII_IDENTIFYING` + sometimes `PHI` (HIPAA pack) |
| `RetentionLedgerWriter` | `oya-mail-retention-policy-kernel` | `-adapter-postgres` | `AUDIT` |
| `LegalHoldEngine` | `oya-mail-legal-hold-kernel` | `-usecase` (orchestrator over MailboxRepository + RetentionLedgerWriter) | `AUDIT` |
| `EDiscoveryExporter` | `oya-mail-legal-hold-kernel` | `-usecase` | `AUDIT` + `SENSITIVE_PIPA_ART23` |
| `SmtpInboundReceiver` | `oya-mail-inbound-smtp-kernel` | `-adapter-smtp` | `PII_IDENTIFYING` |
| `SmtpOutboundSubmitter` | `oya-mail-outbound-smtp-kernel` | `-adapter-smtp` | `PII_IDENTIFYING` |
| `ImapSessionHandler` | `oya-mail-imap-frontend-kernel` | `-adapter` | `PII_IDENTIFYING` |
| `EncryptedTokenIndex` | `oya-mail-search-index-kernel` | `-adapter-search-index` (Tantivy or Elasticsearch) | `BEHAVIORAL_TENANT_PRODUCT` (encrypted tokens; no plaintext) |
| `DkimSigner` / `DkimVerifier` | `oya-mail-outbound-smtp-kernel` / `-inbound-smtp-kernel` | `-adapter` | `SECRET` (DKIM private keys) |
| `ContextBoundaryGuard` | `oya-mail-dual-context-isolation-kernel` | `-usecase` (called from every cross-context API) | `AUDIT` |
| `AbuseClassifier` | `oya-mail-inbound-smtp-kernel` | `-adapter` (SpamAssassin / Rspamd integration) | `BEHAVIORAL_TENANT_PRODUCT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `mail` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice mail` — dependency-direction
- `oya gate validate lean-a2 --microservice mail` — cross-product-refusal
- `oya gate validate port-location --microservice mail` — ports in kernel
- `oya gate validate layer-correctness --microservice mail`
- `oya gate validate per-microservice-layout --microservice mail`
- `oya gate validate statelessness --microservice mail` — relevant for SMTP frontends
- `oya gate validate shardability --microservice mail` — tenant_id + mailbox_id sharding
- `oya gate validate dual-context-cross-boundary --microservice mail` — NEW; refuses any code path that reads a Personal mailbox from a Professional API
- `oya gate validate retention-floor-conformance --microservice mail` — NEW; refuses retention configs below statutory minimum
- `oya gate validate dkim-key-rotation-conformance --microservice mail` — NEW; refuses DKIM keys older than rotation window

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `MessageReceived` | inbound SMTP DATA accepted + mailbox-persisted | `audit-chain`, `messenger` (action-card-in-mail), tenant Workflow if subscribed | mail-receive state-machine |
| `MessageSent` | outbound queue accepted submission | `audit-chain`, tenant Workflow | — |
| `MessageDelivered` | recipient MX returned 2xx | `audit-chain`, `outbound-smtp` reputation cron | — |
| `MessageBounced` | recipient MX returned 5xx or DSN | reputation cron, tenant deliverability dashboard | bounce-classification state |
| `LegalHoldEngaged` | compliance officer applied scoped hold; four-eyes approval present | `audit-chain`, retention scheduler (skip held messages), tenant compliance UI | hold-lifecycle state |
| `LegalHoldReleased` | compliance officer released hold | `audit-chain`, retention scheduler | hold-lifecycle state |
| `RetentionExpired` | nightly worker passed a message's retention horizon AND no hold matches | `audit-chain`, mailbox-store (soft-delete) | — |
| `EDiscoveryExportSealed` | export job completed; chain-of-custody seal emitted | `audit-chain`, compliance UI | — |
| `MailWorkflowHandoffCreated` | user explicitly handed off a mail to Workflow | `audit-chain`, `workflow-engine` | — |
| `MailDeliverabilityReputationChanged` | per-tenant SMTP reputation score crossed threshold | tenant deliverability dashboard, ops-mail on-call | — |
| `MailContextSwitched` | user switched Personal ↔ Professional persona | `audit-chain` | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | `mailbox-store` + `outbound-smtp` | provision per-tenant DKIM keypair; create default mailbox/aliases; register tenant in cross-org pattern |
| `TenantOffboarded` | `tenancy` | `mailbox-store` + `legal-hold` | freeze retention; respect any active hold; coordinate eDiscovery export window |
| `WorkflowHandoffCommitted` | `workflow-engine` | `mailbox-store` | mark source mail with `handoff_committed=workflow_item_id` label |
| `LegalHoldEngagedAcrossChannels` | `audit-chain` (cross-channel coordinator) | `legal-hold` | apply mail-scope hold for the cross-channel hold ID |
| `JurisdictionPolicyChanged` | `tenancy` | `retention-policy` | recompute retention floors |
| `KmsKeyRotated` | `cloud-secrets` | `mailbox-store` (DEK envelope re-wrap) | re-wrap mailbox DEKs under new KEK |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `MailMessage{message_id, mailbox_id, tenant_id, context_kind, headers_ciphertext, body_ciphertext, retention_policy_id, legal_hold_ids, data_class, received_at}` | `addressed_to→Mailbox`, `replies_to→MailMessage`, `attached_to→MimeBlob` | `mailbox-store` | Ed25519 |
| `Mailbox{mailbox_id, tenant_id, owner_ref, context_kind, aliases, quota_policy, region}` | `owned_by→User`, `partition_of→TenantMailServer` | `mailbox-store` | Ed25519 |
| `LegalHold{hold_id, tenant_id, scope, approved_by, approved_at, released_at}` | `holds→MailMessage` | `legal-hold` | Ed25519 |
| `EDiscoveryExportJob{export_id, scope, requested_by, approved_by, artifact_digest, expires_at, chain_of_custody_ref}` | `exports→LegalHold` | `legal-hold` | Ed25519 |
| `RetentionPolicy{policy_id, tenant_id, class, statutory_floor, configured_floor}` | `applies_to→Mailbox` | `retention-policy` | Ed25519 |
| `DeliverabilityReputation{tenant_id, ip_pool, dkim_domain, score, evaluated_at}` | `for→Tenant` | `outbound-smtp` | Ed25519 |
| `TenantMailServer{tenant_id, smtp_ips[], dkim_domains[], mta_sts_policy, tls_rpt_endpoint}` | `serves→Tenant` | `outbound-smtp` + `inbound-smtp` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `User` (with persona context) | `dual-context-isolation` | `filter(user_id).where(active=true)` for persona switch validation |
| `Tenant` (with jurisdiction + KMS region) | `retention-policy` + `mailbox-store` | for retention-floor lookup + DEK region pinning |
| `WorkflowDefinition` (linked from mail handoff) | `mailbox-store` (for handoff event emission) | `filter(workflow_id)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Microsoft Exchange Online | Enterprise mail + retention + legal hold + eDiscovery | SMTP/IMAP + retention + Vault + eDiscovery export | `learn.microsoft.com/exchange` + `learn.microsoft.com/purview` |
| Google Workspace Gmail | Enterprise mail + Vault legal hold | mail + Vault hold + Takeout export | `support.google.com/vault` |
| Apple Mail (iCloud mail) | Personal mail + push | IMAP-compatible mail | `developer.apple.com/icloud` |
| Zoho Mail | Enterprise mail + admin maturity | SMTP/IMAP + per-user retention | `zoho.com/mail/admin` |
| Proton Mail | E2E encrypted mail (personal pillar parity) | PGP-encrypted personal mailbox | `proton.me/support/mail` |
| Naver Works Mail | KR enterprise mail | SMTP/IMAP + KR-FSS retention | `naver.worksmobile.com` |
| KR-FSS regulated mail vendors (Daou Cyworks, Hancom Office Mail) | KR-regulated mail | retention 5y + KR-PIPA-compliant | vendor docs |
| Postfix + Dovecot (DIY) | Self-hosted mail | SMTP/IMAP standards-compatibility | `postfix.org` + `dovecot.org` |

Key parity gaps to close (ordered by priority):

1. **Dual-context isolation native** — no competitor isolates Personal and Professional mailboxes at the kernel layer; org-admin export of personal mail is structurally impossible in oyatie mail. Target: 0 cross-pillar exports across all packs.
2. **Workflow-native handoff with explicit consent/policy basis** — Exchange + Gmail have rules that auto-trigger; none emit audit-chain records linking source + payload + workflow item with explicit consent verification. Target: 100% of handoffs audit-chained.
3. **Sealed eDiscovery with re-derivable digest** — Microsoft Purview + Google Vault produce export bundles but the digest is provider-asserted; oyatie's seal is re-derivable from source blocks. Target: third-party-verifiable seal.
4. **Self-hosted with no vendor coupling** — Exchange Online + Gmail are SaaS; oyatie mail is self-hosted under tenant residency.
5. **Per-tenant SMTP IP reputation as a first-class FinOps + ops surface** — Gmail bundles IP reputation opaquely; Exchange relies on Microsoft's pool; oyatie exposes per-tenant pool + score + remediation actions.
6. **KR-FSS pack at launch** — match Naver Works Mail and Daou Cyworks on KR-PIPA + KR commercial code retention compliance.

## Performance Targets

(See §"Non-Functional Requirements" Performance table. Headline budgets:)

| Metric | Target | Verification |
|---|---|---|
| Inbound receive p99 | ≤1s | cargo bench -p oya-mail-inbound-smtp-domain -- receive_p99 |
| Outbound submission p99 | ≤300ms | cargo bench -p oya-mail-outbound-smtp-domain |
| Search p99 (100k-message mailbox) | ≤500ms | cargo bench -p oya-mail-search-index-domain |
| eDiscovery export (10y archive) | ≤24h | scripted end-to-end drill (quarterly) |
| Mailbox restore (5GB) | ≤15min | scripted drill |
| Hold engage | ≤2s | unit test in oya-mail-legal-hold-domain |

Error budget:
- Monthly error budget for inbound SMTP: 0.05 % (≈22 min/month).
- Burn-rate alarm on inbound SMTP: 14.4× burn rate over 1h triggers page.
- Error budget policy: `microservices/mail/runbooks/error-budget-policy.md` (see runbooks).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for mailbox metadata (per-tenant RLS); S3-compatible object storage for MIME blobs; Tantivy/Elasticsearch for search index; in-memory queues for SMTP submission (durable via WAL); persistent volumes for SMTP queue spool.

**Active-active compatibility**:
- `inbound-smtp` and `outbound-smtp` workers: stateless-compatible (queue is in Postgres/S3-backed).
- `imap-frontend`: stateless (sessions are short-lived).
- `mailbox-store` writers: per-tenant Postgres shard; horizontally shardable by tenant_id.
- `search-index`: per-tenant index sharding.
- `legal-hold`: per-tenant; serialised within tenant via Postgres advisory lock.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active mailboxes | 100k | 1M | Postgres CPU > 70% |
| Inbound message rate | 1k/s | 10k/s | SMTP receiver queue depth > 30s |
| Outbound message rate | 1k/s | 10k/s | Outbound queue depth > 60s |
| Search queries/s | 100/s | 1000/s | Tantivy index node CPU > 70% |
| Concurrent IMAP sessions | 50k | 500k | IMAP frontend memory > 80% |

Scale-out policy:
- HPA on SMTP frontends (CPU > 70%; min 4 replicas for HA; max 50).
- Postgres scaling: Citus distributed table by tenant_id when single-Postgres approaches 80% capacity.
- Search index: per-tenant Tantivy sharding.
- Pre-warmed pool: 2 standby pods per critical surface (inbound-smtp, outbound-smtp, imap-frontend); cold-start ≤500ms.

Cross-region story:
- M03 launch: pack-kr (primary) + pack-eu + pack-us standby (DR pairs per pack).
- HIPAA pack: us-ashburn-1 + us-phoenix-1 DR pair (both HIPAA-eligible).
- Cross-pack replication forbidden by default; eDiscovery export across packs requires tenant SCC.

Sharding:
- Mailboxes partition by `tenant_id+mailbox_id` (per Bominal ADR-0208).
- MIME blobs partition by `tenant_id+mailbox_id+blob_hash` (content-addressable S3 prefix).
- Search index per-tenant.
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Work mailbox receives a message → encrypted under tenant DEK + tagged Org pillar + indexed without plaintext + covered by retention policy | `cargo nextest -p oya-mail-mailbox-store-domain --test storage::test_work_mail_tenant_dek_and_retention` |
| AC-02 | Personal mailbox receives a message → tagged Person pillar + unavailable to org-admin search/export APIs | `cargo nextest -p oya-mail-mailbox-store-domain --test personal::test_org_admin_cannot_export_personal_mail` |
| AC-03 | Compliance officer opens scoped legal hold → matching messages survive retention expiry + all actions audit-chained | `cargo nextest -p oya-mail-legal-hold-domain --test test_hold_blocks_mail_deletion` |
| AC-04 | Tenant migrates from Gmail/Microsoft → import preserves source hash + folder labels + message IDs + retention class | `cargo nextest -p oya-mail-mailbox-store-app --test migration::test_import_preserves_chain_of_custody` |
| AC-05 | User turns work mail into Workflow item → WorkflowHandoffAuditLog links original message + extracted payload + consent/policy basis + workflow item id | `cargo nextest -p oya-mail-mailbox-store-domain --test workflow_handoff::test_mail_to_workflow_requires_policy_basis` |
| AC-06 | Inbound SMTP DKIM verification on a tampered message fails | `cargo nextest -p oya-mail-inbound-smtp-domain --test dkim::test_tampered_message_fails_dkim` |
| AC-07 | Outbound SMTP signs every message with DKIM + applies SPF/DMARC alignment | `cargo nextest -p oya-mail-outbound-smtp-domain --test dkim::test_outbound_signs_with_dkim` |
| AC-08 | Encrypted search returns correct results for a 10k-message mailbox without ever materialising plaintext in the index | `cargo nextest -p oya-mail-search-index-domain --test encrypted_token::test_search_correctness_without_plaintext` |
| AC-09 | eDiscovery export of a 10-year archive (5GB) completes within 24h SLA + bundle digest verifies from source blocks | scripted e2e drill in `tests/e2e/ediscovery-export.sh` |
| AC-10 | Hold engaged within 2s of approval; bypass attempts emit metric + page | `cargo nextest -p oya-mail-legal-hold-domain --test test_hold_engage_under_2s` + scripted e2e bypass-attempt drill |
| AC-11 | Cross-context routing forbidden: Professional API call with Personal mailbox ID returns 403 + audit-emitted | `cargo nextest -p oya-mail-dual-context-isolation-domain --test test_cross_context_routing_refused` |
| AC-12 | `oya gate validate per-microservice-layout --microservice mail` exits 0 | ADR-0131 lane |
| AC-13 | `oya gate validate authority-cohesion` exits 0 | ADR-0123 lane; HG-MAIL registered |
| AC-14 | KR pack: 5y retention floor enforced for KR-FSS tenants | `cargo nextest -p oya-mail-retention-policy-domain --test pack_kr::test_kr_fss_5y_floor` |
| AC-15 | HIPAA pack: BAA absence refuses pack-us-healthcare onboarding | `cargo nextest -p oya-mail-mailbox-store-app --test pack_us_healthcare::test_baa_required` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | JMAP vs IMAP relative priority for client SDK first-launch — bias to JMAP modern protocol or honour Apple-Mail-default IMAP? | axis-mail | ADR-NNNN-jmap-imap-priority |
| 2 | Per-tenant SMTP IP pool sizing and warmup protocol — shared warm pool or per-tenant cold-start? | ops-deliverability + axis-mail | ADR-NNNN-smtp-ip-pool |
| 3 | Search index choice: Tantivy (Rust-native; embedded) vs Elasticsearch (mature but JVM) — performance + ops trade-off | axis-mail | ADR-NNNN-search-index-backend |
| 4 | Personal mailbox E2E encryption key recovery: user-held-only or escrow-with-2-person-rule? | council-privacy + axis-mail | ADR-NNNN-personal-mail-key-recovery |
| 5 | Cross-channel hold coordination authority: `audit-chain` µservice or `legal-hold` itself? | council-architecture | resolved 2026-05-17: audit-chain owns coordinator; mail owns mail-scope hold |
| 6 | Mail-to-Workflow extraction prompt safety: agentic LLM extraction is consent-gated; what's the per-tenant default? | council-privacy + axis-workflow | ADR-NNNN-mail-workflow-extraction-default |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data use boundary | engaged for mail content + metadata |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0117 | Cloud-native infrastructure | residency authority |
| ADR-0123 | Hyperscaler maturity claim gate | HG-MAIL registers |
| ADR-0126 | Connect full social network super-app (parallel-session) | Connect dissolution; dual-context invariants |
| ADR-0130 | Agentic SLO-gated promotion | mail consumes the gate |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 | No-suite forward policy | mail is a µservice, not a "Connect app" |
| ADR-0133 | Cross-tenant mail-server pattern | per-tenant logical partition shape |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited |
| Bominal ADR-0210 | M03 KR group-mail launch | inherited |
| Bominal ADR-0215 | Connect retention/legal-hold dual-context | inherited |
