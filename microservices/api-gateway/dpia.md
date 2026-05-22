# api-gateway — DPIA (Data Protection Impact Assessment)

**Status:** Accepted
**Authority:** GDPR Article 35 + ADR-0157 + ADR-0244 + ADR-0263 + ADR-0273 + ADR-0276 + ADR-0297 (in flight).
**Hyperscaler precedent:** Cloudflare DPIA template (2024) + Apigee privacy whitepaper (2023).
**Last reviewed:** 2026-05-20.
**DPO sign-off:** required before promotion to production per ADR-0250.

## A — Processing description

The api-gateway is the Tier-0 edge µservice; it processes 100% of inbound HTTP traffic to oyatie. It is the first node where PII/PHI may be observed (in headers, URLs, bodies) and the first where pseudonymous identifiers (IP, JA4 fingerprint, cookie session-id) are computed.

### A-1. Categories of data processed

| Category | Examples | Class | Retention |
|---|---|---|---|
| Source IP | Client IPv4/IPv6 | Pseudonymous | ≤72h raw, then aggregated to /24 + /48 |
| TLS ClientHello fingerprint | JA4 / JA4+ | Pseudonymous | 30d (behavioural baseline) |
| HTTP/2-3 frame fingerprint | Settings + ordering | Pseudonymous | 30d |
| User-Agent string | Header value | Pseudonymous | 30d |
| Authentication cookie | `__Host-session-id` (HMAC) | Pseudonymous (rotatable) | Session lifetime (≤7d) |
| Request URL | `/v1/payments/intents` etc. | May contain PII (rare) | Audit log 90d, otherwise not retained |
| Request body | JSON / multipart / binary | May contain PII / PHI | Not retained at gateway (passed through) |
| Bot-score | 0..100 + model-feature-vector | Pseudonymous derived | 30d (bound to fingerprint) |
| Audit event | Admit / deny / WAF / rate-limit | Pseudonymous + tenant_id | 7y (per ADR-0028 audit-chain doctrine) |

### A-2. Purposes

| Purpose | Lawful basis (GDPR Art. 6) | Special category (Art. 9)? |
|---|---|---|
| Prevent abuse (bot, scrape, DDoS, credential-stuffing) | Legitimate interest (Art. 6(1)(f)) | Not normally; PHI handled by downstream µservice. |
| Authentication handoff | Performance of contract (Art. 6(1)(b)) | n/a at gateway tier |
| TLS termination | Performance of contract (Art. 6(1)(b)) | n/a |
| Audit + compliance | Legal obligation (Art. 6(1)(c)) | If PHI in audit body — explicit consent required. |
| Rate-limit | Legitimate interest (Art. 6(1)(f)) | n/a |
| Routing + load-balance | Performance of contract (Art. 6(1)(b)) | n/a |

### A-3. Data subjects

- B2C end users (consumers of every product µservice).
- B2B tenant operators (admins).
- Partner API consumers.
- Machine clients (other oyatie µservices, vendor integrations).
- Anonymous visitors (no identifiers).

## B — Necessity + proportionality assessment

- **Is the processing necessary?** Yes. Without a Tier-0 edge, oyatie cannot serve traffic, cannot apply abuse-defence, cannot rate-limit, cannot route. The processing is the minimum required to operate an internet-facing platform.
- **Is the processing proportionate?** Yes, with controls listed below. We process the minimum data needed for each purpose.
- **Are there less intrusive alternatives?**
  - For abuse-defence: pseudonymous fingerprinting is less intrusive than mandatory CAPTCHA-on-every-request (degrades accessibility).
  - For rate-limit: per-tenant counters less intrusive than per-IP global counters (no cross-tenant linkability).
  - For audit: deny-event includes principal context HASH not raw PII; reconstruction requires separate authorised lookup against identity µservice.

## C — Risks to data subjects

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| JA4 fingerprint enables cross-session tracking without consent | High | Medium | Rotate fingerprint salt every 24h; bot-score features re-derived per session; per-tenant pseudonymity isolation. |
| IP retention reveals subject geo-location | Medium | Medium | ≤72h raw; aggregated to /24 + /48 after; never logged with PII in same record. |
| Bot-score model misclassifies neurodivergent / older / accessibility-tool user as bot | Medium | High | CAPTCHA-on-suspicion never on default path; per-tenant allow-list; human-rights review of model. |
| Audit-event contains URL with PII (e.g. `/api/users/{email}/profile`) | Low | High | URL canonicaliser strips known-PII-shaped path components into `{redacted}`; emit redaction-pattern audit. |
| ECH-disabled client SNI is observable on-the-wire | Medium | Low | ECH advertised; graceful degradation. Documented in privacy notice. |
| Cross-border transfer (sov-cell routing) | Medium | High | Sov-cell overlay forbids cross-jurisdiction routing for regulated tenants; per-tenant routing tier per ADR-0244. |

## D — Mitigations (binding)

- **Data minimisation:** Gateway holds zero PII at rest (stateless data-plane). Only audit events leave the gateway tier.
- **Pseudonymisation:** Per-tenant salted fingerprint derivation; rotating session-id; redacting URL path tokens.
- **Retention:** ≤72h raw IP; 30d fingerprint history; 90d audit body; 7y audit chain (sealed).
- **Encryption in transit:** TLS 1.3 mandatory; ECH where peer supports; PQC hybrid where peer supports.
- **Encryption at rest:** Audit events encrypted at the audit-chain µservice; gateway has no at-rest storage.
- **Access control:** Cedar gate on every admission decision; SPIFFE mTLS on upstream; OpenBao TLS keys.
- **Right to access (Art. 15):** Audit-export-tool per ADR-0276 returns all gateway audit events for a tenant in JSON-LD.
- **Right to erasure (Art. 17):** Audit events retained per legal obligation (Art. 17(3)(b)) but PII-content fields redacted on erasure request.
- **Right to portability (Art. 20):** Portable backup format per ADR-0276.
- **Right to object (Art. 21):** Per-purpose consent surface per ADR-0272; opt-out endpoint at `https://privacy.oyatie.com/optout`.

## E — Special-category data (Art. 9)

The gateway DOES NOT INTENTIONALLY PROCESS special-category data. If a tenant's API surface includes such data in URLs / headers / bodies, the gateway is a *conduit*; processing happens downstream in the relevant µservice. Audit events at the gateway capture metadata (request envelope) not body content. **If a body MUST be inspected for abuse (rare WAF rule)** — explicit consent flag in request headers required AND scoped to the abuse-defence purpose AND scrubbed after evaluation.

PHI handling: HIPAA-regulated tenants route via `pack-us-healthcare` overlay; the gateway forwards a `X-Oya-Compliance-Pack: us-healthcare` header so downstream applies BAA controls.

## F — Pack overlays

| Pack | Variation |
|---|---|
| pack-us (default) | Defaults above. |
| pack-eu | DPIA registered with lead DPA per Art. 35(11); cross-border routing forbidden except adequacy-decision destinations. |
| pack-kr (PIPA/CSAP) | Resident data → sov-cell-kr; cross-border transfer requires explicit consent per PIPA Art. 28. |
| pack-cn (PIPL-2021) | Resident data → sov-cell-cn; PIPL Art. 38 cross-border assessment registered. |
| pack-us-healthcare (HIPAA) | BAA in place; PHI never logged in audit body; `X-Oya-Compliance-Pack: us-healthcare` forwarded; access controls per HIPAA §164.312. |
| pack-il-5-6 (US DoD) | Forbids public-cloud routing; sov-cell-il5/6 mandatory; FIPS 140-3 crypto only. |
| pack-fedramp-high | FedRAMP High control mapping; CT mandatory; PQC mandatory. |
| pack-ksa-pdpl (Saudi PDPL) | Resident data → sov-cell-ksa; cross-border per PDPL Art. 29. |
| pack-ae-pdpl (UAE PDPL) | Resident data → sov-cell-ae; cross-border per UAE Federal Decree-Law 45 of 2021. |

## G — DPIA conclusion

The processing is **necessary, proportionate, and adequately mitigated** with the controls in §D. The residual risks (§C) are *acceptable* with the ongoing controls in `threat-model.md` §E. Pre-launch, a DPO sign-off is required (per ADR-0250 build-ahead-of-certification).

## H — Consultation

- Lead DPA notification (per Art. 36) required if any residual high risk emerges post-launch.
- Internal: ops-security + axis-network + legal-privacy.
- External: lead DPA (Ireland — for EU lead-supervisory model); KR PIPC; PRC CAC; FedRAMP PMO (when applicable).

## I — Review cadence

- Re-review every 12 months OR on substantive change to:
  - Bot-score model.
  - Cedar fragment changes affecting admission.
  - Cross-border routing rules.
  - PQC / ECH protocol changes.

## J — References

- GDPR Articles 5, 6, 9, 12-22, 35, 36.
- ADR-0157, ADR-0244, ADR-0263, ADR-0272, ADR-0273, ADR-0276, ADR-0297 (in flight).
- `docs/standards/documentation-rigor.md` §3.2.3.
- `microservices/api-gateway/threat-model.md`.
- `microservices/api-gateway/compliance.md`.
- Cloudflare DPIA template 2024.
- WP29 / EDPB Guidelines on DPIA (WP248rev01).
- KR PIPA Art. 28, 33-35.
- PIPL 2021 (PRC) Art. 38-44.
- HIPAA §164.312 technical safeguards.
