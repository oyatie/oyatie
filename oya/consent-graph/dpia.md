# consent-graph Data Protection Impact Assessment (DPIA)

- Owner: axis-consent-graph + privacy-officer (acting)
- Date: 2026-05-18
- Authority: GDPR Art. 35; LGPD Art. 38; equivalent provisions in DPDP 2023, APPI, PIPA.
- Status: Initial DPIA; targets Accepted at PHASE-01 GA.

## 1. Description of processing

### 1.1 Purpose
Enable real-time entity-level data sharing across organizational tenants under bilateral consent,
with cryptographic audit defensibility + revocation + sovereignty controls.

### 1.2 Categories of data subjects
- B2B partner contacts (manufacturer reps, retailer reps, insurer reps, etc.)
- Healthcare patients (US-healthcare pack)
- Banking customers (banking pack)
- End consumers (B2C pack)
- IoT-device-related entities (supply-chain pack)
- Marketplace buyers + sellers

### 1.3 Categories of personal data
- Identifiers (consumer-id, patient-id, merchant-id, etc.)
- Contact information (when explicitly in scope)
- Transactional data (PO state, shipment state, transaction state)
- Health-eligibility data (US-healthcare)
- Financial state (balance, transaction status — banking)
- Behavioral aggregates (cohort stats — marketplace)
- **NEVER**: raw payment-card numbers, full health diagnoses (only eligibility), passwords, biometrics

### 1.4 Recipients
- The grantee tenant only, per agreement.
- audit-chain µservice (sealed entries).
- observability µservice (cardinality-bounded metrics only; never PII).

### 1.5 Cross-border transfers
- Default: forbidden (sovereignty pin in grantor region).
- Opt-in per agreement via `geo_replicate_to_grantee_region=true` + adequacy decision check.
- KR / EU strict-residency packs: hardcoded forbidden.

### 1.6 Retention
- Active agreements: indefinite while Active.
- Revoked/Expired agreements: 7y default (pack-configurable: HIPAA 6y, AE/KSA 5y).
- Projection events: 7d Pulsar retention (then aged into ontology projection cache subject to ontology
  retention).
- Audit-chain entries: per audit-chain retention (default 7y).

## 2. Necessity + proportionality

### 2.1 Necessity test
The capability (cross-tenant real-time visibility) is not achievable without processing personal data
because:
- The objects shared (PO state, shipment, eligibility, etc.) inherently involve identifiers + state.
- Without consent-graph, the alternative is EDI/email/manual report — equally personal-data-touching
  but without audit, revocation, or scope controls. consent-graph is the privacy-superior path.

### 2.2 Proportionality controls
- Field-level scope narrowing: only fields per `EntityScope.field_set` projected.
- Three sharing modes — Aggregate mode preferred where row-level is unnecessary.
- k-anonymity ≥5 for Aggregate; ≥10 for sensitive categories (PIPA §23 alignment).
- DP noise on Aggregate.
- Redaction (mask/hash/null/bucket) per agreement.

## 3. Lawful basis (GDPR Art. 6)

Per-agreement determination, recorded in `agreement.terms.metadata.lawful_basis` ∈ {consent, contract,
legal-obligation, vital-interest, public-task, legitimate-interest}.

- B2B supply-chain: contract necessity (Art. 6(1)(b)).
- B2C consumer-initiated: consent (Art. 6(1)(a)).
- Healthcare eligibility: contract + legal-obligation (Art. 6(1)(b), (c)).
- Banking customer-initiated: consent + contract.
- Sensitive (Art. 9): explicit consent or specific Art. 9(2) clause; recorded in
  `agreement.terms.metadata.art9_clause`.

### 3.1 Consent quality
For consent-based agreements:
- Freely given: data subject can decline; B2C self-revoke always available.
- Specific: scope narrowed to specific entity/fields.
- Informed: data subject UI shows scope plain-text + grantee identity.
- Unambiguous: explicit accept click (no pre-checked).
- Withdrawable: self-revocation primitive (per ADR-SVC-CG-005).

### 3.2 Standing consent vs per-event
Most B2C agreements are standing consent (with revocation). High-sensitivity (banking AttestedQuery
for balance) uses per-event consent via separate UI step.

## 4. Risks to data subjects

| Risk | Likelihood | Severity | Mitigation |
|------|-----------|----------|------------|
| Cross-tenant leak of identity | low | high | scope narrowing + redaction + audit |
| Surveillance via aggregate cohort | medium | medium | k-anon ≥5 + DP noise |
| Revocation not honored | low | high | ≤1s propagation SLO + audit trail |
| Re-identification via aggregate | low | medium | k-anon + DP + per-agreement salt |
| Lost erasure cascade | low | high | DSAR runbook + 7d cap |
| Data sovereignty violation | very low | high | grantor-region pin + sovereignty-violation-zero SLO |
| Consent forgery | very low | high | bilateral chain + mTLS + audit |
| Replay of revoked grant | very low | medium | idempotency + audit |
| Pulsar message in flight intercept | low | medium | mTLS + (PHASE-02) per-message sign |

### 4.1 Special-category data (Art. 9)
- Health data (US-healthcare pack): default mode AttestedQuery (no row-level projection); explicit
  consent + lawful basis recorded.
- Biometrics: NOT IN SCOPE — biometric data is explicitly excluded from consent-graph at the schema
  layer (ontology entity-type `BiometricRecord` is non-shareable; PR review enforces).

## 5. Necessity + proportionality conclusion

The processing is necessary for the stated purpose and proportionate to the risks, given:
- Scope narrowing is mandatory (no all-fields default).
- Audit + revocation + sovereignty are first-class.
- Three modes (Projection / Aggregate / AttestedQuery) match privacy-preserving alternative selection.
- Pack overlays tune per-jurisdiction defaults.

## 6. Consultation

- DPO (acting privacy officer): reviewed.
- Data subjects (B2C): consultation via consumer UI mockup user-test (PHASE-02).
- Supervisory authority pre-consultation (GDPR Art. 36): not triggered (residual risks are not high
  after mitigation).

## 7. Monitoring

- SLO `sovereignty-violation-zero` → 0 budget; any breach is DPIA-revisit trigger.
- DSAR cascade success rate → tracked in evidence/.
- Annual DPIA review cycle; out-of-band review on any P0 audit-chain divergence.

## 8. Records of processing (Art. 30)

For each active agreement, the following constitutes the record:
- Controller: grantor tenant.
- Processor: oyatie + consent-graph + ontology + audit-chain (joint processor).
- Purposes: agreement.terms.purpose_of_use.
- Categories of subjects + data: agreement.scope.entity_type + field_set.
- Recipients: agreement.grantee.
- Transfers: agreement.sovereignty.
- Retention: per pack.
- Security measures: this DPIA + threat-model.md + service ADRs.

Queryable via `GET /v1/agreements?` for compliance officer.

## 9. Risk treatment plan

| Risk | Treatment | Target |
|------|-----------|--------|
| Cross-tenant leak | Field-level scope + audit | P0 SLO sovereignty-violation = 0 |
| Re-identification | k-anon ≥5 + DP | DPIA re-review at any breach |
| Revocation lag | ≤1s propagation | SLO + page on burn |
| Consent forgery | bilateral chain | P0 audit-divergence reconciliation |

## 10. Sign-off

- DPO (acting): pending Active phase.
- Engineering owner: axis-consent-graph.
- Compliance reviewer: pending pack overlay review per jurisdiction.

## 11. Cross-references

- `threat-model.md` — security adversary model.
- `compliance.md` — per-pack regulatory map.
- `data-residency.md` — geographic + sovereignty model.
- `runbooks/GDPR-DSAR-cross-tenant.md` — DSAR cascade procedure.
- `runbooks/data-residency-enforcement.md` — sovereignty enforcement procedure.
