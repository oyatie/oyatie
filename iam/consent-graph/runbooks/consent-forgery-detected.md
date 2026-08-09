# Runbook: consent-forgery-detected

- Severity: P0 (security incident; potential regulatory disclosure)
- Trigger conditions:
  - IP-013 reconciler reports HmacMismatch or OrphanCrossPointer.
  - Anomaly detection flags grants with unusual actor patterns.
  - Audit-officer flags suspicious agreement during review.

## Step 1 — Acknowledge + isolate (≤5min)

1. Page incident commander + privacy officer + security on-call.
2. Open private incident channel `#inc-cg-forgery-<id>`.
3. Auto-suspend suspect agreement (already automatic for HmacMismatch; manual otherwise):
   `oya consent-graph agreement suspend <agreement-id> --reason SecurityIncident`.
4. Notify affected tenants (both grantor + grantee) within 1h via designated security contact.

## Step 2 — Forensic snapshot (≤30min)

1. Freeze audit-chain entries for both parties:
   `oya audit-chain freeze --chain-id <grantor-chain> --window 30d`.
   `oya audit-chain freeze --chain-id <grantee-chain> --window 30d`.
2. Snapshot Postgres tables for affected agreement_id:
   `pg_dump --table consent_graph_agreements --where "agreement_id='<id>'" > snap-<id>.sql`.
   Repeat for `consent_graph_cross_pointers`, `consent_graph_revocations`.
3. Export Pulsar topic backlog for revocation + audit-bridge topics involving the pair (last 7d).
4. Record OpenBao key version for the pair-HMAC key (do NOT rotate yet — pre-rotation key needed for
   forensic verification).
5. Generate forensic report:
   `oya consent-graph forensic-snapshot --agreement <id> --output evidence/forensic-<id>.json`.
   Sealed in audit-chain.

## Step 3 — Investigate (24h-48h)

Determine root cause:

| Symptom | Likely cause | Next step |
|---------|--------------|-----------|
| HmacMismatch on one event | corrupted database row | replay from outbox; tamper-test surrounding rows |
| HmacMismatch on multiple events from one pair | per-pair HMAC key compromise OR systematic tampering | rotate pair-HMAC key; quarantine pair |
| OrphanCrossPointer | bilateral emission rolled back; cross-pointer table not cleaned | clean orphan; check audit-bridge worker for crash |
| Grantor entry exists, no grantee entry, no cross-pointer | grantee chain rollback or grantee-side tamper | contact grantee CISO; cross-verify on grantee side |
| All entries valid but unusual actor | possibly social-engineering forged consent | freeze pair; legal review |

## Step 4 — Decision matrix (≤72h)

| Outcome | Action |
|---------|--------|
| Genuine bug, not tampering | code fix; replay; re-pair; unblock pair; post-mortem |
| Tampering by oyatie insider | terminate insider; rotate compromised keys; mandatory disclosure |
| Tampering by partner insider | offboard partner; rotate per-pair HMAC; regulatory disclosure to affected jurisdictions |
| External breach | full breach response per security-incident playbook; regulator + customer notifications |

## Step 5 — Regulatory disclosure

- GDPR Art. 33 (data breach): supervisory authority within 72h if breach risk to data subjects.
- GDPR Art. 34: notify affected data subjects if high risk.
- HIPAA Breach Notification Rule: HHS + affected individuals within 60d if PHI.
- KR PIPA: KCC within 24h.
- KSA / AE / SG: per jurisdiction's PII breach laws.

DPO leads disclosure timing.

## Step 6 — Recovery

1. After investigation closes:
   - Unblock genuine-bug agreements after fix + replay.
   - Permanently revoke tampered agreements; add to "tainted" list.
   - Rotate compromised keys via OpenBao.
   - Trigger re-handshake with affected partner if partner-side compromise.
2. Run IP-013 reconciler ad-hoc to verify zero remaining divergences.
3. Update threat model + ADR-SVC-CG-* if mitigation changes.

## Audit evidence

- Every action sealed in audit-chain.
- Forensic report retained 10y.
- Disclosure events sealed.

## Cross-references

- threat-model.md §2.1, §3.1, §9.1 (consent forgery)
- audit-chain-divergence-recovery.md
- break-glass.md (alternate use: audit-officer break-glass review)
