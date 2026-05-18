# consent-graph compliance map

- Owner: axis-consent-graph + compliance-axis
- Date: 2026-05-18
- Authority: ADR-0214 §1, §8.4 NFR, ADR-0064 (canonical-base + pack overlays).

For each regulatory pack supported, this document maps the consent-graph capability to the regulation's
applicable clauses. Pack overlays in `iac/kustomize/overlays/<pack>/` carry per-pack runtime config
(residency, retention, redaction defaults).

## 1. Supported packs

`kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa` (11 packs; matches audit-chain).

## 2. KR (South Korea) — PIPA

### 2.1 Cross-border transfer (PIPA §17, §18)
- Default: cross-border transfer forbidden unless agreement.sovereignty.cross_border_transfer_permitted
  = true *and* lawful basis cited.
- Pack overlay sets `default_cross_border_transfer_permitted=false`.
- Lawful basis (consent / contract necessity / legal obligation) recorded in agreement.terms.metadata.

### 2.2 Right to know + right to erasure (PIPA §35)
- DSAR cascade runbook (`GDPR-DSAR-cross-tenant.md`) applies KR clock (10 days response).
- Erasure cascade tombstones all projections within 7 days.

### 2.3 Sensitive data (PIPA §23)
- Sensitive categories (health, race, ideology) require explicit consent + tighter scope; pack overlay
  enforces k_anonymity≥10 for aggregate of sensitive cohorts.

## 3. EU — GDPR + EAIA + DGA + Data Act

### 3.1 GDPR Art. 28 (processor)
- consent-graph operates as a processor under agreement terms; grantor is controller.
- agreement.terms.metadata captures processing instructions per Art. 28(3).

### 3.2 GDPR Art. 44–49 (cross-border)
- SCCs reference recorded in agreement.terms.metadata for EU→non-EU transfers.
- Schrems II adequacy decisions consulted; adequacy gap auto-blocks transfer.

### 3.3 GDPR Art. 17 (right to erasure)
- Cascade within 30 days (EU regulatory cap); consent-graph targets 7 days.

### 3.4 GDPR Art. 35 (DPIA)
- See `dpia.md` for full DPIA.

### 3.5 EU AI Act (EAIA)
- consent-graph capabilities classified per `capabilities/*.yaml`:
  - `consent-grant` T3 ⇒ EAIA high-risk class.
  - `consent-project-subscribe` T2 ⇒ EAIA limited-risk.
  - `consent-enforce` T0 ⇒ EAIA none (deterministic read-only).
- Pre-deployment Conformity Assessment required for T3.

### 3.6 EU Data Governance Act (DGA)
- Data intermediation neutrality requirement: consent-graph does not monetize data flows; processing
  fees only.

### 3.7 EU Data Act
- Right to data portability + interoperability of IoT data — covered by AttestedQuery + Projection
  modes.

## 4. US — federal + state

### 4.1 CCPA / CPRA (California)
- Right to know + right to delete + right to opt-out — all covered by DSAR cascade.
- "Sale of personal information" — consent-graph operations are NOT sales (no monetization of subject
  data).
- "Sharing" definition (CPRA-specific) — agreement-based sharing is *sharing under CPRA*; opt-out
  mechanism honored.

### 4.2 Other state privacy laws (CO, CT, UT, VA, ...)
- Covered under same DSAR + cross-tenant tombstone semantics.

### 4.3 GLBA (financial)
- Financial-vertical agreements default to AttestedQuery mode (per template `tmpl-banking-*`).
- Customer's right to opt-out of affiliate-information-sharing → revocation primitive.

## 5. US-Healthcare — HIPAA

### 5.1 HIPAA min-necessary (§164.502(b))
- Cedar policy + EntityScope.field_set enforces min-necessary at every projection emission.
- Audit-chain stores fields-redacted list per event.

### 5.2 HIPAA accounting of disclosures (§164.528)
- 6-year retention of bilateral audit chain ⇒ supports accounting-of-disclosures requests.

### 5.3 HIPAA break-glass
- AttestedQuery mode supports purpose-of-use `emergency-treatment` for break-glass with mandatory
  post-hoc audit review.
- Runbook `consent-forgery-detected.md` (alternate use: break-glass review).

### 5.4 HIPAA Business Associate Agreement (BAA)
- consent-graph operates under BAA executed between oyatie + covered-entity grantor.

### 5.5 TEFCA / Direct Trust
- consent-graph's HIE-style bilateral chain aligns with TEFCA participant audit requirements; ADR-
  SVC-CG-* (PHASE-02) will spec the explicit interop mapping.

## 6. JP — APPI

### 6.1 APPI cross-border (§24)
- Cross-border requires same consent or adequacy decision; pack overlay enforces.

### 6.2 APPI right to disclosure / cessation
- Cascade ≤14 days; consent-graph targets 7 days.

## 7. SG — PDPA

### 7.1 PDPA Cross-Border Transfer Limitation Obligation (§26)
- Transfer prohibited unless recipient bound by comparable protection; partner-directory handshake
  records this attestation in `peer_attestation` field.

### 7.2 Do Not Call / Spam Control
- Not consent-graph's domain (Comms-Email µservice handles).

## 8. AU — Privacy Act + APP

- APP 8 (cross-border): pack overlay mandates partner attestation in handshake.
- APP 11 (security): mTLS + audit-chain seals satisfy reasonable-steps test.

## 9. IN — DPDP 2023

- §10 cross-border: government-notified-country list updated quarterly in pack overlay.
- §11 right to grievance: DSAR cascade includes grievance escalation hook.

## 10. BR — LGPD

- Art. 33 cross-border: adequacy decision or controller's safeguards (mirrored to GDPR Art. 46).

## 11. AE / KSA — region-specific

- AE PDPL Art. 19 cross-border: requires written agreement (matches our DataSharingAgreement).
- KSA PDPL: similar; local cloud residency emphasized in pack overlay.

## 12. Audit/inspection readiness

For each pack, the following evidence is queryable:
- All active agreements + scope + terms + lawful basis (Postgres view, RLS-scoped).
- All historical agreement lifecycle events (audit-chain query).
- All projection-emit + projection-read events (audit-chain).
- All revocations + propagation receipts (audit-chain + cross-pointer reconciliation reports).
- All DSAR cascade reports (evidence/ dir).

Audit-chain retention defaults per pack:
- KR/EU/JP/SG/AU/IN/BR: 7 years.
- US-Healthcare (HIPAA): 6 years (relaxed from 7y to match HIPAA spec).
- AE/KSA: 5 years.

## 13. Cross-references

- `dpia.md` for full Data Protection Impact Assessment per GDPR Art. 35.
- `data-residency.md` for per-region storage + processing geography.
- `iac/kustomize/overlays/<pack>/` for per-pack runtime config.
- `microservices/audit-chain/compliance.md` for the audit substrate map.
