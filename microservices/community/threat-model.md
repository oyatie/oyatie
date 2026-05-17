---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-security
deciders: council-architecture, ops-security, axis-community, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/connect-unbundle.json]
review_cadence: quarterly + on every architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022 A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P", "KR PIPA Arts. 15/17/18/22-2/23", "KCSC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/.310/.312/.314/.316", "42 CFR Part 2 (SUD) when applicable"]
  pack-eu: ["GDPR Arts. 25/32/35/44-50", "DSA 2022/2065 (when oyatie qualifies)", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24"]
  pack-sg: ["PDPA 2012 §11-26"]
  pack-au: ["Privacy Act 1988 APP 1-13", "Online Safety Act 2021"]
  pack-in: ["DPDPA 2023 §6-10", "IT (Intermediary Guidelines and Digital Media Ethics Code) Rules 2021"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33"]
  pack-ae: ["UAE PDPL FDL 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "Anti-Cyber Crime Law (Royal Decree M/17/2007)"]
doc_status: published
---

# Threat Model: community µservice

## Purpose

Identify, classify, and mitigate threats to the confidentiality, integrity, availability, and privacy posture of the org-wide community surface. The community surface is a high-visibility tenant-facing UGC channel; threats here include post tampering, vote manipulation, moderation bypass, mass-spam abuse, KB article impersonation, and cross-tenant mention leakage.

## Scope

### In-scope

| Layer-A (adopted OSS / managed) | Layer-B (oyatie-owned) |
|---|---|
| Postgres + Citus + Patroni (post-store / voting-engine / moderation-queue / kb-article-store) | `oya-community-post-store-*` (10 crates) |
| Elasticsearch / Tantivy (search-index) | `oya-community-thread-tree-*` (7 crates) |
| Redis (hot-feed cache + vote buffer) | `oya-community-voting-engine-*` (8 crates) |
| S3 (KB attachment store) | `oya-community-moderation-queue-*` (9 crates) |
| ClamAV (attachment AV scan inline) | `oya-community-kb-article-store-*` (10 crates) |
| OpenBao (secrets) | `oya-community-search-index-*` (8 crates) |
| `foundry-guardrails` classifier (spam/abuse) | Cedar policy fragments at `policy/*.cedar` |

### Out-of-scope

- Threats to the underlying Kubernetes cluster — owned by `cloud-k8s`.
- Threats to OpenBao itself — owned by `cloud-secrets`.
- Threats to the spam/abuse classifier model itself — owned by `foundry-guardrails`. This document inherits classifier threats as upstream.
- Threats to `messenger` (real-time chat) — owned by the sibling Connect-unbundle µservice.
- Threats to `tenancy` JWT issuance — owned by `tenancy`.

## Trust Boundaries

```text
┌─ Internet ────────────────────────────────────────────────────────────────┐
│                                                                           │
│   Tenant members              Tenant moderators            Public readers │
│     (HTTPS + OIDC)              (HTTPS + OIDC + 2FA)        (HTTPS only)  │
│         │                            │                          │         │
│         ▼                            ▼                          ▼         │
│  ┌─ Public ingress (Envoy/Istio) ─────────────────────────────────────┐   │
│  │  TLS termination + WAF + DDOS + per-tenant rate-limit             │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                              │                                            │
└──────────────────────────────│────────────────────────────────────────────┘
                               ▼
┌─ Community microservice cluster ──────────────────────────────────────────┐
│                                                                           │
│  oya-community-post-store-rest ───► Postgres (Citus + RLS per tenant)     │
│  oya-community-kb-article-store-rest ─► Postgres + S3 (per-tenant prefix) │
│  oya-community-voting-engine-rest ──► Redis (per-tenant key prefix)       │
│  oya-community-search-index-rest ───► Elasticsearch (per-tenant index)    │
│  oya-community-moderation-queue-rest ─► Postgres + audit-chain bridge     │
│                                                                           │
│  worker (NATS subscriber) ─► foundry-guardrails-bridge ─► classifier      │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

## Asset Inventory

| Asset | Data class | Owner | Confidentiality | Integrity | Availability |
|---|---|---|---|---|---|
| Post body + revisions | BEHAVIORAL_TENANT_PRODUCT | post-store | High | High | High |
| KB article + attachments | BEHAVIORAL_TENANT_PRODUCT | kb-article-store | High | High | High |
| Vote tally | AUDIT | voting-engine | Medium | Critical | High |
| Moderation action log | AUDIT | moderation-queue | High | Critical | High |
| Search index docs | INTERNAL_ONLY | search-index | Medium | Medium | High |
| Mention resolution table | INTERNAL_ONLY | post-store | Medium | High | High |
| Cedar policy fragments | INTERNAL_ONLY | ops-security | High | Critical | High |
| Audit-chain seal records | AUDIT | audit-chain (consumer) | Critical | Critical | Critical |

## STRIDE Threat Catalogue

### T1 — Spoofing (S)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T1.1 | Attacker impersonates a tenant member when authoring a post | Stolen / replayed JWT | High | tenancy short-lived JWT (15 min); refresh rotation; HMAC binding to client cert |
| T1.2 | Attacker impersonates a KB article author | Forged `author_id` on insert | Critical | author_id sourced from JWT claim, not request body; RLS rejects mismatch |
| T1.3 | Attacker impersonates a moderator | Privilege escalation via Cedar attribute spoofing | Critical | Cedar attribute source is `tenancy`-signed claim; oya-community never reads claims from request body |
| T1.4 | Webhook spoofing from forged foundry-guardrails events | NATS subject hijack | High | NATS mTLS + per-publisher cert pinning |

### T2 — Tampering (T)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T2.1 | Post tampering (silent edit without revision record) | Direct DB write bypass | Critical | All writes via usecase layer; revision append is a domain invariant; audit-chain seal on every edit |
| T2.2 | Vote manipulation (script-driven mass upvotes) | Bot account farm | High | Per-member vote rate-limit; foundry-guardrails velocity check; account-age + reputation gate |
| T2.3 | Moderation action retro-edit | Direct DB write to moderation_actions | Critical | Append-only table (Postgres trigger forbids UPDATE/DELETE); audit-chain Merkle witness |
| T2.4 | Search index injection (poisoning ranking) | Crafted post body with adversarial tokens | Medium | Tokeniser sanitises; ranker uses signals beyond raw body (account-age, vote ratio, moderator endorsement) |
| T2.5 | KB attachment substitution after publication | S3 object overwrite | Critical | S3 object lock + per-attachment hash recorded in Postgres |

### T3 — Repudiation (R)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T3.1 | Member denies authoring an inflammatory post | Audit log gap | High | Every write event sealed via audit-chain within 1 s; signed witness |
| T3.2 | Moderator denies hiding a legitimate post | Audit log gap on moderation action | High | Append-only moderation_actions + audit-chain Ed25519 signed by moderator's JWT-bound key |

### T4 — Information Disclosure (I)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T4.1 | Cross-tenant post leakage via search | Missing RLS in Elasticsearch query | Critical | Per-tenant index + index-name binding to JWT tenant claim at gateway |
| T4.2 | Cross-tenant mention resolution leak | Mention search across all tenants | Critical | Mention resolution scoped to tenant_id at usecase layer; Cedar deny on cross-tenant |
| T4.3 | KB attachment direct-URL access without auth | Stolen S3 presigned URL | High | Short-lived presigned URLs (5 min); attach JWT subject claim in `x-amz-meta-` then verify at adapter-s3 fetch |
| T4.4 | Search-index snapshot exposure | Backup leak | High | Backups encrypted at rest (KMS); access via IAM role + just-in-time; backup catalog in audit-chain |
| T4.5 | PII in post body persisted to search index without retention | DSR Right-to-Erasure gap | High | DSR cascade: delete from post-store → invalidate search-index → delete attachment → tombstone in audit-chain |

### T5 — Denial of Service (D)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T5.1 | Mass-spam flood from a compromised tenant member | Bot script | High | Per-member post rate-limit (60/min); per-tenant overall cap; foundry-guardrails fast-path block |
| T5.2 | Vote-storm against a target post | Coordinated bot | High | Per-member vote rate-limit (600/min); per-post velocity anomaly detector |
| T5.3 | Search-index rebuild storm | Cascading tenant reindex triggers | Critical | Per-tenant rebuild scheduler with token-bucket; staggered windows; runbook `search-rebuild.md` |
| T5.4 | Moderation queue OOM | Coordinated flag campaign | High | Per-tenant queue depth cap; overflow to S3 cold queue; worker drain priority |
| T5.5 | Large-attachment upload DoS | Multi-GB resumable upload across many sessions | High | Per-tenant upload bandwidth cap; chunk-size cap; ClamAV inline scan budget |
| T5.6 | Hot-feed Redis eviction storm | Trending post causing whole-tenant eviction | Medium | Per-tenant Redis namespace + memory quota; LFU eviction; fallback to Postgres warm path |

### T6 — Elevation of Privilege (E)

| ID | Threat | Vector | Impact | Mitigation |
|---|---|---|---|---|
| T6.1 | Member self-promotes to moderator | Cedar attribute injection | Critical | Cedar attributes always read from `tenancy`-signed claim; never from request body |
| T6.2 | Moderator escalates to admin via API gap | Unscoped moderator action endpoint | Critical | Cedar fragment `tenant-scope.cedar` permits only `moderator` actions; `admin` actions require `tenant_admin` role |
| T6.3 | Tenant operator reads cross-tenant audit log | Auditor scope drift | Critical | `auditor-scope.cedar` per-tenant; auditor JWT bound to a single tenant_id |

## LINDDUN Privacy Threats

| ID | Threat | Mitigation |
|---|---|---|
| L1 | Linkability of pseudonymous posters across spaces | Per-space pseudonym option; salted pseudonym; tenant-controlled |
| L2 | Identifiability via writing-style fingerprinting | Out of community scope; tenant warning in compliance.md |
| L3 | Non-repudiation of pseudonymous posts (forced de-anonymisation) | Court order workflow documented in compliance.md; tenant admin opt-in |
| L4 | Detectability of moderation patterns | Aggregate metrics only; no per-action timing in tenant dashboards |
| L5 | Disclosure of information via mention resolution | Cedar denies cross-tenant; deny on mention to non-member |
| L6 | Unawareness of profiling via foundry-guardrails | DPIA disclosure; tenant opt-in for classifier features |
| L7 | Non-compliance (DSR gap, retention drift) | DSR cascade runbook; retention matrix in data-residency.md |

## OWASP Top 10 (2021) Mapping

| OWASP | Community-specific manifestation | Mitigation |
|---|---|---|
| A01 Broken Access Control | Cross-tenant post / KB / vote / moderation read | Cedar + RLS belt-and-braces |
| A02 Cryptographic Failures | S3 attachment in plaintext / backup unencrypted | KMS-encrypted everything; rotation policy in incident-response.md |
| A03 Injection | SQL injection via post body; ES query injection | Parameterised queries; ES query templates; tokeniser sanitisation |
| A04 Insecure Design | Moderation bypass via direct adapter-postgres write | All writes via usecase; domain invariants enforced |
| A05 Security Misconfiguration | Elasticsearch open to internet; Redis no AUTH | mTLS-only mesh; secrets via OpenBao; CIS Postgres + ES + Redis benchmarks |
| A06 Vulnerable Components | Outdated ES / Postgres / Redis | Renovate + Trivy CI gate |
| A07 Identification + Auth Failures | JWT replay; weak tenant binding | Short-lived JWT; mTLS; refresh rotation |
| A08 Software + Data Integrity | Tampered Helm chart; supply-chain attack | Sigstore-signed images; SBOM in CI; cosign verify on admission |
| A09 Logging + Monitoring Failures | Missing audit log entries | audit-chain seal on every write; gap-detector alert |
| A10 SSRF | Adapter-s3 fetching attacker-controlled URL | Allowlist; in-VPC endpoint only |

## Vote-Manipulation Specific Controls

Vote manipulation is the highest-effort threat per parallel-session ADR-0126. Layered controls:

1. **Account-age gate** — votes from accounts < 24 h old are rate-limited 10×.
2. **Reputation gate** — members < 100 reputation can downvote no more than 30 / day.
3. **Velocity detector** — `foundry-guardrails` consumes `VoteCast` events; alert + auto-pause on z-score > 5 vs. tenant baseline.
4. **Coordinated cohort detector** — graph clustering on IP + user-agent + session; same-cluster bursts auto-quarantined to moderation-queue.
5. **Idempotency** — per `(member_id, post_id)` only one vote (Redis SET NX + Postgres unique constraint).

## Moderation-Bypass Specific Controls

1. **Append-only moderation_actions table** — Postgres trigger forbids UPDATE / DELETE.
2. **Per-action audit-chain seal** — Ed25519 signed by moderator's session key.
3. **Two-eyes on destructive actions** — `delete_post`, `purge_kb_article` require two moderators within 24 h or escalate to tenant_admin.
4. **Reversal trail** — every `unhide` / `unlock` keeps the prior action chain.

## KB Article Impersonation Specific Controls

1. **author_id = JWT subject** — never request-body sourced.
2. **Tenant-level publication review** — opt-in workflow gate before article is index-published.
3. **Attachment integrity** — sha256 recorded in Postgres; S3 object-lock; verify on read.
4. **Revision diff in tenant UI** — every revision view shows author + sha256 of attachments.

## Threat Acceptance + Residual Risk

- Style-based de-anonymisation (L2): accepted residual; tenant warning surfaced in DPIA Annex A.
- Coordinated cohort attacks across multiple tenants: detected at `foundry-guardrails` cross-tenant aggregator; community surface only sees per-tenant signal.

## Review + Update

- Review quarterly + on every architecture change.
- Pen-test: annual + on every major version bump.
- Tabletop exercise: quarterly on incident-response.md scenarios.
