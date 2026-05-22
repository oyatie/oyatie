---
doc_class: Runbook
shape: How-to
related_adrs: [ADR-0201, ADR-0263]
companion_docs:
  - microservices/comms-email/IP-017-inbound-receiver-domain.md
---

# Runbook — inbound-receiver quarantine release

## A. Trigger conditions

- Tenant admin reports legitimate email quarantined as phishing.

## B. Pre-checks

1. `oya comms-email inbound message show $MID --tenant=$TID`.
2. Confirm SPF + DKIM + DMARC verification status.
3. Check phishing-classifier reasoning.

## C. Procedure

1. Validate sender legitimacy (DNS, WHOIS).
2. Verify message content not malicious (sandbox review).
3. `oya comms-email inbound quarantine release $MID`. Audit tag:
   `oya.comms-email.inbound-quarantine-released`.
4. Add sender to tenant's allow-list if pattern recurring.
5. Update phishing-classifier with false-positive sample.
6. Notify tenant via in-app.
7. Verify tenant receives message in their inbox.
8. Document false-positive in `evidence/phishing-fp/$MID.md`.

## D. Verification

- Tenant confirms receipt.
- Phishing classifier weights updated (next training cycle).

## E. Rollback

- Re-quarantine via `oya comms-email inbound quarantine apply $MID --reason=...`.

## F. Post-incident

- Quarterly false-positive review.

## G. References

- ADR-0201 — comms-email substrate
- IP-017 — inbound-receiver domain
