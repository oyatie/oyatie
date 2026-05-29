---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: sites
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-sites + council-privacy
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-SITES-0001, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0006]
doc_status: published
---

# Threat Model — sites µservice

## Purpose

Catalogue STRIDE + LINDDUN threats to the sites µservice and bind each
threat to a concrete mitigation, owning artifact, and verification
lane. This document is the security baseline for HG-SITES.

## Scope

In-scope: site / page / block / theme / navigation / url-routing /
domain-binding / seo / cms-collection / search / cdn-delivery BCs;
their adapters; their REST + Workflow event surfaces; their persisted
state (Postgres + Valkey + S3 + Meilisearch + Loro CRDT log).

Out-of-scope: substrate µservices (`tenancy`, `audit-chain`, `ontology`,
`workflow-engine`, `observability`); they carry their own threat models.

## Trust boundaries

| Boundary | Endpoint pair | Direction | Trust posture |
|---|---|---|---|
| Public internet → CDN edge | (anon) → cdn-delivery-rest | inbound | UNTRUSTED — rate-limit + WAF + Cedar `public-read.cedar` |
| CDN edge → origin | cdn-delivery-edge → cdn-delivery-rest | inbound | semi-trusted; mTLS + signed origin headers |
| Editor browser → REST | tenant-editor → page-rest | inbound | OIDC + per-tenant API key + Cedar `tenant-scope.cedar` |
| ACME server → domain-binding | Let's Encrypt → domain-binding-rest (HTTP-01) | inbound | bounded — DNS-01 preferred to keep this edge minimal |
| Custom-domain DNS → DnsVerifier | tenant-DNS-provider → domain-binding-worker | outbound | UNTRUSTED — verify only via signed TXT records |
| Sites worker → Meilisearch | search-worker → meilisearch | east-west | mTLS + per-tenant index key |
| Sites worker → S3 | cdn-delivery-worker → S3 | east-west | mTLS + per-tenant bucket prefix + IAM scoped |
| Sites adapter → Loro CRDT relay | block-adapter-loro → crdt-relay | east-west | mTLS + per-tenant session token |
| Sites → forms µservice | block (form-block) → forms-rest | east-west | Workflow + Ontology ports; no direct import |
| AI-page-build → LLM provider | page-usecase → foundry-runtime → LLM | east-west | tenant-DEK-wrapped prompt; refuse cross-tenant |

## STRIDE matrix

### S — Spoofing

| Threat | Affected BC | Mitigation | Owning artifact | Verification lane |
|---|---|---|---|---|
| Attacker spoofs custom-domain ownership to bind another tenant's domain | domain-binding | DNS-01 TXT challenge requires control of `_acme-challenge.<domain>` TXT record; verified before cert issuance | `iac/helm/templates/deployment.yaml` (domain-binding-worker); ADR-SITES-0004 | E2E `tests/e2e/acme-dns01-spoof-refuse.rs` |
| Attacker spoofs editor identity to publish pages | site, page | OIDC + MFA + per-tenant API key + Cedar `tenant-scope.cedar` per-tenant isolation | `policy/tenant-scope.cedar` | `oya gate validate cedar-enforcement --microservice sites` |
| Attacker spoofs CDN-purge signed event to invalidate cache pre-emptively | cdn-delivery | Signed purge with Ed25519 + per-tenant scope; refused if signature invalid | `policy/tenant-scope.cedar` + ADR-SITES-0003 | `oya gate validate signed-purge --microservice sites` |
| Attacker spoofs ACME challenge response to issue a cert for another tenant's domain | domain-binding | DNS-01 + ALPN-01 (preferred); HTTP-01 only on tenant-controlled root | ADR-SITES-0004 | unit test `tests/acme_spoof.rs` |
| Attacker spoofs Loro CRDT op message to inject malicious blocks | block | Per-tenant session token + Cedar admission; CRDT operation log signed | `policy/editor-isolation.md` + ADR-SITES-0001 | `cargo nextest -p oya-sites-block-adapter-loro -- crdt_spoof_refuse` |

### T — Tampering

| Threat | Affected BC | Mitigation |
|---|---|---|
| Tamper with published HTML at rest (S3 object) | cdn-delivery | S3 object-lock + WORM + Ed25519 content-hash bound in audit-chain seal; LEAN `oya-check-published-artifact-integrity` validates per page-render |
| Tamper with sitemap.xml to inject foreign URLs | seo | sitemap.xml emitted at publish-time only; LEAN `oya-check-sitemap-tenant-scope` refuses cross-tenant URLs in sitemap |
| Tamper with robots.txt to allow unintended crawling | seo | robots.txt emitted from tenant config; tenant-config writes audit-chained |
| Tamper with redirect map to hijack URL routing (301 → attacker's URL) | url-routing | Redirects audit-chained + Cedar-gated; LEAN `oya-check-redirect-scope` refuses redirect to non-tenant host |
| Tamper with CMS-collection schema to inject hostile fields | cms-collection | Schema versions audit-chained; field-definition Ed25519-sealed; rollback path |
| Tamper with theme CSS to inject hostile selectors (CSS-injection) | theme | CSS-in-rust scoped variants + LightningCSS validator; refuses raw `<style>` injection |
| Tamper with image-pipeline output to inject SVG XSS | cdn-delivery (image-block) | libvips strips embedded scripts from SVG; SVGs sanitised before serve |

### R — Repudiation

| Threat | Affected BC | Mitigation |
|---|---|---|
| Editor denies publishing a page | page | Audit-chain seal on `PagePublished` event with author user_id + Ed25519 sig |
| Tenant denies domain-binding action | domain-binding | Audit-chain seal on `DomainBound` event |
| Tenant denies cert renewal failure | domain-binding | ACME log emits to audit-chain on each renewal attempt; per-cert provenance |
| AI-page-build accepted page denies AI authorship | page (AI-page-build) | `AiPageBuildAccepted` audit event carries: model_id, model_version, prompt_hash, output_hash, user_accept_user_id, EU-AI-Act-flags |
| Compliance officer denies legal-hold action | page | `LegalHoldApplied` Ed25519-sealed; 2-person rule required |

### I — Information disclosure

| Threat | Affected BC | Mitigation |
|---|---|---|
| Non-public page content leaks to anonymous reader | page, cdn-delivery | Cedar `public-read.cedar` default-deny; only `visibility=public + tenant_opt_in=true` admits; LEAN drift check |
| Tenant-A's draft page leaks to tenant-B | page | Postgres per-tenant RLS + tenant-DEK + Cedar |
| Tenant-DEK leaks via debug log | (cross-cutting) | LEAN `oya-check-dek-log-redaction` refuses log lines that match DEK serialisation patterns |
| AI-page-build prompt leaks tenant data into model provider | page (AI-page-build) | Tenant-DEK wraps prompt; provider sees ciphertext only via foundry-runtime private-inference channel; ADR-SITES-0006 |
| AI-page-build model leaks tenant-A content into tenant-B generation | page (AI-page-build) | Per-tenant model fine-tunes prohibited; cross-tenant training structurally forbidden by foundry-runtime; ADR-SITES-0006 |
| CDN edge logs reveal anonymous reader IP across packs | cdn-delivery | Per-pack edge nodes only; IP-redacted access logs per ePrivacy Art. 5(3) |
| Subdomain takeover via stale DNS pointing to released wildcard cert | domain-binding | Cert revocation cascade on domain-unbind; ADR-SITES-0004 |
| Search index leak: tenant-A entry appears in tenant-B query | search | Per-tenant Meilisearch index; LEAN `oya-check-search-index-tenant-scope` |
| Block (Loro CRDT log) leaks across tenant via replay | block | Per-tenant session token + per-tenant CRDT log namespace; ADR-SITES-0001 |
| sitemap.xml leaks private intranet page URLs | seo | sitemap entries gated by `visibility=public` flag; private pages never in sitemap |
| Analytics raw IP / fingerprint leaks (ePrivacy violation) | analytics (cross-cutting) | First-party Plausible-class — no third-party cookies; salt-rotation; IP hash bucketing |

### D — Denial of service

| Threat | Affected BC | Mitigation |
|---|---|---|
| Attacker floods anonymous page reads to exhaust origin | cdn-delivery | CDN edge cache + per-IP rate-limit (Cedar context) + WAF |
| Attacker spams ACME challenges to hit Let's Encrypt rate limit (50 certs/wk/account) | domain-binding | Multi-account pool; per-tenant cert-renew throttle; failover to staging directory; ADR-SITES-0004 |
| Attacker uploads enormous images to OOM libvips | cdn-delivery (image-pipeline) | libvips streaming with per-job memory cap; reject > 100MB source / > 50MP resolution; ADR-SITES-0007 |
| Attacker submits expensive search queries (long regex) | search | Meilisearch query-time cap; per-tenant QPS limit |
| Attacker triggers AI-page-build floods to consume LLM budget | page (AI-page-build) | Per-tenant T2 daily cap; foundry-runtime budget enforcement; ADR-SITES-0006 |
| Attacker creates pathological CRDT op chain to slow merge | block | Per-op bound on Loro op size + per-session bound on ops/sec |
| Publish a 100k-page site at once to overload publish-pipeline | cdn-delivery | Per-tenant concurrent-publish cap; queue back-pressure; runbook `publish-pipeline-rollback.md` |
| Redirect-loop attack via 301 chains | url-routing | LEAN `oya-check-redirect-loop` refuses redirect chains > 5; runtime detection emits 508 |

### E — Elevation of privilege

| Threat | Affected BC | Mitigation |
|---|---|---|
| Editor escalates to delete another tenant's site | site | Cedar `tenant-scope.cedar` + Postgres RLS — refused at DB layer |
| Editor escalates to revoke cert for another tenant's domain | domain-binding | Cedar admission + 2-person rule for cert revoke; audit-chained |
| Editor escalates to override WCAG-AA refusal at publish | page | Override flag carries audit-chain seal + tenant DPA disclosure; LEAN `oya-check-wcag-override-attribution` requires explicit attribution |
| Editor escalates to publish to a domain not bound to their tenant | domain-binding + page | Publish pipeline rejects domain not in `Domain{tenant_id == publishing_tenant_id}` set |
| AI-page-build T2 escalates to T3 (auto-publish without user review) | page (AI-page-build) | Cedar refuses T3 for sites; T2 reversibility window 30s; ADR-SITES-0006 |
| Editor escalates to read CMS-collection in another tenant's site | cms-collection | Postgres RLS + Cedar; LEAN check |

## LINDDUN matrix (privacy)

### L — Linking

| Threat | Mitigation |
|---|---|
| Anonymous visitor linkable across pages on same site | First-party analytics with salt-rotated session id; no third-party cookies |
| Editor activity linkable across tenants | Per-tenant audit log isolation; cross-tenant join requires explicit Cedar admit |

### I — Identifying

| Threat | Mitigation |
|---|---|
| Page-render leaks editor identity in HTML comments | LEAN `oya-check-no-editor-id-in-rendered-output` |
| Image EXIF reveals editor location | libvips strips EXIF on publish per ADR-SITES-0007 |
| Analytics records visitor IP / fingerprint | IP hash-bucketed; salt rotated daily; no fingerprint |

### N — Non-repudiation (privacy adverse)

| Threat | Mitigation |
|---|---|
| Audit-chain records permanent attribution exceeding tenant's retention need | Per-tenant audit retention configurable to legal floor; legal hold extends only on demand |

### D — Detecting

| Threat | Mitigation |
|---|---|
| Editor's draft activity detected via timing side-channel on draft URL | Preview-mode draft URLs bear signed-token + random component; no enumeration |

### D — Disclosure

| Threat | Mitigation |
|---|---|
| Sub-processor (CDN edge provider) discloses traffic logs to LE | Per ePrivacy + GDPR Art. 28: signed DPA; logs IP-hashed at edge; sub-processor list per `legal/sub-processors.md` |

### U — Unawareness

| Threat | Mitigation |
|---|---|
| Visitor unaware analytics tracks them | Tenant must show cookie/consent banner per ePrivacy Art. 5(3); LEAN `oya-check-consent-banner-presence` for pack-eu tenants |
| Editor unaware AI-page-build sent prompt to LLM provider | UI label per EU AI Act Art. 50 + Art. 14 |

### N — Non-compliance

| Threat | Mitigation |
|---|---|
| GDPR Art. 17 erasure not honoured for site authorship records | Erasure orchestrator in page-usecase + legal-hold reconciliation |
| EU DSA Arts. 14/27 transparency for moderation refusal not surfaced | Publish-refusal records include policy-citation; tenant can serve transparency-report sitemap |
| WCAG 2.2 AA correctness not honoured on patient-portal sites | pack-us-healthcare overlay refuses publish at < 100% WCAG correctness; HIPAA + ADA Title III aligned |

## Per-pack threat overlays

### pack-kr (KR PIPA + 전자문서법 + ISMS-P)

- Sensitive personal info (KR PIPA Art. 23) flagged via data-class `SENSITIVE_PIPA_ART23`; refuse to render such fields anonymously.
- 전자문서법 Art. 5 integrity: published pages bear audit-chain seal.
- ISMS-P §2.10 (communications security): mesh mTLS + WAF + signed CDN purge.

### pack-eu (GDPR + EU AI Act + EU DSA + ePrivacy)

- GDPR Art. 17 erasure cascade in page-usecase.
- EU AI Act Art. 50 transparency labels on T2 AI-page-build.
- EU DSA Art. 14 transparency: publish-refusal carries policy citation.
- ePrivacy Art. 5(3): consent banner LEAN check.
- eIDAS Art. 26 AdES: audit-chain Ed25519 satisfies for signed-Sites scenarios.

### pack-us-healthcare (HIPAA + ADA Title III)

- Patient-portal sites carry PHI surface; data-class `HIPAA_PHI` on relevant CMS-collection fields.
- ADA Title III: WCAG 2.2 AA correctness refuse-publish at < 100% (Section 508 + ADA aligned).
- BAA per `legal/baa-template.md`.

### pack-jp/sg/au/in/br/ae/ksa

Per-overlay sections in `compliance.md`.

## Residual risk acceptance

| Risk | Acceptance | Owner | Review cadence |
|---|---|---|---|
| ACME provider outage during cert renewal | Accept; mitigate via 30d-pre-expiry renewal window + multi-account pool | ops-security | quarterly |
| CDN edge cache poisoning by upstream attack | Accept; mitigate via signed-purge + per-pack edge keys | ops-sre-reliability | quarterly |
| AI-page-build HR/legal/medical-context refusal false-positive | Accept until ADR-SITES-XXXX conformity assessment lands | axis-sites + council-privacy | per pre-launch |
| Loro CRDT pre-1.0 API churn | Accept; mitigate via pinned version + parallel-test with docs/sheets/slides | axis-sites | per Loro release |

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: tenant isolation coverage | per-PR | axis-sites |
| LEAN: signed-purge contract | per-PR | axis-sites |
| LEAN: WCAG override attribution | per-PR | ops-security |
| LEAN: redirect-scope refusal | per-PR | axis-sites |
| Pen-test: subdomain takeover | annually | ops-security (external pen-test firm) |
| ACME spoof E2E | per-PR | axis-sites |
| Cross-tenant search-index isolation | per-PR | axis-sites |
| GDPR Art. 17 erasure E2E | quarterly | council-privacy |

## References

- ADR-0028 (Bominal audit-chain).
- ADR-0117 (data residency).
- ADR-0135 (unbundle).
- ADR-0140 (Cedar policy).
- ADR-SITES-0001 (Loro CRDT); ADR-SITES-0003 (CDN); ADR-SITES-0004
  (ACME); ADR-SITES-0006 (AI-page-build EU AI Act bounds).
- STRIDE — Microsoft Threat Modeling.
- LINDDUN — KU Leuven privacy threat methodology.
- RFC 8555 — ACME.
- GDPR Regulation (EU) 2016/679.
- KR PIPA + 전자문서법 + ISMS-P.
- HIPAA 45 CFR §164.
- EU AI Act Regulation (EU) 2024/1689.
- EU DSA Regulation (EU) 2022/2065.
- ADA Title III + Section 508 + WCAG 2.2.
- ePrivacy Directive 2002/58/EC.
- OWASP ASVS v4.
- W3C Subresource Integrity Recommendation.
