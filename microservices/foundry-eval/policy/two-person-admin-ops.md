---
doc_class: Policy
title: Two-Person Admin Operations Policy (foundry-eval)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry
deciders: ops-security, axis-foundry, council-architecture, council-privacy
related_adrs: [ADR-0024, ADR-0131]
related_artifacts:
  - microservices/foundry-eval/threat-model.md
  - microservices/foundry-eval/runbooks/golden-output-restore.md
  - microservices/foundry-eval/runbooks/replay-divergence-investigation.md
review_cadence: quarterly + on every Sev-1 admin-op incident
doc_status: published
---

# Two-Person Admin Operations Policy (foundry-eval µservice)

## Purpose

Enforce 2-person rule for any administrative operation in foundry-eval that could compromise evidence integrity, mass-shred DEKs, bypass adversarial-cohort gating, or revert in-house cutover. Aligns with SOC 2 CC6.6, ISO 27001 A.5.4, HIPAA §164.308(a)(3)(ii)(C), and KR-ISMS-P §2.5.

## Operations Requiring 2-Person Rule

| Operation | Why 2-person | Approval flow |
|---|---|---|
| Mass DEK shred (> 1 subject in 24h window) | Bulk-erasure beyond DSR cascade | Operator A initiates via OpenBao JIT; Operator B approves via OpenBao + audit-chain co-sign |
| Sealed partition seal-break (ClickHouse parity_analytics) | Tampering risk | OpenBao JIT + audit-chain co-sign + 30d soft-window |
| Adversarial-cohort gate manual override (`oya admin capability publish --override-adversarial`) | Bypass of structural integrity gate | OpenBao JIT + audit-chain co-sign + RFC text required |
| In-house cutover manual reverse (`oya admin route reverse-cutover --skip-parity`) | Bypass of parity verdict | OpenBao JIT + audit-chain co-sign |
| Model-upgrade replay-skip (`oya admin model upgrade --skip-replay`) | Bypass of replay determinism gate | OpenBao JIT + audit-chain co-sign |
| KMS KEK rotation outside cadence | Mass-impact change | OpenBao JIT + audit-chain co-sign |
| Postgres eval-set schema migration in production | Schema-integrity risk | OpenBao JIT + PR review + audit-chain |
| Cross-pack data-routing override | Residency breach risk | OpenBao JIT + council-privacy approval + audit-chain |

## Mechanism

1. Operator A authenticates via OIDC + MFA; requests JIT elevation via OpenBao for the operation.
2. OpenBao pauses the elevation pending second-operator approval.
3. Operator B authenticates via OIDC + MFA; reviews the request payload (operation type, scope, RFC text); approves or denies.
4. On approval, OpenBao issues a short-lived (≤ 1h) elevation token bound to both operators' identities.
5. Operation executes; audit-chain emission carries both operator identities + RFC text.
6. Operator A + B independently review the audit-chain entry within 24h.

## Exceptions

- Sev-1 incident response may bypass 2-person rule for ≤ 1h with auto-emission of the bypass record to council-architecture + ops-security; post-incident review required within 5 business days.
- Automated rollback (e.g., automated reverse-cutover on parity-regression detection) is allowed without 2-person rule, because the operation reverts toward a known-good state and is itself audit-chained.

## Verification

- OpenBao JIT log: every admin op carries dual-operator binding.
- Audit-chain regression test: every 2-person op emits dual identity.
- Quarterly review: sample 10 admin ops; verify dual-binding + RFC text.
- Pen-test: attempt single-operator override; should be refused at OpenBao + audit-chain layers.

## Incident Response

Single-operator admin-op detection = Sev-1; immediate ops-security investigation. Possible insider compromise; rotate involved operators' credentials + JIT entitlements.

## References

- SOC 2 CC6.6.
- ISO 27001 A.5.4 (segregation of duties).
- HIPAA §164.308(a)(3)(ii)(C) (workforce clearance procedure).
- KR-ISMS-P §2.5 (인적보안).
- ADR-0024 (eval-set authoring; gating).
- threat-model.md (T-E-05; T-S-04; T-A-03 mitigations).
