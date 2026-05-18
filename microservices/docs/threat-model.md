---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-docs + ops-security
deciders: council-architecture, ops-security, axis-docs, council-privacy
methodology: STRIDE + LINDDUN + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS v4.0 + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140, ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
review_cadence: quarterly + on every BC architectural change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1-CC6.8, CC7.1-CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7-A.5.34, A.8.2-A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44"
  - "WCAG 2.2 AA (accessibility surface)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.514"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014 PAdES", "NIS2 2022/2555", "EU AI Act 2024/1689 Arts. 50/52; Annex III §3"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2/27"]
doc_status: published
---

# Threat Model: docs µservice

## Purpose

Identify, classify, and mitigate threats to the docs µservice's confidentiality, integrity, availability, and privacy posture. Docs carries dual-context PII (personal + professional documents), document authorship + comment identity, organisational content (strategy docs, design docs, ADRs, contracts, clinical notes), and cross-µservice embeds (workflow-studio canvases, sheets cells). A compromise here cascades into corporate-secret leakage, IP exfiltration, ePHI exposure (pack-us-healthcare), and operational-privacy harm.

## Scope

### In-scope

All components introduced for the docs µservice across the eight bounded contexts, deployed in the tenant workload cluster:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (document-metadata store + RLS) | `oya-docs-document-store-*` (11 crates) |
| S3-compatible (OCI Object Storage; content blobs + attachments) | `oya-docs-collab-crdt-*` (9 crates) |
| Redis 7.4 LTS (collab-presence + CRDT op fan-out + cache) | `oya-docs-block-types-*` (7 crates) |
| Loro 1.x CRDT engine (per ADR-DOCS-0001) | `oya-docs-comments-and-suggestions-*` (9 crates) |
| Pandoc 3.x (export-import substrate) | `oya-docs-version-history-*` (8 crates) |
| WeasyPrint 62.x (PDF default backend) | `oya-docs-sharing-and-permissions-*` (8 crates) |
| Chromium-headless (PDF high-fidelity opt-in) | `oya-docs-export-import-*` (11 crates) |
| gVisor runtime (export sandbox) | `oya-docs-embed-resolver-*` (8 crates) |
| ClamAV (default attachment scanner) | |
| OPSWAT MetaDefender (pack-us-healthcare attachment scanner) | |
| `ammonia` Rust crate (HTML sanitiser) | |
| KaTeX / MathJax (math rendering) | |

### Out-of-scope

- Underlying Kubernetes / IaaS layer (owned by `cloud-k8s`).
- Mail delivery (owned by `mail` µservice).
- Tenancy / identity (owned by `tenancy` µservice).
- Audit-chain seal infrastructure (owned by `audit-chain` µservice).
- Workflow-studio canvas authoring (owned by `workflow-studio` µservice; consumed via embed-resolver).
- Observability collectors (owned by `observability` µservice).

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant authors        Customer apps        Reviewers / commenters        │
│         │                       │                       │                  │
│         │ (HTTPS+OIDC+MFA)      │ (per-tenant API key)  │ (signed share-   │
│         ▼                       ▼                       │  link tokens)    │
│  ┌─ Public ingress (Envoy + WAF + DDoS) ──────────────────────────────┐    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Tenant workload cluster ──────────────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → REST + WebSocket ingress                     │
│  ┌─ docs-document-store-rest ──┐ ┌─ docs-collab-crdt-worker (WS) ────────┐ │
│  │ OIDC + RLS                  │ │ WS upgrade + tenant lease             │ │
│  └─────────────────────────────┘ └───────────────────────────────────────┘ │
│  ┌─ docs-sharing-rest ──────┐ ┌─ docs-export-import-rest ────────────────┐ │
│  │ Cedar permission guard   │ │ multipart upload + DOCX/MD parse         │ │
│  └──────────────────────────┘ └──────────────────────────────────────────┘ │
│                                                                            │
│  Trust boundary 2: REST → Postgres (per-tenant RLS + tenant-DEK envelope)  │
│  ┌─ Postgres (document-metadata; per-tenant RLS) ───────────────────┐      │
│  │  Row-level security; encryption-at-rest; tenant-DEK envelope     │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 3: REST → S3 (content blobs; per-tenant prefix; Object Lock)│
│                                                                            │
│  Trust boundary 4: REST → Redis (collab presence; CRDT op spool;            │
│       per-tenant prefix)                                                   │
│                                                                            │
│  Trust boundary 5: Worker → Workflow-event-bus (downstream consumers)      │
│       (audit-chain, mail, messenger, workflow-engine, ontology)            │
│                                                                            │
│  Trust boundary 6: Export-import worker → gVisor sandbox                   │
│       (Pandoc + WeasyPrint + Chromium-headless run inside; tmpfs only;     │
│        no network egress; per-job ephemeral)                               │
│                                                                            │
│  Trust boundary 7: Embed-resolver → cross-µservice mTLS                    │
│       (workflow-studio + sheets + slides; Cedar-gated; pack-tag enforced)  │
│                                                                            │
│  Trust boundary 8: Workers (retention sweep + version compaction +          │
│       embed-refresh) → DB                                                  │
│       (SPIFFE-identity bound; not user-callable)                           │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Eight trust boundaries.

## Assets & Data Classification

Per Bominal ADR-0028 + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Document content (Professional context) | `PROFESSIONAL_DOC_CONTENT` (tenant-DEK encrypted) | Critical | per jurisdiction + legal hold | Postgres + S3 |
| Document content (Personal context) | `PERSONAL_DOC_CONTENT` (E2E where tenant declares) | Critical | per jurisdiction + legal hold | Postgres + S3 |
| Author + commenter identities | `PII_IDENTIFYING` | High | per document retention | Postgres + audit-chain |
| Comment text | `PROFESSIONAL_DOC_CONTENT` / `PERSONAL_DOC_CONTENT` (per parent doc) | High | per document retention | Postgres |
| Suggestion text | as parent | High | per document retention | Postgres |
| Version snapshots | `PROFESSIONAL_DOC_CONTENT` / `PERSONAL_DOC_CONTENT` | Critical | per retention; pinned to compaction policy | Postgres + S3 |
| Share-link tokens | `SECRET` | Critical | per share TTL | OpenBao |
| Per-block ACL records | `BEHAVIORAL_TENANT_PRODUCT` | High | per document retention | Postgres |
| CRDT op spool (transient) | `PROFESSIONAL_DOC_CONTENT` / `PERSONAL_DOC_CONTENT` (until merged + sealed) | Critical | TTL ≤ 60s | Redis |
| Collab presence (cursors) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | session-only | Redis |
| Attachment payloads | as parent doc context | Critical | per retention; legal-hold preserves | S3 (Object Lock for held) |
| Export job artifacts (PDF/DOCX) | as parent doc context | Critical | per retention; signed | S3 |
| Import job source files | as parent | Critical | transient (parsed + dropped); audit-chain seals | tmpfs |
| Legal-hold records | `AUDIT` | Critical | append-only; preserved past retention | Postgres + audit-chain |
| Tenant-DEK | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Embed snapshots (cross-µservice) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | TTL per embed policy | Redis + Postgres |
| Audit-chain seal records | `AUDIT` | High | append-only | audit-chain µservice |
| WCAG accessibility evidence | `INTERNAL_ONLY` | Low | per export | S3 |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Tenant author (human) | Untrusted external | OIDC + MFA | RW own + shared docs |
| Tenant reader (human) | Untrusted external | OIDC + MFA | R per share grant |
| Tenant commenter (human) | Untrusted external | OIDC + MFA | comment per share grant |
| Customer app (machine) | Untrusted external | per-tenant API key (30d rotation) | RW own tenant's docs via SDK |
| External share-link recipient | Untrusted external | signed share-link token (per ADR-DOCS-0004) | R / comment per token scope |
| Workflow µservice | Trusted internal | mTLS + SPIFFE | trigger doc-bound automation |
| Mail µservice | Trusted internal | mTLS + SPIFFE | share-via-email delivery |
| Messenger µservice | Trusted internal | mTLS + SPIFFE | share-to-channel + mention |
| Tenancy µservice | Trusted internal | mTLS + SPIFFE | identity + retention resolution |
| Audit-chain µservice | Trusted internal | mTLS + SPIFFE | seal emission |
| Workflow-studio µservice | Trusted internal | mTLS + SPIFFE (CRDT port-trait re-export only) | embedded canvas snapshot fetch |
| Sheets / slides µservices | Trusted internal | mTLS + SPIFFE | embed snapshot fetch |
| Worker (retention / compaction / embed-refresh) | Trusted internal | SPIFFE + OpenBao SA token | RW on document-store |
| Export-pipeline worker (gVisor-sandboxed) | Trusted internal (sandboxed) | SPIFFE + tmpfs-only | render per-job |
| Council-architecture / ops-security | Trusted internal | OIDC + MFA + JIT | admin-level access |
| External auditor (SOC 2 / ISO 27001) | Read-only time-boxed | OIDC + MFA + JIT ≤ 4h | read-only |
| Attacker (opportunistic / targeted) | Untrusted | none | — |
| Insider (accidental / malicious) | Trusted internal | OIDC + MFA | mitigated via PR review + LEAN gates + audit-chain |

## STRIDE Threat Catalog

### Spoofing

**T-S-01 — Attacker forges share-link token to access a doc they shouldn't**
- Asset: share-link issuance
- L M / I H / Risk H
- Mitigations:
  - Share-link tokens are HMAC-Ed25519-signed `(document_id, grantee_ref, role, expires_at, share_nonce)`; key in OpenBao 90d rotation.
  - Token verification rejects expired + revoked tokens; revocation list cached in Redis per-tenant prefix.
  - Token-binding to receiving IP optional per tenant policy (high-security tenants enable).
- Owner: ops-security + axis-docs
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7; GDPR Art. 32(1)(b)

**T-S-02 — Comment author impersonation (cross-tenant)**
- Asset: comment authorship
- L L / I H / Risk M
- Mitigations:
  - Comments record OIDC subject + SPIFFE-identity; never client-supplied author field.
  - Cedar policy refuses cross-tenant comment writes; LEAN check `oya-check-context-isolation`.
- Owner: ops-security
- Residual: L

**T-S-03 — Embed-source spoofing (forged workflow-studio definition_id in embed)**
- Asset: embed-resolver request
- L L / I M / Risk M
- Mitigations:
  - Embed-resolver calls cross-µservice via SPIFFE mTLS; source µservice validates ownership.
  - Cedar policy refuses embed-resolution that crosses tenant.
- Owner: ops-security
- Residual: L

### Tampering

**T-T-01 — CRDT op forgery (attacker injects ops as another user)**
- Asset: CRDT op stream
- L M / I H / Risk H
- Mitigations:
  - Every CRDT op carries OIDC-derived author SPIFFE-identity + Ed25519 signature at WS gateway.
  - WebSocket gateway lease bound to (tenant_id, document_id); cross-lease ops refused + audit-emitted.
  - LEAN check `oya-check-crdt-op-signature` validates every accepted op has a verifiable signature.
- Owner: axis-docs + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.7

**T-T-02 — DOCX / Markdown / HTML import injection (XXE, macro execution, oversized archive bomb)**
- Asset: import pipeline
- L H / I H / Risk H
- Mitigations:
  - Pandoc + `ammonia` sanitiser refuse macros, external entities, JavaScript URIs, base64 archive bombs > 100MB.
  - Import pipeline runs in gVisor sandbox with tmpfs only; no network egress.
  - Per-import-job size limit (100MB max); excess paginated.
  - Fuzzing: `cargo fuzz` corpus + DOCX / OOXML / Markdown / HTML known-bad inputs.
  - Per-tenant rate limit on imports; max 10/hour.
  - `oya-check-ooxml-import-fidelity` LEAN lane (NEW; per ADR-DOCS-0006) validates parser against OOXML ECMA-376 reference + OWASP injection corpus.
- Owner: axis-docs + ops-security
- Residual: M (fuzz corpus baseline)
- Frameworks: SOC 2 CC6.7, CC7.1; ISO 27001 A.8.28; GDPR Art. 32; OWASP Top 10 A03:2021 (Injection)

**T-T-03 — Tenant-DEK substitution on document read (downgrade attack)**
- Asset: tenant-DEK envelope encryption
- L L / I H / Risk M
- Mitigations:
  - Envelope encryption per Bominal ADR-0111; ciphertext records carry DEK ID + signed integrity check.
  - DEK rotation event re-encrypts; old DEKs maintained for read-only past-record decryption only.
  - LEAN check `oya-check-dek-binding-integrity` validates ciphertext binding.
- Owner: ops-security + cloud-secrets
- Residual: L

**T-T-04 — Suggestion-acceptance race (two reviewers accept incompatible suggestions concurrently)**
- Asset: suggestion state machine
- L M / I M / Risk M
- Mitigations:
  - Suggestion state transitions are CRDT-merged via the same Loro engine; concurrency surfaces conflict in the suggestion-thread UI.
  - LEAN check `oya-check-suggestion-state-determinism` validates state machine determinism.
- Owner: axis-docs
- Residual: L

**T-T-05 — Audit-chain seal omission for document edit**
- Asset: audit emission
- L L / I H / Risk M
- Mitigations:
  - Every edit + share + comment + suggestion + version + export path emits via `audit-chain` µservice port; LEAN check `oya-check-audit-emission-coverage` refuses build if any usecase mutating docs skips emission.
  - Audit-chain µservice acks emission; missing acks trigger `held` SLO state via observability.
- Owner: audit-chain + axis-docs
- Residual: L
- Frameworks: SOC 2 CC4.1, CC7.2, CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2), Art. 30

**T-T-06 — Export pipeline byte-equivalence drift after Pandoc upgrade**
- Asset: export deterministic byte output
- L M / I M / Risk M
- Mitigations:
  - Pandoc pinned per ADR-DOCS-0003 LTS; upgrade gated on 100-doc round-trip-corpus drill.
  - Per-export SHA-256 recorded for evidence; tenant can re-export the same source-version into a deterministic byte stream.
- Owner: axis-docs
- Residual: L

### Repudiation

**T-R-01 — Author denies editing a document version**
- Asset: edit chain
- L L / I M / Risk M
- Mitigations:
  - Every CRDT op carries OIDC + SPIFFE-identity + Ed25519 audit-chain seal.
  - Version snapshots are immutable + chained (Merkle).
- Owner: axis-docs + audit-chain
- Residual: L

**T-R-02 — Reviewer disputes suggestion authorship**
- Asset: suggestion record
- L L / I M / Risk L-M
- Mitigations:
  - Every suggestion records OIDC subject + Ed25519 audit-chain seal; replayable.
- Owner: axis-docs
- Residual: L

### Information Disclosure

**T-I-01 — Per-block ACL bypass (private block leaks into a query result)**
- Asset: per-block ACL enforcement
- L M / I H / Risk H
- Mitigations:
  - Block-tree projection is Cedar-evaluated at every read; LEAN check `oya-check-per-block-acl` (NEW; per ADR-DOCS-0004) validates that no usecase returns a block without ACL pre-check.
  - Postgres RLS at the block level (in addition to doc level); Cedar at the application level.
  - Annual pen-test specifically targets ACL bypass.
- Owner: axis-docs + ops-security
- Residual: L

**T-I-02 — Personal-context document leaks into Professional-context query**
- Asset: dual-context isolation
- L M / I H / Risk H
- Mitigations:
  - Context field is non-nullable + immutable post-creation; Cedar policy `document-isolation.cedar` refuses cross-context read.
  - Rust type system: separate types `PersonalDocument` vs `ProfessionalDocument`; no shared parent struct.
  - LEAN check `oya-check-context-isolation` validates no usecase reads both contexts in same query.
- Owner: axis-docs + ops-security
- Residual: L

**T-I-03 — Embed-resolver leaks source content past ACL boundary**
- Asset: cross-µservice embed
- L M / I H / Risk H
- Mitigations:
  - Embed-resolver re-evaluates ACL against the embedding doc's principal at every fetch; the embed source's policy is also evaluated.
  - Stale embed snapshots are short-TTL (≤ 5 min); revocations propagate via Workflow event.
  - LEAN check `oya-check-embed-resolver-acl-passthrough` validates that the resolver does not bypass source-side ACL.
- Owner: axis-docs + ops-security
- Residual: L

**T-I-04 — Export pipeline leaks plaintext via stderr / temp-file**
- Asset: export pipeline gVisor sandbox
- L L / I H / Risk M
- Mitigations:
  - gVisor sandbox with no network egress, tmpfs only, per-job ephemeral; stderr captured + scrubbed.
  - Per-job tmpfs is wiped before pod scheduling next job.
  - `oya-check-export-sandbox-conformance` LEAN lane validates sandbox config.
- Owner: ops-security + axis-docs
- Residual: L

**T-I-05 — Attachment upload contains stenographic exfiltration channel**
- Asset: attachment storage
- L M / I M / Risk M
- Mitigations:
  - Attachment scan (ClamAV; OPSWAT for pack-us-healthcare) before persistence; per-extension allowlist; archive bombs refused.
  - Server-side image re-encoding strips EXIF + steganographic LSB residue for accepted image types.
- Owner: axis-docs + ops-security
- Residual: M (steganography is hard to fully eliminate)

**T-I-06 — Tenant-DEK leaked via log emission**
- Asset: encryption keys
- L M / I H / Risk H
- Mitigations:
  - DEK wrapped in `Secret<T>` type with stripped `Debug` impl; never serializable.
  - Secret-scanner CI lane scans every commit + log emission.
  - Rotation: 90d for tenant-DEK; rotation event re-encrypts active records.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)

**T-I-07 — Share-link enumeration via timing side-channel**
- Asset: share-link tokens
- L M / I M / Risk M
- Mitigations:
  - Share-link verification returns constant-time response for valid + invalid + revoked.
  - Per-IP rate limit on share-link redemption.
- Owner: ops-security
- Residual: M

**T-I-08 — Comment / suggestion text leak across share boundary**
- Asset: comment thread
- L M / I M / Risk M
- Mitigations:
  - Comments scoped to (document_id, thread_id); per-comment ACL inherits doc ACL by default; private comment marker.
  - Mention notifications to mentioned principal only if mentioned principal has at least comment-level access.
- Owner: axis-docs
- Residual: L

### Denial of Service

**T-D-01 — Editor-session storm: malicious tenant opens 10k editor sessions concurrently**
- Asset: collab-crdt + WS gateway
- L M / I H / Risk H
- Mitigations:
  - Per-tenant editor-session quota; soft cap 1k; hard cap 10k.
  - WS gateway connection rate limit per-tenant + per-IP.
  - Lease pressure metric drives runbook `editor-session-storm-throttle.md`.
- Owner: ops-sre-reliability + axis-docs
- Residual: L

**T-D-02 — Export pipeline storm (1000 concurrent PDF exports of large docs)**
- Asset: export workers
- L M / I H / Risk H
- Mitigations:
  - Per-tenant export quota; cumulative export-second budget.
  - gVisor worker pool size capped; queue depth alarm.
  - Per-tenant rate limit.
- Owner: ops-sre-reliability
- Residual: L

**T-D-03 — Embed-refresh storm (1000 doc reads each fan out to 100 embeds)**
- Asset: embed-resolver
- L M / I H / Risk H
- Mitigations:
  - Embed-resolver coalesces requests per (source, embed_ref) via single-flight.
  - TTL with jitter prevents thundering-herd refresh.
  - Stale-fallback returns prior snapshot when source unavailable.
- Owner: axis-docs
- Residual: L

**T-D-04 — Recursive embed loop (Doc-A embeds Doc-B which embeds Doc-A)**
- Asset: embed-resolver
- L L / I M / Risk M
- Mitigations:
  - Embed depth bounded at 3; cycle detection at resolver layer.
  - Loops emit `EmbedLoopDetected` audit event.
- Owner: axis-docs
- Residual: L

**T-D-05 — Postgres connection pool exhaustion (large query bursts)**
- Asset: Postgres
- L M / I H / Risk H
- Mitigations:
  - HPA scales rest pods; short-term rate-limit at REST layer.
- Owner: ops-sre-reliability
- Residual: L

**T-D-06 — CRDT op log bloat (1M ops on a single document)**
- Asset: collab-crdt persistence
- L M / I M / Risk M
- Mitigations:
  - Version-aligned op log compaction per ADR-DOCS-0001; nightly compaction job; per-doc op-log size SLO ≤ 100MB warm.
- Owner: axis-docs
- Residual: L

### Elevation of Privilege

**T-E-01 — Reviewer escalates to author via crafted suggestion acceptance**
- Asset: document authorship
- L L / I H / Risk M
- Mitigations:
  - Suggestion acceptance can only be performed by author or principal with `accept-suggestion` role; Cedar policy `suggestion-acceptance.cedar`.
- Owner: axis-docs + ops-security
- Residual: L

**T-E-02 — Share-link recipient escalates from view to edit**
- Asset: share grant role
- L L / I H / Risk M
- Mitigations:
  - Share-link role is bound at issuance time + signed in token; client cannot mutate.
  - Role escalation requires re-issuance by author + audit-chain emission.
- Owner: axis-docs
- Residual: L

**T-E-03 — Worker SA token leaked → arbitrary doc writes**
- Asset: worker ServiceAccount
- L L / I H / Risk M
- Mitigations:
  - SA token bound to pod identity; rotation 24h.
  - Network policy: worker → DB only; not user-facing.
- Owner: ops-security + axis-docs
- Residual: L

**T-E-04 — Legal-hold bypass via raw DB access**
- Asset: legal-hold preservation
- L L / I H / Risk M
- Mitigations:
  - Postgres role for application has no DELETE permission; only soft-delete via row column.
  - S3 Object Lock for held content blobs; cannot be deleted past retention without admin OpenBao JIT.
  - Hard-delete restricted to `purge-with-2-person-rule` admin script.
- Owner: ops-security + compliance
- Residual: L

**T-E-05 — Export-pipeline gVisor escape (sandbox break)**
- Asset: export sandbox
- L L / I H / Risk M
- Mitigations:
  - gVisor with seccomp-bpf + AppArmor; no network egress; per-job tmpfs.
  - Pre-deployment escape-attempt test (CVE corpus); `oya-check-export-sandbox-conformance`.
  - Pandoc + WeasyPrint + Chromium pinned per ADR-DOCS-0003.
- Owner: ops-security + axis-docs
- Residual: L

## LINDDUN Privacy Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | comment + suggestion authorship | repeated commentary patterns link individuals across docs | tenant-DEK + access controls; cross-doc aggregation requires explicit grant | M |
| T-L-02 | Identifiability | document content (clinical / legal) | a document's content directly identifies named individuals | encryption-at-rest + per-doc ACL + redaction in export | L |
| T-L-03 | Non-repudiation | suggestion acceptance | end-user disputes acceptance authorship | OIDC + Ed25519 audit-chain seal | L |
| T-L-04 | Detectability | edit pattern timing | edit burst correlates with business events (M&A diligence) | tenant-business-reality; no broader mitigation | M |
| T-L-05 | Disclosure | public share-link drift | tenant misconfigured a private doc as public | LEAN check on public-collection drift + audit | L |
| T-L-06 | Unawareness | mentioned principal | someone mentioned in a doc may not be aware until notification | tenant DPA mandates upstream disclosure | M-H (joint controllership) |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | erasure of an identifier referenced across many docs | DSR cascade: scan all docs; tombstone the identifier; preserve doc minus identifier; legal hold may override | M |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres per-tenant RLS | Preventive | axis-docs | `oya-check-rls-coverage` LEAN lane |
| Tenant-DEK envelope encryption | Preventive | cloud-secrets | DEK binding integrity check |
| Cedar `document-isolation.cedar` | Preventive | ops-security | policy unit-tests |
| Cedar `per-block-acl.cedar` | Preventive | ops-security | LEAN check + pen-test |
| Loro CRDT op signatures | Preventive (S+T) | axis-docs | `oya-check-crdt-op-signature` |
| gVisor sandbox for export pipeline | Preventive (T+I) | ops-security | `oya-check-export-sandbox-conformance` |
| `ammonia` HTML sanitiser | Preventive (T) | axis-docs | fuzz corpus |
| ClamAV / OPSWAT attachment scan | Preventive | ops-security | per-upload metric |
| Image re-encode (strip EXIF + LSB) | Preventive (I) | axis-docs | upload-pipeline test |
| Share-link Ed25519 signature | Preventive | axis-docs | constant-time verify + revoke list |
| Per-tenant rate limits | Preventive (DoS) | ops-sre-reliability | metrics |
| Embed-resolver single-flight + TTL jitter | Preventive (DoS) | axis-docs | embed-resolver SLO |
| Embed depth bound | Preventive (DoS) | axis-docs | cycle detection test |
| Ed25519 audit-chain seal | Detective + non-repudiation | audit-chain | per-event emission |
| SA-token rotation 24h, DEK 90d, share-key 90d | Preventive | cloud-secrets | OpenBao audit |
| 2-person rule on hard-delete | Preventive (insider) | ops-security | OpenBao JIT |
| Postgres role no-DELETE + S3 Object Lock | Preventive | axis-docs + ops-security | role audit |
| DSR cascade runner | Compliance | council-privacy | DSR queue SLO |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-T-02 (.docx injection) | M | fuzz corpus baseline; never fully eliminable | Quarterly |
| T-I-05 (attachment steganography) | M | LSB stripping is best-effort; new techniques emerge | Quarterly |
| T-I-06 (DEK leak via logs) | M | human-error baseline | Quarterly |
| T-I-07 (share-link timing side-channel) | M | inherent network-timing characteristic | Annually |
| T-L-01 (linkability) | M | legitimate document use case | Annually |
| T-L-04 (detectability via edit-timing) | M | tenant business reality | Annually |
| T-L-06 (joint-controllership unawareness) | M-H | tenant disclosure responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | hold-vs-erasure tension | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-KR-01 | I — Information disclosure | KR PIPA Art. 17 cross-border transfer must be SCC-gated | per-pack data residency pinning at `iac/kustomize/overlays/pack-kr/`; cross-pack ingress refused at OIDC issuance |
| T-KR-02 | N — Non-compliance | 전자문서법 Art. 5 audit-chain integrity for tenant documents | Ed25519 + Merkle audit-chain per Bominal ADR-0028; tamper detection runs on every read |
| T-KR-03 | I — Linkability | KR PIPA Art. 23 special-category data in clinical notes | `#[data_class(SENSITIVE_PIPA_ART23)]`; Cedar refuses cross-tenant disclosure |
| T-KR-04 | T — Tampering | KR-FSS document-retention floor 5y for financial-sector tenants | retention floor enforced at `document-store-domain`; legal-hold extends past floor |

References: KR PIPA Art. 17 + Art. 23 + Art. 28; KR-FSS supervisory regulations; 전자문서법 (Electronic Document Act); PIPC Notice 2020-7.

### pack-eu (GDPR + ePrivacy + EU AI Act + eIDAS PAdES)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-EU-01 | I — Information disclosure | GDPR Art. 6(1)(a) lawful-basis for cross-tenant share | Cedar-gated cross-tenant share grant; consent recorded in audit-chain |
| T-EU-02 | N — Non-compliance | GDPR Art. 17 right-to-erasure must reconcile with legal-hold | erasure refused while legal-hold active; tenant comms emitted with concrete-reason citation |
| T-EU-03 | N — EU AI Act Annex III §3 employment-context | T1/T2 AI writing-assist in HR-context (hiring / performance review) may trigger high-risk | T1/T2 HR-context overlays REFUSED at Cedar layer pending ADR-DOCS-0005 conformity assessment |
| T-EU-04 | T — Cross-border | GDPR Chapter V transfers require SCCs | per-pack data residency; cross-pack transfers SCC-gated |
| T-EU-05 | I — eIDAS PAdES | exported PDFs for legal evidence may require advanced electronic signature | PAdES B-LT level via WeasyPrint + signer integration per pack-eu overlay |

References: GDPR Regulation (EU) 2016/679; ePrivacy Directive 2002/58/EC; EU AI Act Regulation (EU) 2024/1689 Art. 50 + Annex III §3; eIDAS 910/2014.

### pack-us-healthcare (HIPAA + clinical notes)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-HC-01 | I — Information disclosure | HIPAA 45 CFR §164.502(b) minimum-necessary | data-class `PHI` on every clinical-note field; Cedar refuses access outside care-team scope |
| T-HC-02 | N — Non-compliance | HIPAA 45 CFR §164.312(a)(2)(iv) encryption controls | Tenant-DEK envelope at rest; TLS 1.3 in transit; OPSWAT MetaDefender attachment scanning |
| T-HC-03 | I — Linkability | HIPAA 45 CFR §164.514(b) de-identification not applicable for clinical content (patient identifier is core) | encryption-at-rest + per-block ACL; legal-hold for ePHI per BAA |
| T-HC-04 | T — Audit-chain | HIPAA 45 CFR §164.312(b) audit controls | Ed25519 + Merkle audit-chain |

References: HIPAA 45 CFR §164.308 + §164.312 + §164.502 + §164.514; FDA 21 CFR Part 11 (electronic records); BAA template per `legal/baa-template.md`.

### pack-jp (APPI)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-JP-01 | I | APPI Art. 24 cross-border transfer requires consent or equivalence | per-pack residency; cross-pack transfers consent-gated |
| T-JP-02 | N | APPI Art. 22 personal-data leakage notification 3-business-day window | incident-response runbook 3-business-day fire |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/docs-overlay.md`.

- PDPA (Singapore) §13 + §24
- Privacy Act 1988 (Australia) APP 8 + APP 11
- DPDPA 2023 (India) §6-11
- LGPD (Brazil) Art. 7 + Art. 33
- UAE PDPL Federal Decree-Law 45/2021 Art. 22
- KSA PDPL Royal Decree M/19 Art. 29 + SDAIA-approved transfer mechanism + Sharia retention (per-tenant extension)

## Re-review Triggers

- Any change to dual-context isolation invariant.
- Any change to CRDT library version (per ADR-DOCS-0001).
- Any change to export pipeline backend (per ADR-DOCS-0003).
- Any new pack activation.
- Any change to per-block ACL semantics (per ADR-DOCS-0004).
- Quarterly scheduled.
- Post-incident.
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519).
- ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140.
- ADR-WS-0001 (cross-µservice CRDT alignment).
- ADR-DOCS-0001 through ADR-DOCS-0006.
- `microservices/docs/PRD.md`, `dpia.md`, `compliance.md`, `policy/*.cedar`.
- Microsoft Threat Modeling (STRIDE), LINDDUN privacy.
- NIST SP 800-154; OWASP ASVS v4.0.
- WCAG 2.2 AA (accessibility).
- eIDAS 910/2014 (PAdES).
