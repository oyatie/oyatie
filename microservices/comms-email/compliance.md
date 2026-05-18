# Compliance — `comms-email` µservice

> Authored: 2026-05-18
> ADR anchors: ADR-0201, ADR-0144, ADR-0145.
> Frameworks: CAN-SPAM (US), GDPR (EU), CCPA (CA), HIPAA
> (US-healthcare pack), KR PIPA, KSA / UAE sovereign DP laws.

## 1. CAN-SPAM (US Federal)

| Requirement | Substrate response |
| ----------- | ------------------ |
| Truthful `From` header | Per-tenant DKIM binding; kernel preflight rejects from-domain mismatch. |
| Accurate `Subject` line | Tenant-supplied; substrate audits but does not enforce content (tenant owns). |
| Identify as advertisement | Tenant-side responsibility; template registry tag flags promotional templates. |
| Postal address in message | CAN-SPAM footer template macro appends tenant's mailing address. |
| Honor unsubscribe within 10 business days | Suppression list inserts `OperatorManual` on opt-out click within seconds; audit chain records inserts. |

## 2. GDPR (EU)

### Lawful basis (Art. 6)

- Each send carries `lawful_basis` ∈ {`consent`,
  `legitimate_interest`, `contract`, `legal_obligation`, `vital`,
  `public_task`} in the audit chain entry.
- Tenants set the default per template; per-send overrides
  honored.

### Consent (Art. 7)

- Consent capture is upstream (tenant's product); the substrate
  records the consent identifier with each send.

### Right to erasure (Art. 17)

- Erasure request inserts the recipient into suppression with
  `reason = GdprErasure`.
- All future sends rejected at preflight.
- Historical audit chain entries retain the address per ADR-0145
  tamper-evident chain (Art. 17 §3(b) — public archive exception
  for accountability).

### Right of access (Art. 15)

- Audit chain query returns all events for a recipient within
  the legal SLA.

### Data residency (Art. 44-50)

- IP-013 multi-region routing pins EU tenants to EU-region
  adapters.

### DPIA

- See `dpia.md`.

## 3. CCPA (California)

| Requirement | Substrate response |
| ----------- | ------------------ |
| Right to opt out | Suppression list with `OperatorManual` or `RegulatoryOptOut`. |
| Right to know | Audit chain query. |
| Right to delete | Maps to GDPR Art. 17 path. |
| Sale of personal info | Transactional email is not a sale; explicit policy. |

## 4. HIPAA (US-healthcare pack)

- BAA-only providers — SES with BAA OR Postal (self-hosted).
  Mailgun BAA-only configurations allowed when the BAA is on
  file.
- PHI in email body: the template registry flags any template
  in the `us-healthcare` pack as PHI-class; per ADR-0184 storage
  tier, attachments are encrypted-at-rest with per-tenant KMS
  keys.
- Audit chain entries for HIPAA-class sends carry
  `data_class = PHI` per ADR-0144.

## 5. KR PIPA

- Korean tenants pin to KR-region Postal (no KR-region SES as
  of 2026-05-18).
- Audit chain region tag enforced.
- Korean-language template overlays per ADR-0064 pack `kr`.

## 6. KSA / UAE sovereign DP laws

- Sovereign packs force Postal-only (IP-014).
- All audit chain entries land in the sovereign region.
- No cross-region routing.

## 7. SOC 2

| Trust criterion | Substrate response |
| --------------- | ------------------ |
| Security | DKIM mandatory, TLS-only transport, OpenBao secrets. |
| Availability | Multi-region routing + provider second-source. |
| Processing integrity | Idempotency-key + suppression list + audit chain. |
| Confidentiality | Row-level security on suppression + per-tenant credentials. |
| Privacy | Data residency + GDPR Art. 17 erasure path. |

## 8. PCI-DSS

- Substrate does not handle cardholder data. Templates that
  appear to include PAN-shaped strings are flagged at template
  CI lint and require an explicit waiver.

## 9. Audit cadence

- Quarterly internal compliance review.
- Annual external audit per SOC 2 / GDPR record-of-processing.

## 10. Open obligations

- Inbound email ingestion ADR (deferred) will need its own
  CAN-SPAM + GDPR posture.
- BIMI ADR will document BIMI's compliance posture.
- Phase-2 in-house relay (IP-015) needs a fresh compliance pass
  before launch.
