---
doc_class: DPIA
template_id: TPL-DPIA
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-community
deciders: council-privacy, ops-security, axis-community, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/connect-unbundle.json]
related_artifacts:
  - microservices/community/threat-model.md
  - microservices/community/policy/community-isolation.md
  - microservices/community/policy/data-residency.md
  - microservices/community/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — PARTIAL (vote-velocity profiling; coordinated-cohort detection)"
  - "Art. 35(3)(b): large-scale processing of special category data — POSSIBLE (PHI in posts under pack-us-healthcare opt-in)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
doc_status: published
---

# DPIA: community µservice

## 1. Description of the Processing

The community µservice processes user-generated content (posts, replies, votes, KB articles, attachments, flags, moderation actions) inside tenant boundaries. Personal data processed includes: member name, member ID, post body (free text, may contain PII), KB article body + attachments, vote records, mention references, IP / user-agent (for abuse detection only, retained 30 days then aggregated).

### 1.1 Nature

- Storage of UGC on tenant-isolated Postgres (Citus) + Elasticsearch + Redis + S3.
- Profiling via vote-velocity + coordinated-cohort detection (only for abuse; never for content ranking visible to other members).
- Retention per `data-residency.md` retention matrix.

### 1.2 Scope

- Per-tenant — never cross-tenant. Cedar fragments deny cross-tenant access.
- Per-region — data resides in the tenant's `jurisdiction_code` region. Cross-region replication is opt-in.

### 1.3 Context

- Tenant operators publish; tenant members read + write; tenant moderators triage flags + take action.
- Cross-µservice consumers: `foundry-guardrails` (classifier input), `audit-chain` (sealing), `ontology` (cross-product entity links), `messenger` (mention resolution), `tenancy` (identity).

### 1.4 Purpose

- Lawful basis: GDPR Art. 6(1)(b) (contract with the tenant) + 6(1)(f) (legitimate interest, abuse prevention).
- KR PIPA: tenant-provided consent at member onboarding.
- HIPAA: tenant Business Associate Agreement when PHI may be processed.

## 2. Necessity + Proportionality

| Test | Assessment |
|---|---|
| Specified, explicit, legitimate purpose | YES — community surface with abuse prevention |
| Adequate, relevant, limited | YES — only data needed for surface + abuse detection |
| Lawful basis under Art. 6 | YES — contract (Art. 6(1)(b)) + legitimate interest (Art. 6(1)(f)) |
| Special-category lawful basis under Art. 9 | OPT-IN — Art. 9(2)(a) explicit consent for PHI under pack-us-healthcare |
| Data subject rights respected | YES — DSR cascade runbook; Right-to-Erasure cascades to post-store + search-index + S3 |
| Sub-processors documented | YES — see Annex B |

## 3. Risk Assessment

| Risk | Likelihood | Severity | Inherent | Mitigation | Residual |
|---|---|---|---|---|---|
| Cross-tenant post leakage | Low | Critical | High | Cedar + RLS belt-and-braces | Low |
| PII persisted to search-index after DSR delete | Medium | High | High | DSR cascade with completion attestation | Low |
| Moderation action target identified to attacker | Medium | High | High | Audit log access limited to tenant_admin + auditor; aggregate metrics in dashboards | Low |
| Vote profiling reveals member preferences to other members | Low | Medium | Medium | Vote-velocity signals never surfaced outside moderation-queue | Low |
| KB attachment leak via stolen presigned URL | Medium | High | High | 5-min presigned URLs; JWT subject claim verification | Low |
| Coordinated-cohort detection false-positive bans innocent members | Medium | Medium | Medium | Two-eyes review for ban actions; appeal workflow; reversal trail | Low |
| Foundry-guardrails classifier model bias against minority dialect | Medium | Medium | Medium | Per-tenant tunable thresholds; moderation human-in-loop; quarterly bias audit | Medium |
| PHI in posts under pack-us-healthcare opt-in flow leaks via search | Medium | Critical | High | Pack-us-healthcare disables cross-space search by default; warning surfaced; audit-chain seal | Medium |

## 4. Data Subject Rights

| Right | Implementation |
|---|---|
| Right to access (Art. 15) | Tenant member dashboard exposes their post history; admin export via `kb-article-store-rest.export` |
| Right to rectification (Art. 16) | Member can edit own posts; revisions sealed |
| Right to erasure (Art. 17) | DSR cascade: tombstone in post-store + invalidate search-index + delete attachment in S3 + audit-chain witness |
| Right to portability (Art. 20) | JSON export of member's posts + KB articles |
| Right to object (Art. 21) | Member opt-out from foundry-guardrails profiling; abuse detection falls back to rate-limit only |
| Right against ADM (Art. 22) | Auto-moderation actions are reversible; tenant-admin override; appeal workflow |

## 5. Consultations

| Party | Status |
|---|---|
| Council-privacy | Reviewed 2026-05-17 |
| Ops-security | Reviewed 2026-05-17 |
| Tenant Data Protection Officer (when EU tenant onboards) | Per-tenant onboarding workflow |
| KR PIPC (when KR tenant onboards) | Pack-kr overlay covers KR-ISMS-P + PIPA Art. 23 |

## 6. Outcome

DPIA outcome: **proceed with controls**. No high-risk residual finding requires DPA prior consultation under Art. 36 for the standard processing baseline.

## Annex A — Special Considerations

- **Style-based de-anonymisation** (L2 LINDDUN): residual risk accepted. Tenant warning surfaced at member onboarding when pseudonymous posting is offered.
- **Foundry-guardrails model bias**: quarterly per-pack audit; bias report shared with tenant.

## Annex B — Sub-processors

| Sub-processor | Purpose | Region | Contract |
|---|---|---|---|
| AWS S3 (or equivalent per pack) | KB attachment store | Per tenant region | Standard DPA |
| Elastic / OpenSearch operator | Search index | Self-hosted in oyatie cluster | N/A (no external sub-processor) |
| Postgres + Citus | Post / vote / moderation / KB store | Self-hosted | N/A |
| Redis | Hot-feed cache + vote buffer | Self-hosted | N/A |

## Annex C — Pack-specific Overlays

- **pack-us-healthcare**: PHI opt-in flow; classifier disabled for opted-out members; BAA in place.
- **pack-kr**: PIPA Art. 23 sensitive data flagging; explicit consent at onboarding.
- **pack-eu**: GDPR Art. 30 Records of Processing maintained by `audit-chain`.
- **pack-jp**: APPI Art. 24 cross-border transfer disclosure; opt-in.

## Annex D — Review Triggers

- New processing purpose introduced (e.g., AI summarisation in M03).
- New data class introduced.
- New sub-processor onboarded.
- Annual cadence.
