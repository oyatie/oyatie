---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0251]
companion_docs: [microservices/notes/policy/phi-hipaa-notes.cedar]
inbound_citations: [microservices/notes/ARCHITECTURE.md]
---

# Runbook: Clinical note leak recovery (HIPAA breach)

## A. Trigger conditions

- PHI classifier verdict + clinical-note shared externally OR exported without PHI_OFFICER role.
- Auditor escalation per HHS Breach Notification Rule.

## B. Pre-checks

1. Operator Cedar permit `oya.notes.phi-incident-respond` + PHI_OFFICER role.
2. Confirm tenant `audience_type=B2B_HIPAA_CLINICAL` + BAA on file.

## C. Procedure

1. **Containment.** Freeze affected workspace; revoke active share-links touching the note via `runbooks/notes-share-link-revocation.md`. Emits `oya.notes.workspace-freeze`. Timing ≤60s.
2. **Enumerate exposure.** Audit chain query for `oya.notes.share-link-redeem` + `oya.notes.export-complete` events involving the note ID.
3. **Legal hold.** Engage via governance; TrueTime timestamp chain-of-custody.
4. **HHS notification (≥500 records → 60d to HHS+media; <500 → 60d to individuals).** Emit `oya.notes.phi-breach-notify`.
5. **Tenant BAA addendum.** Compliance officer reviews.
6. **Forensics.** Pull all SPIFFE-attested access logs.
7. **Credential rotation.** Per `runbooks/e2e-key-rotation-and-recovery.md`.
8. **Cedar gap.** If `policy/phi-hipaa-notes.cedar` had a gap, add rule + soak 60s.
9. **Postmortem within 72h.**

## D. Verification

- No further leak events.
- HHS notification submitted if applicable.
- Workspace-freeze active until cleared.

## E. Rollback

Unfreeze after forensics + BAA addendum signed.

## F. Post-incident

ADR amendment if doctrinal gap.

## G. References

- `policy/phi-hipaa-notes.cedar`
- `runbooks/notes-share-link-revocation.md`
- 45 CFR §164.408
