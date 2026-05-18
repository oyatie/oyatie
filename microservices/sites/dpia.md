---
doc_class: DPIA
template_id: TPL-DPIA
microservice: sites
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-sites
related_adrs: [ADR-0028, ADR-0117, ADR-0140 (retired per ADR-0145), ADR-SITES-0001, ADR-SITES-0006]
doc_status: published
gdpr_art_35_threshold: yes — (a) systematic monitoring of public accessible area (public-facing sites + analytics); (c) processing of sensitive personal data when pack-us-healthcare patient portals are deployed
---

# Data Protection Impact Assessment (DPIA) — sites µservice

## 1. Identification

| Field | Value |
|---|---|
| µservice | sites |
| Controller | tenant (oyatie is processor per GDPR Art. 28) |
| Joint controller | none |
| Processor | oyatie B.V. + sub-processors per `legal/sub-processors.md` |
| DPO | council-privacy@oyatie.dev |
| Assessment date | 2026-05-17 |
| Re-assessment trigger | new BC; new sub-processor; new pack; new T-tier; new cross-µservice integration |

## 2. Processing description

### 2.1 Purpose

Enable tenants to author, publish, and serve websites (intranet +
public) — namely: create + edit + publish pages; bind custom domains
with TLS; structure content via CMS-collections; index for site
search; render via CDN; collect privacy-preserving analytics; embed
forms; cross-link to docs + community; co-edit via Loro CRDT;
optionally generate pages from prompts via T2 AI-page-build.

### 2.2 Lawful basis (GDPR Art. 6)

| Activity | Basis | Notes |
|---|---|---|
| Authoring (tenant editor) | Art. 6(1)(b) contract performance | tenant DPA defines |
| Publishing public pages | Art. 6(1)(b) | tenant elected to publish |
| Analytics (privacy-preserving) | Art. 6(1)(f) legitimate interest | balanced against visitor expectation; ePrivacy Art. 5(3) consent for non-strictly-necessary cookies |
| Custom-domain ACME | Art. 6(1)(b) | required for service |
| Audit-chain seal | Art. 6(1)(c) legal obligation | accountability under Art. 5(2) |
| AI-page-build prompt processing | Art. 6(1)(a) explicit consent + Art. 6(1)(b) | tenant editor opts in |
| Cross-µservice form binding | Art. 6(1)(b) | tenant flow |
| Legal-hold preservation | Art. 6(1)(c) | per jurisdiction-pack |

### 2.3 Data categories

| Category | Examples | Class |
|---|---|---|
| Editor identity | user_id, email, OIDC subject | `PII_IDENTIFYING` |
| Tenant identity | tenant_id, tenant_name, plan-tier | `BEHAVIORAL_TENANT_PRODUCT` |
| Page content (private/intranet) | page body, blocks, draft state | `BEHAVIORAL_TENANT_PRODUCT` |
| Page content (published-public) | page body, blocks, published state | `PUBLIC_BY_TENANT_CHOICE` |
| CMS-collection entries | structured records | `BEHAVIORAL_TENANT_PRODUCT` or `PUBLIC_BY_TENANT_CHOICE` |
| Custom-domain + TLS | domain name, ACME challenge response, cert serial, private key | `SECRET_TLS_KEY` (private key only) + `INTERNAL_ONLY` (rest) |
| Analytics (per-visitor) | hashed IP, hashed session id, user-agent class, referrer, page path | `BEHAVIORAL_VISITOR_AGGREGATE` |
| AI-page-build prompt | prompt text, model output | `BEHAVIORAL_TENANT_PRODUCT` (tenant-DEK-wrapped) |
| Audit-chain seal | event hashes, Ed25519 sigs | `AUDIT` |
| Loro CRDT log | per-page edit history | `BEHAVIORAL_TENANT_PRODUCT` |

### 2.4 Purpose limitation

Data collected for one purpose (e.g., page authorship) is NOT used for
another purpose (e.g., model training) without explicit tenant consent.
Cross-tenant training on tenant content is structurally forbidden per
ADR-SITES-0006 + foundry-runtime private-inference channel.

### 2.5 Data subjects

| Subject | Notes |
|---|---|
| Tenant editor | the user authoring sites |
| Tenant operator | the user binding domains, managing tenant |
| Public visitor | anonymous reader of published page |
| Authenticated visitor | reader of intranet page (per tenant directory) |
| Form submitter (transient) | sites does not persist; forms µservice does |
| Patient (pack-us-healthcare) | reader of patient-portal page; PHI processing per HIPAA |

### 2.6 Recipients

| Recipient | Basis | Class |
|---|---|---|
| audit-chain (internal µservice) | Art. 6(1)(c) | seal recipient |
| ontology (internal µservice) | Art. 6(1)(b) | entity-bindings |
| workflow-engine (internal µservice) | Art. 6(1)(b) | event consumer |
| observability (internal µservice) | Art. 6(1)(f) | telemetry |
| forms (internal µservice) | Art. 6(1)(b) | form binding |
| Let's Encrypt (sub-processor) | Art. 6(1)(b) | ACME server |
| CDN edge provider (sub-processor — per ADR-SITES-0003 Cloudflare-class) | Art. 6(1)(b) | edge delivery |
| LLM provider (sub-processor — when T2 AI-page-build engaged) | Art. 6(1)(a) consent | model inference (tenant-DEK-wrapped) |

Sub-processor list per `legal/sub-processors.md`.

### 2.7 Cross-border transfers

Per `policy/data-residency.md`: tenant data pinned to tenant's
jurisdiction-pack. Cross-pack flow only with SCC clause executed in
tenant DPA. Schrems II transfer-impact assessment (TIA) template at
`legal/tia-template.md`.

### 2.8 Retention

| Data | Default retention | Pack overlay |
|---|---|---|
| Published page (active) | until tenant unpublishes / deletes | pack-kr: 5y floor for KR-FSS financial-services tenants |
| Page draft | until tenant deletes / 90d idle | pack-eu: 30d idle (GDPR Art. 5(1)(e) minimisation) |
| Page version history | 365d default | pack-us-healthcare: 6y (HIPAA §164.316) |
| Audit-chain seal | 7y default | pack-us-healthcare: ≥ 6y; pack-eu: 6y financial; pack-kr: 5y financial |
| Analytics records | 13mo default | pack-eu: 6mo (EDPB recommendation) |
| AI-page-build prompts | 30d (post-bound by tenant choice) | pack-eu: 14d |
| Loro CRDT log | 90d (compactable past retention horizon) | pack-us-healthcare: 6y |

## 3. Necessity + proportionality assessment

| Test | Verdict |
|---|---|
| Is processing necessary for the purpose? | Yes — site publishing requires the listed categories |
| Could the purpose be met with less data? | Analytics: per-visitor IP processed as hash-bucket only (no raw IP stored); ePrivacy bar met |
| Is the data accurate? | Tenant editor controls accuracy; audit-history surface for diff |
| Is retention minimised? | Yes — per-pack retention floors honoured |
| Are data subjects informed? | Yes — tenant DPA + cookie/consent banner (where required) |
| Is automated decision-making engaged? | T2 AI-page-build is a creative-assistance flow, NOT a decision affecting legal/significant rights of a data subject; Art. 22 not engaged at default; if HR/legal/medical-overlay enabled, Annex III §3 high-risk applies → REFUSED until conformity (ADR-SITES-0006) |

## 4. Privacy by design + default (GDPR Art. 25)

| Principle | Implementation |
|---|---|
| Data minimisation | Type-narrowed projections; analytics hash-bucketing; AI prompt scoping |
| Pseudonymisation | Visitor sessions are random salts; tenant audit logs reference user_id but published surface anonymises |
| Encryption | Tenant-DEK envelope for non-public content; TLS 1.3 in transit; ACME private key in OpenBao |
| Access control | Cedar + RLS + per-tenant API key + OIDC + MFA |
| Storage limitation | Per-pack retention enforced at event-store; legal-hold exception |
| Auditability | Audit-chain seal on every state transition |

## 5. Risk assessment (LINDDUN-class)

| Risk | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|
| Subdomain takeover via stale DNS | Medium | High | Cert revoke cascade on unbind; ADR-SITES-0004 | Low |
| AI-page-build prompt leak to LLM provider | High (without mitigation) | Medium | Tenant-DEK wrap + private-inference; ADR-SITES-0006 | Low |
| Anonymous visitor linkability via cookie correlation | Medium | Medium | Plausible-class no-cookie; salt rotation | Low |
| Editor escalation to other tenant's site | Low | High | Cedar + RLS + LEAN | Very low |
| CDN provider's edge logs disclose visitor IP | Medium | Medium | DPA + log IP-hashing at edge; sub-processor agreement | Low |
| WCAG 2.2 AA regression affects disabled users | Low | High (ADA) | Refuse-publish at < 100%; pack-us-healthcare gated | Low |
| Sitemap.xml + robots.txt reveal intranet structure | Medium | Medium | sitemap entries gated by visibility=public | Low |
| Loro CRDT cross-tenant replay | Low | High | Per-tenant CRDT log namespace; signed ops | Low |
| Search-index cross-tenant leak | Low | High | Per-tenant Meilisearch index | Very low |
| Pack-us-healthcare PHI in CMS-collection without BAA | Low | Critical (HIPAA breach) | LEAN refuses pack-us-healthcare tenant without `baa_on_file=true` | Very low |

## 6. Data-subject rights

| Right | Implementation |
|---|---|
| Art. 15 access | tenant editor: full author history; public visitor: per-pack DSAR via tenant |
| Art. 16 rectification | tenant editor authors directly; public visitor via tenant DSAR |
| Art. 17 erasure | page-usecase erasure orchestrator + legal-hold reconciliation; cascades to S3 published artifact, sitemap, search index, CRDT log |
| Art. 18 restriction | tenant can suspend page publishing |
| Art. 20 portability | site-export endpoint emits portable-text + HTML + assets archive (.zip) |
| Art. 21 objection | tenant controls publication |
| Art. 22 automated decision | T2 AI-page-build does NOT make decisions in legal-effect sense at default; HR/legal/medical overlay REFUSED pending ADR |

## 7. Sub-processor engagement

Per `legal/sub-processors.md` (managed by council-privacy):
- Let's Encrypt — ACME cert issuance.
- Cloudflare (or alternative) — CDN edge delivery; pack-eu uses EU
  edges only; signed DPA on file.
- LLM provider (when T2 engaged) — model inference per tenant's choice;
  tenant-DEK ciphertext only.
- OCI / AWS — underlying cloud per ADR-0117.

## 8. Consultation

- Council-privacy consulted: yes (this DPIA).
- Council-architecture consulted: yes (ADR-SITES-0001..0007).
- Ops-security consulted: yes (threat-model.md).
- Council-product consulted: yes (PRD scope review).
- External DPO advisory engagement: pack-eu (Schrems II review) +
  pack-us-healthcare (HIPAA BAA review).

## 9. Conclusions

The sites µservice's data processing is:
- Necessary for the declared purpose.
- Proportional given the mitigations.
- Conforming with GDPR Art. 25 by-design + by-default.
- Engaging EU AI Act high-risk obligation only via the HR/legal/medical
  overlay route, which is REFUSED until ADR-SITES-XXXX conformity
  assessment lands.

Approved for production deployment subject to:
- All Cedar policies green.
- All LEAN checks green.
- WCAG 2.2 AA correctness lane refusing publish at < 100% for
  pack-us-healthcare patient portals.
- HG-SITES claim accepted at p99 SLOs.

## 10. References

- GDPR Regulation (EU) 2016/679 — Arts. 5, 6, 9, 13, 14, 17, 22, 25,
  28, 30, 32, 33, 35, 44–50.
- EDPB Guidelines 4/2019 on Art. 25.
- EU AI Act Regulation (EU) 2024/1689 — Annex III §3; Arts. 14, 50.
- EU DSA Regulation (EU) 2022/2065 — Arts. 14, 27.
- ePrivacy Directive 2002/58/EC — Art. 5(3).
- ADA Title III + Section 508 + WCAG 2.2.
- KR PIPA + ISMS-P + 전자문서법.
- HIPAA 45 CFR §164.
- APPI, PDPA, APP, DPDPA, LGPD, UAE PDPL, KSA PDPL.
- ADR-0028, ADR-0117, ADR-0140, ADR-SITES-0001, ADR-SITES-0003,
  ADR-SITES-0004, ADR-SITES-0006.
- `threat-model.md`, `policy/*`, `legal/sub-processors.md`,
  `legal/baa-template.md`, `legal/tia-template.md`,
  `legal/dpa-template.md`.
