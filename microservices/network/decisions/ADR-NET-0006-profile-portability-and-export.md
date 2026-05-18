---
id: ADR-NET-0006
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, council-privacy, axis-network, gtm-customer-success, ops-compliance
owner: axis-network + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-NET-0001
  - ADR-NET-0005
related_artifacts:
  - microservices/network/PRD.md (FR-19)
  - microservices/network/runbooks/profile-export-vcard-corruption.md
  - microservices/network/policy/data-residency.md (§DSR Art. 20)
  - microservices/network/sdk-plan.md (§"Profile-Export Helpers")
purpose: Establish the profile-portability + export standard for `network` — vCard 4.0 (RFC 6350) + JSON Resume open schema + GDPR Art. 20 portable-JSON bundle — including per-pack redaction overlays + DSR cascade alignment + cryptographic integrity (Ed25519 export signature) for B2B receiver trust.
---

# ADR-NET-0006: Profile portability + export — vCard 4.0 + JSON Resume + GDPR Art. 20 portable JSON; per-pack redaction overlay; Ed25519-signed export for B2B receiver trust

## Status

Accepted — 2026-05-17.

## Context

Professional-network users must be able to export their profile in interoperable formats:

1. **GDPR Art. 20 (right to data portability)**: user is entitled to "receive the personal data concerning him or her, which he or she has provided to a controller, in a structured, commonly used and machine-readable format and have the right to transmit those data to another controller". Aligned with DPDPA 2023 (India) Art. 13 portability + EU Pay Transparency Directive 2023/970 + KR PIPA + AU Privacy Act 1988 portability rights.
2. **Industry-standard formats**:
   - **vCard 4.0 (RFC 6350)** — universal contact-card format; readable by Apple Contacts, Google Contacts, Outlook, every CRM.
   - **JSON Resume** (open schema, `jsonresume.org/schema/`) — open-source community schema for résumés; broadly adopted in developer tooling + community resume builders.
   - **hCard microformats** (HTML embedded) — historical fallback; readable by older indexing systems.
3. **B2B receiver trust**: when a recipient ATS or HRIS receives an exported profile, they want to know whether the data is integrity-bound (e.g., endorsements verified, recommendations attributed). For this, oyatie ships an Ed25519-signed export bundle (signed by the tenant's exporter key) for the GDPR Art. 20 format.
4. **Per-pack redaction**: pack-us-healthcare PHI redactor + pack-eu Pay-Transparency aggregate-only + minor-account never-exported.
5. **Cryptographic integrity**: endorsement chain (per ADR-NET-0005) cryptographic record must be preserved in the export bundle.
6. **DSR cascade alignment**: export feeds into GDPR Art. 17 (erasure) cascade; user wants to download before requesting erasure.

## Decision

oyatie network's profile-export surface (per `paths: /profiles/me/export` in OpenAPI) emits three formats on demand:

### 1. vCard 4.0 (RFC 6350) — `format=vcard4`

- MIME: `text/vcard; charset=utf-8`.
- Encodes: handle, display-name, headline, summary (first 500 chars), location, current role, certifications (as `URL` entries), avatar (as `PHOTO`), languages.
- Does NOT encode: connection-graph (out-of-scope for vCard), endorsements (out-of-scope), salary (privacy), minor-protect (excluded).
- Pack overlay: pack-us-healthcare PHI redactor strips sensitive fields; pack-eu Pay Transparency aggregate-only.

### 2. JSON Resume (open schema) — `format=jsonresume`

- MIME: `application/json`.
- Encodes: full resume — experience, education, skills, certifications, languages, interests, references (where user-consented), volunteer, awards, projects, publications.
- Does NOT encode: connection-graph, salary detail, minor-protect.
- Pack overlay: per pack-overlay redactor; PHI stripped in pack-us-healthcare; pay-band aggregate-only in pack-eu.

### 3. GDPR Art. 20 Portable JSON Bundle — `format=gdpr-art20`

- MIME: `application/json`.
- Encodes: comprehensive bundle:
  - Profile (vCard 4.0 + JSON Resume embedded).
  - Connection-graph references (counts + 1st-degree ULID list; not full graph traversal).
  - Endorsement references (endorsement_id list with per-endorsement Merkle position + sealed Merkle root; cryptographic chain preserved).
  - Recommendation list (recommendation_id list + bodies if user consented).
  - Post list (post_id list; bodies if user consented).
  - InMail thread metadata (no bodies unless user explicitly opts to include).
  - Page admin list + group memberships + event RSVPs.
  - Signed-URL list for media + document attachments (TTL 7d).
- Bundle is Ed25519-signed by the tenant's exporter key (separate from per-endorser keys); receiver can verify integrity.
- Pack overlay: pack-us-healthcare PHI redactor; pack-eu Pay Transparency aggregate-only; minor-account never exported.
- DSR cascade compatibility: export emit emits `ProfileExportEmitted` audit event; subsequent erasure cascade (Art. 17) consults this audit trail for completeness.

### Per-Pack Redaction Overlay

- **pack-us-healthcare**: PHI redactor strips health-context profile fields (medical specialty, hospital affiliation if attested-only, etc.); profile-export tagged `phi_redacted: true`.
- **pack-eu**: Pay Transparency Directive aggregate-only — salary band omitted; aggregate band metadata appended.
- **pack-kr**: KR PIPA Art. 22-2 sensitive consent — political-affiliation, religion, sexual orientation never exported.
- **pack-jp**: APPI sensitive personal information stripped.
- **all packs**: minor-account export is allowed BUT only by the minor's legal-guardian-attested principal (Cedar `tenant-scope.cedar` FORBID minor-protect on direct export by minor); per-pack age-gate.

### Export Signing

- Per-tenant exporter Ed25519 keypair stored in KMS (separate from per-endorser keys per ADR-NET-0005).
- Bundle signature: Ed25519 over `SHA-256(canonical-JSON(bundle))`.
- Receiver SDK helper: `verifyExport(bundle, tenant_id) -> Result<(), VerifyError>`.

### Audit-Chain Seal

Every export emit produces:
- `oya.network.profile.v1.export-emitted` event (per `contracts/asyncapi/network-events.yaml`).
- Audit-chain seal binding the bundle hash to the user_ref + format + emitted_at.

## Alternatives Considered

### A. Single JSON-only export (no vCard, no JSON Resume)

- Pros: simpler; one format to maintain.
- Cons: tenants cannot import into Apple Contacts / Google Contacts (vCard ubiquity); developer community expects JSON Resume; loss of B2B interoperability.
- Rejected.

### B. vCard 4.0 only (no JSON Resume + no Art. 20 bundle)

- Pros: simpler emitter.
- Cons: vCard 4.0 cannot encode full resume; loss of structured experience + education + skills; insufficient for GDPR Art. 20 portability obligation.
- Rejected.

### C. No Ed25519 signing of the bundle (receiver trusts oyatie's runtime)

- Pros: simpler.
- Cons: receiver ATS / HRIS / regulator cannot independently verify bundle integrity; bundle could be tampered with in transit; Hyperscaler-grade conformance differentiator lost.
- Rejected: signing aligns with ADR-NET-0005 endorsement-chain cryptographic integrity model.

### D. Include full connection-graph traversal in the Art. 20 bundle

- Pros: receiver has full network map.
- Cons: connection-graph contains other users' profile references; exporting another person's PII without their consent violates GDPR Art. 20 footnote (user can only export "data concerning him or her"); loose interpretation could enable network-graph scraping.
- Rejected: 1st-degree connection ULID list only (counts + IDs); full traversal forbidden.

### E. Pack-overlay applied at import-receiver-side (not at export emit)

- Pros: receiver controls what they import.
- Cons: data already exfiltrated by the time receiver applies redactor; receiver may not respect pack policy; PHI leakage risk.
- Rejected: redaction must happen at export emit per pack overlay.

### F. Allow third-party signing (user-controlled signing key)

- Pros: user has cryptographic ownership.
- Cons: user-controlled signing-key management is hard; key loss = profile-export integrity loss; UX friction; doesn't match user expectation; cf. ADR-NET-0005 rejection of client-side endorsement signing.
- Rejected for P01; revisit at M05-onward when FIDO2 / Passkeys mature.

## Consequences

### Positive

- Full GDPR Art. 20 portability obligation honoured.
- Three industry-standard formats (vCard 4.0 + JSON Resume + GDPR Art. 20 JSON) cover all consumer + B2B use cases.
- Ed25519-signed Art. 20 bundle supports B2B receiver trust + ATS / HRIS interop.
- Endorsement-chain cryptographic record preserved in export.
- Per-pack redaction overlay honoured at emit time.
- DSR cascade alignment with Art. 17 erasure.
- Differentiator: no LinkedIn / Xing / Indeed / Wantedly competitor ships vCard 4.0 + JSON Resume + signed Art. 20 bundle natively.

### Negative

- Three emitters to maintain (vCard 4.0, JSON Resume, GDPR Art. 20); higher maintenance cost.
- Per-pack redaction discipline ongoing; redactor regex maintenance.
- FM-23 (vCard / JSON Resume / Art. 20 corruption) is a Sev-2 by regulatory class.
- Ed25519 signing adds ~5ms latency per export; trivial.

### Operational

- Cargo workspace: `oya-network-professional-profile-{sdk,rest,app}` includes the emitter; per-format adapter crates if size grows.
- SDK helpers: `exportProfileVCard()`, `exportProfileJsonResume()`, `exportProfileGdprArt20()` per `sdk-plan.md`.
- Audit event: `oya.network.profile.v1.export-emitted` per `contracts/asyncapi/network-events.yaml`.
- LEAN lane: `oya-gate validate profile-export-schema-conformance` validates vCard 4.0 + JSON Resume schema-conformance on every PR.
- Runbook: `profile-export-vcard-corruption.md` (FM-23).
- Drill: quarterly profile-export drill per `runbooks/profile-export-vcard-corruption.md`.
- Cost: KMS sign + S3 emission; ~$0.01 per export at XS tier.

### Regulatory

- **GDPR Art. 20**: full conformance.
- **DPDPA 2023 (India)**: data portability obligation honoured.
- **EU Pay Transparency Directive 2023/970**: aggregate-only salary band in export.
- **KR PIPA Arts. 22-2, 23, 28**: sensitive-data redaction; per-pack overlay.
- **APPI (Japan)**: sensitive personal information stripped.
- **CCPA + CPRA (California)**: portability + DSR cascade alignment.
- **HIPAA §164.524** (right to access): export of health-context profile honoured via Art. 20 bundle when health-context tenant active; PHI redaction at emit.

## References

- ADR-0135 (Connect dissolution, parallel).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-NET-0001 (storage; export reads from canonical PG).
- ADR-NET-0005 (endorsement-chain integrity; signed records preserved in export).
- `microservices/network/runbooks/profile-export-vcard-corruption.md`.
- `microservices/network/policy/data-residency.md` §DSR Art. 20.
- `microservices/network/sdk-plan.md` §"Profile-Export Helpers".
- `microservices/network/contracts/asyncapi/network-events.yaml` (ProfileExportEmitted).
- RFC 6350 (vCard 4.0).
- JSON Resume open schema `jsonresume.org/schema/`.
- hCard microformats (HTML; historical).
- RFC 8032 (Ed25519).
- GDPR Art. 20; DPDPA 2023 Art. 13; EU Pay Transparency Directive 2023/970; HIPAA §164.524.
- KR PIPA Arts. 22-2, 23, 28; APPI Art. 22-2.
- ISO 30414:2018 §4.3 (workforce demographics aggregate-only).
